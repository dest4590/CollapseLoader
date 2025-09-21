use crate::core::storage::accounts::{Account, ACCOUNT_MANAGER};
use crate::core::storage::common::JsonStorage;
use crate::core::storage::favorites::FAVORITE_MANAGER;
use crate::core::storage::flags::{Flags, FLAGS_MANAGER};
use crate::core::storage::settings::{InputSettings, Settings, SETTINGS};
use crate::core::utils::discord_rpc;

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
    let mut flags = FLAGS_MANAGER.lock().unwrap();
    *flags = Flags::default();
    flags.save_to_disk();
    drop(flags);
    Ok(())
}

#[tauri::command]
pub fn save_settings(input_settings: InputSettings) -> Result<(), String> {
    let mut current_settings = SETTINGS.lock().unwrap();
    let config_path = current_settings.config_path.clone();

    let discord_rpc_changed =
        current_settings.discord_rpc_enabled.value != input_settings.discord_rpc_enabled.value;
    let new_discord_rpc_value = input_settings.discord_rpc_enabled.value;

    let new_settings = Settings::from_input(input_settings, config_path);
    *current_settings = new_settings.clone();

    new_settings.save_to_disk();

    drop(current_settings);

    if discord_rpc_changed {
        if let Err(e) = discord_rpc::toggle_rpc(new_discord_rpc_value) {
            eprintln!("Failed to toggle Discord RPC: {e}");
        }
    }

    Ok(())
}

#[tauri::command]
pub fn reset_settings() -> Result<(), String> {
    let mut current_settings = SETTINGS.lock().unwrap();
    *current_settings = Settings::default();
    current_settings.config_path = Settings::default().config_path;

    current_settings.save_to_disk();
    drop(current_settings);

    Ok(())
}

#[tauri::command]
pub fn mark_disclaimer_shown() -> Result<(), String> {
    let mut flags = FLAGS_MANAGER.lock().unwrap();
    flags.disclaimer_shown.value = true;
    flags.save_to_disk();
    drop(flags);
    Ok(())
}

#[tauri::command]
pub fn mark_first_run_shown() -> Result<(), String> {
    let mut flags = FLAGS_MANAGER.lock().unwrap();
    flags.first_run.value = false;
    flags.save_to_disk();
    drop(flags);
    Ok(())
}

#[tauri::command]
pub fn set_optional_telemetry(enabled: bool) -> Result<(), String> {
    let mut settings = SETTINGS.lock().unwrap();
    settings.optional_telemetry.value = enabled;
    settings.save_to_disk();
    drop(settings);
    Ok(())
}

#[tauri::command]
pub fn get_accounts() -> Vec<Account> {
    ACCOUNT_MANAGER.lock().map_or_else(
        |_| Vec::new(),
        |account_manager| account_manager.accounts.clone(),
    )
}

#[tauri::command]
pub fn add_account(username: String, tags: Vec<String>) -> Result<String, String> {
    ACCOUNT_MANAGER.lock().map_or_else(
        |_| Err("Failed to acquire lock on account manager".to_string()),
        |mut account_manager| {
            let id = account_manager.add_account(username, tags);
            account_manager.save_to_disk();
            Ok(id)
        },
    )
}

#[tauri::command]
pub fn remove_account(id: String) -> Result<(), String> {
    ACCOUNT_MANAGER.lock().map_or_else(
        |_| Err("Failed to acquire lock on account manager".to_string()),
        |mut account_manager| {
            if account_manager.remove_account(&id) {
                account_manager.save_to_disk();
                Ok(())
            } else {
                Err("Account not found".to_string())
            }
        },
    )
}

#[tauri::command]
pub fn set_active_account(id: String) -> Result<(), String> {
    ACCOUNT_MANAGER.lock().map_or_else(
        |_| Err("Failed to acquire lock on account manager".to_string()),
        |mut account_manager| {
            if account_manager.set_active_account(&id) {
                account_manager.save_to_disk();
                Ok(())
            } else {
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
    ACCOUNT_MANAGER.lock().map_or_else(
        |_| Err("Failed to acquire lock on account manager".to_string()),
        |mut account_manager| {
            if account_manager.update_account(&id, username, tags) {
                account_manager.save_to_disk();
                Ok(())
            } else {
                Err("Account not found".to_string())
            }
        },
    )
}

#[tauri::command]
pub fn get_active_account() -> Option<Account> {
    ACCOUNT_MANAGER.lock().map_or_else(
        |_| None,
        |account_manager| account_manager.get_active_account().cloned(),
    )
}

#[tauri::command]
pub fn get_favorite_clients() -> Result<Vec<u32>, String> {
    FAVORITE_MANAGER.lock().map_or_else(
        |_| Err("Failed to acquire lock on favorite manager".to_string()),
        |favorite_manager| Ok(favorite_manager.favorites.clone()),
    )
}

#[tauri::command]
pub fn add_favorite_client(client_id: u32) -> Result<(), String> {
    FAVORITE_MANAGER.lock().map_or_else(
        |_| Err("Failed to acquire lock on favorite manager".to_string()),
        |mut favorite_manager| {
            favorite_manager.add_favorite(client_id);
            favorite_manager.save_to_disk();
            Ok(())
        },
    )
}

#[tauri::command]
pub fn remove_favorite_client(client_id: u32) -> Result<(), String> {
    FAVORITE_MANAGER.lock().map_or_else(
        |_| Err("Failed to acquire lock on favorite manager".to_string()),
        |mut favorite_manager| {
            favorite_manager.remove_favorite(client_id);
            favorite_manager.save_to_disk();
            Ok(())
        },
    )
}

#[tauri::command]
pub fn is_client_favorite(client_id: u32) -> Result<bool, String> {
    FAVORITE_MANAGER.lock().map_or_else(
        |_| Err("Failed to acquire lock on favorite manager".to_string()),
        |favorite_manager| Ok(favorite_manager.is_favorite(client_id)),
    )
}

#[tauri::command]
pub fn mark_telemetry_consent_shown() -> Result<(), String> {
    let mut flags = FLAGS_MANAGER.lock().unwrap();
    flags.telemetry_consent_shown.value = true;
    flags.save_to_disk();
    drop(flags);
    Ok(())
}

#[tauri::command]
pub fn is_telemetry_consent_shown() -> Result<bool, String> {
    let flags = FLAGS_MANAGER.lock().unwrap();
    Ok(flags.telemetry_consent_shown.value)
}

#[tauri::command]
pub fn set_custom_clients_display(display: String) -> Result<(), String> {
    let mut flags = FLAGS_MANAGER.lock().unwrap();
    flags.set_custom_clients_display(display);
    flags.save_to_disk();
    drop(flags);
    Ok(())
}
