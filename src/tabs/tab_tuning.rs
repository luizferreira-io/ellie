use crate::database::{QUERIES_TUNING, TuningKey, db_query, get_str};
use crate::palette as P;
use crate::tabs::tab::Tab;
use crate::widgets::widget_menu::{MenuColumn, WidgetMenu};
use crate::widgets::widget_table::{TableColumn, WidgetTable};
use postgres::Client;
use ratatui::{
    Frame,
    crossterm::event::KeyCode,
    layout::{Constraint, Layout, Rect},
    style::Stylize,
    text::{Line, Span},
};

struct ResultDef {
    category: &'static str,
    name: &'static str,
    key: TuningKey,
    db_scoped: bool,
}

#[rustfmt::skip]
const RESULTS: &[ResultDef] = &[
    ResultDef { category: "Disk allocation", name: "Disk allocation by tablespace",    key: TuningKey::DiskAllocationByTablespace,          db_scoped: false },
    ResultDef { category: "Disk allocation", name: "Disk allocation by database",      key: TuningKey::DiskAllocationByDatabase,            db_scoped: false },
    ResultDef { category: "Disk allocation", name: "Disk allocation by schema",        key: TuningKey::DiskAllocationBySchema,              db_scoped: true  },
    ResultDef { category: "Disk allocation", name: "Disk allocation by table",         key: TuningKey::DiskAllocationByTable,               db_scoped: true  },
    ResultDef { category: "Fragmentation",   name: "Bloated tables",                   key: TuningKey::BloatedTables,                       db_scoped: true  },
    ResultDef { category: "Fragmentation",   name: "Updated tables",                   key: TuningKey::UpdatedTables,                       db_scoped: true  },
    ResultDef { category: "Fragmentation",   name: "New page updated tables",          key: TuningKey::NewpageUpdatedTables,                db_scoped: true  },
    ResultDef { category: "Fragmentation",   name: "HOT updated tables",               key: TuningKey::HotUpdatedTables,                    db_scoped: true  },
    ResultDef { category: "Indexing",        name: "Missing indexes",                  key: TuningKey::MissingIndexes,                      db_scoped: true  },
    ResultDef { category: "Indexing",        name: "Unused indexes",                   key: TuningKey::UnusedIndexes,                       db_scoped: true  },
    ResultDef { category: "Indexing",        name: "Redundant indexes",                key: TuningKey::RedundantIndexes,                    db_scoped: true  },
    ResultDef { category: "Shared buffers",  name: "Shared buffers Content",           key: TuningKey::SharedBuffersContentServer,          db_scoped: false },
    ResultDef { category: "Shared buffers",  name: "Shared buffers Content",           key: TuningKey::SharedBuffersContentDatabase,        db_scoped: true  },
    ResultDef { category: "Shared buffers",  name: "Cache hit by database",            key: TuningKey::CacheHitByDb,                        db_scoped: false },
    ResultDef { category: "Shared buffers",  name: "Cache hit by table",               key: TuningKey::CacheHitByTable,                     db_scoped: false },
    ResultDef { category: "Queries",         name: "Time-consuming queries (total)",   key: TuningKey::TimeConsumingQueriesTotalServer,     db_scoped: false },
    ResultDef { category: "Queries",         name: "Time-consuming queries (average)", key: TuningKey::TimeConsumingQueriesAverageServer,   db_scoped: false },
    ResultDef { category: "Queries",         name: "Time-consuming queries (total)",   key: TuningKey::TimeConsumingQueriesTotalDatabase,   db_scoped: true  },
    ResultDef { category: "Queries",         name: "Time-consuming queries (average)", key: TuningKey::TimeConsumingQueriesAverageDatabase, db_scoped: true  },
];

#[derive(PartialEq)]
enum FocusedPanel {
    Menu,
    Results,
}

pub struct TabTuning {
    wdg_menu: WidgetMenu,
    wdg_results: Vec<WidgetTable>,
    current_db: String,
    focused_panel: FocusedPanel,
    refresh_pending: bool,
}

impl TabTuning {
    pub fn new() -> Self {
        // Menu (top panel)
        let menu_items = RESULTS
            .iter()
            .map(|definition| {
                vec![
                    definition.category.to_string(),
                    if definition.db_scoped {
                        "Database"
                    } else {
                        "Server"
                    }
                    .to_string(),
                    definition.name.to_string(),
                ]
            })
            .collect();

        let menu = WidgetMenu::new(
            "Tuning analysis",
            vec![
                MenuColumn::new("Category", Constraint::Length(20)),
                MenuColumn::new("Scope", Constraint::Length(12)),
                MenuColumn::new("Analysis", Constraint::Fill(1)),
            ],
            menu_items,
        );

        // Results (bottom panel)
        let analyses = RESULTS
            .iter()
            .map(|definition| {
                let table_def = QUERIES_TUNING.get(&definition.key).unwrap();
                let columns = table_def
                    .columns
                    .iter()
                    .map(|column| TableColumn::new(column.field, column.title, column.width))
                    .collect();
                let mut table = WidgetTable::new(definition.name, table_def.query, columns);
                table.set_active(false);
                table
            })
            .collect();

        Self {
            wdg_menu: menu,
            wdg_results: analyses,
            current_db: String::new(),
            focused_panel: FocusedPanel::Menu,
            refresh_pending: false,
        }
    }
}

impl Tab for TabTuning {
    fn get_name(&self) -> String {
        String::from("Tuning")
    }

    fn needs_refresh(&self) -> bool {
        self.refresh_pending
    }

    fn update_data(&mut self, client: &mut Client) -> Result<(), Box<dyn std::error::Error>> {
        let rows = client.query("SELECT current_database()::TEXT AS db_name", &[])?;
        if let Some(row) = rows.first() {
            self.current_db = get_str(row, "db_name");
        }
        let selected_analysis = self.wdg_menu.get_selected_line();
        let definition = &RESULTS[selected_analysis];
        let table_def = QUERIES_TUNING.get(&definition.key).unwrap();
        match db_query(client, table_def.query) {
            Ok(db_rows) => {
                let rows = db_rows
                    .iter()
                    .map(|row| {
                        table_def
                            .columns
                            .iter()
                            .map(|column| get_str(row, column.field))
                            .collect()
                    })
                    .collect();
                self.wdg_results[selected_analysis].set_rows(rows);
                self.wdg_results[selected_analysis].set_error(None);
            }
            Err(e) => {
                let error_msg = match definition.key {
                    TuningKey::SharedBuffersContentServer
                    | TuningKey::SharedBuffersContentDatabase => {
                        "Enable pg_buffercache extension\nto view shared buffers content."
                            .to_owned()
                    }
                    TuningKey::TimeConsumingQueriesTotalServer
                    | TuningKey::TimeConsumingQueriesAverageServer
                    | TuningKey::TimeConsumingQueriesTotalDatabase
                    | TuningKey::TimeConsumingQueriesAverageDatabase => {
                        "Enable pg_stat_statements extension\nto view time-consuming queries."
                            .to_owned()
                    }
                    _ => e.to_string(),
                };
                self.wdg_results[selected_analysis].set_rows(vec![]);
                self.wdg_results[selected_analysis].set_error(Some(error_msg));
            }
        }
        self.refresh_pending = false;
        Ok(())
    }

    fn handle_event(&mut self, keycode: KeyCode) {
        match keycode {
            KeyCode::Tab => {
                self.focused_panel = match self.focused_panel {
                    FocusedPanel::Menu => FocusedPanel::Results,
                    FocusedPanel::Results => FocusedPanel::Menu,
                };
            }
            _ => match self.focused_panel {
                FocusedPanel::Menu => {
                    if self.wdg_menu.get_changed_state(keycode) {
                        self.refresh_pending = true;
                    }
                }
                FocusedPanel::Results => {
                    let sel = self.wdg_menu.get_selected_line();
                    self.wdg_results[sel].handle_event(keycode);
                }
            },
        }
    }

    fn render(&mut self, frame: &mut Frame, area: Rect) {
        self.wdg_menu
            .set_active(self.focused_panel == FocusedPanel::Menu);

        // Create layout with two vertical panels (menu and results)
        let [section1, section2] =
            Layout::vertical([Constraint::Ratio(1, 3), Constraint::Ratio(2, 3)]).areas(area);

        // Render menu
        self.wdg_menu.render(frame, section1);

        // Render results
        let selected_analysis = self.wdg_menu.get_selected_line();
        let definition = &RESULTS[selected_analysis];
        let title = if self.wdg_results[selected_analysis].has_error() {
            format!("{} - Error", definition.name)
        } else if definition.db_scoped {
            format!("{} on \"{}\"", definition.name, self.current_db)
        } else {
            definition.name.to_string()
        };
        self.wdg_results[selected_analysis].set_title(title);
        self.wdg_results[selected_analysis].set_active(self.focused_panel == FocusedPanel::Results);
        self.wdg_results[selected_analysis].render(frame, section2);
    }

    fn get_footer(&self) -> Line<'static> {
        let mut spans = vec![
            Span::raw(" TAB").fg(P::APP_FOOTER_SHORTCUT),
            Span::raw(" switch panel |").fg(P::APP_FOOTER_TEXT),
        ];
        match self.focused_panel {
            FocusedPanel::Menu => spans.extend([
                Span::raw(" ↑↓").fg(P::APP_FOOTER_SHORTCUT),
                Span::raw(" navigation |").fg(P::APP_FOOTER_TEXT),
            ]),
            FocusedPanel::Results => spans.extend([
                Span::raw(" ↑↓→←").fg(P::APP_FOOTER_SHORTCUT),
                Span::raw(" navigation |").fg(P::APP_FOOTER_TEXT),
                Span::raw(" +-").fg(P::APP_FOOTER_SHORTCUT),
                Span::raw(" column width |").fg(P::APP_FOOTER_TEXT),
                Span::raw(" <>").fg(P::APP_FOOTER_SHORTCUT),
                Span::raw(" move column |").fg(P::APP_FOOTER_TEXT),
            ]),
        }
        Line::from(spans)
    }
}
