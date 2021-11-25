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
pub struct User {
    pub name: String,
    pub email: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ClientRepositories {
    pub client: Option<Client>,
    pub user: Option<User>,
    pub repositories: Option<Vec<Repository>>,
}

impl Default for ClientRepositories {
    fn default() -> Self {
        Self {
            client: None,
            user: None,
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

                repository.parse_git_log_dates_from_git_history(output_string);
            }
        }

        self
    }

    pub fn compare_logs_and_set_timesheets(&mut self) -> &mut Self {
        if let Some(repositories) = &mut self.repositories {
            for i in 0..repositories.len() {
                // for each repository, build a vec of the git_log_dates from the other repositories
                let adjacent_git_log_dates: Vec<GitLogDates> = repositories
                    .into_iter()
                    .enumerate()
                    .filter(|(index, _)| index != &i)
                    .map(|(_, repo)| repo.git_log_dates.as_ref().unwrap().clone())
                    .collect();

                let timesheet = match &repositories[i].git_log_dates {
                    Some(git_log_dates) => crate::date_parser::get_timesheet_map_from_date_hashmap(
                        git_log_dates.clone(),
                        &mut repositories[i],
                        adjacent_git_log_dates,
                    ),
                    None => {
                        eprintln!("No dates parsed from git log");
                        process::exit(exitcode::DATAERR);
                    }
                };

                repositories[i].set_timesheet(timesheet);
            }
        }

        self
    }
}

#[cfg(test)]
mod tests {
    use crate::client_repositories::{Client, ClientRepositories, User};
    use crate::date_parser::TimesheetYears;
    use crate::repository::{GitLogDates, Repository};
    use serde_json::{Number, Value};
    use std::collections::{HashMap, HashSet};

    // Generate git log dates that overlap by one day each month to test hours being split equally
    fn generate_project_git_log_dates(days: [u32; 3]) -> GitLogDates {
        HashMap::from([
            (
                2019,
                HashMap::from([(1, HashSet::from(days)), (2, HashSet::from(days))]),
            ),
            (
                2020,
                HashMap::from([(5, HashSet::from(days)), (2, HashSet::from(days))]),
            ),
            (
                2021,
                HashMap::from([(9, HashSet::from(days)), (2, HashSet::from(days))]),
            ),
        ])
    }

    #[test]
    fn it_compares_git_logs_and_sets_timesheets() {
        let mut client_repositories: ClientRepositories = ClientRepositories {
            client: Option::Some(Client {
                client_name: "Alphabet".to_string(),
                client_address: "Alphabet way".to_string(),
                client_contact_person: "John Jones".to_string(),
            }),
            user: Option::Some(User {
                name: "Jim Jones".to_string(),
                email: "jim@jones.com".to_string(),
            }),
            repositories: Option::Some(vec![
                Repository {
                    client_name: Option::Some("Alphabet".to_string()),
                    namespace: Option::Some("Project_1".to_string()),
                    git_log_dates: Option::Some(generate_project_git_log_dates([1, 2, 3])),
                    ..Default::default()
                },
                Repository {
                    client_name: Option::Some("Alphabet".to_string()),
                    namespace: Option::Some("Project_2".to_string()),
                    git_log_dates: Option::Some(generate_project_git_log_dates([2, 3, 4])),
                    ..Default::default()
                },
                Repository {
                    client_name: Option::Some("Alphabet".to_string()),
                    namespace: Option::Some("Project_3".to_string()),
                    git_log_dates: Option::Some(generate_project_git_log_dates([3, 4, 5])),
                    ..Default::default()
                },
            ]),
        };

        client_repositories.compare_logs_and_set_timesheets();

        let repositories = client_repositories.repositories.unwrap();
        // Check project 1 has hours split on overlapping days
        let repository = repositories[0].clone();
        let timesheet = &repository
            .timesheet
            .as_ref()
            .unwrap()
            .get("2021")
            .unwrap()
            .get("2")
            .unwrap()[0..3];

        let ts = timesheet
            .into_iter()
            .map(|day| day.get("hours").unwrap().clone().as_f64().unwrap())
            .collect::<Vec<f64>>();

        assert_eq!(ts, vec![8.0, 4.0, 2.6666666666666665]);
    }
}
