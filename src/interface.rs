use crate::{api::Vatsim, models::V3ResponseData};
use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use log::debug;
use once_cell::sync::Lazy;
use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans, Text},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, TableState},
    Terminal,
};

const HELP_TEXT: &str = "   Tab to switch sources. Up and down to navigate. Q to exit.";
static NORMAL_STYLE: Lazy<Style> = Lazy::new(|| Style::default().bg(Color::Blue));
static SELECTED_STYLE: Lazy<Style> =
    Lazy::new(|| Style::default().add_modifier(Modifier::REVERSED));

struct App {
    tab_index: usize,
    table_states: [TableState; 2],
    data: V3ResponseData,
}

impl App {
    fn new(data: V3ResponseData) -> Self {
        let mut state = TableState::default();
        state.select(Some(0));
        Self {
            tab_index: 0,
            table_states: [state.clone(), state.clone()],
            data,
        }
    }

    fn tab_over(&mut self) {
        self.tab_index = if self.tab_index == 0 { 1 } else { 0 };
        self.table_states[0].select(Some(0));
        self.table_states[1].select(Some(0));
    }

    fn down(&mut self) {
        let sel = self.table_states[self.tab_index].selected().unwrap_or(0);
        let length = if self.tab_index == 0 {
            self.data.pilots.len()
        } else {
            self.data.controllers.len()
        };
        let next = if sel >= length - 1 { 0 } else { sel + 1 };
        self.table_states[self.tab_index].select(Some(next));
    }

    fn up(&mut self) {
        let sel = self.table_states[self.tab_index].selected().unwrap_or(0);
        let length = if self.tab_index == 0 {
            self.data.pilots.len()
        } else {
            self.data.controllers.len()
        };
        let next = if sel == 0 { length - 1 } else { sel - 1 };
        self.table_states[self.tab_index].select(Some(next));
    }

    fn page_down(&mut self) {
        let sel = self.table_states[self.tab_index].selected().unwrap_or(0);
        let length = if self.tab_index == 0 {
            self.data.pilots.len()
        } else {
            self.data.controllers.len()
        };
        let next = if sel + 10 >= length {
            length - 1
        } else {
            sel + 10
        };
        self.table_states[self.tab_index].select(Some(next));
    }

    fn page_up(&mut self) {
        let sel = self.table_states[self.tab_index].selected().unwrap_or(0);
        let next = if sel <= 10 { 0 } else { sel - 10 };
        self.table_states[self.tab_index].select(Some(next));
    }

    fn get_tab_data(&self) -> Vec<Vec<String>> {
        if self.tab_index == 0 {
            self.data
                .pilots
                .iter()
                .map(|pilot| {
                    vec![
                        pilot.name.clone(),
                        pilot.callsign.clone(),
                        pilot.flight_plan.as_ref().map_or_else(
                            || String::from("???"),
                            |fp| {
                                if !fp.aircraft_faa.is_empty() {
                                    fp.aircraft_faa.clone()
                                } else if !fp.aircraft_short.is_empty() {
                                    fp.aircraft_short.clone()
                                } else {
                                    String::from("???")
                                }
                            },
                        ),
                        pilot.latitude.to_string(),
                        pilot.longitude.to_string(),
                    ]
                })
                .collect()
        } else {
            self.data
                .controllers
                .iter()
                .map(|controller| {
                    vec![
                        controller.name.clone(),
                        controller.callsign.clone(),
                        controller.frequency.clone(),
                        Vatsim::controller_rating_lookup(&self.data, controller.rating),
                    ]
                })
                .collect()
        }
    }

    fn get_headers(&self) -> Vec<&'static str> {
        if self.tab_index == 0 {
            vec!["Name", "Callsign", "Aircraft", "Lat", "Long"]
        } else {
            vec!["Name", "Callsign", "Frequency", "Rating"]
        }
    }

    fn get_selected_title(&self) -> &'static str {
        if self.tab_index == 0 {
            "Pilots"
        } else {
            "Controllers"
        }
    }

    fn current_table_state(&mut self) -> &mut TableState {
        &mut self.table_states[self.tab_index]
    }

    fn tab_header(&self) -> Vec<Span> {
        let active = Style::default()
            .bg(Color::LightGreen)
            .fg(Color::Black)
            .add_modifier(Modifier::BOLD);
        let inactive = Style::default();
        vec![
            Span::raw("   "),
            Span::styled(
                "Pilots",
                if self.tab_index == 0 {
                    active
                } else {
                    inactive
                },
            ),
            Span::raw("  <->  "),
            Span::styled(
                "Controllers",
                if self.tab_index == 1 {
                    active
                } else {
                    inactive
                },
            ),
        ]
    }
}

pub fn run(data: V3ResponseData) -> Result<()> {
    debug!(
        "interface::run, {} pilots, {} controllers",
        data.pilots.len(),
        data.controllers.len()
    );

    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    enable_raw_mode()?;
    terminal.hide_cursor()?;
    let mut app = App::new(data);

    loop {
        let _ = terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .horizontal_margin(1)
                .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
                .split(f.size());

            let title_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Length(32),
                    Constraint::Min(1),
                    Constraint::Length((HELP_TEXT.len() + 5).try_into().unwrap()),
                ])
                .split(chunks[0]);

            let tab_header = Paragraph::new(vec![Spans::from(app.tab_header())])
                .block(Block::default().borders(Borders::ALL).title("Data sources"));
            f.render_widget(tab_header, title_chunks[0]);
            f.render_widget(
                Paragraph::new(Text::from(HELP_TEXT))
                    .block(Block::default().borders(Borders::ALL).title("Help")),
                title_chunks[2],
            );

            let headers = app.get_headers();
            let header_cells = headers.iter().map(|&h| Cell::from(h));
            let header = Row::new(header_cells).style(*NORMAL_STYLE).height(1);
            let tab_data = app.get_tab_data();
            let rows = tab_data
                .iter()
                .map(|items| Row::new(items.iter().map(|c| Cell::from(c.clone()))));
            let table = Table::new(rows)
                .header(header)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(app.get_selected_title()),
                )
                .widths(&[
                    Constraint::Percentage(25),
                    Constraint::Percentage(25),
                    Constraint::Percentage(25),
                    Constraint::Percentage(25),
                ])
                .highlight_style(*SELECTED_STYLE)
                .highlight_symbol(">> ");
            f.render_stateful_widget(table, chunks[1], app.current_table_state());
        })?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => break,
                KeyCode::Down => app.down(),
                KeyCode::Up => app.up(),
                KeyCode::Tab => app.tab_over(),
                KeyCode::PageDown => app.page_down(),
                KeyCode::PageUp => app.page_up(),
                _ => {}
            }
        }
    }

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    Ok(())
}
