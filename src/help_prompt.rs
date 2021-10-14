use crate::timesheet::Timesheet;
/// Help prompt handles all of the interactions with the user.
/// It writes to the std output, and returns input data or a boolean
use dialoguer::{Confirm, Input};
use std::cell::RefMut;

pub struct HelpPrompt {}

impl HelpPrompt {
    pub fn onboarding(mut timesheet: RefMut<Timesheet>) -> Result<(), std::io::Error> {
        println!(
            "This looks like the first time you're running timesheet-gen. \n\
        Initialise timesheet-gen for current repository?"
        );

        if Confirm::new().default(true).interact()? {
            timesheet.repo_path =
                Option::from(String::from("Did this get updated and later referenced?"));
        } else {
            println!("Please give a path to the repository you would like to use:");

            let path = crate::file_reader::get_home_path()
                .to_str()
                .unwrap()
                .to_string();

            let input: String = Input::new().with_initial_text(path).interact_text()?;

            // Here this data needs to be pushed into state somehow
            println!("{}", input);
        }

        Ok(())
    }

    //pub fn add_client_details() {}
}
