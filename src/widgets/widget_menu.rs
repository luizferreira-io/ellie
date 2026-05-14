use crate::palette as P;
use ratatui::{
    Frame,
    crossterm::event::KeyCode,
    layout::{Constraint, Rect},
    symbols,
    widgets::{Block, Row, Scrollbar, ScrollbarOrientation, ScrollbarState, Table, TableState},
};

struct VerticalScrollState {
    table_state: TableState,
    table_page_height: usize,
}

impl VerticalScrollState {
    fn new() -> Self {
        let mut table_state = TableState::default();
        table_state.select(Some(0));
        Self {
            table_state,
            table_page_height: 1,
        }
    }

    fn sync_scroll(&mut self, area_height: u16) {
        self.table_page_height = (area_height as usize).saturating_sub(3).max(1);
    }

    fn page_height(&self) -> usize {
        self.table_page_height
    }

    fn scrollbar_state(&self, row_count: usize) -> ScrollbarState {
        ScrollbarState::new(row_count.max(1)).position(self.table_state.selected().unwrap_or(0))
    }

    fn handle_event(&mut self, keycode: KeyCode, row_count: usize) {
        let last = row_count.saturating_sub(1);
        match keycode {
            KeyCode::Up => self.table_state.select_previous(),
            KeyCode::Down => {
                let next = self
                    .table_state
                    .selected()
                    .map(|i| (i + 1).min(last))
                    .unwrap_or(0);
                self.table_state.select(Some(next));
            }
            KeyCode::Home => self.table_state.select_first(),
            KeyCode::End => self.table_state.select(Some(last)),
            KeyCode::PageDown => {
                let next = self
                    .table_state
                    .selected()
                    .map(|i| (i + self.table_page_height).min(last))
                    .unwrap_or(0);
                self.table_state.select(Some(next));
            }
            KeyCode::PageUp => {
                let next = self
                    .table_state
                    .selected()
                    .map(|i| i.saturating_sub(self.table_page_height))
                    .unwrap_or(0);
                self.table_state.select(Some(next));
            }
            _ => {}
        }
    }
}

pub struct MenuColumn {
    pub title: &'static str,
    pub constraint: Constraint,
}

impl MenuColumn {
    pub fn new(title: &'static str, constraint: Constraint) -> Self {
        Self { title, constraint }
    }
}

pub struct WidgetMenu {
    title: &'static str,
    columns: Vec<MenuColumn>,
    items: Vec<Vec<String>>,
    active: bool,
    scroll: VerticalScrollState,
}

impl WidgetMenu {
    pub fn new(title: &'static str, columns: Vec<MenuColumn>, items: Vec<Vec<String>>) -> Self {
        Self {
            title,
            columns,
            items,
            active: false,
            scroll: VerticalScrollState::new(),
        }
    }

    pub fn set_active(&mut self, active: bool) {
        self.active = active;
    }

    pub fn get_selected_line(&self) -> usize {
        self.scroll.table_state.selected().unwrap_or(0)
    }

    pub fn get_changed_state(&mut self, keycode: KeyCode) -> bool {
        let previous_selected_line = self.get_selected_line();
        self.scroll.handle_event(keycode, self.items.len());
        self.get_selected_line() != previous_selected_line
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        self.scroll.sync_scroll(area.height);

        // Render table
        let header = Row::new(
            self.columns
                .iter()
                .map(|column| column.title)
                .collect::<Vec<_>>(),
        )
        .style((P::TABLE_HEADER_TEXT, P::TABLE_HEADER_BKG));

        let constraints: Vec<Constraint> = self
            .columns
            .iter()
            .map(|column| column.constraint)
            .collect();

        let rows: Vec<Row> = self
            .items
            .iter()
            .map(|column| Row::new(column.iter().map(|cell| cell.clone()).collect::<Vec<_>>()))
            .collect();

        let border_color = if self.active {
            P::WIDGET_BORDER_ACTIVE
        } else {
            P::WIDGET_BORDER_INACTIVE
        };

        let title_color = if self.active {
            P::WIDGET_TITLE_ACTIVE
        } else {
            P::WIDGET_TITLE_INACTIVE
        };

        let table = Table::new(rows, constraints)
            .block(
                Block::bordered()
                    .title(format!(" {} ", self.title))
                    .title_style(title_color)
                    .border_set(symbols::border::PLAIN)
                    .style((P::APP_COLOR1, P::APP_BKG))
                    .border_style(border_color),
            )
            .header(header)
            .row_highlight_style((P::TABLE_SELECTED_LINE_TEXT, P::TABLE_SELECTED_LINE_BKG));

        frame.render_stateful_widget(table, area, &mut self.scroll.table_state);

        // Render scrollbar if needed
        if self.items.len() > self.scroll.page_height() {
            let mut scrollbar_state = self.scroll.scrollbar_state(self.items.len());

            let scrollbar_area = Rect {
                x: area.right() - 1,
                y: area.y + 1,
                width: 1,
                height: area.height.saturating_sub(2),
            };

            frame.render_stateful_widget(
                Scrollbar::new(ScrollbarOrientation::VerticalRight)
                    .begin_symbol(Some("↑"))
                    .end_symbol(Some("↓"))
                    .track_symbol(Some("░"))
                    .thumb_symbol("▓"),
                scrollbar_area,
                &mut scrollbar_state,
            );
        }
    }
}
