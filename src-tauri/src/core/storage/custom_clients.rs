use std::{path::PathBuf, sync::Mutex};

use crate::core::clients::custom_clients::CustomClient;
use crate::core::storage::data::DATA;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};

use super::common::JsonStorage;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CustomClientManager {
    pub clients: Vec<CustomClient>,
    pub custom_clients_path: PathBuf,
}

impl CustomClientManager {
    #[allow(dead_code)]
    // TODO: implement all it
    pub fn add_client(&mut self, custom_client: CustomClient) {
        if !self.clients.contains(&custom_client) {
            self.clients.push(custom_client);
        }
    }
}

impl JsonStorage for CustomClientManager {
    fn file_path(&self) -> &PathBuf {
        &self.custom_clients_path
    }

    fn resource_name() -> &'static str {
        "custom_clients"
    }

    fn create_default() -> Self {
        Self::default()
    }
}

impl Default for CustomClientManager {
    fn default() -> Self {
        Self {
            clients: Vec::new(),
            custom_clients_path: DATA.get_local("custom_clients.json"),
        }
    }
}

lazy_static! {
    pub static ref CUSTOM_CLIENT_MANAGER: Mutex<CustomClientManager> = Mutex::new(
        CustomClientManager::load_from_disk(DATA.get_local("custom_clients.json"))
    );
}
