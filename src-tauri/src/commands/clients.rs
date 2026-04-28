use crate::AppState;
use core::clients::{
    client::{Client, CLIENT_LOGS},
    manager::ClientManager,
};
use tauri::{AppHandle, Manager, State};

use crate::core::{
    clients::client::ClientType,
    network::servers::{ServerConnectivityStatus, SERVERS},
    utils::helpers::emit_to_main_window,
};
use crate::core::{
    clients::custom_clients::CustomClient, storage::custom_clients::CustomClientUpdate,
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
    core::{self, storage::data::DATA},
    log_debug, log_error, log_info, log_warn,
};

use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use zip::ZipArchive;

fn get_client_by_id(
    id: u32,
    manager: &std::sync::Arc<std::sync::Mutex<ClientManager>>,
) -> Result<Client, String> {
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

    let hash_verify_enabled = {
        let settings = SETTINGS
            .lock()
            .map_err(|_| "Failed to access settings".to_string())?;
        settings.hash_verify.value
    };

    log_info!(
        "Launching '{}' (ID: {}, Play Count: {})...",
        client.name,
        id,
        client.launches
    );

    log_debug!(
        "Resolution: Path='{}', HashCheck={}, OverlayCheck={}",
        jar_path.display(),
        if hash_verify_enabled {
            "Enabled"
        } else {
            "Disabled"
        },
        if *SKIP_AGENT_OVERLAY_VERIFICATION {
            "Skip"
        } else {
            "Run"
        }
    );

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
        Ok(true) => {}
        Ok(false) => {
            if !*SKIP_AGENT_OVERLAY_VERIFICATION {
                log_warn!("Agent/overlay files verification failed, attempting to download...");
                AgentOverlayManager::download_agent_overlay_files()
                    .await
                    .map_err(|e| format!("Failed to download required agent/overlay files: {e}"))?;
            } else {
                log_debug!("Agent/overlay files verification failed, but skipping download due to SKIP_AGENT_OVERLAY_VERIFICATION being enabled.");
            }
        }
        Err(e) => {
            log_error!("Error verifying agent/overlay files: {}", e);
        }
    }

    let options = LaunchOptions::new(app_handle.clone(), user_token.clone(), false);

    let minimize_on_launch = {
        let settings = SETTINGS.lock().unwrap();
        settings.minimize_to_tray_on_launch.value
    };

    if minimize_on_launch {
        if let Some(window) = app_handle.get_webview_window("main") {
            let _ = window.hide();
        }
    }

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

    log_debug!("Sent client download analytics for ID: {}", id);

    tokio::try_join!(client_download, requirements_download)?;
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
    if let Some(client) = state
        .clients
        .manager
        .lock()
        .map_err(|_| "Failed to acquire lock on client manager".to_string())?
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
    if let Some(client) = state
        .clients
        .manager
        .lock()
        .map_err(|_| "Failed to acquire lock on client manager".to_string())?
        .clients
        .iter_mut()
        .find(|c| c.id == id)
    {
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

#[tauri::command]
pub fn get_custom_clients(state: State<'_, AppState>) -> Vec<CustomClient> {
    state.custom_clients.lock().clients.clone()
}

#[tauri::command]
pub fn add_custom_client(
    name: String,
    version: String,
    filename: String,
    file_path: String,
    main_class: String,
    java_path: Option<String>,
    java_args: Option<String>,
    client_type: ClientType,
    state: State<'_, AppState>,
) -> Result<(), String> {
    log_info!("Adding new custom client: '{}'", name);
    let mut manager = state.custom_clients.lock();

    let path_buf = PathBuf::from(file_path);
    let mut custom_client = CustomClient::new(0, name, version, filename, path_buf, main_class);
    custom_client.java_path = java_path;
    custom_client.java_args = java_args;
    custom_client.client_type = client_type;

    log_debug!("New custom client details: {:?}", custom_client);
    manager.add_client(custom_client)
}

#[tauri::command]
pub fn remove_custom_client(id: u32, state: State<'_, AppState>) -> Result<(), String> {
    log_info!("Removing custom client with ID: {}", id);
    let mut manager = state.custom_clients.lock();

    manager.remove_client(id)
}

#[tauri::command]
pub fn update_custom_client(
    id: u32,
    name: Option<String>,
    version: Option<String>,
    main_class: Option<String>,
    java_path: Option<String>,
    java_args: Option<String>,
    client_type: Option<ClientType>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    log_info!("Updating custom client with ID: {}", id);
    let mut manager = state.custom_clients.lock();

    let updates = CustomClientUpdate {
        name,
        version,
        main_class,
        java_path,
        java_args,
        client_type,
    };

    log_debug!("Applying updates to custom client ID {}: {:?}", id, updates);
    manager.update_client(id, updates)
}

#[tauri::command]
pub async fn launch_custom_client(
    id: u32,
    user_token: String,
    app_handle: AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    log_info!("Attempting to launch custom client with ID: {}", id);
    let custom_client = {
        let mut manager = state.custom_clients.lock();

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

    let minimize_on_launch = {
        let settings = SETTINGS.lock().unwrap();
        settings.minimize_to_tray_on_launch.value
    };

    if minimize_on_launch {
        if let Some(window) = app_handle.get_webview_window("main") {
            let _ = window.hide();
        }
    }

    client.run(options, state.clients.manager.clone()).await
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

    let client = get_client_by_id(id, &state.clients.manager)?;

    let mods_folder_relative = match client.client_type {
        ClientType::Fabric | ClientType::Forge | ClientType::Default => {
            let (folder, _) = client.get_launch_paths().map_err(|e| e.to_string())?;
            let root = DATA.root_dir.lock().unwrap().clone();
            let relative = folder
                .strip_prefix(&root)
                .map_err(|_| "Client folder is outside of root directory".to_string())?;
            relative.join("mods")
        }
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
    let client = get_client_by_id(id, &state.clients.manager)?;

    let mods_folder = match client.client_type {
        ClientType::Fabric | ClientType::Forge | ClientType::Default => {
            let (folder, _) = client.get_launch_paths().map_err(|e| e.to_string())?;
            folder.join("mods")
        }
    };

    if !mods_folder.exists() {
        return Ok(Vec::new());
    }

    let mut mods = Vec::new();
    let mut entries = tokio::fs::read_dir(mods_folder)
        .await
        .map_err(|e| format!("Failed to read mods directory: {}", e))?;

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

#[tauri::command]
pub async fn stop_custom_client(id: u32, state: State<'_, AppState>) -> Result<(), String> {
    log_info!("Attempting to stop custom client with ID: {}", id);
    let custom_client = {
        let manager = state.custom_clients.lock();

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

#[tauri::command]
pub fn open_custom_client_folder(id: u32, state: State<'_, AppState>) -> Result<(), String> {
    log_info!("Attempting to open folder for custom client ID: {}", id);
    let manager = state.custom_clients.lock();
    let custom_client = manager
        .get_client(id)
        .cloned()
        .ok_or_else(|| "Custom client not found".to_string())?;
    drop(manager);

    let folder = custom_client.file_path.parent()
        .ok_or_else(|| "Cannot determine client folder".to_string())?
        .to_path_buf();

    if !folder.exists() {
        return Err("Custom client folder does not exist".to_string());
    }

    let folder_absolute = folder
        .canonicalize()
        .map_err(|e| format!("Failed to get absolute path: {e}"))?;

    opener::open(&folder_absolute).map_err(|e| format!("Failed to open folder: {e}"))
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

    let mods_folder = custom_client.file_path
        .parent()
        .ok_or_else(|| "Cannot determine client folder".to_string())?
        .join("mods");

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

#[tauri::command]
pub async fn install_mod_for_custom_client(
    id: u32,
    url: String,
    filename: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    log_info!("Installing mod for custom client {}: {} from {}", id, filename, url);

    let custom_client = {
        let manager = state.custom_clients.lock();
        manager
            .get_client(id)
            .cloned()
            .ok_or_else(|| "Custom client not found".to_string())?
    };

    let mods_folder = custom_client.file_path
        .parent()
        .ok_or_else(|| "Cannot determine client folder".to_string())?
        .join("mods");

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
        return Err(format!("Download failed with status: {}", response.status()));
    }

    let bytes = response
        .bytes()
        .await
        .map_err(|e| format!("Failed to read mod response: {e}"))?;

    tokio::fs::write(&dest, &bytes)
        .await
        .map_err(|e| format!("Failed to write mod file: {e}"))?;

    log_info!("Successfully installed mod: {} ({} bytes)", filename, bytes.len());
    Ok(())
}
