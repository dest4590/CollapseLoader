use core::clients::{
    client::{Client, CLIENT_LOGS},
    manager::{initialize_client_manager, CLIENT_MANAGER},
};
use tauri::AppHandle;

use crate::core::{
    clients::custom_clients::{CustomClient, Version},
    network::analytics::Analytics,
    storage::{
        custom_clients::{CustomClientUpdate, CUSTOM_CLIENT_MANAGER},
        data::Data,
    },
};
use crate::core::{
    clients::{client::LaunchOptions, internal::agent_overlay::AgentOverlayManager},
    storage::common::JsonStorage,
};
use crate::core::{
    storage::settings::SETTINGS,
    utils::{discord_rpc, logging},
};
use crate::{
    commands::utils::get_auth_url,
    core::{
        clients::client::ClientType,
        network::servers::{ServerConnectivityStatus, SERVERS},
        utils::helpers::emit_to_main_window,
    },
};
use crate::{
    core::{self, storage::data::DATA},
    log_debug, log_error, log_info, log_warn,
};
use std::path::MAIN_SEPARATOR;

use std::path::PathBuf;

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
    logging::APP_LOGS
        .lock()
        .map(|logs| logs.clone())
        .unwrap_or_default()
        .into()
}

#[tauri::command]
pub async fn initialize_api() -> Result<(), String> {
    initialize_client_manager().await
}

#[tauri::command]
pub fn initialize_rpc() -> Result<(), String> {
    if let Err(e) = discord_rpc::initialize() {
        log_error!("Failed to initialize Discord RPC: {}", e);
    }
    Ok(())
}

#[tauri::command]
pub fn get_server_connectivity_status() -> ServerConnectivityStatus {
    let servers = &SERVERS;
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

    let filename_for_if = if client.filename.contains("fabric/") {
        client.filename.replace("fabric/", "")
    } else if client.filename.contains("/fabric") {
        client.filename.replace("/fabric", "")
    } else {
        client.filename.clone()
    };

    let file_name = Data::get_filename(&client.filename);
    let jar_path = match client.client_type {
        ClientType::Default => {
            DATA.get_local(&format!("{file_name}{MAIN_SEPARATOR}{}", client.filename))
        }
        ClientType::Fabric => DATA.get_local(&format!(
            "{file_name}{MAIN_SEPARATOR}mods{MAIN_SEPARATOR}{filename_for_if}"
        )),
    };

    if !jar_path.exists() {
        return Err(format!(
            "Client {} is not installed. Please download it first.",
            client.name
        ));
    }

    let hash_verify_enabled = {
        let settings = SETTINGS
            .lock()
            .map_err(|_| "Failed to access settings".to_string())?;
        settings.hash_verify.value
    };

    if hash_verify_enabled {
        emit_to_main_window(
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
        let current_hash = Data::calculate_md5_hash(&jar_path)?;
        if current_hash == client.md5_hash {
            log_info!(
                "MD5 hash verification successful for client {}",
                client.name
            );

            emit_to_main_window(
                &app_handle,
                "client-hash-verification-done",
                &serde_json::json!({
                    "id": id,
                    "name": client.name
                }),
            );
        } else {
            log_warn!(
                "Hash mismatch for client {}. Expected: {}, Got: {}. Redownloading...",
                client.name,
                client.md5_hash,
                current_hash
            );

            emit_to_main_window(
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

            emit_to_main_window(
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
                .map_err(|e| format!("Failed to download required agent/overlay files: {e}"))?;
        }
        Err(e) => {
            log_error!("Error verifying agent/overlay files: {}", e);
        }
    }

    let options = LaunchOptions::new(app_handle.clone(), user_token.clone(), false);

    client.run(options).await
}

#[tauri::command]
pub async fn get_running_client_ids() -> Vec<u32> {
    let handle = tokio::task::spawn_blocking(|| {
        Client::get_running_clients()
            .iter()
            .map(|client| client.id)
            .collect()
    });

    handle.await.unwrap_or_else(|_| Vec::new())
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

    download_result.as_ref()?;

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
    if let Some(client) = CLIENT_MANAGER
        .lock()
        .map_err(|_| "Failed to acquire lock on client manager".to_string())?
        .as_mut()
        .ok_or_else(|| "Client manager not initialized".to_string())?
        .clients
        .iter_mut()
        .find(|c| c.id == id)
    {
        client.meta.installed = installed;
        Ok(())
    } else {
        Err("Client not found".to_string())
    }
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
    let api_url = get_auth_url().await?;
    let url = format!("{api_url}api/client/{client_id}/detailed");

    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Failed to fetch client details: {e}"))?;

    if !response.status().is_success() {
        return Err(format!("API returned error: {}", response.status()));
    }

    let details: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse client details: {e}"))?;

    Ok(details)
}

#[tauri::command]
pub fn increment_client_counter(id: u32, counter_type: String) -> Result<(), String> {
    if let Some(client) = CLIENT_MANAGER
        .lock()
        .map_err(|_| "Failed to acquire lock on client manager".to_string())?
        .as_mut()
        .ok_or_else(|| "Client manager not initialized".to_string())?
        .clients
        .iter_mut()
        .find(|c| c.id == id)
    {
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
                return Err(format!("Invalid counter type: {counter_type}"));
            }
        }
        Ok(())
    } else {
        Err("Client not found".to_string())
    }
}

#[tauri::command]
pub fn get_custom_clients() -> Vec<CustomClient> {
    CUSTOM_CLIENT_MANAGER
        .lock()
        .ok()
        .map(|manager| manager.clients.clone())
        .unwrap_or_default()
}

#[tauri::command]
pub fn add_custom_client(
    name: String,
    version: String,
    filename: String,
    file_path: String,
    main_class: String,
) -> Result<(), String> {
    let mut manager = CUSTOM_CLIENT_MANAGER
        .lock()
        .map_err(|_| "Failed to acquire lock on custom client manager".to_string())?;

    let version_enum = Version::from_string(&version);
    let path_buf = PathBuf::from(file_path);

    let custom_client = CustomClient::new(0, name, version_enum, filename, path_buf, main_class);

    manager.add_client(custom_client)
}

#[tauri::command]
pub fn remove_custom_client(id: u32) -> Result<(), String> {
    let mut manager = CUSTOM_CLIENT_MANAGER
        .lock()
        .map_err(|_| "Failed to acquire lock on custom client manager".to_string())?;

    manager.remove_client(id)
}

#[tauri::command]
pub fn update_custom_client(
    id: u32,
    name: Option<String>,
    version: Option<String>,
    main_class: Option<String>,
) -> Result<(), String> {
    let mut manager = CUSTOM_CLIENT_MANAGER
        .lock()
        .map_err(|_| "Failed to acquire lock on custom client manager".to_string())?;

    let version_enum = version.map(|v| Version::from_string(&v));

    let updates = CustomClientUpdate {
        name,
        version: version_enum,
        main_class,
    };

    manager.update_client(id, updates)
}

#[tauri::command]
pub async fn launch_custom_client(
    id: u32,
    user_token: String,
    app_handle: AppHandle,
) -> Result<(), String> {
    let custom_client = {
        let mut manager = CUSTOM_CLIENT_MANAGER
            .lock()
            .map_err(|_| "Failed to acquire lock on custom client manager".to_string())?;

        let client = manager
            .get_client_mut(id)
            .ok_or_else(|| "Custom client not found".to_string())?;

        client.launches += 1;
        let client_clone = client.clone();
        manager.save_to_disk();

        client_clone
    };

    custom_client.validate_file()?;

    log_info!("Launching custom client: {}", custom_client.name);

    let client = custom_client.to_client();

    emit_to_main_window(
        &app_handle,
        "custom-client-launched",
        &serde_json::json!({
            "name": custom_client.name
        }),
    );

    let options = LaunchOptions::new(app_handle.clone(), user_token.clone(), true);

    client.run(options).await
}

#[tauri::command]
pub async fn get_running_custom_client_ids() -> Vec<u32> {
    let handle = tokio::task::spawn_blocking(|| {
        CustomClient::get_running_custom_clients()
            .iter()
            .map(|client| client.id)
            .collect()
    });

    handle.await.unwrap_or_else(|_| Vec::new())
}

#[tauri::command]
pub async fn stop_custom_client(id: u32) -> Result<(), String> {
    let custom_client = {
        let manager = CUSTOM_CLIENT_MANAGER
            .lock()
            .map_err(|_| "Failed to acquire lock on custom client manager".to_string())?;

        manager
            .get_client(id)
            .cloned()
            .ok_or_else(|| "Custom client not found".to_string())?
    };

    let client_clone = custom_client.clone();
    let handle = tokio::task::spawn_blocking(move || client_clone.stop());

    handle
        .await
        .map_err(|e| format!("Stop custom client task error: {e}"))?
}
