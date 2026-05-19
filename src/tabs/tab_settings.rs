use crate::database::{QUERIES_SETTINGS, SettingsKey};
use crate::palette as P;
use crate::tabs::tab::Tab;
use crate::widgets::widget_table::{TableColumn, WidgetTable};
use crate::widgets::widget_text_box::WidgetTextBox;
use postgres::Client;
use ratatui::{
    Frame,
    crossterm::event::KeyCode,
    layout::{Constraint, Layout, Rect},
    style::Stylize,
    text::{Line, Span},
};

pub struct TabSettings {
    wdg_table: WidgetTable,
    wdg_description: WidgetTextBox,
}

impl TabSettings {
    pub fn new() -> Self {
        let table_definitions = QUERIES_SETTINGS.get(&SettingsKey::Settings).unwrap();
        let table_columns = table_definitions
            .columns
            .iter()
            .map(|column| {
                TableColumn::new(
                    column.field,
                    column.title,
                    column.width,
                    column.constraint.clone(),
                )
            })
            .collect();
        Self {
            wdg_table: WidgetTable::new("Settings", table_definitions.query, table_columns),
            wdg_description: WidgetTextBox::new("Description"),
        }
    }
}

impl Tab for TabSettings {
    fn get_name(&self) -> String {
        String::from("Settings")
    }

    fn update_data(&mut self, client: &mut Client) -> Result<(), Box<dyn std::error::Error>> {
        self.wdg_table.update_data(client)
    }

    fn handle_event(&mut self, keycode: KeyCode) {
        self.wdg_table.handle_event(keycode);
    }

    fn render(&mut self, frame: &mut Frame, area: Rect) {
        let layout = Layout::vertical([Constraint::Min(0), Constraint::Length(7)]);
        let [area_table, area_description] = layout.areas(area);

        self.wdg_table.render(frame, area_table);

        let description_text = self
            .wdg_table
            .get_selected_line_values()
            .and_then(|map| map.get("_description").cloned())
            .unwrap_or_default();
        self.wdg_description.refresh_data(description_text);
        self.wdg_description.render(frame, area_description);
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
