use crate::help_prompt::HelpPrompt;
use crate::timesheet::Timesheet;
use std::cell::RefMut;
use std::fs::File;
use std::io;
use std::io::Read;
use std::path::PathBuf;

const CONFIG_FILE_NAME: &str = ".timesheet-gen.txt";

/// Find the path to the users home directory
pub fn get_home_path() -> PathBuf {
    match dirs::home_dir() {
        Some(dir) => dir,
        None => panic!("Home directory not found"),
    }
}

/// Create filepath to config file
fn get_filepath(path: PathBuf) -> String {
    let home_path = path.to_str();
    home_path.unwrap().to_owned() + "/" + CONFIG_FILE_NAME
}

/// Read config file or throw error and call error function
fn read_file(buffer: &mut String, path: String, prompt: HelpPrompt) -> Result<(), io::Error> {
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

pub fn read_data_from_config_file(
    buffer: &mut String,
    prompt: HelpPrompt,
) -> Result<(), io::Error> {
    let config_path = get_filepath(get_home_path());
    read_file(buffer, config_path, prompt)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::path::Path;
    use std::process::Command;
    use std::rc::Rc;

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
        let mock_error_fn: Box<dyn FnMut() -> Result<(), std::io::Error>> = Box::new(|| {
            assert!(false);
            Ok(())
        });

        let timesheet = Rc::new(RefCell::new(Timesheet {
            namespace: None,
            repo_path: None,
            name: None,
            email: None,
            client_name: None,
            client_contact_person: None,
            client_address: None,
            po_number: None,
            timesheet: None,
        }));
        let mut_timesheet = timesheet.borrow_mut();
        let mut buffer = String::new();

        read_file(
            &mut buffer,
            String::from("./testing-utils/.timesheet-gen.txt"),
            mock_error_fn,
        )
        .unwrap();

        assert_eq!(buffer.trim(), "hello");
    }

    #[test]
    fn read_file_calls_the_error_function() {
        let mock_error_fn: Box<dyn FnMut() -> Result<(), std::io::Error>> = Box::new(|| {
            assert!(true);
            Ok(())
        });

        let timesheet = Rc::new(RefCell::new(Timesheet {
            namespace: None,
            repo_path: None,
            name: None,
            email: None,
            client_name: None,
            client_contact_person: None,
            client_address: None,
            po_number: None,
            timesheet: None,
        }));
        let mut_timesheet = timesheet.borrow_mut();
        let mut buffer = String::new();

        read_file(
            &mut buffer,
            String::from("./testing-utils/.timesheet-gen.txt"),
            mock_error_fn,
        );
    }
}
