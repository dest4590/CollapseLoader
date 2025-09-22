use crate::core::{
    clients::client::{Client, ClientType, Meta},
    storage::{custom_clients::CUSTOM_CLIENT_MANAGER, data::DATA},
    utils::globals::{FILE_EXTENSION, JDK_FOLDER},
};
use crate::{log_debug, log_error, log_info, log_warn};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum Version {
    V1_16_5,
    V1_12_2,
}

impl Version {
    pub fn from_string(version: &str) -> Self {
        match version {
            "1.16.5" => Self::V1_16_5,
            "1.12.2" => Self::V1_12_2,
            _ => {
                log_warn!(
                    "Unsupported version string '{}', defaulting to 1.16.5",
                    version
                );
                Self::V1_16_5
            }
        }
    }
}

impl std::fmt::Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::V1_16_5 => write!(f, "1.16.5"),
            Self::V1_12_2 => write!(f, "1.12.2"),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
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
        use std::process::Command;

        #[cfg(target_os = "windows")]
        use std::os::windows::process::CommandExt;

        let jps_path = DATA
            .root_dir
            .join(JDK_FOLDER)
            .join("bin")
            .join("jps".to_owned() + FILE_EXTENSION);
        let mut command = Command::new(jps_path);

        #[cfg(windows)]
        command.creation_flags(0x0800_0000);

        let output = match command.arg("-m").output() {
            Ok(output) => output,
            Err(e) => {
                log_error!("Failed to execute jps command: {}", e);
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

        let running_clients: Vec<Self> = custom_clients
            .into_iter()
            .filter(|client| outputs.iter().any(|line| line.contains(&client.filename)))
            .collect();

        running_clients
    }

    pub fn stop(&self) -> Result<(), String> {
        use crate::core::storage::data::DATA;
        use std::process::Command;

        #[cfg(target_os = "windows")]
        use std::os::windows::process::CommandExt;

        log_info!("Attempting to stop custom client '{}'", self.name);
        let jps_path = DATA
            .root_dir
            .join(JDK_FOLDER)
            .join("bin")
            .join("jps".to_owned() + FILE_EXTENSION);
        let mut command = Command::new(jps_path);

        #[cfg(windows)]
        command.creation_flags(0x0800_0000);

        let output = command.arg("-m").output().map_err(|e| {
            log_error!("Failed to execute jps command for stopping: {}", e);
            format!("Failed to execute jps command: {e}")
        })?;

        let binding = String::from_utf8_lossy(&output.stdout);
        let outputs: Vec<&str> = binding.lines().collect();

        let mut process_found = false;
        for line in &outputs {
            if line.contains(&self.filename) {
                process_found = true;
                let pid = line.split_whitespace().next().unwrap_or_default();
                log_debug!(
                    "Found process for custom client '{}' with PID: {}",
                    self.name,
                    pid
                );

                let mut kill_command = Command::new("taskkill");

                #[cfg(windows)]
                kill_command.creation_flags(0x0800_0000);

                let kill_output = kill_command
                    .arg("/PID")
                    .arg(pid)
                    .arg("/F")
                    .output()
                    .map_err(|e| {
                        log_error!("Failed to execute taskkill for PID {}: {}", pid, e);
                        format!("Failed to kill process: {e}")
                    })?;

                if kill_output.status.success() {
                    log_info!(
                        "Successfully killed process {} for custom client '{}'",
                        pid,
                        self.name
                    );
                } else {
                    log_error!(
                        "taskkill failed for PID {}: {}",
                        pid,
                        String::from_utf8_lossy(&kill_output.stderr)
                    );
                }
            }
        }

        if !process_found {
            crate::log_info!("No process found for custom client: {}", self.name);
        }

        Ok(())
    }
}
