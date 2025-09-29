use std::{
    collections::HashMap,
    io::{BufRead, BufReader},
    process::{Command, Stdio},
    sync::{Arc, Mutex},
    thread,
};

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

use super::manager::CLIENT_MANAGER;
use crate::core::storage::accounts::ACCOUNT_MANAGER;
use crate::core::utils::globals::FILE_EXTENSION;
use crate::core::utils::helpers::{emit_to_main_window, emit_to_main_window_filtered};
use crate::core::{clients::internal::agent_overlay::AgentArguments, utils::globals::JDK_FOLDER};
use crate::core::{clients::log_checker::LogChecker, utils::globals::IS_LINUX};
use crate::core::{network::analytics::Analytics, storage::data::Data};
use crate::{
    core::storage::{data::DATA, settings::SETTINGS},
    log_debug, log_error, log_info,
};
use chrono::{DateTime, Utc};
use semver::Version;
use serde::{Deserialize, Serialize};
use tauri::AppHandle;
use tokio::sync::Semaphore;

pub static CLIENT_LOGS: std::sync::LazyLock<Mutex<HashMap<u32, Vec<String>>>> =
    std::sync::LazyLock::new(|| Mutex::new(HashMap::new()));
pub static REQUIREMENTS_DOWNLOADING: std::sync::LazyLock<Mutex<bool>> =
    std::sync::LazyLock::new(|| Mutex::new(false));
pub static REQUIREMENTS_SEMAPHORE: std::sync::LazyLock<Arc<Semaphore>> =
    std::sync::LazyLock::new(|| Arc::new(Semaphore::new(1)));

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Default)]
pub struct Meta {
    pub is_new: bool,
    pub asset_index: String,
    pub installed: bool,
    pub is_custom: bool,
    pub size: u64,
}

impl Meta {
    pub fn new(version: &str, filename: &str) -> Self {
        let semver = Version::parse(version).unwrap_or_else(|err| {
            log_error!("Failed to parse version '{}': {}", version, err);
            Version::new(1, 16, 5)
        });

        let asset_index = format!("{}.{}", semver.major, semver.minor);
        let is_new_version = semver.minor >= 16;

        let client_base_name = Data::get_filename(filename);
        let jar_path = if filename.contains("fabric/") {
            let jar_basename = std::path::Path::new(filename)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or(filename);
            DATA.root_dir
                .join(&client_base_name)
                .join("mods")
                .join(jar_basename)
        } else {
            DATA.get_local(&format!(
                "{}{}{}",
                client_base_name,
                std::path::MAIN_SEPARATOR,
                filename
            ))
        };

        Self {
            is_new: is_new_version,
            asset_index,
            installed: jar_path.exists(),
            is_custom: false,
            size: 0,
        }
    }
}

fn add_log_line(client_id: u32, line: String) {
    if let Ok(mut logs) = CLIENT_LOGS.lock() {
        if let Some(client_logs) = logs.get_mut(&client_id) {
            client_logs.push(line);
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
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Default)]
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
    pub size: u64,
    #[serde(default = "default_meta")]
    pub meta: Meta,
    #[serde(default)]
    pub requirement_mods: Option<Vec<String>>,
    #[serde(default)]
    pub client_type: ClientType,
    #[serde(default = "default_created_at")]
    pub created_at: DateTime<Utc>,
}

const fn default_meta() -> Meta {
    Meta {
        is_new: false,
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
    pub const fn new(app_handle: AppHandle, user_token: String, is_custom: bool) -> Self {
        Self {
            app_handle,
            user_token,
            is_custom,
        }
    }
}

impl Client {
    pub async fn download(&self) -> Result<(), String> {
        log_debug!(
            "Starting download for client '{}' filename='{}'",
            self.name,
            self.filename
        );
        match DATA.download(&self.filename).await {
            Ok(()) => {
                log_info!("Successfully downloaded client '{}'", self.name);
                if let Ok(mut manager) = CLIENT_MANAGER.lock() {
                    if let Some(manager) = manager.as_mut() {
                        if let Some(client) = manager.clients.iter_mut().find(|c| c.id == self.id) {
                            client.meta.installed = true;
                            client.meta.size = self.size;
                            log_debug!(
                                "Updated manager: marked '{}' installed, size={}",
                                self.name,
                                self.size
                            );
                        }
                    }
                }
                Ok(())
            }
            Err(e) => {
                log_error!(
                    "Failed to download client '{}' filename='{}' : {}",
                    self.name,
                    self.filename,
                    e
                );
                if let Ok(mut manager) = CLIENT_MANAGER.lock() {
                    if let Some(manager) = manager.as_mut() {
                        if let Some(client) = manager.clients.iter_mut().find(|c| c.id == self.id) {
                            client.meta.installed = false;
                            log_debug!(
                                "Updated manager: marked '{}' not installed after failure",
                                self.name
                            );
                        }
                    }
                }
                Err(e)
            }
        }
    }

    pub fn remove_installation(&self) -> Result<(), String> {
        let client_folder = DATA.get_as_folder(&self.filename);
        log_debug!(
            "Removing installation for client '{}' at {}",
            self.name,
            client_folder.display()
        );

        if client_folder.exists() {
            std::fs::remove_dir_all(&client_folder).map_err(|e| {
                log_error!(
                    "Failed to remove client folder '{}' : {}",
                    client_folder.display(),
                    e
                );
                format!("Failed to remove client folder: {e}")
            })?;
            log_info!("Removed installation folder for '{}'", self.name);
        } else {
            log_debug!(
                "No installation folder found for '{}', skipping removal",
                self.name
            );
        }

        if let Ok(mut manager) = CLIENT_MANAGER.lock() {
            if let Some(manager) = manager.as_mut() {
                if let Some(client) = manager.clients.iter_mut().find(|c| c.id == self.id) {
                    client.meta.installed = false;
                    log_debug!(
                        "Updated manager: marked '{}' not installed after removal",
                        self.name
                    );
                }
            }
        }

        Ok(())
    }

    pub fn get_running_clients() -> Vec<Self> {
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
            Err(_) => {
                return Vec::new();
            }
        };

        let binding = String::from_utf8_lossy(&output.stdout);
        let outputs: Vec<&str> = binding.lines().collect();

        let clients = CLIENT_MANAGER
            .lock()
            .ok()
            .and_then(|guard| guard.as_ref().map(|manager| manager.clients.clone()))
            .unwrap_or_default();

        clients
            .into_iter()
            .filter(|client| outputs.iter().any(|line| line.contains(&client.filename)))
            .collect()
    }

    pub fn stop(&self) -> Result<(), String> {
        let jps_path = DATA
            .root_dir
            .join(JDK_FOLDER)
            .join("bin")
            .join("jps".to_owned() + FILE_EXTENSION);
        let mut command = Command::new(jps_path);

        #[cfg(windows)]
        command.creation_flags(0x0800_0000);

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
                kill_command.creation_flags(0x0800_0000);

                let kill_output = kill_command
                    .arg("/PID")
                    .arg(pid)
                    .arg("/F")
                    .output()
                    .map_err(|e| {
                        log_error!(
                            "Failed to kill process {} for client '{}': {}",
                            pid,
                            self.name,
                            e
                        );
                        format!("Failed to kill process: {e}")
                    })?;

                if !kill_output.status.success() {
                    log_error!(
                        "taskkill returned non-zero for PID {}: stdout='{}' stderr='{}'",
                        pid,
                        String::from_utf8_lossy(&kill_output.stdout),
                        String::from_utf8_lossy(&kill_output.stderr)
                    );
                } else {
                    log_info!("Successfully killed PID {} for client '{}'", pid, self.name);
                }
            }
        }

        if !process_found {
            log_info!("No process found for client: {}", self.name);
        }

        Ok(())
    }

    fn determine_requirements_to_check(&self) -> Vec<String> {
        let mut requirements = vec![format!("{JDK_FOLDER}.zip")];
        if self.client_type == ClientType::Fabric {
            requirements.push("assets_fabric.zip".to_string());
            requirements.push("libraries_fabric.zip".to_string());
        } else {
            requirements.push("assets.zip".to_string());
            if self.meta.is_new {
                requirements.push(if !IS_LINUX {
                    "natives.zip".to_string()
                } else {
                    "natives-linux.zip".to_string()
                });
                requirements.push("libraries.zip".to_string());
            } else {
                requirements.push(if !IS_LINUX {
                    "natives-1.12.zip".to_string()
                } else {
                    "natives-linux.zip".to_string()
                });
                requirements.push("libraries-1.12.zip".to_string());
            }
        }
        requirements
    }

    fn check_if_download_needed(
        &self,
        requirements_to_check: &[String],
        client_jar: &Option<String>,
    ) -> (bool, Vec<String>) {
        let files_to_download: Vec<String> = requirements_to_check
            .iter()
            .filter(|file| !DATA.get_as_folder(file).exists())
            .cloned()
            .collect();

        let mut need_download = !files_to_download.is_empty();
        log_debug!(
            "Files missing check for '{}': missing_count={}",
            self.name,
            files_to_download.len()
        );

        if let Some(ref fabric_jar) = client_jar {
            if !DATA
                .root_dir
                .join("minecraft_versions")
                .join(fabric_jar)
                .exists()
            {
                need_download = true;
            }
        }

        if self.client_type == ClientType::Fabric {
            if let Some(mods) = &self.requirement_mods {
                let client_base = Data::get_filename(&self.filename);
                let mods_folder = DATA.root_dir.join(&client_base).join("mods");
                if mods.iter().any(|mod_name| {
                    let mod_basename = mod_name.trim_end_matches(".jar");
                    !mods_folder.join(format!("{mod_basename}.jar")).exists()
                }) {
                    need_download = true;
                }
            }
        }

        (need_download, files_to_download)
    }

    async fn download_file(&self, file_to_dl: &str) -> Result<(), String> {
        log_info!(
            "Downloading requirement '{}' for client '{}'",
            file_to_dl,
            self.name
        );
        DATA.download(file_to_dl).await.map_err(|e| {
            log_error!(
                "Failed to download '{}' for client '{}': {}",
                file_to_dl,
                self.name,
                e
            );
            format!("Failed to download {file_to_dl}: {e}")
        })?;

        if IS_LINUX && file_to_dl.starts_with(JDK_FOLDER) {
            let java_path = DATA.root_dir.join(JDK_FOLDER).join("bin").join("java");
            if java_path.exists() {
                #[cfg(unix)]
                if let Ok(mut perms) = std::fs::metadata(&java_path).map(|m| m.permissions()) {
                    use std::os::unix::fs::PermissionsExt;
                    perms.set_mode(0o755);
                    if let Err(e) = std::fs::set_permissions(&java_path, perms) {
                        log_error!("Failed to set exec perm on {}: {}", java_path.display(), e);
                    } else {
                        log_info!("Set exec perm on {}", java_path.display());
                    }
                }
            }
        }
        log_info!(
            "Successfully downloaded '{}' for '{}'",
            file_to_dl,
            self.name
        );
        Ok(())
    }

    async fn download_fabric_mods(&self) -> Result<(), String> {
        if self.client_type == ClientType::Fabric {
            if let Some(mods) = &self.requirement_mods {
                for mod_name in mods.iter() {
                    let mod_basename = mod_name.trim_end_matches(".jar");
                    let filename_on_cdn = format!("fabric/deps/{mod_basename}.jar");
                    let client_base = Data::get_filename(&self.filename);
                    let dest_folder = format!("{client_base}/mods");
                    let dest_path = DATA
                        .root_dir
                        .join(&client_base)
                        .join("mods")
                        .join(format!("{mod_basename}.jar"));

                    if !dest_path.exists() {
                        log_info!("Downloading Fabric requirement mod: {}", filename_on_cdn);
                        DATA.download_to_folder(&filename_on_cdn, &dest_folder)
                            .await
                            .map_err(|e| {
                                log_error!("Failed to download mod {filename_on_cdn}: {e}");
                                format!("Failed to download mod {filename_on_cdn}: {e}")
                            })?;
                        log_info!("Successfully downloaded mod {}", filename_on_cdn);
                    }
                }
            }
        }
        Ok(())
    }

    async fn download_required_files(
        &self,
        app_handle: &AppHandle,
        files_to_download: Vec<String>,
        client_jar: Option<String>,
    ) -> Result<(), String> {
        {
            let mut downloading = REQUIREMENTS_DOWNLOADING
                .lock()
                .map_err(|_| "Failed to lock REQUIREMENTS_DOWNLOADING mutex".to_string())?;
            *downloading = true;
        }
        emit_to_main_window(app_handle, "requirements-status", true);

        for file_to_dl in files_to_download {
            self.download_file(&file_to_dl).await?;
        }

        if let Some(client_jar) = client_jar {
            let dest_path = DATA.root_dir.join("minecraft_versions").join(&client_jar);
            if !dest_path.exists() {
                log_info!(
                    "Downloading MC client jar '{}' for '{}'",
                    client_jar,
                    self.name
                );
                DATA.download_to_folder(&client_jar, "minecraft_versions")
                    .await
                    .map_err(|e| {
                        log_error!("Failed to download MC client jar '{}': {}", client_jar, e);
                        format!("Failed to download MC client jar {client_jar}: {e}")
                    })?;
                log_info!("Successfully downloaded MC client jar '{}'", client_jar);
            }
        }

        self.download_fabric_mods().await?;

        log_info!("All requirements downloaded successfully");
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

        {
            let mut downloading = REQUIREMENTS_DOWNLOADING
                .lock()
                .map_err(|_| "Failed to lock REQUIREMENTS_DOWNLOADING mutex".to_string())?;
            *downloading = false;
        }
        emit_to_main_window(app_handle, "requirements-status", false);

        Ok(())
    }

    #[allow(clippy::cognitive_complexity)]
    pub async fn download_requirements(&self, app_handle: &AppHandle) -> Result<(), String> {
        let _permit = REQUIREMENTS_SEMAPHORE.acquire().await.map_err(|_| {
            log_error!(
                "Failed to acquire requirements semaphore for '{}'",
                self.name
            );
            "Failed to acquire requirements semaphore".to_string()
        })?;

        let requirements_to_check = self.determine_requirements_to_check();
        let client_jar = if self.client_type == ClientType::Fabric {
            Some(format!("fabric_{}.jar", self.version.replace(' ', "_")))
        } else {
            None
        };

        let (need_download, files_to_download) =
            self.check_if_download_needed(&requirements_to_check, &client_jar);

        if !need_download {
            log_info!(
                "All requirements present for '{}', skipping downloads",
                self.name
            );
            return Ok(());
        }

        log_info!(
            "Requirements missing for '{}' -> will download: {:?}",
            self.name,
            files_to_download
        );

        self.download_required_files(app_handle, files_to_download, client_jar)
            .await
    }

    pub async fn run(self, options: LaunchOptions) -> Result<(), String> {
        if !options.is_custom && SETTINGS.lock().is_ok_and(|s| s.optional_telemetry.value) {
            Analytics::send_client_analytics(self.id);
        }

        {
            let mut logs = CLIENT_LOGS.lock().unwrap();
            logs.insert(self.id, Vec::new());
        }

        let client_id = self.id;
        let client_name = self.name.clone();
        let app_handle_clone_for_run = options.app_handle.clone();
        let app_handle_clone_for_crash_handling = options.app_handle.clone();
        let optional_analytics = SETTINGS.lock().is_ok_and(|s| s.optional_telemetry.value);
        // let cordshare = SETTINGS.lock().is_ok_and(|s| s.cordshare.value);
        let irc_chat = SETTINGS.lock().is_ok_and(|s| s.irc_chat.value);

        let agent_arguments = AgentArguments::new(
            options.user_token.clone(),
            client_name.clone(),
            if self.meta.is_custom {
                false
            } else {
                optional_analytics
            },
            // cordshare,
            irc_chat,
        );

        agent_arguments.log_info();

        log_debug!(
            "Preparing to download requirements for client '{}'",
            self.name
        );
        if let Err(e) = self.download_requirements(&app_handle_clone_for_run).await {
            log_info!("Error downloading requirements for '{}' : {}", self.name, e);
            emit_to_main_window_filtered(
                &app_handle_clone_for_crash_handling,
                "client-crashed",
                serde_json::json!({
                    "id": client_id,
                    "name": client_name.clone(),
                    "error": e
                }),
            );
            return Err(e);
        }

        log_debug!("Spawning thread to run client '{}'", self.name);
        let self_clone = self.clone();
        let handle = std::thread::spawn(move || -> Result<(), String> {
            let (natives_path, libraries_path) = if self_clone.meta.is_new {
                (
                    DATA.root_dir
                        .join("natives".to_owned() + if IS_LINUX { "-linux" } else { "" }),
                    if self_clone.client_type == ClientType::Fabric {
                        DATA.root_dir.join("libraries_fabric")
                    } else {
                        DATA.root_dir.join("libraries")
                    },
                )
            } else {
                (
                    DATA.root_dir
                        .join("natives".to_owned() + if IS_LINUX { "-linux" } else { "-1.12" }),
                    DATA.root_dir.join("libraries-1.12"),
                )
            };

            let (client_folder, client_jar_path) = if self_clone.meta.is_custom {
                let folder = DATA.root_dir.join("custom_clients").join(&self_clone.name);
                let jar = folder.join(&self_clone.filename);
                (folder, jar)
            } else if self_clone.filename.contains("fabric/") {
                let base_name = Data::get_filename(&self_clone.filename);
                let folder = DATA.root_dir.join(&base_name);
                let jar_basename = std::path::Path::new(&self_clone.filename)
                    .file_name()
                    .unwrap();
                let jar = folder.join("mods").join(jar_basename);
                (folder, jar)
            } else {
                let folder = DATA
                    .root_dir
                    .join(Data::get_as_folder_string(&self_clone.filename));
                let jar = folder.join(&self_clone.filename);
                (folder, jar)
            };

            let agent_overlay_folder = DATA.root_dir.join("agent_overlay");
            let minecraft_client_folder = DATA.root_dir.join("minecraft_versions");

            let sep = if IS_LINUX { ":" } else { ";" };

            let classpath = if self_clone.client_type == ClientType::Fabric {
                format!(
                    "{}{}{}{}*{}{}",
                    minecraft_client_folder
                        .join(format!("fabric_{}.jar", self.version))
                        .display(),
                    sep,
                    libraries_path.display(),
                    std::path::MAIN_SEPARATOR,
                    sep,
                    agent_overlay_folder.display()
                )
            } else {
                format!(
                    "{}{}{}{}*{}{}",
                    client_jar_path.display(),
                    sep,
                    libraries_path.display(),
                    std::path::MAIN_SEPARATOR,
                    sep,
                    agent_overlay_folder.display()
                )
            };

            let java_executable = DATA
                .root_dir
                .join(JDK_FOLDER)
                .join("bin")
                .join("java".to_owned() + FILE_EXTENSION);

            let mut command = Command::new(java_executable);
            log_debug!(
                "Prepared java command for '{}' (will spawn shortly)",
                self_clone.name
            );

            #[cfg(windows)]
            command.creation_flags(0x0800_0000);

            std::env::set_current_dir(&client_folder)
                .map_err(|e| format!("Failed to set current directory: {e}"))?;

            let username = ACCOUNT_MANAGER
                .lock()
                .ok()
                .and_then(|manager| manager.get_active_account().map(|a| a.username.clone()))
                .unwrap_or_else(|| {
                    let random_digits = rand::random::<u32>() % 100_000;
                    format!("Collapse{random_digits:05}")
                });

            let assets_dir = if self_clone.client_type == ClientType::Fabric {
                DATA.root_dir.join("assets_fabric")
            } else {
                DATA.root_dir.join("assets")
            };

            let ram_mb = SETTINGS.lock().map(|s| s.ram.value).unwrap_or(3072);

            command.arg("-Xverify:none");

            if !IS_LINUX {
                command.arg(format!(
                    "-javaagent:{}={}",
                    agent_overlay_folder.join("CollapseAgent.jar").display(),
                    agent_arguments.encode()
                ));
            }

            command.arg(format!("-Xmx{ram_mb}M"));

            if self_clone.client_type != ClientType::Fabric {
                command.arg(format!(
                    "-Djava.library.path={}{}{}",
                    natives_path.display(),
                    sep,
                    agent_overlay_folder.display()
                ));
            } else if self_clone.client_type == ClientType::Fabric {
                command.arg(format!(
                    "-Djava.library.path={}",
                    agent_overlay_folder.display()
                ));
            }

            command
                .arg("-cp")
                .arg(&classpath)
                .arg(&self_clone.main_class)
                .arg("--username")
                .arg(username)
                .arg("--gameDir")
                .arg(client_folder.display().to_string())
                .arg("--assetsDir")
                .arg(assets_dir.display().to_string())
                .arg("--assetIndex")
                .arg(&self_clone.meta.asset_index)
                .arg("--uuid")
                .arg("N/A")
                .arg("--accessToken")
                .arg("0")
                .arg("--userType")
                .arg("legacy")
                .arg("--version")
                .arg(&self_clone.version)
                .arg("--client")
                .arg(&self_clone.filename)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped());

            add_log_line(client_id, format!("Starting client: {}", self_clone.name));

            let mut secure_command = format!("{command:#?}");

            if let Some(start) = secure_command.find("-javaagent:") {
                if let Some(end) = secure_command[start..].find(" -") {
                    let actual_end = start + end;
                    secure_command.replace_range(start..actual_end, "-javaagent:[HIDDEN]");
                } else if let Some(end) = secure_command[start..].find("\"") {
                    let actual_end = start + end;
                    secure_command.replace_range(start..actual_end, "-javaagent:[HIDDEN]");
                }
            }

            add_log_line(client_id, secure_command);

            let mut child = command
                .spawn()
                .map_err(|e| format!("Failed to start client: {e}"))?;

            emit_to_main_window_filtered(
                &app_handle_clone_for_crash_handling,
                "client-launched",
                serde_json::json!({
                    "id": client_id,
                    "name": self_clone.name.clone(),
                    "version": self_clone.version.clone()
                }),
            );

            if let Some(stdout) = child.stdout.take() {
                thread::spawn(move || {
                    let reader = BufReader::new(stdout);
                    for line in reader.lines().map_while(Result::ok) {
                        add_log_line(client_id, line);
                    }
                });
            }

            if let Some(stderr) = child.stderr.take() {
                thread::spawn(move || {
                    let reader = BufReader::new(stderr);
                    for line in reader.lines().map_while(Result::ok) {
                        add_log_line(client_id, line);
                    }
                });
            }

            match child.wait() {
                Ok(status) => {
                    let log_line = format!("Process finished with status: {status:?}");
                    log_debug!("{}", log_line);
                    add_log_line(client_id, log_line);

                    let log_checker = LogChecker::new(self_clone.clone());
                    log_checker.check(&app_handle_clone_for_crash_handling);

                    if let Ok(client_manager) = CLIENT_MANAGER.lock() {
                        if let Some(manager) = client_manager.as_ref() {
                            if let Err(e) = manager
                                .update_status_on_client_exit(&app_handle_clone_for_crash_handling)
                            {
                                log_error!("Failed to update user status on client exit: {}", e);
                            } else {
                                log_debug!("User status updated on client exit, cleared client playing status");
                            }
                        }
                    }

                    emit_to_main_window_filtered(
                        &app_handle_clone_for_crash_handling,
                        "client-exited",
                        serde_json::json!({
                            "id": client_id,
                            "name": self_clone.name.clone(),
                            "exitCode": status.code().unwrap_or(-1)
                        }),
                    );
                    Ok(())
                }
                Err(e) => {
                    let log_line = format!("Error waiting for process: {e}");
                    log_error!("{}", log_line);
                    add_log_line(client_id, log_line.clone());
                    emit_to_main_window_filtered(
                        &app_handle_clone_for_crash_handling,
                        "client-crashed",
                        serde_json::json!({
                            "id": client_id,
                            "name": self_clone.name.clone(),
                            "error": log_line
                        }),
                    );
                    Err(log_line)
                }
            }
        });

        handle
            .join()
            .map_err(|e| format!("Client execution thread panicked: {e:?}"))?
    }
}
