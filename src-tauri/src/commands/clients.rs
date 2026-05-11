use crate::AppState;
use core::clients::{
    client::{Client, CLIENT_LOGS},
    manager::ClientManager,
};
use tauri::{AppHandle, State};

use crate::commands::utils::refresh_tray_menu;
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
    utils::{discord_rpc, hashing::calculate_md5_hash, helpers::hide_main_window, logging},
};
use crate::{
    core::{self, storage::data::DATA},
    log_debug, log_error, log_info, log_warn,
};

use serde::Serialize;
use sysinfo::{Pid, ProcessesToUpdate, System};

use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use zip::ZipArchive;

pub(crate) fn get_client_by_id(
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
        .ok_or_else(|| format!("Client with ID {id} not found"))
}

#[derive(Debug, Clone, Serialize)]
pub struct ClientRamUsage {
    pub client_id: u32,
    pub is_running: bool,
    pub process_count: usize,
    pub pids: Vec<u32>,
    pub total_memory_bytes: u64,
    pub total_memory_mib: f64,
    pub system_total_memory_bytes: u64,
    pub system_total_memory_mib: f64,
    pub system_memory_percent: f64,
}

fn collect_client_ram_usage(client: &Client) -> ClientRamUsage {
    let pids = crate::core::utils::process::find_processes_by_filename(&client.filename)
        .into_iter()
        .filter_map(|pid| pid.parse::<u32>().ok())
        .collect::<Vec<_>>();

    let mut system = System::new_all();
    system.refresh_memory();
    let _ = system.refresh_processes(ProcessesToUpdate::All, true);

    let total_memory_bytes = pids
        .iter()
        .filter_map(|pid| system.process(Pid::from_u32(*pid)))
        .map(|process| process.memory())
        .sum::<u64>();

    let system_total_memory_bytes = system.total_memory();
    let total_memory_mib = total_memory_bytes as f64 / 1024.0 / 1024.0;
    let system_total_memory_mib = system_total_memory_bytes as f64 / 1024.0 / 1024.0;
    let system_memory_percent = if system_total_memory_bytes > 0 {
        (total_memory_bytes as f64 / system_total_memory_bytes as f64) * 100.0
    } else {
        0.0
    };

    ClientRamUsage {
        client_id: client.id,
        is_running: !pids.is_empty(),
        process_count: pids.len(),
        pids,
        total_memory_bytes,
        total_memory_mib,
        system_total_memory_bytes,
        system_total_memory_mib,
        system_memory_percent,
    }
}

fn with_client_manager<R>(
    state: &State<'_, AppState>,
    operation: impl FnOnce(&mut ClientManager) -> Result<R, String>,
) -> Result<R, String> {
    let mut manager = state
        .clients
        .manager
        .lock()
        .map_err(|_| "Failed to acquire lock on client manager".to_string())?;

    operation(&mut manager)
}

fn with_custom_client_manager<R>(
    state: &State<'_, AppState>,
    operation: impl FnOnce(
        &mut crate::core::storage::custom_clients::CustomClientManager,
    ) -> Result<R, String>,
) -> Result<R, String> {
    let mut manager = state.custom_clients.lock();
    operation(&mut manager)
}

fn refresh_tray_menu_after_client_change(
    state: State<'_, AppState>,
    result: Result<(), String>,
) -> Result<(), String> {
    if result.is_ok() {
        refresh_tray_menu(state);
    }
    result
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

    let sync_enabled = crate::core::storage::settings::SETTINGS
        .lock()
        .map(|s| s.sync_client_settings.value)
        .unwrap_or(false);

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
    let hash_verify_enabled = SETTINGS.lock().map(|s| s.hash_verify.value).unwrap_or(true);

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

    let sync_enabled = SETTINGS
        .lock()
        .map(|s| s.sync_client_settings.value)
        .unwrap_or(false);

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

    let minimize_on_launch = SETTINGS
        .lock()
        .map(|s| s.minimize_to_tray_on_launch.value)
        .unwrap_or(false);

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

    let sync_enabled = crate::core::storage::settings::SETTINGS
        .lock()
        .map(|s| s.sync_client_settings.value)
        .unwrap_or(false);

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
pub async fn get_client_ram_usage(
    id: u32,
    state: State<'_, AppState>,
) -> Result<ClientRamUsage, String> {
    let client = get_client_by_id(id, &state.clients.manager)?;

    let client_clone = client.clone();
    tokio::task::spawn_blocking(move || collect_client_ram_usage(&client_clone))
        .await
        .map_err(|e| format!("Failed to get client RAM usage: {e}"))
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
    let path_buf = PathBuf::from(file_path);
    let mut custom_client = CustomClient::new(0, name, version, filename, path_buf, main_class);
    custom_client.java_path = java_path;
    custom_client.java_args = java_args;
    custom_client.client_type = client_type;

    log_debug!("New custom client details: {:?}", custom_client);
    with_custom_client_manager(&state, |manager| manager.add_client(custom_client))
}

#[tauri::command]
pub fn remove_custom_client(id: u32, state: State<'_, AppState>) -> Result<(), String> {
    log_info!("Removing custom client with ID: {}", id);
    with_custom_client_manager(&state, |manager| manager.remove_client(id))
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
    let updates = CustomClientUpdate {
        name,
        version,
        main_class,
        java_path,
        java_args,
        client_type,
    };

    log_debug!("Applying updates to custom client ID {}: {:?}", id, updates);
    with_custom_client_manager(&state, |manager| manager.update_client(id, updates))
}

#[tauri::command]
pub async fn launch_custom_client(
    id: u32,
    user_token: String,
    app_handle: AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    log_info!("Attempting to launch custom client with ID: {}", id);
    let custom_client = with_custom_client_manager(&state, |manager| {
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

        Ok(client_clone)
    })?;

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
        hide_main_window(&app_handle);
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
pub async fn uninstall_mod(
    id: u32,
    filename: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let client = get_client_by_id(id, &state.clients.manager)?;

    let mods_folder = match client.client_type {
        ClientType::Fabric | ClientType::Forge | ClientType::Default => {
            let (folder, _) = client.get_launch_paths().map_err(|e| e.to_string())?;
            folder.join("mods")
        }
    };

    let target_file = mods_folder.join(filename);

    if !target_file.exists() {
        return Err("Mod file does not exist".to_string());
    }

    tokio::fs::remove_file(target_file)
        .await
        .map_err(|e| format!("Failed to delete mod file: {}", e))?;

    Ok(())
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

    let folder = custom_client
        .file_path
        .parent()
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

    let mods_folder = custom_client
        .file_path
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

    let mods_folder = custom_client
        .file_path
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

    let mods_folder = custom_client
        .file_path
        .parent()
        .ok_or_else(|| "Cannot determine client folder".to_string())?
        .join("mods");

    let target_file = mods_folder.join(filename);

    if !target_file.exists() {
        return Err("Mod file does not exist".to_string());
    }

    tokio::fs::remove_file(target_file)
        .await
        .map_err(|e| format!("Failed to delete mod file: {}", e))?;

    Ok(())
}

#[tauri::command]
pub fn create_client_shortcut(
    id: u32,
    custom_id: Option<u32>,
    is_custom: bool,
    shortcut_name: Option<String>,
    icon_path: Option<String>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let (client_name, _client_path) = if is_custom {
        let cid = custom_id.unwrap_or(id);
        let manager = state.custom_clients.lock();
        let c = manager
            .get_client(cid)
            .cloned()
            .ok_or_else(|| "Custom client not found".to_string())?;
        drop(manager);
        (c.name.clone(), c.file_path.clone())
    } else {
        let c = get_client_by_id(id, &state.clients.manager)?;
        let (folder, _) = c.get_launch_paths()?;
        (c.name.clone(), folder)
    };

    let display_name = shortcut_name
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| client_name.clone());

    let exe_path = std::env::current_exe()
        .map_err(|e| format!("Failed to get executable path: {e}"))?;

    log_info!(
        "Creating shortcut '{}' for client '{}' (exe: {})",
        display_name,
        client_name,
        exe_path.display()
    );

    create_shortcut_platform(&display_name, &exe_path, id, custom_id, is_custom, icon_path.as_deref())
}

#[cfg(target_os = "windows")]
fn create_shortcut_platform(
    display_name: &str,
    exe_path: &std::path::Path,
    id: u32,
    custom_id: Option<u32>,
    is_custom: bool,
    icon_path: Option<&str>,
) -> Result<(), String> {
    let client_id = if is_custom { custom_id.unwrap_or(id) } else { id };
    let args = format!("collapseloader://launch-client/{}", client_id);

    let exe_str = exe_path.to_string_lossy().into_owned();
    let icon_str = icon_path.unwrap_or(&exe_str).to_string();
    let safe_name = sanitize_filename(display_name);

    let script = format!(
        r#"
$desktop = [Environment]::GetFolderPath('Desktop');
$lnk = Join-Path $desktop '{safe_name}.lnk';
$exe = '{exe}';
$ico = '{ico}';
$ws = New-Object -ComObject WScript.Shell;
$s = $ws.CreateShortcut($lnk);
$s.TargetPath = $exe;
$s.Arguments = '{link_args}';
$s.Description = 'Launch {desc} via CollapseLoader';
$s.IconLocation = $ico;
$s.Save();
Write-Output $lnk
"#,
        safe_name = safe_name.replace('\'', "''"),
        exe = exe_str.replace('\'', "''"),
        ico = icon_str.replace('\'', "''"),
        link_args = args,
        desc = display_name.replace('\'', "''"),
    );

    let output = std::process::Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", &script])
        .output()
        .map_err(|e| format!("Failed to run PowerShell: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("PowerShell shortcut creation failed: {stderr}"));
    }

    let lnk_out = String::from_utf8_lossy(&output.stdout).trim().to_string();
    log_info!("Shortcut created at: {}", lnk_out);
    Ok(())
}

#[cfg(target_os = "linux")]
fn create_shortcut_platform(
    display_name: &str,
    exe_path: &std::path::Path,
    id: u32,
    custom_id: Option<u32>,
    is_custom: bool,
    icon_path: Option<&str>,
) -> Result<(), String> {
    let client_id = if is_custom { custom_id.unwrap_or(id) } else { id };
    let deep_link = format!("collapseloader://launch-client/{}", client_id);

    let home = std::env::var("HOME").map_err(|_| "Cannot find HOME".to_string())?;
    let desktop = std::path::PathBuf::from(&home).join("Desktop");

    let target_dir = if desktop.exists() { desktop } else { std::path::PathBuf::from(&home) };
    let desktop_file = target_dir.join(format!("{}.desktop", sanitize_filename(display_name)));

    let icon_line = if let Some(ip) = icon_path {
        format!("Icon={}", ip)
    } else {
        format!("Icon={}", exe_path.to_string_lossy())
    };

    let content = format!(
        "[Desktop Entry]\nVersion=1.0\nType=Application\nName={name}\nExec={exe} {link}\n{icon}\nTerminal=false\nComment=Launch {name} via CollapseLoader\n",
        name = display_name,
        exe = exe_path.to_string_lossy(),
        link = deep_link,
        icon = icon_line,
    );

    std::fs::write(&desktop_file, &content)
        .map_err(|e| format!("Failed to write .desktop file: {e}"))?;

    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(&desktop_file, std::fs::Permissions::from_mode(0o755))
        .map_err(|e| format!("Failed to set permissions: {e}"))?;

    log_info!("Desktop shortcut created at: {}", desktop_file.display());
    Ok(())
}

#[cfg(target_os = "macos")]
fn create_shortcut_platform(
    display_name: &str,
    exe_path: &std::path::Path,
    id: u32,
    custom_id: Option<u32>,
    is_custom: bool,
    _icon_path: Option<&str>,
) -> Result<(), String> {
    let client_id = if is_custom { custom_id.unwrap_or(id) } else { id };
    let deep_link = format!("collapseloader://launch-client/{}", client_id);

    let home = std::env::var("HOME").map_err(|_| "Cannot find HOME".to_string())?;
    let desktop = std::path::PathBuf::from(&home).join("Desktop");
    let target_dir = if desktop.exists() { desktop } else { std::path::PathBuf::from(&home) };

    let app_bundle = target_dir.join(format!("{}.app", sanitize_filename(display_name)));
    let contents = app_bundle.join("Contents");
    let macos_dir = contents.join("MacOS");

    std::fs::create_dir_all(&macos_dir)
        .map_err(|e| format!("Failed to create .app bundle: {e}"))?;

    let plist = format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
         <!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">\n\
         <plist version=\"1.0\"><dict>\n\
         <key>CFBundleName</key><string>{name}</string>\n\
         <key>CFBundleExecutable</key><string>launch</string>\n\
         <key>CFBundleIdentifier</key><string>com.collapseloader.shortcut.{id}</string>\n\
         </dict></plist>\n",
        name = display_name,
        id = client_id,
    );
    std::fs::write(contents.join("Info.plist"), plist)
        .map_err(|e| format!("Failed to write Info.plist: {e}"))?;

    let script = format!(
        "#!/bin/sh\nopen '{}'\n",
        deep_link
    );
    let script_path = macos_dir.join("launch");
    std::fs::write(&script_path, script)
        .map_err(|e| format!("Failed to write launcher script: {e}"))?;

    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(&script_path, std::fs::Permissions::from_mode(0o755))
        .map_err(|e| format!("Failed to set permissions: {e}"))?;

    log_info!("macOS app shortcut created at: {}", app_bundle.display());
    Ok(())
}

fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            c => c,
        })
        .collect()
}
