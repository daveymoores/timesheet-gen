use crate::client_repositories::ClientRepositories;
use crate::help_prompt::Onboarding;
use serde_json::json;
use std::cell::RefCell;
use std::fs::File;
use std::io::{Read, Write};
use std::ops::Deref;
use std::path::PathBuf;
use std::rc::Rc;
use tempfile::tempfile;

const CONFIG_FILE_NAME: &str = ".timesheet-gen.txt";

/// Find the path to the users home directory
pub fn get_home_path() -> PathBuf {
    match dirs::home_dir() {
        Some(dir) => dir,
        None => panic!("Home directory not found"),
    }
}

/// Create filepath to config file
pub fn get_filepath(path: PathBuf) -> Result<String, Box<dyn std::error::Error>> {
    return if crate::utils::is_test_mode() {
        let path_string = &*format!("./testing-utils/{}", CONFIG_FILE_NAME);
        Ok(path_string.to_owned())
    } else {
        let home_path = path.to_str();
        Ok(home_path.unwrap().to_owned() + "/" + CONFIG_FILE_NAME)
    };
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
            prompt.borrow_mut().onboarding(true)?;
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
    let config_path = get_filepath(get_home_path())?;
    read_file(buffer, config_path, prompt)?;

    Ok(())
}

pub fn delete_config_file() -> Result<(), Box<dyn std::error::Error>> {
    if crate::utils::is_test_mode() {
        return Ok(());
    }

    let config_path = get_filepath(get_home_path())?;
    std::fs::remove_file(config_path)?;

    Ok(())
}

pub fn write_json_to_config_file(
    json: String,
    config_path: String,
) -> Result<(), Box<dyn std::error::Error>> {
    if crate::utils::is_test_mode() {
        let mut file = tempfile()?;
        file.write_all(json.as_bytes())?;
        return Ok(());
    }

    let mut file = File::create(config_path)?;

    file.write_all(json.as_bytes())?;

    Ok(())
}

pub fn serialize_config(
    client_repository: Rc<RefCell<Vec<ClientRepositories>>>,
    deserialized_config: Option<&mut Vec<ClientRepositories>>,
) -> Result<String, Box<dyn std::error::Error>> {
    let config_data = match deserialized_config {
        // if deserialized_config doesn't exist, then create fresh json for file
        None => {
            json!(client_repository.deref())
        }
        // if it does exist, lets add it to the existing client repositories, or
        // push it into the vec to add a new client
        Some(config) => {
            // get the values from the current repo so that it can be merged back into the config
            // if deserialized_config is none, there is only one value in the vec so we can safely pull it out
            let client_repo_borrow = client_repository.borrow_mut();
            let client = client_repo_borrow[0].client.clone();
            let user = client_repo_borrow[0].user.clone();
            let approver = client_repo_borrow[0].approver.clone();
            let repository = client_repo_borrow[0].repositories.as_ref().unwrap()[0].clone();
            let client_name = &client.as_ref().unwrap().client_name;

            let config_data: Vec<ClientRepositories> = if config
                .into_iter()
                .any(|x| &x.get_client_name() == client_name)
            {
                let x: Vec<ClientRepositories> = config
                    .iter_mut()
                    .map(|c| {
                        if &c.get_client_name() == client_name {
                            return ClientRepositories {
                                approver: approver.clone(),
                                client: client.clone(),
                                user: user.clone(),
                                repositories: Some(
                                    vec![
                                        c.clone().repositories.unwrap(),
                                        vec![repository.to_owned()],
                                    ]
                                    .concat(),
                                ),
                                ..Default::default()
                            };
                        }
                        c.clone()
                    })
                    .collect();

                x
            } else {
                config.push(client_repo_borrow[0].clone());
                config.to_vec()
            };

            json!(config_data)
        }
    };

    let json = serde_json::to_string(&config_data)?;

    Ok(json)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client_repositories::Client;
    use crate::repository::Repository;
    use envtestkit::lock::lock_test;
    use envtestkit::set_env;
    use nanoid::nanoid;
    use std::cell::RefCell;
    use std::error::Error;
    use std::ffi::OsString;
    use std::path::Path;

    fn create_mock_client_repository(client_repository: Rc<RefCell<Vec<ClientRepositories>>>) {
        let repo = RefCell::new(Repository {
            client_name: Option::from("alphabet".to_string()),
            client_address: Option::from("Spaghetti Way, USA".to_string()),
            client_contact_person: Option::from("John Smith".to_string()),
            name: Option::from("Jim Jones".to_string()),
            email: Option::from("jim@jones.com".to_string()),
            namespace: Option::from("timesheet-gen".to_string()),
            ..Default::default()
        });

        client_repository.borrow_mut()[0].set_values(repo.borrow());
    }

    #[test]
    fn it_serializes_a_config_and_adds_to_an_existing_client() {
        let client_repositories = Rc::new(RefCell::new(vec![ClientRepositories {
            ..Default::default()
        }]));

        create_mock_client_repository(client_repositories.clone());

        let mut deserialized_config = client_repositories.borrow_mut().deref().clone();

        let json_string =
            serialize_config(client_repositories, Option::from(&mut deserialized_config)).unwrap();

        let constructed_client_repos: Vec<ClientRepositories> =
            serde_json::from_str(&json_string).unwrap();

        //before
        assert_eq!(
            &deserialized_config[0]
                .repositories
                .as_ref()
                .unwrap()
                .iter()
                .len(),
            &1
        );
        //after
        assert_eq!(
            &constructed_client_repos[0]
                .repositories
                .as_ref()
                .unwrap()
                .iter()
                .len(),
            &2
        );
    }

    #[test]
    fn it_serializes_a_config_and_adds_a_new_client() {
        let client_repositories = Rc::new(RefCell::new(vec![ClientRepositories {
            ..Default::default()
        }]));

        create_mock_client_repository(client_repositories.clone());

        let mut deserialized_config = vec![ClientRepositories {
            client: Some(Client {
                id: nanoid!(),
                client_name: "New client".to_string(),
                client_address: "Somewhere".to_string(),
                client_contact_person: "Jim Jones".to_string(),
            }),
            user: None,
            repositories: None,
            ..Default::default()
        }];

        let length_before = &deserialized_config.len();

        let json_string =
            serialize_config(client_repositories, Option::from(&mut deserialized_config)).unwrap();

        let constructed_client_repos: Vec<ClientRepositories> =
            serde_json::from_str(&json_string).unwrap();

        //before
        assert_eq!(length_before, &1);
        //after
        assert_eq!(&constructed_client_repos.len(), &2);
    }

    #[test]
    fn get_filepath_returns_path_with_file_name() {
        let _lock = lock_test();
        let _test = set_env(OsString::from("TEST_MODE"), "false");

        let path_buf = PathBuf::from("/path/to/usr");
        assert_eq!(
            get_filepath(path_buf).unwrap(),
            "/path/to/usr/.timesheet-gen.txt"
        );
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
            fn onboarding(&self, _new_user: bool) -> Result<(), Box<dyn Error>> {
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
            fn onboarding(&self, _new_user: bool) -> Result<(), Box<dyn Error>> {
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
        let client_repositories = Rc::new(RefCell::new(vec![ClientRepositories {
            ..Default::default()
        }]));

        create_mock_client_repository(client_repositories.clone());

        // creates mock directory that is destroyed when it goes out of scope
        let dir = tempfile::tempdir().unwrap();
        let mock_config_path = dir.path().join("my-temporary-note.txt");

        let file = File::create(&mock_config_path).unwrap();
        let string_path_from_temp_dir = mock_config_path.to_str().unwrap().to_owned();

        let json = serialize_config(client_repositories, None).unwrap();

        assert!(write_json_to_config_file(json, string_path_from_temp_dir).is_ok());

        drop(file);
        dir.close().unwrap();
    }

    #[test]
    fn it_throws_an_error_when_writing_config_if_file_doesnt_exist() {
        let _lock = lock_test();
        let _test = set_env(OsString::from("TEST_MODE"), "false");

        let client_repositories = Rc::new(RefCell::new(vec![ClientRepositories {
            ..Default::default()
        }]));

        create_mock_client_repository(client_repositories.clone());

        let json = serialize_config(client_repositories, None).unwrap();

        assert!(write_json_to_config_file(json, "./a/fake/path".to_string()).is_err());
    }

    #[test]
    fn it_finds_and_updates_a_client() {
        let client_repositories = Rc::new(RefCell::new(vec![ClientRepositories {
            ..Default::default()
        }]));

        create_mock_client_repository(client_repositories.clone());

        let mut deserialized_config = client_repositories.borrow().clone();

        let json =
            serialize_config(client_repositories, Option::from(&mut deserialized_config)).unwrap();
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
