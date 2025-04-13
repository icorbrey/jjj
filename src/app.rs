//! Application initialization.

use std::time::Duration;

use bevy::{app::ScheduleRunnerPlugin, prelude::*, state::app::StatesPlugin};
use bevy_ratatui::RatatuiPlugins;

use crate::{backend, events, frontend, screens};

#[tracing::instrument(skip_all)]
pub fn plugin(app: &mut App) {
    trace!("Initializing plugin...");

    app.configure_sets(
        Update,
        (
            AppSet::TickTimers,
            AppSet::RecordInput,
            AppSet::Update,
            AppSet::PrepFrames,
            AppSet::Render,
        ),
    );

    let frame_time = Duration::from_secs_f32(1.0 / 60.0);

    app.add_plugins((
        MinimalPlugins.set(ScheduleRunnerPlugin::run_loop(frame_time)),
        RatatuiPlugins::default(),
        StatesPlugin,
    ));

    app.add_plugins((
        events::plugin,
        screens::plugin,
        frontend::plugin,
        backend::plugin,
    ));

    trace!("Plugin initialized.");
}

/// Coordinates systems running under the [`Update`] schedule.
#[derive(SystemSet, Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub enum AppSet {
    TickTimers,
    RecordInput,
    Update,
    PrepFrames,
    Render,
}
