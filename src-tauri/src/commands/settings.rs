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

#[cfg(target_os = "windows")]
fn set_autostart_registry(enabled: bool) -> Result<(), String> {
    use winreg::enums::*;
    use winreg::RegKey;

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let run_key = hkcu
        .open_subkey_with_flags(
            "Software\\Microsoft\\Windows\\CurrentVersion\\Run",
            KEY_SET_VALUE | KEY_QUERY_VALUE,
        )
        .map_err(|e| format!("Failed to open registry key: {}", e))?;

    if enabled {
        let exe_path = std::env::current_exe()
            .map_err(|e| format!("Failed to get exe path: {}", e))?;
        let exe_str = exe_path
            .to_str()
            .ok_or("Failed to convert exe path to string")?;
        run_key
            .set_value("CollapseLoader", &exe_str)
            .map_err(|e| format!("Failed to set registry value: {}", e))?;
        log_info!("Autostart enabled: {}", exe_str);
    } else {
        match run_key.delete_value("CollapseLoader") {
            Ok(_) => log_info!("Autostart disabled"),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                log_info!("Autostart entry not found, nothing to remove");
            }
            Err(e) => return Err(format!("Failed to delete registry value: {}", e)),
        }
    }

    Ok(())
}

#[cfg(target_os = "macos")]
fn set_autostart_registry(enabled: bool) -> Result<(), String> {
    let home = std::env::var("HOME").map_err(|e| format!("Failed to get HOME: {}", e))?;
    let plist_dir = format!("{}/Library/LaunchAgents", home);
    let plist_path = format!("{}/org.collapseloader.app.plist", plist_dir);

    if enabled {
        let exe_path = std::env::current_exe()
            .map_err(|e| format!("Failed to get exe path: {}", e))?;
        let exe_str = exe_path
            .to_str()
            .ok_or("Failed to convert exe path to string")?;

        std::fs::create_dir_all(&plist_dir)
            .map_err(|e| format!("Failed to create LaunchAgents dir: {}", e))?;

        let plist_content = format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>org.collapseloader.app</string>
    <key>ProgramArguments</key>
    <array>
        <string>{}</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
</dict>
</plist>"#,
            exe_str
        );

        std::fs::write(&plist_path, plist_content)
            .map_err(|e| format!("Failed to write plist: {}", e))?;
        log_info!("Autostart enabled via LaunchAgent: {}", plist_path);
    } else {
        match std::fs::remove_file(&plist_path) {
            Ok(_) => log_info!("Autostart disabled, plist removed"),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                log_info!("Autostart plist not found, nothing to remove");
            }
            Err(e) => return Err(format!("Failed to remove plist: {}", e)),
        }
    }

    Ok(())
}

#[cfg(target_os = "linux")]
fn set_autostart_registry(enabled: bool) -> Result<(), String> {
    let home = std::env::var("HOME").map_err(|e| format!("Failed to get HOME: {}", e))?;
    let autostart_dir = format!("{}/.config/autostart", home);
    let desktop_path = format!("{}/collapseloader.desktop", autostart_dir);

    if enabled {
        let exe_path = std::env::current_exe()
            .map_err(|e| format!("Failed to get exe path: {}", e))?;
        let exe_str = exe_path
            .to_str()
            .ok_or("Failed to convert exe path to string")?;

        std::fs::create_dir_all(&autostart_dir)
            .map_err(|e| format!("Failed to create autostart dir: {}", e))?;

        let desktop_content = format!(
            "[Desktop Entry]\nType=Application\nName=CollapseLoader\nExec={}\nHidden=false\nNoDisplay=false\nX-GNOME-Autostart-enabled=true\n",
            exe_str
        );

        std::fs::write(&desktop_path, desktop_content)
            .map_err(|e| format!("Failed to write desktop file: {}", e))?;
        log_info!("Autostart enabled via .desktop file: {}", desktop_path);
    } else {
        match std::fs::remove_file(&desktop_path) {
            Ok(_) => log_info!("Autostart disabled, desktop file removed"),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                log_info!("Autostart desktop file not found, nothing to remove");
            }
            Err(e) => return Err(format!("Failed to remove desktop file: {}", e)),
        }
    }

    Ok(())
}

#[tauri::command]
pub fn get_settings() -> Settings {
    SETTINGS.lock().unwrap().clone()
}

#[tauri::command]
pub fn get_setting_bool(key: String) -> bool {
    let s = SETTINGS.lock().unwrap();
    match key.as_str() {
        "auto_update" => s.auto_update.value,
        "discord_rpc_enabled" => s.discord_rpc_enabled.value,
        "hash_verify" => s.hash_verify.value,
        "minimize_to_tray_on_launch" => s.minimize_to_tray_on_launch.value,
        "close_to_tray" => s.close_to_tray.value,
        _ => false,
    }
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
    let mut current_settings = SETTINGS.lock().unwrap();
    let config_path = current_settings.config_path.clone();

    let old_discord_rpc_enabled = current_settings.discord_rpc_enabled.value;
    let old_autostart = current_settings.autostart.value;
    #[cfg(target_os = "windows")]
    let old_dpi_bypass_enabled = current_settings.dpi_bypass.value;

    let discord_rpc_changed = old_discord_rpc_enabled != input_settings.discord_rpc_enabled.value;
    let new_discord_rpc_value = input_settings.discord_rpc_enabled.value;
    let autostart_changed = old_autostart != input_settings.autostart.value;
    let new_autostart_value = input_settings.autostart.value;

    #[cfg(target_os = "windows")]
    let dpi_bypass_changed = old_dpi_bypass_enabled != input_settings.dpi_bypass.value;
    #[cfg(target_os = "windows")]
    let new_dpi_bypass_value = input_settings.dpi_bypass.value;

    let new_settings = Settings::from_input(input_settings, config_path);
    *current_settings = new_settings.clone();

    new_settings.save_to_disk();

    drop(current_settings);

    if autostart_changed {
        if let Err(e) = set_autostart_registry(new_autostart_value) {
            log_error!("Failed to set autostart: {e}");
        }
    }

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
        if dpi_bypass_changed {
            if new_dpi_bypass_value {
                log_info!("DPI bypass enabled. Preparing to download and run package");
                if let Err(e) = dpi::enable_dpi_bypass_async() {
                    log_error!("Failed to initiate DPI bypass setup: {e}");
                }
            } else {
                log_info!("DPI bypass disabled. Killing existing processes");
                dpi::kill_winws();
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
pub fn set_all_favorites(client_ids: Vec<u32>) -> Result<(), String> {
    log_info!("Setting all favorites to: {:?}", client_ids);
    FAVORITE_MANAGER.lock().map_or_else(
        |e| {
            log_error!("Failed to acquire lock on favorite manager: {}", e);
            Err("Failed to acquire lock on favorite manager".to_string())
        },
        |mut favorite_manager| {
            favorite_manager.favorites = client_ids;
            favorite_manager.save_to_disk();
            log_info!("All favorites updated and saved");
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
