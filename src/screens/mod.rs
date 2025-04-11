//! Rendering logic for various application screens.

use bevy::prelude::*;

use crate::backend::config::Config;

pub mod interface;
pub mod splash;

/// Sets up the application screens.
#[tracing::instrument(skip_all)]
pub fn plugin(app: &mut App) {
    app.enable_state_scoped_entities::<Screen>();
    app.init_state::<Screen>();

    app.add_systems(Startup, init_state);

    app.add_plugins((splash::plugin, interface::plugin));

    debug!("Finished loading");
}

fn init_state(mut next_state: ResMut<NextState<Screen>>, config: Res<Config>) {
    next_state.set(if !config.splash.skip {
        Screen::Splash
    } else {
        Screen::Interface
    });
}

/// Determines what screen should be shown.
#[derive(States, Debug, Hash, PartialEq, Eq, Clone, Copy, Default)]
pub enum Screen {
    #[default]
    Splash,
    Interface,
}
