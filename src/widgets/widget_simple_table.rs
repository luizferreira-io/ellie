use crate::database::{DatabaseColumnDefinition, get_str};
use crate::palette as P;
use postgres::Row;
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Rect},
    style::Style,
    symbols,
    widgets::{Block, Cell, Paragraph, Row as RataRow, Table, Widget},
};

pub struct WidgetSimpleTable {
    title: &'static str,
    columns: Vec<DatabaseColumnDefinition>,
    rows: Vec<Row>,
    error: Option<String>,
}

impl WidgetSimpleTable {
    pub(crate) fn new(title: &'static str, columns: Vec<DatabaseColumnDefinition>) -> Self {
        Self {
            title,
            columns,
            rows: Vec::new(),
            error: None,
        }
    }

    pub fn update_data(&mut self, rows: Vec<Row>, error: Option<String>) {
        self.rows = rows;
        self.error = error;
    }

    pub fn get_height(&self) -> u16 {
        2 + 1 + self.rows.len() as u16 // borders + header + rows
    }
}

impl Widget for &WidgetSimpleTable {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Render error paragraph if there's an error message
        if let Some(ref err) = self.error {
            let title = format!(" {} ", self.title);
            let block = Block::bordered()
                .title(title)
                .title_style(P::WIDGET_TITLE_ACTIVE)
                .border_set(symbols::border::PLAIN)
                .style((P::APP_COLOR1, P::APP_BKG))
                .border_style(P::WIDGET_BORDER_ACTIVE);
            let inner = block.inner(area);
            block.render(area, buf);
            let lines = err.lines().count() as u16;
            let y = inner.y + inner.height.saturating_sub(lines) / 2;
            let centered = Rect {
                x: inner.x,
                y,
                width: inner.width,
                height: lines.min(inner.height),
            };
            Paragraph::new(err.as_str())
                .alignment(Alignment::Center)
                .style(Style::new().fg(P::APP_ERROR))
                .render(centered, buf);
            return;
        }

        // Table definitions
        let constraints: Vec<Constraint> = self
            .columns
            .iter()
            .map(|col| Constraint::Length(col.width))
            .collect();
        let headers: Vec<Cell> = self
            .columns
            .iter()
            .map(|col| Cell::new(col.title))
            .collect();
        let rows: Vec<RataRow> = self
            .rows
            .iter()
            .map(|row| {
                RataRow::new(
                    self.columns
                        .iter()
                        .map(|col| Cell::new(get_str(row, col.field)))
                        .collect::<Vec<_>>(),
                )
            })
            .collect();
        let title = format!(" {} ", self.title);

        // Render table
        Table::new(rows, constraints)
            .block(
                Block::bordered()
                    .title(title)
                    .title_style(P::WIDGET_TITLE_ACTIVE)
                    .border_set(symbols::border::PLAIN)
                    .style((P::APP_COLOR1, P::APP_BKG))
                    .border_style(P::WIDGET_BORDER_ACTIVE),
            )
            .header(RataRow::new(headers).style((P::TABLE_HEADER_TEXT, P::TABLE_HEADER_BKG)))
            .column_spacing(1)
            .render(area, buf);
    }
}
