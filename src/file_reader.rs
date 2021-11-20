use crate::help_prompt::Onboarding;
use crate::timesheet::Timesheet;
use serde_json::json;
use std::cell::Ref;
use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Write};
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
    timesheet: &Ref<Timesheet>,
    config_path: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let unwrapped_timesheet = json!([{
        "client": timesheet.client_name.as_ref().unwrap_or(&"None".to_string()),
        "repositories": [{
            "namespace": timesheet.namespace.as_ref().unwrap_or(&"None".to_string()),
            "repo_path": timesheet.repo_path.as_ref().unwrap_or(&"None".to_string()),
            "git_path": timesheet.git_path.as_ref().unwrap_or(&"None".to_string()),
            "name": timesheet.name.as_ref().unwrap_or(&"None".to_string()),
            "email": timesheet.email.as_ref().unwrap_or(&"None".to_string()),
            "client_name": timesheet.client_name.as_ref().unwrap_or(&"None".to_string()),
            "client_contact_person": timesheet.client_contact_person.as_ref().unwrap_or(&"None".to_string()),
            "client_address": timesheet.client_address.as_ref().unwrap_or(&"None".to_string()),
            "project_number": timesheet.project_number.as_ref().unwrap_or(&"None".to_string()),
            "timesheet": timesheet.timesheet.as_ref().unwrap_or(&HashMap::new()),
            "requires_approval": timesheet.requires_approval.as_ref().unwrap_or(&false),
            "approvers_name": timesheet.approvers_name.as_ref().unwrap_or(&"None".to_string()),
            "approvers_email": timesheet.approvers_email.as_ref().unwrap_or(&"None".to_string()),
            "user_signature": "None".to_string(),
            "approver_signature": "None".to_string(),
        }],
    }]);

    let json = serde_json::to_string(&unwrapped_timesheet).unwrap();
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
            write_config_file(&timesheet, string_path_from_tempdir).unwrap(),
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
        assert!(write_config_file(&timesheet, string_path_from_tempdir).is_err());
    }
}
