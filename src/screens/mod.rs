//! Rendering logic for various application screens.

use bevy::prelude::*;

pub mod interface;
pub mod splash;

/// Sets up the application screens.
pub fn plugin(app: &mut App) {
    app.init_state::<Screen>();
    app.enable_state_scoped_entities::<Screen>();

    app.add_plugins((splash::plugin, interface::plugin));
}

/// Determines what screen should be shown.
#[derive(States, Debug, Hash, PartialEq, Eq, Clone, Copy, Default)]
pub enum Screen {
    #[default]
    Splash,
    Interface,
}
