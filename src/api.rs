use crate::models::{Status, V3ResponseData};
use anyhow::{anyhow, Result};
use log::debug;
use rand::seq::SliceRandom;
use reqwest::blocking::{Client, ClientBuilder};

/// Initial VATSIM API requests are made to this endpoint.
const STATUS_URL: &str = "https://status.vatsim.net/status.json";

/// API struct.
pub struct Vatsim {
    client: Client,
    v3_url: String,
}

impl Vatsim {
    /// New API struct instance.
    ///
    /// Makes the API call to the status endpoint to get the endpoint
    /// to make V3 API calls.
    pub fn new() -> Result<Self> {
        debug!("Creating VATSIM struct instance");
        let client = ClientBuilder::new()
            .user_agent("github.com/celeo/vatsim_online")
            .build()?;
        let url = Vatsim::get_v3_url(&client)?;
        Ok(Self {
            client,
            v3_url: url,
        })
    }

    /// Get the V3 URL by querying the status endpoint.
    fn get_v3_url(client: &Client) -> Result<String> {
        debug!("Getting V3 url from status page");
        let response = client.get(STATUS_URL).send()?;
        if !response.status().is_success() {
            return Err(anyhow!(
                "Got status {} from status endpoint",
                response.status().as_u16()
            ));
        }
        let data: Status = response.json()?;
        let url = data
            .data
            .v3
            .choose(&mut rand::thread_rng())
            .ok_or_else(|| anyhow!("No V3 URLs returned"))?
            .clone();
        debug!("V3 URL: {}", url);
        Ok(url)
    }

    /// Query the stored V3 endpoint.
    pub fn get_data(&self) -> Result<V3ResponseData> {
        debug!("Getting current data");
        let response = self.client.get(&self.v3_url).send()?;
        if !response.status().is_success() {
            return Err(anyhow!(
                "Got status {} from status endpoint",
                response.status().as_u16()
            ));
        }
        let mut data: V3ResponseData = response.json()?;
        data.pilots
            .sort_by(|a, b| a.callsign.partial_cmp(&b.callsign).unwrap());
        data.controllers
            .sort_by(|a, b| a.callsign.partial_cmp(&b.callsign).unwrap());
        Ok(data)
    }

    /// Look up a controller's rating in the data.
    ///
    /// Transforms number into name like "S1", "C3", "L1", etc.
    pub fn controller_rating_lookup(data: &V3ResponseData, rating: i8) -> String {
        data.ratings
            .iter()
            .find(|&item| item.id == rating)
            .map_or_else(|| String::from("?"), |item| item.short.clone())
    }
}
