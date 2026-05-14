use crate::palette as P;
use ratatui::prelude::*;
use ratatui::style::Style;
use ratatui::widgets::{Block, Paragraph, Wrap};

pub struct WidgetTextBox {
    pub title: String,
    pub style: Style,
    text: String,
}

impl WidgetTextBox {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: format!(" {} ", title.into()),
            style: Style::new().fg(P::APP_COLOR2).bg(P::APP_BKG),
            text: String::new(),
        }
    }
    pub fn refresh_data(&mut self, text: impl Into<String>) {
        self.text = text.into();
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let paragraph = Paragraph::new(self.text.as_str())
            .block(
                Block::bordered()
                    .title(self.title.as_str())
                    .title_style(P::WIDGET_TITLE_ACTIVE)
                    .border_style(P::WIDGET_BORDER_ACTIVE)
                    .style(self.style),
            )
            .wrap(Wrap { trim: false });
        frame.render_widget(paragraph, area);
    }
}
