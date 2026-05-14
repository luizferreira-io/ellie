use postgres::Client;
use ratatui::{Frame, crossterm::event::KeyCode, layout::Rect, text::Line};

pub trait Tab {
    fn get_name(&self) -> String;
    fn get_footer(&self) -> Line<'static> {
        Line::default()
    }
    fn render(&mut self, frame: &mut Frame, area: Rect);
    fn handle_event(&mut self, _keycode: KeyCode) {}
    fn update_data(&mut self, _client: &mut Client) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
    fn needs_refresh(&self) -> bool {
        false
    }
}
