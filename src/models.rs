use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct StatusData {
    pub v3: Vec<String>,
    pub transceivers: Vec<String>,
    pub servers: Vec<String>,
    pub servers_sweatbox: Vec<String>,
    pub servers_all: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Status {
    pub data: StatusData,
    pub user: Vec<String>,
    pub metar: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct FlightPlan {
    pub flight_rules: String,
    pub aircraft: String,
    pub aircraft_faa: String,
    pub aircraft_short: String,
    pub departure: String,
    pub arrival: String,
    pub alternate: String,
    pub cruise_tas: String,
    pub altitude: String,
    pub deptime: String,
    pub enroute_time: String,
    pub fuel_time: String,
    pub remarks: String,
    pub route: String,
    pub revision_id: i64,
    pub assigned_transponder: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Pilot {
    pub cid: i64,
    pub name: String,
    pub callsign: String,
    pub server: String,
    pub pilot_rating: i8,
    pub latitude: f64,
    pub longitude: f64,
    pub altitude: i64,
    pub groundspeed: i64,
    pub transponder: String,
    pub heading: i64,
    pub qnh_i_hg: f64,
    pub qnh_mb: i64,
    pub flight_plan: Option<FlightPlan>,
    pub logon_time: String,
    pub last_updated: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Controller {
    pub cid: i64,
    pub name: String,
    pub callsign: String,
    pub frequency: String,
    pub facility: i64,
    pub rating: i8,
    pub server: String,
    pub visual_range: i64,
    pub text_atis: Option<Vec<String>>,
    pub last_updated: String,
    pub logon_time: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GeneralData {
    pub version: i64,
    pub reload: i64,
    pub update: String,
    pub update_timestamp: String,
    pub connected_clients: i64,
    pub unique_users: i64,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ReferenceItem {
    pub id: i8,
    pub short: String,
    pub long: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct V3ResponseData {
    pub general: GeneralData,
    pub pilots: Vec<Pilot>,
    pub controllers: Vec<Controller>,
    // atis: Vec<?>,
    // servers: Vec<?>,
    pub facilities: Vec<ReferenceItem>,
    pub ratings: Vec<ReferenceItem>,
    // pilot_ratings: Vec<?>,
}
