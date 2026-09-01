pub mod data_folder;
pub mod tray;

pub use data_folder::*;
pub use tray::*;

use base64::{engine::general_purpose, Engine};

use crate::core::storage::launch_history::LaunchEntry;
use crate::core::utils::discord_rpc;
use crate::core::utils::fs as fs_utils;
use crate::core::utils::globals::{API_SERVERS, CDN_SERVERS, CODENAME};
use crate::core::utils::helpers::is_development_enabled;
use crate::core::{network::servers::SERVERS, storage::data::DATA};
use crate::AppState;
use crate::{log_debug, log_error, log_info, log_warn};
use std::fs;
use tauri::{State, Theme, Window};

#[tauri::command]
pub fn cancel_download(name: String) -> Result<bool, String> {
    Ok(crate::core::network::downloader::cancel_download(&name))
}

#[tauri::command]
pub fn get_version() -> Result<serde_json::Value, String> {
    let result = serde_json::json!({
      "version":  env!("CARGO_PKG_VERSION").to_string(),
      "codename": CODENAME,
      "commitHash": env!("GIT_HASH").to_string(),
      "commitMessage": env!("GIT_COMMIT_BODY").to_string(),
      "branch": env!("GIT_BRANCH").to_string(),
      "development": env!("DEVELOPMENT").to_lowercase(),
    });

    Ok(result)
}

#[tauri::command]
pub fn is_development() -> Result<bool, String> {
    Ok(is_development_enabled())
}

#[tauri::command]
pub fn open_data_folder() -> Result<String, String> {
    let path = DATA.root_dir.lock().unwrap().to_string_lossy().to_string();
    log_info!("Opening data folder at: {}", path);

    if let Err(e) = open::that(&path) {
        log_error!("Failed to open data folder at {}: {}", path, e);
        return Err(format!("Failed to open data folder: {e}"));
    }

    Ok(path)
}

#[tauri::command]
pub async fn reset_requirements() -> Result<(), String> {
    if let Err(e) = DATA.reset_requirements().await {
        log_error!("Failed to reset requirements: {}", e);
        return Err(format!("Failed to reset requirements: {e}"));
    }
    log_info!("Client requirements reset successfully");
    Ok(())
}

#[tauri::command]
pub fn get_data_folder() -> Result<String, String> {
    let path = DATA.root_dir.lock().unwrap().to_string_lossy().to_string();
    // log_debug!("Getting data folder path: {}", path);
    Ok(path)
}

#[tauri::command]
pub async fn get_api_url() -> Result<String, String> {
    SERVERS.wait_for_initial_check().await;
    SERVERS.get_api_server_url().map_or_else(
        || {
            Ok(API_SERVERS
                .first()
                .map(|s| s.url.clone())
                .unwrap_or_default())
        },
        Ok,
    )
}

#[tauri::command]
pub async fn get_cdn_url() -> Result<String, String> {
    SERVERS.wait_for_initial_check().await;
    SERVERS.get_cdn_server_url().map_or_else(
        || {
            Ok(CDN_SERVERS
                .first()
                .map(|s| s.url.clone())
                .unwrap_or_default())
        },
        Ok,
    )
}

// #[tauri::command]
// pub fn get_api_version() -> Result<String, String> {
//     Ok(API_VERSION.to_string())
// }

#[tauri::command]
pub async fn encode_base64(input: String) -> Result<String, String> {
    let encoded = general_purpose::STANDARD.encode(input);
    Ok(encoded)
}

#[tauri::command]
pub async fn decode_base64(input: String) -> Result<String, String> {
    general_purpose::STANDARD.decode(&input).ok().map_or_else(
        || {
            log_warn!("Failed to decode Base64 string");
            Err("Failed to decode base64".to_string())
        },
        |decoded| {
            String::from_utf8(decoded).map_err(|e| {
                log_warn!("Failed to convert decoded bytes to UTF-8 string: {}", e);
                "Failed to decode base64 to UTF-8 string".to_string()
            })
        },
    )
}

#[tauri::command]
pub fn update_presence(details: String, state: String) -> Result<(), String> {
    log_debug!(
        "Updating Discord presence: details='{}', state='{}'",
        details,
        state
    );
    discord_rpc::update_activity_async(details, state);
    Ok(())
}

#[tauri::command]
pub fn is_macos() -> bool {
    cfg!(target_os = "macos")
}

#[tauri::command]
pub fn set_window_theme(window: Window, theme: String) {
    std::thread::spawn(move || {
        let target_theme = match theme.as_str() {
            "dark" => Some(Theme::Dark),
            "light" => Some(Theme::Light),
            _ => None,
        };

        if let Some(t) = target_theme {
            let _ = window.set_theme(Some(t));
        }
    });
}

#[derive(serde::Serialize)]
pub struct StorageUsage {
    pub clients: u64,
    pub libraries: u64,
    pub natives: u64,
    pub assets: u64,
    pub java: u64,
    pub other: u64,
    pub total: u64,
}

pub(crate) fn dir_size(path: &std::path::Path) -> u64 {
    if !path.exists() {
        return 0;
    }
    let mut total = 0u64;
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_dir() {
                total += dir_size(&p);
            } else if let Ok(meta) = fs::metadata(&p) {
                total += meta.len();
            }
        }
    }
    total
}

#[tauri::command]
pub async fn get_storage_usage() -> StorageUsage {
    tokio::task::spawn_blocking(|| {
        let root = DATA.root_dir.lock().unwrap().clone();

        let libraries = dir_size(&root.join("libraries"))
            + dir_size(&root.join("libraries-fabric"))
            + dir_size(&root.join("libraries-legacy"));

        let natives = dir_size(&root.join("natives"))
            + dir_size(&root.join("natives-macos-x64"))
            + dir_size(&root.join("natives-macos-arm64"))
            + dir_size(&root.join("natives-linux"))
            + dir_size(&root.join("natives-legacy"))
            + dir_size(&root.join("natives-legacy-linux"))
            + dir_size(&root.join("natives-fabric"));

        let assets = dir_size(&root.join("assets")) + dir_size(&root.join("assets-fabric"));

        let mc_versions = dir_size(&root.join("minecraft-versions"));
        let custom_clients_size = dir_size(&root.join("custom_clients"));

        let mut java = 0u64;
        let mut client_folders = 0u64;

        if let Ok(entries) = fs::read_dir(&root) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_lowercase();
                let path = entry.path();

                if !path.is_dir() {
                    continue;
                }

                if name.starts_with("jdk") {
                    java += dir_size(&path);
                } else if !fs_utils::SYSTEM_DIRS.contains(&name.as_str()) {
                    client_folders += dir_size(&path);
                }
            }
        }

        let clients = mc_versions + custom_clients_size + client_folders;
        let total = dir_size(&root);
        let accounted = clients + libraries + natives + assets + java;
        let other = total.saturating_sub(accounted);

        StorageUsage {
            clients,
            libraries,
            natives,
            assets,
            java,
            other,
            total,
        }
    })
    .await
    .unwrap_or(StorageUsage {
        clients: 0,
        libraries: 0,
        natives: 0,
        assets: 0,
        java: 0,
        other: 0,
        total: 0,
    })
}

#[tauri::command]
pub fn get_launch_history(state: State<'_, AppState>) -> Vec<LaunchEntry> {
    state.launch_history().entries.clone()
}

#[tauri::command]
pub fn clear_launch_history(state: State<'_, AppState>) -> Result<(), String> {
    state.launch_history().clear();
    Ok(())
}

#[tauri::command]
pub fn record_launch(
    state: State<'_, AppState>,
    client_id: u32,
    client_name: String,
    client_version: String,
    account_name: Option<String>,
) -> Result<(), String> {
    let launched_at = chrono::Utc::now().to_rfc3339();
    let entry = LaunchEntry {
        client_id,
        client_name,
        client_version,
        launched_at,
        account_name,
    };
    state.launch_history().record(entry);
    Ok(())
}
