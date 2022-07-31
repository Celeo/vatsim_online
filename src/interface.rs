use crate::{
    models::V3ResponseData,
    state::{App, SelectedRow},
};
use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use log::{debug, error};
use once_cell::sync::Lazy;
use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Spans, Text},
    widgets::{Block, Borders, Cell, Clear, Paragraph, Row, Table, Widget, Wrap},
    Terminal,
};

/// Text shown in the top right.
const HELP_TEXT: &str =
    "   Tab to switch sources. Up and down to navigate. Enter to examine; Esc to close. O to view online stats. Q to exit.";
/// Style applied to the table header row.
static NORMAL_STYLE: Lazy<Style> = Lazy::new(|| Style::default().bg(Color::Blue));
/// Style applied to non-header table rows.
static SELECTED_STYLE: Lazy<Style> =
    Lazy::new(|| Style::default().add_modifier(Modifier::REVERSED));

/// Run the terminal interface.
pub fn run(data: V3ResponseData) -> Result<()> {
    debug!(
        "interface::run, {} pilots, {} controllers",
        data.pilots.len(),
        data.controllers.len()
    );

    // configure terminal
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    enable_raw_mode()?;
    terminal.hide_cursor()?;
    let mut app = App::new(data);

    loop {
        let view_data = app.get_view_data();
        let _ = terminal.draw(|f| {
            // general layout
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .horizontal_margin(1)
                .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
                .split(f.size());

            // "title row" layout
            let title_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Length(32),
                    Constraint::Min(1),
                    Constraint::Length((HELP_TEXT.len() + 5).try_into().unwrap()),
                ])
                .split(chunks[0]);

            // data sources switcher and help text
            let tab_header = Paragraph::new(vec![Spans::from(app.tab_header())])
                .block(Block::default().borders(Borders::ALL).title("Data sources"));
            f.render_widget(tab_header, title_chunks[0]);
            f.render_widget(
                Paragraph::new(Text::from(HELP_TEXT))
                    .block(Block::default().borders(Borders::ALL).title("Help")),
                title_chunks[2],
            );

            // table
            let header_cells = view_data.headers.iter().map(|&h| Cell::from(h));
            let header = Row::new(header_cells).style(*NORMAL_STYLE).height(1);
            let rows = view_data
                .data
                .iter()
                .map(|items| Row::new(items.iter().map(|c| Cell::from(c.clone()))));
            let table = Table::new(rows)
                .header(header)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(view_data.title),
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

            // popup
            if view_data.show_popup {
                let area = centered_rect(70, 50, f.size());
                f.render_widget(Clear, area);
                f.render_widget(popup_text(&view_data.selected_row_data), area);
            }
        })?;

        // key press handlers
        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') => break,
                KeyCode::Down => {
                    if !view_data.show_popup {
                        app.down()
                    }
                }
                KeyCode::Up => {
                    if !view_data.show_popup {
                        app.up()
                    }
                }
                KeyCode::Tab => {
                    if !view_data.show_popup {
                        app.tab_over()
                    }
                }
                KeyCode::PageDown => {
                    if !view_data.show_popup {
                        app.page_down()
                    }
                }
                KeyCode::PageUp => {
                    if !view_data.show_popup {
                        app.page_up()
                    }
                }
                KeyCode::Enter => app.toggle_popup(true),
                KeyCode::Esc => app.toggle_popup(false),
                KeyCode::Char('o') => {
                    let cid = match view_data.selected_row_data {
                        SelectedRow::Pilot(p) => p.cid,
                        SelectedRow::Controller(c) => c.cid,
                    };
                    if let Err(e) =
                        webbrowser::open(&format!("https://stats.vatsim.net/stats/{}", cid))
                    {
                        error!("Could not open web browser: {}", e);
                    }
                }
                _ => {}
            }
        }
    }

    // exit, restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    Ok(())
}

/// Helper function to create a centered rect using up certain percentage of the available rect `r`.
///
/// <https://github.com/fdehau/tui-rs/blob/a6b25a487786534205d818a76acb3989658ae58c/examples/popup.rs#L103-L128>
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ]
            .as_ref(),
        )
        .split(r);
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ]
            .as_ref(),
        )
        .split(popup_layout[1])[1]
}

/// Construct the text to be shown in the popup window.
fn popup_text(data: &SelectedRow) -> Paragraph {
    let text = match data {
        SelectedRow::Pilot(p) => {
            format!(
                "CID: {}\nServer: {}\nGround speed: {}\nTransponder: {}\nHeading: {}\nLogon time: {}",
                p.cid, p.server, p.groundspeed, p.transponder, p.heading, p.logon_time
            )
        }
        SelectedRow::Controller(c) => {
            format!(
                "CID: {}\nServer: {}\nFrequency: {}\nVisual range: {}\nLogon time: {}",
                c.cid, c.server, c.frequency, c.visual_range, c.logon_time
            )
        }
    };
    Paragraph::new(Text::from(text))
        .block(
            Block::default()
                .title("Additional information")
                .borders(Borders::ALL),
        )
        .wrap(Wrap { trim: false })
}
