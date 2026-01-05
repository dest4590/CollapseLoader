use crate::core::{
    clients::client::{Client, ClientType, Meta},
    storage::custom_clients::CUSTOM_CLIENT_MANAGER,
    utils::process,
};
use crate::{log_debug, log_error};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct CustomClient {
    pub id: u32,
    pub name: String,
    pub version: String,
    pub filename: String,
    pub file_path: PathBuf,
    pub main_class: String,
    pub created_at: String,
    pub is_installed: bool,
    pub launches: u32,
    pub insecure: bool,
}

impl CustomClient {
    pub fn new(
        id: u32,
        name: String,
        version: String,
        filename: String,
        file_path: PathBuf,
        main_class: String,
    ) -> Self {
        Self {
            id,
            name,
            version,
            filename,
            file_path,
            main_class,
            created_at: chrono::Utc::now().to_rfc3339(),
            is_installed: false,
            launches: 0,
            insecure: false,
        }
    }

    pub fn to_client(&self) -> Client {
        Client {
            id: self.id,
            name: self.name.clone(),
            version: self.version.clone(),
            filename: self.filename.clone(),
            md5_hash: String::new(),
            main_class: self.main_class.clone(),
            show: true,
            working: true,
            insecure: self.insecure,
            launches: self.launches,
            downloads: 0,
            rating_avg: None,
            rating_count: None,
            size: 0,
            requirement_mods: None,
            client_type: ClientType::Default,
            created_at: chrono::Utc::now(),
            meta: Meta {
                is_new: false,
                is_fabric: false,
                asset_index: String::new(),
                installed: self.is_installed,
                is_custom: true,
                size: 0,
            },
        }
    }

    pub fn validate_file(&self) -> Result<(), String> {
        log_debug!(
            "Validating file for custom client '{}' at path: {}",
            self.name,
            self.file_path.display()
        );
        if !self.file_path.exists() {
            let err_msg = format!("File {} does not exist", self.file_path.display());
            log_error!("Validation failed for '{}': {}", self.name, err_msg);
            return Err(err_msg);
        }

        if !self.file_path.is_file() {
            let err_msg = format!("Path {} is not a file", self.file_path.display());
            log_error!("Validation failed for '{}': {}", self.name, err_msg);
            return Err(err_msg);
        }

        let extension = self
            .file_path
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("");

        if extension != "jar" {
            let err_msg = "File must be a .jar file".to_string();
            log_error!("Validation failed for '{}': {}", self.name, err_msg);
            return Err(err_msg);
        }

        log_debug!(
            "File validation successful for custom client '{}'",
            self.name
        );
        Ok(())
    }

    pub fn get_running_custom_clients() -> Vec<Self> {
        let custom_clients = CUSTOM_CLIENT_MANAGER
            .lock()
            .ok()
            .map(|manager| manager.clients.clone())
            .unwrap_or_default();

        process::filter_running(custom_clients, |client| &client.filename)
    }

    pub fn stop(&self) -> Result<(), String> {
        process::stop_process_by_filename(&self.filename, &self.name)
    }
}
