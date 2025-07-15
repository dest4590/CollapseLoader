use crate::api::clients::clients::CLIENT_MANAGER;
use crate::api::core::settings::SETTINGS;
use crate::api::globals::ROOT_DIR;
use crate::api::network::servers::SERVERS;
use crate::api::utils;
use crate::{log_debug, log_error, log_info, log_warn};
use lazy_static::lazy_static;
use std::fs;
use std::path::{Path, PathBuf, MAIN_SEPARATOR};
use std::sync::Mutex;

pub struct DataManager {
    pub root_dir: PathBuf,
}

lazy_static! {
    pub static ref APP_HANDLE: Mutex<Option<tauri::AppHandle>> = Mutex::new(None);
}

impl DataManager {
    pub fn new(root_dir: PathBuf) -> Self {
        if !root_dir.exists() {
            std::fs::create_dir_all(&root_dir).expect("Failed to create root directory");
        }

        Self { root_dir }
    }

    pub fn get_local(&self, relative_path: &str) -> PathBuf {
        self.root_dir.join(relative_path)
    }

    pub fn unzip(&self, file: &str) -> Result<(), String> {
        if let Some(app_handle) = APP_HANDLE.lock().unwrap().as_ref() {
            utils::emit_to_main_window(app_handle, "unzip-start", &file);
        }

        let zip_path = self.get_local(file);
        let unzip_path = self.get_local(file.trim_end_matches(".zip"));

        if unzip_path.exists() {
            log_debug!(
                "Directory {} exists, will overwrite contents",
                unzip_path.display()
            );
        } else {
            std::fs::create_dir_all(&unzip_path).map_err(|e| e.to_string())?;
        }

        let mut archive =
            zip::ZipArchive::new(std::fs::File::open(&zip_path).map_err(|e| e.to_string())?)
                .map_err(|e| e.to_string())?;

        let total_files = archive.len() as u64;

        let mut files_extracted: u64 = 0;
        let mut last_percentage: u8 = 0;

        for i in 0..archive.len() {
            let mut file_entry = archive.by_index(i).map_err(|e| e.to_string())?;
            let outpath = unzip_path.join(file_entry.mangled_name());

            if file_entry.name().ends_with('/') {
                std::fs::create_dir_all(&outpath).map_err(|e| e.to_string())?;
            } else {
                if let Some(parent) = outpath.parent() {
                    std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
                }
                let mut outfile = std::fs::File::create(&outpath).map_err(|e| e.to_string())?;
                std::io::copy(&mut file_entry, &mut outfile).map_err(|e| e.to_string())?;
            }

            files_extracted += 1;

            let percentage = ((files_extracted as f64 / total_files as f64) * 100.0) as u8;
            if percentage != last_percentage {
                last_percentage = percentage;

                if let Some(app_handle) = APP_HANDLE.lock().unwrap().as_ref() {
                    let progress_data = serde_json::json!({
                        "file": file,
                        "percentage": percentage,
                        "action": "extracting",
                        "files_extracted": files_extracted,
                        "total_files": total_files
                    });
                    utils::emit_to_main_window(app_handle, "unzip-progress", progress_data);
                }
            }
        }

        if let Err(e) = std::fs::remove_file(&zip_path) {
            log_debug!("Failed to delete zip file {}: {}", zip_path.display(), e);
        }

        if let Some(app_handle) = APP_HANDLE.lock().unwrap().as_ref() {
            utils::emit_to_main_window(app_handle, "unzip-complete", &file);
        }

        Ok(())
    }

    pub fn get_as_folder(&self, file: &str) -> PathBuf {
        let file_name = Path::new(file).file_stem().unwrap().to_str().unwrap();
        self.root_dir.join(file_name)
    }

    pub fn get_as_folder_string(&self, file: &str) -> String {
        let file_name = Path::new(file).file_stem().unwrap().to_str().unwrap();
        format!("{}{}", file_name, std::path::MAIN_SEPARATOR)
    }

    pub fn get_filename(&self, file: &str) -> String {
        let file_name = Path::new(file).file_stem().unwrap().to_str().unwrap();
        file_name.to_string()
    }

    pub async fn download(&self, file: &str) -> Result<(), String> {
        let file_name = self.get_filename(file);

        let is_essential_requirement = file == "jdk-21.0.2.zip"
            || file == "assets.zip"
            || file == "natives.zip"
            || file == "libraries.zip"
            || file == "natives-1.12.zip"
            || file == "libraries-1.12.zip";

        let file_exists = if file.ends_with(".zip") {
            let extract_path = self.root_dir.join(&file_name);
            extract_path.exists()
        } else if file.ends_with(".jar") {
            let jar_path = self.get_local(&format!("{file_name}{MAIN_SEPARATOR}{file}"));
            jar_path.exists()
        } else {
            false
        };

        if file_exists && !is_essential_requirement {
            log_debug!(
                "File {} already exists and is not essential, skipping download",
                file
            );
            return Ok(());
        }

        log_debug!("Starting download for file: {}", file);

        if let Some(app_handle) = APP_HANDLE.lock().unwrap().as_ref() {
            utils::emit_to_main_window(app_handle, "download-start", &file);
        }

        if file.ends_with(".jar") {
            let local_path = self.get_as_folder(file).to_path_buf();
            if let Err(e) = fs::create_dir_all(&local_path) {
                log_error!("Failed to create directory {}: {}", local_path.display(), e);
                return Err(format!("Failed to create directory: {}", e));
            }
        }

        let cdn_url = SERVERS.selected_cdn_server.as_ref().map_or_else(
            || {
                log_error!("No CDN server available for download");
                Err("No CDN server available for download.".to_string())
            },
            |server| Ok(server.url.clone()),
        )?;

        let download_url = format!("{cdn_url}{file}");

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(600))
            .build()
            .map_err(|e| {
                log_error!("Failed to create HTTP client: {}", e);
                format!("Failed to create HTTP client: {}", e)
            })?;

        let response = client.get(&download_url).send().await.map_err(|e| {
            log_error!("Failed to make HTTP request to {}: {}", download_url, e);
            format!("Failed to download file {}: {}", file, e)
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
        let dest_path = if file.ends_with(".jar") {
            self.root_dir.join(format!("{file_name}/{file}"))
        } else {
            self.root_dir.join(file)
        };

        let mut dest = tokio::fs::File::create(&dest_path).await.map_err(|e| {
            log_error!(
                "Failed to create destination file {}: {}",
                dest_path.display(),
                e
            );
            format!("Failed to create file: {}", e)
        })?;

        let mut downloaded: u64 = 0;
        let mut last_percentage: u8 = 0;
        let mut stream = response.bytes_stream();

        use futures_util::StreamExt;
        use tokio::io::AsyncWriteExt;

        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|e| {
                log_error!("Failed to read response data for {}: {}", file, e);
                format!("Network read error: {}", e)
            })?;

            dest.write_all(&chunk).await.map_err(|e| {
                log_error!(
                    "Failed to write data to file {}: {}",
                    dest_path.display(),
                    e
                );
                format!("File write error: {}", e)
            })?;

            downloaded += chunk.len() as u64;

            let percentage = if let Some(total) = total_size {
                ((downloaded as f64 / total as f64) * 100.0) as u8
            } else {
                std::cmp::min(99, (downloaded / 1024 / 1024) as u8)
            };

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
                    utils::emit_to_main_window(app_handle, "download-progress", progress_data);
                }
            }
        }

        dest.flush().await.map_err(|e| {
            log_error!("Failed to flush file {}: {}", dest_path.display(), e);
            format!("File flush error: {}", e)
        })?;

        if let Some(app_handle) = APP_HANDLE.lock().unwrap().as_ref() {
            utils::emit_to_main_window(app_handle, "download-complete", &file);
        }

        if file.ends_with(".zip") {
            self.unzip(file).map_err(|e| {
                log_error!("Failed to extract {}: {}", file, e);
                e
            })?;
        }

        if file.ends_with(".jar") {
            log_debug!("Verifying MD5 hash for client file: {}", file);
            self.verify_client_hash(file, &dest_path).await?;
        }

        Ok(())
    }

    async fn verify_client_hash(
        &self,
        filename: &str,
        file_path: &std::path::PathBuf,
    ) -> Result<(), String> {
        let hash_verify_enabled = {
            let settings = SETTINGS
                .lock()
                .map_err(|_| "Failed to access settings".to_string())?;
            log_debug!("Hash verification setting: {}", settings.hash_verify.value);
            settings.hash_verify.value
        };

        let (expected_hash, client_id, client_name) = {
            CLIENT_MANAGER
                .lock()
                .map_err(|_| "Failed to acquire lock on client manager".to_string())?
                .as_ref()
                .ok_or_else(|| "Client manager not initialized".to_string())?
                .clients
                .iter()
                .find(|c| c.filename == filename)
                .map(|c| (c.md5_hash.clone(), c.id, c.name.clone()))
                .ok_or_else(|| format!("Client not found for filename: {}", filename))?
        };

        if !hash_verify_enabled {
            log_info!(
                "Hash verification is disabled, skipping check for {}",
                filename
            );
            return Ok(());
        }

        if let Some(app_handle) = APP_HANDLE.lock().unwrap().as_ref() {
            utils::emit_to_main_window(
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
        let calculated_hash = self.calculate_md5_hash(file_path)?;

        if calculated_hash != expected_hash {
            if let Err(e) = std::fs::remove_file(file_path) {
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
                utils::emit_to_main_window(
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
                "Hash verification failed for {}. Expected: {}, Got: {}. The file has been removed and needs to be redownloaded.",
                filename, expected_hash, calculated_hash
            ));
        }

        log_info!("MD5 hash verification successful for {}", filename);

        if let Some(app_handle) = APP_HANDLE.lock().unwrap().as_ref() {
            utils::emit_to_main_window(
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

    fn calculate_md5_hash(&self, path: &std::path::PathBuf) -> Result<String, String> {
        let bytes =
            std::fs::read(path).map_err(|e| format!("Failed to read file for hashing: {}", e))?;

        let digest = md5::compute(&bytes);
        Ok(format!("{:x}", digest))
    }

    pub fn reset_requirements(&self) -> Result<(), Box<dyn std::error::Error>> {
        let requirements = vec![
            "jdk-21.0.2",
            "jdk-21.0.2.zip",
            "assets",
            "assets.zip",
            "natives",
            "natives.zip",
            "libraries",
            "libraries.zip",
            "natives-1.12",
            "natives-1.12.zip",
            "libraries-1.12",
            "libraries-1.12.zip",
        ];

        for requirement in requirements {
            let path = self.root_dir.join(requirement);
            if path.exists() {
                if path.is_dir() {
                    std::fs::remove_dir_all(&path)?;
                } else {
                    std::fs::remove_file(&path)?;
                }
            }
        }

        Ok(())
    }

    pub fn reset_cache(&self) -> Result<(), Box<dyn std::error::Error>> {
        let cache_dir = self.root_dir.join("cache");
        if cache_dir.exists() {
            std::fs::remove_dir_all(&cache_dir)?;
        }
        Ok(())
    }
}

lazy_static! {
    pub static ref DATA: DataManager = DataManager::new(ROOT_DIR.clone().into());
}
