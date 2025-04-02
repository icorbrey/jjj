//! Application initialization.

use std::time::Duration;

use bevy::{app::ScheduleRunnerPlugin, prelude::*, state::app::StatesPlugin};
use bevy_ratatui::{event::KeyEvent, RatatuiPlugins};
use crossterm::event::KeyCode;

use crate::{
    frontend::{change_buffer, status_line},
    screens,
};

pub fn plugin(app: &mut App) {
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

    app.add_plugins((screens::plugin, status_line::plugin, change_buffer::plugin));

    app.add_systems(Update, read_quit.in_set(AppSet::RecordInput));
}

/// Quits the application if the user presses Q.
fn read_quit(mut events: EventReader<KeyEvent>, mut exit: EventWriter<AppExit>) {
    for event in events.read() {
        if let KeyCode::Char('q') = event.code {
            exit.send_default();
        }
    }
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
