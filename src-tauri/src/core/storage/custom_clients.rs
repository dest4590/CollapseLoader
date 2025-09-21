use std::{fs, path::PathBuf, sync::Mutex};

use crate::core::clients::custom_clients::CustomClient;
use crate::core::clients::custom_clients::Version;
use crate::core::storage::data::DATA;
use crate::core::storage::settings::SETTINGS;
use crate::log_warn;
use serde::{Deserialize, Serialize};
use std::sync::LazyLock;

use super::common::JsonStorage;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CustomClientManager {
    pub clients: Vec<CustomClient>,
    pub custom_clients_path: PathBuf,
    pub next_id: u32,
}

impl CustomClientManager {
    pub fn add_client(&mut self, mut custom_client: CustomClient) -> Result<(), String> {
        if !custom_client.file_path.exists() {
            return Err(format!(
                "File '{}' does not exist. Please select a valid .jar file.",
                custom_client.file_path.display()
            ));
        }

        custom_client.validate_file()?;

        custom_client.id = self.next_id;
        self.next_id += 1;

        if self.clients.iter().any(|c| c.name == custom_client.name) {
            return Err(format!(
                "A client with name '{}' already exists",
                custom_client.name
            ));
        }

        let custom_clients_dir = DATA.get_local("custom_clients");
        if !custom_clients_dir.exists() {
            fs::create_dir_all(&custom_clients_dir)
                .map_err(|e| format!("Failed to create custom clients directory: {e}"))?;
        }

        let client_dir = custom_clients_dir.join(&custom_client.name);
        if !client_dir.exists() {
            fs::create_dir_all(&client_dir)
                .map_err(|e| format!("Failed to create client directory: {e}"))?;
        }

        let target_path = client_dir.join(&custom_client.filename);
        fs::copy(&custom_client.file_path, &target_path)
            .map_err(|e| format!("Failed to copy file: {e}"))?;

        custom_client.file_path = target_path;
        custom_client.is_installed = true;

        if SETTINGS
            .lock()
            .map(|s| s.sync_client_settings.value)
            .unwrap_or(false)
        {
            if let Err(e) = DATA.ensure_client_synced(&custom_client.name) {
                log_warn!(
                    "Failed to ensure client sync for custom client {}: {}",
                    custom_client.name,
                    e
                );
            }
        }

        self.clients.push(custom_client);
        self.save_to_disk();
        Ok(())
    }

    pub fn remove_client(&mut self, id: u32) -> Result<(), String> {
        if let Some(index) = self.clients.iter().position(|c| c.id == id) {
            let client = &self.clients[index];

            if client.file_path.exists() {
                fs::remove_file(&client.file_path)
                    .map_err(|e| format!("Failed to remove file: {e}"))?;
            }

            self.clients.remove(index);
            self.save_to_disk();
            Ok(())
        } else {
            Err("Client not found".to_string())
        }
    }

    pub fn get_client(&self, id: u32) -> Option<&CustomClient> {
        self.clients.iter().find(|c| c.id == id)
    }

    pub fn get_client_mut(&mut self, id: u32) -> Option<&mut CustomClient> {
        self.clients.iter_mut().find(|c| c.id == id)
    }

    pub fn update_client(&mut self, id: u32, updates: CustomClientUpdate) -> Result<(), String> {
        if let Some(ref name) = updates.name {
            if self.clients.iter().any(|c| c.id != id && c.name == *name) {
                return Err(format!("A client with name '{name}' already exists"));
            }
        }

        if let Some(client) = self.get_client_mut(id) {
            if let Some(name) = updates.name {
                client.name = name;
            }

            if let Some(version) = updates.version {
                client.version = version;
            }

            if let Some(main_class) = updates.main_class {
                client.main_class = main_class;
            }

            self.save_to_disk();
            Ok(())
        } else {
            Err("Client not found".to_string())
        }
    }
}

#[derive(Debug)]
pub struct CustomClientUpdate {
    pub name: Option<String>,
    pub version: Option<Version>,
    pub main_class: Option<String>,
}

impl JsonStorage for CustomClientManager {
    fn file_path(&self) -> &PathBuf {
        &self.custom_clients_path
    }

    fn resource_name() -> &'static str {
        "custom_clients"
    }

    fn create_default() -> Self {
        Self {
            clients: Vec::new(),
            custom_clients_path: DATA.get_local("custom_clients.json"),
            next_id: 1,
        }
    }
}

impl Default for CustomClientManager {
    fn default() -> Self {
        Self {
            clients: Vec::new(),
            custom_clients_path: DATA.get_local("custom_clients.json"),
            next_id: 1,
        }
    }
}

pub static CUSTOM_CLIENT_MANAGER: LazyLock<Mutex<CustomClientManager>> = LazyLock::new(|| {
    Mutex::new(CustomClientManager::load_from_disk(
        DATA.get_local("custom_clients.json"),
    ))
});
