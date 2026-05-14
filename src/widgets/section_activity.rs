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

const HEIGHT_CHARTS: u16 = 11;

pub struct SectionActivity {
    wdg_title: WidgetSectionTitle,
    wdg_table_activity: WidgetSimpleTable,
    wdg_chart_transactions: WidgetChart,
    wdg_chart_rollbacks: WidgetChart,
    wdg_chart_locks: WidgetChart,
    wdg_chart_conflicts: WidgetChart,
}

impl SectionActivity {
    pub fn new(history_size: usize) -> Self {
        Self {
            wdg_title: WidgetSectionTitle::new("Activity"),
            wdg_table_activity: WidgetSimpleTable::new(
                "Activity by database",
                QUERIES_DASHBOARD
                    .get(&DashboardKey::DatabaseActivity)
                    .unwrap()
                    .columns
                    .clone(),
            ),
            wdg_chart_transactions: WidgetChart::new("Transactions/s", history_size, HEIGHT_CHARTS),
            wdg_chart_rollbacks: WidgetChart::new("Rollbacks/s", history_size, HEIGHT_CHARTS),
            wdg_chart_locks: WidgetChart::new("Locks", history_size, HEIGHT_CHARTS),
            wdg_chart_conflicts: WidgetChart::new(
                "Conflicts & deadlocks/s",
                history_size,
                HEIGHT_CHARTS,
            ),
        }
    }

    pub fn update_data(
        &mut self,
        db_activity: Vec<Row>,
        transactions_data: Vec<(f64, f64)>,
        transactions_current: Option<f64>,
        rollbacks_data: Vec<(f64, f64)>,
        rollbacks_current: Option<f64>,
        locks_data: Vec<(f64, f64)>,
        locks_current: Option<f64>,
        conflicts_data: Vec<(f64, f64)>,
        conflicts_current: Option<f64>,
    ) {
        self.wdg_table_activity.update_data(db_activity, None);

        let transactions_max = transactions_data
            .iter()
            .map(|&(_, v)| v)
            .fold(0.0_f64, f64::max)
            .max(1.0);
        self.wdg_chart_transactions.update_data(
            transactions_data,
            transactions_current.map(|v| format!(" [{}] ", format_unit(v, ValueUnit::Count))),
            transactions_max,
            ValueUnit::Count,
        );

        let rollbacks_max = rollbacks_data
            .iter()
            .map(|&(_, v)| v)
            .fold(0.0_f64, f64::max)
            .max(1.0);
        self.wdg_chart_rollbacks.update_data(
            rollbacks_data,
            rollbacks_current.map(|v| format!(" [{}] ", format_unit(v, ValueUnit::Count))),
            rollbacks_max,
            ValueUnit::Count,
        );

        let locks_max = locks_data
            .iter()
            .map(|&(_, v)| v)
            .fold(0.0_f64, f64::max)
            .max(1.0);
        self.wdg_chart_locks.update_data(
            locks_data,
            locks_current.map(|v| format!(" [{}] ", format_unit(v, ValueUnit::Count))),
            locks_max,
            ValueUnit::Count,
        );

        let conflicts_max = conflicts_data
            .iter()
            .map(|&(_, v)| v)
            .fold(0.0_f64, f64::max)
            .max(1.0);
        self.wdg_chart_conflicts.update_data(
            conflicts_data,
            conflicts_current.map(|v| format!(" [{}] ", format_unit(v, ValueUnit::Count))),
            conflicts_max,
            ValueUnit::Count,
        );
    }

    pub fn get_height(&self) -> u16 {
        self.wdg_title.get_height() + self.wdg_table_activity.get_height() + 2 * HEIGHT_CHARTS
    }
}

impl Widget for &SectionActivity {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Create vertical layout for the section title, table, and charts
        let [section1_area, section2_area, section3_area, section4_area] = Layout::vertical([
            Constraint::Length(self.wdg_title.get_height()),
            Constraint::Length(self.wdg_table_activity.get_height()),
            Constraint::Length(HEIGHT_CHARTS),
            Constraint::Length(HEIGHT_CHARTS),
        ])
        .areas(area);

        // Create horizontal layout for the charts in section 3
        let [section3_left_area, _, section3_right_area] = Layout::horizontal([
            Constraint::Min(0),
            Constraint::Length(1),
            Constraint::Min(0),
        ])
        .areas(section3_area);

        // Create horizontal layout for the charts in section 4
        let [section4_left_area, _, section4_right_area] = Layout::horizontal([
            Constraint::Min(0),
            Constraint::Length(1),
            Constraint::Min(0),
        ])
        .areas(section4_area);

        // Render widgets
        (&self.wdg_title).render(section1_area, buf);
        (&self.wdg_table_activity).render(section2_area, buf);

        (&self.wdg_chart_transactions).render(section3_left_area, buf);
        (&self.wdg_chart_rollbacks).render(section3_right_area, buf);

        (&self.wdg_chart_locks).render(section4_left_area, buf);
        (&self.wdg_chart_conflicts).render(section4_right_area, buf);
    }
}
