use crate::core::utils::fs as fs_utils;
use crate::AppState;
use crate::{log_debug, log_error, log_info, log_warn};
use std::{fs, path::PathBuf};
use tauri::{AppHandle, Emitter, Manager, State};
use tokio::task;

use crate::commands::clients::{
    get_running_client_ids, get_running_custom_client_ids, stop_client, stop_custom_client,
};

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
        fs_utils::ensure_dir(&new_dir).map_err(|e| {
            log_error!("Failed to create target directory {:?}: {}", new_dir, e);
            format!("Failed to create target dir: {e}")
        })?;
    }

    log_info!("Stopping all running clients before changing data folder");

    let running: Vec<u32> = get_running_client_ids(state.clone()).await?;

    for id in running {
        log_debug!("Stopping client with ID: {}", id);
        let _ = stop_client(id, state.clone()).await;
    }

    let running_custom: Vec<u32> = get_running_custom_client_ids().await;
    for id in running_custom {
        log_debug!("Stopping custom client with ID: {}", id);
        let _ = stop_custom_client(id, state.clone()).await;
    }

    let current_dir = crate::core::storage::data::DATA
        .root_dir
        .lock()
        .unwrap()
        .clone();
    log_debug!("Current data directory is: {:?}", current_dir);

    if mode == "move" {
        log_info!("Moving data from old folder to new folder");
        if current_dir.exists() {
            task::spawn_blocking(move || -> Result<(), String> {
                log_debug!(
                    "Starting recursive copy from {:?} to {:?}",
                    current_dir,
                    new_dir
                );
                fs_utils::copy_dir_recursive(&current_dir, &new_dir, true)?;
                log_debug!(
                    "Finished recursive copy. Removing old directory contents (except aci.json)."
                );
                if current_dir.exists() {
                    if let Ok(entries) = fs::read_dir(&current_dir) {
                        for entry in entries.flatten() {
                            let path = entry.path();
                            if path.is_file()
                                && path.file_name().and_then(|n| n.to_str()) == Some("aci.json")
                            {
                                continue;
                            }
                            let _ = fs_utils::remove_path(&path);
                        }
                    }
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
        log_info!("Wiping old data folder (preserving aci.json)");
        if current_dir.exists() {
            if let Ok(entries) = fs::read_dir(&current_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_file()
                        && path.file_name().and_then(|n| n.to_str()) == Some("aci.json")
                    {
                        log_debug!("Preserving aci.json during wipe");
                        continue;
                    }
                    let _ = fs_utils::remove_path(&path);
                }
            }
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
        let mut root = crate::core::storage::data::DATA.root_dir.lock().unwrap();
        *root = PathBuf::from(new_path.clone());
    }

    let new_root = PathBuf::from(new_path.clone());

    {
        let mut s = state.settings();
        s.config_path = new_root.join("config.json");
        log_debug!("Updated SETTINGS path: {:?}", s.config_path);
    }
    {
        let mut pm = state.presets();
        pm.config_path = new_root.join("presets.json");
        log_debug!("Updated PRESET_MANAGER path: {:?}", pm.config_path);
    }
    {
        let mut am = state.accounts();
        am.accounts_path = new_root.join("accounts.json");
        log_debug!("Updated ACCOUNT_MANAGER path: {:?}", am.accounts_path);
    }
    {
        let mut ccm = state.custom_clients.lock();
        ccm.custom_clients_path = new_root.join("custom_clients.json");
        log_debug!(
            "Updated CUSTOM_CLIENT_MANAGER path: {:?}",
            ccm.custom_clients_path
        );
    }
    {
        let mut fm = state.favorites();
        fm.favorites_path = new_root.join("favorites.json");
        log_debug!("Updated FAVORITE_MANAGER path: {:?}", fm.favorites_path);
    }
    {
        let mut f = state.flags();
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
