use api::clients::{
    agent_overlay::AgentOverlayManager,
    client::{Client, CLIENT_LOGS},
    clients::{initialize_client_manager, CLIENT_MANAGER},
};
use tauri::AppHandle;

use crate::{
    api::{self, analytics::Analytics, core::data::DATA, utils},
    log_debug, log_error, log_info, log_warn,
};

fn get_client_by_id(id: u32) -> Result<Client, String> {
    CLIENT_MANAGER
        .lock()
        .map_err(|_| "Failed to acquire lock on client manager".to_string())?
        .as_ref()
        .ok_or_else(|| "Client manager not initialized".to_string())?
        .clients
        .iter()
        .find(|c| c.id == id)
        .cloned()
        .ok_or_else(|| "Client not found".to_string())
}

#[tauri::command]
pub fn get_app_logs() -> Vec<String> {
    api::logging::APP_LOGS
        .lock()
        .map(|logs| logs.clone())
        .unwrap_or_default()
}

#[tauri::command]
pub async fn initialize_api() -> Result<(), String> {
    initialize_client_manager().await
}

#[tauri::command]
pub fn initialize_rpc() -> Result<(), String> {
    if let Err(e) = api::discord_rpc::initialize() {
        log_error!("Failed to initialize Discord RPC: {}", e);
    } else {
        log_info!("Discord RPC initialized successfully");
    }
    Ok(())
}

#[tauri::command]
pub fn get_server_connectivity_status() -> api::network::servers::ServerConnectivityStatus {
    let servers = &api::network::servers::SERVERS;
    servers.connectivity_status.lock().unwrap().clone()
}

#[tauri::command]
pub fn get_clients() -> Vec<Client> {
    CLIENT_MANAGER
        .lock()
        .ok()
        .and_then(|manager| manager.as_ref().map(|m| m.clients.clone()))
        .unwrap_or_default()
}

#[tauri::command]
pub async fn launch_client(
    id: u32,
    user_token: String,
    app_handle: AppHandle,
) -> Result<(), String> {
    let client = get_client_by_id(id)?;

    let file_name = DATA.get_filename(&client.filename);
    let jar_path = DATA.get_local(&format!("{file_name}/{}", client.filename));

    if !jar_path.exists() {
        return Err(format!(
            "Client {} is not installed. Please download it first.",
            client.name
        ));
    }

    let hash_verify_enabled = {
        let settings = api::core::settings::SETTINGS
            .lock()
            .map_err(|_| "Failed to access settings".to_string())?;
        log_debug!("Hash verification setting: {}", settings.hash_verify.value);
        settings.hash_verify.value
    };

    if hash_verify_enabled {
        utils::emit_to_main_window(
            &app_handle,
            "client-hash-verification-start",
            &serde_json::json!({
                "id": id,
                "name": client.name
            }),
        );

        log_info!(
            "Verifying MD5 hash for client {} before launch",
            client.name
        );
        let current_hash = calculate_md5_hash(&jar_path)?;
        if current_hash != client.md5_hash {
            log_warn!(
                "Hash mismatch for client {}. Expected: {}, Got: {}. Redownloading...",
                client.name,
                client.md5_hash,
                current_hash
            );

            utils::emit_to_main_window(
                &app_handle,
                "client-hash-verification-failed",
                &serde_json::json!({
                    "id": id,
                    "name": client.name,
                    "expected_hash": client.md5_hash,
                    "actual_hash": current_hash
                }),
            );

            if let Err(e) = std::fs::remove_file(&jar_path) {
                log_warn!("Failed to remove corrupted client file: {}", e);
            }

            update_client_installed_status(id, false)?;

            log_info!("Redownloading client: {} (ID: {})", client.name, id);
            client
                .download()
                .await
                .map_err(|e| {
                    if e.contains("Hash verification failed") {
                        format!("Hash verification failed for {}: The downloaded file is corrupted. Please try downloading again.", client.name)
                    } else {
                        format!("Failed to redownload client {}: {}", client.name, e)
                    }
                })?;

            utils::emit_to_main_window(
                &app_handle,
                "client-redownload-complete",
                &serde_json::json!({
                    "id": id,
                    "name": client.name
                }),
            );

            log_info!(
                "Client {} redownloaded and verified successfully",
                client.name
            );
        } else {
            log_info!(
                "MD5 hash verification successful for client {}",
                client.name
            );

            utils::emit_to_main_window(
                &app_handle,
                "client-hash-verification-done",
                &serde_json::json!({
                    "id": id,
                    "name": client.name
                }),
            );
        }
    } else {
        log_debug!(
            "Hash verification disabled, skipping verification for client {}",
            client.name
        );
    }

    match AgentOverlayManager::verify_agent_overlay_files().await {
        Ok(true) => {
            log_debug!("Agent and overlay files verified successfully");
        }
        Ok(false) => {
            log_warn!("Agent/overlay files verification failed, attempting to download...");
            AgentOverlayManager::download_agent_overlay_files()
                .await
                .map_err(|e| format!("Failed to download required agent/overlay files: {}", e))?;
        }
        Err(e) => {
            log_error!("Error verifying agent/overlay files: {}", e);
        }
    }

    client.run(app_handle, user_token).await
}

#[tauri::command]
pub async fn get_running_client_ids() -> Vec<u32> {
    let handle = tokio::task::spawn_blocking(|| {
        Client::get_running_clients()
            .iter()
            .map(|client| client.id)
            .collect()
    });

    match handle.await {
        Ok(result) => result,
        Err(_) => Vec::new(),
    }
}

#[tauri::command]
pub async fn stop_client(id: u32) -> Result<(), String> {
    let client = get_client_by_id(id)?;

    let client_clone = client.clone();
    let handle = tokio::task::spawn_blocking(move || client_clone.stop());

    handle
        .await
        .map_err(|e| format!("Stop client task error: {e}"))?
}

#[tauri::command]
pub fn get_client_logs(id: u32) -> Vec<String> {
    CLIENT_LOGS
        .lock()
        .ok()
        .and_then(|logs| logs.get(&id).cloned())
        .unwrap_or_default()
}

#[tauri::command]
pub async fn download_client_only(id: u32, app_handle: AppHandle) -> Result<(), String> {
    let client = get_client_by_id(id)?;

    log_info!("Downloading client: {} (ID: {})", client.name, id);

    let client_download = async {
        client.download().await.map_err(|e| {
            if e.contains("Hash verification failed") {
                let _ = update_client_installed_status(id, false);
                format!(
                    "Hash verification failed for {}: The downloaded file is corrupted. Please try downloading again.",
                    client.name
                )
            } else {
                e
            }
        })
    };

    let requirements_download = client.download_requirements(&app_handle);

    Analytics::send_client_download_analytics(id);

    tokio::try_join!(client_download, requirements_download)?;

    Ok(())
}

#[tauri::command]
pub async fn reinstall_client(id: u32, app_handle: AppHandle) -> Result<(), String> {
    let client = get_client_by_id(id)?;

    let client_clone = client.clone();
    let handle = tokio::task::spawn_blocking(move || -> Result<(), String> {
        client_clone.remove_installation()?;
        Ok(())
    });

    handle
        .await
        .map_err(|e| format!("Reinstall task error: {e}"))??;

    update_client_installed_status(id, false)?;

    let download_result = client.download().await.map_err(|e| {
        if e.contains("Hash verification failed") {
            format!(
                "Hash verification failed for {}: The downloaded file is corrupted. Please try again.",
                client.name
            )
        } else {
            e
        }
    });

    if download_result.is_err() {
        return download_result;
    }

    let result = client.download_requirements(&app_handle).await;

    if result.is_ok() {
        let _ = AgentOverlayManager::download_agent_overlay_files()
            .await
            .map_err(|e| {
                log_warn!(
                    "Failed to download agent/overlay files during reinstall: {}",
                    e
                );
                e
            });
    }

    result
}

#[tauri::command]
pub fn open_client_folder(id: u32) -> Result<(), String> {
    let client = get_client_by_id(id)?;

    let client_dir_relative = DATA.get_as_folder(&client.filename);

    if !client_dir_relative.exists() {
        return Err("Client folder does not exist".to_string());
    }

    let client_dir_absolute = client_dir_relative
        .canonicalize()
        .map_err(|e| format!("Failed to get absolute path: {e}"))?;

    opener::open(&client_dir_absolute).map_err(|e| {
        format!(
            "Failed to open client folder: {} at path {}",
            e,
            client_dir_absolute.display()
        )
    })
}

#[tauri::command]
pub fn get_latest_client_logs(id: u32) -> Result<String, String> {
    CLIENT_LOGS
        .lock()
        .map_err(|_| "Failed to acquire lock on client logs".to_string())?
        .get(&id)
        .map(|logs| logs.join("\n"))
        .ok_or_else(|| "No logs found for this client".to_string())
}

#[tauri::command]
pub fn update_client_installed_status(id: u32, installed: bool) -> Result<(), String> {
    let mut manager = CLIENT_MANAGER
        .lock()
        .map_err(|_| "Failed to acquire lock on client manager".to_string())?;

    let manager = manager
        .as_mut()
        .ok_or_else(|| "Client manager not initialized".to_string())?;

    let client = manager
        .clients
        .iter_mut()
        .find(|c| c.id == id)
        .ok_or_else(|| "Client not found".to_string())?;

    client.meta.installed = installed;
    Ok(())
}

#[tauri::command]
pub async fn delete_client(id: u32) -> Result<(), String> {
    let client = get_client_by_id(id)?;

    let handle = tokio::task::spawn_blocking(move || client.remove_installation());

    match handle.await {
        Ok(result) => {
            if result.is_ok() {
                update_client_installed_status(id, false)?;
            }
            result
        }
        Err(e) => Err(format!("Delete task error: {e}")),
    }
}

#[tauri::command]
pub async fn get_client_details(client_id: u32) -> Result<serde_json::Value, String> {
    let api_url = crate::commands::utils::get_api_url().await?;
    let url = format!("{}/api/client/{}/detailed", api_url, client_id);

    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Failed to fetch client details: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("API returned error: {}", response.status()));
    }

    let details: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse client details: {}", e))?;

    Ok(details)
}

#[tauri::command]
pub fn increment_client_counter(id: u32, counter_type: String) -> Result<(), String> {
    let mut manager = CLIENT_MANAGER
        .lock()
        .map_err(|_| "Failed to acquire lock on client manager".to_string())?;

    let manager = manager
        .as_mut()
        .ok_or_else(|| "Client manager not initialized".to_string())?;

    let client = manager
        .clients
        .iter_mut()
        .find(|c| c.id == id)
        .ok_or_else(|| "Client not found".to_string())?;

    match counter_type.as_str() {
        "download" => {
            client.downloads += 1;
            log_info!(
                "Incremented download counter for client {} (ID: {}). New count: {}",
                client.name,
                id,
                client.downloads
            );
        }
        "launch" => {
            client.launches += 1;
            log_info!(
                "Incremented launch counter for client {} (ID: {}). New count: {}",
                client.name,
                id,
                client.launches
            );
        }
        _ => {
            return Err(format!("Invalid counter type: {}", counter_type));
        }
    }

    Ok(())
}

fn calculate_md5_hash(path: &std::path::PathBuf) -> Result<String, String> {
    let bytes =
        std::fs::read(path).map_err(|e| format!("Failed to read file for hashing: {}", e))?;

    let digest = md5::compute(&bytes);
    Ok(format!("{:x}", digest))
}
