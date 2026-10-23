//! Application configuration loaded from `Config.toml`.

use std::path::{Path, PathBuf};

#[derive(Debug, Default, Clone, serde::Deserialize)]
pub struct Config {
    pub bvg_stops: Vec<String>,
    pub num_results: u32,
    pub assets_dir: PathBuf,
    pub fullscreen: bool,
    pub retro_ui: RetroUi,
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

#[derive(Debug, Default, Clone, serde::Deserialize)]
pub struct RetroUi {
    pub font: String,
    pub font_size: u32,
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
