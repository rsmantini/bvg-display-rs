use std::{fmt::Display, sync::Arc};

use chrono::{DateTime, Local};
use iced::{Font, widget::image};
use rapidhash::RapidHashMap;
use reqwest::Client;

use crate::{api::departures::Departure, config::Config};

#[derive(Clone)]
pub struct State {
    pub config: Arc<Config>,
    pub http_client: Client,
    pub departures: Departures,
    pub bvg_logo: image::Handle,
    pub retro_font: Font,
}

impl State {
    pub fn new(config: Config, bvg_logo: Vec<u8>) -> Self {
        let font_name: &'static str = Box::leak(Box::new(config.retro_ui.font.clone()));
        State {
            config: Arc::new(config),
            http_client: Client::new(),
            departures: Departures::default(),
            bvg_logo: image::Handle::from_bytes(bvg_logo),
            retro_font: Font::with_name(font_name),
        }
    }
}

impl Default for State {
    fn default() -> Self {
        Self::new(Config::default(), vec![])
    }
}

#[derive(Debug, Default, Clone)]
pub struct Departures {
    pub departures: RapidHashMap<String, Vec<DisplayDeparture>>,
    pub last_update: Option<DateTime<Local>>,
}

#[derive(Debug, Clone)]
pub struct DisplayDeparture {
    pub line: String,
    pub direction: String,
    pub when: String,
    pub cancelled: bool,
    pub typ: String,
    pub arriving: bool,
}

impl From<Departure> for DisplayDeparture {
    fn from(departure: Departure) -> Self {
        let now = chrono::Utc::now();
        let when = departure.when.unwrap_or(departure.planned_when);
        let delta = when - now;

        DisplayDeparture {
            line: departure.line.name,
            direction: departure.direction,
            when: delta.num_minutes().to_string(),
            cancelled: departure.cancelled,
            typ: departure.line.mode.as_german_str().to_string(),
            arriving: delta.num_minutes() <= 0,
        }
    }
}

impl Display for DisplayDeparture {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "line: {}, direction: {}, when: {}, cancelled: {}, typ: {}, arriving: {}",
            self.line, self.direction, self.when, self.cancelled, self.typ, self.arriving
        )
    }
}
