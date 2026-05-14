use crate::palette as P;
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::Style,
    symbols,
    text::Line,
    widgets::{Axis, Block, Chart, Dataset, GraphType, Widget},
};

#[derive(Copy, Clone)]
pub enum ValueUnit {
    Count,
    Percentage,
}

pub fn format_unit(value: f64, unit: ValueUnit) -> String {
    match unit {
        ValueUnit::Count => {
            if value >= 1_000_000.0 {
                format!("{:.1}M", value / 1_000_000.0)
            } else if value >= 1_000.0 {
                format!("{:.1}K", value / 1_000.0)
            } else {
                format!("{:.0}", value)
            }
        }
        ValueUnit::Percentage => format!("{:.2}%", value),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- Count ---

    #[test]
    fn count_zero() {
        assert_eq!(format_unit(0.0, ValueUnit::Count), "0");
    }

    #[test]
    fn count_below_1000_no_suffix() {
        assert_eq!(format_unit(1.0, ValueUnit::Count), "1");
        assert_eq!(format_unit(999.0, ValueUnit::Count), "999");
    }

    #[test]
    fn count_thousands_uses_k_suffix() {
        assert_eq!(format_unit(1_000.0, ValueUnit::Count), "1.0K");
        assert_eq!(format_unit(1_500.0, ValueUnit::Count), "1.5K");
    }

    #[test]
    fn count_millions_uses_m_suffix() {
        assert_eq!(format_unit(1_000_000.0, ValueUnit::Count), "1.0M");
        assert_eq!(format_unit(2_500_000.0, ValueUnit::Count), "2.5M");
    }

    #[test]
    fn count_negative_below_threshold_no_suffix() {
        assert_eq!(format_unit(-5.0, ValueUnit::Count), "-5");
    }

    // --- Percentage ---

    #[test]
    fn percentage_zero() {
        assert_eq!(format_unit(0.0, ValueUnit::Percentage), "0.00%");
    }

    #[test]
    fn percentage_hundred() {
        assert_eq!(format_unit(100.0, ValueUnit::Percentage), "100.00%");
    }

    #[test]
    fn percentage_rounds_to_two_decimals() {
        assert_eq!(format_unit(33.333, ValueUnit::Percentage), "33.33%");
        assert_eq!(format_unit(66.666, ValueUnit::Percentage), "66.67%");
    }
}

pub struct WidgetChart {
    title: String,
    right_title: Option<String>,
    data: Vec<(f64, f64)>,
    history_size: usize,
    y_bounds: [f64; 2],
    y_labels: [String; 3],
    height: u16,
}

impl WidgetChart {
    pub fn new(title: impl Into<String>, history_size: usize, height: u16) -> Self {
        Self {
            title: format!(" {} ", title.into()),
            right_title: None,
            data: Vec::new(),
            history_size,
            y_bounds: [0.0, 1.0],
            y_labels: ["    0".into(), "  0.5".into(), "    1".into()],
            height,
        }
    }

    pub fn get_height(&self) -> u16 {
        self.height
    }

    pub fn update_data(
        &mut self,
        data: Vec<(f64, f64)>,
        right_title: Option<String>,
        max_value: f64,
        unit: ValueUnit,
    ) {
        self.data = data;
        self.right_title = right_title;
        let (y_bounds, y_labels) = match unit {
            ValueUnit::Percentage => (
                [0.0, 100.0],
                ["    0".into(), "   50".into(), "  100".into()],
            ),
            _ => (
                [0.0, max_value],
                [
                    format!("{:>5}", "0"),
                    format!("{:>5}", format_unit(max_value / 2.0, unit)),
                    format!("{:>5}", format_unit(max_value, unit)),
                ],
            ),
        };
        self.y_bounds = y_bounds;
        self.y_labels = y_labels;
    }
}

impl Widget for &WidgetChart {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        Chart::new(vec![
            Dataset::default()
                .data(&self.data)
                .marker(symbols::Marker::Braille)
                .graph_type(GraphType::Line)
                .style(Style::default().fg(P::APP_COLOR2)),
        ])
        .style(Style::default().bg(P::APP_BKG))
        .block({
            let mut block = Block::bordered()
                .title(self.title.clone())
                .title_style(P::WIDGET_TITLE_ACTIVE)
                .border_set(symbols::border::PLAIN)
                .style(Style::default().fg(P::APP_COLOR1).bg(P::APP_BKG))
                .border_style(P::WIDGET_BORDER_ACTIVE);
            if let Some(ref right_title) = self.right_title {
                block =
                    block.title_top(Line::from(right_title.clone()).alignment(Alignment::Right));
            }
            block
        })
        .x_axis(
            Axis::default()
                .bounds([0.0, (self.history_size - 1) as f64])
                .style(Style::default().fg(P::APP_COLOR1)),
        )
        .y_axis(
            Axis::default()
                .bounds(self.y_bounds)
                .labels(self.y_labels.clone()),
        )
        .render(area, buffer);
    }
}
