pub mod app;
pub mod backend;
pub mod cli;
pub mod errors;
pub mod events;
pub mod frontend;
pub mod screens;
pub mod utils;

use bevy::prelude::*;
use clap::Parser;
use cli::Cli;

fn main() {
    Cli::parse();
    App::new().add_plugins(app::plugin).run();
}
