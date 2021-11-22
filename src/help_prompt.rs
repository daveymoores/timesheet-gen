use crate::config::TimesheetConfig;
use crate::timesheet::Timesheet;
/// Help prompt handles all of the interactions with the user.
/// It writes to the std output, and returns input data or a boolean
use dialoguer::{Confirm, Editor, Input, Select};
use std::cell::RefCell;
use std::error::Error;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct HelpPrompt {
    timesheet: Rc<RefCell<Timesheet>>,
}

pub trait Onboarding {
    fn onboarding(&self) -> Result<(), Box<dyn std::error::Error>>;
}

impl Onboarding for HelpPrompt {
    fn onboarding(&self) -> Result<(), Box<dyn Error>> {
        self.confirm_repository_path()?
            .search_for_repository_details()?
            .add_client_details()?
            .prompt_for_manager_approval()?
            .show_details();
        Ok(())
    }
}

impl HelpPrompt {
    pub fn new(timesheet: Rc<RefCell<Timesheet>>) -> Self {
        Self { timesheet }
    }

    pub fn repo_already_initialised() {
        println!(
            "timesheet-gen has already been initialised for this repository! \n\
    Try 'timesheet-gen make' to create your first timesheet \n\
    or 'timesheet-gen help' for more options."
        );
    }

    pub fn prompt_for_client(
        &self,
        deserialized_config: Vec<TimesheetConfig>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        println!(
            "Looks like this repository hasn't been initialised yet.\n\
        Would you like to add it to any of these existing clients?"
        );
        let no_client_value = "Create a new client".to_string();

        let mut clients: Vec<String> = deserialized_config
            .iter()
            .map(|client| client.client.clone())
            .collect();
        clients.push(no_client_value);

        let selection: usize = Select::new().items(&clients).interact()?;
        let client_name = &clients[selection];

        self.timesheet
            .borrow_mut()
            .set_client_name(client_name.clone());

        Ok(())
    }

    pub fn confirm_repository_path(&self) -> Result<&Self, Box<dyn std::error::Error>> {
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

    pub fn search_for_repository_details(&self) -> Result<&Self, Box<dyn std::error::Error>> {
        self.timesheet
            .borrow_mut()
            .find_repository_details_from()?
            .exec_generate_timesheets_from_git_history();

        println!("Repository details found!");
        Ok(self)
    }

    pub fn add_client_details(&self) -> Result<&Self, std::io::Error> {
        println!("Client company name");
        let input: String = Input::new().interact_text()?;
        self.timesheet.borrow_mut().set_client_name(input);

        println!("Client contact person");
        let input: String = Input::new().interact_text()?;
        self.timesheet.borrow_mut().set_client_contact_person(input);

        println!("Client address");
        if let Some(input) = Editor::new().edit("Enter an address").unwrap() {
            self.timesheet.borrow_mut().set_client_address(input);
        }

        Ok(self)
    }

    pub fn prompt_for_manager_approval(&self) -> Result<&Self, Box<dyn Error>> {
        println!("Does your timesheet need approval? (This will enable signing functionality, see https://timesheet-gen.io/docs/signing)");
        if Confirm::new().default(true).interact()? {
            self.timesheet.borrow_mut().set_requires_approval(true);

            println!("Approvers name");
            let input: String = Input::new().interact_text()?;
            self.timesheet.borrow_mut().set_approvers_name(input);

            println!("Approvers email");
            let input: String = Input::new().interact_text()?;
            self.timesheet.borrow_mut().set_approvers_email(input);
        } else {
            self.timesheet.borrow_mut().set_requires_approval(false);
        }

        Ok(self)
    }

    pub fn add_project_number(&self) -> Result<&Self, Box<dyn Error>> {
        println!("Does this timesheet require a project/PO number?");
        if Confirm::new().default(true).interact()? {
            println!("Project number");
            let input: String = Input::new().interact_text()?;
            self.timesheet.borrow_mut().set_project_number(input);
        }

        Ok(self)
    }

    pub fn show_details(&self) -> &Self {
        println!("These are the details associated with this repository:");

        if let Some(namespace) = &self.timesheet.borrow().namespace.clone() {
            println!("Project: {}", namespace);
        }
        if let Some(email) = &self.timesheet.borrow().email.clone() {
            println!("Email: {}", email);
        }
        if let Some(name) = &self.timesheet.borrow().name.clone() {
            println!("Name: {}", name);
        }
        if let Some(client_name) = &self.timesheet.borrow().client_name.clone() {
            println!("Client name: {}", client_name);
        }
        if let Some(client_contact_person) = &self.timesheet.borrow().client_contact_person.clone()
        {
            println!("Client contact person: {}", client_contact_person);
        }
        if let Some(client_address) = &self.timesheet.borrow().client_address.clone() {
            println!("Client address: {}", client_address);
        }
        if let Some(approvers_name) = &self.timesheet.borrow().approvers_name.clone() {
            println!("Approvers name: {}", approvers_name);
        }
        if let Some(approvers_email) = &self.timesheet.borrow().approvers_email.clone() {
            println!("Approvers email: {}", approvers_email);
        }

        self
    }
}
