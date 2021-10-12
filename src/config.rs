/// Creates and modifies the config file. Config does not directly hold the information
/// contained in the config file, but provides the various operations that can be
/// performed on it. The data is a stored within the Timesheet struct.
pub struct Config {}

pub trait New {
    fn new() -> Config {
        Config {}
    }
}

impl New for Config {}

pub trait Init {
    /// Generate a config file with user variables
    fn init(&self, options: Vec<Option<String>>);
}

impl Init for Config {
    fn init(&self, _options: Vec<Option<String>>) {}
}

pub trait Make {
    /// Edit a day entry within the timesheet
    fn make(&self, options: Vec<Option<String>>);
}

impl Make for Config {
    fn make(&self, _options: Vec<Option<String>>) {}
}

pub trait Edit {
    /// Generate a config file with user variables
    fn edit(&self, options: Vec<Option<String>>);
}

impl Edit for Config {
    fn edit(&self, _options: Vec<Option<String>>) {}
}

pub trait Remove {
    /// Remove an entry within the timesheet
    fn remove(&self, options: Vec<Option<String>>);
}

impl Remove for Config {
    fn remove(&self, _options: Vec<Option<String>>) {}
}

pub trait RunMode {
    /// Specify a run mode
    fn run_mode(&self, options: Vec<Option<String>>);
}

impl RunMode for Config {
    fn run_mode(&self, _options: Vec<Option<String>>) {}
}
