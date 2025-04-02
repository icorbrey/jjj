//! Application initialization.

use std::time::Duration;

use bevy::{app::ScheduleRunnerPlugin, prelude::*, state::app::StatesPlugin};
use bevy_ratatui::{event::KeyEvent, RatatuiPlugins};
use crossterm::event::{KeyCode, KeyEventKind};
use rand::Rng;

use crate::{backend, errors::ErrorEvent, events, frontend, screens};

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

    app.add_plugins((
        events::plugin,
        screens::plugin,
        frontend::plugin,
        backend::plugin,
    ));

    app.add_systems(
        Update,
        (read_quit, simulate_error).in_set(AppSet::RecordInput),
    );
}

/// Quits the application if the user presses Q.
fn read_quit(mut ev_keypresses: EventReader<KeyEvent>, mut exit: EventWriter<AppExit>) {
    for keypress in ev_keypresses.read() {
        if let KeyCode::Char('q') = keypress.code {
            exit.send_default();
        }
    }
}

fn simulate_error(mut ev_keypresses: EventReader<KeyEvent>, mut ev_error: EventWriter<ErrorEvent>) {
    for keypress in ev_keypresses.read() {
        if keypress.code == KeyCode::Char('e') && keypress.kind == KeyEventKind::Press {
            let mut rng = rand::rng();
            let len = rng.random_range(1..100);
            ev_error.send(ErrorEvent::from(format!(
                "Simulated error! {}",
                rng.random_iter::<u8>()
                    .take(len)
                    .map(|i| i.to_string())
                    .collect::<String>()
            )));
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
