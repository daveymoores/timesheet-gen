use std::env::JoinPathsError;
use std::fs::File;
use std::io;
use std::io::{BufReader, ErrorKind, Read};
use std::path::PathBuf;

const CONFIG_FILE_NAME: &str = ".timesheet-gen.txt";

fn get_home_path() -> PathBuf {
    match dirs::home_dir() {
        Some(dir) => dir,
        None => panic!("Home directory not found"),
    }
}

fn get_filepath(path: PathBuf) -> String {
    let home_path = path.to_str();
    home_path.unwrap().to_owned() + "/" + CONFIG_FILE_NAME
}

fn read_file(buffer: &mut String, path: String, error_fn: fn()) -> Result<&mut String, io::Error> {
    match File::open(&path) {
        Ok(mut file) => {
            file.read_to_string(buffer)?;
        }
        Err(err) => {
            println!("{:?}", err);
            error_fn();
        }
    };

    Ok(buffer)
}

pub fn read_data_from_config_file(
    buffer: &mut String,
    error_fn: fn(),
) -> Result<&mut String, io::Error> {
    let config_path = get_filepath(get_home_path());
    let filled_buffer: &mut String = read_file(buffer, config_path, error_fn)?;

    Ok(filled_buffer)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;
    use std::process::Command;

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
        fn mock_error_fn() {
            assert!(false);
        }
        let mut buffer = String::new();
        let file_data = read_file(
            &mut buffer,
            String::from("./testing-utils/.timesheet-gen.txt"),
            mock_error_fn,
        )
        .unwrap();

        assert_eq!(file_data.trim(), "hello");
    }

    #[test]
    fn read_file_calls_the_error_function() {
        fn mock_error_fn() {
            assert!(true);
        }

        let mut buffer = String::new();
        read_file(
            &mut buffer,
            String::from("./testing-utils/.timesheet-gen.txt"),
            mock_error_fn,
        );
    }
}
