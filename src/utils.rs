use chrono::NaiveDate;
use dotenv::dotenv;
use random_string::generate;
use regex::Regex;
use std::env;
use std::error::Error;
use std::io;
use std::io::ErrorKind;
use std::process::Output;

pub fn is_test_mode() -> bool {
    dotenv().ok();
    let test_mode = env::var("TEST_MODE").expect("TEST MODE not set");
    test_mode.parse::<bool>().unwrap()
}

pub fn exit_process() {
    if !is_test_mode() {
        std::process::exit(exitcode::OK);
    }
}

pub fn trim_output_from_utf8(output: Output) -> Result<String, Box<dyn std::error::Error>> {
    let x = String::from_utf8(output.stdout)?.trim().parse().unwrap();
    Ok(x)
}

pub fn generate_random_path() -> String {
    let charset = "0123456789abcdefghijklmnopqrstuvwxyz";
    generate(10, charset)
}

pub fn get_days_from_month(year: i32, month: u32) -> u32 {
    NaiveDate::from_ymd(
        match month {
            12 => year + 1,
            _ => year,
        },
        match month {
            12 => 1,
            _ => month + 1,
        },
        1,
    )
    .signed_duration_since(NaiveDate::from_ymd(year, month, 1))
    .num_days() as u32
}

pub fn check_for_valid_month(month: &Option<String>) -> Result<u32, Box<dyn Error>> {
    let month_u32 = month
        .as_ref()
        .ok_or_else(|| io::Error::new(ErrorKind::InvalidData, format!("Month not found")))?
        .parse::<u32>()?;

    let month_regex = Regex::new(r"^(1[0-2]|[1-9])$").unwrap();

    if !month_regex.is_match(&*month_u32.to_string()) {
        return Err("Not a real month".into());
    }

    Ok(month_u32)
}

pub fn check_for_valid_year(year: &Option<String>) -> Result<&String, Box<dyn Error>> {
    let year_string = year
        .as_ref()
        .ok_or_else(|| io::Error::new(ErrorKind::InvalidData, format!("Year not found")))?;

    let year_regex = Regex::new(r"^((19|20)\d{2})$").unwrap();

    if !year_regex.is_match(year_string) {
        // into implements a conversion between &str and Box<dyn Error>
        return Err("Not a real year".into());
    }

    Ok(year_string)
}

pub fn check_for_valid_day(
    day: &Option<String>,
    month: u32,
    year: i32,
) -> Result<&String, Box<dyn Error>> {
    let day_string = day
        .as_ref()
        .ok_or_else(|| io::Error::new(ErrorKind::InvalidData, format!("Day not found")))?;

    let days_in_month = get_days_from_month(year, month);
    let day_regex = Regex::new(r"^(3[0-1]|2[0-9]|1[0-9]|[1-9])$").unwrap();

    if !day_regex.is_match(day_string) {
        // into implements a conversion between &str and Box<dyn Error>
        return Err("Day index in the month doesn't exist".into());
    }

    if days_in_month < day_string.parse().unwrap() {
        return Err("The day given doesn't exist for the given month/year".into());
    }

    Ok(day_string)
}

pub fn config_file_found(buffer: &mut String) -> bool {
    !buffer.is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;
    use envtestkit::lock::lock_test;
    use envtestkit::set_env;
    use std::ffi::OsString;
    use std::os::unix::process::ExitStatusExt;
    use std::process::{ExitStatus, Output};

    #[test]
    fn it_returns_test_mode_is_true() {
        let _lock = lock_test();
        let _test = set_env(OsString::from("TEST_MODE"), "true");

        assert_eq!(is_test_mode(), true);
    }

    #[test]
    fn it_returns_test_mode_is_false() {
        let _lock = lock_test();
        let _test = set_env(OsString::from("TEST_MODE"), "false");

        assert_eq!(is_test_mode(), false);
    }

    #[test]
    fn should_return_true_if_config_file_is_found() {
        let mut buffer = String::new();
        assert_eq!(config_file_found(&mut buffer), false);
    }

    #[test]
    fn should_return_false_if_config_file_is_not_found() {
        let mut buffer = String::from("buffer is full");
        assert_eq!(config_file_found(&mut buffer), true);
    }

    #[test]
    fn it_checks_for_valid_day_and_throws() {
        // days is a string
        assert!(check_for_valid_day(&Option::from("foo".to_string()), 10, 2021).is_err());
        // days don't exist for any month
        assert!(check_for_valid_day(&Option::from("32".to_string()), 10, 2021).is_err());
        // days don't exist for november
        assert!(check_for_valid_day(&Option::from("31".to_string()), 11, 2021).is_err());
    }

    #[test]
    fn it_checks_for_valid_day() {
        assert!(check_for_valid_day(&Option::from("31".to_string()), 10, 2021).is_ok());
    }

    #[test]
    fn it_checks_for_valid_month_and_throws() {
        // month is a string that can't be parsed
        assert!(check_for_valid_month(&Option::from("foo".to_string())).is_err());
        // month doesn't exist
        assert!(check_for_valid_month(&Option::from("99".to_string())).is_err());
        // month doesn't exist
        assert!(check_for_valid_month(&Option::from("0".to_string())).is_err());
    }

    #[test]
    fn it_checks_for_valid_month() {
        assert!(check_for_valid_month(&Option::from("12".to_string())).is_ok());
    }

    #[test]
    fn it_checks_for_valid_year_and_throws() {
        // year is a string that can't be parsed
        assert!(check_for_valid_year(&Option::from("foo".to_string())).is_err());
        // year doesn't exist
        assert!(check_for_valid_year(&Option::from("3000".to_string())).is_err());
        // year is unlikely
        assert!(check_for_valid_year(&Option::from("1898".to_string())).is_err());
    }

    #[test]
    fn it_checks_for_valid_year() {
        assert!(check_for_valid_year(&Option::from("1998".to_string())).is_ok());
        assert!(check_for_valid_year(&Option::from("2020".to_string())).is_ok());
        assert!(check_for_valid_year(&Option::from("2099".to_string())).is_ok());
    }

    #[test]
    fn it_trims_output_from_utf8() {
        let output_path = Output {
            status: ExitStatus::from_raw(0),
            stdout: vec![68, 97, 118, 101, 121, 32, 77, 111, 111, 114, 101, 115, 10],
            stderr: vec![],
        };

        assert_eq!(trim_output_from_utf8(output_path).unwrap(), "Davey Moores");
    }

    #[test]
    fn it_generates_a_random_string() {
        let random_string = generate_random_path();
        let regex = regex::Regex::new(r"^[a-z0-9]{10}$");
        match regex.unwrap().find(&*random_string) {
            Some(_x) => assert!(true),
            None => panic!("Pattern not matched"),
        }
    }

    #[test]
    fn it_finds_the_number_of_days_for_a_specific_month_and_year() {
        assert_eq!(get_days_from_month(2021, 10), 31);
        assert_eq!(get_days_from_month(1989, 2), 28);
        assert_eq!(get_days_from_month(1945, 6), 30);
        // leap year
        assert_eq!(get_days_from_month(2024, 2), 29);
    }
}
