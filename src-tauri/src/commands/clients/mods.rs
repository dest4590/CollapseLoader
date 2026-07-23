use crate::core::storage::data::DATA;
use crate::log_info;
use crate::AppState;
use tauri::State;

use std::path::PathBuf;

fn get_mods_folder_for_client(
    client: &crate::core::clients::client::Client,
) -> Result<PathBuf, String> {
    let (folder, _) = client.get_launch_paths().map_err(|e| e.to_string())?;
    Ok(folder.join("mods"))
}

fn get_mods_folder_for_custom_client(
    custom_client: &crate::core::clients::custom_clients::CustomClient,
) -> Result<PathBuf, String> {
    custom_client
        .file_path
        .parent()
        .ok_or_else(|| "Cannot determine client folder".to_string())
        .map(|p| p.join("mods"))
}

async fn list_jar_files(mods_folder: &std::path::Path) -> Result<Vec<String>, String> {
    if !mods_folder.exists() {
        return Ok(Vec::new());
    }
    let mut mods = Vec::new();
    let mut entries = tokio::fs::read_dir(mods_folder)
        .await
        .map_err(|e| format!("Failed to read mods directory: {e}"))?;
    while let Some(entry) = entries.next_entry().await.map_err(|e| e.to_string())? {
        let path = entry.path();
        if path.is_file() {
            if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                if filename.ends_with(".jar") {
                    mods.push(filename.to_string());
                }
            }
        }
    }
    Ok(mods)
}

async fn remove_mod_file(mods_folder: &std::path::Path, filename: &str) -> Result<(), String> {
    let target_file = mods_folder.join(filename);
    if !target_file.exists() {
        return Err("Mod file does not exist".to_string());
    }
    tokio::fs::remove_file(target_file)
        .await
        .map_err(|e| format!("Failed to delete mod file: {e}"))
}

#[tauri::command]
pub async fn install_mod_from_url(
    id: u32,
    url: String,
    filename: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    log_info!(
        "Installing mod for client {}: {} from {}",
        id,
        filename,
        url
    );

    let client = super::get_client_by_id(id, &state.clients.manager)?;

    let mods_folder_relative = {
        let (folder, _) = client.get_launch_paths().map_err(|e| e.to_string())?;
        let root = DATA.root_dir.lock().unwrap().clone();
        let relative = folder
            .strip_prefix(&root)
            .map_err(|_| "Client folder is outside of root directory".to_string())?;
        relative.join("mods")
    };

    let mods_folder_str = mods_folder_relative
        .to_str()
        .ok_or_else(|| "Invalid mods folder path".to_string())?;

    DATA.download_to_folder(&url, mods_folder_str).await?;

    log_info!(
        "Successfully installed mod: {} to {}",
        filename,
        mods_folder_str
    );

    Ok(())
}

#[tauri::command]
pub async fn list_installed_mods(
    id: u32,
    state: State<'_, AppState>,
) -> Result<Vec<String>, String> {
    let client = super::get_client_by_id(id, &state.clients.manager)?;
    let mods_folder = get_mods_folder_for_client(&client)?;
    list_jar_files(&mods_folder).await
}

#[tauri::command]
pub async fn uninstall_mod(
    id: u32,
    filename: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let client = super::get_client_by_id(id, &state.clients.manager)?;
    let mods_folder = get_mods_folder_for_client(&client)?;
    remove_mod_file(&mods_folder, &filename).await
}

#[tauri::command]
pub async fn install_mod_for_custom_client(
    id: u32,
    url: String,
    filename: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    log_info!(
        "Installing mod for custom client {}: {} from {}",
        id,
        filename,
        url
    );

    let custom_client = {
        let manager = state.custom_clients.lock();
        manager
            .get_client(id)
            .cloned()
            .ok_or_else(|| "Custom client not found".to_string())?
    };

    let mods_folder = get_mods_folder_for_custom_client(&custom_client)?;

    tokio::fs::create_dir_all(&mods_folder)
        .await
        .map_err(|e| format!("Failed to create mods folder: {e}"))?;

    let dest = mods_folder.join(&filename);

    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .header("User-Agent", "CollapseLoader-Reborn")
        .send()
        .await
        .map_err(|e| format!("Failed to download mod: {e}"))?;

    if !response.status().is_success() {
        return Err(format!(
            "Download failed with status: {}",
            response.status()
        ));
    }

    let bytes = response
        .bytes()
        .await
        .map_err(|e| format!("Failed to read mod response: {e}"))?;

    tokio::fs::write(&dest, &bytes)
        .await
        .map_err(|e| format!("Failed to write mod file: {e}"))?;

    log_info!(
        "Successfully installed mod: {} ({} bytes)",
        filename,
        bytes.len()
    );
    Ok(())
}

#[tauri::command]
pub async fn list_installed_mods_custom(
    id: u32,
    state: State<'_, AppState>,
) -> Result<Vec<String>, String> {
    let custom_client = {
        let manager = state.custom_clients.lock();
        manager
            .get_client(id)
            .cloned()
            .ok_or_else(|| "Custom client not found".to_string())?
    };

    let mods_folder = get_mods_folder_for_custom_client(&custom_client)?;
    list_jar_files(&mods_folder).await
}

#[tauri::command]
pub async fn uninstall_mod_custom(
    id: u32,
    filename: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let custom_client = {
        let manager = state.custom_clients.lock();
        manager
            .get_client(id)
            .cloned()
            .ok_or_else(|| "Custom client not found".to_string())?
    };

    let mods_folder = get_mods_folder_for_custom_client(&custom_client)?;
    remove_mod_file(&mods_folder, &filename).await
}
