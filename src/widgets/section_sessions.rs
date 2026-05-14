use crate::database::{DashboardKey, QUERIES_DASHBOARD};
use crate::widgets::widget_chart::{ValueUnit, WidgetChart};
use crate::widgets::widget_section_title::WidgetSectionTitle;
use crate::widgets::widget_simple_table::WidgetSimpleTable;
use postgres::Row;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    widgets::Widget,
};

const HEIGHT_CHART: u16 = 11;

pub struct SectionSessions {
    wdg_title: WidgetSectionTitle,
    wdg_chart_sessions: WidgetChart,
    wdg_table_sessions_by_db: WidgetSimpleTable,
    wdg_table_sessions_by_app: WidgetSimpleTable,
}

impl SectionSessions {
    pub fn new(history_size: usize) -> Self {
        Self {
            wdg_title: WidgetSectionTitle::new("Sessions"),
            wdg_chart_sessions: WidgetChart::new(
                "Sessions on server instance",
                history_size,
                HEIGHT_CHART,
            ),
            wdg_table_sessions_by_db: WidgetSimpleTable::new(
                "Sessions by database",
                QUERIES_DASHBOARD
                    .get(&DashboardKey::SessionsByDatabase)
                    .unwrap()
                    .columns
                    .clone(),
            ),
            wdg_table_sessions_by_app: WidgetSimpleTable::new(
                "Sessions by application",
                QUERIES_DASHBOARD
                    .get(&DashboardKey::SessionsByApplication)
                    .unwrap()
                    .columns
                    .clone(),
            ),
        }
    }

    pub fn update_data(
        &mut self,
        chart_data: Vec<(f64, f64)>,
        chart_current: Option<f64>,
        max_connections: i64,
        table_left: Vec<Row>,
        table_right: Vec<Row>,
        table_right_error: Option<String>,
    ) {
        let right_title = Some(format!(
            " [{}/{}] ",
            chart_current.map_or(0, |v| v as i64),
            max_connections,
        ));
        self.wdg_chart_sessions.update_data(
            chart_data,
            right_title,
            max_connections as f64,
            ValueUnit::Count,
        );
        self.wdg_table_sessions_by_db.update_data(table_left, None);
        self.wdg_table_sessions_by_app
            .update_data(table_right, table_right_error);
    }

    pub fn get_height(&self) -> u16 {
        self.wdg_title.get_height()
            + self.wdg_chart_sessions.get_height()
            + self
                .wdg_table_sessions_by_db
                .get_height()
                .max(self.wdg_table_sessions_by_app.get_height())
    }
}

impl Widget for &SectionSessions {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        // Create layout for the section: title, chart, and tables
        let [section1_area, section2_area, section3_area] = Layout::vertical([
            Constraint::Length(self.wdg_title.get_height()),
            Constraint::Length(self.wdg_chart_sessions.get_height()),
            Constraint::Length(
                self.wdg_table_sessions_by_db
                    .get_height()
                    .max(self.wdg_table_sessions_by_app.get_height()),
            ),
        ])
        .areas(area);

        // Create horizontal layout for the two tables
        let [section3_left_area, _, section3_right_area] = Layout::horizontal([
            Constraint::Min(0),
            Constraint::Length(1),
            Constraint::Min(0),
        ])
        .areas(section3_area);

        // Render widgets
        (&self.wdg_title).render(section1_area, buffer);
        (&self.wdg_chart_sessions).render(section2_area, buffer);
        (&self.wdg_table_sessions_by_db).render(section3_left_area, buffer);
        (&self.wdg_table_sessions_by_app).render(section3_right_area, buffer);
    }
}
