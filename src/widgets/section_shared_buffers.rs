use crate::database::{DashboardKey, QUERIES_DASHBOARD};
use crate::widgets::widget_chart::{ValueUnit, WidgetChart, format_unit};
use crate::widgets::widget_section_title::WidgetSectionTitle;
use crate::widgets::widget_simple_table::WidgetSimpleTable;
use postgres::Row;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    widgets::Widget,
};

const HEIGHT_CHART: u16 = 11;

pub struct SectionSharedBuffers {
    wdg_section_title: WidgetSectionTitle,
    wdg_chart_cache_hit: WidgetChart,
    wdg_table_cache_hit: WidgetSimpleTable,
    wdg_table_content: WidgetSimpleTable,
}

impl SectionSharedBuffers {
    pub fn new(history_size: usize) -> Self {
        Self {
            wdg_section_title: WidgetSectionTitle::new("Shared Buffers"),
            wdg_chart_cache_hit: WidgetChart::new("Cache hit ratio", history_size, HEIGHT_CHART),
            wdg_table_cache_hit: WidgetSimpleTable::new(
                "Cache hit by database",
                QUERIES_DASHBOARD
                    .get(&DashboardKey::CacheHitByDatabase)
                    .unwrap()
                    .columns
                    .clone(),
            ),
            wdg_table_content: WidgetSimpleTable::new(
                "Shared buffers content",
                QUERIES_DASHBOARD
                    .get(&DashboardKey::SharedBuffersContentTop10)
                    .unwrap()
                    .columns
                    .clone(),
            ),
        }
    }

    pub fn update_data(
        &mut self,
        cache_hit_data: Vec<(f64, f64)>,
        cache_hit_current: Option<f64>,
        cache_hit_by_db: Vec<Row>,
        shared_buffers_content: Vec<Row>,
        shared_buffers_error: Option<String>,
    ) {
        let right_title = cache_hit_current
            .map(|value| format!(" [{}] ", format_unit(value, ValueUnit::Percentage)));
        self.wdg_chart_cache_hit.update_data(
            cache_hit_data,
            right_title,
            100.0,
            ValueUnit::Percentage,
        );
        self.wdg_table_cache_hit.update_data(cache_hit_by_db, None);
        self.wdg_table_content
            .update_data(shared_buffers_content, shared_buffers_error);
    }

    pub fn get_height(&self) -> u16 {
        self.wdg_section_title.get_height()
            + self.wdg_chart_cache_hit.get_height()
            + self
                .wdg_table_cache_hit
                .get_height()
                .max(self.wdg_table_content.get_height())
    }
}

impl Widget for &SectionSharedBuffers {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        // Create vertical layout for the section title, chart, and tables
        let [section1_area, section2_area, section3_area] = Layout::vertical([
            Constraint::Length(self.wdg_section_title.get_height()),
            Constraint::Length(self.wdg_chart_cache_hit.get_height()),
            Constraint::Length(
                self.wdg_table_cache_hit
                    .get_height()
                    .max(self.wdg_table_content.get_height()),
            ),
        ])
        .areas(area);

        // Create horizontal layout for the two tables in section 3
        let [section3_left_area, _, section3_right_area] = Layout::horizontal([
            Constraint::Min(0),
            Constraint::Length(1),
            Constraint::Min(0),
        ])
        .areas(section3_area);

        // Render widgets
        (&self.wdg_section_title).render(section1_area, buffer);
        (&self.wdg_chart_cache_hit).render(section2_area, buffer);
        (&self.wdg_table_cache_hit).render(section3_left_area, buffer);
        (&self.wdg_table_content).render(section3_right_area, buffer);
    }
}
