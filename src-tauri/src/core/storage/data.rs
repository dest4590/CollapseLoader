use crate::core::clients::client::ClientType;
use crate::core::clients::manager::CLIENT_MANAGER;
use crate::core::network::downloader::download_file;
use crate::core::network::servers::SERVERS;
use crate::core::storage::settings::SETTINGS;
use crate::core::utils::archive::unzip;
use crate::core::utils::globals::{JDK_FOLDER, ROOT_DIR};
use crate::core::utils::hashing::calculate_md5_hash;
use crate::core::utils::helpers::emit_to_main_window;
use crate::{log_debug, log_error, log_info, log_warn};
use std::fs;
use std::path::{Path, PathBuf, MAIN_SEPARATOR};
use std::sync::Mutex;

pub struct Data {
    pub root_dir: PathBuf,
}

pub static APP_HANDLE: std::sync::LazyLock<Mutex<Option<tauri::AppHandle>>> =
    std::sync::LazyLock::new(|| Mutex::new(None));

struct FileInfo {
    local_file: String,
    file_name: String,
    is_fabric_client: bool,
}

impl Data {
    pub fn new(root_dir: PathBuf) -> Self {
        if !root_dir.exists() {
            log_debug!(
                "Root data directory does not exist, creating: {}",
                root_dir.display()
            );
            fs::create_dir_all(&root_dir).expect("Failed to create root directory");
            log_info!("Created root data directory: {}", root_dir.display());
        }

        Self { root_dir }
    }

    pub fn has_extension(file_path: &str, extension: &str) -> bool {
        Path::new(file_path)
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case(extension))
    }

    pub fn get_local(&self, relative_path: &str) -> PathBuf {
        self.root_dir.join(relative_path)
    }

    pub fn unzip(&self, file: &str) -> Result<(), String> {
        let (emit_name, local_name) = if file.starts_with("http://") || file.starts_with("https://")
        {
            (
                file.to_string(),
                file.rsplit('/').next().unwrap_or(file).to_string(),
            )
        } else {
            (file.to_string(), file.to_string())
        };

        let zip_path = self.get_local(&local_name);
        let unzip_path = self.get_local(local_name.trim_end_matches(".zip"));

        let app_handle = APP_HANDLE.lock().unwrap();
        unzip(&zip_path, &unzip_path, &emit_name, app_handle.as_ref())
    }

    pub fn get_as_folder(&self, file: &str) -> PathBuf {
        let file_name = Path::new(file).file_stem().unwrap().to_str().unwrap();
        self.root_dir.join(file_name)
    }

    pub fn get_as_folder_string(file: &str) -> String {
        let file_name = Path::new(file).file_stem().unwrap().to_str().unwrap();
        format!("{file_name}{MAIN_SEPARATOR}")
    }

    pub fn get_filename(file: &str) -> String {
        let file_name = Path::new(file).file_stem().unwrap().to_str().unwrap();
        file_name.to_string()
    }

    fn resolve_local_file_info(file: &str) -> FileInfo {
        let is_url = file.starts_with("http://") || file.starts_with("https://");
        let local_file = if is_url {
            file.rsplit('/').next().unwrap_or(file).to_string()
        } else {
            file.to_string()
        };

        let file_name = Self::get_filename(&local_file);
        let is_fabric_client = local_file.starts_with("fabric/") && local_file.ends_with(".jar");

        FileInfo {
            local_file,
            file_name,
            is_fabric_client,
        }
    }

    fn get_destination_path(&self, file: &str, info: &FileInfo) -> PathBuf {
        if info.is_fabric_client {
            let jar_basename = Path::new(file)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or(file);
            self.root_dir
                .join(&info.file_name)
                .join("mods")
                .join(jar_basename)
        } else if Self::has_extension(&info.local_file, "jar") {
            self.root_dir
                .join(format!("{}/{}", info.file_name, info.local_file))
        } else {
            self.root_dir.join(&info.local_file)
        }
    }

    fn should_skip_download(&self, info: &FileInfo) -> bool {
        if Self::has_extension(&info.local_file, "zip") {
            let zip_path = self.root_dir.join(&info.local_file);
            zip_path.exists()
        } else if Self::has_extension(&info.local_file, "jar") {
            if info.is_fabric_client {
                let jar_basename = Path::new(&info.local_file)
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or(&info.local_file);
                let jar_path = self
                    .root_dir
                    .join(&info.file_name)
                    .join("mods")
                    .join(jar_basename);
                jar_path.exists()
            } else {
                let jar_path = self.get_local(&format!(
                    "{}{MAIN_SEPARATOR}{}",
                    info.file_name, info.local_file
                ));
                jar_path.exists()
            }
        } else {
            false
        }
    }

    fn prepare_download_dirs(&self, info: &FileInfo) -> Result<(), String> {
        if Self::has_extension(&info.local_file, "jar") {
            if info.is_fabric_client {
                let mods_dir = self.root_dir.join(&info.file_name).join("mods");
                if let Err(e) = fs::create_dir_all(&mods_dir) {
                    log_error!(
                        "Failed to create fabric mods directory {}: {}",
                        mods_dir.display(),
                        e
                    );
                    return Err(format!("Failed to create mods directory: {e}"));
                }
                log_debug!("Created fabric mods directory: {}", mods_dir.display());
            } else {
                let local_path = self.get_as_folder(&info.local_file);
                if let Err(e) = fs::create_dir_all(&local_path) {
                    log_error!("Failed to create directory {}: {}", local_path.display(), e);
                    return Err(format!("Failed to create directory: {e}"));
                }
                log_debug!("Created client local directory: {}", local_path.display());
                if SETTINGS
                    .lock()
                    .map(|s| s.sync_client_settings.value)
                    .unwrap_or(false)
                {
                    if let Err(e) = self.ensure_client_synced(&info.file_name) {
                        log_warn!("Failed to ensure client sync for {}: {}", info.file_name, e);
                    }
                }
            }
        }
        Ok(())
    }

    fn get_download_url(file: &str) -> Result<String, String> {
        let is_url = file.starts_with("http://") || file.starts_with("https://");
        if is_url {
            Ok(file.to_string())
        } else {
            let cdn_url = SERVERS.selected_cdn.read().unwrap().as_ref().map_or_else(
                || {
                    log_error!("No CDN server available for download");
                    Err("No CDN server available for download.".to_string())
                },
                |server| Ok(server.url.clone()),
            )?;
            Ok(format!("{cdn_url}{file}"))
        }
    }

    pub async fn download(&self, file: &str) -> Result<(), String> {
        let info = Self::resolve_local_file_info(file);

        if self.should_skip_download(&info) {
            log_debug!("File {} already exists, skipping download", file);
            return Ok(());
        }

        log_debug!("Starting download for file: {}", file);

        if let Some(app_handle) = APP_HANDLE.lock().unwrap().as_ref() {
            emit_to_main_window(app_handle, "download-start", &file);
        }

        self.prepare_download_dirs(&info)?;

        let download_url = Self::get_download_url(file)?;
        let dest_path = self.get_destination_path(file, &info);

        let app_handle = APP_HANDLE.lock().unwrap().clone();
        download_file(&download_url, &dest_path, file, app_handle.as_ref()).await?;

        if let Some(handle) = app_handle.as_ref() {
            emit_to_main_window(handle, "download-complete", &file);
        }

        if Self::has_extension(&info.local_file, "zip") {
            self.unzip(file).map_err(|e| {
                log_error!("Failed to extract {}: {}", file, e);
                if let Some(app_handle) = APP_HANDLE.lock().unwrap().as_ref() {
                    let err_data = serde_json::json!({
                        "file": file,
                        "error": e
                    });
                    emit_to_main_window(app_handle, "unzip-error", err_data);
                }
                e
            })?;
        }

        if Self::has_extension(&info.local_file, "jar") {
            log_debug!("Verifying MD5 hash for client file: {}", file);
            self.verify_client_hash(file, &dest_path).map_err(|e| {
                log_error!("Failed to verify client hash for {}: {}", file, e);
                e
            })?;
        }

        Ok(())
    }

    pub fn ensure_client_synced(&self, client_base: &str) -> Result<(), String> {
        let global_options_dir = self.root_dir.join("synced_options");
        if !global_options_dir.exists() {
            fs::create_dir_all(&global_options_dir)
                .map_err(|e| format!("Failed to create global options dir: {e}"))?;
        }

        let client_dir = self.root_dir.join(client_base);
        if !client_dir.exists() {
            fs::create_dir_all(&client_dir)
                .map_err(|e| format!("Failed to create client dir: {e}"))?;
        }

        let sync_item = |name: &str, is_dir: bool| -> Result<(), String> {
            let target = global_options_dir.join(name);
            if !target.exists() {
                if is_dir {
                    fs::create_dir_all(&target).map_err(|e| {
                        format!(
                            "Failed to create global {} dir: {}: {}",
                            name,
                            target.display(),
                            e
                        )
                    })?;
                } else {
                    fs::write(&target, "").map_err(|e| {
                        format!(
                            "Failed to create global {} file at {}: {}",
                            name,
                            target.display(),
                            e
                        )
                    })?;
                }
            }

            let client_target = client_dir.join(name);
            if client_target.exists() {
                let res = if client_target.is_dir() {
                    fs::remove_dir_all(&client_target)
                } else {
                    fs::remove_file(&client_target)
                };
                if let Err(e) = res {
                    log_warn!(
                        "Failed to remove existing client {} at {}: {}",
                        name,
                        client_target.display(),
                        e
                    );
                }
            }

            if let Err(e) = Self::create_symlink(&target, &client_target) {
                log_warn!(
                    "Failed to symlink {} for {}: {} -> {}: {}",
                    name,
                    client_base,
                    target.display(),
                    client_target.display(),
                    e
                );
            }
            Ok(())
        };

        for folder in &["resourcepacks"] {
            let _ = sync_item(folder, true);
        }

        for file in &["options.txt", "optionsof.txt"] {
            let _ = sync_item(file, false);
        }

        Ok(())
    }

    fn create_symlink(src: &std::path::Path, dst: &std::path::Path) -> Result<(), String> {
        #[cfg(target_family = "unix")]
        {
            std::os::unix::fs::symlink(src, dst).map_err(|e| e.to_string())
        }

        #[cfg(target_family = "windows")]
        {
            use std::os::windows::fs::{symlink_dir, symlink_file};
            if src.is_dir() {
                symlink_dir(src, dst).map_err(|e| e.to_string())
            } else if src.is_file() {
                symlink_file(src, dst).map_err(|e| e.to_string())
            } else {
                symlink_file(src, dst).map_err(|e| e.to_string())
            }
        }
    }

    pub async fn download_to_folder(&self, file: &str, dest_folder: &str) -> Result<(), String> {
        if let Some(app_handle) = APP_HANDLE.lock().unwrap().as_ref() {
            emit_to_main_window(app_handle, "download-start", &file);
        }

        let download_url = Self::get_download_url(file)?;

        let dest_dir = self.root_dir.join(dest_folder);
        if let Err(e) = std::fs::create_dir_all(&dest_dir) {
            log_error!(
                "Failed to create destination folder {}: {}",
                dest_dir.display(),
                e
            );
            return Err(format!("Failed to create destination folder: {e}"));
        }

        let dest_filename = file.rsplit('/').next().unwrap_or(file);
        let dest_path = dest_dir.join(dest_filename);

        let app_handle = APP_HANDLE.lock().unwrap().clone();
        download_file(&download_url, &dest_path, file, app_handle.as_ref()).await?;

        if let Some(handle) = app_handle.as_ref() {
            emit_to_main_window(handle, "download-complete", &file);
        }

        Ok(())
    }

    fn verify_client_hash(&self, filename: &str, file_path: &PathBuf) -> Result<(), String> {
        let hash_verify_enabled = SETTINGS
            .lock()
            .map_err(|_| "Failed to access settings".to_string())?
            .hash_verify
            .value;

        if !hash_verify_enabled {
            log_info!(
                "Hash verification is disabled, skipping check for {}",
                filename
            );
            return Ok(());
        }

        let client_info = {
            let manager_guard = CLIENT_MANAGER
                .lock()
                .map_err(|_| "Failed to acquire lock on client manager".to_string())?;
            let manager = manager_guard
                .as_ref()
                .ok_or_else(|| "Client manager not initialized".to_string())?;
            manager
                .clients
                .iter()
                .find(|c| c.filename == filename)
                .map(|c| {
                    (
                        c.md5_hash.clone(),
                        c.id,
                        c.name.clone(),
                        c.client_type == ClientType::Fabric,
                    )
                })
        };

        let (expected_hash, client_id, client_name, is_fabric) =
            client_info.ok_or_else(|| format!("Client not found for filename: {filename}"))?;

        if let Some(app_handle) = APP_HANDLE.lock().unwrap().as_ref() {
            emit_to_main_window(
                app_handle,
                "client-hash-verification-start",
                &serde_json::json!({
                    "id": client_id,
                    "name": client_name
                }),
            );
        }

        log_info!(
            "Verifying MD5 hash for downloaded client file: {}",
            filename
        );
        let calculated_hash = if is_fabric {
            let client_folder = self.root_dir.join(Self::get_filename(filename));
            let jar_basename = std::path::Path::new(filename)
                .file_name()
                .and_then(|n| n.to_str())
                .ok_or_else(|| "Invalid fabric client filename".to_string())?;
            let fabric_jar_path = client_folder.join("mods").join(jar_basename);
            calculate_md5_hash(&fabric_jar_path)?
        } else {
            calculate_md5_hash(file_path)?
        };

        if calculated_hash != expected_hash {
            if let Err(e) = fs::remove_file(file_path) {
                log_warn!("Failed to remove corrupted file {}: {}", filename, e);
            }

            if let Ok(mut manager) = CLIENT_MANAGER.lock() {
                if let Some(manager) = manager.as_mut() {
                    if let Some(client) = manager.clients.iter_mut().find(|c| c.id == client_id) {
                        client.meta.installed = false;
                    }
                }
            }

            if let Some(app_handle) = APP_HANDLE.lock().unwrap().as_ref() {
                emit_to_main_window(
                    app_handle,
                    "client-hash-verification-failed",
                    &serde_json::json!({
                        "id": client_id,
                        "name": client_name,
                        "expected_hash": expected_hash,
                        "actual_hash": calculated_hash
                    }),
                );
            }

            return Err(format!(
                "Hash verification failed for {filename}. Expected: {expected_hash}, Got: {calculated_hash}. The file has been removed and needs to be redownloaded."
            ));
        }

        log_info!("MD5 hash verification successful for {}", filename);

        if let Some(app_handle) = APP_HANDLE.lock().unwrap().as_ref() {
            emit_to_main_window(
                app_handle,
                "client-hash-verification-done",
                &serde_json::json!({
                    "id": client_id,
                    "name": client_name
                }),
            );
        }

        Ok(())
    }

    pub fn reset_requirements(&self) -> Result<(), Box<dyn std::error::Error>> {
        let base_requirements = [
            JDK_FOLDER,
            "assets",
            "natives",
            "libraries",
            "natives-1.12",
            "libraries-1.12",
            "assets_fabric",
            "libraries_fabric",
            "natives_fabric",
        ];

        let mut requirements = Vec::new();
        for req in base_requirements {
            requirements.push(req.to_string());
            requirements.push(format!("{}.zip", req));
        }
        requirements.push("minecraft_versions".to_string());

        for requirement in &requirements {
            let path = self.root_dir.join(requirement);
            if path.exists() {
                if path.is_dir() {
                    fs::remove_dir_all(&path)?;
                } else {
                    fs::remove_file(&path)?;
                }
            }
        }

        Ok(())
    }

    pub fn reset_cache(&self) -> Result<(), Box<dyn std::error::Error>> {
        let cache_dir = self.root_dir.join("cache");
        if cache_dir.exists() {
            fs::remove_dir_all(&cache_dir)?;
        }
        Ok(())
    }
}

pub static DATA: std::sync::LazyLock<Data> =
    std::sync::LazyLock::new(|| Data::new(ROOT_DIR.clone().into()));
