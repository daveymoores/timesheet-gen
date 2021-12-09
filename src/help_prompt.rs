use crate::client_repositories::ClientRepositories;
use crate::repository::Repository;
use ansi_term::Style;
use ascii_table::AsciiTable;
/// Help prompt handles all of the interactions with the user.
/// It writes to the std output, and returns input data or a boolean
use dialoguer::{Confirm, Editor, Input, Select};
use nanoid::nanoid;
use std::cell::{RefCell, RefMut};
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
            .show_details();
        Ok(())
    }
}

impl ExistingClientOnboarding for HelpPrompt {
    fn existing_client_onboarding(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.confirm_repository_path(false)?
            .search_for_repository_details()?
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
            "timesheet-gen has already been initialised for this repository \u{1F916} \n\
    Try 'timesheet-gen make' to create your first timesheet \n\
    or 'timesheet-gen help' for more options."
        );
        std::process::exit(exitcode::OK);
    }

    pub fn show_write_new_config_success() {
        println!(
            "\n{}",
            Style::new()
                .bold()
                .paint("timesheet-gen initialised \u{1F389} \n")
        );
        println!(
            "Try 'timesheet-gen make' to create your first timesheet \n\
            or 'timesheet-gen help' for more options."
        );
        std::process::exit(exitcode::OK);
    }

    pub fn show_write_new_repo_success() {
        println!(
            "\n{}",
            Style::new()
                .bold()
                .paint("New repository added \u{1F389} \n")
        );
        println!(
            "Try 'timesheet-gen make' to create your first timesheet \n\
            or 'timesheet-gen help' for more options."
        );
        std::process::exit(exitcode::OK);
    }

    pub fn show_edited_config_success() {
        println!("timesheet-gen successfully edited \u{1F389}");
        crate::utils::exit_process();
    }

    pub fn show_updated_config_success() {
        println!("timesheet-gen successfully updated \u{1F389}");
        crate::utils::exit_process();
    }

    pub fn prompt_for_update(
        &mut self,
        client_repositories: Rc<RefCell<Vec<ClientRepositories>>>,
        options: Vec<Option<String>>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut client_repo_borrow = client_repositories.borrow_mut();

        if options[1].is_some() {
            Self::print_question(&*format!(
                "Updating project '{}' for client '{}'. What would you like to update?",
                &options[1].as_ref().unwrap(),
                &options[0].as_ref().unwrap()
            ));

            let opt = vec!["Namespace", "Repository path"];
            let selection: usize = Select::new().items(&opt).interact()?;
            let value = opt[selection];

            match value {
                "Namespace" => {
                    let input: String = Input::new().interact_text()?;
                    client_repo_borrow[0]
                        .repositories
                        .as_mut()
                        .map(|repo| repo[0].set_namespace(input));
                }
                "Repository path" => {
                    //TODO - need to check this path exists before allowing update
                    let input: String = Input::new().interact_text()?;
                    client_repo_borrow[0]
                        .repositories
                        .as_mut()
                        .map(|repo| repo[0].set_repo_path(input));
                }
                _ => {}
            };
        } else {
            Self::print_question(&*format!(
                "Updating client '{}'. What would you like to update?",
                &options[0].as_ref().unwrap()
            ));

            let opt = vec![
                "Approver name",
                "Approver email",
                "Client company name",
                "Client contact person",
                "Client address",
            ];
            let selection: usize = Select::new().items(&opt).interact()?;
            let value = opt[selection];

            match value {
                "Approver name" => {
                    println!("Approver's name");
                    let input: String = Input::new().interact_text()?;
                    client_repo_borrow[0].set_approvers_name(input);
                }
                "Approver email" => {
                    println!("Approver's email");
                    let input: String = Input::new().interact_text()?;
                    client_repo_borrow[0].set_approvers_email(input);
                }
                "Client company name" => {
                    println!("Client company name");
                    let input: String = Input::new().interact_text()?;
                    client_repo_borrow[0].update_client_name(input);
                }
                "Client contact person" => {
                    println!("Client contact person");
                    let input: String = Input::new().interact_text()?;
                    client_repo_borrow[0].update_client_contact_person(input);
                }
                "Client address" => {
                    println!("Client address");
                    let input: String = Input::new().interact_text()?;
                    client_repo_borrow[0].update_client_address(input);
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
        Self::print_question("Initialising new repository.");

        let mut clients: Vec<String> = deserialized_config
            .iter()
            .map(|client| client.get_client_name().clone())
            .collect();

        // If the clients array is empty, lets just onboard
        if clients.len() == 0 {
            self.onboarding(false)?;
        }

        let no_client_value = "Create a new client".to_string();
        clients.push(no_client_value.clone());

        Self::print_question("Would you like to add it to any of these existing clients?");
        let selection: usize = Select::new().items(&clients).interact()?;
        let client_name = &clients[selection];

        // if this is a new client, onboard as normal
        if client_name == &no_client_value {
            self.onboarding(false)?;
        } else {
            // otherwise pre-populate the client details
            let client = deserialized_config
                .iter()
                .find(|client| &client.get_client_name() == client_name)
                .unwrap();

            let unwrapped_client = client.client.as_ref().unwrap();

            self.repository
                .borrow_mut()
                .set_client_id(unwrapped_client.id.clone())
                .set_client_name(unwrapped_client.client_name.clone())
                .set_client_address(unwrapped_client.client_address.clone())
                .set_client_contact_person(unwrapped_client.client_contact_person.clone());

            self.existing_client_onboarding()?;
        }

        Ok(())
    }

    fn print_question(text: &str) {
        println!("\n{}", Style::new().bold().paint(text));
    }

    pub fn confirm_repository_path(
        &self,
        new_user: bool,
    ) -> Result<&Self, Box<dyn std::error::Error>> {
        let repo_path = self.repository.clone();
        let mut borrow = repo_path.borrow_mut();
        let path = match borrow.repo_path.as_ref() {
            Some(path) => path.clone(),
            None => {
                println!(
                    "A configuration file hasn't been found, which suggests \n\
                    timesheet-gen hasn't been initialised yet, or that all \n\
                    clients and repositories were removed. \n\
                    \n\
                    Run timesheet-gen init to add your first client."
                );
                std::process::exit(exitcode::OK);
            }
        };

        if new_user {
            let ascii_table = AsciiTable::default();
            let logo = [[Style::new().bold().paint("A U T O L O G")]];
            ascii_table.print(logo);
        }

        let current_repo_path = crate::utils::get_canonical_path(".");
        if path == current_repo_path {
            Self::print_question("Initialise for current repository?");
        } else {
            Self::print_question(&*format!("With the project at this path {}?", path));
        };

        if Confirm::new().default(true).interact()? {
            borrow.set_repo_path(String::from(path));
        } else {
            Self::print_question("Give a path to the repository you would like to use");

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

        self.repository.borrow_mut().set_user_id(nanoid!());
        self.repository.borrow_mut().set_repository_id(nanoid!());

        println!("{}", Self::dim_text("Repository details found \u{1F916}"));
        Ok(self)
    }

    pub fn add_client_details(&self) -> Result<&Self, std::io::Error> {
        Self::print_question("Client company name");
        let input: String = Input::new().interact_text()?;
        self.repository.borrow_mut().set_client_name(input);

        Self::print_question("Client contact person");
        let input: String = Input::new().interact_text()?;
        self.repository
            .borrow_mut()
            .set_client_contact_person(input);

        Self::print_question("Would you like to add a Client address?");
        if Confirm::new().default(true).interact()? {
            if let Some(input) = Editor::new().edit("Enter an address").unwrap() {
                self.repository.borrow_mut().set_client_address(input);
            }
        }
        self.repository.borrow_mut().set_client_id(nanoid!());

        Ok(self)
    }

    pub fn prompt_for_manager_approval(
        &self,
        client_repositories: Rc<RefCell<Vec<ClientRepositories>>>,
    ) -> Result<&Self, Box<dyn Error>> {
        let mut client_repos_borrow = client_repositories.borrow_mut();

        Self::print_question("Does your timesheet need approval? (This will enable signing functionality, see https://timesheet-gen.io/docs/signing)");
        if Confirm::new().default(true).interact()? {
            client_repos_borrow[0].set_requires_approval(true);

            Self::print_question("Approvers name");
            let input: String = Input::new().interact_text()?;
            client_repos_borrow[0].set_approvers_name(input);

            Self::print_question("Approvers email");
            let input: String = Input::new().interact_text()?;
            client_repos_borrow[0].set_approvers_email(input);
        } else {
            client_repos_borrow[0].set_requires_approval(false);
        }

        Ok(self)
    }

    pub fn add_project_numbers(
        &self,
        client_repositories: Rc<RefCell<Vec<ClientRepositories>>>,
    ) -> Result<&Self, Box<dyn Error>> {
        let mut cr_borrow = client_repositories.borrow_mut();
        Self::print_question(&*format!(
            "Finding project data for '{}'...",
            cr_borrow[0].get_client_name()
        ));

        for i in 0..cr_borrow[0].repositories.as_ref().unwrap().len() {
            Self::print_question(&*format!(
                "Does '{}' require a project/PO number?",
                cr_borrow[0].repositories.as_ref().unwrap()[i]
                    .namespace
                    .as_ref()
                    .unwrap()
            ));
            if Confirm::new().default(true).interact()? {
                Self::print_question("Project number");
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
        mut client_repositories: RefMut<Vec<ClientRepositories>>,
        options: Vec<Option<String>>,
    ) -> Result<&Self, Box<dyn Error>> {
        if options[1].is_some() {
            Self::print_question(&*format!(
                "Remove '{}' from client '{}'?",
                &options[1].as_ref().unwrap(),
                &options[0].as_ref().unwrap()
            ));
            // remove the namespace from a client
            if crate::utils::confirm()? {
                for i in 0..client_repositories.len() {
                    if client_repositories[i]
                        .client
                        .as_ref()
                        .unwrap()
                        .client_name
                        .to_lowercase()
                        == options[0].as_ref().unwrap().to_lowercase()
                    {
                        let repo_len = client_repositories[i].repositories.as_ref().unwrap().len();
                        client_repositories[i]
                            .remove_repository_by_namespace(options[1].as_ref().unwrap());

                        if repo_len != client_repositories[i].repositories.as_ref().unwrap().len() {
                            Self::print_question(&*format!(
                                "Success! '{}' removed.",
                                &options[1].as_ref().unwrap()
                            ));
                            crate::utils::exit_process();
                        } else {
                            Self::print_question(
                                "Client or repository not found. Nothing removed.",
                            );
                        }
                    }
                }
            }
        } else {
            Self::print_question(&*format!(
                "Remove client '{}'?",
                &options[0].as_ref().unwrap()
            ));
            // client is required and will be set, so remove from deserialized config
            if crate::utils::confirm()? {
                let config_len = client_repositories.len();
                client_repositories.retain(|client_repo| {
                    &client_repo
                        .client
                        .as_ref()
                        .unwrap()
                        .client_name
                        .to_lowercase()
                        != &options[0].as_ref().unwrap().to_lowercase()
                });

                if config_len != client_repositories.len() {
                    Self::print_question(&*format!(
                        "Success! '{}' removed.",
                        &options[0].as_ref().unwrap()
                    ));
                } else {
                    Self::print_question("Client not found. Nothing removed.");
                }
            }
        }

        Ok(self)
    }

    fn dim_text(text: &str) -> String {
        format!("{}", Style::new().dimmed().paint(text))
    }

    pub fn show_details(&self) -> &Self {
        Self::print_question("These are the details associated with this repository:");
        let ascii_table = AsciiTable::default();
        let mut data = vec![];

        if let Some(namespace) = self.repository.borrow().namespace.as_ref() {
            let row = vec![Self::dim_text("Namespace:"), namespace.clone()];
            data.append(&mut vec![row]);
        }
        if let Some(repo_path) = self.repository.borrow().repo_path.as_ref() {
            let row = vec![Self::dim_text("Repository path:"), repo_path.clone()];
            data.append(&mut vec![row]);
        }
        if let Some(git_path) = self.repository.borrow().git_path.as_ref() {
            let row = vec![Self::dim_text("Git path:"), git_path.clone()];
            data.append(&mut vec![row]);
        }
        if let Some(email) = self.repository.borrow().email.as_ref() {
            let row = vec![Self::dim_text("Email:"), email.clone()];
            data.append(&mut vec![row]);
        }
        if let Some(name) = self.repository.borrow().name.as_ref().clone() {
            let row = vec![Self::dim_text("Name:"), name.clone()];
            data.append(&mut vec![row]);
        }
        if let Some(client_name) = self.repository.borrow().client_name.as_ref().clone() {
            let row = vec![Self::dim_text("Client company name:"), client_name.clone()];
            data.append(&mut vec![row]);
        }
        if let Some(client_contact_person) = self
            .repository
            .borrow()
            .client_contact_person
            .as_ref()
            .clone()
        {
            let row = vec![
                Self::dim_text("Client contact person:"),
                client_contact_person.clone(),
            ];
            data.append(&mut vec![row]);
        }
        if let Some(client_address) = self.repository.borrow().client_address.as_ref().clone() {
            let row = vec![
                Self::dim_text("Client address:"),
                client_address.clone().replace("\n", " "),
            ];
            data.append(&mut vec![row]);
        }

        ascii_table.print(data);

        self
    }
}
