use crate::cli::RcHelpPrompt;
use crate::link_builder;
use crate::timesheet::Timesheet;
use crate::timesheet_config::TimesheetConfig;
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
    /// Find and update client if sheet exists, otherwise write a new one
    fn write_to_config_file(
        timesheet: Rc<RefCell<Timesheet>>,
        deserialized_config: Option<Vec<TimesheetConfig>>,
    ) {
        let config_path = crate::file_reader::get_filepath(crate::file_reader::get_home_path());
        let json = crate::file_reader::serialize_config(deserialized_config, timesheet.borrow())
            .unwrap_or_else(|err| {
                eprintln!("Error serializing json: {}", err);
                std::process::exit(exitcode::CANTCREAT);
            });

        crate::file_reader::write_json_to_config_file(json, config_path).unwrap_or_else(|err| {
            eprintln!("Error writing data to file: {}", err);
            std::process::exit(exitcode::CANTCREAT);
        });

        process::exit(exitcode::OK)
    }

    fn check_for_repo_in_buffer(
        self,
        deserialized_config: &mut Vec<TimesheetConfig>,
    ) -> Result<Option<(&Timesheet, &TimesheetConfig)>, Box<dyn std::error::Error>> {
        let mut temp_timesheet = Timesheet {
            repo_path: Option::from(".".to_string()),
            ..Default::default()
        };

        temp_timesheet
            .find_git_path_from_directory_from()?
            .find_namespace_from_git_path()?;

        let namespace: String = temp_timesheet.namespace.unwrap();

        let found_timesheet_config = deserialized_config.iter().find(|client| {
            match client
                .repositories
                .as_ref()
                .unwrap()
                .iter()
                .find(|repository| repository.namespace.as_ref().unwrap() == &namespace)
            {
                Some(_) => true,
                None => false,
            }
        });

        if let Some(ts_config) = found_timesheet_config {
            let timesheet: Option<&Timesheet> = ts_config
                .repositories
                .as_ref()
                .unwrap()
                .into_iter()
                .find(|ts| ts.namespace.as_ref().unwrap() == &namespace);

            if let Some(ts) = timesheet {
                return Ok(Option::from((ts, ts_config)));
            }
        }

        Ok(None)
    }

    fn check_for_config_file(
        self,
        buffer: &mut String,
        timesheet: Rc<RefCell<Timesheet>>,
        timesheet_config: Rc<RefCell<TimesheetConfig>>,
        prompt: RcHelpPrompt,
    ) {
        // pass a prompt for if the config file doesn't exist
        crate::file_reader::read_data_from_config_file(buffer, prompt.clone()).unwrap_or_else(
            |err| {
                eprintln!("Error initialising timesheet-gen: {}", err);
                std::process::exit(exitcode::CANTCREAT);
            },
        );

        // if the buffer is empty, there is no existing file, user has been onboarded
        // and timesheet state holds the data. Write this data to file.
        if buffer.is_empty() {
            Config::write_to_config_file(timesheet, None);
            return;
        }

        // ..if the there is an existing config file, check whether the current repository exists under any clients
        // if it does pass timesheet values to Timesheet
        let mut deserialized_config: Vec<TimesheetConfig> = serde_json::from_str(&buffer)
            .expect("Initialisation of timesheet struct from buffer failed");

        if let Some(ts_tuple) = self
            .check_for_repo_in_buffer(&mut deserialized_config)
            .unwrap_or_else(|err| {
                eprintln!("Error trying to read from config file: {}", err);
                std::process::exit(exitcode::DATAERR);
            })
        {
            // if it exists, get the client + repos and the repo we're editing
            // and update the git log data based on both repositories
            let ts_clone = ts_tuple.clone();

            timesheet_config
                .borrow_mut()
                .set_values_from_buffer(ts_clone.1);
            // ...and fetch a new batch of interaction data
            timesheet
                .borrow_mut()
                .set_values_from_buffer(ts_clone.0)
                .exec_generate_timesheets_from_git_history(timesheet_config.clone());
        } else {
            // if it doesn't, onboard them and check whether current repo
            // should exist under an existing client
            prompt
                .borrow_mut()
                .prompt_for_client(deserialized_config)
                .unwrap_or_else(|err| {
                    eprintln!("Couldn't find client: {}", err);
                    std::process::exit(exitcode::CANTCREAT);
                });
        }
    }
}

pub trait Init {
    /// Generate a config file with user variables
    fn init(
        &self,
        options: Vec<Option<String>>,
        timesheet: Rc<RefCell<Timesheet>>,
        timesheet_config: Rc<RefCell<TimesheetConfig>>,
        prompt: RcHelpPrompt,
    );
}

impl Init for Config {
    fn init(
        &self,
        _options: Vec<Option<String>>,
        timesheet: Rc<RefCell<Timesheet>>,
        timesheet_config: Rc<RefCell<TimesheetConfig>>,
        prompt: RcHelpPrompt,
    ) {
        // try to read config file. Write a new one if it doesn't exist
        let mut buffer = String::new();
        self.check_for_config_file(&mut buffer, Rc::clone(&timesheet), timesheet_config, prompt);

        crate::help_prompt::HelpPrompt::repo_already_initialised();
    }
}

pub trait Make {
    /// Edit a day entry within the timesheet
    fn make(
        &self,
        options: Vec<Option<String>>,
        timesheet: Rc<RefCell<Timesheet>>,
        timesheet_config: Rc<RefCell<TimesheetConfig>>,
        prompt: RcHelpPrompt,
    );
}

impl Make for Config {
    #[tokio::main]
    async fn make(
        &self,
        options: Vec<Option<String>>,
        timesheet: Rc<RefCell<Timesheet>>,
        timesheet_config: Rc<RefCell<TimesheetConfig>>,
        prompt: RcHelpPrompt,
    ) {
        // try to read config file. Write a new one if it doesn't exist
        let mut buffer = String::new();
        self.check_for_config_file(
            &mut buffer,
            Rc::clone(&timesheet),
            timesheet_config,
            prompt.clone(),
        );

        // if buffer is not empty, then read timesheet and generate the link
        if !buffer.is_empty() {
            prompt
                .borrow_mut()
                .add_project_number()
                .unwrap_or_else(|err| {
                    eprintln!("Error parsing project number: {}", err);
                    std::process::exit(exitcode::CANTCREAT);
                });
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
        timesheet_config: Rc<RefCell<TimesheetConfig>>,
        prompt: RcHelpPrompt,
    );
}

impl Edit for Config {
    fn edit(
        &self,
        options: Vec<Option<String>>,
        timesheet: Rc<RefCell<Timesheet>>,
        timesheet_config: Rc<RefCell<TimesheetConfig>>,
        prompt: RcHelpPrompt,
    ) {
        // try to read config file. Write a new one if it doesn't exist
        let mut buffer = String::new();
        self.check_for_config_file(&mut buffer, Rc::clone(&timesheet), timesheet_config, prompt);

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
            Config::write_to_config_file(timesheet, None);
        }
    }
}

pub trait RunMode {
    /// Specify a run mode
    fn run_mode(
        &self,
        options: Vec<Option<String>>,
        timesheet: Rc<RefCell<Timesheet>>,
        timesheet_config: Rc<RefCell<TimesheetConfig>>,
        prompt: RcHelpPrompt,
    );
}

impl RunMode for Config {
    fn run_mode(
        &self,
        _options: Vec<Option<String>>,
        timesheet: Rc<RefCell<Timesheet>>,
        timesheet_config: Rc<RefCell<TimesheetConfig>>,
        prompt: RcHelpPrompt,
    ) {
        // try to read config file. Write a new one if it doesn't exist
        let mut buffer = String::new();
        self.check_for_config_file(
            &mut buffer,
            Rc::clone(&timesheet),
            timesheet_config,
            Rc::clone(&prompt),
        );

        // if buffer is not empty, then read timesheet, change the run-mode and write to file
        if !buffer.is_empty() {}
    }
}

#[cfg(test)]
mod tests {
    use crate::config::{Config, New};
    use crate::timesheet::Timesheet;
    use crate::timesheet_config::{Client, TimesheetConfig};

    #[test]
    fn it_checks_for_repo_in_buffer_and_returns_a_tuple() {
        let mut deserialized_config = vec![TimesheetConfig {
            client: Option::from(Client {
                client_name: "alphabet".to_string(),
                client_address: "Spaghetti Way, USA".to_string(),
                client_contact_person: "John Smith".to_string(),
            }),
            repositories: Option::from(vec![Timesheet {
                namespace: Option::from("timesheet-gen".to_string()),
                ..Default::default()
            }]),
        }];

        let config: Config = Config::new();
        let option = config
            .check_for_repo_in_buffer(&mut deserialized_config)
            .unwrap();

        if let Some((timesheet, _timesheet_config)) = option {
            assert_eq!(
                *timesheet.namespace.as_ref().unwrap(),
                "timesheet-gen".to_string()
            )
        }
    }
}
