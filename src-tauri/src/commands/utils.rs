use base64::{engine::general_purpose, Engine};

use crate::commands::clients::{
    get_running_client_ids, get_running_custom_client_ids, stop_client, stop_custom_client,
};
use crate::core::clients::manager::ClientManager;
use crate::core::utils::globals::CODENAME;
use crate::core::utils::helpers::is_development_enabled;
use crate::core::{network::servers::SERVERS, storage::data::DATA};
use crate::{log_debug, log_error, log_info, log_warn};
use std::sync::{Arc, Mutex};
use std::{fs, path::PathBuf};
use tauri::{AppHandle, Emitter, Manager, State};
use tokio::task;

#[tauri::command]
pub fn get_version() -> Result<serde_json::Value, String> {
    let result = serde_json::json!({
      "version":  env!("CARGO_PKG_VERSION").to_string(),
      "codename": CODENAME,
      "commitHash": env!("GIT_HASH").to_string(),
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
    let path = DATA.root_dir.to_string_lossy().to_string();
    log_info!("Opening data folder at: {}", path);

    if let Err(e) = open::that(&path) {
        log_error!("Failed to open data folder at {}: {}", path, e);
        return Err(format!("Failed to open data folder: {e}"));
    }

    Ok(path)
}

#[tauri::command]
pub async fn reset_requirements() -> Result<(), String> {
    log_info!("Resetting client requirements");
    if let Err(e) = DATA.reset_requirements().await {
        log_error!("Failed to reset requirements: {}", e);
        return Err(format!("Failed to reset requirements: {e}"));
    }
    log_info!("Client requirements reset successfully");
    Ok(())
}

#[tauri::command]
pub async fn reset_cache() -> Result<(), String> {
    log_info!("Resetting application cache");
    if let Err(e) = DATA.reset_cache().await {
        log_error!("Failed to reset cache: {}", e);
        return Err(format!("Failed to reset cache: {e}"));
    }
    log_info!("Application cache reset successfully");
    Ok(())
}

#[tauri::command]
pub fn get_data_folder() -> Result<String, String> {
    let path = DATA.root_dir.to_string_lossy().to_string();
    log_debug!("Getting data folder path: {}", path);
    Ok(path)
}

#[tauri::command]
pub async fn change_data_folder(
    app: AppHandle,
    new_path: String,
    mode: String,
    state: State<'_, Arc<Mutex<ClientManager>>>,
) -> Result<(), String> {
    log_info!(
        "Changing data folder to '{}' with mode '{}'",
        new_path,
        mode
    );
    let new_dir = PathBuf::from(new_path.clone());
    if new_dir.as_os_str().is_empty() {
        log_warn!("Change data folder failed: Target path is empty");
        return Err("Target path is empty".to_string());
    }

    if !new_dir.exists() {
        log_debug!(
            "Target directory does not exist, creating it: {:?}",
            new_dir
        );
        fs::create_dir_all(&new_dir).map_err(|e| {
            log_error!("Failed to create target directory {:?}: {}", new_dir, e);
            format!("Failed to create target dir: {e}")
        })?;
    }

    log_info!("Stopping all running clients before changing data folder");
    let running: Vec<u32> = get_running_client_ids(state.clone())
        .await
        .map_err(|e| e.to_string())?;
    for id in running {
        log_debug!("Stopping client with ID: {}", id);
        let _ = stop_client(id, state.clone()).await;
    }

    let running_custom: Vec<u32> = get_running_custom_client_ids().await;
    for id in running_custom {
        log_debug!("Stopping custom client with ID: {}", id);
        let _ = stop_custom_client(id).await;
    }

    let current_dir = DATA.root_dir.clone();
    log_debug!("Current data directory is: {:?}", current_dir);

    if mode == "move" {
        log_info!("Moving data from old folder to new folder");
        if current_dir.exists() {
            task::spawn_blocking(move || -> Result<(), String> {
                fn copy_dir_recursive(
                    src: &std::path::Path,
                    dst: &std::path::Path,
                ) -> Result<(), String> {
                    for entry in fs::read_dir(src).map_err(|e| e.to_string())? {
                        let entry = entry.map_err(|e| e.to_string())?;
                        let path = entry.path();
                        let target = dst.join(entry.file_name());
                        if path.is_dir() {
                            fs::create_dir_all(&target).map_err(|e| e.to_string())?;
                            copy_dir_recursive(&path, &target)?;
                        } else {
                            fs::copy(&path, &target).map_err(|e| e.to_string())?;
                        }
                    }
                    Ok(())
                }
                log_debug!(
                    "Starting recursive copy from {:?} to {:?}",
                    current_dir,
                    new_dir
                );
                copy_dir_recursive(&current_dir, &new_dir)?;
                log_debug!("Finished recursive copy. Removing old directory.");
                if let Err(e) = fs::remove_dir_all(&current_dir) {
                    log_warn!("Failed to remove old data directory: {}", e);
                    let _ = e;
                }
                Ok(())
            })
            .await
            .map_err(|e| {
                log_error!("Task to move data folder failed: {}", e);
                format!("Task join error: {e}")
            })??;
        }
    } else if mode == "wipe" {
        log_info!("Wiping old data folder");
        if current_dir.exists() {
            fs::remove_dir_all(&current_dir).map_err(|e| {
                log_error!("Failed to wipe old data folder: {}", e);
                format!("Failed to wipe old folder: {e}")
            })?;
        }
    } else {
        log_warn!("Invalid mode for changing data folder: {}", mode);
        return Err("Invalid mode".to_string());
    }

    let roaming_dir = std::env::var("APPDATA")
        .unwrap_or_else(|_| std::env::var("HOME").unwrap_or_else(|_| ".".to_string()));
    let override_file = PathBuf::from(roaming_dir).join("CollapseLoaderRoot.txt");
    log_info!(
        "Writing new data folder path to override file: {:?}",
        override_file
    );
    fs::write(&override_file, &new_path).map_err(|e| {
        log_error!("Failed to write override file: {}", e);
        format!("Failed to write override: {e}")
    })?;

    if let Some(window) = app.get_webview_window("main") {
        log_debug!("Emitting 'data-folder-changed' event to main window");
        let _ = window.emit("data-folder-changed", &new_path);
    }

    log_info!("Data folder change process completed successfully");
    Ok(())
}

#[tauri::command]
pub async fn get_auth_url() -> Result<String, String> {
    SERVERS
        .get_auth_server_url()
        .map_or_else(|| Ok("https://auth.collapseloader.org".to_string()), Ok)
}

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
