use crate::timesheet::Timesheet;
use std::cell::RefCell;
use std::rc::Rc;

/// Creates and modifies the config file. Config does not directly hold the information
/// contained in the config file, but provides the various operations that can be
/// performed on it. The data is a stored within the Timesheet struct.

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Config {}

pub trait New {
    fn new() -> Self;
}

impl New for Config {
    fn new() -> Self {
        Config {}
    }
}

impl Config {}

pub trait Init {
    /// Generate a config file with user variables
    fn init(&self, options: Vec<Option<String>>, timesheet: Rc<RefCell<Timesheet>>);
}

impl Init for Config {
    fn init(&self, _options: Vec<Option<String>>, timesheet: Rc<RefCell<Timesheet>>) {
        let timesheet_clone = Rc::clone(&timesheet);
        // create buffer to read
        let mut buffer = String::new();
        // pass a prompt for if the config file doesn't exist
        let prompt = crate::help_prompt::HelpPrompt::new(timesheet_clone);

        crate::file_reader::read_data_from_config_file(&mut buffer, prompt).unwrap_or_else(|err| {
            eprintln!("Error initialising timesheet-gen: {}", err);
            std::process::exit(1);
        });

        println!("{:#?}", timesheet);
        // if the buffer is empty, there is no existing file and timesheet
        // state holds the data. Write this data to file.
        if buffer.is_empty() {
        } else {
            // otherwise parse the file data into timesheet state
        }
    }
}

pub trait Make {
    /// Edit a day entry within the timesheet
    fn make(&self, options: Vec<Option<String>>);
}

impl Make for Config {
    fn make(&self, _options: Vec<Option<String>>) {}
}

pub trait Edit {
    /// Generate a config file with user variables
    fn edit(&self, options: Vec<Option<String>>);
}

impl Edit for Config {
    fn edit(&self, _options: Vec<Option<String>>) {}
}

pub trait Remove {
    /// Remove an entry within the timesheet
    fn remove(&self, options: Vec<Option<String>>);
}

impl Remove for Config {
    fn remove(&self, _options: Vec<Option<String>>) {}
}

pub trait RunMode {
    /// Specify a run mode
    fn run_mode(&self, options: Vec<Option<String>>);
}

impl RunMode for Config {
    fn run_mode(&self, _options: Vec<Option<String>>) {}
}

#[cfg(test)]
mod tests {
    use super::*;
}
