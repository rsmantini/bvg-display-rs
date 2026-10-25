use std::{fmt::Display, sync::Arc};

use chrono::{DateTime, Local};
use iced::{Font, widget::image};
use rapidhash::RapidHashMap;
use reqwest::Client;

use crate::{
    api::departures::Departure,
    config::{Config, ModernUiConfig, RetroUiConfig},
};

#[derive(Clone)]
pub struct State {
    pub config: Arc<Config>,
    pub http_client: Client,
    pub departures: Departures,
    pub bvg_logo: image::Handle,
    pub displayed_arriving_trips: RapidHashMap<String, i64>,
    pub retro_ui: RetroUiState,
    pub modern_ui: ModernUiState,
}

impl State {
    pub fn new(config: Config, bvg_logo: Vec<u8>) -> Self {
        let retro_ui = RetroUiState::new(&config.retro_ui);
        let modern_ui = ModernUiState::new(&config.modern_ui);
        State {
            config: Arc::new(config),
            http_client: Client::new(),
            departures: Departures::default(),
            bvg_logo: image::Handle::from_bytes(bvg_logo),
            displayed_arriving_trips: RapidHashMap::default(),
            retro_ui,
            modern_ui,
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
    pub vehicle_type: String,
    pub arriving: bool,
    pub trip_id: String,
}

impl From<Departure> for DisplayDeparture {
    fn from(departure: Departure) -> Self {
        let now = chrono::Utc::now();
        let when = departure.when.unwrap_or(departure.planned_when);
        let delta = when - now;

        DisplayDeparture {
            line: departure.line.name,
            direction: departure.direction.replace(" (Berlin)", ""),
            when: delta.num_minutes().to_string(),
            cancelled: departure.cancelled,
            vehicle_type: departure.line.mode.as_german_str().to_string(),
            arriving: delta.num_minutes() <= 0,
            trip_id: departure.trip_id,
        }
    }
}

impl Display for DisplayDeparture {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "line: {}, direction: {}, when: {}, cancelled: {}, typ: {}, arriving: {}, trip_id: {}",
            self.line,
            self.direction,
            self.when,
            self.cancelled,
            self.vehicle_type,
            self.arriving,
            self.trip_id,
        )
    }
}

#[derive(Clone)]
pub struct RetroUiState {
    pub blink_tick: u64,
    pub font: Font,
}

impl RetroUiState {
    pub fn new(config: &RetroUiConfig) -> Self {
        let font_name: &'static str = Box::leak(Box::new(config.font.clone()));
        let font = Font::with_name(font_name);
        Self {
            blink_tick: 0,
            font,
        }
    }
}

#[derive(Clone)]
pub struct ModernUiState {
    pub font: Font,
}

impl ModernUiState {
    pub fn new(config: &ModernUiConfig) -> Self {
        let font_name: &'static str = Box::leak(Box::new(config.font.clone()));
        let font = Font::with_name(font_name);
        Self { font }
    }
}
