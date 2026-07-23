use super::with_custom_client_manager;
use crate::core::clients::client::{ClientType, LaunchOptions};
use crate::core::clients::custom_clients::CustomClient;
use crate::core::storage::common::JsonStorage;
use crate::core::storage::custom_clients::CustomClientUpdate;
use crate::core::utils::helpers::{emit_to_main_window, hide_main_window};
use crate::AppState;
use crate::{log_debug, log_error, log_info, log_warn};
use std::path::PathBuf;
use tauri::{AppHandle, State};

#[tauri::command]
pub fn get_custom_clients(state: State<'_, AppState>) -> Vec<CustomClient> {
    state.custom_clients.lock().clients.clone()
}

#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub async fn add_custom_client(
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
    let sync_needed =
        with_custom_client_manager(&state, |manager| manager.add_client(custom_client))?;

    if let Some(client_base) = sync_needed {
        if let Err(e) = crate::core::storage::data::DATA
            .ensure_client_synced(&client_base)
            .await
        {
            log_warn!("Failed to ensure client sync for custom client: {}", e);
        }
    }

    Ok(())
}

#[tauri::command]
pub fn remove_custom_client(id: u32, state: State<'_, AppState>) -> Result<(), String> {
    log_info!("Removing custom client with ID: {}", id);
    with_custom_client_manager(&state, |manager| manager.remove_client(id))
}

#[tauri::command]
#[allow(clippy::too_many_arguments)]
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

    let minimize_on_launch = state.settings().minimize_to_tray_on_launch.value;

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
