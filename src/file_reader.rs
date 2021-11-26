use crate::client_repositories::ClientRepositories;
use crate::help_prompt::Onboarding;
use crate::repository::Repository;
use serde_json::json;
use std::cell::RefCell;
use std::fs::File;
use std::io::{Read, Write};
use std::ops::Deref;
use std::path::PathBuf;
use std::rc::Rc;

const CONFIG_FILE_NAME: &str = ".timesheet-gen.txt";

/// Find the path to the users home directory
pub fn get_home_path() -> PathBuf {
    match dirs::home_dir() {
        Some(dir) => dir,
        None => panic!("Home directory not found"),
    }
}

/// Create filepath to config file
pub fn get_filepath(path: PathBuf) -> String {
    let home_path = path.to_str();
    home_path.unwrap().to_owned() + "/" + CONFIG_FILE_NAME
}

/// Read config file or throw error and call error function
fn read_file<T>(
    buffer: &mut String,
    path: String,
    prompt: Rc<RefCell<T>>,
) -> Result<(), Box<dyn std::error::Error>>
where
    T: Onboarding,
{
    match File::open(&path) {
        Ok(mut file) => {
            file.read_to_string(buffer)?;
        }
        Err(_) => {
            prompt.borrow_mut().onboarding()?;
        }
    };

    Ok(())
}

pub fn read_data_from_config_file<T>(
    buffer: &mut String,
    prompt: Rc<RefCell<T>>,
) -> Result<(), Box<dyn std::error::Error>>
where
    T: Onboarding,
{
    let config_path = get_filepath(get_home_path());
    read_file(buffer, config_path, prompt)?;

    Ok(())
}

pub fn write_json_to_config_file(
    json: String,
    config_path: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = File::create(config_path)?;

    file.write_all(json.as_bytes())?;

    println!(
        "timesheet-gen initialised! \n\
    Try 'timesheet-gen make' to create your first timesheet \n\
    or 'timesheet-gen help' for more options."
    );

    Ok(())
}

pub fn serialize_config(
    deserialized_config: Option<Vec<ClientRepositories>>,
    client_repository: Rc<RefCell<ClientRepositories>>,
) -> Result<String, Box<dyn std::error::Error>> {
    // get the values from the current repo so that it can be merged back into the config
    let client = client_repository.borrow_mut().client.clone();
    let user = client_repository.borrow_mut().user.clone();
    let repository = client_repository
        .borrow_mut()
        .repositories
        .as_ref()
        .unwrap()[0]
        .clone();
    let namespace = repository.namespace.as_ref().unwrap();
    let client_name = &client.as_ref().unwrap().client_name;

    let config_data = match deserialized_config {
        None => {
            json!(vec![client_repository.deref()])
        }
        Some(config) => {
            let config_data: Vec<ClientRepositories> = config
                .into_iter()
                .map(|c| {
                    if &c.client.as_ref().unwrap().client_name == &client_name.clone() {
                        return ClientRepositories {
                            client: client.clone(),
                            user: user.clone(),
                            repositories: Some(
                                c.clone()
                                    .repositories
                                    .unwrap()
                                    .into_iter()
                                    .map(|repo| {
                                        if repo.namespace.as_ref().unwrap() == namespace {
                                            return repository.to_owned();
                                        }

                                        repo
                                    })
                                    .collect(),
                            ),
                        };
                    }
                    c
                })
                .collect();

            json!(config_data)
        }
    };

    let json = serde_json::to_string(&config_data)?;

    Ok(json)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::error::Error;
    use std::path::Path;

    fn create_mock_client_repository(client_repository: Rc<RefCell<ClientRepositories>>) {
        let repo = RefCell::new(Repository {
            client_name: Option::from("alphabet".to_string()),
            client_address: Option::from("Spaghetti Way, USA".to_string()),
            client_contact_person: Option::from("John Smith".to_string()),
            name: Option::from("Jim Jones".to_string()),
            email: Option::from("jim@jones.com".to_string()),
            namespace: Option::from("timesheet-gen".to_string()),
            ..Default::default()
        });

        client_repository.borrow_mut().set_values(repo.borrow());
    }

    #[test]
    fn get_filepath_returns_path_with_file_name() {
        let path_buf = PathBuf::from("/path/to/usr");
        assert_eq!(get_filepath(path_buf), "/path/to/usr/.timesheet-gen.txt");
    }

    #[test]
    fn get_home_path_should_return_a_path() {
        let path_buf = get_home_path();
        let path = path_buf.to_str().unwrap();

        assert!(Path::new(path).exists());
    }

    #[test]
    fn read_file_returns_a_buffer() {
        #[derive(Clone)]
        struct MockPrompt {}

        impl Onboarding for MockPrompt {
            fn onboarding(&self) -> Result<(), Box<dyn Error>> {
                assert!(false);
                Ok(())
            }
        }

        let mock_prompt = Rc::new(RefCell::new(MockPrompt {}));

        let mut buffer = String::new();

        read_file(
            &mut buffer,
            String::from("./testing-utils/.hello.txt"),
            mock_prompt,
        )
        .unwrap();

        assert_eq!(buffer.trim(), "hello");
    }

    #[test]
    fn read_file_calls_the_error_function() {
        #[derive(Clone)]
        struct MockPrompt {}

        impl Onboarding for MockPrompt {
            fn onboarding(&self) -> Result<(), Box<dyn Error>> {
                assert!(true);
                Ok(())
            }
        }

        let mock_prompt = Rc::new(RefCell::new(MockPrompt {}));

        let mut buffer = String::new();

        read_file(
            &mut buffer,
            String::from("./testing-utils/.timesheet.txt"),
            mock_prompt,
        )
        .unwrap();
    }

    #[test]
    fn it_writes_a_config_file_when_file_exists() {
        let client_repositories = Rc::new(RefCell::new(ClientRepositories {
            ..Default::default()
        }));

        create_mock_client_repository(client_repositories.clone());

        // creates mock directory that is destroyed when it goes out of scope
        let dir = tempfile::tempdir().unwrap();
        let mock_config_path = dir.path().join("my-temporary-note.txt");

        let file = File::create(&mock_config_path).unwrap();
        let string_path_from_temp_dir = mock_config_path.to_str().unwrap().to_owned();

        let json = serialize_config(None, client_repositories).unwrap();

        assert!(write_json_to_config_file(json, string_path_from_temp_dir).is_ok());

        drop(file);
        dir.close().unwrap();
    }

    #[test]
    fn it_throws_an_error_when_writing_config_if_file_doesnt_exist() {
        let client_repositories = Rc::new(RefCell::new(ClientRepositories {
            ..Default::default()
        }));

        create_mock_client_repository(client_repositories.clone());

        let json = serialize_config(None, client_repositories).unwrap();

        assert!(write_json_to_config_file(json, "./a/fake/path".to_string()).is_err());
    }

    #[test]
    fn it_finds_and_updates_a_client() {
        let client_repositories = Rc::new(RefCell::new(ClientRepositories {
            ..Default::default()
        }));

        create_mock_client_repository(client_repositories.clone());

        let deserialized_config = vec![client_repositories.borrow().clone()];

        let json =
            serialize_config(Option::from(deserialized_config), client_repositories).unwrap();
        let value: Vec<ClientRepositories> = serde_json::from_str(&*json).unwrap();

        assert_eq!(
            value[0].repositories.as_ref().unwrap()[0]
                .client_contact_person
                .as_ref()
                .unwrap(),
            &"John Smith".to_string()
        );
    }
}
