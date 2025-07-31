use crate::core::clients::client::Client;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum Version {
    V1_16_5,
    V1_12_2,
    Custom(String),
}

impl Version {
    pub fn to_string(&self) -> String {
        match self {
            Version::V1_16_5 => "1.16.5".to_string(),
            Version::V1_12_2 => "1.12.2".to_string(),
            Version::Custom(version) => version.clone(),
        }
    }

    pub fn from_string(version: &str) -> Self {
        match version {
            "1.16.5" => Version::V1_16_5,
            "1.12.2" => Version::V1_12_2,
            _ => Version::Custom(version.to_string()),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct CustomClient {
    pub id: u32,
    pub name: String,
    pub version: Version,
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
        version: Version,
        filename: String,
        file_path: PathBuf,
        main_class: String,
    ) -> Self {
        CustomClient {
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
            version: self.version.to_string(),
            filename: self.filename.clone(),
            md5_hash: String::new(),
            main_class: self.main_class.clone(),
            show: true,
            working: true,
            insecure: self.insecure,
            launches: self.launches,
            downloads: 0,
            size: 0,
            meta: crate::core::clients::client::Meta {
                is_new: false,
                asset_index: String::new(),
                installed: self.is_installed,
                size: 0,
            },
        }
    }

    pub fn validate_file(&self) -> Result<(), String> {
        if !self.file_path.exists() {
            return Err(format!("File {} does not exist", self.file_path.display()));
        }

        if !self.file_path.is_file() {
            return Err(format!("Path {} is not a file", self.file_path.display()));
        }

        let extension = self
            .file_path
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("");

        if extension != "jar" {
            return Err("File must be a .jar file".to_string());
        }

        Ok(())
    }
}
