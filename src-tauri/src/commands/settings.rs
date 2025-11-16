use crate::core::storage::accounts::{Account, ACCOUNT_MANAGER};
use crate::core::storage::common::JsonStorage;
use crate::core::storage::favorites::FAVORITE_MANAGER;
use crate::core::storage::flags::{Flags, FLAGS_MANAGER};
use crate::core::storage::settings::{InputSettings, Settings, SETTINGS};
use crate::core::utils::discord_rpc;
#[cfg(target_os = "windows")]
use crate::core::utils::dpi;
use crate::{log_debug, log_error, log_info};
use sysinfo::System;

#[tauri::command]
pub fn get_settings() -> Settings {
    SETTINGS.lock().unwrap().clone()
}

#[tauri::command]
pub fn get_flags() -> Flags {
    FLAGS_MANAGER.lock().unwrap().clone()
}

#[tauri::command]
pub fn reset_flags() -> Result<(), String> {
    log_info!("Resetting application flags to default");
    let mut flags = FLAGS_MANAGER.lock().unwrap();
    *flags = Flags::default();
    flags.save_to_disk();
    drop(flags);
    Ok(())
}

#[tauri::command]
pub fn save_settings(input_settings: InputSettings) -> Result<(), String> {
    log_info!("Saving application settings");
    let mut current_settings = SETTINGS.lock().unwrap();
    let config_path = current_settings.config_path.clone();

    let old_discord_rpc_enabled = current_settings.discord_rpc_enabled.value;
    let old_dpi_bypass_enabled = current_settings.dpi_bypass.value;

    let discord_rpc_changed = old_discord_rpc_enabled != input_settings.discord_rpc_enabled.value;
    let new_discord_rpc_value = input_settings.discord_rpc_enabled.value;

    #[cfg(target_os = "windows")]
    let dpi_bypass_changed = old_dpi_bypass_enabled != input_settings.dpi_bypass.value;
    #[cfg(target_os = "windows")]
    let new_dpi_bypass_value = input_settings.dpi_bypass.value;

    log_debug!("Applying new settings");
    let input_settings_clone = input_settings.clone();
    let new_settings = Settings::from_input(input_settings_clone, config_path);
    *current_settings = new_settings.clone();

    new_settings.save_to_disk();
    log_info!("Settings saved to disk");

    drop(current_settings);

    if discord_rpc_changed {
        log_info!(
            "Discord RPC setting changed. Toggling RPC to: {}",
            new_discord_rpc_value
        );
        if let Err(e) = discord_rpc::toggle_rpc(new_discord_rpc_value) {
            log_error!("Failed to toggle Discord RPC: {e}");
        }
    }

    #[cfg(target_os = "windows")]
    {
        if dpi_bypass_changed && new_dpi_bypass_value {
            log_info!("DPI bypass enabled. Preparing to download and run package");

            if let Err(e) = dpi::enable_dpi_bypass_async() {
                log_error!("Failed to initiate DPI bypass setup: {e}");
            }
        }
    }

    Ok(())
}

#[tauri::command]
pub fn reset_settings() -> Result<(), String> {
    log_info!("Resetting application settings to default");
    let mut current_settings = SETTINGS.lock().unwrap();
    *current_settings = Settings::default();
    current_settings.config_path = Settings::default().config_path;

    current_settings.save_to_disk();
    log_info!("Default settings saved to disk");
    drop(current_settings);

    Ok(())
}

#[tauri::command]
pub fn mark_disclaimer_shown() -> Result<(), String> {
    log_info!("Marking disclaimer as shown");
    let mut flags = FLAGS_MANAGER.lock().unwrap();
    flags.disclaimer_shown.value = true;
    flags.save_to_disk();
    drop(flags);
    Ok(())
}

#[tauri::command]
pub fn mark_first_run_shown() -> Result<(), String> {
    log_info!("Marking first run as shown");
    let mut flags = FLAGS_MANAGER.lock().unwrap();
    flags.first_run.value = false;
    flags.save_to_disk();
    drop(flags);
    Ok(())
}

#[tauri::command]
pub fn set_optional_telemetry(enabled: bool) -> Result<(), String> {
    log_info!("Setting optional telemetry to: {}", enabled);
    let mut settings = SETTINGS.lock().unwrap();
    settings.optional_telemetry.value = enabled;
    settings.save_to_disk();
    drop(settings);
    Ok(())
}

#[tauri::command]
pub fn get_accounts() -> Vec<Account> {
    ACCOUNT_MANAGER.lock().map_or_else(
        |e| {
            log_error!("Failed to acquire lock on account manager: {}", e);
            Vec::new()
        },
        |account_manager| account_manager.accounts.clone(),
    )
}

#[tauri::command]
pub fn add_account(username: String, tags: Vec<String>) -> Result<String, String> {
    log_info!("Adding new account for user: '{}'", username);
    ACCOUNT_MANAGER.lock().map_or_else(
        |e| {
            log_error!("Failed to acquire lock on account manager: {}", e);
            Err("Failed to acquire lock on account manager".to_string())
        },
        |mut account_manager| {
            let id = account_manager.add_account(username.clone(), tags);
            log_debug!("New account created with ID: {}", id);
            account_manager.save_to_disk();
            log_info!("Account for '{}' saved to disk", username);
            Ok(id)
        },
    )
}

#[tauri::command]
pub fn remove_account(id: String) -> Result<(), String> {
    log_info!("Removing account with ID: {}", id);
    ACCOUNT_MANAGER.lock().map_or_else(
        |e| {
            log_error!("Failed to acquire lock on account manager: {}", e);
            Err("Failed to acquire lock on account manager".to_string())
        },
        |mut account_manager| {
            if account_manager.remove_account(&id) {
                account_manager.save_to_disk();
                log_info!("Account ID {} removed and saved to disk", id);
                Ok(())
            } else {
                log_error!("Account with ID {} not found for removal", id);
                Err("Account not found".to_string())
            }
        },
    )
}

#[tauri::command]
pub fn set_active_account(id: String) -> Result<(), String> {
    log_info!("Setting active account to ID: {}", id);
    ACCOUNT_MANAGER.lock().map_or_else(
        |e| {
            log_error!("Failed to acquire lock on account manager: {}", e);
            Err("Failed to acquire lock on account manager".to_string())
        },
        |mut account_manager| {
            if account_manager.set_active_account(&id) {
                account_manager.save_to_disk();
                log_info!("Active account set to {} and saved to disk", id);
                Ok(())
            } else {
                log_error!("Account with ID {} not found to set as active", id);
                Err("Account not found".to_string())
            }
        },
    )
}

#[tauri::command]
pub fn update_account(
    id: String,
    username: Option<String>,
    tags: Option<Vec<String>>,
) -> Result<(), String> {
    log_info!("Updating account with ID: {}", id);
    ACCOUNT_MANAGER.lock().map_or_else(
        |e| {
            log_error!("Failed to acquire lock on account manager: {}", e);
            Err("Failed to acquire lock on account manager".to_string())
        },
        |mut account_manager| {
            if account_manager.update_account(&id, username, tags) {
                account_manager.save_to_disk();
                log_info!("Account ID {} updated and saved to disk", id);
                Ok(())
            } else {
                log_error!("Account with ID {} not found for update", id);
                Err("Account not found".to_string())
            }
        },
    )
}

#[tauri::command]
pub fn get_active_account() -> Option<Account> {
    log_debug!("Fetching active account");
    ACCOUNT_MANAGER.lock().map_or_else(
        |e| {
            log_error!("Failed to acquire lock on account manager: {}", e);
            None
        },
        |account_manager| account_manager.get_active_account().cloned(),
    )
}

#[tauri::command]
pub fn get_favorite_clients() -> Result<Vec<u32>, String> {
    FAVORITE_MANAGER.lock().map_or_else(
        |e| {
            log_error!("Failed to acquire lock on favorite manager: {}", e);
            Err("Failed to acquire lock on favorite manager".to_string())
        },
        |favorite_manager| Ok(favorite_manager.favorites.clone()),
    )
}

#[tauri::command]
pub fn add_favorite_client(client_id: u32) -> Result<(), String> {
    log_info!("Adding client ID {} to favorites", client_id);
    FAVORITE_MANAGER.lock().map_or_else(
        |e| {
            log_error!("Failed to acquire lock on favorite manager: {}", e);
            Err("Failed to acquire lock on favorite manager".to_string())
        },
        |mut favorite_manager| {
            favorite_manager.add_favorite(client_id);
            favorite_manager.save_to_disk();
            log_info!("Client ID {} added to favorites and saved", client_id);
            Ok(())
        },
    )
}

#[tauri::command]
pub fn remove_favorite_client(client_id: u32) -> Result<(), String> {
    log_info!("Removing client ID {} from favorites", client_id);
    FAVORITE_MANAGER.lock().map_or_else(
        |e| {
            log_error!("Failed to acquire lock on favorite manager: {}", e);
            Err("Failed to acquire lock on favorite manager".to_string())
        },
        |mut favorite_manager| {
            favorite_manager.remove_favorite(client_id);
            favorite_manager.save_to_disk();
            log_info!("Client ID {} removed from favorites and saved", client_id);
            Ok(())
        },
    )
}

#[tauri::command]
pub fn is_client_favorite(client_id: u32) -> Result<bool, String> {
    log_debug!("Checking if client ID {} is a favorite", client_id);
    FAVORITE_MANAGER.lock().map_or_else(
        |e| {
            log_error!("Failed to acquire lock on favorite manager: {}", e);
            Err("Failed to acquire lock on favorite manager".to_string())
        },
        |favorite_manager| Ok(favorite_manager.is_favorite(client_id)),
    )
}

#[tauri::command]
pub fn mark_telemetry_consent_shown() -> Result<(), String> {
    log_info!("Marking telemetry consent as shown");
    let mut flags = FLAGS_MANAGER.lock().unwrap();
    flags.telemetry_consent_shown.value = true;
    flags.save_to_disk();
    drop(flags);
    Ok(())
}

#[tauri::command]
pub fn is_telemetry_consent_shown() -> Result<bool, String> {
    log_debug!("Checking if telemetry consent has been shown");
    let flags = FLAGS_MANAGER.lock().unwrap();
    Ok(flags.telemetry_consent_shown.value)
}

#[tauri::command]
pub fn set_custom_clients_display(display: String) -> Result<(), String> {
    log_info!("Setting custom clients display to: {}", display);
    let mut flags = FLAGS_MANAGER.lock().unwrap();
    flags.set_custom_clients_display(display);
    flags.save_to_disk();
    drop(flags);
    Ok(())
}

#[tauri::command]
pub fn get_system_memory() -> Result<u64, String> {
    let mut sys = System::new_all();
    sys.refresh_all();

    let total_memory = sys.total_memory();
    Ok(total_memory)
}
