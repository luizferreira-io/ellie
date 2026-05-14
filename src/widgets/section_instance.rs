use crate::widgets::widget_summary::WidgetSummary;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    widgets::Widget,
};

const HEIGHT_SEPARATOR: u16 = 1;

fn format_uptime(secs: i64) -> String {
    let days = secs / 86400;
    let hours = (secs % 86400) / 3600;
    let minutes = (secs % 3600) / 60;
    let mut parts: Vec<String> = Vec::new();
    if days > 0 {
        parts.push(format!("{} day{}", days, if days == 1 { "" } else { "s" }));
    }
    if hours > 0 {
        parts.push(format!("{}h", hours));
    }
    parts.push(format!("{}m", minutes));
    parts.join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uptime_zero_seconds() {
        assert_eq!(format_uptime(0), "0m");
    }

    #[test]
    fn uptime_minutes_only() {
        assert_eq!(format_uptime(5 * 60), "5m");
    }

    #[test]
    fn uptime_one_hour_exact() {
        assert_eq!(format_uptime(3600), "1h 0m");
    }

    #[test]
    fn uptime_hours_and_minutes() {
        assert_eq!(format_uptime(3600 + 45 * 60), "1h 45m");
    }

    #[test]
    fn uptime_one_day_singular() {
        assert_eq!(format_uptime(86400), "1 day 0m");
    }

    #[test]
    fn uptime_multiple_days_plural() {
        assert_eq!(format_uptime(86400 * 3), "3 days 0m");
    }

    #[test]
    fn uptime_full_days_hours_minutes() {
        let secs = 86400 + 2 * 3600 + 30 * 60;
        assert_eq!(format_uptime(secs), "1 day 2h 30m");
    }

    #[test]
    fn uptime_seconds_are_truncated() {
        // 1m 59s deve aparecer como 1m, não 2m
        assert_eq!(format_uptime(60 + 59), "1m");
    }
}

pub struct ServerInstanceData {
    pub version: String,
    pub uptime_secs: i64,
    pub start_time: String,
    pub shared_buffers: String,
    pub effective_cache_size: String,
    pub maintenance_work_memory: String,
    pub work_memory: String,
    pub max_wal_size: String,
    pub max_worker_processes: String,
    pub max_parallel_workers: String,
    pub extensions: Vec<String>,
}

pub struct SectionInstance {
    wdg_instance: WidgetSummary,
    wdg_memory: WidgetSummary,
    wdg_workers: WidgetSummary,
    wdg_extensions: WidgetSummary,
}

impl SectionInstance {
    pub fn new() -> Self {
        Self {
            wdg_instance: WidgetSummary::new("Instance", 8),
            wdg_memory: WidgetSummary::new("Memory", 24),
            wdg_workers: WidgetSummary::new("Workers", 24),
            wdg_extensions: WidgetSummary::new("Extensions", 0),
        }
    }

    pub fn update_data(&mut self, data: ServerInstanceData) {
        let uptime = format!(
            "{} (started at {})",
            format_uptime(data.uptime_secs),
            data.start_time
        );
        self.wdg_instance
            .update_data([("Version", data.version), ("Uptime", uptime)]);
        self.wdg_memory.update_data([
            ("Shared buffers", data.shared_buffers),
            ("Effective cache size", data.effective_cache_size),
            ("Maintenance work memory", data.maintenance_work_memory),
            ("Work memory", data.work_memory),
            ("Maximum WAL size", data.max_wal_size),
        ]);
        self.wdg_workers.update_data([
            ("Maximum worker processes", data.max_worker_processes),
            ("Maximum parallel workers", data.max_parallel_workers),
        ]);
        self.wdg_extensions
            .update_data(data.extensions.into_iter().map(|e| ("", e)));
    }

    pub fn get_height(&self) -> u16 {
        let section1_height = self.wdg_instance.get_height();
        let section2_height = self
            .wdg_memory
            .get_height()
            .max(self.wdg_workers.get_height())
            .max(self.wdg_extensions.get_height());
        section1_height + HEIGHT_SEPARATOR + section2_height
    }
}

impl Widget for &SectionInstance {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        // Create vertical layout for the section title, chart, and tables
        let section1_height = self.wdg_instance.get_height();
        let section2_height = self
            .wdg_memory
            .get_height()
            .max(self.wdg_workers.get_height())
            .max(self.wdg_extensions.get_height());
        let [section1_area, _, section2_area] = Layout::vertical([
            Constraint::Length(section1_height),
            Constraint::Length(HEIGHT_SEPARATOR),
            Constraint::Length(section2_height),
        ])
        .areas(area);

        // Create horizontal layout for section 2 (memory, workers, extensions)
        let [
            section2_left_area,
            _,
            section2_middle_area,
            _,
            section2_right_area,
        ] = Layout::horizontal([
            Constraint::Min(0),
            Constraint::Length(1),
            Constraint::Min(0),
            Constraint::Length(1),
            Constraint::Min(0),
        ])
        .areas(section2_area);

        // Render widgets
        (&self.wdg_instance).render(section1_area, buffer);
        (&self.wdg_memory).render(section2_left_area, buffer);
        (&self.wdg_workers).render(section2_middle_area, buffer);
        (&self.wdg_extensions).render(section2_right_area, buffer);
    }
}
