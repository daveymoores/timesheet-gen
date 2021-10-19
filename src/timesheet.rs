use regex::{Captures, Match};
use serde_json::{Map, Value};
use std::path::Path;
use std::process;
use std::process::Command;

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

    pub fn find_git_path_from_directory(
        &mut self,
    ) -> Result<&mut Self, Box<dyn std::error::Error>> {
        let output_path = Command::new("git")
            .arg("-C")
            .arg(&self.repo_path.clone().unwrap())
            .arg("rev-parse")
            .arg("--show-toplevel")
            .output()
            .expect("Failed to find 'git_path'");

        let path_string = String::from_utf8(output_path.stdout)?;

        self.set_git_path(path_string.trim().to_owned() + &*String::from("/.git/").to_owned());

        Ok(self)
    }

    pub fn find_repository_details(&mut self) -> Result<&mut Self, Box<dyn std::error::Error>> {
        let output_name = Command::new("git")
            .arg("-C")
            .arg(&self.repo_path.clone().unwrap())
            .arg("config")
            .arg("user.name")
            .output()
            .expect("Failed to find 'name'");

        let output_email = Command::new("git")
            .arg("-C")
            .arg(&self.repo_path.clone().unwrap())
            .arg("config")
            .arg("user.email")
            .output()
            .expect("Failed to find 'email'");

        self.set_name(String::from_utf8(output_name.stdout)?);
        self.set_email(String::from_utf8(output_email.stdout)?);

        self.find_git_path_from_directory()?
            .find_namespace_from_git_path()?;

        Ok(self)
    }
}
