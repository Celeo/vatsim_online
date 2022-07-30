use crate::models::{Status, V3ResponseData};
use anyhow::{anyhow, Result};
use log::debug;
use rand::seq::SliceRandom;
use reqwest::blocking::{Client, ClientBuilder};

const STATUS_URL: &str = "https://status.vatsim.net/status.json";

pub struct Vatsim {
    client: Client,
    v3_url: String,
}

impl Vatsim {
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
            .ok_or(anyhow!("No V3 URLs returned"))?
            .to_owned();
        debug!("V3 URL: {}", url);
        Ok(url)
    }

    pub fn get_data(&self) -> Result<V3ResponseData> {
        debug!("Getting current data");
        let response = self.client.get(&self.v3_url).send()?;
        if !response.status().is_success() {
            return Err(anyhow!(
                "Got status {} from status endpoint",
                response.status().as_u16()
            ));
        }
        let data = response.json()?;
        Ok(data)
    }

    pub fn controller_rating_lookup(data: &V3ResponseData, rating: i8) -> String {
        data.ratings
            .iter()
            .find(|&item| item.id == rating)
            .map(|item| item.short.clone())
            .unwrap_or(String::from("?"))
    }
}
