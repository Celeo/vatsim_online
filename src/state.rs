use crate::{
    api::Vatsim,
    models::{Controller, Pilot, V3ResponseData},
};
use tui::{
    style::{Color, Modifier, Style},
    text::Span,
    widgets::TableState,
};

/// Information from the V3 API data for the current interface view.
pub struct ViewData {
    pub title: &'static str,
    pub headers: Vec<&'static str>,
    pub data: Vec<Vec<String>>,
    pub show_popup: bool,
    pub selected_row_data: SelectedRow,
}

/// The data for a selected row in the interface.
#[derive(Debug, Clone)]
pub enum SelectedRow {
    Pilot(Pilot),
    Controller(Controller),
}

/// State of the interface.
pub struct App {
    tab_index: usize,
    table_states: [TableState; 2],
    data: V3ResponseData,
    show_popup: bool,
}

impl App {
    /// Create a new interface state from the VATSIM V3 data.
    pub fn new(data: V3ResponseData) -> Self {
        let mut state = TableState::default();
        state.select(Some(0));
        Self {
            tab_index: 0,
            table_states: [state.clone(), state.clone()],
            data,
            show_popup: false,
        }
    }

    /// Switch between the pilots and controllers data in the table.
    ///
    /// Effectively the "Tabs" component from tui, just manual.
    pub fn tab_over(&mut self) {
        self.tab_index = if self.tab_index == 0 { 1 } else { 0 };
        self.table_states[0].select(Some(0));
        self.table_states[1].select(Some(0));
    }

    /// Scroll down the table. Wrap-around supported.
    pub fn down(&mut self) {
        let sel = self.table_states[self.tab_index].selected().unwrap_or(0);
        let length = if self.tab_index == 0 {
            self.data.pilots.len()
        } else {
            self.data.controllers.len()
        };
        let next = if sel >= length - 1 { 0 } else { sel + 1 };
        self.table_states[self.tab_index].select(Some(next));
    }

    /// Scroll up the table. Wrap-around supported.
    pub fn up(&mut self) {
        let sel = self.table_states[self.tab_index].selected().unwrap_or(0);
        let length = if self.tab_index == 0 {
            self.data.pilots.len()
        } else {
            self.data.controllers.len()
        };
        let next = if sel == 0 { length - 1 } else { sel - 1 };
        self.table_states[self.tab_index].select(Some(next));
    }

    /// Scroll down 10 to the button. No wrap-around.
    pub fn page_down(&mut self) {
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

    /// Scroll up 10 to the top. No wrap-around.
    pub fn page_up(&mut self) {
        let sel = self.table_states[self.tab_index].selected().unwrap_or(0);
        let next = if sel <= 10 { 0 } else { sel - 10 };
        self.table_states[self.tab_index].select(Some(next));
    }

    /// Toggle the inspection popup on a table row.
    pub fn toggle_popup(&mut self, open: bool) {
        self.show_popup = open;
    }

    /// Get data from the selected "tab" for the table.
    fn get_tab_data(&self) -> Vec<Vec<String>> {
        if self.tab_index == 0 {
            self.data
                .pilots
                .iter()
                .map(|pilot| {
                    vec![
                        pilot.callsign.clone(),
                        pilot.name.clone(),
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
                        controller.callsign.clone(),
                        controller.name.clone(),
                        controller.frequency.clone(),
                        Vatsim::controller_rating_lookup(&self.data, controller.rating),
                    ]
                })
                .collect()
        }
    }

    /// Get table headers for the selected "tab".
    fn get_headers(&self) -> Vec<&'static str> {
        if self.tab_index == 0 {
            vec!["Callsign", "Name", "Aircraft", "Lat", "Long"]
        } else {
            vec!["Callsign", "Name", "Frequency", "Rating"]
        }
    }

    /// Get the table border title for the selected "tab".
    fn get_selected_title(&self) -> &'static str {
        if self.tab_index == 0 {
            "Pilots"
        } else {
            "Controllers"
        }
    }

    /// Get data to render in the interface.
    pub fn get_view_data(&self) -> ViewData {
        ViewData {
            title: self.get_selected_title(),
            headers: self.get_headers(),
            data: self.get_tab_data(),
            show_popup: self.show_popup,
            selected_row_data: self.get_selected_row_data(),
        }
    }

    /// Get the current "tab"'s `TableState` as a mutable reference.
    pub fn current_table_state(&mut self) -> &mut TableState {
        &mut self.table_states[self.tab_index]
    }

    /// Construct the "tab" selector.
    pub fn tab_header(&self) -> Vec<Span> {
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

    /// Get the currently selected row's data.
    fn get_selected_row_data(&self) -> SelectedRow {
        let row = self.table_states[self.tab_index].selected().unwrap_or(0);
        if self.tab_index == 0 {
            SelectedRow::Pilot(self.data.pilots.get(row).unwrap().clone())
        } else {
            SelectedRow::Controller(self.data.controllers.get(row).unwrap().clone())
        }
    }
}
