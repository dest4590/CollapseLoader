use base64::{engine::general_purpose, Engine};

use crate::commands::clients::{
    get_running_client_ids, get_running_custom_client_ids, stop_client, stop_custom_client,
};
use crate::core::utils::globals::CODENAME;
use crate::core::{network::servers::SERVERS, storage::data::DATA};
use std::{fs, path::PathBuf};
use tauri::{AppHandle, Emitter, Manager};
use tokio::task;

#[tauri::command]
pub fn get_version() -> Result<serde_json::Value, String> {
    let result = serde_json::json!({
      "version":  env!("CARGO_PKG_VERSION").to_string(),
      "codename": CODENAME,
      "commitHash": env!("GIT_HASH").to_string(),
      "branch": env!("GIT_BRANCH").to_string(),
      "development": env!("DEVELOPMENT").to_string(),
    });

    Ok(result)
}

#[tauri::command]
pub fn is_development() -> Result<bool, String> {
    let development = env!("DEVELOPMENT").to_string();
    Ok(development == "true")
}

#[tauri::command]
pub fn open_data_folder() -> Result<String, String> {
    let path = DATA.root_dir.to_string_lossy().to_string();

    if let Err(e) = open::that(&path) {
        return Err(format!("Failed to open data folder: {e}"));
    }

    Ok(path)
}

#[tauri::command]
pub fn reset_requirements() -> Result<(), String> {
    if let Err(e) = DATA.reset_requirements() {
        return Err(format!("Failed to reset requirements: {e}"));
    }
    Ok(())
}

#[tauri::command]
pub fn reset_cache() -> Result<(), String> {
    if let Err(e) = DATA.reset_cache() {
        return Err(format!("Failed to reset cache: {e}"));
    }
    Ok(())
}

#[tauri::command]
pub fn get_data_folder() -> Result<String, String> {
    Ok(DATA.root_dir.to_string_lossy().to_string())
}

#[tauri::command]
pub async fn change_data_folder(
    app: AppHandle,
    new_path: String,
    mode: String,
) -> Result<(), String> {
    let new_dir = PathBuf::from(new_path.clone());
    if new_dir.as_os_str().is_empty() {
        return Err("Target path is empty".to_string());
    }

    if !new_dir.exists() {
        fs::create_dir_all(&new_dir).map_err(|e| format!("Failed to create target dir: {e}"))?;
    }

    let running: Vec<u32> = get_running_client_ids().await;
    for id in running {
        let _ = stop_client(id).await;
    }

    let running_custom: Vec<u32> = get_running_custom_client_ids().await;
    for id in running_custom {
        let _ = stop_custom_client(id).await;
    }

    let current_dir = DATA.root_dir.clone();

    if mode == "move" {
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
                copy_dir_recursive(&current_dir, &new_dir)?;
                if let Err(e) = fs::remove_dir_all(&current_dir) {
                    let _ = e;
                }
                Ok(())
            })
            .await
            .map_err(|e| format!("Task join error: {e}"))??;
        }
    } else if mode == "wipe" {
        if current_dir.exists() {
            fs::remove_dir_all(&current_dir)
                .map_err(|e| format!("Failed to wipe old folder: {e}"))?;
        }
    } else {
        return Err("Invalid mode".to_string());
    }

    let roaming_dir = std::env::var("APPDATA")
        .unwrap_or_else(|_| std::env::var("HOME").unwrap_or_else(|_| ".".to_string()));
    let override_file = PathBuf::from(roaming_dir).join("CollapseLoaderRoot.txt");
    fs::write(&override_file, &new_path).map_err(|e| format!("Failed to write override: {e}"))?;

    if let Some(window) = app.get_webview_window("main") {
        let _ = window.emit("data-folder-changed", &new_path);
    }

    Ok(())
}

#[tauri::command]
pub async fn get_auth_url() -> Result<String, String> {
    if let Some(auth_url) = SERVERS.get_auth_server_url() {
        Ok(auth_url)
    } else {
        Ok("https://auth.collapseloader.org".to_string())
    }
}

#[tauri::command]
pub async fn encode_base64(input: String) -> Result<String, String> {
    let encoded = general_purpose::STANDARD.encode(input);
    Ok(encoded)
}

#[tauri::command]
pub async fn decode_base64(input: String) -> Result<String, String> {
    match general_purpose::STANDARD.decode(&input) {
        Ok(decoded) => match String::from_utf8(decoded) {
            Ok(decoded_str) => Ok(decoded_str),
            Err(_) => Err("Failed to decode base64 to UTF-8 string".to_string()),
        },
        Err(_) => Err("Failed to decode base64".to_string()),
    }
}
