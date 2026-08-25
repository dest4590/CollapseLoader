use crate::core::network::servers::SERVERS;
use crate::core::storage::data::DATA;
use crate::core::utils::globals::{AGENT_OVERLAY_FOLDER, TITLEBAR_FILE};
use crate::core::utils::hashing::calculate_md5_hash;
use crate::{log_debug, log_error, log_info, log_warn};
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize)]
struct TitlebarHashes {
    titlebar_windows: Option<String>,
    titlebar_linux: Option<String>,
    titlebar_macos: Option<String>,
}

pub struct TitlebarBrandingManager;

impl TitlebarBrandingManager {
    fn get_api_base_url() -> Result<String, String> {
        SERVERS
            .selected_api
            .read()
            .unwrap()
            .as_ref()
            .map(|server| server.url.clone())
            .ok_or_else(|| "No API server available".to_string())
    }

    fn system_name() -> &'static str {
        if cfg!(target_os = "windows") {
            "windows"
        } else if cfg!(target_os = "macos") {
            "macos"
        } else {
            "linux"
        }
    }

    fn download_url(base: &str, system: &str) -> String {
        match system {
            "windows" => format!("{base}/agent/libCollapseTitlebar.dll"),
            "macos" => format!("{base}/agent/libCollapseTitlebar.dylib"),
            _ => format!("{base}/agent/libCollapseTitlebar.so"),
        }
    }

    fn expected_hash(info: &TitlebarHashes, system: &str) -> Option<String> {
        match system {
            "windows" => info.titlebar_windows.clone(),
            "macos" => info.titlebar_macos.clone(),
            _ => info.titlebar_linux.clone(),
        }
    }

    pub async fn download_titlebar_file() -> Result<(), String> {
        log_debug!("Starting download of titlebar branding file...");

        let info = Self::get_titlebar_hashes().await?;

        let system = Self::system_name();

        if Self::expected_hash(&info, system).is_none() {
            log_warn!(
                "No titlebar hash available for platform '{}', skipping download",
                system
            );
            return Ok(());
        }

        let folder = DATA.root_dir.lock().unwrap().join(AGENT_OVERLAY_FOLDER);
        if !folder.exists() {
            log_debug!("Titlebar folder missing, creating: {}", folder.display());
            fs::create_dir_all(&folder).map_err(|e| format!("Failed to create directory: {e}"))?;
            log_info!("Created titlebar folder: {}", folder.display());
        }

        let titlebar_path = folder.join(TITLEBAR_FILE);

        let base_url = Self::get_api_base_url()?;
        let base = base_url.trim_end_matches('/').to_string();

        let url = Self::download_url(&base, system);
        log_info!("Downloading titlebar branding file for {system}");

        match Self::download_file(&url, &titlebar_path).await {
            Ok(()) => {}
            Err(e) => {
                log_warn!(
                    "Titlebar branding file not available for '{}': {}",
                    system,
                    e
                );
                return Ok(());
            }
        }

        let downloaded_hash = calculate_md5_hash(&titlebar_path)?;
        let expected_hash = Self::expected_hash(&info, system).unwrap_or_default();

        if !expected_hash.is_empty() && downloaded_hash != expected_hash {
            log_error!(
                "Titlebar file hash mismatch. expected={} got={}",
                expected_hash,
                downloaded_hash
            );
            return Err(format!(
                "Titlebar file hash mismatch. Expected: {}, Got: {}",
                expected_hash, downloaded_hash
            ));
        }

        log_info!("Titlebar branding file downloaded and verified successfully");
        Ok(())
    }

    pub async fn verify_titlebar_file() -> Result<bool, String> {
        log_debug!("Verifying titlebar branding file...");

        let folder = DATA.root_dir.lock().unwrap().join(AGENT_OVERLAY_FOLDER);
        if !folder.exists() {
            log_debug!(
                "Titlebar folder missing during verify, creating: {}",
                folder.display()
            );
            fs::create_dir_all(&folder).map_err(|e| format!("Failed to create directory: {e}"))?;
        }

        let titlebar_path = folder.join(TITLEBAR_FILE);

        if !titlebar_path.exists() {
            log_warn!(
                "Titlebar branding file is missing: {}",
                titlebar_path.display()
            );
            return Ok(false);
        }

        let info = Self::get_titlebar_hashes().await?;
        let system = Self::system_name();

        let expected_hash = match Self::expected_hash(&info, system) {
            Some(h) => h,
            None => {
                log_warn!(
                    "No titlebar hash for platform '{}', skipping verification",
                    system
                );
                return Ok(false);
            }
        };

        let hash = calculate_md5_hash(&titlebar_path)?;

        if hash != expected_hash {
            log_warn!(
                "Titlebar file hash verification failed. Expected: {}, Got: {}",
                expected_hash,
                hash
            );
            return Ok(false);
        }

        log_info!("Titlebar branding file verified successfully");
        Ok(true)
    }

    pub fn has_branding_in_jar(jar_path: &Path) -> bool {
        let data = match fs::read(jar_path) {
            Ok(d) => d,
            Err(_) => return false,
        };

        let marker = b"@CollapseLoader";

        if let Some(pos) = data.windows(marker.len()).position(|w| w == marker) {
            let context_start = pos.saturating_sub(16);
            let context_end = (pos + marker.len() + 16).min(data.len());
            let context = &data[context_start..context_end];
            let context_str = String::from_utf8_lossy(context);

            log_info!(
                "Found existing titlebar branding in JAR at offset {}: ...{}...",
                pos,
                context_str.replace('\n', " ").replace('\r', "")
            );
            return true;
        }

        false
    }

    async fn get_titlebar_hashes() -> Result<TitlebarHashes, String> {
        let base_url = Self::get_api_base_url()?;
        let base = base_url.trim_end_matches('/').to_string();

        let url = format!("{base}/agent/hashes.json");

        let client = reqwest::Client::new();
        let response = client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("Failed to get titlebar hashes: {e}"))?;

        if !response.status().is_success() {
            return Err(format!("Backend returned error: {}", response.status()));
        }

        let hashes: TitlebarHashes = response.json().await.map_err(|e| {
            log_error!("Failed to parse titlebar hashes response: {}", e);
            format!("Failed to parse titlebar hashes: {e}")
        })?;

        Ok(hashes)
    }

    async fn download_file(url: &str, path: &PathBuf) -> Result<(), String> {
        let client = reqwest::Client::new();
        let response = client.get(url).send().await.map_err(|e| {
            log_error!("HTTP request failed for {}: {}", url, e);
            format!("Failed to download file: {e}")
        })?;

        if !response.status().is_success() {
            log_error!("Download failed for {}: HTTP {}", url, response.status());
            return Err(format!(
                "Download failed with status: {}",
                response.status()
            ));
        }

        let bytes = response.bytes().await.map_err(|e| {
            log_error!("Failed to read bytes from response for {}: {}", url, e);
            format!("Failed to read file bytes: {e}")
        })?;

        fs::write(path, bytes).map_err(|e| {
            log_error!(
                "Failed to write downloaded file to {}: {}",
                path.display(),
                e
            );
            format!("Failed to write file to disk: {e}")
        })?;

        Ok(())
    }
}
