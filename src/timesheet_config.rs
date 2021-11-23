use crate::config::New;
use crate::timesheet::Timesheet;
use serde::{Deserialize, Serialize};

/// Timesheets are modified at a Timesheet level and a client level.
/// TimesheetConfig holds the client and the repositories when they are found in the buffer
/// Storing the data here allows the timesheet being currently operated on to be cross referenced
/// against all the repos under the same client, and hence generate the correct working hours.

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Client {
    pub client_name: String,
    pub client_address: String,
    pub client_contact_person: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TimesheetConfig {
    pub client: Option<Client>,
    pub repositories: Option<Vec<Timesheet>>,
}

impl Default for TimesheetConfig {
    fn default() -> Self {
        Self {
            client: None,
            repositories: None,
        }
    }
}

impl New for TimesheetConfig {
    fn new() -> Self {
        Self {
            ..Default::default()
        }
    }
}
