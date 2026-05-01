use std::{
    collections::HashMap,
    path::{Path, PathBuf, MAIN_SEPARATOR},
    sync::{Arc, Mutex},
};

use chrono::{DateTime, Utc};
use semver::Version;
use serde::{Deserialize, Serialize};
use tauri::AppHandle;
use tokio::sync::Semaphore;

use crate::core::clients::manager::ClientManager;
use crate::core::storage::data::{Data, DATA};
use crate::core::utils::{
    globals::{
        CUSTOM_CLIENTS_FOLDER, FILE_EXTENSION, IS_LINUX, IS_MACOS, IS_WINDOWS, JDK21_FOLDER,
        JDK8_FOLDER, MINECRAFT_VERSIONS_FOLDER, MODS_FOLDER,
    },
    process,
};
use crate::{log_error, log_info};

mod launch;
mod requirements;

pub static CLIENT_LOGS: std::sync::LazyLock<Mutex<HashMap<u32, Vec<String>>>> =
    std::sync::LazyLock::new(|| Mutex::new(HashMap::new()));

pub static REQUIREMENTS_DOWNLOADING: std::sync::LazyLock<Mutex<bool>> =
    std::sync::LazyLock::new(|| Mutex::new(false));

pub static REQUIREMENTS_SEMAPHORE: std::sync::LazyLock<Arc<Semaphore>> =
    std::sync::LazyLock::new(|| Arc::new(Semaphore::new(1)));

const MAX_CLIENT_LOG_LINES: usize = 5000;

fn sanitize_version_for_paths(version: &str) -> String {
    version.trim().replace(['/', '\\'], "_").replace(' ', "_")
}

fn is_minecraft_version_dir_name(name: &str) -> bool {
    let parts: Vec<&str> = name.split('.').collect();
    if !(2..=3).contains(&parts.len()) {
        return false;
    }
    if parts[0] != "1" {
        return false;
    }
    parts
        .iter()
        .all(|p| !p.is_empty() && p.chars().all(|c| c.is_ascii_digit()))
}

fn collect_jars_recursive(dir: &Path, skip_root_mc_version_dirs: bool) -> Vec<PathBuf> {
    let mut jars = Vec::new();

    if !dir.exists() {
        return jars;
    }

    let mut dirs_to_visit = vec![(dir.to_path_buf(), 0)];
    let max_depth = 15;

    while let Some((current_dir, depth)) = dirs_to_visit.pop() {
        if depth >= max_depth {
            continue;
        }

        if let Ok(entries) = std::fs::read_dir(&current_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    if skip_root_mc_version_dirs && current_dir == dir {
                        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                            if is_minecraft_version_dir_name(name) {
                                continue;
                            }
                        }
                    }
                    dirs_to_visit.push((path, depth + 1));
                } else if path
                    .extension()
                    .is_some_and(|ext| ext.eq_ignore_ascii_case("jar"))
                {
                    let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

                    // Filter by platform (only for natives)
                    let mut should_include = true;
                    let is_native = filename.contains("natives");

                    if is_native {
                        if IS_WINDOWS {
                            if filename.contains("-linux") || filename.contains("-macos") {
                                should_include = false;
                            }
                        } else if IS_LINUX {
                            if filename.contains("-windows") || filename.contains("-macos") {
                                should_include = false;
                            }
                        } else if IS_MACOS
                            && (filename.contains("-windows") || filename.contains("-linux")) {
                                should_include = false;
                            }
                    }

                    if should_include {
                        let filename_lower = filename.to_lowercase();
                        if filename_lower.contains("slf4j") || filename_lower.contains("log4j") {
                            //log_info!("Found critical logging library: {}", filename);
                        } else {
                            //log_debug!("Found JAR: {}", filename);
                        }
                        jars.push(path);
                    }
                }
            }
        }
    }
    jars.sort();
    jars
}

fn dir_has_any_jars(dir: &Path, skip_root_mc_version_dirs: bool) -> bool {
    let jars = collect_jars_recursive(dir, skip_root_mc_version_dirs);
    !jars.is_empty()
}

fn add_log_line(client_id: u32, line: String) {
    if let Ok(mut logs) = CLIENT_LOGS.lock() {
        if let Some(client_logs) = logs.get_mut(&client_id) {
            client_logs.push(line);
            if client_logs.len() > MAX_CLIENT_LOG_LINES {
                let to_remove = client_logs.len() - MAX_CLIENT_LOG_LINES;
                client_logs.drain(0..to_remove);
            }
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Default)]
pub struct Meta {
    pub is_new: bool,
    pub is_fabric: bool,
    pub is_forge: bool,
    pub asset_index: String,
    pub installed: bool,
    pub is_custom: bool,
    pub size: u64,
}

impl Meta {
    pub fn new(version: &str, filename: &str, client_type: &ClientType) -> Self {
        let semver = Version::parse(version).unwrap_or_else(|err| {
            log_error!("Failed to parse version '{}': {}", version, err);
            Version::new(1, 16, 5)
        });

        let asset_index = format!("{}.{}", semver.major, semver.minor);
        let is_new_version = semver.minor >= 16;
        let is_fabric = *client_type == ClientType::Fabric || filename.contains("fabric/");
        let is_forge = *client_type == ClientType::Forge || filename.contains("forge/");

        let jar_path = match client_type {
            ClientType::Fabric | ClientType::Forge => {
                let jar_basename = Path::new(filename)
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or(filename);
                DATA.root_dir
                    .lock()
                    .unwrap()
                    .join(Data::get_filename(filename))
                    .join(MODS_FOLDER)
                    .join(jar_basename)
            }
            ClientType::Default => {
                if filename.contains("fabric/") || filename.contains("forge/") {
                    let jar_basename = Path::new(filename)
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or(filename);
                    DATA.root_dir
                        .lock()
                        .unwrap()
                        .join(Data::get_filename(filename))
                        .join(MODS_FOLDER)
                        .join(jar_basename)
                } else {
                    DATA.get_local(&format!(
                        "{}{}{}",
                        Data::get_filename(filename),
                        MAIN_SEPARATOR,
                        filename
                    ))
                }
            }
        };

        Self {
            is_new: is_new_version,
            asset_index,
            installed: jar_path.exists(),
            is_custom: false,
            is_fabric,
            is_forge,
            size: 0,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Default)]
pub enum ClientType {
    #[serde(rename = "default")]
    #[default]
    Default,
    #[serde(rename = "fabric")]
    Fabric,
    #[serde(rename = "forge")]
    Forge,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
pub struct Client {
    pub id: u32,
    pub name: String,
    pub version: String,
    pub filename: String,
    pub md5_hash: String,
    pub main_class: String,
    pub show: bool,
    pub working: bool,
    #[serde(default)]
    pub insecure: bool,
    pub launches: u32,
    pub downloads: u32,
    #[serde(default)]
    pub rating_avg: Option<f32>,
    #[serde(default)]
    pub rating_count: Option<u32>,
    #[serde(default)]
    pub size: u64,
    #[serde(default = "default_meta")]
    pub meta: Meta,
    #[serde(default)]
    pub dependencies: Option<Vec<Requirement>>,
    #[serde(default)]
    pub client_type: ClientType,
    #[serde(default = "default_created_at")]
    pub created_at: DateTime<Utc>,
    #[serde(default)]
    pub java_path: Option<String>,
    #[serde(default)]
    pub java_args: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Requirement {
    name: String,
    md5_hash: Option<String>,
    size: Option<u64>,
}

fn default_meta() -> Meta {
    Meta {
        is_new: false,
        is_fabric: false,
        is_forge: false,
        asset_index: String::new(),
        installed: false,
        is_custom: false,
        size: 0,
    }
}

fn default_created_at() -> DateTime<Utc> {
    Utc::now()
}

pub struct LaunchOptions {
    pub app_handle: AppHandle,
    pub user_token: String,
    pub is_custom: bool,
}

impl LaunchOptions {
    pub fn new(app_handle: AppHandle, user_token: String, is_custom: bool) -> Self {
        Self {
            app_handle,
            user_token,
            is_custom,
        }
    }
}

impl Client {
    fn is_legacy_client(&self) -> bool {
        let semver = Version::parse(&self.version).unwrap_or_else(|_| Version::new(1, 12, 2));
        semver.major == 1 && semver.minor <= 12
    }

    fn jdk_folder_name(&self) -> &'static str {
        if self.client_type == ClientType::Forge {
            JDK8_FOLDER
        } else {
            JDK21_FOLDER
        }
    }

    fn jdk_zip_name(&self) -> String {
        if self.client_type == ClientType::Forge {
            format!("misc/{JDK8_FOLDER}.zip")
        } else {
            format!("misc/{JDK21_FOLDER}.zip")
        }
    }

    fn java_executable_path(&self) -> PathBuf {
        DATA.root_dir
            .lock()
            .unwrap()
            .join(self.jdk_folder_name())
            .join("bin")
            .join(format!("java{FILE_EXTENSION}"))
    }

    pub fn get_launch_paths(&self) -> Result<(PathBuf, PathBuf), String> {
        if self.meta.is_custom {
            let folder = DATA
                .root_dir
                .lock()
                .unwrap()
                .join(CUSTOM_CLIENTS_FOLDER)
                .join(&self.name);
            let jar = folder.join(&self.filename);
            return Ok((folder, jar));
        }

        let base_name = Data::get_filename(&self.filename);
        let folder = DATA.root_dir.lock().unwrap().join(&base_name);

        match self.client_type {
            ClientType::Forge | ClientType::Fabric => {
                let jar_basename = Path::new(&self.filename)
                    .file_name()
                    .and_then(|n| n.to_str())
                    .ok_or("Invalid filename")?;
                Ok((folder.clone(), folder.join(MODS_FOLDER).join(jar_basename)))
            }
            ClientType::Default => {
                let jar_basename = Path::new(&self.filename)
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or(&self.filename);
                if self.filename.contains("fabric/") {
                    Ok((folder.clone(), folder.join(MODS_FOLDER).join(jar_basename)))
                } else {
                    Ok((folder.clone(), folder.join(jar_basename)))
                }
            }
        }
    }

    fn get_minecraft_jar_path(&self) -> PathBuf {
        if self.meta.is_custom && self.client_type == ClientType::Default {
            return self
                .get_launch_paths()
                .map(|(_, jar)| jar)
                .unwrap_or_default();
        }

        let safe_ver = sanitize_version_for_paths(&self.version);
        let root = DATA.root_dir.lock().unwrap();
        root.join(MINECRAFT_VERSIONS_FOLDER)
            .join(match self.client_type {
                ClientType::Fabric => format!("fabric_{}.jar", safe_ver),
                ClientType::Forge => format!("forge_{}.jar", safe_ver),
                ClientType::Default => format!("{}.jar", safe_ver),
            })
    }

    pub fn get_running_clients(manager: &Arc<Mutex<ClientManager>>) -> Vec<Self> {
        let clients = manager
            .lock()
            .ok()
            .map(|manager| manager.clients.clone())
            .unwrap_or_default();

        process::filter_running(clients, |client| &client.filename)
    }

    pub fn get_client_folder(&self) -> Result<PathBuf, String> {
        let (folder, _) = self.get_launch_paths()?;
        Ok(folder)
    }

    pub fn stop(&self) -> Result<(), String> {
        process::stop_process_by_filename(&self.filename, &self.name)
    }

    fn mark_installed(&self, manager: &Arc<Mutex<ClientManager>>, installed: bool) {
        ClientManager::get_client(manager, self.id, |c| {
            c.meta.installed = installed;
        });
    }

    pub fn remove_installation(&self, manager: &Arc<Mutex<ClientManager>>) -> Result<(), String> {
        let (_, jar_path) = self.get_launch_paths()?;

        if jar_path.exists() {
            log_info!("Deleting jar: {}", jar_path.display());
            std::fs::remove_file(&jar_path).map_err(|e| e.to_string())?;
        }

        if self.client_type == ClientType::Forge {
            let mc_jar = self.get_minecraft_jar_path();
            if mc_jar.exists() {
                let _ = std::fs::remove_file(mc_jar);
            }
        }

        self.mark_installed(manager, false);
        Ok(())
    }
}
