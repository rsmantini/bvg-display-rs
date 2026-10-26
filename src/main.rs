//! CLI entry point and iced bootstrap.

pub mod api;
pub mod assets;
pub mod config;
pub mod fetch;
pub mod state;
pub mod state_updater;
pub mod ui;

use std::error::Error;

use clap::Parser;
use iced::{Settings, Task, window::Settings as WindowSettings};

use crate::{
    assets::Assets,
    state::State,
    state_updater::{Message, escape_subscription, update},
    ui::{retro::blink_tick_subscription, view},
};

#[derive(Parser)]
struct Cli {
    /// Path to the config file
    #[arg(long, default_value = "Config.local.toml")]
    config: std::path::PathBuf,
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    let (settings, state) = init(&cli)?;
    let fullscreen = state.config.fullscreen;

    iced::application(move || new(state.clone()), update, view)
        .window(iced::window::Settings {
            fullscreen,
            ..WindowSettings::default()
        })
        .subscription(escape_subscription)
        .subscription(blink_tick_subscription)
        .settings(settings)
        .run()
        .map_err(Into::into)
}

fn new(state: State) -> (State, Task<Message>) {
    (state, Task::done(Message::FetchDepartures))
}

fn init(cli: &Cli) -> Result<(Settings, State), Box<dyn Error>> {
    let config = config::Config::load(&cli.config).map_err(|e| -> Box<dyn Error> { e })?;
    let assets = Assets::load(&config.assets_dir)?;

    let settings = Settings {
        fonts: assets.fonts,
        ..Settings::default()
    };

    Ok((
        settings,
        State::new(config, assets.bvg_logo, assets.line_icons),
    ))
}
