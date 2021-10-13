/// Help prompt handles all of the interactions with the user.
/// It writes to the std output, and returns a input data or a boolean
use dialoguer::{Confirm, Input};

pub struct HelpPrompt {}

impl HelpPrompt {
    pub fn new() -> Self {
        Self {}
    }

    pub fn onboarding() -> Result<(), std::io::Error> {
        println!(
            "This looks like the first time you're running timesheet-gen. \n\
        Initialise timesheet-gen for current repository?"
        );

        if Confirm::new().default(true).interact()? {
            println!("Looks like you want to continue");
        } else {
            println!("Please give a path to the repository you would like to use:");

            let path = crate::file_reader::get_home_path()
                .to_str()
                .unwrap()
                .to_string();

            let input: String = Input::new().with_initial_text(path).interact_text()?;
            println!("{}", input);
        }

        Ok(())
    }
}
