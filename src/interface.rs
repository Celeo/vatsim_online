use crate::models::V3ResponseData;
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
    widgets::{Block, Borders, Cell, Row, Table, TableState},
    Terminal,
};

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
            self.data.controllers.len()
        } else {
            self.data.pilots.len()
        };
        let next = if sel >= length - 1 { 0 } else { sel + 1 };
        self.table_states[self.tab_index].select(Some(next));
    }

    fn up(&mut self) {
        let sel = self.table_states[self.tab_index].selected().unwrap_or(0);
        let length = if self.tab_index == 0 {
            self.data.controllers.len()
        } else {
            self.data.pilots.len()
        };
        let next = if sel == 0 { length - 1 } else { sel - 1 };
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
                        controller.rating.to_string(), // TODO look up in table rather than bitmask
                    ]
                })
                .collect()
        }
    }

    fn get_headers(&self) -> Vec<&'static str> {
        if self.tab_index == 0 {
            vec!["Name", "Callsign", "Lat", "Long"]
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
                .margin(1)
                .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
                .split(f.size());

            f.render_widget(
                Block::default().borders(Borders::ALL).title("Data sources"),
                chunks[0],
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
                    Constraint::Percentage(20),
                    Constraint::Percentage(20),
                    Constraint::Percentage(20),
                    Constraint::Percentage(20),
                ])
                .column_spacing(1)
                .highlight_style(*SELECTED_STYLE)
                .highlight_symbol(">> ");
            f.render_stateful_widget(table, chunks[1], &mut app.current_table_state());
        })?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => break,
                KeyCode::Down => app.down(),
                KeyCode::Up => app.up(),
                KeyCode::Tab => app.tab_over(),
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
