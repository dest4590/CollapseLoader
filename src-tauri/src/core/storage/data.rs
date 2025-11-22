use crate::core::clients::client::ClientType;
use crate::core::clients::manager::CLIENT_MANAGER;
use crate::core::network::servers::SERVERS;
use crate::core::storage::settings::SETTINGS;
use crate::core::utils::globals::{JDK_FOLDER, ROOT_DIR};
use crate::core::utils::helpers::emit_to_main_window;
use crate::{log_debug, log_error, log_info, log_warn};
use futures_util::StreamExt;
use std::path::{Path, PathBuf, MAIN_SEPARATOR};
use std::sync::Mutex;
use std::time::Duration;
use std::{fs, io};
use tokio::io::AsyncWriteExt;

pub struct Data {
    pub root_dir: PathBuf,
}

pub static APP_HANDLE: std::sync::LazyLock<Mutex<Option<tauri::AppHandle>>> =
    std::sync::LazyLock::new(|| Mutex::new(None));

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

        if let Some(app_handle) = APP_HANDLE.lock().unwrap().as_ref() {
            emit_to_main_window(app_handle, "unzip-start", &emit_name);
        }

        let zip_path = self.get_local(&local_name);
        let unzip_path = self.get_local(local_name.trim_end_matches(".zip"));

        if unzip_path.exists() {
            log_debug!(
                "Directory {} exists, will overwrite contents",
                unzip_path.display()
            );
        } else {
            log_debug!("Creating unzip directory: {}", unzip_path.display());
            fs::create_dir_all(&unzip_path).map_err(|e| e.to_string())?;
        }

        if !zip_path.exists() {
            log_error!(
                "Zip file not found at expected path: {}",
                zip_path.display()
            );
        } else {
            match fs::metadata(&zip_path) {
                Ok(_) => {}
                Err(e) => log_warn!("Failed to read metadata for {}: {}", zip_path.display(), e),
            }
        }

        let mut archive = zip::ZipArchive::new(fs::File::open(&zip_path).map_err(|e| {
            log_error!("Failed to open zip file {}: {}", zip_path.display(), e);
            e.to_string()
        })?)
        .map_err(|e| {
            log_error!("Failed to read zip archive {}: {}", zip_path.display(), e);
            e.to_string()
        })?;

        let total_files = archive.len() as u64;

        let mut files_extracted: u64 = 0;
        let mut last_percentage: u8 = 0;

        for i in 0..archive.len() {
            let mut file_entry = archive.by_index(i).map_err(|e| e.to_string())?;
            let outpath = unzip_path.join(file_entry.mangled_name());

            if file_entry.name().ends_with('/') {
                fs::create_dir_all(&outpath).map_err(|e| e.to_string())?;
            } else {
                if let Some(parent) = outpath.parent() {
                    if !parent.exists() {
                        log_debug!(
                            "Creating parent dir for {} -> {}",
                            file_entry.name(),
                            parent.display()
                        );
                        fs::create_dir_all(parent).map_err(|e| {
                            log_error!("Failed to create parent dir {}: {}", parent.display(), e);
                            e.to_string()
                        })?;
                    }
                }
                let mut outfile = fs::File::create(&outpath).map_err(|e| {
                    log_error!("Failed to create output file {}: {}", outpath.display(), e);
                    e.to_string()
                })?;
                io::copy(&mut file_entry, &mut outfile).map_err(|e| {
                    log_error!(
                        "Failed to write extracted file {}: {}",
                        outpath.display(),
                        e
                    );
                    e.to_string()
                })?;
            }

            files_extracted += 1;

            let percentage = ((files_extracted as f64 / total_files as f64) * 100.0) as u8;
            if percentage != last_percentage {
                last_percentage = percentage;

                if let Some(app_handle) = APP_HANDLE.lock().unwrap().as_ref() {
                    let progress_data = serde_json::json!({
                        "file": emit_name,
                        "percentage": percentage,
                        "action": "extracting",
                        "files_extracted": files_extracted,
                        "total_files": total_files
                    });
                    emit_to_main_window(app_handle, "unzip-progress", progress_data);
                }
            }
        }

        if let Err(e) = fs::remove_file(&zip_path) {
            log_debug!("Failed to delete zip file {}: {}", zip_path.display(), e);
        }

        if let Some(app_handle) = APP_HANDLE.lock().unwrap().as_ref() {
            emit_to_main_window(app_handle, "unzip-complete", &emit_name);
        }

        Ok(())
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

    pub async fn download(&self, file: &str) -> Result<(), String> {
        let is_url = file.starts_with("http://") || file.starts_with("https://");
        let local_file = if is_url {
            file.rsplit('/').next().unwrap_or(file)
        } else {
            file
        };

        let file_name = Self::get_filename(local_file);
        let is_fabric_client = local_file.starts_with("fabric/") && local_file.ends_with(".jar");

        let file_exists = if Self::has_extension(local_file, "zip") {
            let zip_path = self.root_dir.join(local_file);
            zip_path.exists()
        } else if Self::has_extension(local_file, "jar") {
            if is_fabric_client {
                let jar_basename = Path::new(local_file)
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or(local_file);
                let jar_path = self
                    .root_dir
                    .join(&file_name)
                    .join("mods")
                    .join(jar_basename);
                jar_path.exists()
            } else {
                let jar_path = self.get_local(&format!("{file_name}{MAIN_SEPARATOR}{local_file}"));
                jar_path.exists()
            }
        } else {
            false
        };

        if file_exists {
            log_debug!("File {} already exists, skipping download", file);
            return Ok(());
        }

        log_debug!("Starting download for file: {}", file);

        if let Some(app_handle) = APP_HANDLE.lock().unwrap().as_ref() {
            emit_to_main_window(app_handle, "download-start", &file);
        }

        if Self::has_extension(local_file, "jar") {
            if is_fabric_client {
                let mods_dir = self.root_dir.join(&file_name).join("mods");
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
                let local_path = self.get_as_folder(local_file);
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
                    if let Err(e) = self.ensure_client_synced(&file_name) {
                        log_warn!("Failed to ensure client sync for {}: {}", file_name, e);
                    }
                }
            }
        }

        let cdn_url = SERVERS
            .selected_cdn
            .read()
            .unwrap()
            .as_ref()
            .map_or_else(
                || {
                    log_error!("No CDN server available for download");
                    Err("No CDN server available for download.".to_string())
                },
                |server| Ok(server.url.clone()),
            )?;

        let download_url = if is_url {
            file.to_string()
        } else {
            format!("{cdn_url}{file}")
        };

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(600))
            .build()
            .map_err(|e| {
                log_error!("Failed to create HTTP client: {}", e);
                format!("Failed to create HTTP client: {e}")
            })?;

        let response = client.get(&download_url).send().await.map_err(|e| {
            log_error!("Failed to make HTTP request to {}: {}", download_url, e);
            format!("Failed to download file {file}: {e}")
        })?;

        if !response.status().is_success() {
            let error_msg = format!(
                "Failed to download file {}: HTTP {} - {}",
                file,
                response.status().as_u16(),
                response
                    .status()
                    .canonical_reason()
                    .unwrap_or("Unknown error")
            );
            log_error!("{}", error_msg);
            return Err(error_msg);
        }

        let total_size = response.content_length();
        let dest_path = if is_fabric_client {
            let jar_basename = Path::new(file)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or(file);
            self.root_dir
                .join(&file_name)
                .join("mods")
                .join(jar_basename)
        } else if Self::has_extension(local_file, "jar") {
            self.root_dir.join(format!("{file_name}/{local_file}"))
        } else {
            self.root_dir.join(local_file)
        };

        let mut dest = tokio::fs::File::create(&dest_path).await.map_err(|e| {
            log_error!(
                "Failed to create destination file {}: {}",
                dest_path.display(),
                e
            );
            format!("Failed to create file: {e}")
        })?;

        log_info!("Created destination file: {}", dest_path.display());

        let mut downloaded: u64 = 0;
        let mut last_percentage: u8 = 0;
        let mut stream = response.bytes_stream();

        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|e| {
                log_error!("Failed to read response data for {}: {}", file, e);
                format!("Network read error: {e}")
            })?;

            dest.write_all(&chunk).await.map_err(|e| {
                log_error!(
                    "Failed to write data to file {}: {}",
                    dest_path.display(),
                    e
                );
                format!("File write error: {e}")
            })?;

            downloaded += chunk.len() as u64;

            let percentage = total_size.map_or_else(
                || std::cmp::min(99, (downloaded / 1024 / 1024) as u8),
                |total| ((downloaded as f64 / total as f64) * 100.0) as u8,
            );

            if percentage != last_percentage {
                last_percentage = percentage;
                if let Some(app_handle) = APP_HANDLE.lock().unwrap().as_ref() {
                    let progress_data = serde_json::json!({
                        "file": file,
                        "percentage": percentage,
                        "downloaded": downloaded,
                        "total": total_size.unwrap_or(0),
                        "action": "downloading"
                    });
                    emit_to_main_window(app_handle, "download-progress", progress_data);
                }
            }
        }

        dest.flush().await.map_err(|e| {
            log_error!("Failed to flush file {}: {}", dest_path.display(), e);
            format!("File flush error: {e}")
        })?;

        if let Some(app_handle) = APP_HANDLE.lock().unwrap().as_ref() {
            emit_to_main_window(app_handle, "download-complete", &file);
        }

        if Self::has_extension(local_file, "zip") {
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

        if Self::has_extension(local_file, "jar") {
            log_debug!("Verifying MD5 hash for client file: {}", file);
            self.verify_client_hash(file, &dest_path).map_err(|e| {
                log_error!("Failed to verify client hash for {}: {}", file, e);
                e
            })?;
        }

        Ok(())
    }

    pub fn ensure_client_synced(&self, client_base: &str) -> Result<(), String> {
        let folders_to_sync = ["resourcepacks"];
        let files_to_sync = ["options.txt", "optionsof.txt"];

        let global_options_dir = self.root_dir.join("synced_options");
        if !global_options_dir.exists() {
            if let Err(e) = fs::create_dir_all(&global_options_dir) {
                return Err(format!("Failed to create global options dir: {e}"));
            }
        }

        let client_dir = self.root_dir.join(client_base);
        if !client_dir.exists() {
            if let Err(e) = fs::create_dir_all(&client_dir) {
                return Err(format!("Failed to create client dir: {e}"));
            }
        }

        for folder in &folders_to_sync {
            let target = global_options_dir.join(folder);
            if !target.exists() {
                if let Err(e) = fs::create_dir_all(&target) {
                    log_warn!(
                        "Failed to create global {} dir: {}: {}",
                        folder,
                        target.display(),
                        e
                    );
                    continue;
                }
            }

            let client_target = client_dir.join(folder);

            if client_target.exists() {
                if let Err(e) = fs::remove_dir_all(&client_target) {
                    log_warn!(
                        "Failed to remove existing client {} at {}: {}",
                        folder,
                        client_target.display(),
                        e
                    );
                }
            }

            if let Err(e) = Self::create_symlink(&target, &client_target) {
                log_warn!(
                    "Failed to symlink {} for {}: {} -> {}: {}",
                    folder,
                    client_base,
                    target.display(),
                    client_target.display(),
                    e
                );
            }
        }

        for file in &files_to_sync {
            let global_file = global_options_dir.join(file);
            if !global_file.exists() {
                if let Err(e) = fs::write(&global_file, "") {
                    log_warn!(
                        "Failed to create global {} file at {}: {}",
                        file,
                        global_file.display(),
                        e
                    );
                    continue;
                }
            }

            let client_file = client_dir.join(file);

            if client_file.exists() {
                if let Err(e) = fs::remove_file(&client_file) {
                    log_warn!(
                        "Failed to remove existing client {} at {}: {}",
                        file,
                        client_file.display(),
                        e
                    );
                }
            }

            if let Err(e) = Self::create_symlink(&global_file, &client_file) {
                log_warn!(
                    "Failed to symlink {} for {}: {} -> {}: {}",
                    file,
                    client_base,
                    global_file.display(),
                    client_file.display(),
                    e
                );
            }
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

        let is_url = file.starts_with("http://") || file.starts_with("https://");
        let cdn_url = SERVERS
            .selected_cdn
            .read()
            .unwrap()
            .as_ref()
            .map_or_else(
                || {
                    log_error!("No CDN server available for download");
                    Err("No CDN server available for download.".to_string())
                },
                |server| Ok(server.url.clone()),
            )?;

        let download_url = if is_url {
            file.to_string()
        } else {
            format!("{cdn_url}{file}")
        };

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(600))
            .build()
            .map_err(|e| {
                log_error!("Failed to create HTTP client: {}", e);
                format!("Failed to create HTTP client: {e}")
            })?;

        let response = client.get(&download_url).send().await.map_err(|e| {
            log_error!("Failed to make HTTP request to {}: {}", download_url, e);
            format!("Failed to download file {file}: {e}")
        })?;

        if !response.status().is_success() {
            let error_msg = format!(
                "Failed to download file {}: HTTP {} - {}",
                file,
                response.status().as_u16(),
                response
                    .status()
                    .canonical_reason()
                    .unwrap_or("Unknown error")
            );
            log_error!("{}", error_msg);
            return Err(error_msg);
        }

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

        let mut dest = tokio::fs::File::create(&dest_path).await.map_err(|e| {
            log_error!(
                "Failed to create destination file {}: {}",
                dest_path.display(),
                e
            );
            format!("Failed to create file: {e}")
        })?;

        log_info!(
            "Created destination file for folder download: {}",
            dest_path.display()
        );

        let total_size = response.content_length();
        let mut downloaded: u64 = 0;
        let mut last_percentage: u8 = 0;
        let mut stream = response.bytes_stream();

        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|e| {
                log_error!("Failed to read response data for {}: {}", file, e);
                format!("Network read error: {e}")
            })?;

            dest.write_all(&chunk).await.map_err(|e| {
                log_error!(
                    "Failed to write data to file {}: {}",
                    dest_path.display(),
                    e
                );
                format!("File write error: {e}")
            })?;

            downloaded += chunk.len() as u64;

            let percentage = total_size.map_or_else(
                || std::cmp::min(99, (downloaded / 1024 / 1024) as u8),
                |total| ((downloaded as f64 / total as f64) * 100.0) as u8,
            );

            if percentage != last_percentage {
                last_percentage = percentage;
                if let Some(app_handle) = APP_HANDLE.lock().unwrap().as_ref() {
                    let progress_data = serde_json::json!({
                        "file": file,
                        "percentage": percentage,
                        "downloaded": downloaded,
                        "total": total_size.unwrap_or(0),
                        "action": "downloading"
                    });
                    emit_to_main_window(app_handle, "download-progress", progress_data);
                }
            }
        }

        dest.flush().await.map_err(|e| {
            log_error!("Failed to flush file {}: {}", dest_path.display(), e);
            format!("File flush error: {e}")
        })?;

        if let Some(app_handle) = APP_HANDLE.lock().unwrap().as_ref() {
            emit_to_main_window(app_handle, "download-complete", &file);
        }

        Ok(())
    }

    fn verify_client_hash(&self, filename: &str, file_path: &PathBuf) -> Result<(), String> {
        let hash_verify_enabled = {
            let settings = SETTINGS
                .lock()
                .map_err(|_| "Failed to access settings".to_string())?;
            settings.hash_verify.value
        };

        let (expected_hash, client_id, client_name, is_fabric) = {
            CLIENT_MANAGER
                .lock()
                .map_err(|_| "Failed to acquire lock on client manager".to_string())?
                .as_ref()
                .ok_or_else(|| "Client manager not initialized".to_string())?
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
                .ok_or_else(|| format!("Client not found for filename: {filename}"))?
        };

        if !hash_verify_enabled {
            log_info!(
                "Hash verification is disabled, skipping check for {}",
                filename
            );
            return Ok(());
        }

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
            Self::calculate_md5_hash(&fabric_jar_path)?
        } else {
            Self::calculate_md5_hash(file_path)?
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

    pub fn calculate_md5_hash(path: &PathBuf) -> Result<String, String> {
        let bytes = fs::read(path).map_err(|e| format!("Failed to read file for hashing: {e}"))?;

        let digest = md5::compute(&bytes);
        Ok(format!("{digest:x}"))
    }

    pub fn reset_requirements(&self) -> Result<(), Box<dyn std::error::Error>> {
        let requirements: Vec<String> = vec![
            JDK_FOLDER.to_string(),
            format!("{JDK_FOLDER}.zip"),
            "assets".to_string(),
            "assets.zip".to_string(),
            "natives".to_string(),
            "natives.zip".to_string(),
            "libraries".to_string(),
            "libraries.zip".to_string(),
            "natives-1.12".to_string(),
            "natives-1.12.zip".to_string(),
            "libraries-1.12".to_string(),
            "libraries-1.12.zip".to_string(),
            "assets_fabric".to_string(),
            "assets_fabric.zip".to_string(),
            "libraries_fabric".to_string(),
            "libraries_fabric.zip".to_string(),
            "natives_fabric".to_string(),
            "natives_fabric.zip".to_string(),
            "minecraft_versions".to_string(),
        ];

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
