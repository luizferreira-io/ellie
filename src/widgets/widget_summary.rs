use crate::palette as P;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    prelude::Stylize,
    style::Modifier,
    text::{Line, Span},
    widgets::{Paragraph, Widget},
};

pub struct WidgetSummary {
    title: &'static str,
    label_width: usize,
    rows: Vec<(String, String)>,
}

impl WidgetSummary {
    pub fn new(title: &'static str, label_width: usize) -> Self {
        Self {
            title,
            label_width,
            rows: Vec::new(),
        }
    }

    pub fn update_data(
        &mut self,
        rows: impl IntoIterator<Item = (impl Into<String>, impl Into<String>)>,
    ) {
        self.rows = rows
            .into_iter()
            .map(|(label, value)| (label.into(), value.into()))
            .collect();
    }

    pub fn get_height(&self) -> u16 {
        (2 + self.rows.len()) as u16 // title + separator + rows
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_height_empty_is_two() {
        let w = WidgetSummary::new("Title", 10);
        assert_eq!(w.get_height(), 2); // título + separador, sem linhas
    }

    #[test]
    fn get_height_counts_rows() {
        let mut w = WidgetSummary::new("Title", 10);
        w.update_data([("a", "1"), ("b", "2"), ("c", "3")]);
        assert_eq!(w.get_height(), 5); // 2 + 3 linhas
    }

    #[test]
    fn update_data_replaces_previous_rows() {
        let mut w = WidgetSummary::new("Title", 10);
        w.update_data([("a", "1"), ("b", "2")]);
        w.update_data([("c", "3")]);
        assert_eq!(w.get_height(), 3); // 2 + 1 linha
    }

    #[test]
    fn update_data_accepts_owned_strings() {
        let mut w = WidgetSummary::new("Title", 10);
        w.update_data([(String::from("chave"), String::from("valor"))]);
        assert_eq!(w.get_height(), 3);
    }

    #[test]
    fn update_data_empty_iterator() {
        let mut w = WidgetSummary::new("Title", 10);
        w.update_data([("a", "1")]);
        w.update_data(std::iter::empty::<(&str, &str)>());
        assert_eq!(w.get_height(), 2);
    }
}

impl Widget for &WidgetSummary {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Render title and separator line
        let mut lines = vec![
            Line::from(
                Span::raw(self.title)
                    .fg(P::APP_COLOR3)
                    .add_modifier(Modifier::BOLD),
            ),
            Line::from(Span::raw("─".repeat(area.width as usize)).fg(P::APP_COLOR1)),
        ];

        // Render rows
        for (label, value) in &self.rows {
            if label.is_empty() || self.label_width == 0 {
                lines.push(Line::from(Span::raw(value.clone()).fg(P::APP_COLOR1)));
            } else {
                lines.push(Line::from(vec![
                    Span::raw(format!("{:<w$}  ", label, w = self.label_width)).fg(P::APP_COLOR2),
                    Span::raw(value.clone()).fg(P::APP_COLOR1),
                ]));
            }
        }
        Paragraph::new(lines).render(area, buf);
    }
}
