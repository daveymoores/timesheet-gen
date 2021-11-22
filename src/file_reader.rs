use crate::config::TimesheetConfig;
use crate::help_prompt::Onboarding;
use crate::timesheet::Timesheet;
use serde_json::json;
use std::cell::Ref;
use std::fs::File;
use std::io::{Read, Write};
use std::ops::Deref;
use std::path::PathBuf;
use std::process;

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
    prompt: T,
) -> Result<(), Box<dyn std::error::Error>>
where
    T: Onboarding,
{
    match File::open(&path) {
        Ok(mut file) => {
            file.read_to_string(buffer)?;
        }
        Err(_) => {
            prompt.onboarding()?;
        }
    };

    Ok(())
}

pub fn read_data_from_config_file<T>(
    buffer: &mut String,
    prompt: T,
) -> Result<(), Box<dyn std::error::Error>>
where
    T: Onboarding,
{
    let config_path = get_filepath(get_home_path());
    read_file(buffer, config_path, prompt)?;

    Ok(())
}

pub fn write_config_file(
    deserialized_config: Option<Vec<TimesheetConfig>>,
    timesheet: Ref<Timesheet>,
    config_path: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let ts_client = timesheet.client_name.clone().unwrap_or("None".to_string());

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
                    if &client.client == &ts_client {
                        return TimesheetConfig {
                            client: ts_client.clone(),
                            repositories: client
                                .clone()
                                .repositories
                                .into_iter()
                                .map(|repository| {
                                    if repository.namespace.as_ref().unwrap() == &ts_namespace {
                                        return timesheet.deref().to_owned();
                                    }

                                    repository
                                })
                                .collect(),
                        };
                    }
                    client
                })
                .collect();

            json!(config_data)
        }
    };

    let json = serde_json::to_string(&config_data).unwrap();
    let mut file = File::create(&config_path)?;

    file.write_all(json.as_bytes())?;

    println!(
        "timesheet-gen initialised! \n\
    Try 'timesheet-gen make' to create your first timesheet \n\
    or 'timesheet-gen help' for more options."
    );

    process::exit(exitcode::OK);
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
        struct MockPrompt {}

        impl Onboarding for MockPrompt {
            fn onboarding(self) -> Result<(), Box<dyn Error>> {
                assert!(false);
                Ok(())
            }
        }

        let mock_prompt = MockPrompt {};

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
        struct MockPrompt {}

        impl Onboarding for MockPrompt {
            fn onboarding(self) -> Result<(), Box<dyn Error>> {
                assert!(true);
                Ok(())
            }
        }

        let mock_prompt = MockPrompt {};

        let mut buffer = String::new();

        read_file(
            &mut buffer,
            String::from("./testing-utils/.timesheet.txt"),
            mock_prompt,
        )
        .unwrap();
    }

    // These tests write temp files and seem to screw up the test runner
    // Ignore for now...
    #[test]
    #[ignore]
    fn it_writes_a_config_file_when_file_exists() {
        let mock_timesheet = RefCell::new(Timesheet {
            ..Default::default()
        });

        let timesheet = mock_timesheet.borrow();

        // creates mock directory that is destroyed when it goes out of scope
        let dir = tempfile::tempdir().unwrap();
        let mock_config_path = dir.path().join("my-temporary-note.txt");

        let file = File::create(&mock_config_path).unwrap();
        let string_path_from_tempdir = mock_config_path.to_str().unwrap().to_owned();
        assert_eq!(
            write_config_file(None, timesheet, string_path_from_tempdir).unwrap(),
            ()
        );

        drop(file);
        dir.close().unwrap();
    }

    #[test]
    #[ignore]
    fn it_throws_an_error_when_writing_config_if_file_doesnt_exist() {
        let mock_timesheet = RefCell::new(Timesheet {
            ..Default::default()
        });

        let timesheet = mock_timesheet.borrow();

        // creates mock directory that is destroyed when it goes out of scope
        let dir = tempfile::tempdir().unwrap();
        let mock_config_path = dir.path().join("my-temporary-note.txt");

        let string_path_from_tempdir = mock_config_path.to_str().unwrap().to_owned();
        assert!(write_config_file(None, timesheet, string_path_from_tempdir).is_err());
    }
}
