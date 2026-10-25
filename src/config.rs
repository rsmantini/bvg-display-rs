//! Application configuration loaded from `Config.toml`.

use std::path::{Path, PathBuf};

#[derive(Debug, Default, Clone, serde::Deserialize)]
pub struct Config {
    pub bvg_stops: Vec<String>,
    pub max_api_results: u32,
    pub max_departure_time_minutes: u32,
    pub assets_dir: PathBuf,
    pub fullscreen: bool,
    #[serde(default)]
    pub ui: Ui,
    pub retro_ui: RetroUiConfig,
    #[serde(default)]
    pub modern_ui: ModernUiConfig,
    #[serde(default)]
    pub show_arriving_trips_only_once: bool,
}

/// Which UI the app displays.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Ui {
    Retro,
    Modern,
}

impl Default for Ui {
    fn default() -> Self {
        Ui::Retro
    }
}

impl Config {
    pub fn load(path: &Path) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let contents = std::fs::read_to_string(path)
            .map_err(|e| format!("failed to open config file '{}': {e}", path.display()))?;
        let config: Config = toml::from_str(&contents)
            .map_err(|e| format!("failed to parse config file '{}': {e}", path.display()))?;
        Ok(config)
    }
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct RetroUiConfig {
    pub font: String,
    pub font_size: u32,
    pub max_display_rows: usize,
    pub reduce_canceled_text: bool,
    pub blink_speed_millis: u64,
}

impl Default for RetroUiConfig {
    fn default() -> Self {
        Self {
            font: "Advanced Dot Digital-7".to_string(),
            font_size: 20,
            max_display_rows: 10,
            reduce_canceled_text: false,
            blink_speed_millis: 700,
        }
    }
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct ModernUiConfig {
    pub font: String,
    pub font_size: u32,
    pub max_display_rows: usize,
}

impl Default for ModernUiConfig {
    fn default() -> Self {
        Self {
            font: "Fira Sans Black".to_string(),
            font_size: 24,
            max_display_rows: 10,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loads_config_toml() {
        let config = Config::load(Path::new("Config.toml")).expect("Config.toml should load");
        assert_eq!(config.bvg_stops, vec!["900040101"]);
    }
}
