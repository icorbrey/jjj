use std::collections::HashMap;

use anyhow::anyhow;
use bevy::prelude::*;
use serde::{de::DeserializeOwned, Deserialize};

use super::JujutsuCli;

#[mutants::skip]
#[tracing::instrument(skip_all)]
pub fn plugin(app: &mut App) {
    trace!("Initializing plugin...");
    app.init_resource::<Config>();
    trace!("Plugin initialized.");
}

#[allow(unused)]
#[derive(Debug, Deserialize)]
struct ConfigRoot {
    #[serde(default = "default_table")]
    jjj: Config,
}

#[mutants::skip]
fn default_table<De: DeserializeOwned>() -> De {
    toml::de::from_str::<De>("").unwrap()
}

/// Application configuration.
#[derive(Debug, Deserialize, PartialEq, Eq, Resource)]
pub struct Config {
    /// Logging configuration.
    #[serde(default = "default_table")]
    pub log: LogConfig,

    /// Splash screen configuration.
    #[serde(default = "default_table")]
    pub splash: SplashConfig,

    /// Key bindings
    #[serde(default = "default_table")]
    pub keys: KeysConfig,
}

impl FromWorld for Config {
    fn from_world(world: &mut World) -> Self {
        let jj_cli = world.resource::<JujutsuCli>();

        let config_toml = jj_cli.list_config().unwrap();

        let root = toml::de::from_str::<ConfigRoot>(&config_toml)
            .map_err(|e| anyhow!("Failed to parse config: {e}"))
            .unwrap();

        info!("{:?}", &root.jjj);

        root.jjj
    }
}

impl Config {
    #[mutants::skip]
    #[allow(clippy::new_without_default)] // Default conflicts with FromWorld
    pub fn new() -> Self {
        default_table()
    }
}

/// Configuration for logging.
#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct LogConfig {
    /// How frequently jjj should check the status of the current repo.
    #[serde(default = "LogConfig::default_poll_interval_ms")]
    pub poll_interval_ms: u64,
}

impl LogConfig {
    #[mutants::skip]
    fn default_poll_interval_ms() -> u64 {
        1000
    }
}

/// Configuration for the splash screen.
#[derive(Debug, Deserialize, PartialEq, Eq)]
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
    #[mutants::skip]
    fn default_total_duration_ms() -> u64 {
        1950
    }

    #[mutants::skip]
    fn default_line_interval_ms() -> u64 {
        150
    }
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct KeysConfig {
    #[serde(default = "KeysConfig::default_normal")]
    pub normal: KeyMap,

    #[serde(default = "KeysConfig::default_command")]
    pub command: KeyMap,
}

impl KeysConfig {
    #[mutants::skip]
    fn default_normal() -> KeyMap {
        todo!()
    }

    #[mutants::skip]
    fn default_command() -> KeyMap {
        todo!()
    }
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct KeyMap(HashMap<String, KeyDefinition>);

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub enum KeyDefinition {
    Command(String),
    MinorMode(Box<KeyMap>),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loads_blank_config() {
        let mut app = App::new();

        app.insert_resource(JujutsuCli::mock(|_| Ok("".into())));
        app.init_resource::<Config>();

        assert_eq!(app.world().get_resource::<Config>(), Some(&default_table()));
    }

    #[test]
    fn loads_populated_config() {
        let mut app = App::new();

        app.insert_resource(JujutsuCli::mock(|_| {
            Ok(r#"
                ui.editor = "hx"
                jjj.log.poll_interval_ms = 123
                jjj.splash.skip = true
                jjj.splash.total_duration_ms = 4567
            "#
            .into())
        }));
        app.init_resource::<Config>();

        assert_eq!(
            app.world().get_resource::<Config>(),
            Some(&Config {
                log: LogConfig {
                    poll_interval_ms: 123,
                },
                splash: SplashConfig {
                    skip: true,
                    total_duration_ms: 4567,
                    ..default_table()
                },
            })
        );
    }
}
