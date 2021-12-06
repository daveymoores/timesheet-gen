use crate::client_repositories::ClientRepositories;
use crate::repository::Repository;
/// Help prompt handles all of the interactions with the user.
/// It writes to the std output, and returns input data or a boolean
use dialoguer::{Confirm, Editor, Input, Select};
use std::cell::RefCell;
use std::error::Error;
use std::rc::Rc;

//TODO - consider using termion https://docs.rs/termion/1.5.6/termion/
#[derive(Debug, Clone)]
pub struct HelpPrompt {
    repository: Rc<RefCell<Repository>>,
}

pub trait Onboarding {
    fn onboarding(&self, new_user: bool) -> Result<(), Box<dyn std::error::Error>>;
}

pub trait ExistingClientOnboarding {
    fn existing_client_onboarding(&self) -> Result<(), Box<dyn std::error::Error>>;
}

impl Onboarding for HelpPrompt {
    fn onboarding(&self, new_user: bool) -> Result<(), Box<dyn Error>> {
        self.confirm_repository_path(new_user)?
            .search_for_repository_details()?
            .add_client_details()?
            .prompt_for_manager_approval()?
            .show_details();
        Ok(())
    }
}

impl ExistingClientOnboarding for HelpPrompt {
    fn existing_client_onboarding(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.confirm_repository_path(false)?
            .search_for_repository_details()?
            .prompt_for_manager_approval()?
            .show_details();
        Ok(())
    }
}

impl HelpPrompt {
    pub fn new(repository: Rc<RefCell<Repository>>) -> Self {
        Self { repository }
    }

    pub fn repo_already_initialised() {
        println!(
            "timesheet-gen has already been initialised for this repository! \n\
    Try 'timesheet-gen make' to create your first timesheet \n\
    or 'timesheet-gen help' for more options."
        );
        std::process::exit(exitcode::OK);
    }

    pub fn show_write_new_config_success() {
        println!(
            "timesheet-gen initialised! \n\
    Try 'timesheet-gen make' to create your first timesheet \n\
    or 'timesheet-gen help' for more options."
        );
        std::process::exit(exitcode::OK);
    }

    pub fn show_write_new_repo_success() {
        println!(
            "timesheet-gen initialised! \n\
    Try 'timesheet-gen make' to create your first timesheet \n\
    or 'timesheet-gen help' for more options."
        );
        std::process::exit(exitcode::OK);
    }

    pub fn show_edited_config_success() {
        println!("timesheet-gen successfully edited!");
        std::process::exit(exitcode::OK);
    }

    pub fn prompt_for_update(
        &mut self,
        deserialized_config: &mut Vec<ClientRepositories>,
        options: Vec<Option<String>>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if options[1].is_some() {
            println!(
                "Updating project '{}' for client '{}'. What would you like to update?",
                &options[1].as_ref().unwrap(),
                &options[0].as_ref().unwrap()
            );

            let opt = vec!["Approver name", "Approver email"];
            let selection: usize = Select::new().items(&opt).interact()?;
            let value = opt[selection];

            match value {
                "Approver name" => {
                    println!("Approver name");
                    let input: String = Input::new().interact_text()?;
                    deserialized_config[0]
                        .update_approver_name(input, options[1].as_ref().unwrap());
                }
                "Approver email" => {
                    println!("Approver email");
                    let input: String = Input::new().interact_text()?;
                    deserialized_config[0]
                        .update_approver_email(input, options[1].as_ref().unwrap());
                }
                _ => {}
            };
        } else {
            println!(
                "Updating client '{}'. What would you like to update?",
                &options[0].as_ref().unwrap()
            );

            let opt = vec![
                "Client company name",
                "Client contact person",
                "Client address",
            ];
            let selection: usize = Select::new().items(&opt).interact()?;
            let value = opt[selection];

            match value {
                "Client company name" => {
                    println!("Client company name");
                    let input: String = Input::new().interact_text()?;
                    deserialized_config[0].update_client_name(input);
                }
                "Client contact person" => {
                    println!("Client contact person");
                    let input: String = Input::new().interact_text()?;
                    deserialized_config[0].update_client_contact_person(input);
                }
                "Client address" => {
                    println!("Client address");
                    let input: String = Input::new().interact_text()?;
                    deserialized_config[0].update_client_address(input);
                }
                _ => {}
            };
        }

        Ok(())
    }

    pub fn prompt_for_client_then_onboard(
        &mut self,
        deserialized_config: &mut Vec<ClientRepositories>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        println!("Initialising new repository.");

        let mut clients: Vec<String> = deserialized_config
            .iter()
            .map(|client| client.client.as_ref().unwrap().client_name.clone())
            .collect();

        // If the clients array is empty, lets just onboard
        if clients.len() == 0 {
            self.onboarding(false)?;
        }

        let no_client_value = "Create a new client".to_string();
        clients.push(no_client_value.clone());

        println!("Would you like to add it to any of these existing clients?");
        let selection: usize = Select::new().items(&clients).interact()?;
        let client_name = &clients[selection];

        // if this is a new client, onboard as normal
        if client_name == &no_client_value {
            self.onboarding(false)?;
        } else {
            // otherwise pre-populate the client details
            let client = deserialized_config
                .iter()
                .find(|client| &client.client.as_ref().unwrap().client_name.clone() == client_name)
                .unwrap();

            let unwrapped_client = client.client.as_ref().unwrap();

            self.repository
                .borrow_mut()
                .set_client_name(unwrapped_client.client_name.clone())
                .set_client_address(unwrapped_client.client_address.clone())
                .set_client_contact_person(unwrapped_client.client_contact_person.clone());

            self.existing_client_onboarding()?;
        }

        Ok(())
    }

    pub fn confirm_repository_path(
        &self,
        new_user: bool,
    ) -> Result<&Self, Box<dyn std::error::Error>> {
        let repo_path = self.repository.clone();
        let mut borrow = repo_path.borrow_mut();
        let path = borrow.repo_path.as_ref().unwrap().clone();

        if new_user {
            println!(
                "  __  _ _ ____ ___  _   ___  ___\n\
                   /_`) ))`) ))  )) ) ))  )) ) ))_\n\
                 (( ( ((_( ((  ((_( ((__((_( ((_(\n\
                 "
            );
        }

        if path == "." {
            println!("Initialise for current repository?");
        } else {
            println!("With the project at this path {}?", path);
        };

        if Confirm::new().default(true).interact()? {
            borrow.set_repo_path(String::from(path));
        } else {
            println!("Please give a path to the repository you would like to use");

            let path = crate::file_reader::get_home_path()
                .to_str()
                .unwrap()
                .to_string();

            let input: String = Input::new().with_initial_text(path).interact_text()?;

            borrow.set_repo_path(String::from(input));
        }

        Ok(self)
    }

    pub fn search_for_repository_details(&self) -> Result<&Self, Box<dyn std::error::Error>> {
        self.repository
            .borrow_mut()
            .find_repository_details_from()?;

        println!("Repository details found!");
        Ok(self)
    }

    pub fn add_client_details(&self) -> Result<&Self, std::io::Error> {
        println!("Client company name");
        let input: String = Input::new().interact_text()?;
        self.repository.borrow_mut().set_client_name(input);

        println!("Client contact person");
        let input: String = Input::new().interact_text()?;
        self.repository
            .borrow_mut()
            .set_client_contact_person(input);

        println!("Client address");
        if let Some(input) = Editor::new().edit("Enter an address").unwrap() {
            self.repository.borrow_mut().set_client_address(input);
        }

        Ok(self)
    }

    pub fn prompt_for_manager_approval(&self) -> Result<&Self, Box<dyn Error>> {
        println!("Does your timesheet need approval? (This will enable signing functionality, see https://timesheet-gen.io/docs/signing)");
        if Confirm::new().default(true).interact()? {
            self.repository.borrow_mut().set_requires_approval(true);

            println!("Approvers name");
            let input: String = Input::new().interact_text()?;
            self.repository.borrow_mut().set_approvers_name(input);

            println!("Approvers email");
            let input: String = Input::new().interact_text()?;
            self.repository.borrow_mut().set_approvers_email(input);
        } else {
            self.repository.borrow_mut().set_requires_approval(false);
        }

        Ok(self)
    }

    pub fn add_project_numbers(
        &self,
        client_repositories: Rc<RefCell<Vec<ClientRepositories>>>,
    ) -> Result<&Self, Box<dyn Error>> {
        let mut cr_borrow = client_repositories.borrow_mut();
        println!(
            "Finding project data for '{}'...",
            cr_borrow[0].client.as_ref().unwrap().client_name
        );

        for i in 0..cr_borrow[0].repositories.as_ref().unwrap().len() {
            println!(
                "Does '{}' require a project/PO number?",
                cr_borrow[0].repositories.as_ref().unwrap()[i]
                    .namespace
                    .as_ref()
                    .unwrap()
            );
            if Confirm::new().default(true).interact()? {
                println!("Project number");
                let input: String = Input::new().interact_text()?;
                cr_borrow[0]
                    .repositories
                    .as_mut()
                    .map(|repo| repo[i].set_project_number(input));
            }
        }

        Ok(self)
    }

    pub fn prompt_for_client_repo_removal(
        &self,
        deserialized_config: &mut Vec<ClientRepositories>,
        options: Vec<Option<String>>,
    ) -> Result<&Self, Box<dyn Error>> {
        if options[1].is_some() {
            println!(
                "Remove '{}' from client '{}'?",
                &options[1].as_ref().unwrap(),
                &options[0].as_ref().unwrap()
            );
            // remove the namespace from a client
            if Confirm::new().default(true).interact()? {
                for i in 0..deserialized_config.len() {
                    if deserialized_config[i]
                        .client
                        .as_ref()
                        .unwrap()
                        .client_name
                        .to_lowercase()
                        == options[0].as_ref().unwrap().to_lowercase()
                    {
                        let repo_len = deserialized_config[i].repositories.as_ref().unwrap().len();
                        deserialized_config[i]
                            .remove_repository_by_namespace(options[1].as_ref().unwrap());

                        if repo_len != deserialized_config[i].repositories.as_ref().unwrap().len() {
                            println!("Success! '{}' removed.", &options[1].as_ref().unwrap());
                        } else {
                            println!("Client or repository not found. Nothing removed.");
                        }
                    }
                }
            }
        } else {
            println!("Remove client '{}'?", &options[0].as_ref().unwrap());
            // client is required and will be set, so remove from deserialized config
            if Confirm::new().default(true).interact()? {
                let config_len = deserialized_config.len();
                deserialized_config.retain(|client_repo| {
                    &client_repo
                        .client
                        .as_ref()
                        .unwrap()
                        .client_name
                        .to_lowercase()
                        != &options[0].as_ref().unwrap().to_lowercase()
                });

                if config_len != deserialized_config.len() {
                    println!("Success! '{}' removed.", &options[0].as_ref().unwrap());
                } else {
                    println!("Client not found. Nothing removed.");
                }
            }
        }

        Ok(self)
    }

    pub fn show_details(&self) -> &Self {
        println!("These are the details associated with this repository:");

        if let Some(namespace) = &self.repository.borrow().namespace.clone() {
            println!("Project: {}", namespace);
        }
        if let Some(email) = &self.repository.borrow().email.clone() {
            println!("Email: {}", email);
        }
        if let Some(name) = &self.repository.borrow().name.clone() {
            println!("Name: {}", name);
        }
        if let Some(client_name) = &self.repository.borrow().client_name.clone() {
            println!("Client name: {}", client_name);
        }
        if let Some(client_contact_person) = &self.repository.borrow().client_contact_person.clone()
        {
            println!("Client contact person: {}", client_contact_person);
        }
        if let Some(client_address) = &self.repository.borrow().client_address.clone() {
            println!("Client address: {}", client_address);
        }
        if let Some(approvers_name) = &self.repository.borrow().approvers_name.clone() {
            println!("Approvers name: {}", approvers_name);
        }
        if let Some(approvers_email) = &self.repository.borrow().approvers_email.clone() {
            println!("Approvers email: {}", approvers_email);
        }

        self
    }
}
