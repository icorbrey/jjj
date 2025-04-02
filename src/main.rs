pub mod app;
pub mod backend;
pub mod frontend;
pub mod screens;

use bevy::prelude::*;

fn main() {
    App::new().add_plugins(app::plugin).run();
}
