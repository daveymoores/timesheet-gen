use crate::cli::RcHelpPrompt;
use crate::client_repositories::ClientRepositories;
use crate::link_builder;
use crate::repository::Repository;
use std::cell::RefCell;
use std::process;
use std::rc::Rc;

/// Creates and modifies the config file. Config does not directly hold the information
/// contained in the config file, but provides the various operations that can be
/// performed on it. The data is a stored within the Repository struct.

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
        client_repositories: Rc<RefCell<ClientRepositories>>,
        deserialized_config: Option<&mut Vec<ClientRepositories>>,
    ) {
        // get path for where to write the config file
        let config_path = crate::file_reader::get_filepath(crate::file_reader::get_home_path());
        let json = crate::file_reader::serialize_config(
            deserialized_config,
            Rc::clone(&client_repositories),
        )
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

    fn check_for_repo_in_buffer<'a>(
        self,
        deserialized_config: &'a mut Vec<ClientRepositories>,
        options: &'a Vec<Option<String>>,
    ) -> Result<Option<(&'a Repository, &'a ClientRepositories)>, Box<dyn std::error::Error>> {
        let mut temp_repository = Repository {
            repo_path: Option::from(options[0].as_ref().unwrap().to_string()),
            ..Default::default()
        };

        // get namespace of working repository
        temp_repository
            .find_git_path_from_directory_from()?
            .find_namespace_from_git_path()?;

        let namespace: String = temp_repository.namespace.unwrap();

        // check whether any clients contain the namespace
        let found_client_repositories = deserialized_config.iter().find(|client| {
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

        if let Some(ts_config) = found_client_repositories {
            let repository: Option<&Repository> = ts_config
                .repositories
                .as_ref()
                .unwrap()
                .into_iter()
                .find(|ts| ts.namespace.as_ref().unwrap() == &namespace);

            if let Some(ts) = repository {
                return Ok(Option::from((ts, ts_config)));
            }
        }

        Ok(None)
    }

    fn check_for_config_file(
        self,
        options: &Vec<Option<String>>,
        buffer: &mut String,
        repository: Rc<RefCell<Repository>>,
        client_repositories: Rc<RefCell<ClientRepositories>>,
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
        // and Repository state holds the data. Write this data to file.
        if buffer.is_empty() {
            client_repositories
                .borrow_mut()
                .set_values(repository.borrow())
                .exec_generate_timesheets_from_git_history()
                .compare_logs_and_set_timesheets();

            Config::write_to_config_file(client_repositories, None);
            return;
        }

        // ..if the there is an existing config file, check whether the (passed path) repository exists under any clients
        // if it does pass Repository values to Repository
        let mut deserialized_config: Vec<ClientRepositories> = serde_json::from_str(&buffer)
            .expect("Initialisation of ClientRepository struct from buffer failed");

        if let Some(ts_tuple) = self
            .check_for_repo_in_buffer(&mut deserialized_config, &options)
            .unwrap_or_else(|err| {
                eprintln!("Error trying to read from config file: {}", err);
                std::process::exit(exitcode::DATAERR);
            })
        {
            // if it exists, get the client + repos and the repo we're editing
            // and update the git log data based on all repositories
            let ts_clone = ts_tuple.clone();

            // ...and fetch a new batch of interaction data
            client_repositories
                .borrow_mut()
                .set_values_from_buffer(ts_clone.1)
                .exec_generate_timesheets_from_git_history()
                .compare_logs_and_set_timesheets();

            // set the working repo to the timesheet struct as it may be operated on
            repository.borrow_mut().set_values_from_buffer(ts_clone.0);
        } else {
            // if it doesn't, onboard them and check whether (passed path) repo
            // should exist under an existing client
            prompt
                .borrow_mut()
                .prompt_for_client_then_onboard(&mut deserialized_config)
                .unwrap_or_else(|err| {
                    eprintln!("Couldn't find client: {}", err);
                    std::process::exit(exitcode::CANTCREAT);
                });

            client_repositories
                .borrow_mut()
                .set_values(repository.borrow())
                .exec_generate_timesheets_from_git_history()
                .compare_logs_and_set_timesheets();

            Config::write_to_config_file(
                client_repositories,
                Option::from(&mut deserialized_config),
            );
        }
    }
}

pub trait Init {
    /// Generate a config file with user variables
    fn init(
        &self,
        options: Vec<Option<String>>,
        repository: Rc<RefCell<Repository>>,
        client_repositories: Rc<RefCell<ClientRepositories>>,
        prompt: RcHelpPrompt,
    );
}

impl Init for Config {
    fn init(
        &self,
        options: Vec<Option<String>>,
        repository: Rc<RefCell<Repository>>,
        client_repositories: Rc<RefCell<ClientRepositories>>,
        prompt: RcHelpPrompt,
    ) {
        // try to read config file. Write a new one if it doesn't exist
        let mut buffer = String::new();
        self.check_for_config_file(
            &options,
            &mut buffer,
            Rc::clone(&repository),
            client_repositories,
            prompt,
        );

        crate::help_prompt::HelpPrompt::repo_already_initialised();
    }
}

pub trait Make {
    /// Edit a day entry within the repository
    fn make(
        &self,
        options: Vec<Option<String>>,
        repository: Rc<RefCell<Repository>>,
        client_repositories: Rc<RefCell<ClientRepositories>>,
        prompt: RcHelpPrompt,
    );
}

impl Make for Config {
    #[tokio::main]
    async fn make(
        &self,
        options: Vec<Option<String>>,
        repository: Rc<RefCell<Repository>>,
        client_repositories: Rc<RefCell<ClientRepositories>>,
        prompt: RcHelpPrompt,
    ) {
        // try to read config file. Write a new one if it doesn't exist
        let mut buffer = String::new();
        self.check_for_config_file(
            &options,
            &mut buffer,
            Rc::clone(&repository),
            Rc::clone(&client_repositories),
            prompt.clone(),
        );

        // if buffer is not empty, then read client_repositories and generate the link
        if !buffer.is_empty() {
            // TODO - add_project_number should be on a per repo basis
            prompt
                .borrow_mut()
                .add_project_number()
                .unwrap_or_else(|err| {
                    eprintln!("Error parsing project number: {}", err);
                    std::process::exit(exitcode::CANTCREAT);
                });
            // generate timesheet-gen.io link using existing config
            link_builder::build_unique_uri(Rc::clone(&client_repositories), options)
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
        repository: Rc<RefCell<Repository>>,
        client_repositories: Rc<RefCell<ClientRepositories>>,
        prompt: RcHelpPrompt,
    );
}

impl Edit for Config {
    fn edit(
        &self,
        options: Vec<Option<String>>,
        repository: Rc<RefCell<Repository>>,
        client_repositories: Rc<RefCell<ClientRepositories>>,
        prompt: RcHelpPrompt,
    ) {
        // try to read config file. Write a new one if it doesn't exist
        let mut buffer = String::new();
        self.check_for_config_file(
            &options,
            &mut buffer,
            Rc::clone(&repository),
            Rc::clone(&client_repositories),
            prompt,
        );

        // if buffer is not empty, then read repository, edit a value and write to file
        if !buffer.is_empty() {
            // otherwise lets set the repository struct values
            // and fetch a new batch of interaction data
            repository
                .borrow_mut()
                .update_hours_on_month_day_entry(&options)
                .unwrap_or_else(|err| {
                    eprintln!("Error editing timesheet: {}", err);
                    process::exit(exitcode::DATAERR);
                });

            client_repositories
                .borrow_mut()
                .set_values(repository.borrow())
                .exec_generate_timesheets_from_git_history()
                .compare_logs_and_set_timesheets();

            // TODO give success message here
            Config::write_to_config_file(client_repositories, None);
        }
    }
}

pub trait RunMode {
    /// Specify a run mode
    fn run_mode(
        &self,
        options: Vec<Option<String>>,
        repository: Rc<RefCell<Repository>>,
        client_repositories: Rc<RefCell<ClientRepositories>>,
        prompt: RcHelpPrompt,
    );
}

impl RunMode for Config {
    fn run_mode(
        &self,
        options: Vec<Option<String>>,
        repository: Rc<RefCell<Repository>>,
        client_repositories: Rc<RefCell<ClientRepositories>>,
        prompt: RcHelpPrompt,
    ) {
        // try to read config file. Write a new one if it doesn't exist
        let mut buffer = String::new();
        self.check_for_config_file(
            &options,
            &mut buffer,
            Rc::clone(&repository),
            client_repositories,
            Rc::clone(&prompt),
        );

        // if buffer is not empty, then read repository, change the run-mode and write to file
        if !buffer.is_empty() {}
    }
}

#[cfg(test)]
mod tests {
    use crate::client_repositories::{Client, ClientRepositories, User};
    use crate::config::{Config, New};
    use crate::repository::Repository;

    #[test]
    fn it_checks_for_repo_in_buffer_and_returns_a_tuple() {
        let mut deserialized_config = vec![ClientRepositories {
            client: Option::from(Client {
                client_name: "alphabet".to_string(),
                client_address: "Spaghetti Way, USA".to_string(),
                client_contact_person: "John Smith".to_string(),
            }),
            user: Option::Some(User {
                name: "Jim Jones".to_string(),
                email: "jim@jones.com".to_string(),
            }),
            repositories: Option::from(vec![Repository {
                namespace: Option::from("timesheet-gen".to_string()),
                ..Default::default()
            }]),
        }];

        let options = vec![Option::from(".".to_string())];
        let config: Config = Config::new();
        let option = config
            .check_for_repo_in_buffer(&mut deserialized_config, &options)
            .unwrap();

        if let Some((repository, _client_repositories)) = option {
            assert_eq!(
                *repository.namespace.as_ref().unwrap(),
                "timesheet-gen".to_string()
            )
        }
    }
}
