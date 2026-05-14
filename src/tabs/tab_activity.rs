use crate::database::{ActivityKey, QUERIES_ACTIVITY};
use crate::palette as P;
use crate::tabs::tab::Tab;
use crate::widgets::widget_table::{TableColumn, WidgetTable};
use postgres::Client;
use ratatui::{
    Frame,
    crossterm::event::KeyCode,
    layout::Rect,
    style::Stylize,
    text::{Line, Span},
};

pub struct TabActivity {
    wdg_table: WidgetTable,
}

impl TabActivity {
    pub fn new() -> Self {
        let table_definitions = QUERIES_ACTIVITY.get(&ActivityKey::Activity).unwrap();
        let columns = table_definitions
            .columns
            .iter()
            .map(|column| TableColumn::new(column.field, column.title, column.width))
            .collect();
        Self {
            wdg_table: WidgetTable::new("Instance Activity", table_definitions.query, columns),
        }
    }
}

impl Tab for TabActivity {
    fn get_name(&self) -> String {
        String::from("Activity")
    }

    fn update_data(&mut self, client: &mut Client) -> Result<(), Box<dyn std::error::Error>> {
        self.wdg_table.update_data(client)
    }

    fn handle_event(&mut self, keycode: KeyCode) {
        self.wdg_table.handle_event(keycode);
    }

    fn render(&mut self, frame: &mut Frame, area: Rect) {
        self.wdg_table.render(frame, area);
    }

    fn get_footer(&self) -> Line<'static> {
        Line::from_iter([
            Span::raw(" ↑↓→←").fg(P::APP_FOOTER_SHORTCUT),
            Span::raw(" navigation |").fg(P::APP_FOOTER_TEXT),
            Span::raw(" +-").fg(P::APP_FOOTER_SHORTCUT),
            Span::raw(" column width |").fg(P::APP_FOOTER_TEXT),
            Span::raw(" <>").fg(P::APP_FOOTER_SHORTCUT),
            Span::raw(" move column |").fg(P::APP_FOOTER_TEXT),
        ])
    }
}
