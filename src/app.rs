use crate::tabs::{
    Tab, TabAbout, TabActivity, TabDashboard, TabFileSettings, TabSettings, TabTuning,
};
use crate::{args::*, database::db_connect, palette as P};
use postgres::Client;
use ratatui::{
    Frame,
    crossterm::event::{self, Event, KeyCode, KeyEventKind},
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Tabs},
};
use std::time::Duration;

enum AppState {
    Starting,
    Running,
    Quitting,
}

pub struct App {
    args: ArgsStruct,
    state: AppState,
    tabs: Vec<Box<dyn Tab>>,
    current_tab: usize,
    client: Option<Client>,
    last_error: Option<String>,
}

impl App {
    pub fn new() -> Self {
        Self {
            args: get_arguments(),
            state: AppState::Starting,
            tabs: Vec::new(),
            current_tab: 0,
            client: None,
            last_error: None,
        }
    }

    pub fn init(&mut self) -> &mut Self {
        if self.args.help {
            print_help();
            std::process::exit(0);
        }

        self.tabs = vec![
            Box::new(TabDashboard::new()),
            Box::new(TabActivity::new()),
            Box::new(TabSettings::new()),
            Box::new(TabFileSettings::new()),
            Box::new(TabTuning::new()),
            Box::new(TabAbout::new()),
        ];

        self.state = AppState::Running;
        self
    }

    pub fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut client = db_connect(&self.args)?;
        for tab in self.tabs.iter_mut() {
            let _ = tab.update_data(&mut client);
        }
        self.client = Some(client);

        let mut terminal = ratatui::init();
        let result = self.event_loop(&mut terminal);
        ratatui::restore();
        result
    }

    fn event_loop(
        &mut self,
        terminal: &mut ratatui::DefaultTerminal,
    ) -> Result<(), Box<dyn std::error::Error>> {
        while matches!(self.state, AppState::Running) {
            terminal.draw(|frame| {
                let size = frame.area();
                let layout = Layout::vertical([
                    Constraint::Length(1), // area_header
                    Constraint::Min(0),    // area_main
                    Constraint::Length(1), // area_footer
                ]);
                let [area_header, area_main, area_footer] = layout.areas(size);

                self.render_header(frame, area_header);
                self.render_footer(frame, area_footer);
                self.tabs[self.current_tab].render(frame, area_main);
            })?;
            self.handle_event()?;
            self.refresh_current_tab_if_needed();
        }
        Ok(())
    }

    fn handle_event(&mut self) -> std::io::Result<()> {
        if event::poll(Duration::from_millis(250))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    self.last_error = None;
                    match key.code {
                        KeyCode::Char(']') => self.tab_next(),
                        KeyCode::Char('[') => self.tab_previous(),
                        KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => self.quit(),
                        keycode @ KeyCode::Char('1'..='9') => self.tab_set(keycode),
                        keycode => self.tabs[self.current_tab].handle_event(keycode),
                    }
                }
            }
        }
        Ok(())
    }

    fn quit(&mut self) {
        self.state = AppState::Quitting;
    }

    fn refresh_current_tab_if_needed(&mut self) {
        if !self.tabs[self.current_tab].needs_refresh() {
            return;
        }
        if let Some(ref mut client) = self.client {
            match self.tabs[self.current_tab].update_data(client) {
                Ok(()) => self.last_error = None,
                Err(e) => self.last_error = Some(e.to_string()),
            }
        }
    }

    fn render_header(&mut self, frame: &mut Frame, area: Rect) {
        let block = Block::default().style(Style::new().bg(P::APP_HEADER_BKG));
        frame.render_widget(block, area);

        let mut shortcut = 0;
        let titles = self.tabs.iter().map(|tab| {
            shortcut += 1;
            format!("[{}] {}", shortcut, tab.get_name())
        });

        let tabs = Tabs::new(titles)
            .style(
                Style::new()
                    .fg(P::APP_HEADER_TAB_INACTIVE)
                    .bg(P::APP_HEADER_BKG),
            )
            .highlight_style(
                Style::new()
                    .fg(P::APP_HEADER_TAB_ACTIVE)
                    .bg(P::APP_HEADER_BKG),
            )
            .select(self.current_tab)
            .padding(" ", " ")
            .add_modifier(Modifier::BOLD)
            .divider("|");
        frame.render_widget(tabs, area);
    }

    fn render_footer(&mut self, frame: &mut Frame, area: Rect) {
        if let Some(ref err) = self.last_error {
            frame.render_widget(
                Block::default().style(Style::new().bg(P::APP_FOOTER_SHORTCUT)),
                area,
            );
            frame.render_widget(
                Line::from_iter([
                    Span::raw(" Error: ")
                        .fg(P::APP_COLOR2)
                        .add_modifier(Modifier::BOLD),
                    Span::raw(err.clone()).fg(P::APP_COLOR1),
                ]),
                area,
            );
            return;
        }

        frame.render_widget(
            Block::default().style(Style::new().fg(P::APP_FOOTER_TEXT).bg(P::APP_FOOTER_BKG)),
            area,
        );

        let tabs_len = self.tabs.len();
        let tab_footer = self.tabs[self.current_tab].get_footer();

        let mut spans = vec![
            // 1-6/[] tabs
            Span::raw(" 1").fg(P::APP_FOOTER_SHORTCUT),
            Span::raw("-"),
            Span::raw(format!("{}", tabs_len)).fg(P::APP_FOOTER_SHORTCUT),
            Span::raw("/"),
            Span::raw("[]").fg(P::APP_FOOTER_SHORTCUT),
            Span::raw(" tabs"),
            Span::raw(" | "),
            // q/ESC quit
            Span::raw("q").fg(P::APP_FOOTER_SHORTCUT),
            Span::raw("/"),
            Span::raw("ESC").fg(P::APP_FOOTER_SHORTCUT),
            Span::raw(" quit |"),
        ];
        if !tab_footer.spans.is_empty() {
            //spans.push(Span::raw(" | "));
            spans.extend(tab_footer.spans);
        }

        frame.render_widget(Line::from(spans), area);
        frame.render_widget(
            Span::raw(format!("Ellie {} ", crate::VERSION))
                .fg(P::APP_COLOR4)
                .into_right_aligned_line()
                .add_modifier(Modifier::BOLD),
            area,
        );
    }

    fn tab_next(&mut self) {
        if self.current_tab < self.tabs.len() - 1 {
            self.current_tab += 1;
        }
    }

    fn tab_previous(&mut self) {
        if self.current_tab > 0 {
            self.current_tab -= 1;
        }
    }

    fn tab_set(&mut self, keycode: KeyCode) {
        if let KeyCode::Char(c) = keycode {
            if let Some(digit) = c.to_digit(10) {
                let index = digit as usize;
                if index > 0 && index <= self.tabs.len() {
                    self.current_tab = index - 1;
                }
            }
        }
    }
}
