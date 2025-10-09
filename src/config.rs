use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[derive(Default)]
pub enum DiffMode {
    #[default]
    SideBySide,
    Inline,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayConfig {
    #[serde(default)]
    pub diff_mode: DiffMode,

    #[serde(default = "default_context_lines")]
    pub context_lines: u32,

    #[serde(default = "default_context_expand_increment")]
    pub context_expand_increment: u32,

    #[serde(default = "default_horizontal_scroll_amount")]
    pub horizontal_scroll_amount: u32,

    #[serde(default = "default_syntax_theme")]
    pub syntax_theme: String,
}

fn default_context_lines() -> u32 {
    8
}

fn default_context_expand_increment() -> u32 {
    8
}

fn default_horizontal_scroll_amount() -> u32 {
    4
}

fn default_syntax_theme() -> String {
    "base16-ocean.dark".to_string()
}

impl Default for DisplayConfig {
    fn default() -> Self {
        Self {
            diff_mode: DiffMode::default(),
            context_lines: default_context_lines(),
            context_expand_increment: default_context_expand_increment(),
            horizontal_scroll_amount: default_horizontal_scroll_amount(),
            syntax_theme: default_syntax_theme(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    #[serde(default = "default_log_pane_width_ratio")]
    pub log_pane_width_ratio: f32,

    #[serde(default = "default_show_line_numbers")]
    pub show_line_numbers: bool,
}

fn default_log_pane_width_ratio() -> f32 {
    0.35
}

fn default_show_line_numbers() -> bool {
    true
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            log_pane_width_ratio: default_log_pane_width_ratio(),
            show_line_numbers: default_show_line_numbers(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub display: DisplayConfig,

    #[serde(default)]
    pub ui: UiConfig,
}

impl Config {
    /// Get the config file path
    pub fn config_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .context("Could not determine config directory")?
            .join("git-review");

        Ok(config_dir.join("config.toml"))
    }

    /// Load config from file, or return default if file doesn't exist
    pub fn load_or_default() -> Result<Self> {
        let path = Self::config_path()?;

        if !path.exists() {
            return Ok(Self::default());
        }

        let contents = fs::read_to_string(&path)
            .context(format!("Failed to read config file: {}", path.display()))?;

        let config: Config = toml::from_str(&contents)
            .context(format!("Failed to parse config file: {}", path.display()))?;

        Ok(config)
    }

    /// Save config to file
    pub fn save(&self) -> Result<()> {
        let path = Self::config_path()?;

        // Create config directory if it doesn't exist
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).context(format!(
                "Failed to create config directory: {}",
                parent.display()
            ))?;
        }

        let contents = toml::to_string_pretty(self).context("Failed to serialize config")?;

        fs::write(&path, contents)
            .context(format!("Failed to write config file: {}", path.display()))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.display.diff_mode, DiffMode::SideBySide);
        assert_eq!(config.display.context_lines, 8);
        assert_eq!(config.display.context_expand_increment, 8);
        assert_eq!(config.ui.log_pane_width_ratio, 0.35);
        assert!(config.ui.show_line_numbers);
    }

    #[test]
    fn test_serialize_deserialize() {
        let config = Config::default();
        let toml_str = toml::to_string(&config).unwrap();
        let deserialized: Config = toml::from_str(&toml_str).unwrap();

        assert_eq!(config.display.diff_mode, deserialized.display.diff_mode);
        assert_eq!(
            config.display.context_lines,
            deserialized.display.context_lines
        );
    }
}
