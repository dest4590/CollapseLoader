use core::clients::{
    client::{Client, CLIENT_LOGS},
    manager::ClientManager,
};
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, State};

use crate::core::{
    clients::custom_clients::{CustomClient, Version},
    network::analytics::Analytics,
    storage::{
        custom_clients::{CustomClientUpdate, CUSTOM_CLIENT_MANAGER},
        data::Data,
    },
    utils::globals::SKIP_AGENT_OVERLAY_VERIFICATION,
};
use crate::core::{
    clients::{client::LaunchOptions, internal::agent_overlay::AgentOverlayManager},
    storage::common::JsonStorage,
};
use crate::core::{
    storage::settings::SETTINGS,
    utils::{discord_rpc, hashing::calculate_md5_hash, logging},
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

use std::path::PathBuf;

fn get_client_by_id(id: u32, manager: &Arc<Mutex<ClientManager>>) -> Result<Client, String> {
    manager
        .lock()
        .map_err(|_| "Failed to acquire lock on client manager".to_string())?
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
pub async fn initialize_api(state: State<'_, Arc<Mutex<ClientManager>>>) -> Result<(), String> {
    log_info!("Initializing client manager via API");
    let clients = ClientManager::fetch_clients()
        .await
        .map_err(|e| e.to_string())?;
    let mut manager = state
        .lock()
        .map_err(|_| "Failed to lock state".to_string())?;
    manager.clients = clients;
    Ok(())
}

#[tauri::command]
pub fn initialize_rpc() -> Result<(), String> {
    log_info!("Initializing Discord RPC");
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
pub fn get_clients(state: State<'_, Arc<Mutex<ClientManager>>>) -> Vec<Client> {
    state
        .lock()
        .ok()
        .map(|manager| manager.clients.clone())
        .unwrap_or_default()
}

#[tauri::command]
pub async fn launch_client(
    id: u32,
    user_token: String,
    app_handle: AppHandle,
    state: State<'_, Arc<Mutex<ClientManager>>>,
) -> Result<(), String> {
    log_info!("Attempting to launch client with ID: {}", id);
    let client = get_client_by_id(id, &state)?;
    log_debug!("Found client '{}' for launch", client.name);

    let filename_for_if = if client.filename.contains("fabric/") {
        client.filename.replace("fabric/", "")
    } else if client.filename.contains("/fabric") {
        client.filename.replace("/fabric", "")
    } else {
        client.filename.clone()
    };

    const MAIN_SEPARATOR: char = std::path::MAIN_SEPARATOR;

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
        log_warn!(
            "Launch failed: Client '{}' is not installed at path: {}",
            client.name,
            jar_path.display()
        );
        return Err(format!(
            "Client {} is not installed. Please download it first.",
            client.name
        ));
    }
    log_debug!(
        "Client '{}' found at path: {}",
        client.name,
        jar_path.display()
    );

    let hash_verify_enabled = {
        let settings = SETTINGS
            .lock()
            .map_err(|_| "Failed to access settings".to_string())?;
        settings.hash_verify.value
    };

    if hash_verify_enabled {
        log_info!("Hash verification is enabled for client '{}'", client.name);
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
        let current_hash = calculate_md5_hash(&jar_path)?;
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

            update_client_installed_status(id, false, state.clone())?;

            log_info!("Redownloading client: {} (ID: {})", client.name, id);
            client
                .download(&state)
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

    log_info!("Verifying agent and overlay files before launch");
    match AgentOverlayManager::verify_agent_overlay_files().await {
        Ok(true) => {
            log_debug!("Agent and overlay files verified successfully");
        }
        Ok(false) => {
            if !*SKIP_AGENT_OVERLAY_VERIFICATION {
                log_warn!("Agent/overlay files verification failed, attempting to download...");
                AgentOverlayManager::download_agent_overlay_files()
                    .await
                    .map_err(|e| format!("Failed to download required agent/overlay files: {e}"))?;
            } else {
                log_warn!("Agent/overlay files verification failed, but skipping download due to SKIP_AGENT_OVERLAY_VERIFICATION being enabled.");
            }
        }
        Err(e) => {
            log_error!("Error verifying agent/overlay files: {}", e);
        }
    }

    let options = LaunchOptions::new(app_handle.clone(), user_token.clone(), false);

    log_info!("Executing client run for '{}'", client.name);
    client.run(options, (*state).clone()).await
}

#[tauri::command]
pub async fn get_running_client_ids(
    state: State<'_, Arc<Mutex<ClientManager>>>,
) -> Result<Vec<u32>, String> {
    let manager = (*state).clone();
    let handle = tokio::task::spawn_blocking(move || {
        Client::get_running_clients(&manager)
            .iter()
            .map(|client| client.id)
            .collect()
    });

    handle
        .await
        .map_err(|e| format!("Failed to get running client IDs: {}", e))
}

#[tauri::command]
pub async fn stop_client(
    id: u32,
    state: State<'_, Arc<Mutex<ClientManager>>>,
) -> Result<(), String> {
    log_info!("Attempting to stop client with ID: {}", id);
    let client = get_client_by_id(id, &state)?;
    log_debug!("Found client '{}' to stop", client.name);

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
pub async fn download_client_only(
    id: u32,
    app_handle: AppHandle,
    state: State<'_, Arc<Mutex<ClientManager>>>,
) -> Result<(), String> {
    let client = get_client_by_id(id, &state)?;

    log_info!("Starting download for client: {} (ID: {})", client.name, id);

    let state_clone = state.clone();
    let client_clone = client.clone();
    let client_download = async move {
        client_clone.download(&state_clone).await.map_err(|e| {
            if e.contains("Hash verification failed") {
                let _ = update_client_installed_status(id, false, state_clone.clone());
                format!(
                    "Hash verification failed for {}: The downloaded file is corrupted. Please try downloading again.",
                    client_clone.name
                )
            } else {
                e
            }
        })
    };

    let requirements_download = client.download_requirements(&app_handle);

    Analytics::send_client_download_analytics(id);
    log_debug!("Sent client download analytics for ID: {}", id);

    tokio::try_join!(client_download, requirements_download)?;

    log_info!(
        "Successfully downloaded client and requirements for '{}'",
        client.name
    );

    Ok(())
}

#[tauri::command]
pub async fn reinstall_client(
    id: u32,
    app_handle: AppHandle,
    state: State<'_, Arc<Mutex<ClientManager>>>,
) -> Result<(), String> {
    log_info!("Starting reinstall for client ID: {}", id);
    let client = get_client_by_id(id, &state)?;
    log_debug!("Found client '{}' for reinstall", client.name);

    let client_clone = client.clone();
    let manager = (*state).clone();
    let handle = tokio::task::spawn_blocking(move || -> Result<(), String> {
        log_info!("Removing existing installation for '{}'", client_clone.name);
        client_clone.remove_installation(&manager)?;
        log_info!(
            "Successfully removed existing installation for '{}'",
            client_clone.name
        );
        Ok(())
    });

    handle
        .await
        .map_err(|e| format!("Reinstall task error: {e}"))??;

    update_client_installed_status(id, false, state.clone())?;
    log_debug!(
        "Updated installed status to false for client '{}'",
        client.name
    );

    let download_result = client.download(&state).await.map_err(|e| {
        if e.contains("Hash verification failed") {
            format!(
                "Hash verification failed for {}: The downloaded file is corrupted. Please try again.",
                client.name
            )
        } else {
            log_error!("Client download failed during reinstall: {}", e);
            e
        }
    });

    if let Err(e) = download_result.as_ref() {
        log_error!(
            "Aborting reinstall for '{}' due to download failure: {}",
            client.name,
            e
        );
        return Err(e.clone());
    }
    log_info!("Client '{}' downloaded successfully", client.name);

    let result = client.download_requirements(&app_handle).await;
    if result.is_ok() {
        log_info!(
            "Client requirements for '{}' downloaded successfully",
            client.name
        );
    }

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
pub fn open_client_folder(
    id: u32,
    state: State<'_, Arc<Mutex<ClientManager>>>,
) -> Result<(), String> {
    log_info!("Attempting to open folder for client ID: {}", id);
    let client = get_client_by_id(id, &state)?;
    log_debug!("Found client '{}' to open folder", client.name);

    let client_dir_relative = DATA.get_as_folder(&client.filename);

    if !client_dir_relative.exists() {
        log_warn!(
            "Cannot open folder for client '{}', it does not exist at path: {}",
            client.name,
            client_dir_relative.display()
        );
        return Err("Client folder does not exist".to_string());
    }

    let client_dir_absolute = client_dir_relative
        .canonicalize()
        .map_err(|e| format!("Failed to get absolute path: {e}"))?;

    log_debug!(
        "Opening client folder at: {}",
        client_dir_absolute.display()
    );
    opener::open(&client_dir_absolute).map_err(|e| {
        log_error!(
            "Failed to open client folder at {}: {}",
            client_dir_absolute.display(),
            e
        );
        format!(
            "Failed to open client folder: {} at path {}",
            e,
            client_dir_absolute.display()
        )
    })
}

#[tauri::command]
pub fn get_latest_client_logs(id: u32) -> Result<String, String> {
    log_debug!("Fetching latest logs for client ID: {}", id);
    CLIENT_LOGS
        .lock()
        .map_err(|_| "Failed to acquire lock on client logs".to_string())?
        .get(&id)
        .map(|logs| logs.join("\n"))
        .ok_or_else(|| "No logs found for this client".to_string())
}

#[tauri::command]
pub fn update_client_installed_status(
    id: u32,
    installed: bool,
    state: State<'_, Arc<Mutex<ClientManager>>>,
) -> Result<(), String> {
    if let Some(client) = state
        .lock()
        .map_err(|_| "Failed to acquire lock on client manager".to_string())?
        .clients
        .iter_mut()
        .find(|c| c.id == id)
    {
        client.meta.installed = installed;
        Ok(())
    } else {
        log_warn!(
            "Could not update installed status: Client ID {} not found",
            id
        );
        Err("Client not found".to_string())
    }
}

#[tauri::command]
pub async fn delete_client(
    id: u32,
    state: State<'_, Arc<Mutex<ClientManager>>>,
) -> Result<(), String> {
    log_info!("Attempting to delete client with ID: {}", id);
    let client = get_client_by_id(id, &state)?;
    log_debug!("Found client '{}' for deletion", client.name);

    let manager = (*state).clone();
    let handle = tokio::task::spawn_blocking(move || client.remove_installation(&manager));

    match handle.await {
        Ok(result) => {
            if result.is_ok() {
                log_info!("Successfully deleted files for client ID: {}", id);
                update_client_installed_status(id, false, state.clone())?;
            } else {
                log_error!("Failed to delete files for client ID {}: {:?}", id, result);
            }
            result
        }
        Err(e) => {
            log_error!("Task to delete client ID {} failed: {}", id, e);
            Err(format!("Delete task error: {e}"))
        }
    }
}

#[tauri::command]
pub async fn get_client_details(client_id: u32) -> Result<serde_json::Value, String> {
    log_debug!("Fetching details for client ID: {}", client_id);
    let api_url = get_auth_url().await?;
    let url = format!("{api_url}api/client/{client_id}/detailed");
    log_debug!("Requesting client details from URL: {}", url);

    let client = reqwest::Client::new();
    let response = client.get(&url).send().await.map_err(|e| {
        log_error!("Failed to fetch client details from {}: {}", url, e);
        format!("Failed to fetch client details: {e}")
    })?;

    if !response.status().is_success() {
        log_warn!(
            "API returned non-success status ({}) for client details request to {}",
            response.status(),
            url
        );
        return Err(format!("API returned error: {}", response.status()));
    }

    let details: serde_json::Value = response.json().await.map_err(|e| {
        log_error!("Failed to parse client details JSON: {}", e);
        format!("Failed to parse client details: {e}")
    })?;

    log_info!("Successfully fetched details for client ID: {}", client_id);
    Ok(details)
}

#[tauri::command]
pub fn increment_client_counter(
    id: u32,
    counter_type: String,
    state: State<'_, Arc<Mutex<ClientManager>>>,
) -> Result<(), String> {
    if let Some(client) = state
        .lock()
        .map_err(|_| "Failed to acquire lock on client manager".to_string())?
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
    log_info!("Adding new custom client: '{}'", name);
    let mut manager = CUSTOM_CLIENT_MANAGER
        .lock()
        .map_err(|_| "Failed to acquire lock on custom client manager".to_string())?;

    let version_enum = Version::from_string(&version);
    let path_buf = PathBuf::from(file_path);

    let custom_client = CustomClient::new(0, name, version_enum, filename, path_buf, main_class);

    log_debug!("New custom client details: {:?}", custom_client);
    manager.add_client(custom_client)
}

#[tauri::command]
pub fn remove_custom_client(id: u32) -> Result<(), String> {
    log_info!("Removing custom client with ID: {}", id);
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
    log_info!("Updating custom client with ID: {}", id);
    let mut manager = CUSTOM_CLIENT_MANAGER
        .lock()
        .map_err(|_| "Failed to acquire lock on custom client manager".to_string())?;

    let version_enum = version.map(|v| Version::from_string(&v));

    let updates = CustomClientUpdate {
        name,
        version: version_enum,
        main_class,
    };

    log_debug!("Applying updates to custom client ID {}: {:?}", id, updates);
    manager.update_client(id, updates)
}

#[tauri::command]
pub async fn launch_custom_client(
    id: u32,
    user_token: String,
    app_handle: AppHandle,
    state: State<'_, Arc<Mutex<ClientManager>>>,
) -> Result<(), String> {
    log_info!("Attempting to launch custom client with ID: {}", id);
    let custom_client = {
        let mut manager = CUSTOM_CLIENT_MANAGER
            .lock()
            .map_err(|_| "Failed to acquire lock on custom client manager".to_string())?;

        let client = manager
            .get_client_mut(id)
            .ok_or_else(|| "Custom client not found".to_string())?;

        client.launches += 1;
        log_debug!(
            "Incremented launch count for custom client '{}' to {}",
            client.name,
            client.launches
        );
        let client_clone = client.clone();
        manager.save_to_disk();

        client_clone
    };

    custom_client.validate_file()?;
    log_debug!("Custom client file validated for '{}'", custom_client.name);

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

    client.run(options, (*state).clone()).await
}

#[tauri::command]
pub async fn get_running_custom_client_ids() -> Vec<u32> {
    let handle = tokio::task::spawn_blocking(|| {
        CustomClient::get_running_custom_clients()
            .iter()
            .map(|client| client.id)
            .collect()
    });

    handle.await.unwrap_or_else(|e| {
        log_error!("Failed to get running custom client IDs: {}", e);
        Vec::new()
    })
}

#[tauri::command]
pub async fn stop_custom_client(id: u32) -> Result<(), String> {
    log_info!("Attempting to stop custom client with ID: {}", id);
    let custom_client = {
        let manager = CUSTOM_CLIENT_MANAGER
            .lock()
            .map_err(|_| "Failed to acquire lock on custom client manager".to_string())?;

        manager
            .get_client(id)
            .cloned()
            .ok_or_else(|| "Custom client not found".to_string())?
    };
    log_debug!("Found custom client '{}' to stop", custom_client.name);

    let client_clone = custom_client.clone();
    let handle = tokio::task::spawn_blocking(move || client_clone.stop());

    handle
        .await
        .map_err(|e| format!("Stop custom client task error: {e}"))?
}
