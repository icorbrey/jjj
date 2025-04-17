//! Handles interactions with underlying JJ repositories.

use std::{fmt::Display, process::Command};

use anyhow::{anyhow, Result};
use bevy::prelude::*;

pub mod config;
pub mod log;
pub mod poll;
pub mod revisions;

#[mutants::skip]
#[tracing::instrument(skip_all)]
pub fn plugin(app: &mut App) {
    trace!("Initializing plugin...");
    app.init_resource::<JujutsuCli>();
    app.add_plugins((log::plugin, poll::plugin, config::plugin));
    trace!("Plugin initialized.");
}

#[derive(Deref, Resource)]
pub struct JujutsuCli(Box<dyn JujutsuShell + Send + Sync>);

impl JujutsuCli {
    pub fn list_config(&self) -> Result<String> {
        self.execute(vec!["config", "list"])
            .map_err(|e| anyhow!("Failed to read config: {e}"))
    }

    pub fn log(
        &self,
        revset: impl AsRef<str> + Display,
        template: impl AsRef<str> + Display,
    ) -> Result<String> {
        self.execute(vec!["log", "-r", revset.as_ref(), "-T", template.as_ref()])
            .map_err(|_| anyhow!("Couldn't read log for revset `{revset}`"))
    }

    #[cfg(test)]
    pub fn mock<F>(f: F) -> Self
    where
        F: Fn(Vec<&str>) -> Result<String> + Send + Sync + 'static,
    {
        Self(Box::new(MockedJujutsuShell(f)))
    }
}

impl Default for JujutsuCli {
    fn default() -> Self {
        Self(Box::new(RealJujutsuShell))
    }
}

pub trait JujutsuShell {
    fn execute(&self, args: Vec<&str>) -> Result<String>;
}

struct RealJujutsuShell;

impl JujutsuShell for RealJujutsuShell {
    #[tracing::instrument(skip_all)]
    fn execute(&self, args: Vec<&str>) -> Result<String> {
        let mut _args = vec!["--color", "never"];
        _args.append(&mut args.clone());
        trace!("jj {}", _args.join(" "));

        let output = String::from_utf8(Command::new("jj").args(_args).output()?.stdout)?;
        trace!(output);
        Ok(output)
    }
}

#[cfg(test)]
struct MockedJujutsuShell<F>(F)
where
    F: Fn(Vec<&str>) -> Result<String>;

#[cfg(test)]
impl<F> JujutsuShell for MockedJujutsuShell<F>
where
    F: Fn(Vec<&str>) -> Result<String>,
{
    fn execute(&self, args: Vec<&str>) -> Result<String> {
        (self.0)(args)
    }
}
