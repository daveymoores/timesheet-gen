/// Holds the data from the config file. Config can access these values
// and perform various operations on it

#[derive(Debug)]
pub struct Timesheet {
    pub repo_path: Option<String>,
}
