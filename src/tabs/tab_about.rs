use crate::{VERSION, palette as P, tabs::tab::Tab};
use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Stylize},
    symbols,
    text::{Line, Span},
    widgets::{Block, Padding, Paragraph},
};

pub struct TabAbout {}

impl TabAbout {
    pub fn new() -> Self {
        Self {}
    }
}

impl Tab for TabAbout {
    fn get_name(&self) -> String {
        String::from("About")
    }

    fn render(&mut self, frame: &mut Frame, area: Rect) {
        let paragraph = Paragraph::new(vec![
            Line::raw(""),
            Line::raw(""),
            Line::raw(""),
            Line::raw(""),
            Line::raw(""),
            Span::raw(format!("Ellie {}", VERSION))
                .fg(P::APP_COLOR4)
                .add_modifier(Modifier::BOLD)
                .into_centered_line(),
            Line::raw(""),
            Line::raw(""),
            Line::raw(""),
            Line::raw("PostgreSQL performance tuning tool"),
            Line::raw(""),
            Line::raw("Developed in Rust language"),
            Line::raw(""),
            Line::raw("by Luiz Ferreira"),
            Line::raw(""),
            Span::raw("https://github.com/luizferreira-io/ellie")
                .fg(P::APP_COLOR2)
                .into_centered_line(),
            Line::raw(""),
            Line::raw(""),
            Line::raw(""),
            Line::raw("The name \"Ellie\" comes from the female"),
            Line::raw("mammoth character in the Ice Age movies."),
        ])
        .centered()
        .block(
            Block::bordered()
                .title(" About ")
                .title_style(P::WIDGET_TITLE_ACTIVE)
                .border_set(symbols::border::PLAIN)
                .padding(Padding::horizontal(0))
                .style(
                    ratatui::style::Style::new()
                        .fg(P::APP_COLOR1)
                        .bg(P::APP_BKG),
                )
                .border_style(P::WIDGET_BORDER_ACTIVE),
        );
        frame.render_widget(paragraph, area);
    }
}
