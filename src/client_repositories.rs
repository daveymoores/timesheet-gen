use crate::config::New;
use crate::repository::Repository;
use serde::{Deserialize, Serialize};

/// Repositories are modified at a Repository level and a client level.
/// ClientRepositories  holds the client and the repositories when they are found in the buffer
/// Storing the data here allows the repository  being currently operated on to be cross referenced
/// against all the repos under the same client, and hence generate the correct working hours.

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Client {
    pub client_name: String,
    pub client_address: String,
    pub client_contact_person: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ClientRepositories {
    pub client: Option<Client>,
    pub repositories: Option<Vec<Repository>>,
}

impl Default for ClientRepositories {
    fn default() -> Self {
        Self {
            client: None,
            repositories: None,
        }
    }
}

impl New for ClientRepositories {
    fn new() -> Self {
        Self {
            ..Default::default()
        }
    }
}

impl ClientRepositories {
    pub fn set_values_from_buffer(
        &mut self,
        client_repositories: &ClientRepositories,
    ) -> &mut ClientRepositories {
        *self = client_repositories.clone();
        self
    }
}
