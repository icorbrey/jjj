use anyhow::anyhow;
use bevy::prelude::*;
use serde::{de::DeserializeOwned, Deserialize};

use super::execute_jj_command;

pub fn plugin(app: &mut App) {
    trace!("Initializing plugin...");
    app.insert_resource(init_config());
    trace!("Plugin initialized.");
}

#[tracing::instrument(skip_all)]
fn init_config() -> Config {
    let config_toml = execute_jj_command(vec!["config", "list"])
        .map_err(|e| anyhow!("Failed to read config: {e}"))
        .unwrap();

    let root = toml::de::from_str::<ConfigRoot>(&config_toml)
        .map_err(|e| anyhow!("Failed to parse config: {e}"))
        .unwrap();

    info!("{:?}", &root.jjj);

    root.jjj
}

#[allow(unused)]
#[derive(Debug, Deserialize)]
struct ConfigRoot {
    #[serde(default = "default_table")]
    jjj: Config,
}

fn default_table<De: DeserializeOwned>() -> De {
    toml::de::from_str::<De>("").unwrap()
}

/// Application configuration.
#[derive(Debug, Deserialize, Reflect, Resource)]
#[serde(rename_all = "kebab-case")]
pub struct Config {
    /// Logging configuration.
    #[serde(default = "default_table")]
    pub log: LogConfig,

    /// Splash screen configuration.
    #[serde(default = "default_table")]
    pub splash: SplashConfig,
}

/// Configuration for logging.
#[derive(Debug, Deserialize, Reflect)]
#[serde(rename_all = "kebab-case")]
pub struct LogConfig {
    /// How frequently jjj should check the status of the current repo.
    #[serde(default = "LogConfig::default_poll_interval_ms")]
    pub poll_interval_ms: u64,
}

impl LogConfig {
    fn default_poll_interval_ms() -> u64 {
        1000
    }
}

/// Configuration for the splash screen.
#[derive(Debug, Deserialize, Reflect)]
#[serde(rename_all = "kebab-case")]
pub struct SplashConfig {
    /// Whether to skip the splash screen on startup.
    #[serde(default)]
    pub skip: bool,

    /// The total max duration the splash screen should be displayed for.
    #[serde(default = "SplashConfig::default_total_duration_ms")]
    pub total_duration_ms: u64,

    /// The duration between splash screen animation frames.
    #[serde(default = "SplashConfig::default_line_interval_ms")]
    pub line_interval_ms: u64,
}

impl SplashConfig {
    fn default_total_duration_ms() -> u64 {
        1950
    }

    fn default_line_interval_ms() -> u64 {
        150
    }
}
