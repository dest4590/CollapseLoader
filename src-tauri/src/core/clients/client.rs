use std::{
    collections::HashMap,
    path::{Path, PathBuf, MAIN_SEPARATOR},
    process::Stdio,
    sync::{Arc, Mutex},
};

use chrono::{DateTime, Utc};
use semver::Version;
use serde::{Deserialize, Serialize};
use tauri::AppHandle;
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::Command,
    sync::Semaphore,
};

use crate::core::clients::internal::agent_overlay::AgentArguments;
use crate::core::clients::log_checker::LogChecker;
use crate::core::clients::manager::ClientManager;
use crate::core::network::analytics::Analytics;
use crate::core::storage::{
    accounts::ACCOUNT_MANAGER,
    data::{Data, DATA},
    settings::SETTINGS,
};
use crate::core::utils::{
    globals::{
        AGENT_FILE, AGENT_OVERLAY_FOLDER, ASSETS_FABRIC_FOLDER, ASSETS_FABRIC_ZIP, ASSETS_FOLDER,
        ASSETS_ZIP, CUSTOM_CLIENTS_FOLDER, FILE_EXTENSION, IS_LINUX, JDK8_FOLDER, JDK8_ZIP,
        JDK_FOLDER, LEGACY_SUFFIX, LIBRARIES_FABRIC_FOLDER, LIBRARIES_FABRIC_ZIP, LIBRARIES_FOLDER,
        LIBRARIES_LEGACY_FOLDER, LIBRARIES_LEGACY_ZIP, LIBRARIES_ZIP, LINUX_SUFFIX,
        MINECRAFT_VERSIONS_FOLDER, MODS_FOLDER, NATIVES_FOLDER, NATIVES_LEGACY_ZIP,
        NATIVES_LINUX_ZIP, NATIVES_ZIP, PATH_SEPARATOR,
    },
    hashing::calculate_md5_hash,
    helpers::emit_to_main_window,
    process,
};
use crate::{log_debug, log_error, log_info, log_warn};

pub static CLIENT_LOGS: std::sync::LazyLock<Mutex<HashMap<u32, Vec<String>>>> =
    std::sync::LazyLock::new(|| Mutex::new(HashMap::new()));

pub static REQUIREMENTS_DOWNLOADING: std::sync::LazyLock<Mutex<bool>> =
    std::sync::LazyLock::new(|| Mutex::new(false));

pub static REQUIREMENTS_SEMAPHORE: std::sync::LazyLock<Arc<Semaphore>> =
    std::sync::LazyLock::new(|| Arc::new(Semaphore::new(1)));

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

fn is_fabric_loader_jar(path: &Path) -> bool {
    path.file_name()
        .and_then(|n| n.to_str())
        .is_some_and(|n| n.starts_with("fabric-loader") && n.ends_with(".jar"))
}

fn collect_jars_recursive(dir: &Path, skip_root_mc_version_dirs: bool) -> Vec<PathBuf> {
    let mut jars = Vec::new();

    if !dir.exists() {
        return jars;
    }

    let mut dirs_to_visit = vec![(dir.to_path_buf(), 0)];
    let max_depth = 8;

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
                    jars.push(path);
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
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Default)]
pub struct Meta {
    pub is_new: bool,
    pub is_fabric: bool,
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

        let jar_path = match client_type {
            ClientType::Fabric | ClientType::Forge => {
                let jar_basename = Path::new(filename)
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or(filename);
                DATA.root_dir
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
    pub requirement_mods: Option<Vec<String>>,
    #[serde(default)]
    pub client_type: ClientType,
    #[serde(default = "default_created_at")]
    pub created_at: DateTime<Utc>,
}

const fn default_meta() -> Meta {
    Meta {
        is_new: false,
        is_fabric: false,
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
    fn is_legacy_client(&self) -> bool {
        let semver = Version::parse(&self.version).unwrap_or_else(|_| Version::new(1, 12, 2));
        semver.major == 1 && semver.minor <= 12
    }

    fn jdk_folder_name(&self) -> &'static str {
        if self.client_type == ClientType::Forge {
            JDK8_FOLDER
        } else {
            JDK_FOLDER
        }
    }

    fn jdk_zip_name(&self) -> String {
        if self.client_type == ClientType::Forge {
            JDK8_ZIP.to_string()
        } else {
            format!("{JDK_FOLDER}.zip")
        }
    }

    fn java_executable_path(&self) -> PathBuf {
        DATA.root_dir
            .join(self.jdk_folder_name())
            .join("bin")
            .join(format!("java{FILE_EXTENSION}"))
    }

    fn get_launch_paths(&self) -> Result<(PathBuf, PathBuf), String> {
        if self.meta.is_custom {
            let folder = DATA.root_dir.join(CUSTOM_CLIENTS_FOLDER).join(&self.name);
            let jar = folder.join(&self.filename);
            return Ok((folder, jar));
        }

        let base_name = Data::get_filename(&self.filename);
        let folder = DATA.root_dir.join(&base_name);

        match self.client_type {
            ClientType::Forge | ClientType::Fabric => {
                let jar_basename = Path::new(&self.filename)
                    .file_name()
                    .and_then(|n| n.to_str())
                    .ok_or("Invalid filename")?;
                Ok((folder.clone(), folder.join(MODS_FOLDER).join(jar_basename)))
            }
            ClientType::Default => {
                if self.filename.contains("fabric/") {
                    let jar_basename = Path::new(&self.filename).file_name().unwrap();
                    Ok((folder.clone(), folder.join(MODS_FOLDER).join(jar_basename)))
                } else {
                    Ok((folder.clone(), folder.join(&self.filename)))
                }
            }
        }
    }

    fn get_minecraft_jar_path(&self) -> PathBuf {
        let safe_ver = sanitize_version_for_paths(&self.version);
        match self.client_type {
            ClientType::Fabric => DATA
                .root_dir
                .join(MINECRAFT_VERSIONS_FOLDER)
                .join(format!("fabric_{}.jar", safe_ver)),
            ClientType::Forge => {
                let base_name = Data::get_filename(&self.filename);
                DATA.root_dir
                    .join(base_name)
                    .join(format!("forge_{}.jar", safe_ver))
            }
            ClientType::Default => self.get_launch_paths().unwrap_or_default().1,
        }
    }

    pub fn get_running_clients(manager: &Arc<Mutex<ClientManager>>) -> Vec<Self> {
        let clients = manager
            .lock()
            .ok()
            .map(|manager| manager.clients.clone())
            .unwrap_or_default();

        process::filter_running(clients, |client| &client.filename)
    }

    pub fn stop(&self) -> Result<(), String> {
        process::stop_process_by_filename(&self.filename, &self.name)
    }

    pub async fn download(&self, manager: &Arc<Mutex<ClientManager>>) -> Result<(), String> {
        log_debug!(
            "Starting download for client '{}' (type: {:?})",
            self.name,
            self.client_type
        );

        let download_result = match self.client_type {
            ClientType::Forge => {
                let client_base = Data::get_filename(&self.filename);
                let mods_folder = format!("{client_base}{MAIN_SEPARATOR}{MODS_FOLDER}");
                let safe_ver = sanitize_version_for_paths(&self.version);
                let forge_jar_name = format!("forge_{}.jar", safe_ver);

                DATA.download_to_folder(&self.filename, &mods_folder)
                    .await
                    .map_err(|e| format!("Forge Mod download failed: {e}"))?;

                DATA.download_to_folder(&forge_jar_name, &client_base)
                    .await
                    .map_err(|e| format!("Forge MC Jar download failed: {e}"))
            }
            _ => DATA.download(&self.filename).await,
        };

        if let Err(e) = download_result {
            log_error!("Download failed for {}: {}", self.name, e);
            self.mark_installed(manager, false);
            return Err(e);
        }

        if let Err(e) = self.verify_hash().await {
            self.mark_installed(manager, false);
            return Err(e);
        }

        if let Err(e) = self.download_fabric_mods().await {
            self.mark_installed(manager, false);
            return Err(e);
        }

        ClientManager::get_client(manager, self.id, |c| {
            c.meta.installed = true;
            c.meta.size = self.size;
        });
        log_debug!(
            "Client '{}' downloaded and installed successfully",
            self.name
        );

        Ok(())
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

    async fn verify_hash(&self) -> Result<(), String> {
        let verify = SETTINGS.lock().map(|s| s.hash_verify.value).unwrap_or(true);

        if !verify {
            return Ok(());
        }

        let (_, jar_path) = self.get_launch_paths()?;
        if !jar_path.exists() {
            return Err("File not found for hash verification".to_string());
        }

        let calculated = tokio::task::spawn_blocking(move || calculate_md5_hash(&jar_path))
            .await
            .map_err(|e| e.to_string())??;

        if calculated != self.md5_hash {
            return Err(format!(
                "Hash mismatch. Expected: {}, Got: {}",
                self.md5_hash, calculated
            ));
        }

        Ok(())
    }

    async fn download_fabric_mods(&self) -> Result<(), String> {
        if self.client_type != ClientType::Fabric {
            return Ok(());
        }

        if let Some(mods) = &self.requirement_mods {
            let client_base = Data::get_filename(&self.filename);
            let mods_folder_rel = format!("{client_base}{MAIN_SEPARATOR}{MODS_FOLDER}");
            let mods_folder_abs = DATA.root_dir.join(&client_base).join(MODS_FOLDER);

            for mod_name in mods {
                let mod_basename = mod_name.trim_end_matches(".jar");
                let dest = mods_folder_abs.join(format!("{mod_basename}.jar"));

                if !dest.exists() {
                    let remote_path = format!("fabric/deps/{mod_basename}.jar");
                    log_info!("Downloading dependency: {}", remote_path);
                    DATA.download_to_folder(&remote_path, &mods_folder_rel)
                        .await
                        .map_err(|e| {
                            format!("Failed to download dependency {mod_basename}: {e}")
                        })?;
                }
            }
        }
        Ok(())
    }

    pub async fn download_requirements(&self, app_handle: &AppHandle) -> Result<(), String> {
        let _permit = REQUIREMENTS_SEMAPHORE
            .acquire()
            .await
            .map_err(|_| "Failed to acquire requirement lock".to_string())?;

        let mut files_to_download = Vec::new();

        if !DATA.get_as_folder(&self.jdk_zip_name()).exists() {
            files_to_download.push(self.jdk_zip_name());
        }

        match self.client_type {
            ClientType::Fabric => {
                if !DATA.get_as_folder(ASSETS_FABRIC_ZIP).exists() {
                    files_to_download.push(ASSETS_FABRIC_ZIP.to_string());
                }
                let mc_jar = self.get_minecraft_jar_path();
                if !mc_jar.exists() {}
            }
            ClientType::Forge | ClientType::Default => {
                if !DATA.get_as_folder(ASSETS_ZIP).exists() {
                    files_to_download.push(ASSETS_ZIP.to_string());
                }

                let (libs, natives) = if self.is_legacy_client() {
                    (LIBRARIES_LEGACY_ZIP, NATIVES_LEGACY_ZIP)
                } else if self.meta.is_new {
                    (LIBRARIES_ZIP, NATIVES_ZIP)
                } else {
                    (LIBRARIES_LEGACY_ZIP, NATIVES_LEGACY_ZIP)
                };

                let actual_natives = if IS_LINUX { NATIVES_LINUX_ZIP } else { natives };

                if !DATA.get_as_folder(libs).exists() {
                    files_to_download.push(libs.to_string());
                }
                if !DATA.get_as_folder(actual_natives).exists() {
                    files_to_download.push(actual_natives.to_string());
                }
            }
        }

        let needs_jar_dl = match self.client_type {
            ClientType::Fabric => {
                if !self.get_minecraft_jar_path().exists() {
                    Some(format!(
                        "fabric_{}.jar",
                        sanitize_version_for_paths(&self.version)
                    ))
                } else {
                    None
                }
            }
            ClientType::Forge => {
                if !self.get_minecraft_jar_path().exists() {}
                None
            }
            _ => None,
        };

        if !files_to_download.is_empty() || needs_jar_dl.is_some() {
            self.batch_download_requirements(app_handle, files_to_download, needs_jar_dl)
                .await?;
        }

        match self.client_type {
            ClientType::Fabric => self.ensure_fabric_libraries().await?,
            ClientType::Forge => {
                let forge_jar_path = self.get_minecraft_jar_path();
                if !forge_jar_path.exists() {
                    let base_name = Data::get_filename(&self.filename);
                    let safe_ver = sanitize_version_for_paths(&self.version);
                    DATA.download_to_folder(&format!("forge_{}.jar", safe_ver), &base_name)
                        .await?;
                }
            }
            _ => {}
        }

        Ok(())
    }

    async fn batch_download_requirements(
        &self,
        app_handle: &AppHandle,
        files: Vec<String>,
        extra_mc_jar: Option<String>,
    ) -> Result<(), String> {
        {
            let mut downloading = REQUIREMENTS_DOWNLOADING.lock().unwrap();
            *downloading = true;
        }
        emit_to_main_window(app_handle, "requirements-status", true);

        for file in files {
            log_info!("Downloading requirement: {}", file);
            DATA.download(&file)
                .await
                .map_err(|e| format!("Failed {file}: {e}"))?;

            if IS_LINUX && file.starts_with(self.jdk_folder_name()) {
                self.fix_java_permissions();
            }
        }

        if let Some(jar) = extra_mc_jar {
            log_info!("Downloading MC Jar: {}", jar);
            DATA.download_to_folder(&jar, MINECRAFT_VERSIONS_FOLDER)
                .await?;
        }

        self.download_fabric_mods().await?;

        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

        {
            let mut downloading = REQUIREMENTS_DOWNLOADING.lock().unwrap();
            *downloading = false;
        }
        emit_to_main_window(app_handle, "requirements-status", false);

        Ok(())
    }

    fn fix_java_permissions(&self) {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let java_path = self.java_executable_path();
            if java_path.exists() {
                if let Ok(mut perms) = std::fs::metadata(&java_path).map(|m| m.permissions()) {
                    perms.set_mode(0o755);
                    let _ = std::fs::set_permissions(&java_path, perms);
                }
            }
        }
    }

    async fn ensure_fabric_libraries(&self) -> Result<(), String> {
        let common_dir = DATA.root_dir.join(LIBRARIES_FABRIC_FOLDER);
        if !dir_has_any_jars(&common_dir, true) {
            log_info!("Downloading common Fabric libraries");
            DATA.download(LIBRARIES_FABRIC_ZIP).await?;
        }

        let versioned_zip = format!(
            "{}/{}.zip",
            LIBRARIES_FABRIC_FOLDER,
            sanitize_version_for_paths(&self.version)
        );
        let versioned_dir = DATA.root_dir.join(&versioned_zip.replace(".zip", ""));

        if !dir_has_any_jars(&versioned_dir, false) {
            log_info!("Downloading versioned Fabric libraries: {}", versioned_zip);
            DATA.download(&versioned_zip).await?;
        }
        Ok(())
    }

    pub async fn ensure_java_available(&self, app_handle: &AppHandle) -> Result<(), String> {
        if self.java_executable_path().exists() {
            return Ok(());
        }

        log_warn!("Java executable missing. Redownloading requirements...");

        let _ = tokio::fs::remove_dir_all(DATA.root_dir.join(self.jdk_folder_name())).await;
        let _ = tokio::fs::remove_file(DATA.root_dir.join(self.jdk_zip_name())).await;

        self.download_requirements(app_handle).await?;

        if !self.java_executable_path().exists() {
            return Err("Java still missing after download".to_string());
        }
        Ok(())
    }

    fn build_classpath(&self) -> Result<String, String> {
        let (_, client_jar) = self.get_launch_paths()?;
        let agent_overlay = DATA.root_dir.join(AGENT_OVERLAY_FOLDER);

        let mut cp_parts = Vec::new();

        match self.client_type {
            ClientType::Fabric => {
                cp_parts.push(self.get_minecraft_jar_path());

                let v_libs = DATA
                    .root_dir
                    .join(LIBRARIES_FABRIC_FOLDER)
                    .join(sanitize_version_for_paths(&self.version));
                cp_parts.extend(collect_jars_recursive(&v_libs, false));

                let g_libs = DATA.root_dir.join(LIBRARIES_FABRIC_FOLDER);
                for jar in collect_jars_recursive(&g_libs, true) {
                    if !is_fabric_loader_jar(&jar) {
                        cp_parts.push(jar);
                    }
                }
            }
            ClientType::Forge => {
                cp_parts.push(self.get_minecraft_jar_path());
                let libs = DATA.root_dir.join(LIBRARIES_LEGACY_FOLDER);
                cp_parts.extend(collect_jars_recursive(&libs, false));
            }
            ClientType::Default => {
                let libs = if self.is_legacy_client() {
                    DATA.root_dir.join(LIBRARIES_LEGACY_FOLDER)
                } else {
                    DATA.root_dir.join(LIBRARIES_FOLDER)
                };

                return Ok(format!(
                    "{}{}*{}{}{}{}",
                    libs.display(),
                    MAIN_SEPARATOR,
                    PATH_SEPARATOR,
                    client_jar.display(),
                    PATH_SEPARATOR,
                    agent_overlay.display()
                ));
            }
        }

        cp_parts.push(agent_overlay);

        Ok(cp_parts
            .iter()
            .map(|p| p.display().to_string())
            .collect::<Vec<_>>()
            .join(PATH_SEPARATOR))
    }

    pub async fn run(
        self,
        options: LaunchOptions,
        manager: Arc<Mutex<ClientManager>>,
    ) -> Result<(), String> {
        if !options.is_custom && SETTINGS.lock().is_ok_and(|s| s.optional_telemetry.value) {
            Analytics::send_client_analytics(self.id);
        }

        {
            let mut logs = CLIENT_LOGS.lock().unwrap();
            logs.insert(self.id, Vec::new());
        }

        let app_handle = options.app_handle.clone();
        let client_id = self.id;

        if let Err(e) = self.download_requirements(&app_handle).await {
            self.emit_crash(&app_handle, &e);
            return Err(e);
        }

        if let Err(e) = self.ensure_java_available(&app_handle).await {
            self.emit_crash(&app_handle, &e);
            return Err(e);
        }

        let java_bin = self.java_executable_path();
        let (client_folder, _) = self.get_launch_paths()?;
        let assets_dir = if self.client_type == ClientType::Fabric {
            DATA.root_dir.join(ASSETS_FABRIC_FOLDER)
        } else {
            DATA.root_dir.join(ASSETS_FOLDER)
        };

        let natives_path = if self.is_legacy_client() {
            DATA.root_dir.join(format!(
                "{}{}",
                NATIVES_FOLDER,
                if IS_LINUX {
                    LINUX_SUFFIX
                } else {
                    LEGACY_SUFFIX
                }
            ))
        } else if self.meta.is_new {
            DATA.root_dir.join(format!(
                "{}{}",
                NATIVES_FOLDER,
                if IS_LINUX { LINUX_SUFFIX } else { "" }
            ))
        } else {
            DATA.root_dir.join(format!(
                "{}{}",
                NATIVES_FOLDER,
                if IS_LINUX {
                    LINUX_SUFFIX
                } else {
                    LEGACY_SUFFIX
                }
            ))
        };

        let classpath = self.build_classpath()?;

        let (analytics, irc, lang, ram_mb) = {
            let s = SETTINGS.lock().unwrap();
            (
                s.optional_telemetry.value,
                s.irc_chat.value,
                s.language.value.clone(),
                s.ram.value,
            )
        };

        let username = ACCOUNT_MANAGER
            .lock()
            .ok()
            .and_then(|m| m.get_active_account().map(|a| a.username.clone()))
            .unwrap_or_else(|| {
                let rnd = rand::random::<u32>() % 100_000;
                format!("Collapse{rnd:05}")
            });

        let agent_args = AgentArguments::new(
            options.user_token,
            self.name.clone(),
            if self.meta.is_custom {
                false
            } else {
                analytics
            },
            irc,
            lang,
        );

        let agent_overlay_path = DATA.root_dir.join(AGENT_OVERLAY_FOLDER);

        let mut cmd = Command::new(java_bin);

        #[cfg(windows)]
        cmd.creation_flags(0x0800_0000);

        cmd.current_dir(&client_folder);

        cmd.arg("-Xverify:none");

        let is_legacy_vanilla = self.client_type == ClientType::Default && !self.meta.is_new;
        if !IS_LINUX && self.client_type != ClientType::Forge && !is_legacy_vanilla {
            cmd.arg(format!(
                "-javaagent:{}={}",
                agent_overlay_path.join(AGENT_FILE).display(),
                agent_args.encode()
            ));
        }

        cmd.arg(format!("-Xmx{ram_mb}M"));

        if self.client_type != ClientType::Fabric {
            cmd.arg(format!(
                "-Djava.library.path={}{}{}",
                natives_path.display(),
                PATH_SEPARATOR,
                agent_overlay_path.display()
            ));
        } else {
            cmd.arg(format!(
                "-Djava.library.path={}",
                agent_overlay_path.display()
            ));
        }

        cmd.arg("-cp").arg(classpath).arg(&self.main_class);

        if self.client_type == ClientType::Forge {
            cmd.arg("--tweakClass")
                .arg("net.minecraftforge.fml.common.launcher.FMLTweaker");
        }

        cmd.arg("--username")
            .arg(&username)
            .arg("--gameDir")
            .arg(client_folder)
            .arg("--assetsDir")
            .arg(assets_dir)
            .arg("--assetIndex")
            .arg(&self.meta.asset_index)
            .arg("--uuid")
            .arg("N/A")
            .arg("--accessToken")
            .arg("0")
            .arg("--userType")
            .arg("legacy")
            .arg("--version")
            .arg(&self.version)
            .arg("--client")
            .arg(&self.filename);

        cmd.stdout(Stdio::piped()).stderr(Stdio::piped());

        log_debug!("Spawning client process: {}", self.name);

        let mut child = cmd
            .spawn()
            .map_err(|e| format!("Failed to spawn process: {e}"))?;
        let self_clone = self.clone();

        emit_to_main_window(
            &app_handle,
            "client-launched",
            serde_json::json!({
                "id": client_id,
                "name": self.name,
                "version": self.version
            }),
        );

        if let Some(stdout) = child.stdout.take() {
            let id = client_id;
            tokio::spawn(async move {
                let mut reader = BufReader::new(stdout).lines();
                while let Ok(Some(line)) = reader.next_line().await {
                    add_log_line(id, line);
                }
            });
        }

        if let Some(stderr) = child.stderr.take() {
            let id = client_id;
            tokio::spawn(async move {
                let mut reader = BufReader::new(stderr).lines();
                while let Ok(Some(line)) = reader.next_line().await {
                    add_log_line(id, line);
                }
            });
        }

        tokio::spawn(async move {
            match child.wait().await {
                Ok(status) => {
                    let msg = format!("Process finished with status: {status}");
                    log_info!("{}", msg);
                    add_log_line(client_id, msg);

                    let checker = LogChecker::new(self_clone.clone());
                    checker.check(&app_handle);

                    if let Ok(m) = manager.lock() {
                        let _ = m.update_status_on_client_exit(&app_handle);
                    }

                    emit_to_main_window(
                        &app_handle,
                        "client-exited",
                        serde_json::json!({
                            "id": client_id,
                            "name": self_clone.name,
                            "exitCode": status.code().unwrap_or(-1)
                        }),
                    );
                }
                Err(e) => {
                    let msg = format!("Error waiting for process: {e}");
                    log_error!("{}", msg);
                    add_log_line(client_id, msg.clone());
                    emit_to_main_window(
                        &app_handle,
                        "client-crashed",
                        serde_json::json!({
                            "id": client_id,
                            "name": self_clone.name,
                            "error": msg
                        }),
                    );
                }
            }
        });

        Ok(())
    }

    fn emit_crash(&self, app_handle: &AppHandle, error: &str) {
        emit_to_main_window(
            app_handle,
            "client-crashed",
            serde_json::json!({
                "id": self.id,
                "name": self.name,
                "error": error
            }),
        );
    }
}
