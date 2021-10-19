use crate::timesheet::Timesheet;
/// Help prompt handles all of the interactions with the user.
/// It writes to the std output, and returns input data or a boolean
use dialoguer::{Confirm, Editor, Input};
use std::cell::{RefCell, RefMut};
use std::error::Error;
use std::rc::Rc;

pub struct HelpPrompt {
    timesheet: Rc<RefCell<Timesheet>>,
}

pub trait Onboarding {
    fn onboarding(self) -> Result<(), Box<dyn std::error::Error>>;
}

impl Onboarding for HelpPrompt {
    fn onboarding(self) -> Result<(), Box<dyn Error>> {
        self.confirm_repository_path()?
            .confirm_found_repository_details()?
            .add_client_details()?;
        Ok(())
    }
}

impl HelpPrompt {
    pub fn new(timesheet: Rc<RefCell<Timesheet>>) -> Self {
        Self { timesheet }
    }

    pub fn confirm_repository_path(self) -> Result<Self, Box<dyn std::error::Error>> {
        println!(
            "This looks like the first time you're running timesheet-gen. \n\
        Initialise timesheet-gen for current repository?"
        );

        if Confirm::new().default(true).interact()? {
            self.timesheet.borrow_mut().set_repo_path(String::from("."));
        } else {
            println!("Please give a path to the repository you would like to use");

            let path = crate::file_reader::get_home_path()
                .to_str()
                .unwrap()
                .to_string();

            let input: String = Input::new().with_initial_text(path).interact_text()?;

            self.timesheet
                .borrow_mut()
                .set_repo_path(String::from(input));
        }

        Ok(self)
    }

    pub fn confirm_found_repository_details(self) -> Result<Self, Box<dyn std::error::Error>> {
        self.timesheet.borrow_mut().find_repository_details_from()?;

        println!(
            "These are the details associated with this repository: \n\
        Project: {:?} \n\
        Name: {:?} \n\
        Email: {:?}",
            &self.timesheet.borrow().namespace.clone().unwrap(),
            &self.timesheet.borrow().email.clone().unwrap(),
            &self.timesheet.borrow().name.clone().unwrap(),
        );

        Ok(self)
    }

    pub fn add_client_details(self) -> Result<Self, std::io::Error> {
        println!("Would you like to add a client to this repo?");

        if Confirm::new().default(true).interact()? {
            println!("Client name");
            let input: String = Input::new().interact_text()?;
            self.timesheet.borrow_mut().set_client_name(input);

            println!("Client contact person");
            let input: String = Input::new().interact_text()?;
            self.timesheet.borrow_mut().set_client_contact_person(input);

            println!("Client address");
            if let Some(input) = Editor::new().edit("Enter an address").unwrap() {
                self.timesheet.borrow_mut().set_client_contact_person(input);
            }
        }

        Ok(self)
    }
}
