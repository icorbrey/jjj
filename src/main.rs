pub mod app;
pub mod screens;

use bevy::prelude::*;

fn main() {
    App::new().add_plugins(app::plugin).run();
}
