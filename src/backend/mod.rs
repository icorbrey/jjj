//! Handles interactions with underlying JJ repositories.

use std::process::Command;

use anyhow::Result;
use bevy::prelude::*;

pub mod log;
pub mod poll;
pub mod revisions;

#[tracing::instrument(skip_all)]
pub fn plugin(app: &mut App) {
    app.add_plugins((log::plugin, poll::plugin));

    debug!("Finished loading");
}

pub(super) fn execute_jj_command(args: Vec<&str>) -> Result<String> {
    let mut _args = vec!["--color", "never"];
    _args.append(&mut args.clone());
    Ok(String::from_utf8(
        Command::new("jj").args(_args).output()?.stdout,
    )?)
}
