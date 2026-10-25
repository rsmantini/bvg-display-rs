pub mod api;
pub mod config;
pub mod state;
pub mod state_updater;
pub mod ui;

use std::{borrow::Cow, error::Error};

use clap::Parser;
use iced::{Settings, Task, window::Settings as WindowSettings};

use crate::{
    config::Ui,
    state::State,
    state_updater::{Message, escape_subscription, update},
    ui::modern::modern_view,
    ui::retro::{blink_tick_subscription, retro_view},
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

    // Pick the view function based on the configured UI.
    let view = if state.config.ui == Ui::Modern {
        modern_view
    } else {
        retro_view
    };

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

    let fonts = load_fonts(&config.assets_dir.join("fonts"))?;
    let bvg_logo = std::fs::read(config.assets_dir.join("images/bvg-logo.svg.png"))?;

    let settings = Settings {
        fonts,
        ..Settings::default()
    };

    Ok((settings, State::new(config, bvg_logo)))
}

/// Recursively load every `.ttf` font found under `dir`.
fn load_fonts(dir: &std::path::Path) -> Result<Vec<Cow<'static, [u8]>>, Box<dyn Error>> {
    let mut fonts: Vec<Cow<'static, [u8]>> = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(path) = stack.pop() {
        if !path.is_dir() {
            continue;
        }
        for entry in std::fs::read_dir(&path)? {
            let entry = entry?;
            let p = entry.path();
            if p.is_dir() {
                stack.push(p);
            } else if p.extension().is_some_and(|e| e == "ttf") {
                println!("reading font file {:?}", p);
                fonts.push(Cow::Owned(std::fs::read(p)?));
            }
        }
    }
    Ok(fonts)
}
