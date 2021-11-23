use crate::help_prompt::Onboarding;
use crate::timesheet::Timesheet;
use crate::timesheet_config::{Client, TimesheetConfig};
use serde_json::json;
use std::cell::{Ref, RefCell};
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
    deserialized_config: Option<Vec<TimesheetConfig>>,
    timesheet: Ref<Timesheet>,
) -> Result<String, Box<dyn std::error::Error>> {
    let ts_client = Client {
        client_name: timesheet.client_name.clone().unwrap_or("None".to_string()),
        client_address: timesheet
            .client_address
            .clone()
            .unwrap_or("None".to_string()),
        client_contact_person: timesheet
            .client_contact_person
            .clone()
            .unwrap_or("None".to_string()),
    };

    let ts_namespace = timesheet.namespace.clone().unwrap_or("None".to_string());

    // if the client and namespace exists, update it with current timesheet
    let config_data = match deserialized_config {
        None => {
            json!([{
                "client": &ts_client,
                "repositories": [timesheet.deref()],
            }])
        }
        Some(config) => {
            let config_data: Vec<TimesheetConfig> = config
                .into_iter()
                .map(|client| {
                    if &client.client.as_ref().unwrap().client_name == &ts_client.client_name {
                        return TimesheetConfig {
                            client: Option::from(ts_client.clone()),
                            repositories: Some(
                                client
                                    .clone()
                                    .repositories
                                    .unwrap()
                                    .into_iter()
                                    .map(|repository| {
                                        if repository.namespace.as_ref().unwrap() == &ts_namespace {
                                            return timesheet.deref().to_owned();
                                        }

                                        repository
                                    })
                                    .collect(),
                            ),
                        };
                    }
                    client
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
        let mock_timesheet = RefCell::new(Timesheet {
            ..Default::default()
        });

        let timesheet = mock_timesheet.borrow();

        // creates mock directory that is destroyed when it goes out of scope
        let dir = tempfile::tempdir().unwrap();
        let mock_config_path = dir.path().join("my-temporary-note.txt");

        let file = File::create(&mock_config_path).unwrap();
        let string_path_from_temp_dir = mock_config_path.to_str().unwrap().to_owned();

        let json = serialize_config(None, timesheet).unwrap();

        assert!(write_json_to_config_file(json, string_path_from_temp_dir).is_ok());

        drop(file);
        dir.close().unwrap();
    }

    #[test]
    fn it_throws_an_error_when_writing_config_if_file_doesnt_exist() {
        let mock_timesheet = RefCell::new(Timesheet {
            ..Default::default()
        });

        let timesheet = mock_timesheet.borrow();

        let json = serialize_config(None, timesheet).unwrap();

        assert!(write_json_to_config_file(json, "./a/fake/path".to_string()).is_err());
    }

    #[test]
    fn it_finds_and_updates_a_client() {
        let deserialized_config = vec![TimesheetConfig {
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

        let mock_timesheet = RefCell::new(Timesheet {
            client_name: Option::from("alphabet".to_string()),
            namespace: Option::from("timesheet-gen".to_string()),
            client_contact_person: Option::from("John Jones".to_string()),
            ..Default::default()
        });

        let timesheet = mock_timesheet.borrow();

        let json = serialize_config(Option::from(deserialized_config), timesheet).unwrap();
        let value: Vec<TimesheetConfig> = serde_json::from_str(&*json).unwrap();

        assert_eq!(
            value[0].repositories.as_ref().unwrap()[0]
                .client_contact_person
                .as_ref()
                .unwrap(),
            &"John Jones".to_string()
        );
    }
}
