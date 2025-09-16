use crate::core::{
    clients::client::{Client, ClientType, Meta},
    storage::{custom_clients::CUSTOM_CLIENT_MANAGER, data::DATA},
    utils::globals::JDK_FOLDER,
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum Version {
    V1_16_5,
    V1_12_2,
}

impl Version {
    pub fn from_string(version: &str) -> Self {
        match version {
            "1.16.5" => Version::V1_16_5,
            "1.12.2" => Version::V1_12_2,
            _ => Version::V1_16_5,
        }
    }
}

impl std::fmt::Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Version::V1_16_5 => write!(f, "1.16.5"),
            Version::V1_12_2 => write!(f, "1.12.2"),
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
            requirement_mods: None,
            client_type: ClientType::Default,
            created_at: chrono::Utc::now(),
            meta: Meta {
                is_new: self.version == Version::V1_16_5,
                asset_index: String::new(),
                installed: self.is_installed,
                is_custom: true,
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

    pub fn get_running_custom_clients() -> Vec<CustomClient> {
        use std::process::Command;

        #[cfg(target_os = "windows")]
        use std::os::windows::process::CommandExt;

        let jps_path = DATA.root_dir.join(JDK_FOLDER).join("bin").join("jps.exe");
        let mut command = Command::new(jps_path);

        #[cfg(windows)]
        command.creation_flags(0x08000000);

        let output = match command.arg("-m").output() {
            Ok(output) => output,
            Err(_) => {
                return Vec::new();
            }
        };

        let binding = String::from_utf8_lossy(&output.stdout);
        let outputs: Vec<&str> = binding.lines().collect();

        let custom_clients = CUSTOM_CLIENT_MANAGER
            .lock()
            .ok()
            .map(|manager| manager.clients.clone())
            .unwrap_or_default();

        custom_clients
            .into_iter()
            .filter(|client| outputs.iter().any(|line| line.contains(&client.filename)))
            .collect()
    }

    pub fn stop(&self) -> Result<(), String> {
        use crate::core::storage::data::DATA;
        use std::process::Command;

        #[cfg(target_os = "windows")]
        use std::os::windows::process::CommandExt;

        let jps_path = DATA.root_dir.join(JDK_FOLDER).join("bin").join("jps.exe");
        let mut command = Command::new(jps_path);

        #[cfg(windows)]
        command.creation_flags(0x08000000);

        let output = command
            .arg("-m")
            .output()
            .map_err(|e| format!("Failed to execute jps command: {e}"))?;

        let binding = String::from_utf8_lossy(&output.stdout);
        let outputs: Vec<&str> = binding.lines().collect();

        let mut process_found = false;
        for line in &outputs {
            if line.contains(&self.filename) {
                process_found = true;
                let pid = line.split_whitespace().next().unwrap_or_default();

                let mut kill_command = Command::new("taskkill");

                #[cfg(windows)]
                kill_command.creation_flags(0x08000000);

                kill_command
                    .arg("/PID")
                    .arg(pid)
                    .arg("/F")
                    .output()
                    .map_err(|e| format!("Failed to kill process: {e}"))?;
            }
        }

        if !process_found {
            crate::log_info!("No process found for custom client: {}", self.name);
        }

        Ok(())
    }
}
