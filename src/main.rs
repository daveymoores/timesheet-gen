use std::process;

mod cli;
mod config;
mod timesheet;

fn main() {
    let cli = cli::Cli::new();
    cli.run().unwrap_or_else(|err| {
        eprintln!("Problem parsing arguments: {}", err);
        process::exit(1);
    });
}
