use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version = version(), about)]
pub struct Cli;

const VERSION_MESSAGE: &str = concat!("v", env!("CARGO_PKG_VERSION"));

pub fn version() -> String {
    format!("{}\n\nAuthors: {}", VERSION_MESSAGE, clap::crate_authors!())
}
