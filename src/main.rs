pub mod api;
pub mod config;
pub mod state;
pub mod state_updater;
pub mod ui;

use std::{borrow::Cow, error::Error};

use iced::{Settings, Task};

use crate::{
    state::State,
    state_updater::{Message, update},
    ui::retro::retro_view,
};

const MATRIX_FONT: &str = "Advanced Dot Digital-7";

fn main() -> Result<(), Box<dyn Error>> {
    let matrix_font =
        std::fs::read("assets/fonts/advanced-dot-digital-7/advanced_dot_digital-7.ttf")?;

    let settings = Settings {
        fonts: vec![Cow::Owned(matrix_font)],
        ..Settings::default()
    };

    iced::application(new, update, retro_view)
        .settings(settings)
        .run()
        .map_err(Into::into)
}

fn new() -> (State, Task<Message>) {
    let config = config::Config::load(std::path::Path::new("Config.toml")).unwrap();
    let bvg_logo = std::fs::read("assets/images/bvg-logo.svg.png").unwrap();
    let state = State::new(config, bvg_logo);
    (state, Task::done(Message::FetchDepartures))
}
