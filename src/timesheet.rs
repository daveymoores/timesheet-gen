use crate::date_parser::TimesheetYears;
use crate::timesheet_config::TimesheetConfig;
use crate::utils::{check_for_valid_day, check_for_valid_month, check_for_valid_year};
use chrono::{DateTime, Datelike};
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::process;
use std::process::{Command, Output};
use std::rc::Rc;

pub type GitLogDates = HashMap<i32, HashMap<u32, HashSet<u32>>>;

/// Holds the data from the config file. Config can access these values
// and perform various operations on it

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Timesheet {
    pub namespace: Option<String>,
    pub repo_path: Option<String>,
    pub git_path: Option<String>,
    pub git_log_dates: Option<GitLogDates>,
    pub name: Option<String>,
    pub email: Option<String>,
    pub client_name: Option<String>,
    pub client_contact_person: Option<String>,
    pub client_address: Option<String>,
    pub project_number: Option<String>,
    pub timesheet: Option<TimesheetYears>,
    pub requires_approval: Option<bool>,
    pub approvers_name: Option<String>,
    pub approvers_email: Option<String>,
    pub user_signature: Option<String>,
    pub approver_signature: Option<String>,
}

impl Default for Timesheet {
    fn default() -> Self {
        Self {
            namespace: None,
            repo_path: None,
            git_path: None,
            git_log_dates: None,
            name: None,
            email: None,
            client_name: None,
            client_contact_person: None,
            client_address: None,
            project_number: None,
            timesheet: None,
            requires_approval: None,
            approvers_name: None,
            approvers_email: None,
            user_signature: None,
            approver_signature: None,
        }
    }
}

struct Iter<'a> {
    inner: &'a Timesheet,
    index: u8,
}

impl<'a> Iterator for Iter<'a> {
    type Item = &'a Option<String>;

    fn next(&mut self) -> Option<Self::Item> {
        let ret = match self.index {
            0 => &self.inner.namespace,
            1 => &self.inner.repo_path,
            2 => &self.inner.git_path,
            _ => return None,
        };
        self.index += 1;
        Some(ret)
    }
}

impl Timesheet {
    pub fn new() -> Self {
        Timesheet {
            ..Default::default()
        }
    }

    fn iter(&self) -> Iter<'_> {
        Iter {
            inner: self,
            index: 0,
        }
    }

    /// Get values from buffer and set these to the Timesheet struct fields
    pub fn set_values_from_buffer(&mut self, timesheet: &Timesheet) -> &mut Timesheet {
        *self = timesheet.clone();
        self
    }

    pub fn set_approvers_name(&mut self, value: String) -> &mut Self {
        self.approvers_name = Option::from(value);
        self
    }

    pub fn set_approvers_email(&mut self, value: String) -> &mut Self {
        self.approvers_email = Option::from(value);
        self
    }

    pub fn set_requires_approval(&mut self, value: bool) -> &mut Self {
        self.requires_approval = Option::from(value);
        self
    }

    pub fn set_project_number(&mut self, value: String) -> &mut Self {
        self.project_number = Option::from(value);
        self
    }

    pub fn set_namespace(&mut self, value: String) -> &mut Self {
        self.namespace = Option::from(value);
        self
    }

    pub fn set_repo_path(&mut self, value: String) -> &mut Self {
        self.repo_path = Option::from(value);
        self
    }

    pub fn set_name(&mut self, value: String) -> &mut Self {
        self.name = Option::from(value);
        self
    }

    pub fn set_email(&mut self, value: String) -> &mut Self {
        self.email = Option::from(value);
        self
    }

    pub fn set_client_name(&mut self, value: String) -> &mut Self {
        self.client_name = Option::from(value);
        self
    }

    pub fn set_client_contact_person(&mut self, value: String) -> &mut Self {
        self.client_contact_person = Option::from(value);
        self
    }

    pub fn set_client_address(&mut self, value: String) -> &mut Self {
        self.client_address = Option::from(value);
        self
    }

    pub fn set_timesheet(&mut self, value: TimesheetYears) -> &mut Self {
        self.timesheet = Option::from(value);
        self
    }

    pub fn set_git_path(&mut self, value: String) -> &mut Self {
        self.git_path = Option::from(value);
        self
    }

    pub fn set_git_log_dates(&mut self, value: GitLogDates) -> &mut Self {
        self.git_log_dates = Option::from(value);
        self
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

        self.find_git_path_from_directory(output_path)?;

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

        self.find_repository_details(output_name, output_email)?;

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

    pub fn exec_generate_timesheets_from_git_history(
        &mut self,
        timesheet_config: Rc<RefCell<TimesheetConfig>>,
    ) -> &mut Self {
        let command = String::from("--author");

        // can safely unwrap here as name would have been set in the previous step
        let author = [command, self.name.as_ref().unwrap().to_string()].join("=");
        let output = Command::new("git")
            .arg("-C")
            .arg(self.git_path.as_ref().unwrap().to_string())
            .arg("log")
            .arg("--date=rfc")
            .arg(author)
            .arg("--all")
            .output()
            .expect("Failed to execute command");

        let output_string = crate::utils::trim_output_from_utf8(output)
            .unwrap_or_else(|_| "Parsing output failed".to_string());

        self.parse_git_log_dates_from_git_history(output_string);

        self
    }

    pub fn parse_git_log_dates_from_git_history(&mut self, git_history: String) {
        let mut year_month_map: GitLogDates = HashMap::new();

        let regex = regex::Regex::new(
            r"([a-zA-Z]{3}),\s(?P<day>\d{1,2})\s(?P<month>[a-zA-Z]{3})\s(?P<year>\d{4})\s(\d+:?){3}\s([+-]?\d{4})",
        )
        .unwrap();

        for cap in regex.captures_iter(&git_history) {
            // for each year insert the entry
            // if the value is empty, insert a new hashset, or insert a month into the hashset
            let date_time = DateTime::parse_from_rfc2822(&cap[0]);
            let date = date_time.unwrap().date();

            let year = date.year();
            let month = date.month();
            let day = date.day();

            year_month_map
                .entry(year)
                .and_modify(|year| {
                    year.entry(month)
                        .and_modify(|month| {
                            month.insert(day);
                        })
                        .or_insert_with_key(|_| {
                            let mut x: HashSet<u32> = HashSet::new();
                            x.insert(day);
                            x
                        });
                })
                .or_insert_with_key(|_found_year| {
                    let mut y: HashMap<u32, HashSet<u32>> = HashMap::new();
                    let mut x: HashSet<u32> = HashSet::new();
                    x.insert(day);
                    y.insert(month, x);
                    y
                });
        }

        self.set_git_log_dates(year_month_map);

        let timesheet = match &self.git_log_dates {
            Some(date_map) => {
                crate::date_parser::get_timesheet_map_from_date_hashmap(date_map.clone(), self)
            }
            None => {
                eprintln!("No dates parsed from git log");
                process::exit(exitcode::DATAERR);
            }
        };

        self.set_timesheet(timesheet);
    }

    pub fn mutate_timesheet_entry(
        &mut self,
        year_string: &String,
        month_u32: &u32,
        day: usize,
        entry: Vec<(String, i32)>,
    ) -> Result<&mut Self, Box<dyn std::error::Error>> {
        self.timesheet
            .as_mut()
            .unwrap()
            .get_mut(year_string)
            .ok_or("Passed year not found in timesheet data")?
            .get_mut(&*month_u32.to_string())
            .ok_or("Passed month not found in timesheet data")?[day]
            .extend(entry);

        Ok(self)
    }

    pub fn get_timesheet_entry(
        &self,
        year_string: &String,
        month_u32: &u32,
        day: usize,
        entry: String,
    ) -> Result<Option<&i32>, Box<dyn std::error::Error>> {
        let value = self
            .timesheet
            .as_ref()
            .unwrap()
            .get(year_string)
            .ok_or("Passed year not found in timesheet data")?
            .get(&*month_u32.to_string())
            .ok_or("Passed month not found in timesheet data")?[day]
            .get(&*entry);

        Ok(value)
    }

    pub fn update_hours_on_month_day_entry(
        &mut self,
        options: &Vec<Option<String>>,
    ) -> Result<&mut Self, Box<dyn std::error::Error>> {
        let year_string = check_for_valid_year(&options[3])?;
        let month_u32 = check_for_valid_month(&options[2])?;
        let day_string = check_for_valid_day(&options[1], month_u32, year_string.parse().unwrap())?;

        let hour: i32 = options[0].as_ref().unwrap().parse()?;
        let day: usize = day_string.parse()?;

        let is_weekend =
            match self.get_timesheet_entry(&year_string, &month_u32, day, "weekend".to_string()) {
                Ok(result) => result.cloned().unwrap(),
                Err(err) => {
                    eprintln!("Error retrieving timesheet entry: {}", err);
                    process::exit(exitcode::DATAERR);
                }
            };

        // update hour value
        self.mutate_timesheet_entry(
            &year_string,
            &month_u32,
            day,
            vec![
                ("hours".to_string(), hour),
                ("user_edited".to_string(), 1),
                ("weekend".to_string(), is_weekend),
            ],
        )?;

        Ok(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::os::unix::process::ExitStatusExt;
    use std::process::ExitStatus;

    fn get_mock_year_map() -> TimesheetYears {
        let mut year_map: TimesheetYears = HashMap::new();

        year_map.insert(
            "2021".to_string(),
            vec![(
                "11".to_string(),
                vec![vec![("user_edited".to_string(), 1)]
                    .into_iter()
                    .collect::<HashMap<String, i32>>()],
            )]
            .into_iter()
            .collect::<HashMap<String, Vec<HashMap<String, i32>>>>(),
        );

        year_map
    }

    #[test]
    fn it_sets_values_from_buffer() {
        let mut timesheet = Timesheet {
            ..Default::default()
        };

        let mut ts = Timesheet {
            namespace: Option::from("timesheet-gen".to_string()),
            git_path: Option::from(".".to_string()),
            repo_path: Option::from(
                "/Users/djm/WebstormProjects/rust-projects/timesheet-gen/.git/".to_string(),
            ),
            ..Default::default()
        };

        timesheet.set_values_from_buffer(&mut ts);
        let x: Vec<&String> = ts.iter().map(|y| y.as_ref().unwrap()).collect();
        println!("{:?}", x);
        assert_eq!(
            x,
            [
                &"timesheet-gen".to_string(),
                &"/Users/djm/WebstormProjects/rust-projects/timesheet-gen/.git/".to_string(),
                &".".to_string(),
            ]
        );
    }

    #[test]
    fn it_mutates_timesheet_entry() {
        let mut ts = Timesheet {
            ..Default::default()
        };

        let year_map = get_mock_year_map();
        ts.set_timesheet(year_map);

        ts.mutate_timesheet_entry(
            &"2021".to_string(),
            &11,
            0,
            vec![("user_edited".to_string(), 0)],
        )
        .unwrap();

        assert_eq!(
            ts.get_timesheet_entry(&"2021".to_string(), &11, 0, "user_edited".to_string())
                .unwrap()
                .unwrap(),
            &0 as &i32
        );
    }

    #[test]
    fn it_gets_timesheet_entry() {
        let mut ts = Timesheet {
            ..Default::default()
        };

        let year_map = get_mock_year_map();
        ts.set_timesheet(year_map);

        assert_eq!(
            ts.get_timesheet_entry(&"2021".to_string(), &11, 0, "user_edited".to_string())
                .unwrap()
                .unwrap(),
            &1 as &i32
        );
    }

    #[test]
    fn it_parses_git_log_dates_from_git_history() {
        let mut timesheet = Timesheet {
            ..Default::default()
        };

        let std_output = "commit c2c1354f6e73073f6eb9a2273c550a38f0e624d7
Author: Davey Moores <daveymoores@gmail.com>
Date:   Sat, 23 Oct 2021 13:02:36 +0200

    getting month, year and number of days in month from date string

commit bad43e994462238b0470fae8c5af6f1f7d544e18 (origin/feature/redirect-to-onboarding, feature/redirect-to-onboarding)
Author: Davey Moores <daveymoores@gmail.com>
Date:   Thu, 21 Oct 2021 10:06:14 +0200

    testing that it writes to the config file

commit 6604ce77b0dce8f842ea72ca52b3d39212668389
Author: Davey Moores <daveymoores@gmail.com>
Date:   Wed, 20 Oct 2021 12:09:16 +0200

    write data to file

commit 9bc3e9720963d6aa06c1fd64cf826c8a0a6570a4
Author: Davey Moores <daveymoores@gmail.com>
Date:   Wed, 20 Oct 2021 11:06:17 +0200

    initialise if config isn't found

commit 9bc3e9720963d6aa06c1fd64cf826c8a0a6570a4
Author: Davey Moores <daveymoores@gmail.com>
Date:   Wed, 08 Sep 2021 11:06:17 +0200

    initialise if config isn't found

commit 9bc3e9720963d6aa06c1fd64cf826c8a0a6570a4
Author: Davey Moores <daveymoores@gmail.com>
Date:   Sat, 1 Aug 2020 11:06:17 +0200

    initialise if config isn't found

commit 9bc3e9720963d6aa06c1fd64cf826c8a0a6570a4
Author: Davey Moores <daveymoores@gmail.com>
Date:   Thu, 3 Jan 2019 11:06:17 +0200

    initialise if config isn't found
".to_string();

        timesheet.parse_git_log_dates_from_git_history(std_output);
        let x = timesheet.git_log_dates.unwrap();

        // to check the hashmap shape is correct, lets create an array
        // of the numeric values and order them. Not great but snapshot testing with hashmaps isn't a thing in rust...
        let mut k = vec![];
        for (key, value) in x.into_iter() {
            k.push(key.clone());
            for (key, value) in value.into_iter() {
                k.push(key as i32);
                let x = value.into_iter().map(|x| x).collect::<Vec<u32>>();

                for y in x {
                    k.push(y as i32);
                }
            }
        }

        // sort them as hashmaps and hashsets don't have an order
        k.sort();

        let expected_array: Vec<i32> = vec![1, 1, 3, 8, 8, 9, 10, 20, 21, 23, 2019, 2020, 2021];
        assert_eq!(k, expected_array);
    }

    #[test]
    fn it_finds_namespace_from_git_path() {
        let mut timesheet = Timesheet {
            ..Default::default()
        };

        timesheet.set_git_path("/rust/timesheet-gen/.git/".to_string());
        timesheet.find_namespace_from_git_path().unwrap();
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

        timesheet.find_git_path_from_directory(output_path).unwrap();
        assert_eq!(
            timesheet.git_path.unwrap(),
            "/Users/djm/WebstormProjects/rust-projects/timesheet-gen/.git/".to_string()
        );
    }

    #[test]
    fn it_sets_requires_approval() {
        let mut timesheet = Timesheet {
            ..Default::default()
        };

        timesheet.set_requires_approval(true);
        assert_eq!(timesheet.requires_approval.unwrap(), true);
    }

    #[test]
    fn it_sets_approvers_email() {
        let mut timesheet = Timesheet {
            ..Default::default()
        };

        timesheet.set_approvers_email("approver@gmail.com".to_string());
        assert_eq!(
            timesheet.approvers_email.unwrap(),
            "approver@gmail.com".to_string()
        );
    }

    #[test]
    fn it_sets_approvers_name() {
        let mut timesheet = Timesheet {
            ..Default::default()
        };

        timesheet.set_approvers_name("Mr Approver".to_string());
        assert_eq!(timesheet.approvers_name.unwrap(), "Mr Approver".to_string());
    }

    #[test]
    fn it_sets_project_number() {
        let mut timesheet = Timesheet {
            ..Default::default()
        };

        timesheet.set_project_number("Project number".to_string());
        assert_eq!(
            timesheet.project_number.unwrap(),
            "Project number".to_string()
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
    fn it_sets_timesheet() {
        let mut ts = Timesheet {
            ..Default::default()
        };

        let mut year_map: TimesheetYears = HashMap::new();
        year_map.insert(
            "foo".to_string(),
            vec![(
                "bar".to_string(),
                vec![vec![("baz".to_string(), 8)]
                    .into_iter()
                    .collect::<HashMap<String, i32>>()],
            )]
            .into_iter()
            .collect::<HashMap<String, Vec<HashMap<String, i32>>>>(),
        );

        ts.set_timesheet(year_map);
        assert!(ts.timesheet.clone().unwrap().contains_key("foo"));
        assert_eq!(
            json!(ts.timesheet.clone()).to_string(),
            "{\"foo\":{\"bar\":[{\"baz\":8}]}}"
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
