use std::{fmt::Display, sync::Arc};

use chrono::{DateTime, Local, Utc};
use iced::{Font, widget::image};
use rapidhash::RapidHashMap;

use crate::{
    api::{
        client::BvgClient,
        departures::{Departure, Mode, Product, Products},
    },
    assets::LineIcon,
    config::{Config, ModernUiConfig, RetroUiConfig, Ui},
    fetch::ArrivingTripLog,
};

#[derive(Clone)]
pub struct State {
    pub config: Arc<Config>,
    pub selected_ui: Ui,
    pub bvg_client: BvgClient,
    pub departures: Departures,
    pub bvg_logo: image::Handle,
    pub arriving_trips: ArrivingTripLog,
    pub retro_ui: RetroUiState,
    pub modern_ui: ModernUiState,
}

impl State {
    /// Create the state.
    ///
    /// `line_icons` maps a line name (e.g. `S41`, `U7`, `bus`, `tram`) to
    /// its preloaded image handle and original pixel dimensions and is only
    /// populated when the modern UI is selected.
    #[must_use]
    pub fn new(
        config: Config,
        bvg_logo: Vec<u8>,
        line_icons: RapidHashMap<String, LineIcon>,
    ) -> Self {
        let retro_ui = RetroUiState::new(&config.retro_ui);
        let modern_ui = ModernUiState::new(&config.modern_ui, line_icons);
        let selected_ui = config.ui;
        State {
            config: Arc::new(config),
            selected_ui,
            bvg_client: BvgClient::new(),
            departures: Departures::default(),
            bvg_logo: image::Handle::from_bytes(bvg_logo),
            arriving_trips: ArrivingTripLog::default(),
            retro_ui,
            modern_ui,
        }
    }

    pub fn ingest(&mut self, mut departures: Departures) {
        // Arriving trips sometimes linger for minutes in the API response
        // show them only once if this config is set
        if self.config.show_arriving_trips_only_once {
            self.arriving_trips.record_and_filter(&mut departures);
        }
        self.departures = departures;
    }
}

#[derive(Debug, Default, Clone)]
pub struct Departures {
    pub stops: RapidHashMap<Station, Vec<DisplayDeparture>>,
    pub last_update: Option<DateTime<Local>>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
pub struct Station {
    pub name: String,
    pub products: Option<Products>,
}

#[derive(Debug, Clone)]
pub struct DisplayDeparture {
    pub line: String,
    pub direction: String,
    pub when: String,
    pub cancelled: bool,
    pub mode: Mode,
    pub product: Product,
    pub arriving: bool,
    pub trip_id: String,
}

impl DisplayDeparture {
    #[must_use]
    pub fn from_api(departure: &Departure, now: DateTime<Utc>) -> Self {
        let when = departure.when.unwrap_or(departure.planned_when);
        let delta_minutes = std::cmp::max(0, (when - now).num_minutes());

        let mode = departure.line.mode;
        DisplayDeparture {
            line: departure.line.name.clone(),
            direction: strip_berlin(&departure.direction),
            when: delta_minutes.to_string(),
            cancelled: departure.cancelled,
            mode,
            product: departure.line.product.unwrap_or(Product::Unknown),
            arriving: delta_minutes <= 0,
            trip_id: departure.trip_id.clone(),
        }
    }
}

impl Display for DisplayDeparture {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "line: {}, direction: {}, when: {}, cancelled: {}, typ: {}, mode: {:?}, arriving: {}, trip_id: {}",
            self.line,
            self.direction,
            self.when,
            self.cancelled,
            self.mode.as_german_str(),
            self.mode,
            self.arriving,
            self.trip_id,
        )
    }
}

/// Create an iced `Font` from a font name, leaking the string to a
/// `'static` lifetime as required by `Font::with_name`.
fn font_named(name: &str) -> Font {
    let static_name: &'static str = Box::leak(Box::new(name.to_string()));
    Font::with_name(static_name)
}

#[derive(Clone)]
pub struct RetroUiState {
    pub blink_tick: u64,
    pub font: Font,
}

impl RetroUiState {
    #[must_use]
    pub fn new(config: &RetroUiConfig) -> Self {
        Self {
            blink_tick: 0,
            font: font_named(&config.font),
        }
    }
}

#[derive(Clone)]
pub struct ModernUiState {
    pub font: Font,
    /// Line icons keyed by line name (`S41`, `U7`, ...), plus the generic
    /// `bus` and `tram` icons.
    pub line_icons: RapidHashMap<String, LineIcon>,
}

impl ModernUiState {
    #[must_use]
    pub fn new(config: &ModernUiConfig, line_icons: RapidHashMap<String, LineIcon>) -> Self {
        Self {
            font: font_named(&config.font),
            line_icons,
        }
    }
}

#[must_use]
pub fn strip_berlin(name: &str) -> String {
    name.replace(" (Berlin)", "")
}
