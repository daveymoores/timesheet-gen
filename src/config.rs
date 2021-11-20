use crate::help_prompt::HelpPrompt;
use crate::link_builder;
use crate::timesheet::Timesheet;
use serde::Deserialize;
use std::cell::RefCell;
use std::process;
use std::rc::Rc;

/// Creates and modifies the config file. Config does not directly hold the information
/// contained in the config file, but provides the various operations that can be
/// performed on it. The data is a stored within the Timesheet struct.

#[derive(Debug, Deserialize)]
struct TimesheetConfig {
    client: String,
    repositories: Vec<Timesheet>,
}

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

    fn check_for_repo_in_buffer(
        self,
        deserialized_sheet: &mut Vec<TimesheetConfig>,
    ) -> Result<Option<&Timesheet>, Box<dyn std::error::Error>> {
        let mut temp_timesheet = Timesheet {
            git_path: Option::from(".".to_string()),
            ..Default::default()
        };

        temp_timesheet
            .find_git_path_from_directory_from()?
            .find_namespace_from_git_path()?;

        let namespace: String = temp_timesheet.namespace.unwrap();

        let found_timesheet = deserialized_sheet.iter().find(|client| {
            match client
                .repositories
                .iter()
                .find(|repository| repository.namespace.as_ref().unwrap() == &namespace)
            {
                Some(_) => true,
                None => false,
            }
        });

        if let Some(timesheet_config) = found_timesheet {
            let timesheet: Option<&Timesheet> = timesheet_config
                .repositories
                .iter()
                .find(|timesheet| timesheet.namespace.as_ref().unwrap() == &namespace);
            return Ok(timesheet);
        }

        Ok(None)
    }

    fn check_for_config_file(
        self,
        buffer: &mut String,
        timesheet: Rc<RefCell<Timesheet>>,
        prompt: HelpPrompt,
    ) {
        // pass a prompt for if the config file doesn't exist
        crate::file_reader::read_data_from_config_file(buffer, prompt).unwrap_or_else(|err| {
            eprintln!("Error initialising timesheet-gen: {}", err);
            std::process::exit(exitcode::CANTCREAT);
        });

        // if the buffer is empty, there is no existing file, user has been onboarded
        // and timesheet state holds the data. Write this data to file.
        if buffer.is_empty() {
            Config::write_to_config_file(timesheet);
            return;
        }

        // check whether this repository exists under any clients
        // if it does pass timesheet values to Timesheet
        let mut deserialized_sheet: Vec<TimesheetConfig> = serde_json::from_str(&buffer)
            .expect("Initialisation of timesheet struct from buffer failed");

        if let Some(ts) = self
            .check_for_repo_in_buffer(&mut deserialized_sheet)
            .unwrap_or_else(|err| {
                eprintln!("Error trying to read from config file: {}", err);
                std::process::exit(exitcode::DATAERR);
            })
        {
            let mut ts_clone = ts.clone();
            // otherwise lets set the timesheet struct values
            // and fetch a new batch of interaction data
            timesheet
                .borrow_mut()
                .set_values_from_buffer(&mut ts_clone)
                .exec_generate_timesheets_from_git_history();
        } else {
            //if it doesn't, onboard them
            println!("Looks like this repository hasn't been initialised yet. Would you like to add it to any of these existing clients?");
        }
    }
}

pub trait Init {
    /// Generate a config file with user variables
    fn init(
        &self,
        options: Vec<Option<String>>,
        timesheet: Rc<RefCell<Timesheet>>,
        prompt: HelpPrompt,
    );
}

impl Init for Config {
    fn init(
        &self,
        _options: Vec<Option<String>>,
        timesheet: Rc<RefCell<Timesheet>>,
        prompt: HelpPrompt,
    ) {
        // try to read config file. Write a new one if it doesn't exist
        let mut buffer = String::new();
        self.check_for_config_file(&mut buffer, Rc::clone(&timesheet), prompt);

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
    fn make(
        &self,
        options: Vec<Option<String>>,
        timesheet: Rc<RefCell<Timesheet>>,
        prompt: HelpPrompt,
    );
}

impl Make for Config {
    #[tokio::main]
    async fn make(
        &self,
        options: Vec<Option<String>>,
        timesheet: Rc<RefCell<Timesheet>>,
        prompt: HelpPrompt,
    ) {
        // try to read config file. Write a new one if it doesn't exist
        let mut buffer = String::new();
        self.check_for_config_file(&mut buffer, Rc::clone(&timesheet), prompt);

        // if buffer is not empty, then read timesheet and generate the link
        if !buffer.is_empty() {
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
    fn edit(
        &self,
        options: Vec<Option<String>>,
        timesheet: Rc<RefCell<Timesheet>>,
        prompt: HelpPrompt,
    );
}

impl Edit for Config {
    fn edit(
        &self,
        options: Vec<Option<String>>,
        timesheet: Rc<RefCell<Timesheet>>,
        prompt: HelpPrompt,
    ) {
        // try to read config file. Write a new one if it doesn't exist
        let mut buffer = String::new();
        self.check_for_config_file(&mut buffer, Rc::clone(&timesheet), prompt);

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
    fn run_mode(
        &self,
        options: Vec<Option<String>>,
        timesheet: Rc<RefCell<Timesheet>>,
        prompt: HelpPrompt,
    );
}

impl RunMode for Config {
    fn run_mode(
        &self,
        _options: Vec<Option<String>>,
        timesheet: Rc<RefCell<Timesheet>>,
        prompt: HelpPrompt,
    ) {
        // try to read config file. Write a new one if it doesn't exist
        let mut buffer = String::new();
        self.check_for_config_file(&mut buffer, Rc::clone(&timesheet), prompt);

        // if buffer is not empty, then read timesheet, change the run-mode and write to file
        if !buffer.is_empty() {}
    }
}

#[cfg(test)]
mod tests {
    use crate::config::{Config, New, TimesheetConfig};
    use crate::timesheet::Timesheet;

    #[test]
    fn it_checks_for_repo_in_buffer_and_returns_a_timesheet() {
        let mut deserialized_sheet = vec![TimesheetConfig {
            client: "alphabet".to_string(),
            repositories: vec![Timesheet {
                namespace: Option::from("timesheet-gen".to_string()),
                ..Default::default()
            }],
        }];
        let config: Config = Config::new();
        let timesheet = config
            .check_for_repo_in_buffer(&mut deserialized_sheet)
            .unwrap();

        assert_eq!(
            *timesheet.unwrap().namespace.as_ref().unwrap(),
            "timesheet-gen".to_string()
        )
    }
}
