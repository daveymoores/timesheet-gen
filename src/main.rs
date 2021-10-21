use std::process;

mod cli;
mod config;
mod date_parser;
mod file_reader;
mod help_prompt;
mod timesheet;
mod utils;

fn main() {
    let cli = cli::Cli::new();
    cli.run().unwrap_or_else(|err| {
        eprintln!("Problem parsing arguments: {}", err);
        process::exit(1);
    });
}
