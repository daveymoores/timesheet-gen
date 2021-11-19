use crate::link_builder;
use crate::timesheet::Timesheet;
use std::cell::RefCell;
use std::process;
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

impl Config {
    fn write_to_config_file(timesheet: Rc<RefCell<Timesheet>>) {
        let config_path = crate::file_reader::get_filepath(crate::file_reader::get_home_path());
        crate::file_reader::write_config_file(&timesheet.borrow(), config_path).unwrap_or_else(
            |err| {
                eprintln!("Error writing data to file: {}", err);
                std::process::exit(exitcode::CANTCREAT);
            },
        );
    }

    fn check_for_config_file(buffer: &mut String, timesheet: Rc<RefCell<Timesheet>>) {
        // pass a prompt for if the config file doesn't exist
        let prompt = crate::help_prompt::HelpPrompt::new(Rc::clone(&timesheet));

        crate::file_reader::read_data_from_config_file(buffer, prompt).unwrap_or_else(|err| {
            eprintln!("Error initialising timesheet-gen: {}", err);
            std::process::exit(exitcode::CANTCREAT);
        });

        // if the buffer is empty, there is no existing file and timesheet
        // state holds the data. Write this data to file.
        if buffer.is_empty() {
            Config::write_to_config_file(timesheet);
        } else {
            // otherwise lets set the timesheet struct values
            // and fetch a new batch of interaction data
            timesheet
                .borrow_mut()
                .set_values_from_buffer(&buffer)
                .exec_generate_timesheets_from_git_history();
        }
    }
}

pub trait Init {
    /// Generate a config file with user variables
    fn init(&self, options: Vec<Option<String>>, timesheet: Rc<RefCell<Timesheet>>);
}

impl Init for Config {
    fn init(&self, _options: Vec<Option<String>>, timesheet: Rc<RefCell<Timesheet>>) {
        // try to read config file. Write a new one if it doesn't exist
        let mut buffer = String::new();
        Config::check_for_config_file(&mut buffer, Rc::clone(&timesheet));

        if !buffer.is_empty() {
            println!(
                "timesheet-gen already initialised! \n\
    Try 'timesheet-gen make' to create your first timesheet \n\
    or 'timesheet-gen help' for more options."
            );
        }
    }
}

pub trait Make {
    /// Edit a day entry within the timesheet
    fn make(&self, options: Vec<Option<String>>, timesheet: Rc<RefCell<Timesheet>>);
}

impl Make for Config {
    #[tokio::main]
    async fn make(&self, options: Vec<Option<String>>, timesheet: Rc<RefCell<Timesheet>>) {
        // try to read config file. Write a new one if it doesn't exist
        let mut buffer = String::new();
        Config::check_for_config_file(&mut buffer, Rc::clone(&timesheet));

        // if buffer is not empty, then read timesheet and generate the link
        if !buffer.is_empty() {
            // set po number and approver

            // generate timesheet-gen.io link using existing config
            link_builder::build_unique_uri(Rc::clone(&timesheet), options)
                .await
                .unwrap_or_else(|err| {
                    eprintln!("Error building unique link: {}", err);
                    std::process::exit(exitcode::CANTCREAT);
                });
        }
    }
}

pub trait Edit {
    /// Generate a config file with user variables
    fn edit(&self, options: Vec<Option<String>>, timesheet: Rc<RefCell<Timesheet>>);
}

impl Edit for Config {
    fn edit(&self, options: Vec<Option<String>>, mut timesheet: Rc<RefCell<Timesheet>>) {
        // try to read config file. Write a new one if it doesn't exist
        let mut buffer = String::new();
        Config::check_for_config_file(&mut buffer, Rc::clone(&timesheet));

        // if buffer is not empty, then read timesheet, edit a value and write to file
        if !buffer.is_empty() {
            // otherwise lets set the timesheet struct values
            // and fetch a new batch of interaction data
            timesheet
                .borrow_mut()
                .update_hours_on_month_day_entry(&options)
                .unwrap_or_else(|err| {
                    eprintln!("Error editing timesheet: {}", err);
                    process::exit(exitcode::DATAERR);
                });

            // TODO give success message here
            Config::write_to_config_file(timesheet);
        }
    }
}

pub trait RunMode {
    /// Specify a run mode
    fn run_mode(&self, options: Vec<Option<String>>, timesheet: Rc<RefCell<Timesheet>>);
}

impl RunMode for Config {
    fn run_mode(&self, _options: Vec<Option<String>>, timesheet: Rc<RefCell<Timesheet>>) {
        // try to read config file. Write a new one if it doesn't exist
        let mut buffer = String::new();
        Config::check_for_config_file(&mut buffer, Rc::clone(&timesheet));

        // if buffer is not empty, then read timesheet, change the run-mode and write to file
        if !buffer.is_empty() {}
    }
}
