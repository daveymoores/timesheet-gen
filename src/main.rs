use std::process;
mod interface;
mod helpers;
mod config;
mod utils;
mod data;
mod db;

fn main() {
    let cli = interface::cli::Cli::new();
    cli.run().unwrap_or_else(|err| {
        eprintln!("Problem parsing arguments: {}", err);
        process::exit(1);
    });
}
