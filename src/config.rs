use crate::cli::RcHelpPrompt;
use crate::client_repositories::ClientRepositories;
use crate::link_builder;
use crate::repository::Repository;
use crate::utils::exit_process;
use std::cell::{Ref, RefCell, RefMut};
use std::ops::Deref;
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
    fn update_client_repositories(
        new_client_repos: &mut Vec<ClientRepositories>,
        deserialized_config: Vec<ClientRepositories>,
        old_client_repos: Ref<Vec<ClientRepositories>>,
    ) {
        let client_id = old_client_repos[0].get_client_id();

        for i in 0..deserialized_config.len() {
            if deserialized_config[i].get_client_id() == client_id {
                new_client_repos.push(old_client_repos.deref()[0].clone())
            } else {
                new_client_repos.push(deserialized_config[i].clone())
            }
        }
    }

    fn push_found_values_into_rcs(
        repository: Rc<RefCell<Repository>>,
        client_repositories: Rc<RefCell<Vec<ClientRepositories>>>,
        found_repo: Option<&Repository>,
        found_client_repo: Option<&ClientRepositories>,
    ) {
        // ...and fetch a new batch of interaction data
        client_repositories.borrow_mut()[0]
            .set_values_from_buffer(&found_client_repo.unwrap())
            .exec_generate_timesheets_from_git_history()
            .compare_logs_and_set_timesheets();

        // if it's been found, set the working repo to the timesheet struct as it may be operated on
        if found_repo.is_some() {
            repository
                .borrow_mut()
                .set_values_from_buffer(&found_repo.unwrap());
        }
    }

    fn fetch_interaction_data(
        mut client_repositories: RefMut<Vec<ClientRepositories>>,
        repository: Ref<Repository>,
    ) {
        client_repositories[0]
            .set_values(repository)
            .exec_generate_timesheets_from_git_history()
            .compare_logs_and_set_timesheets();
    }

    /// Find and update client if sheet exists, otherwise write a new one
    fn write_to_config_file(
        client_repositories: Rc<RefCell<Vec<ClientRepositories>>>,
        deserialized_config: Option<&mut Vec<ClientRepositories>>,
    ) {
        // get path for where to write the config file
        let config_path = crate::file_reader::get_filepath(crate::file_reader::get_home_path())
            .unwrap_or_else(|err| {
                eprintln!("Error constructing filepath: {}", err);
                std::process::exit(exitcode::CANTCREAT);
            });

        let json = crate::file_reader::serialize_config(
            Rc::clone(&client_repositories),
            deserialized_config,
        )
        .unwrap_or_else(|err| {
            eprintln!("Error serializing json: {}", err);
            std::process::exit(exitcode::CANTCREAT);
        });

        crate::file_reader::write_json_to_config_file(json, config_path).unwrap_or_else(|err| {
            eprintln!("Error writing data to file: {}", err);
            std::process::exit(exitcode::CANTCREAT);
        });
    }

    // Check for repo by path or by namespace
    fn check_for_client_or_repo_in_buffer<'a>(
        self,
        deserialized_config: &'a mut Vec<ClientRepositories>,
        repo_path: Option<&String>,
        repo_namespace: Option<&String>,
        client_name: Option<&String>,
    ) -> Result<(Option<&'a Repository>, Option<&'a ClientRepositories>), Box<dyn std::error::Error>>
    {
        // function should return either a repository, a client repository, or both
        let mut namespace: Option<String> = repo_namespace.map(|x| x.to_owned());

        if let Some(path) = repo_path {
            let mut temp_repository = Repository {
                repo_path: Option::from(path.to_owned()),
                ..Default::default()
            };

            // get namespace of working repository
            temp_repository
                .find_git_path_from_directory_from()?
                .find_namespace_from_git_path()?;

            namespace = temp_repository.namespace;
        }

        let mut option: (Option<&Repository>, Option<&ClientRepositories>) =
            (Option::None, Option::None);
        // if a client name is passed, get ClientRepositories from that
        // if this is true, repo_path and repo_namespace will be None
        if let Some(c) = client_name {
            for i in 0..deserialized_config.len() {
                if deserialized_config[i].get_client_name().to_lowercase()
                    == c.to_owned().to_lowercase()
                {
                    option = (Option::None, Option::from(&deserialized_config[i]));
                } else if i == &deserialized_config.len() - 1 {
                    // if the client is passed but not found
                    //TODO - if this happens it would be good to give options - i.e list of clients, and list of repos
                    eprintln!(
                        "The client, or client + namespace combination you passed has not be found.");
                    std::process::exit(exitcode::CANTCREAT);
                }
            }
        } else {
            // otherwise check whether any clients contain the namespace
            // and return the repository and the client
            for client in deserialized_config.iter() {
                option = match client
                    .repositories
                    .as_ref()
                    .unwrap()
                    .iter()
                    .find(|repository| {
                        repository.namespace.as_ref().unwrap().to_lowercase()
                            == namespace.as_ref().unwrap().to_lowercase()
                    }) {
                    Some(repository) => (Option::from(repository), Option::from(client)),
                    None => option,
                };
            }
        }

        Ok(option)
    }

    fn check_for_config_file(
        self,
        buffer: &mut String,
        repository: Rc<RefCell<Repository>>,
        client_repositories: Rc<RefCell<Vec<ClientRepositories>>>,
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
            Config::fetch_interaction_data(client_repositories.borrow_mut(), repository.borrow());
            Config::write_to_config_file(client_repositories, None);
            crate::help_prompt::HelpPrompt::show_write_new_config_success();
            return;
        }
    }
}

pub trait Init {
    /// Generate a config file with user variables
    fn init(
        &self,
        options: Vec<Option<String>>,
        repository: Rc<RefCell<Repository>>,
        client_repositories: Rc<RefCell<Vec<ClientRepositories>>>,
        prompt: RcHelpPrompt,
    );
}

impl Init for Config {
    fn init(
        &self,
        options: Vec<Option<String>>,
        repository: Rc<RefCell<Repository>>,
        client_repositories: Rc<RefCell<Vec<ClientRepositories>>>,
        prompt: RcHelpPrompt,
    ) {
        // try to read config file. Write a new one if it doesn't exist
        let mut buffer = String::new();
        self.check_for_config_file(
            &mut buffer,
            Rc::clone(&repository),
            Rc::clone(&client_repositories),
            Rc::clone(&prompt),
        );

        // ..if the there is an existing config file, check whether the (passed path or namespace) repository exists under any clients
        // if it does pass Repository values to Repository
        if crate::utils::config_file_found(&mut buffer) {
            let mut deserialized_config: Vec<ClientRepositories> = serde_json::from_str(&buffer)
                .expect("Initialisation of ClientRepository struct from buffer failed");

            let (found_repo, found_client_repo) = self
                .check_for_client_or_repo_in_buffer(
                    &mut deserialized_config,
                    Option::from(&options[0]),
                    Option::None,
                    Option::None,
                )
                .unwrap_or_else(|err| {
                    eprintln!("Error trying to read from config file: {}", err);
                    std::process::exit(exitcode::DATAERR);
                });

            if found_repo.is_some() & found_client_repo.is_some() {
                crate::help_prompt::HelpPrompt::repo_already_initialised();
            } else {
                // Otherwise onboard them and check whether (passed path or namespace) repo
                // should exist under an existing client
                let mut deserialized_config: Vec<ClientRepositories> =
                    serde_json::from_str(&mut buffer)
                        .expect("Initialisation of ClientRepository struct from buffer failed");

                prompt
                    .borrow_mut()
                    .prompt_for_client_then_onboard(&mut deserialized_config)
                    .unwrap_or_else(|err| {
                        eprintln!("Error adding repository to client: {}", err);
                        std::process::exit(exitcode::CANTCREAT);
                    });

                // ...and fetch a new batch of interaction data
                Config::fetch_interaction_data(
                    client_repositories.borrow_mut(),
                    repository.borrow(),
                );
                Config::write_to_config_file(
                    client_repositories,
                    Option::from(&mut deserialized_config),
                );

                crate::help_prompt::HelpPrompt::show_write_new_repo_success();
            }
        }
    }
}

pub trait Make {
    /// Edit a day entry within the repository
    fn make(
        &self,
        options: Vec<Option<String>>,
        repository: Rc<RefCell<Repository>>,
        client_repositories: Rc<RefCell<Vec<ClientRepositories>>>,
        prompt: RcHelpPrompt,
    );
}

impl Make for Config {
    #[tokio::main]
    async fn make(
        &self,
        options: Vec<Option<String>>,
        repository: Rc<RefCell<Repository>>,
        client_repositories: Rc<RefCell<Vec<ClientRepositories>>>,
        prompt: RcHelpPrompt,
    ) {
        // try to read config file. Write a new one if it doesn't exist
        let mut buffer = String::new();
        let current_repo_path = crate::utils::get_canonical_path(".");

        self.check_for_config_file(
            &mut buffer,
            Rc::clone(&repository),
            Rc::clone(&client_repositories),
            prompt.clone(),
        );

        if crate::utils::config_file_found(&mut buffer) {
            let mut deserialized_config: Vec<ClientRepositories> = serde_json::from_str(&buffer)
                .expect("Initialisation of ClientRepository struct from buffer failed");

            let (found_repo, found_client_repo) = self
                .check_for_client_or_repo_in_buffer(
                    &mut deserialized_config,
                    Option::from(&current_repo_path),
                    Option::None,
                    Option::from(&options[0]),
                )
                .unwrap_or_else(|err| {
                    eprintln!("Error trying to read from config file: {}", err);
                    std::process::exit(exitcode::DATAERR);
                });

            if found_client_repo.is_some() {
                Self::push_found_values_into_rcs(
                    Rc::clone(&repository),
                    Rc::clone(&client_repositories),
                    found_repo.clone(),
                    found_client_repo.clone(),
                );

                prompt
                    .borrow_mut()
                    .add_project_numbers(Rc::clone(&client_repositories))
                    .unwrap_or_else(|err| {
                        eprintln!("Error parsing project number: {}", err);
                        std::process::exit(exitcode::CANTCREAT);
                    })
                    .prompt_for_manager_approval(Rc::clone(&client_repositories))
                    .unwrap_or_else(|err| {
                        eprintln!("Error setting manager approval: {}", err);
                        std::process::exit(exitcode::CANTCREAT);
                    });

                // generate timesheet-gen.io link using existing config
                link_builder::build_unique_uri(Rc::clone(&client_repositories), options)
                    .await
                    .unwrap_or_else(|err| {
                        eprintln!("Error building unique link: {}", err);
                        std::process::exit(exitcode::CANTCREAT);
                    });
            } else {
                crate::help_prompt::HelpPrompt::client_or_repository_not_found();
            }
        }
    }
}

pub trait Edit {
    /// Generate a config file with user variables
    fn edit(
        &self,
        options: Vec<Option<String>>,
        repository: Rc<RefCell<Repository>>,
        client_repositories: Rc<RefCell<Vec<ClientRepositories>>>,
        prompt: RcHelpPrompt,
    );
}

impl Edit for Config {
    fn edit(
        &self,
        options: Vec<Option<String>>,
        repository: Rc<RefCell<Repository>>,
        client_repositories: Rc<RefCell<Vec<ClientRepositories>>>,
        prompt: RcHelpPrompt,
    ) {
        // try to read config file. Write a new one if it doesn't exist
        let mut buffer = String::new();
        self.check_for_config_file(
            &mut buffer,
            Rc::clone(&repository),
            Rc::clone(&client_repositories),
            Rc::clone(&prompt),
        );

        if crate::utils::config_file_found(&mut buffer) {
            let mut deserialized_config: Vec<ClientRepositories> = serde_json::from_str(&buffer)
                .expect("Initialisation of ClientRepository struct from buffer failed");

            let (found_repo, found_client_repo) = self
                .check_for_client_or_repo_in_buffer(
                    &mut deserialized_config,
                    Option::None,
                    Option::from(&options[0]),
                    Option::None,
                )
                .unwrap_or_else(|err| {
                    eprintln!("Error trying to read from config file: {}", err);
                    std::process::exit(exitcode::DATAERR);
                });

            if found_client_repo.is_some() {
                Self::push_found_values_into_rcs(
                    Rc::clone(&repository),
                    Rc::clone(&client_repositories),
                    found_repo.clone(),
                    found_client_repo.clone(),
                );

                repository
                    .borrow_mut()
                    .update_hours_on_month_day_entry(&options)
                    .unwrap_or_else(|err| {
                        eprintln!("Error editing timesheet: {}", err);
                        process::exit(exitcode::DATAERR);
                    });

                client_repositories.borrow_mut()[0]
                    .set_values(repository.borrow())
                    .exec_generate_timesheets_from_git_history()
                    .compare_logs_and_set_timesheets();

                let client_borrow = client_repositories.borrow();
                let mut new_client_repos = vec![];
                Self::update_client_repositories(
                    &mut new_client_repos,
                    deserialized_config,
                    client_borrow,
                );

                Config::write_to_config_file(Rc::new(RefCell::new(new_client_repos)), None);
                crate::help_prompt::HelpPrompt::show_edited_config_success();
            } else {
                crate::help_prompt::HelpPrompt::client_or_repository_not_found();
            }
        }
    }
}

pub trait Remove {
    /// Update client or repository details
    fn remove(
        &self,
        options: Vec<Option<String>>,
        repository: Rc<RefCell<Repository>>,
        client_repositories: Rc<RefCell<Vec<ClientRepositories>>>,
        prompt: RcHelpPrompt,
    );
}

impl Remove for Config {
    fn remove(
        &self,
        options: Vec<Option<String>>,
        repository: Rc<RefCell<Repository>>,
        client_repositories: Rc<RefCell<Vec<ClientRepositories>>>,
        prompt: RcHelpPrompt,
    ) {
        // try to read config file. Write a new one if it doesn't exist
        let mut buffer = String::new();
        self.check_for_config_file(
            &mut buffer,
            Rc::clone(&repository),
            Rc::clone(&client_repositories),
            Rc::clone(&prompt),
        );

        // Find repo or client and remove them from config file
        if crate::utils::config_file_found(&mut buffer) {
            let mut deserialized_config: Vec<ClientRepositories> = serde_json::from_str(&buffer)
                .expect("Initialisation of ClientRepository struct from buffer failed");

            let (found_repo, found_client_repo) = self
                .check_for_client_or_repo_in_buffer(
                    &mut deserialized_config,
                    Option::None,
                    Option::from(&options[1]),
                    Option::from(&options[0]),
                )
                .unwrap_or_else(|err| {
                    eprintln!("Error trying to read from config file: {}", err);
                    std::process::exit(exitcode::DATAERR);
                });

            if found_client_repo.is_some() {
                Self::push_found_values_into_rcs(
                    Rc::clone(&repository),
                    Rc::clone(&client_repositories),
                    found_repo.clone(),
                    found_client_repo.clone(),
                );

                let mut client_repo_borrow = client_repositories.borrow_mut();

                client_repo_borrow.append(&mut deserialized_config);

                prompt
                    .borrow_mut()
                    .prompt_for_client_repo_removal(client_repo_borrow, options)
                    .expect("Remove failed");

                // if there are no clients, lets remove the file and next time will be onboarding
                //TODO - would be nice to improve this
                if client_repositories.borrow().len() == 0 {
                    crate::file_reader::delete_config_file().expect(
                        "Config file was empty so timesheet-gen tried to remove it. That failed.",
                    );
                    exit_process();
                    return;
                }

                // pass modified config as new client_repository and thus write it straight to file
                Config::write_to_config_file(Rc::clone(&client_repositories), None);
            } else {
                crate::help_prompt::HelpPrompt::client_or_repository_not_found();
            }
        }
    }
}

pub trait Update {
    /// Update client or repository details
    fn update(
        &self,
        options: Vec<Option<String>>,
        repository: Rc<RefCell<Repository>>,
        client_repositories: Rc<RefCell<Vec<ClientRepositories>>>,
        prompt: RcHelpPrompt,
    );
}

impl Update for Config {
    fn update(
        &self,
        options: Vec<Option<String>>,
        repository: Rc<RefCell<Repository>>,
        client_repositories: Rc<RefCell<Vec<ClientRepositories>>>,
        prompt: RcHelpPrompt,
    ) {
        // try to read config file. Write a new one if it doesn't exist
        let mut buffer = String::new();
        self.check_for_config_file(
            &mut buffer,
            Rc::clone(&repository),
            Rc::clone(&client_repositories),
            Rc::clone(&prompt),
        );

        if crate::utils::config_file_found(&mut buffer) {
            let mut deserialized_config: Vec<ClientRepositories> = serde_json::from_str(&buffer)
                .expect("Initialisation of ClientRepository struct from buffer failed");

            let (found_repo, found_client_repo) = self
                .check_for_client_or_repo_in_buffer(
                    &mut deserialized_config,
                    Option::None,
                    Option::from(&options[1]),
                    Option::from(&options[0]),
                )
                .unwrap_or_else(|err| {
                    eprintln!("Error trying to read from config file: {}", err);
                    std::process::exit(exitcode::DATAERR);
                });

            if found_client_repo.is_some() {
                Self::push_found_values_into_rcs(
                    Rc::clone(&repository),
                    Rc::clone(&client_repositories),
                    found_repo.clone(),
                    found_client_repo.clone(),
                );

                prompt
                    .borrow_mut()
                    .prompt_for_update(Rc::clone(&client_repositories), options)
                    .expect("Update failed");

                let client_borrow = client_repositories.borrow();
                let mut new_client_repos = vec![];
                Self::update_client_repositories(
                    &mut new_client_repos,
                    deserialized_config,
                    client_borrow,
                );

                // pass modified config as new client_repository and thus write it straight to file
                Config::write_to_config_file(Rc::new(RefCell::new(new_client_repos)), None);
                crate::help_prompt::HelpPrompt::show_updated_config_success();
            } else {
                crate::help_prompt::HelpPrompt::client_or_repository_not_found();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::client_repositories::ClientRepositories;
    use crate::config::{Config, Edit, New, Remove};
    use crate::repository::Repository;
    use envtestkit::lock::lock_test;
    use envtestkit::set_env;
    use serde_json::{Number, Value};
    use std::cell::RefCell;
    use std::ffi::OsString;
    use std::rc::Rc;

    fn create_mock_client_repository(client_repository: &mut ClientRepositories) {
        let repo = RefCell::new(Repository {
            client_name: Option::from("alphabet".to_string()),
            client_address: Option::from("Spaghetti Way, USA".to_string()),
            client_contact_person: Option::from("John Smith".to_string()),
            name: Option::from("Jim Jones".to_string()),
            email: Option::from("jim@jones.com".to_string()),
            namespace: Option::from("timesheet-gen".to_string()),
            ..Default::default()
        });

        client_repository.set_values(repo.borrow());
    }

    #[test]
    fn it_modifies_the_hour_entry_in_a_client_repository_day_entry() {
        let _lock = lock_test();
        let _test = set_env(OsString::from("TEST_MODE"), "true");

        let config = Config::new();
        let options = vec![
            Option::from("timesheet-gen".to_string()),
            Option::from("20".to_string()),
            Option::from("1".to_string()),
            Option::from("11".to_string()),
            Option::from("2021".to_string()),
        ];

        let client_repos = Rc::new(RefCell::new(vec![ClientRepositories {
            ..Default::default()
        }]));

        let repo = Rc::new(RefCell::new(Repository {
            ..Default::default()
        }));

        let prompt = Rc::new(RefCell::new(crate::help_prompt::HelpPrompt::new(
            Rc::clone(&repo),
        )));

        config.edit(
            options,
            Rc::clone(&repo),
            Rc::clone(&client_repos),
            Rc::clone(&prompt),
        );

        let repo_borrow = repo.borrow();

        let month = repo_borrow
            .timesheet
            .as_ref()
            .unwrap()
            .get("2021")
            .as_ref()
            .unwrap()
            .get("11")
            .as_ref()
            .unwrap()
            .clone();

        let hour_value = month[0].get("hours").as_ref().unwrap().clone();
        let edited_value = month[0].get("user_edited").as_ref().unwrap().clone();

        assert_eq!(hour_value, &Value::Number(Number::from_f64(20.0).unwrap()));
        assert_eq!(edited_value, &Value::Bool(true));
    }

    fn is_repo_in_client_repos(config: &Vec<ClientRepositories>, namespace: &String) -> bool {
        config.iter().any(|client| {
            client.repositories.as_ref().unwrap().iter().any(|repo| {
                repo.namespace.as_ref().unwrap().to_lowercase() == namespace.to_lowercase()
            })
        })
    }

    fn is_client_in_client_repos(config: &Vec<ClientRepositories>, client_name: &String) -> bool {
        config.iter().any(|client| {
            client.client.as_ref().unwrap().client_name.to_lowercase() == client_name.to_lowercase()
        })
    }

    #[test]
    fn it_removes_a_repository() {
        let _lock = lock_test();
        let _test = set_env(OsString::from("TEST_MODE"), "true");

        let mut buffer = String::new();
        let namespace = "pila-app".to_string();
        let config = Config::new();
        let options = vec![
            Option::from("apple".to_string()),
            Option::from(namespace.clone()),
        ];

        let client_repos = Rc::new(RefCell::new(vec![ClientRepositories {
            ..Default::default()
        }]));

        let repo = Rc::new(RefCell::new(Repository {
            ..Default::default()
        }));

        let prompt = Rc::new(RefCell::new(crate::help_prompt::HelpPrompt::new(
            Rc::clone(&repo),
        )));

        crate::file_reader::read_data_from_config_file(&mut buffer, Rc::clone(&prompt))
            .expect("Read of test data failed");

        let deserialized_config: Vec<ClientRepositories> = serde_json::from_str(&mut buffer)
            .expect("Initialisation of ClientRepository struct from buffer failed");

        assert_eq!(
            is_repo_in_client_repos(&deserialized_config, &namespace),
            true
        );

        config.remove(
            options,
            Rc::clone(&repo),
            Rc::clone(&client_repos),
            Rc::clone(&prompt),
        );

        assert_eq!(
            is_repo_in_client_repos(&client_repos.borrow_mut(), &namespace),
            false
        );
    }

    #[test]
    fn it_removes_a_client() {
        let _lock = lock_test();
        let _test = set_env(OsString::from("TEST_MODE"), "true");

        let mut buffer = String::new();
        let client = "apple".to_string();
        let config = Config::new();
        let options = vec![Option::from(client.clone()), Option::None];

        let client_repos = Rc::new(RefCell::new(vec![ClientRepositories {
            ..Default::default()
        }]));

        let repo = Rc::new(RefCell::new(Repository {
            ..Default::default()
        }));

        let prompt = Rc::new(RefCell::new(crate::help_prompt::HelpPrompt::new(
            Rc::clone(&repo),
        )));

        crate::file_reader::read_data_from_config_file(&mut buffer, Rc::clone(&prompt))
            .expect("Read of test data failed");

        let deserialized_config: Vec<ClientRepositories> = serde_json::from_str(&mut buffer)
            .expect("Initialisation of ClientRepository struct from buffer failed");

        assert_eq!(
            is_client_in_client_repos(&deserialized_config, &client),
            true
        );

        config.remove(
            options,
            Rc::clone(&repo),
            Rc::clone(&client_repos),
            Rc::clone(&prompt),
        );

        assert_eq!(
            is_client_in_client_repos(&client_repos.borrow_mut(), &client),
            false
        );
    }

    #[test]
    fn it_checks_for_repo_in_buffer_by_path_and_returns_a_tuple() {
        let mut deserialized_config = ClientRepositories {
            ..Default::default()
        };

        create_mock_client_repository(&mut deserialized_config);

        let config: Config = Config::new();

        if let Some(repository) = config
            .check_for_client_or_repo_in_buffer(
                &mut vec![deserialized_config],
                Option::from(&".".to_string()),
                Option::None,
                Option::None,
            )
            .unwrap()
            .0
        {
            assert_eq!(
                *repository.namespace.as_ref().unwrap(),
                "timesheet-gen".to_string()
            )
        }
    }

    #[test]
    fn it_checks_for_repo_in_buffer_by_namespace_and_returns_a_tuple() {
        let mut deserialized_config = ClientRepositories {
            ..Default::default()
        };

        create_mock_client_repository(&mut deserialized_config);

        let config: Config = Config::new();

        if let Some(repository) = config
            .check_for_client_or_repo_in_buffer(
                &mut vec![deserialized_config],
                Option::None,
                Option::from(&"timesheet-gen".to_string()),
                Option::None,
            )
            .unwrap()
            .0
        {
            assert_eq!(
                *repository.namespace.as_ref().unwrap(),
                "timesheet-gen".to_string()
            )
        }
    }

    #[test]
    fn it_checks_for_repo_in_buffer_by_client_and_returns_a_tuple() {
        let mut deserialized_config = ClientRepositories {
            ..Default::default()
        };

        create_mock_client_repository(&mut deserialized_config);

        let config: Config = Config::new();

        if let Some(client_repo) = config
            .check_for_client_or_repo_in_buffer(
                &mut vec![deserialized_config],
                Option::None,
                Option::None,
                Option::from(&"alphabet".to_string()),
            )
            .unwrap()
            .1
        {
            assert_eq!(
                *client_repo.client.as_ref().unwrap().client_name,
                "alphabet".to_string()
            )
        }
    }
}
