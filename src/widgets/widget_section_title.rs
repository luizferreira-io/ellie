use crate::palette as P;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Stylize},
    text::{Line, Span},
    widgets::{Paragraph, Widget},
};

pub struct WidgetSectionTitle {
    title: &'static str,
}

impl WidgetSectionTitle {
    pub fn new(title: &'static str) -> Self {
        Self { title }
    }

    pub fn get_height(&self) -> u16 {
        2 // title + separator
    }
}

impl Widget for &WidgetSectionTitle {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        Paragraph::new(vec![
            Line::from(
                Span::raw(self.title)
                    .fg(P::APP_COLOR3)
                    .add_modifier(Modifier::BOLD),
            ),
            Line::from(Span::raw("─".repeat(area.width as usize)).fg(P::APP_COLOR1)),
        ])
        .render(area, buffer);
    }
}
