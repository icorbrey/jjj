pub mod app;
pub mod backend;
pub mod cli;
pub mod frontend;
pub mod screens;

use bevy::prelude::*;
use clap::Parser;
use cli::Cli;

fn main() {
    Cli::parse();
    App::new().add_plugins(app::plugin).run();
}
