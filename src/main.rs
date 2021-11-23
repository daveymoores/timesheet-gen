use std::process;

mod cli;
mod config;
mod date_parser;
mod db;
mod file_reader;
mod help_prompt;
mod link_builder;
mod timesheet;
mod timesheet_config;
mod utils;

fn main() {
    let cli = cli::Cli::new();
    cli.run().unwrap_or_else(|err| {
        eprintln!("Problem parsing arguments: {}", err);
        process::exit(1);
    });
}
