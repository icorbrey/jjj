//! Rendering logic for various application screens.

use bevy::prelude::*;

pub mod interface;
pub mod splash;

/// Sets up the application screens.
#[tracing::instrument(skip_all)]
pub fn plugin(app: &mut App) {
    app.init_state::<Screen>();
    app.enable_state_scoped_entities::<Screen>();

    app.add_plugins((splash::plugin, interface::plugin));

    debug!("Finished loading");
}

/// Determines what screen should be shown.
#[derive(States, Debug, Hash, PartialEq, Eq, Clone, Copy, Default)]
pub enum Screen {
    #[default]
    Splash,
    Interface,
}
