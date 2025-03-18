//! Rendering logic for the general interface.

use bevy::prelude::*;

use super::Screen;

pub fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::Interface), quit);
}

fn quit(mut exit: EventWriter<AppExit>) {
    exit.send_default();
}
