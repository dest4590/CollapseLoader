use super::{get_client_by_id, refresh_tray_menu_after_client_change, with_client_manager};
use crate::core::clients::client::{Client, LaunchOptions, CLIENT_LOGS};
use crate::core::clients::internal::agent_overlay::AgentOverlayManager;
use crate::core::clients::manager::ClientManager;
use crate::core::network::servers::{ServerConnectivityStatus, SERVERS};
use crate::core::storage::data::DATA;
use crate::core::utils::{
    discord_rpc,
    globals::SKIP_AGENT_OVERLAY_VERIFICATION,
    hashing::calculate_md5_hash,
    helpers::{emit_to_main_window, hide_main_window},
    logging,
};
use crate::AppState;
use crate::{log_debug, log_error, log_info, log_warn};

use std::fs::File;
use std::io::Read;
use zip::ZipArchive;

use tauri::{AppHandle, State};

#[tauri::command]
pub fn get_app_logs() -> Vec<String> {
    logging::APP_LOGS
        .lock()
        .map(|logs| logs.clone())
        .unwrap_or_default()
        .into()
}

#[tauri::command]
pub async fn initialize_api(state: State<'_, AppState>) -> Result<(), String> {
    let clients = ClientManager::fetch_clients().await.map_err(|e| {
        log_error!("Failed to fetch clients: {}", e);
        e.to_string()
    })?;

    if clients.is_empty() {
        log_warn!("Fetched client list is empty - this may indicate an API or network issue");
        return Err("Fetched client list is empty".to_string());
    }

    {
        let mut manager = state
            .clients
            .manager
            .lock()
            .map_err(|_| "Failed to lock state".to_string())?;

        manager.clients = clients;
    }

    let sync_enabled = state.settings().sync_client_settings.value;

    if sync_enabled {
        if let Err(e) = crate::core::storage::data::DATA
            .sync_all_installed_clients()
            .await
        {
            log_warn!("Failed to sync all clients on startup: {}", e);
        }
    }

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
pub async fn get_server_connectivity_status() -> ServerConnectivityStatus {
    let servers = &SERVERS;
    servers.wait_for_initial_check().await;
    servers.connectivity_status.lock().unwrap().clone()
}

#[tauri::command]
pub fn get_clients(state: State<'_, AppState>) -> Vec<Client> {
    state
        .clients
        .manager
        .lock()
        .ok()
        .map(|manager| manager.clients.clone())
        .unwrap_or_default()
}

async fn verify_client_hash(
    client: &Client,
    jar_path: &std::path::Path,
    app_handle: &AppHandle,
    state: &State<'_, AppState>,
) -> Result<(), String> {
    let hash_verify_enabled = state.settings().hash_verify.value;

    if !hash_verify_enabled {
        log_debug!(
            "Hash verification disabled, skipping verification for client {}",
            client.name
        );
        return Ok(());
    }

    log_info!("Hash verification is enabled for client '{}'", client.name);
    emit_to_main_window(
        app_handle,
        "client-hash-verification-start",
        &serde_json::json!({ "id": client.id, "name": client.name }),
    );

    log_info!(
        "Verifying MD5 hash for client {} before launch",
        client.name
    );

    let current_hash = calculate_md5_hash(jar_path)?;
    if current_hash == client.md5_hash {
        log_info!(
            "MD5 hash verification successful for client {}",
            client.name
        );
        emit_to_main_window(
            app_handle,
            "client-hash-verification-done",
            &serde_json::json!({ "id": client.id, "name": client.name }),
        );
        return Ok(());
    }

    log_warn!(
        "Hash mismatch for client {}. Expected: {}, Got: {}. Redownloading...",
        client.name,
        client.md5_hash,
        current_hash
    );

    emit_to_main_window(
        app_handle,
        "client-hash-verification-failed",
        &serde_json::json!({
            "id": client.id,
            "name": client.name,
            "expected_hash": client.md5_hash,
            "actual_hash": current_hash
        }),
    );

    let _ = std::fs::remove_file(jar_path);
    update_client_installed_status(client.id, false, state.clone())?;

    log_info!("Redownloading client: {} (ID: {})", client.name, client.id);
    client
        .download(&state.clients.manager)
        .await
        .map_err(|e| {
            if e.contains("Hash verification failed") {
                format!("Hash verification failed for {}: The downloaded file is corrupted. Please try downloading again.", client.name)
            } else {
                format!("Failed to redownload client {}: {}", client.name, e)
            }
        })?;

    emit_to_main_window(
        app_handle,
        "client-redownload-complete",
        &serde_json::json!({ "id": client.id, "name": client.name }),
    );

    log_info!(
        "Client {} redownloaded and verified successfully",
        client.name
    );
    Ok(())
}

async fn ensure_agent_overlay() -> Result<(), String> {
    match AgentOverlayManager::verify_agent_overlay_files().await {
        Ok(true) => Ok(()),
        Ok(false) => {
            if !*SKIP_AGENT_OVERLAY_VERIFICATION {
                log_warn!("Agent/overlay files verification failed, attempting to download...");
                AgentOverlayManager::download_agent_overlay_files()
                    .await
                    .map_err(|e| format!("Failed to download required agent/overlay files: {e}"))
            } else {
                log_debug!("Agent/overlay files verification failed, but skipping download due to SKIP_AGENT_OVERLAY_VERIFICATION being enabled.");
                Ok(())
            }
        }
        Err(e) => {
            log_error!("Error verifying agent/overlay files: {}", e);
            Ok(())
        }
    }
}

#[tauri::command]
pub async fn launch_client(
    id: u32,
    user_token: String,
    app_handle: AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let client = get_client_by_id(id, &state.clients.manager)?;
    let (_, jar_path) = client.get_launch_paths()?;

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

    log_info!(
        "Launching '{}' (ID: {}, Play Count: {})...",
        client.name,
        id,
        client.launches
    );

    verify_client_hash(&client, &jar_path, &app_handle, &state).await?;
    ensure_agent_overlay().await?;

    let sync_enabled = state.settings().sync_client_settings.value;

    if sync_enabled {
        let client_base = std::path::Path::new(&client.filename)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or(&client.name)
            .to_string();
        if let Err(e) = crate::core::storage::data::DATA
            .ensure_client_synced(&client_base)
            .await
        {
            if !e.contains("5") {
                log_warn!("Failed to sync client {} before launch: {}", client_base, e);
            }
        }
    }

    let minimize_on_launch = state.settings().minimize_to_tray_on_launch.value;

    if minimize_on_launch {
        hide_main_window(&app_handle);
    }

    let options = LaunchOptions::new(app_handle.clone(), user_token, false);
    client.run(options, state.clients.manager.clone()).await
}

#[tauri::command]
pub async fn get_running_client_ids(state: State<'_, AppState>) -> Result<Vec<u32>, String> {
    let manager = state.clients.manager.clone();
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
pub async fn stop_client(id: u32, state: State<'_, AppState>) -> Result<(), String> {
    log_info!("Attempting to stop client with ID: {}", id);
    let client = get_client_by_id(id, &state.clients.manager)?;
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
    state: State<'_, AppState>,
) -> Result<(), String> {
    let client = get_client_by_id(id, &state.clients.manager)?;
    let sync_enabled = state.settings().sync_client_settings.value;
    let state_clone = state.clients.manager.clone();
    let client_clone = client.clone();
    let client_download = async move {
        client_clone.download(&state_clone).await.map_err(|e| {
            if e.contains("Hash verification failed") {
                let _ = update_client_installed_status(id, false, state.clone());
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

    tokio::try_join!(client_download, requirements_download)?;

    if sync_enabled {
        let client_base = crate::core::storage::data::Data::get_filename(&client.filename);
        if let Err(e) = crate::core::storage::data::DATA
            .ensure_client_synced(&client_base)
            .await
        {
            if e.contains("5") {
                log_warn!(
                    "Failed to sync client {} after download: {}",
                    client_base,
                    e
                );
            }
        }
    }

    Ok(())
}

#[tauri::command]
pub async fn reinstall_client(
    id: u32,
    app_handle: AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    log_info!("Starting reinstall for client ID: {}", id);
    let client = get_client_by_id(id, &state.clients.manager)?;
    log_debug!("Found client '{}' for reinstall", client.name);

    let client_clone = client.clone();
    let manager = state.clients.manager.clone();
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

    let download_result = client
        .download(&state.clients.manager)
        .await
        .map_err(|e| {
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

    let result = client.download_requirements(&app_handle).await;

    if download_result.is_ok() && result.is_ok() {
        log_info!(
            "Client '{}' successfully installed with all requirements",
            client.name
        );
    }

    result
}

#[tauri::command]
pub fn open_client_folder(id: u32, state: State<'_, AppState>) -> Result<(), String> {
    log_info!("Attempting to open folder for client ID: {}", id);
    let client = get_client_by_id(id, &state.clients.manager)?;
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
    state: State<'_, AppState>,
) -> Result<(), String> {
    let result = with_client_manager(&state, |manager| {
        if let Some(client) = manager.clients.iter_mut().find(|c| c.id == id) {
            client.meta.installed = installed;
            Ok(())
        } else {
            Err("Client not found".to_string())
        }
    });

    refresh_tray_menu_after_client_change(state, result)
}

#[tauri::command]
pub async fn delete_client(id: u32, state: State<'_, AppState>) -> Result<(), String> {
    let client = get_client_by_id(id, &state.clients.manager)?;
    let manager = state.clients.manager.clone();
    let handle = tokio::task::spawn_blocking(move || client.remove_installation(&manager));

    match handle.await {
        Ok(result) => {
            if result.is_ok() {
                update_client_installed_status(id, false, state.clone())?;
            }
            result
        }
        Err(e) => Err(format!("Delete task error: {e}")),
    }
}

#[tauri::command]
pub fn increment_client_counter(
    id: u32,
    counter_type: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let result = with_client_manager(&state, |manager| {
        if let Some(client) = manager.clients.iter_mut().find(|c| c.id == id) {
            match counter_type.as_str() {
                "download" => {
                    client.downloads += 1;
                }
                "launch" => {
                    client.launches += 1;
                }
                _ => {
                    return Err(format!("Invalid counter type: {counter_type}"));
                }
            }
            Ok(())
        } else {
            Err("Client not found".to_string())
        }
    });

    refresh_tray_menu_after_client_change(state, result)
}

#[tauri::command]
pub fn detect_main_class(file_path: String) -> Result<String, String> {
    log_info!("Attempting to detect main class from: {}", file_path);

    let file = File::open(&file_path).map_err(|e| format!("Failed to open file: {}", e))?;
    let mut archive = ZipArchive::new(file).map_err(|e| format!("Failed to read jar: {}", e))?;

    let mut manifest = archive
        .by_name("META-INF/MANIFEST.MF")
        .map_err(|_| "MANIFEST.MF not found in jar".to_string())?;

    let mut content = String::new();
    manifest
        .read_to_string(&mut content)
        .map_err(|e| format!("Failed to read manifest: {}", e))?;

    for line in content.lines() {
        if line.starts_with("Main-Class:") {
            let main_class = line.replace("Main-Class:", "").trim().to_string();
            log_info!("Detected main class: {}", main_class);
            return Ok(main_class);
        }
    }

    log_warn!("Main-Class attribute not found in manifest");
    Err("Main-Class attribute not found in manifest".to_string())
}
