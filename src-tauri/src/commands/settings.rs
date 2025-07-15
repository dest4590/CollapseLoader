use crate::api::core::accounts::{Account, ACCOUNT_MANAGER};
use crate::api::core::common::JsonStorage;
use crate::api::core::favorites::FAVORITE_MANAGER;
use crate::api::core::flags::{Flags, FLAGS_MANAGER};
use crate::api::core::settings::{InputSettings, Settings, SETTINGS};
use crate::api::discord_rpc;

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
            eprintln!("Failed to toggle Discord RPC: {}", e);
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

    Ok(())
}

#[tauri::command]
pub fn mark_disclaimer_shown() -> Result<(), String> {
    let mut flags = FLAGS_MANAGER.lock().unwrap();
    flags.disclaimer_shown.value = true;
    flags.save_to_disk();
    Ok(())
}

#[tauri::command]
pub fn mark_first_run_shown() -> Result<(), String> {
    let mut flags = FLAGS_MANAGER.lock().unwrap();
    flags.first_run.value = false;
    flags.save_to_disk();
    Ok(())
}

#[tauri::command]
pub fn set_optional_telemetry(enabled: bool) -> Result<(), String> {
    let mut settings = SETTINGS.lock().unwrap();
    settings.optional_telemetry.value = enabled;
    settings.save_to_disk();
    Ok(())
}

#[tauri::command]
pub fn get_accounts() -> Vec<Account> {
    if let Ok(account_manager) = ACCOUNT_MANAGER.lock() {
        account_manager.accounts.clone()
    } else {
        Vec::new()
    }
}

#[tauri::command]
pub fn add_account(username: String, tags: Vec<String>) -> Result<String, String> {
    if let Ok(mut account_manager) = ACCOUNT_MANAGER.lock() {
        let id = account_manager.add_account(username, tags);
        account_manager.save_to_disk();
        Ok(id)
    } else {
        Err("Failed to acquire lock on account manager".to_string())
    }
}

#[tauri::command]
pub fn remove_account(id: String) -> Result<(), String> {
    if let Ok(mut account_manager) = ACCOUNT_MANAGER.lock() {
        if account_manager.remove_account(&id) {
            account_manager.save_to_disk();
            Ok(())
        } else {
            Err("Account not found".to_string())
        }
    } else {
        Err("Failed to acquire lock on account manager".to_string())
    }
}

#[tauri::command]
pub fn set_active_account(id: String) -> Result<(), String> {
    if let Ok(mut account_manager) = ACCOUNT_MANAGER.lock() {
        if account_manager.set_active_account(&id) {
            account_manager.save_to_disk();
            Ok(())
        } else {
            Err("Account not found".to_string())
        }
    } else {
        Err("Failed to acquire lock on account manager".to_string())
    }
}

#[tauri::command]
pub fn update_account(
    id: String,
    username: Option<String>,
    tags: Option<Vec<String>>,
) -> Result<(), String> {
    if let Ok(mut account_manager) = ACCOUNT_MANAGER.lock() {
        if account_manager.update_account(&id, username, tags) {
            account_manager.save_to_disk();
            Ok(())
        } else {
            Err("Account not found".to_string())
        }
    } else {
        Err("Failed to acquire lock on account manager".to_string())
    }
}

#[tauri::command]
pub fn get_active_account() -> Option<Account> {
    if let Ok(account_manager) = ACCOUNT_MANAGER.lock() {
        account_manager.get_active_account().cloned()
    } else {
        None
    }
}

#[tauri::command]
pub fn get_favorite_clients() -> Result<Vec<u32>, String> {
    if let Ok(favorite_manager) = FAVORITE_MANAGER.lock() {
        Ok(favorite_manager.favorites.clone())
    } else {
        Err("Failed to acquire lock on favorite manager".to_string())
    }
}

#[tauri::command]
pub fn add_favorite_client(client_id: u32) -> Result<(), String> {
    if let Ok(mut favorite_manager) = FAVORITE_MANAGER.lock() {
        favorite_manager.add_favorite(client_id);
        favorite_manager.save_to_disk();
        Ok(())
    } else {
        Err("Failed to acquire lock on favorite manager".to_string())
    }
}

#[tauri::command]
pub fn remove_favorite_client(client_id: u32) -> Result<(), String> {
    if let Ok(mut favorite_manager) = FAVORITE_MANAGER.lock() {
        favorite_manager.remove_favorite(client_id);
        favorite_manager.save_to_disk();
        Ok(())
    } else {
        Err("Failed to acquire lock on favorite manager".to_string())
    }
}

#[tauri::command]
pub fn is_client_favorite(client_id: u32) -> Result<bool, String> {
    if let Ok(favorite_manager) = FAVORITE_MANAGER.lock() {
        Ok(favorite_manager.is_favorite(client_id))
    } else {
        Err("Failed to acquire lock on favorite manager".to_string())
    }
}

#[tauri::command]
pub fn mark_telemetry_consent_shown() -> Result<(), String> {
    let mut flags = FLAGS_MANAGER.lock().unwrap();
    flags.telemetry_consent_shown.value = true;
    flags.save_to_disk();
    Ok(())
}

#[tauri::command]
pub fn is_telemetry_consent_shown() -> Result<bool, String> {
    let flags = FLAGS_MANAGER.lock().unwrap();
    Ok(flags.telemetry_consent_shown.value)
}
