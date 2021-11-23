use crate::config::New;
use crate::repository::{GitLogDates, Repository};
use chrono::{DateTime, Datelike};
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::process;
use std::process::Command;
use std::rc::Rc;

/// Repositories are modified at a Repository level and a client level.
/// ClientRepositories  holds the client and the repositories when they are found in the buffer
/// Storing the data here allows the repository  being currently operated on to be cross referenced
/// against all the repos under the same client, and hence generate the correct working hours.

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Client {
    pub client_name: String,
    pub client_address: String,
    pub client_contact_person: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ClientRepositories {
    pub client: Option<Client>,
    pub repositories: Option<Vec<Repository>>,
}

impl Default for ClientRepositories {
    fn default() -> Self {
        Self {
            client: None,
            repositories: None,
        }
    }
}

impl New for ClientRepositories {
    fn new() -> Self {
        Self {
            ..Default::default()
        }
    }
}

impl ClientRepositories {
    pub fn set_values_from_buffer(
        &mut self,
        client_repositories: &ClientRepositories,
    ) -> &mut ClientRepositories {
        *self = client_repositories.clone();
        self
    }

    pub fn exec_generate_timesheets_from_git_history(&mut self) -> &mut Self {
        if let Some(repositories) = &mut self.repositories {
            for repository in repositories {
                let command = String::from("--author");

                // can safely unwrap here as name would have been set in the previous step
                let author = [command, repository.name.as_ref().unwrap().to_string()].join("=");

                let output = Command::new("git")
                    .arg("-C")
                    .arg(repository.git_path.as_ref().unwrap().to_string())
                    .arg("log")
                    .arg("--date=rfc")
                    .arg(author)
                    .arg("--all")
                    .output()
                    .expect("Failed to execute command");

                let output_string = crate::utils::trim_output_from_utf8(output)
                    .unwrap_or_else(|_| "Parsing output failed".to_string());

                repository.parse_git_log_dates_from_git_history(output_string)
            }
        }

        self
    }
}
