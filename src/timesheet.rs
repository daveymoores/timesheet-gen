use regex::{Captures, Match};
use serde_json::{Map, Value};
use std::cell::RefCell;
use std::path::Path;
use std::process;
use std::process::{Command, Output};
use std::rc::Rc;

/// Holds the data from the config file. Config can access these values
// and perform various operations on it

#[derive(Debug)]
pub struct Timesheet {
    pub namespace: Option<String>,
    pub repo_path: Option<String>,
    pub git_path: Option<String>,
    pub name: Option<String>,
    pub email: Option<String>,
    pub client_name: Option<String>,
    pub client_contact_person: Option<String>,
    pub client_address: Option<String>,
    pub po_number: Option<String>,
    pub timesheet: Option<Map<String, Value>>,
}

impl Default for Timesheet {
    fn default() -> Self {
        Self {
            namespace: None,
            repo_path: None,
            git_path: None,
            name: None,
            email: None,
            client_name: None,
            client_contact_person: None,
            client_address: None,
            po_number: None,
            timesheet: None,
        }
    }
}

impl Timesheet {
    pub fn set_namespace(&mut self, value: String) {
        self.namespace = Option::from(value);
    }

    pub fn set_repo_path(&mut self, value: String) {
        self.repo_path = Option::from(value);
    }

    pub fn set_name(&mut self, value: String) {
        self.name = Option::from(value);
    }

    pub fn set_email(&mut self, value: String) {
        self.email = Option::from(value);
    }

    pub fn set_client_name(&mut self, value: String) {
        self.client_name = Option::from(value);
    }

    pub fn set_client_contact_person(&mut self, value: String) {
        self.client_contact_person = Option::from(value);
    }

    pub fn set_client_address(&mut self, value: String) {
        self.client_address = Option::from(value);
    }

    pub fn set_po_number(&mut self, value: String) {
        self.po_number = Option::from(value);
    }

    pub fn set_timesheet(&mut self, value: Map<String, Value>) {
        self.timesheet = Option::from(value);
    }

    pub fn set_git_path(&mut self, value: String) {
        self.git_path = Option::from(value);
    }

    pub fn find_namespace_from_git_path(
        &mut self,
    ) -> Result<&mut Self, Box<dyn std::error::Error>> {
        let reg = regex::Regex::new(r"(?P<namespace>[^/][\w\d()_\-,.]+)/\.git/")?;

        match reg.captures(&self.git_path.clone().unwrap().as_str()) {
            None => {
                println!("No regex matches against git path. Please check that the path contains valid characters");
                process::exit(exitcode::DATAERR);
            }
            Some(cap) => match cap.name("namespace") {
                None => {
                    println!("No matches found for project namespace");
                    process::exit(exitcode::DATAERR);
                }
                Some(capture) => {
                    self.set_namespace((&capture.as_str()).parse().unwrap());
                }
            },
        }

        Ok(self)
    }

    pub fn find_git_path_from_directory_from(
        &mut self,
    ) -> Result<&mut Self, Box<dyn std::error::Error>> {
        let output_path = Command::new("git")
            .arg("-C")
            .arg(&self.repo_path.clone().unwrap())
            .arg("rev-parse")
            .arg("--show-toplevel")
            .output()
            .expect("Failed to find 'git_path'");

        self.find_git_path_from_directory(output_path);

        Ok(self)
    }

    pub fn find_git_path_from_directory(
        &mut self,
        output_path: Output,
    ) -> Result<&mut Self, Box<dyn std::error::Error>> {
        let path_string: String = crate::utils::trim_output_from_utf8(output_path)?;

        self.set_git_path(path_string.to_owned() + &*String::from("/.git/").to_owned());

        Ok(self)
    }

    pub fn find_repository_details_from(
        &mut self,
    ) -> Result<&mut Self, Box<dyn std::error::Error>> {
        let output_name = Command::new("git")
            .arg("-C")
            .arg(&self.repo_path.clone().unwrap())
            .arg("config")
            .arg("user.name")
            .output()
            .expect("Failed to find 'user.name'");

        let output_email = Command::new("git")
            .arg("-C")
            .arg(&self.repo_path.clone().unwrap())
            .arg("config")
            .arg("user.email")
            .output()
            .expect("Failed to find 'user.email'");

        self.find_repository_details(output_name, output_email);

        Ok(self)
    }

    pub fn find_repository_details(
        &mut self,
        output_name: Output,
        output_email: Output,
    ) -> Result<&mut Self, Box<dyn std::error::Error>> {
        self.set_name(crate::utils::trim_output_from_utf8(output_name)?);
        self.set_email(crate::utils::trim_output_from_utf8(output_email)?);

        self.find_git_path_from_directory_from()?
            .find_namespace_from_git_path()?;

        Ok(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::process::ExitStatusExt;
    use std::process::ExitStatus;

    #[test]
    fn it_finds_namespace_from_git_path() {
        let mut timesheet = Timesheet {
            ..Default::default()
        };

        timesheet.set_git_path("/rust/timesheet-gen/.git/".to_string());
        timesheet.find_namespace_from_git_path();
        assert_eq!(timesheet.namespace.unwrap(), "timesheet-gen".to_string());
    }

    #[test]
    fn it_finds_git_path_from_directory() {
        let mut timesheet = Timesheet {
            ..Default::default()
        };

        let output_path = Output {
            status: ExitStatus::from_raw(0),
            stdout: vec![
                47, 85, 115, 101, 114, 115, 47, 100, 106, 109, 47, 87, 101, 98, 115, 116, 111, 114,
                109, 80, 114, 111, 106, 101, 99, 116, 115, 47, 114, 117, 115, 116, 45, 112, 114,
                111, 106, 101, 99, 116, 115, 47, 116, 105, 109, 101, 115, 104, 101, 101, 116, 45,
                103, 101, 110, 10,
            ],
            stderr: vec![],
        };

        timesheet.find_git_path_from_directory(output_path);
        assert_eq!(
            timesheet.git_path.unwrap(),
            "/Users/djm/WebstormProjects/rust-projects/timesheet-gen/.git/".to_string()
        );
    }

    #[test]
    fn it_sets_namespace() {
        let mut timesheet = Timesheet {
            ..Default::default()
        };

        timesheet.set_namespace("namespace".to_string());
        assert_eq!(timesheet.namespace.unwrap(), "namespace".to_string());
    }

    #[test]
    fn it_sets_repo_path() {
        let mut timesheet = Timesheet {
            ..Default::default()
        };

        timesheet.set_repo_path("repo_path".to_string());
        assert_eq!(timesheet.repo_path.unwrap(), "repo_path".to_string());
    }

    #[test]
    fn it_sets_name() {
        let mut timesheet = Timesheet {
            ..Default::default()
        };

        timesheet.set_name("name".to_string());
        assert_eq!(timesheet.name.unwrap(), "name".to_string());
    }

    #[test]
    fn it_sets_email() {
        let mut timesheet = Timesheet {
            ..Default::default()
        };

        timesheet.set_email("email".to_string());
        assert_eq!(timesheet.email.unwrap(), "email".to_string());
    }

    #[test]
    fn it_sets_client_name() {
        let mut timesheet = Timesheet {
            ..Default::default()
        };

        timesheet.set_client_name("client name".to_string());
        assert_eq!(timesheet.client_name.unwrap(), "client name".to_string());
    }

    #[test]
    fn it_sets_client_contact_person() {
        let mut timesheet = Timesheet {
            ..Default::default()
        };

        timesheet.set_client_contact_person("client contact person".to_string());
        assert_eq!(
            timesheet.client_contact_person.unwrap(),
            "client contact person".to_string()
        );
    }

    #[test]
    fn it_sets_client_address() {
        let mut timesheet = Timesheet {
            ..Default::default()
        };

        timesheet.set_client_address("client address".to_string());
        assert_eq!(
            timesheet.client_address.unwrap(),
            "client address".to_string()
        );
    }

    #[test]
    fn it_sets_po_number() {
        let mut timesheet = Timesheet {
            ..Default::default()
        };

        timesheet.set_po_number("po number".to_string());
        assert_eq!(timesheet.po_number.unwrap(), "po number".to_string());
    }

    #[test]
    fn it_sets_timesheet() {
        let mut ts = Timesheet {
            ..Default::default()
        };

        let mut map = Map::new();
        map.insert("foo".to_string(), Value::from("bar"));

        ts.set_timesheet(map);
        assert!(ts.timesheet.clone().unwrap().contains_key("foo"));
        assert_eq!(
            ts.timesheet
                .clone()
                .unwrap()
                .values()
                .into_iter()
                .next()
                .unwrap()
                .to_owned(),
            Value::from("bar")
        );
    }

    #[test]
    fn it_sets_git_path() {
        let mut timesheet = Timesheet {
            ..Default::default()
        };

        timesheet.set_git_path("/path/to/string".to_string());
        assert_eq!(timesheet.git_path.unwrap(), "/path/to/string".to_string());
    }
}
