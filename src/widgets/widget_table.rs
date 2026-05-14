#![allow(dead_code)]

use crate::database::{db_query, get_str};
use crate::palette as P;
use postgres::Client;
use ratatui::{
    Frame,
    crossterm::event::KeyCode,
    layout::{Alignment, Constraint, Rect},
    style::Style,
    symbols,
    text::Line,
    widgets::{Block, Cell, Paragraph, Row, Scrollbar, ScrollbarOrientation, ScrollbarState, Table, TableState},
};
use std::collections::HashMap;
use std::time::Instant;

const MIN_COL_WIDTH: u16 = 5;
const MAX_COL_WIDTH: u16 = 120;

struct CompleteScrollState {
    table_state: TableState,
    table_page_height: usize,
    table_selected_column: usize,
    table_first_visible_column: usize,
    table_column_widths: Vec<u16>,
    table_column_order: Vec<usize>,
}

impl CompleteScrollState {
    fn new(col_widths: &[u16]) -> Self {
        let mut table_state = TableState::default();
        table_state.select(Some(0));
        Self {
            table_state,
            table_page_height: 1,
            table_selected_column: 0,
            table_first_visible_column: 0,
            table_column_widths: col_widths.to_vec(),
            table_column_order: (0..col_widths.len()).collect(),
        }
    }

    fn visible_column_end(&self, starting: usize, width: u16) -> usize {
        let mut used: u16 = 0;
        for i in 0..(self.table_column_order.len() - starting) {
            let col_w = self.table_column_widths[self.table_column_order[starting + i]];
            let spacing: u16 = if i == 0 { 0 } else { 1 };
            if i > 0 && used + spacing + col_w > width {
                return starting + i;
            }
            used += spacing + col_w;
        }
        self.table_column_order.len()
    }

    fn sync_scroll(&mut self, area_height: u16, inner_width: u16) -> Vec<usize> {
        self.table_page_height = (area_height as usize).saturating_sub(3).max(1);

        if self.table_selected_column < self.table_first_visible_column {
            self.table_first_visible_column = self.table_selected_column;
        }
        while self.visible_column_end(self.table_first_visible_column, inner_width)
            <= self.table_selected_column
        {
            self.table_first_visible_column =
                (self.table_first_visible_column + 1).min(self.table_selected_column);
        }
        let last = self.visible_column_end(self.table_first_visible_column, inner_width);
        let relative = self.table_selected_column - self.table_first_visible_column;
        self.table_state.select_column(Some(relative));

        (self.table_first_visible_column..last)
            .map(|i| self.table_column_order[i])
            .collect()
    }

    fn scrollbar_info(&self) -> (usize, usize, usize, usize) {
        (
            self.table_state.selected().unwrap_or(0),
            self.table_page_height,
            self.table_selected_column,
            self.table_column_order.len(),
        )
    }

    fn col_constraints(&self, col_defs: &[usize]) -> Vec<Constraint> {
        col_defs
            .iter()
            .map(|&i| Constraint::Length(self.table_column_widths[i]))
            .collect()
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
            KeyCode::Left => {
                self.table_selected_column = self.table_selected_column.saturating_sub(1);
            }
            KeyCode::Right => {
                let next = self.table_selected_column + 1;
                if next < self.table_column_order.len()
                    && self.table_column_widths[self.table_column_order[next]] > 0
                {
                    self.table_selected_column = next;
                }
            }
            KeyCode::Char('+') => {
                let col_i = self.table_column_order[self.table_selected_column];
                let w = &mut self.table_column_widths[col_i];
                *w = (*w + 1).min(MAX_COL_WIDTH);
            }
            KeyCode::Char('-') => {
                let col_i = self.table_column_order[self.table_selected_column];
                let w = &mut self.table_column_widths[col_i];
                *w = w.saturating_sub(1).max(MIN_COL_WIDTH);
            }
            KeyCode::Char('<') => {
                let pos = self.table_selected_column;
                if pos > 0 {
                    self.table_column_order.swap(pos - 1, pos);
                    self.table_selected_column -= 1;
                }
            }
            KeyCode::Char('>') => {
                let pos = self.table_selected_column;
                if pos < self.table_column_order.len() - 1 {
                    self.table_column_order.swap(pos, pos + 1);
                    self.table_selected_column += 1;
                }
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scroll(widths: &[u16]) -> CompleteScrollState {
        CompleteScrollState::new(widths)
    }

    // --- visible_column_end ---

    #[test]
    fn visible_column_end_all_columns_fit() {
        let s = scroll(&[10, 10, 10]);
        // 10 + 1+10 + 1+10 = 32, width=40 → todas cabem
        assert_eq!(s.visible_column_end(0, 40), 3);
    }

    #[test]
    fn visible_column_end_two_columns_fit() {
        let s = scroll(&[10, 10, 10]);
        // 10 + 1+10 = 21 ≤ 25; 21+1+10=32 > 25 → corta na terceira
        assert_eq!(s.visible_column_end(0, 25), 2);
    }

    #[test]
    fn visible_column_end_only_first_fits() {
        let s = scroll(&[10, 10, 10]);
        // 10 ≤ 10; 10+1+10=21 > 10 → corta na segunda
        assert_eq!(s.visible_column_end(0, 10), 1);
    }

    #[test]
    fn visible_column_end_from_offset() {
        let s = scroll(&[10, 10, 10]);
        // a partir da col 1: 10 + 1+10 = 21 ≤ 25 → ambas cabem
        assert_eq!(s.visible_column_end(1, 25), 3);
    }

    #[test]
    fn visible_column_end_single_column() {
        let s = scroll(&[15]);
        assert_eq!(s.visible_column_end(0, 100), 1);
    }

    // --- scrollbar_info ---

    #[test]
    fn scrollbar_info_initial_state() {
        let s = scroll(&[10, 20, 30]);
        let (row, page_height, col, total_cols) = s.scrollbar_info();
        assert_eq!(row, 0);
        assert_eq!(page_height, 1); // default antes de sync_scroll
        assert_eq!(col, 0);
        assert_eq!(total_cols, 3);
    }

    // --- sync_scroll ---

    #[test]
    fn sync_scroll_returns_all_visible_columns() {
        let mut s = scroll(&[10, 10, 10]);
        let cols = s.sync_scroll(10, 40);
        assert_eq!(cols, vec![0, 1, 2]);
    }

    #[test]
    fn sync_scroll_limits_columns_by_width() {
        let mut s = scroll(&[10, 10, 10]);
        // width=10: só a col 0 cabe (10 ≤ 10; 10+1+10=21 > 10)
        let cols = s.sync_scroll(10, 10);
        assert_eq!(cols, vec![0]);
    }

    #[test]
    fn sync_scroll_updates_page_height() {
        let mut s = scroll(&[10, 10]);
        s.sync_scroll(10, 100); // height=10 → page_height = max(1, 10-3) = 7
        let (_, page_height, _, _) = s.scrollbar_info();
        assert_eq!(page_height, 7);
    }

    #[test]
    fn sync_scroll_minimum_page_height_is_one() {
        let mut s = scroll(&[10]);
        s.sync_scroll(2, 100); // height=2 → 2-3 saturating = 0 → max(1,0) = 1
        let (_, page_height, _, _) = s.scrollbar_info();
        assert_eq!(page_height, 1);
    }
}

pub struct TableColumn {
    pub field: &'static str,
    pub title: &'static str,
    pub width: u16,
}

impl TableColumn {
    pub fn new(field: &'static str, title: &'static str, width: u16) -> Self {
        Self {
            field,
            title,
            width,
        }
    }
}

pub struct WidgetTable {
    title: String,
    query: &'static str,
    columns: Vec<TableColumn>,
    rows: Vec<Vec<String>>,
    error: Option<String>,
    scroll: CompleteScrollState,
    auto_refresh_secs: u64,
    last_refresh: Option<Instant>,
    interactive: bool,
    active: bool,
}

impl WidgetTable {
    pub fn new(title: impl Into<String>, query: &'static str, columns: Vec<TableColumn>) -> Self {
        let widths: Vec<u16> = columns.iter().map(|c| c.width).collect();
        Self {
            title: title.into(),
            query,
            columns,
            rows: Vec::new(),
            error: None,
            scroll: CompleteScrollState::new(&widths),
            auto_refresh_secs: 0,
            last_refresh: None,
            interactive: true,
            active: true,
        }
    }

    pub fn get_row_count(&self) -> usize {
        self.rows.len()
    }

    pub fn get_selected_row(&self) -> Option<usize> {
        self.scroll.table_state.selected()
    }

    pub fn get_selected_line_values(&self) -> Option<HashMap<&'static str, String>> {
        let row_idx = self.scroll.table_state.selected()?;
        let row = self.rows.get(row_idx)?;
        Some(
            self.columns
                .iter()
                .enumerate()
                .map(|(i, col)| (col.title, row[i].clone()))
                .collect(),
        )
    }

    pub fn set_auto_refresh(&mut self, secs: u64) {
        self.auto_refresh_secs = secs;
    }

    pub fn set_interactive(&mut self, interactive: bool) {
        self.interactive = interactive;
    }

    pub fn set_active(&mut self, active: bool) {
        self.active = active;
    }

    pub fn set_title(&mut self, title: impl Into<String>) {
        self.title = title.into();
    }

    pub fn set_error(&mut self, error: Option<String>) {
        self.error = error;
    }

    pub fn set_rows(&mut self, rows: Vec<Vec<String>>) {
        self.rows = rows;
        self.last_refresh = Some(Instant::now());
    }

    pub fn has_error(&self) -> bool {
        self.error.is_some()
    }

    pub fn is_interactive(&self) -> bool {
        self.active && self.interactive
    }

    pub fn needs_auto_refresh(&self) -> bool {
        if self.auto_refresh_secs == 0 {
            return false;
        }
        match self.last_refresh {
            None => true,
            Some(t) => t.elapsed().as_secs() >= self.auto_refresh_secs,
        }
    }

    pub fn update_data(&mut self, client: &mut Client) -> Result<(), Box<dyn std::error::Error>> {
        let db_rows = db_query(client, self.query)?;
        let new_rows: Vec<Vec<String>> = db_rows
            .iter()
            .map(|row| {
                self.columns
                    .iter()
                    .map(|col| get_str(row, col.field))
                    .collect()
            })
            .collect();
        self.rows = new_rows;
        self.last_refresh = Some(Instant::now());
        Ok(())
    }

    pub fn handle_event(&mut self, keycode: KeyCode) {
        if !self.is_interactive() {
            return;
        }
        let row_count = self.rows.len();
        self.scroll.handle_event(keycode, row_count);
    }

    pub fn get_height(&self) -> u16 {
        2 + self.rows.len() as u16
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        let (border_color, title_color) = if self.active {
            (P::WIDGET_BORDER_ACTIVE, P::WIDGET_TITLE_ACTIVE)
        } else {
            (P::WIDGET_BORDER_INACTIVE, P::WIDGET_TITLE_INACTIVE)
        };

        // Render error paragraph if there's an error message
        if let Some(ref err) = self.error {
            let title = format!(" {} ", self.title);
            let block = Block::bordered()
                .title(title)
                .title_style(title_color)
                .border_set(symbols::border::PLAIN)
                .style((P::APP_COLOR1, P::APP_BKG))
                .border_style(border_color);
            let inner = block.inner(area);
            frame.render_widget(block, area);
            let lines = err.lines().count() as u16;
            let y = inner.y + inner.height.saturating_sub(lines) / 2;
            let centered = Rect { x: inner.x, y, width: inner.width, height: lines.min(inner.height) };
            frame.render_widget(
                Paragraph::new(err.as_str())
                    .alignment(Alignment::Center)
                    .style(Style::new().fg(P::APP_ERROR)),
                centered,
            );
            return;
        }

        // Table definitions
        let inner_width = area.width.saturating_sub(2);
        let visible_cols = self.scroll.sync_scroll(area.height, inner_width);
        let table_headers: Vec<&str> = visible_cols
            .iter()
            .map(|&i| self.columns[i].title)
            .collect();
        let table_widths = self.scroll.col_constraints(&visible_cols);
        let table_rows: Vec<Row> = if self.rows.is_empty() {
            let cells: Vec<Cell> = visible_cols
                .iter()
                .enumerate()
                .map(|(n, _)| Cell::new(if n == 0 { "No data found." } else { "" }))
                .collect();
            vec![Row::new(cells)]
        } else {
            self.rows
                .iter()
                .map(|row| {
                    Row::new(
                        visible_cols
                            .iter()
                            .map(|&i| Cell::new(row[i].clone()))
                            .collect::<Vec<_>>(),
                    )
                })
                .collect()
        };
        let scrollbar_style = Style::new().fg(border_color);

        let title = format!(" {} ", self.title);
        let selected_row = self.scroll.table_state.selected().unwrap_or(0);
        let total = self.rows.len();
        let row_info = if total > 0 {
            Line::from(format!(" [{}/{}] ", (selected_row + 1).min(total), total))
                .alignment(Alignment::Right)
        } else {
            Line::default()
        };

        // Render table
        let table = Table::new(table_rows, table_widths)
            .block(
                Block::bordered()
                    .title(title)
                    .title_top(row_info)
                    .title_style(title_color)
                    .border_set(symbols::border::PLAIN)
                    .style((P::APP_COLOR1, P::APP_BKG))
                    .border_style(border_color),
            )
            .header(Row::new(table_headers).style((P::TABLE_HEADER_TEXT, P::TABLE_HEADER_BKG)))
            .column_spacing(1)
            .column_highlight_style((P::APP_COLOR1, P::TABLE_SELECTED_COLUMN_BKG))
            .row_highlight_style((P::TABLE_SELECTED_LINE_TEXT, P::TABLE_SELECTED_LINE_BKG))
            .cell_highlight_style((P::TABLE_SELECTED_LINE_TEXT, P::TABLE_SELECTED_LINE_BKG));

        frame.render_stateful_widget(table, area, &mut self.scroll.table_state);

        // Vertical scrollbar
        let (selected_row, page_height, selected_col, total_cols) = self.scroll.scrollbar_info();
        let row_count = self.rows.len();
        if row_count > page_height {
            let mut vs = ScrollbarState::new(row_count).position(selected_row);
            let vs_rect = Rect {
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
                    .thumb_symbol("▓")
                    .style(scrollbar_style),
                vs_rect,
                &mut vs,
            );
        }

        // Horizontal scrollbar
        if visible_cols.len() < total_cols {
            let mut hs = ScrollbarState::new(total_cols).position(selected_col);
            let hs_rect = Rect {
                x: area.x + 1,
                y: area.bottom() - 1,
                width: area.width.saturating_sub(2),
                height: 1,
            };
            frame.render_stateful_widget(
                Scrollbar::new(ScrollbarOrientation::HorizontalBottom)
                    .begin_symbol(Some("←"))
                    .end_symbol(Some("→"))
                    .track_symbol(Some("░"))
                    .thumb_symbol("▓")
                    .style(scrollbar_style),
                hs_rect,
                &mut hs,
            );
        }
    }
}
