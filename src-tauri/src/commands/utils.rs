use base64::{engine::general_purpose, Engine};

use crate::commands::clients::{
    get_running_client_ids, get_running_custom_client_ids, stop_client, stop_custom_client,
};
use crate::core::utils::discord_rpc;
use crate::core::utils::globals::{API_VERSION, CODENAME};
use crate::core::utils::helpers::is_development_enabled;
use crate::core::storage::accounts::ACCOUNT_MANAGER;
use crate::core::storage::custom_clients::CUSTOM_CLIENT_MANAGER;
use crate::core::storage::favorites::FAVORITE_MANAGER;
use crate::core::storage::flags::FLAGS_MANAGER;
use crate::core::storage::presets::PRESET_MANAGER;
use crate::core::storage::settings::SETTINGS;
use crate::core::{network::servers::SERVERS, storage::data::DATA};
use crate::AppState;
use crate::{log_debug, log_error, log_info, log_warn};
use std::{fs, path::PathBuf};
use tauri::{AppHandle, Emitter, Manager, State};
use tokio::task;

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
    log_info!("Resetting client requirements");
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
    log_debug!("Getting data folder path: {}", path);
    Ok(path)
}

#[tauri::command]
pub async fn change_data_folder(
    app: AppHandle,
    new_path: String,
    mode: String,
    state: State<'_, AppState>,
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
        .await?;

    for id in running {
        log_debug!("Stopping client with ID: {}", id);
        let _ = stop_client(id, state.clone()).await;
    }

    let running_custom: Vec<u32> = get_running_custom_client_ids().await;
    for id in running_custom {
        log_debug!("Stopping custom client with ID: {}", id);
        let _ = stop_custom_client(id, state.clone()).await;
    }

    let current_dir = DATA.root_dir.lock().unwrap().clone();
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
        log_error!("Failed to write to override file: {:?}", e);
        format!("Failed to write to override file: {e}")
    })?;

    {
        let mut root = DATA.root_dir.lock().unwrap();
        *root = PathBuf::from(new_path.clone());
    }

    let new_root = PathBuf::from(new_path.clone());

    if let Ok(mut s) = SETTINGS.lock() {
        s.config_path = new_root.join("config.json");
        log_debug!("Updated SETTINGS path: {:?}", s.config_path);
    }
    if let Ok(mut pm) = PRESET_MANAGER.lock() {
        pm.config_path = new_root.join("presets.json");
        log_debug!("Updated PRESET_MANAGER path: {:?}", pm.config_path);
    }
    if let Ok(mut am) = ACCOUNT_MANAGER.lock() {
        am.accounts_path = new_root.join("accounts.json");
        log_debug!("Updated ACCOUNT_MANAGER path: {:?}", am.accounts_path);
    }
    if let Ok(mut ccm) = CUSTOM_CLIENT_MANAGER.lock() {
        ccm.custom_clients_path = new_root.join("custom_clients.json");
        log_debug!("Updated CUSTOM_CLIENT_MANAGER path: {:?}", ccm.custom_clients_path);
    }
    if let Ok(mut fm) = FAVORITE_MANAGER.lock() {
        fm.favorites_path = new_root.join("favorites.json");
        log_debug!("Updated FAVORITE_MANAGER path: {:?}", fm.favorites_path);
    }
    if let Ok(mut f) = FLAGS_MANAGER.lock() {
        f.flags_path = new_root.join("flags.json");
        log_debug!("Updated FLAGS_MANAGER path: {:?}", f.flags_path);
    }

    if let Some(window) = app.get_webview_window("main") {
        log_debug!("Emitting 'data-folder-changed' event to main window");
        let _ = window.emit("data-folder-changed", &new_path);
    }

    log_info!("Data folder change process completed successfully");
    Ok(())
}

#[tauri::command]
pub async fn get_api_url() -> Result<String, String> {
    SERVERS
        .get_api_server_url()
        .map_or_else(|| Ok("https://atlas.collapseloader.org".to_string()), Ok)
}

#[tauri::command]
pub fn get_api_version() -> Result<String, String> {
    Ok(API_VERSION.to_string())
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
