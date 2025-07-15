use std::{
    collections::HashMap,
    io::{BufRead, BufReader},
    process::{Command, Stdio},
    sync::{Arc, Mutex},
    thread,
};

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

use crate::api::{
    analytics::Analytics,
    clients::{agent_overlay::AgentArguments, log_checker::LogChecker},
    core::accounts::ACCOUNT_MANAGER,
    utils,
};
use crate::{
    api::core::{data::DATA, settings::SETTINGS},
    log_debug, log_error, log_info,
};
use semver::Version;
use serde::{Deserialize, Serialize};
use tauri::AppHandle;
use tokio::sync::Semaphore;

use super::clients::CLIENT_MANAGER;

lazy_static::lazy_static! {
    pub static ref CLIENT_LOGS: Mutex<HashMap<u32, Vec<String>>> = Mutex::new(HashMap::new());
    pub static ref REQUIREMENTS_DOWNLOADING: Mutex<bool> = Mutex::new(false);
    pub static ref REQUIREMENTS_SEMAPHORE: Arc<Semaphore> = Arc::new(Semaphore::new(1));
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Meta {
    pub is_new: bool,
    pub asset_index: String,
    pub installed: bool,
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

        let file_name = DATA.get_filename(filename);
        let jar_path = DATA.get_local(&format!(
            "{}{}{}",
            file_name,
            std::path::MAIN_SEPARATOR,
            filename
        ));

        Self {
            is_new: is_new_version,
            asset_index,
            installed: jar_path.exists(),
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

#[derive(Serialize, Deserialize, Clone, Debug)]
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
}

fn default_meta() -> Meta {
    Meta {
        is_new: false,
        asset_index: String::new(),
        installed: false,
        size: 0,
    }
}

impl Client {
    pub async fn download(&self) -> Result<(), String> {
        match DATA.download(&self.filename).await {
            Ok(()) => {
                log_info!("Successfully downloaded and verified client: {}", self.name);
                if let Ok(mut manager) = CLIENT_MANAGER.lock() {
                    if let Some(manager) = manager.as_mut() {
                        if let Some(client) = manager.clients.iter_mut().find(|c| c.id == self.id) {
                            client.meta.installed = true;
                            client.meta.size = self.size;
                        }
                    }
                }
                Ok(())
            }
            Err(e) => {
                log_error!("Failed to download client {}: {}", self.name, e);
                if let Ok(mut manager) = CLIENT_MANAGER.lock() {
                    if let Some(manager) = manager.as_mut() {
                        if let Some(client) = manager.clients.iter_mut().find(|c| c.id == self.id) {
                            client.meta.installed = false;
                        }
                    }
                }
                Err(e)
            }
        }
    }

    pub fn remove_installation(&self) -> Result<(), String> {
        let file_name = DATA.get_filename(&self.filename);
        let jar_path = format!(
            "{}{}{}",
            file_name,
            std::path::MAIN_SEPARATOR,
            self.filename
        );
        let jar_file = DATA.get_local(&jar_path);

        if jar_file.exists() {
            std::fs::remove_file(&jar_file)
                .map_err(|e| format!("Failed to remove client JAR file: {e}"))?;
        }

        if let Ok(mut manager) = CLIENT_MANAGER.lock() {
            if let Some(manager) = manager.as_mut() {
                if let Some(client) = manager.clients.iter_mut().find(|c| c.id == self.id) {
                    client.meta.installed = false;
                }
            }
        }

        Ok(())
    }

    pub fn get_running_clients() -> Vec<Client> {
        let jps_path = DATA.root_dir.join("jdk-21.0.2").join("bin").join("jps.exe");
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
        let jps_path = DATA.root_dir.join("jdk-21.0.2").join("bin").join("jps.exe");
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
            log_info!("No process found for client: {}", self.name);
        }

        Ok(())
    }

    pub async fn download_requirements(&self, app_handle: &AppHandle) -> Result<(), String> {
        let _permit = REQUIREMENTS_SEMAPHORE
            .acquire()
            .await
            .map_err(|_| "Failed to acquire requirements download semaphore".to_string())?;

        let mut requirements_to_check = vec!["jdk-21.0.2.zip", "assets.zip"];

        if self.meta.is_new {
            requirements_to_check.push("natives.zip");
            requirements_to_check.push("libraries.zip");
        } else {
            requirements_to_check.push("natives-1.12.zip");
            requirements_to_check.push("libraries-1.12.zip");
        }

        let files_to_download: Vec<&str> = requirements_to_check
            .iter()
            .filter(|file| !DATA.get_as_folder(file).exists())
            .copied()
            .collect();

        if files_to_download.is_empty() {
            return Ok(());
        }

        log_info!("Need to download requirements: {:?}", files_to_download);

        utils::emit_to_main_window(app_handle, "requirements-status", true);

        {
            let mut downloading = REQUIREMENTS_DOWNLOADING
                .lock()
                .map_err(|_| "Failed to lock REQUIREMENTS_DOWNLOADING mutex".to_string())?;
            *downloading = true;
        }

        for file_to_dl in files_to_download {
            log_info!("Downloading requirement: {}", file_to_dl);
            DATA.download(file_to_dl).await.map_err(|e| {
                log_error!("Failed to download {}: {}", file_to_dl, e);
                format!("Failed to download {}: {}", file_to_dl, e)
            })?;
            log_info!("Successfully downloaded {}", file_to_dl);
        }

        log_info!("All requirements downloaded successfully");

        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

        {
            let mut downloading = REQUIREMENTS_DOWNLOADING
                .lock()
                .map_err(|_| "Failed to lock REQUIREMENTS_DOWNLOADING mutex".to_string())?;
            *downloading = false;
        }

        utils::emit_to_main_window(app_handle, "requirements-status", false);

        Ok(())
    }

    pub async fn run(self, app_handle: AppHandle, user_token: String) -> Result<(), String> {
        Analytics::send_client_analytics(self.id);

        {
            let mut logs = CLIENT_LOGS.lock().unwrap();
            logs.insert(self.id, Vec::new());
        }

        let client_id = self.id;
        let client_name = self.name.clone();
        let app_handle_clone_for_run = app_handle.clone();
        let app_handle_clone_for_crash_handling = app_handle.clone();
        let optional_analytics = SETTINGS
            .lock()
            .map_or(false, |s| s.optional_telemetry.value);
        let cordshare = SETTINGS.lock().map_or(false, |s| s.cordshare.value);
        let irc_chat = SETTINGS.lock().map_or(false, |s| s.irc_chat.value);

        let agent_arguments = AgentArguments::new(
            user_token.clone(),
            client_name.clone(),
            optional_analytics,
            cordshare,
            irc_chat,
        );

        agent_arguments.log_info();

        if let Err(e) = self.download_requirements(&app_handle_clone_for_run).await {
            log_info!("Error downloading requirements: {}", e);
            utils::emit_to_main_window_filtered(
                &app_handle_clone_for_crash_handling,
                "client-crashed",
                serde_json::json!({
                    "id": client_id,
                    "name": client_name.clone(),
                    "error": e.clone()
                }),
            );
            return Err(e);
        }

        let self_clone = self.clone();
        let handle = std::thread::spawn(move || -> Result<(), String> {
            let (natives_path, libraries_path) = if self_clone.meta.is_new {
                (
                    DATA.root_dir.join("natives"),
                    DATA.root_dir.join("libraries"),
                )
            } else {
                (
                    DATA.root_dir.join("natives-1.12"),
                    DATA.root_dir.join("libraries-1.12"),
                )
            };

            let client_folder = DATA
                .root_dir
                .join(DATA.get_as_folder_string(&self_clone.filename));
            let client_jar_path = client_folder.join(&self_clone.filename);

            let agent_overlay_folder = DATA.root_dir.join("agent_overlay");

            let classpath = format!(
                "{}{}*;{};{}",
                libraries_path.display(),
                std::path::MAIN_SEPARATOR,
                agent_overlay_folder.display(),
                client_jar_path.display(),
            );

            let java_executable = DATA
                .root_dir
                .join("jdk-21.0.2")
                .join("bin")
                .join(if cfg!(windows) { "java.exe" } else { "java" });

            let mut command = Command::new(java_executable);

            #[cfg(windows)]
            command.creation_flags(0x08000000);

            std::env::set_current_dir(&client_folder)
                .map_err(|e| format!("Failed to set current directory: {e}"))?;

            let username = ACCOUNT_MANAGER
                .lock()
                .ok()
                .and_then(|manager| manager.get_active_account().map(|a| a.username.clone()))
                .unwrap_or_else(|| {
                    let random_digits = rand::random::<u32>() % 100000;
                    format!("Collapse{:05}", random_digits)
                });

            let assets_dir = DATA.root_dir.join("assets");
            let ram_mb = SETTINGS.lock().map(|s| s.ram.value).unwrap_or(3072);

            command
                .arg("-Xverify:none")
                .arg(format!(
                    "-javaagent:{}={}",
                    agent_overlay_folder.join("CollapseAgent.jar").display(),
                    agent_arguments.encrypt()
                ))
                .arg(format!("-Xmx{}M", ram_mb))
                .arg(format!(
                    "-Djava.library.path={};{}",
                    natives_path.display(),
                    agent_overlay_folder.display()
                ))
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

            let mut secure_command = format!("{:#?}", command);

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

            utils::emit_to_main_window_filtered(
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
                    for line in reader.lines().filter_map(Result::ok) {
                        add_log_line(client_id, line);
                    }
                });
            }

            if let Some(stderr) = child.stderr.take() {
                thread::spawn(move || {
                    let reader = BufReader::new(stderr);
                    for line in reader.lines().filter_map(Result::ok) {
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
                    utils::emit_to_main_window_filtered(
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
                    utils::emit_to_main_window_filtered(
                        &app_handle_clone_for_crash_handling,
                        "client-crashed",
                        serde_json::json!({
                            "id": client_id,
                            "name": self_clone.name.clone(),
                            "error": log_line.clone()
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
