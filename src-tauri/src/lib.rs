use crate::core::clients::manager::ClientManager;
#[cfg(target_os = "windows")]
use crate::core::platform::messagebox;
use crate::core::utils::discord_rpc;
use crate::{core::storage::data::APP_HANDLE, logging::Logger};
use std::sync::OnceLock;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::Manager;

use crate::core::{platform::check_platform_dependencies, utils::globals::CODENAME};

#[cfg(target_os = "linux")]
use crate::core::platform::check_webkit_environment;

use self::core::network::analytics::Analytics;
use crate::core::network::servers::SERVERS;
use tauri::async_runtime::spawn;

pub mod commands;
pub mod core;

use self::core::platform::error::StartupError;
pub use crate::core::state::AppState;
use crate::core::utils::helpers::emit_to_main_window;
pub use crate::core::utils::logging;

pub fn check_dependencies() -> Result<(), StartupError> {
    check_platform_dependencies()
}

#[cfg(target_os = "linux")]
pub fn check_webkit_warning() -> Result<(), StartupError> {
    log_info!("Checking WebKit environment variables...");
    check_webkit_environment()
}

#[cfg(target_os = "windows")]
pub fn handle_startup_error(error: &StartupError) {
    if matches!(error, StartupError::WebView2NotInstalled) {
        let should_install = messagebox::show_confirm(
            "WebView2 Not Installed",
            "WebView2 is not installed. Would you like to download and install it now?",
        );

        if should_install {
            if let Err(install_error) = crate::core::platform::windows::attempt_install_webview2() {
                install_error.show_and_exit();
            } else {
                let message = "WebView2 has been installed. Please restart the application.";
                eprintln!("{message}");
                messagebox::show_message("Restart Required", message);

                std::process::exit(0);
            }
        } else {
            error.show_and_exit();
        }
    } else {
        error.show_and_exit();
    }
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

fn parse_verify_params(url: &str) -> Option<(String, String)> {
    let query_part = url.split_once('?')?.1;
    let mut code = String::new();
    let mut email = String::new();

    for pair in query_part.split('&') {
        if let Some((key, value)) = pair.split_once('=') {
            match key {
                "code" => code = value.to_string(),
                "email" => email = value.to_string(),
                _ => {}
            }
        }
    }

    if code.is_empty() {
        None
    } else {
        Some((code, email))
    }
}

fn parse_client_name(url: &str) -> Option<&str> {
    let (_, query_part) = url.split_once('?')?;

    query_part
        .split('&')
        .find_map(|pair| pair.split_once('='))
        .and_then(|(key, value)| (key == "client").then_some(value))
}

fn handle_deep_link_url(app: &tauri::AppHandle, url: String, was_already_running: bool) {
    if was_already_running {
        if let Some(window) = app.get_webview_window("main") {
            let _ = window.set_focus();
            let _ = window.show();
        }
    }

    log_debug!(
        "Handling deep link URL: {} (already running: {})",
        url,
        was_already_running
    );

    if !should_handle_deep_link(&url) && was_already_running {
        log_debug!("Deep link already handled, skipping: {}", url);
        return;
    }

    if url.contains("verify") {
        if let Some((code, email)) = parse_verify_params(&url) {
            log_debug!("Emitting verify-email event for code: {}", code);
            emit_to_main_window(
                app,
                "verify-email",
                serde_json::json!({ "code": code, "email": email }),
            );
            return;
        }
    }

    if let Some(client_name) = parse_client_name(&url) {
        log_debug!("Launching client from deep link: {}", client_name);
        emit_to_main_window(
            app,
            "launch-client",
            serde_json::json!({
                "id": client_name,
                "was_already_running": was_already_running
            }),
        );
    }
}

fn should_handle_deep_link(url: &str) -> bool {
    static LAST_HANDLED: OnceLock<Mutex<Option<(String, Instant)>>> = OnceLock::new();
    let last = LAST_HANDLED.get_or_init(|| Mutex::new(None));
    let mut guard = match last.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    let normalized = url.trim().to_string();

    if let Some((ref prev, prev_time)) = guard.as_ref() {
        if prev == &normalized && prev_time.elapsed() < Duration::from_secs(2) {
            return false;
        }
    }

    *guard = Some((normalized, Instant::now()));
    true
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
            commands::clients::launch_client,
            commands::clients::launch_custom_client,
            commands::clients::list_installed_mods,
            commands::clients::open_client_folder,
            commands::clients::reinstall_client,
            commands::clients::remove_custom_client,
            commands::clients::stop_client,
            commands::clients::stop_custom_client,
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
            commands::settings::get_system_memory,
            commands::settings::is_client_favorite,
            commands::settings::is_telemetry_consent_shown,
            commands::settings::mark_disclaimer_shown,
            commands::settings::mark_first_run_shown,
            commands::settings::mark_telemetry_consent_shown,
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
            commands::utils::decode_base64,
            commands::utils::encode_base64,
            commands::utils::get_api_url,
            commands::utils::get_api_version,
            commands::utils::get_cdn_url,
            commands::utils::get_data_folder,
            commands::utils::get_version,
            commands::utils::is_development,
            commands::utils::open_data_folder,
            commands::utils::reset_requirements,
            commands::utils::update_presence,
            commands::utils::is_macos,
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

            let is_dev = env!("DEVELOPMENT") == "true";
            let git_hash = env!("GIT_HASH")
                .to_string()
                .chars()
                .take(7)
                .collect::<String>();
            let git_branch = env!("GIT_BRANCH").to_string();

            let version = env!("CARGO_PKG_VERSION");
            let codename = CODENAME.to_string().to_uppercase();
            let window_title = format!(
                "CollapseLoader v{version} ({codename}) {}",
                if is_dev {
                    format!("(development build, {git_hash}, {git_branch} branch)")
                } else {
                    String::new()
                }
            );

            if let Some(window) = app_handle.get_webview_window("main") {
                if let Err(e) = window.set_title(&window_title) {
                    log_warn!("Failed to set window title: {}", e);
                }

                #[cfg(target_os = "macos")]
                if let Err(e) = window.set_decorations(true) {
                    log_warn!("Failed to enable window decorations: {}", e);
                }
            }

            Logger::print_startup_banner(version, &codename, is_dev, &git_hash, &git_branch);

            spawn(async {
                SERVERS.check_servers().await;
                Analytics::send_start_analytics();
            });

            let show = MenuItem::with_id(app, "show", "Show", true, None::<&str>)?;
            let hide = MenuItem::with_id(app, "hide", "Hide", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show, &hide, &quit])?;

            let tray_icon = app
                .default_window_icon()
                .cloned()
                .ok_or_else(|| "Default window icon is missing".to_string())?;

            let _tray = TrayIconBuilder::new()
                .icon(tray_icon)
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "show" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    "hide" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.hide();
                        }
                    }
                    "quit" => {
                        app.exit(0);
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
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
                                let _ = window.hide();
                            } else {
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                    }
                })
                .build(app)?;

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
