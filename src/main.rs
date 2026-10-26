pub mod api;
pub mod config;
pub mod state;
pub mod state_updater;
pub mod ui;

use std::{borrow::Cow, error::Error};

use clap::Parser;
use iced::{Element, Settings, Task, widget::image, window::Settings as WindowSettings};
use rapidhash::{HashMapExt, RapidHashMap};

use crate::{
    config::Ui,
    state::{LineIcon, State},
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

/// Pick the view based on the currently selected UI and make a click
/// anywhere on the screen toggle between the UIs.
fn view(state: &State) -> Element<'_, Message> {
    let content = if state.selected_ui == Ui::Modern {
        modern_view(state)
    } else {
        retro_view(state)
    };

    iced::widget::MouseArea::new(content)
        .on_press(Message::ToggleUi)
        .into()
}

fn init(cli: &Cli) -> Result<(Settings, State), Box<dyn Error>> {
    let config = config::Config::load(&cli.config).map_err(|e| -> Box<dyn Error> { e })?;

    let fonts = load_fonts(&config.assets_dir.join("fonts"))?;
    let bvg_logo = std::fs::read(config.assets_dir.join("images/bvg-logo.svg.png"))?;
    let line_icons = load_line_icons(&config.assets_dir)?;

    let settings = Settings {
        fonts,
        ..Settings::default()
    };

    Ok((settings, State::new(config, bvg_logo, line_icons)))
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

/// Load every line icon under `assets_dir/line_icons` (S-Bahn, U-Bahn,
/// tram and bus) into a map keyed by the line name: `S41.png` becomes
/// `S41`, `U7.png` becomes `U7`, and the generic `bus.png` / `tram.png`
/// become `bus` / `tram`. Each entry holds the image handle together with
/// the original pixel dimensions, needed to scale the icon at a fixed
/// height while keeping its aspect ratio.
fn load_line_icons(
    assets_dir: &std::path::Path,
) -> Result<RapidHashMap<String, LineIcon>, Box<dyn Error>> {
    let dir = assets_dir.join("line_icons");
    let mut line_icons = RapidHashMap::new();
    for entry in std::fs::read_dir(&dir)? {
        let path = entry?.path();
        if path.extension().is_some_and(|e| e == "png") {
            let key = path
                .file_stem()
                .ok_or_else(|| format!("icon file without a stem: {:?}", path))?
                .to_string_lossy()
                .into_owned();
            println!("reading line icon file {:?}", path);
            let bytes = std::fs::read(&path)?;
            let (width, height) = png_size(&bytes).unwrap_or((1, 1));
            line_icons.insert(
                key,
                LineIcon {
                    handle: image::Handle::from_bytes(bytes),
                    width,
                    height,
                },
            );
        }
    }
    Ok(line_icons)
}

/// Read the original pixel dimensions from a PNG's `IHDR` chunk (the
/// big-endian width and height at bytes 16..24) without decoding the image.
fn png_size(bytes: &[u8]) -> Option<(u32, u32)> {
    const SIGNATURE: [u8; 8] = [137, 80, 78, 71, 13, 10, 26, 10];
    if !bytes.starts_with(&SIGNATURE) {
        return None;
    }
    if &bytes[12..16] != b"IHDR" {
        return None;
    }
    let width = u32::from_be_bytes(bytes[16..20].try_into().ok()?);
    let height = u32::from_be_bytes(bytes[20..24].try_into().ok()?);
    Some((width, height))
}
