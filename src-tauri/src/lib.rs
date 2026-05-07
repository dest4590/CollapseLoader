use crate::core::clients::manager::ClientManager;
use crate::core::app_runtime::{
    DeepLinkAction, DeepLinkDeduplicator, StartupMetadata, StartupRuntime, TrayMenuAction,
};
#[cfg(target_os = "windows")]
use crate::core::platform::messagebox;
use crate::core::utils::discord_rpc;
use crate::core::storage::data::APP_HANDLE;
use std::sync::{Arc, Mutex};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::Manager;

use self::core::network::analytics::Analytics;
use crate::core::network::servers::SERVERS;
use tauri::async_runtime::spawn;

pub mod commands;
pub mod core;

#[cfg(test)]
mod tests;

use self::core::platform::error::StartupError;
pub use crate::core::state::AppState;
use crate::core::utils::helpers::{emit_to_main_window, show_main_window};
pub use crate::core::utils::logging;

pub fn prepare_startup() -> Result<(), StartupError> {
    StartupRuntime::prepare()
}

#[cfg(target_os = "windows")]
pub fn handle_startup_error(error: &StartupError) {
    if matches!(error, StartupError::WebView2NotInstalled) {
        let should_install = messagebox::show_confirm(
            "WebView2 Not Installed",
            "WebView2 is not installed. Would you like to download and install it now?",
        );

        if should_install {
            match crate::core::platform::windows::attempt_install_webview2() {
                Ok(_) => {
                    let message = "WebView2 has been installed. Please restart the application.";
                    eprintln!("{message}");
                    messagebox::show_message("Restart Required", message);
                    std::process::exit(0);
                }
                Err(install_error) => install_error.show_and_exit(),
            }
            return;
        }
    }
    error.show_and_exit();
}

#[cfg(not(target_os = "windows"))]
pub fn handle_startup_error(error: &StartupError) {
    #[cfg(target_os = "linux")]
    if matches!(
        error,
        StartupError::LinuxWebKitWarning | StartupError::LinuxWebKitWaylandWarning
    ) {
        error.show_warning();
        return;
    }

    error.show_and_exit();
}

fn handle_tray_menu_action(app: &tauri::AppHandle, action_id: &str) {
    match TrayMenuAction::parse(action_id) {
        TrayMenuAction::Show => show_main_window(app),
        TrayMenuAction::Quit => {
            app.exit(0);
        }
        TrayMenuAction::LaunchClient(client_id) => {
            show_main_window(app);
            crate::core::utils::helpers::emit_to_main_window(
                app,
                "tray-launch-client",
                serde_json::json!({ "id": client_id }),
            );
        }
        TrayMenuAction::Ignore => {}
    }
}

fn handle_deep_link_url(app: &tauri::AppHandle, url: String, was_already_running: bool) {
    if was_already_running {
        show_main_window(app);
    }

    log_debug!(
        "Handling deep link URL: {} (already running: {})",
        url,
        was_already_running
    );

    if was_already_running && !DeepLinkDeduplicator::should_handle(&url) {
        log_debug!("Deep link already handled, skipping: {}", url);
        return;
    }

    match DeepLinkAction::parse(&url) {
        Some(DeepLinkAction::VerifyEmail { code, email }) => {
            log_debug!("Emitting verify-email event for code: {}", code);
            emit_to_main_window(
                app,
                "verify-email",
                serde_json::json!({ "code": code, "email": email }),
            );
        }
        Some(DeepLinkAction::LaunchClient { client_id }) => {
            log_debug!("Launching client from deep link: {}", client_id);
            emit_to_main_window(
                app,
                "launch-client",
                serde_json::json!({
                    "id": client_id,
                    "was_already_running": was_already_running
                }),
            );
        }
        None => {}
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
#[allow(clippy::large_stack_frames)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_deep_link::init())
        .plugin(tauri_plugin_single_instance::init(|app, args, _cwd| {
            if let Some(url) = args.iter().find(|a| a.starts_with("collapseloader://")) {
                handle_deep_link_url(app, url.clone(), true);
            }
        }))
        .manage(AppState::new(Arc::new(
            Mutex::new(ClientManager::default()),
        )))
        .manage(commands::irc::IrcState::default())
        .invoke_handler(tauri::generate_handler![
            // client commands
            commands::clients::add_custom_client,
            commands::clients::delete_client,
            commands::clients::detect_main_class,
            commands::clients::download_client_only,
            commands::clients::get_app_logs,
            commands::clients::get_client_logs,
            commands::clients::get_clients,
            commands::clients::get_custom_clients,
            commands::clients::get_latest_client_logs,
            commands::clients::get_running_client_ids,
            commands::clients::get_running_custom_client_ids,
            commands::clients::increment_client_counter,
            commands::clients::initialize_api,
            commands::clients::initialize_rpc,
            commands::clients::install_mod_from_url,
            commands::clients::install_mod_for_custom_client,
            commands::clients::uninstall_mod,
            commands::clients::uninstall_mod_custom,
            commands::clients::launch_client,
            commands::clients::launch_custom_client,
            commands::clients::list_installed_mods,
            commands::clients::open_client_folder,
            commands::clients::reinstall_client,
            commands::clients::remove_custom_client,
            commands::clients::stop_client,
            commands::clients::get_client_ram_usage,
            commands::clients::stop_custom_client,
            commands::clients::open_custom_client_folder,
            commands::clients::list_installed_mods_custom,
            commands::clients::update_client_installed_status,
            commands::clients::update_custom_client,
            // irc commands
            commands::irc::connect_irc,
            commands::irc::disconnect_irc,
            commands::irc::send_irc_message,
            // preset commands
            commands::presets::create_preset,
            commands::presets::delete_preset,
            commands::presets::duplicate_preset,
            commands::presets::get_all_presets,
            commands::presets::get_preset,
            commands::presets::update_preset,
            // settings commands
            commands::settings::add_account,
            commands::settings::add_favorite_client,
            commands::settings::get_accounts,
            commands::settings::get_active_account,
            commands::settings::get_favorite_clients,
            commands::settings::get_flags,
            commands::settings::get_settings,
            commands::settings::get_setting_bool,
            commands::settings::get_system_memory,
            commands::settings::is_client_favorite,
            commands::settings::is_telemetry_consent_shown,
            commands::settings::mark_disclaimer_shown,
            commands::settings::mark_first_run_shown,
            commands::settings::mark_telemetry_consent_shown,
            commands::settings::reorder_accounts,
            commands::settings::remove_account,
            commands::settings::remove_favorite_client,
            commands::settings::reset_flags,
            commands::settings::reset_settings,
            commands::settings::save_settings,
            commands::settings::set_active_account,
            commands::settings::set_all_favorites,
            commands::settings::set_custom_clients_display,
            commands::settings::set_optional_telemetry,
            commands::settings::update_account,
            // updater commands
            commands::updater::check_for_updates,
            commands::updater::download_and_install_update,
            commands::updater::get_changelog,
            // utils commands
            commands::utils::change_data_folder,
            commands::utils::cancel_download,
            commands::utils::decode_base64,
            commands::utils::encode_base64,
            commands::utils::get_api_url,
            // commands::utils::get_api_version,
            commands::utils::get_cdn_url,
            commands::utils::get_data_folder,
            commands::utils::get_version,
            commands::utils::is_development,
            commands::utils::open_data_folder,
            commands::utils::get_storage_usage,
            commands::utils::reset_requirements,
            commands::utils::update_presence,
            commands::utils::is_macos,
            commands::utils::set_window_theme,
            commands::utils::update_tray_menu,
            // network commands
            commands::network::api_request,
            commands::network::get_network_history,
            commands::network::clear_network_history,
            commands::report::generate_network_report,
            commands::report::export_network_report,
            // server connectivity
            commands::clients::get_server_connectivity_status,
        ])
        .setup(|app| {
            #[cfg(all(desktop, not(target_os = "macos")))]
            {
                use tauri_plugin_deep_link::DeepLinkExt;
                let handle = app.handle().clone();
                app.deep_link().on_open_url(move |event| {
                    handle_deep_link_url(&handle, event.urls()[0].to_string(), true);
                });
                app.deep_link().register_all()?;
            }

            let args: Vec<String> = std::env::args().collect();
            if let Some(url) = args.iter().find(|a| a.starts_with("collapseloader://")) {
                let handle = app.handle().clone();
                let url_clone = url.clone();
                spawn(async move {
                    tokio::time::sleep(std::time::Duration::from_millis(2000)).await;
                    handle_deep_link_url(&handle, url_clone, false);
                });
            }

            let app_handle = app.handle();
            *APP_HANDLE.lock().unwrap() = Some(app_handle.clone());

            let startup_metadata = StartupMetadata::from_env();
            startup_metadata.configure_main_window(&app_handle);
            startup_metadata.print_banner();

            spawn(async {
                SERVERS.check_servers().await;
                Analytics::send_start_analytics();
            });

            let tray_icon = app
                .default_window_icon()
                .cloned()
                .ok_or_else(|| "Default window icon is missing".to_string())?;

            let _tray = TrayIconBuilder::with_id("main")
                .icon(tray_icon)
                .show_menu_on_left_click(false)
                .on_menu_event(|app: &tauri::AppHandle, event| {
                    handle_tray_menu_action(app, event.id.as_ref());
                })
                .on_tray_icon_event(|tray: &tauri::tray::TrayIcon, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            let is_visible = window.is_visible().unwrap_or(true);
                            if is_visible {
                                crate::core::utils::helpers::hide_main_window(app);
                            } else {
                                show_main_window(app);
                            }
                        }
                    }
                })
                .build(app)?;

            if let Some(state) = app.try_state::<AppState>() {
                let _ = crate::commands::utils::update_tray_menu(app.handle().clone(), state);
            }

            #[cfg(target_os = "windows")]
            {
                use crate::core::utils::dpi;
                dpi::start_winws_background_if_configured();
            }

            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                let settings = crate::core::storage::settings::SETTINGS.lock().unwrap();
                if settings.close_to_tray.value {
                    api.prevent_close();
                    let _ = window.hide();
                } else {
                    discord_rpc::shutdown();
                }
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
