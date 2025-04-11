#![feature(panic_update_hook)]

pub mod app;
pub mod backend;
pub mod cli;
pub mod errors;
pub mod events;
pub mod frontend;
pub mod logger;
pub mod screens;
pub mod utils;

use bevy::prelude::*;
use clap::Parser;
use cli::Cli;

fn main() {
    logger::install();

    Cli::parse();
    App::new().add_plugins(app::plugin).run();

    logger::dump();
}
