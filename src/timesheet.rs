use serde_json::{Map, Value};

/// Holds the data from the config file. Config can access these values
// and perform various operations on it

#[derive(Debug)]
pub struct Timesheet {
    pub namespace: Option<String>,
    pub repo_path: Option<String>,
    pub name: Option<String>,
    pub email: Option<String>,
    pub client_name: Option<String>,
    pub client_contact_person: Option<String>,
    pub client_address: Option<String>,
    pub po_number: Option<String>,
    pub timesheet: Option<Map<String, Value>>,
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
        self.name = Option::from(value);
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
}
