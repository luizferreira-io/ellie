use super::metric_history::MetricHistory;
use crate::database::{DashboardKey, QUERIES_DASHBOARD, db_query, get_str};
use crate::widgets::section_activity::SectionActivity;
use crate::widgets::section_instance::{SectionInstance, ServerInstanceData};
use crate::widgets::section_sessions::SectionSessions;
use crate::widgets::section_shared_buffers::SectionSharedBuffers;
use crate::{palette as P, tabs::tab::Tab};
use postgres::Client;
use ratatui::{
    Frame,
    crossterm::event::KeyCode,
    layout::{Constraint, Layout, Rect, Size},
    style::{Style, Stylize},
    symbols,
    text::{Line, Span},
    widgets::{Block, Padding, Scrollbar, ScrollbarOrientation, ScrollbarState},
};
use std::time::{Duration, Instant};
use tui_scrollview::{ScrollView, ScrollViewState, ScrollbarVisibility};

const HEIGHT_SEPARATOR: u16 = 1;
const HISTORY_SIZE: usize = 120;
const REFRESH_INTERVAL: Duration = Duration::from_secs(5);

pub struct TabDashboard {
    section_instance: SectionInstance,
    section_shared_buffers: SectionSharedBuffers,
    section_sessions: SectionSessions,
    section_activity: SectionActivity,

    metric_cache_hit: MetricHistory,
    metric_sessions: MetricHistory,
    metric_transactions: MetricHistory,
    metric_rollbacks: MetricHistory,
    metric_locks: MetricHistory,
    metric_conflicts: MetricHistory,

    scroll_state: ScrollViewState,
    last_refresh: Option<Instant>,
}

impl TabDashboard {
    pub fn new() -> Self {
        Self {
            section_instance: SectionInstance::new(),
            section_shared_buffers: SectionSharedBuffers::new(HISTORY_SIZE),
            section_sessions: SectionSessions::new(HISTORY_SIZE),
            section_activity: SectionActivity::new(HISTORY_SIZE),

            metric_cache_hit: MetricHistory::new(HISTORY_SIZE, REFRESH_INTERVAL),
            metric_sessions: MetricHistory::new(HISTORY_SIZE, REFRESH_INTERVAL),
            metric_transactions: MetricHistory::new(HISTORY_SIZE, REFRESH_INTERVAL),
            metric_rollbacks: MetricHistory::new(HISTORY_SIZE, REFRESH_INTERVAL),
            metric_locks: MetricHistory::new(HISTORY_SIZE, REFRESH_INTERVAL),
            metric_conflicts: MetricHistory::new(HISTORY_SIZE, REFRESH_INTERVAL),

            scroll_state: ScrollViewState::new(),
            last_refresh: None,
        }
    }
}

impl Tab for TabDashboard {
    fn get_name(&self) -> String {
        String::from("Dashboard")
    }

    fn needs_refresh(&self) -> bool {
        self.last_refresh
            .is_some_and(|t| t.elapsed() >= REFRESH_INTERVAL)
    }

    fn update_data(&mut self, client: &mut Client) -> Result<(), Box<dyn std::error::Error>> {
        // Update instance data
        let data_instance = db_query(
            client,
            QUERIES_DASHBOARD
                .get(&DashboardKey::Instance)
                .unwrap()
                .query,
        )?;
        let (version, uptime_secs, start_time) = data_instance
            .first()
            .map(|row| {
                let version = get_str(row, "version");
                let start_time = get_str(row, "start_time");
                let uptime_secs: i64 = get_str(row, "uptime_secs").parse().unwrap_or(0);
                (version, uptime_secs, start_time)
            })
            .unwrap_or_default();

        // Update settings data
        let data_settings = db_query(
            client,
            QUERIES_DASHBOARD
                .get(&DashboardKey::Settings)
                .unwrap()
                .query,
        )?;
        let mut map = std::collections::HashMap::new();
        for row in &data_settings {
            map.insert(get_str(row, "name"), get_str(row, "value"));
        }
        let get = |key: &str| map.get(key).cloned().unwrap_or_else(|| String::from("—"));

        // Update extensions data
        let data_extensions = db_query(
            client,
            QUERIES_DASHBOARD
                .get(&DashboardKey::Extensions)
                .unwrap()
                .query,
        )?;
        let extensions = data_extensions
            .iter()
            .map(|row| get_str(row, "name_extension"))
            .collect();

        // Update cache hit by database table
        let data_cache_hit_by_db = db_query(
            client,
            QUERIES_DASHBOARD
                .get(&DashboardKey::CacheHitByDatabase)
                .unwrap()
                .query,
        )?;

        // Update shared buffers content table
        let (data_shared_buffers_content, shared_buffers_error) = match db_query(
            client,
            QUERIES_DASHBOARD
                .get(&DashboardKey::SharedBuffersContentTop10)
                .unwrap()
                .query,
        ) {
            Ok(rows) => (rows, None),
            Err(_) => (
                Vec::new(),
                Some("Enable pg_buffercache extension\nto view shared buffers content.".to_owned()),
            ),
        };

        // Update sessions by database table
        let data_sessions_by_db = db_query(
            client,
            QUERIES_DASHBOARD
                .get(&DashboardKey::SessionsByDatabase)
                .unwrap()
                .query,
        )?;

        // Update sessions by application table
        let data_sessions_by_app = db_query(
            client,
            QUERIES_DASHBOARD
                .get(&DashboardKey::SessionsByApplication)
                .unwrap()
                .query,
        )?;

        // Update database activity table
        let data_activity_by_db = db_query(
            client,
            QUERIES_DASHBOARD
                .get(&DashboardKey::ActivityByDatabase)
                .unwrap()
                .query,
        )?;

        // Update sessions metrics
        let sessions_rows = db_query(
            client,
            QUERIES_DASHBOARD
                .get(&DashboardKey::SessionsMetrics)
                .unwrap()
                .query,
        )?;
        let (total_sessions, max_connections) = sessions_rows.first().map_or((0, 100), |row| {
            let total: i64 = get_str(row, "total_sessions").parse().unwrap_or(0);
            let max: i64 = get_str(row, "max_connections").parse().unwrap_or(100);
            (total, max)
        });
        self.metric_sessions.push_absolute(total_sessions);

        // Update metrics
        let metrics_rows = db_query(
            client,
            QUERIES_DASHBOARD.get(&DashboardKey::Metrics).unwrap().query,
        )?;
        if let Some(row) = metrics_rows.first() {
            self.metric_transactions
                .push_delta(get_str(row, "transactions").parse::<i64>().unwrap_or(0));
            self.metric_locks
                .push_absolute(get_str(row, "locks").parse::<i64>().unwrap_or(0));
            self.metric_conflicts
                .push_delta(get_str(row, "conflicts").parse::<i64>().unwrap_or(0));
            self.metric_rollbacks
                .push_delta(get_str(row, "rollbacks").parse::<i64>().unwrap_or(0));
            let blocks_hit: i64 = get_str(row, "blocks_hit").parse().unwrap_or(0);
            let blocks_read: i64 = get_str(row, "blocks_read").parse().unwrap_or(0);
            let total = blocks_hit + blocks_read;
            let ratio = if total > 0 {
                100.0 * blocks_hit as f64 / total as f64
            } else {
                0.0
            };
            self.metric_cache_hit.push_value(ratio);
        }

        // Update section data

        self.section_instance.update_data(ServerInstanceData {
            version,
            uptime_secs,
            start_time,
            shared_buffers: get("shared_buffers"),
            effective_cache_size: get("effective_cache_size"),
            maintenance_work_memory: get("maintenance_work_mem"),
            work_memory: get("work_mem"),
            max_wal_size: get("max_wal_size"),
            max_worker_processes: get("max_worker_processes"),
            max_parallel_workers: get("max_parallel_workers"),
            extensions,
        });

        self.section_shared_buffers.update_data(
            self.metric_cache_hit.chart_data(),
            self.metric_cache_hit.current_value(),
            data_cache_hit_by_db,
            data_shared_buffers_content,
            shared_buffers_error,
        );

        self.section_sessions.update_data(
            self.metric_sessions.chart_data(),
            self.metric_sessions.current_value(),
            max_connections,
            data_sessions_by_db,
            data_sessions_by_app,
            None,
        );

        self.section_activity.update_data(
            data_activity_by_db,
            self.metric_transactions.chart_data(),
            self.metric_transactions.current_value(),
            self.metric_rollbacks.chart_data(),
            self.metric_rollbacks.current_value(),
            self.metric_locks.chart_data(),
            self.metric_locks.current_value(),
            self.metric_conflicts.chart_data(),
            self.metric_conflicts.current_value(),
        );

        self.last_refresh = Some(Instant::now());
        Ok(())
    }

    fn handle_event(&mut self, keycode: KeyCode) {
        match keycode {
            KeyCode::Up => self.scroll_state.scroll_up(),
            KeyCode::Down => self.scroll_state.scroll_down(),
            KeyCode::PageUp => self.scroll_state.scroll_page_up(),
            KeyCode::PageDown => self.scroll_state.scroll_page_down(),
            KeyCode::Home => self.scroll_state.scroll_to_top(),
            KeyCode::End => self.scroll_state.scroll_to_bottom(),
            _ => {}
        }
    }

    fn render(&mut self, frame: &mut Frame, area: Rect) {
        // Render the main block with title and border
        let block = Block::bordered()
            .title(" Dashboard ")
            .title_style(P::WIDGET_TITLE_ACTIVE)
            .border_set(symbols::border::PLAIN)
            .padding(Padding::horizontal(1))
            .style((P::APP_COLOR1, P::APP_BKG))
            .border_style(P::WIDGET_BORDER_ACTIVE);
        frame.render_widget(&block, area);
        let viewport = block.inner(area);

        // Calculate total content height based on sections and separators
        let height_server_instance = self.section_instance.get_height();
        let height_shared_buffers = self.section_shared_buffers.get_height();
        let height_sessions = self.section_sessions.get_height();
        let height_activity = self.section_activity.get_height();
        let content_height = HEIGHT_SEPARATOR
            + height_server_instance
            + HEIGHT_SEPARATOR
            + height_shared_buffers
            + HEIGHT_SEPARATOR
            + height_sessions
            + HEIGHT_SEPARATOR
            + height_activity;

        // Create a scroll view for the content
        let mut scroll_view = ScrollView::new(Size::new(viewport.width, content_height))
            .horizontal_scrollbar_visibility(ScrollbarVisibility::Never)
            .vertical_scrollbar_visibility(ScrollbarVisibility::Never);
        let virtual_area = Rect::new(0, 0, viewport.width, content_height);
        scroll_view
            .buf_mut()
            .set_style(virtual_area, Style::default().bg(P::APP_BKG));

        // Split the virtual area into sections with separators blank lines
        let [
            _,
            section1_area,
            _,
            section2_area,
            _,
            section3_area,
            _,
            section4_area,
        ] = Layout::vertical([
            Constraint::Length(HEIGHT_SEPARATOR),
            Constraint::Length(height_server_instance),
            Constraint::Length(HEIGHT_SEPARATOR),
            Constraint::Length(height_shared_buffers),
            Constraint::Length(HEIGHT_SEPARATOR),
            Constraint::Length(height_sessions),
            Constraint::Length(HEIGHT_SEPARATOR),
            Constraint::Length(height_activity),
        ])
        .areas(virtual_area);

        // Render each section into its respective area in the scroll view
        scroll_view.render_widget(&self.section_instance, section1_area);
        scroll_view.render_widget(&self.section_shared_buffers, section2_area);
        scroll_view.render_widget(&self.section_sessions, section3_area);
        scroll_view.render_widget(&self.section_activity, section4_area);

        // Render the scroll view
        frame.render_stateful_widget(scroll_view, viewport, &mut self.scroll_state);

        // Render the vertical scrollbar
        let max_scroll = content_height.saturating_sub(viewport.height);
        let mut scrollbar_state = ScrollbarState::new(max_scroll as usize)
            .position(self.scroll_state.offset().y as usize);
        frame.render_stateful_widget(
            Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .begin_symbol(Some("↑"))
                .end_symbol(Some("↓"))
                .track_symbol(Some("░"))
                .thumb_symbol("▓"),
            Rect {
                x: area.right() - 1,
                y: area.y + 1,
                width: 1,
                height: area.height.saturating_sub(2),
            },
            &mut scrollbar_state,
        );
    }

    fn get_footer(&self) -> Line<'static> {
        Line::from_iter([
            Span::raw(" ↑↓").fg(P::APP_FOOTER_SHORTCUT),
            Span::raw(" scroll |"),
        ])
    }
}
