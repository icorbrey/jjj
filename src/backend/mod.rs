//! Handles interactions with underlying JJ repositories.

use std::{fmt::Display, process::Command};

use anyhow::{anyhow, Result};
use bevy::prelude::*;
use revisions::ChangeId;

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
    pub fn abandon(&self, change_id: ChangeId) -> Result<()> {
        self.execute(vec!["abandon", change_id.0.as_str()])
            .map_err(|_| anyhow!("Couldn't abandon commit `{change_id}`"))?;
        Ok(())
    }

    pub fn edit(&self, change_id: ChangeId) -> Result<()> {
        self.execute(vec!["edit", change_id.0.as_str()])
            .map_err(|_| anyhow!("Couldn't select commit `{change_id}`"))?;
        Ok(())
    }

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

    pub fn new(&self, change_id: ChangeId) -> Result<()> {
        self.execute(vec!["new", change_id.0.as_str()])
            .map_err(|_| anyhow!("Couldn't create new commit after `{change_id}`"))?;
        Ok(())
    }

    pub fn undo(&self) -> Result<()> {
        self.execute(vec!["undo"])
            .map_err(|_| anyhow!("Couldn't undo"))?;
        Ok(())
    }

    pub fn squash(&self, change_id: ChangeId) -> Result<()> {
        self.execute(vec!["squash", "-r", change_id.0.as_str()])
            .map_err(|_| anyhow!("Couldn't squash commit `{change_id}`"))?;
        Ok(())
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

        let output = Command::new("jj").args(_args).output()?;
        trace!("{:?}", output);

        if output.status.success() {
            Ok(String::from_utf8(output.stdout)?)
        } else {
            Err(anyhow!("{}", String::from_utf8(output.stderr)?))
        }
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
