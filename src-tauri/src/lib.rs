use tauri::Manager;

use self::core::network::analytics::Analytics;

mod commands;
mod core;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            commands::clients::initialize_api,
            commands::clients::initialize_rpc,
            commands::clients::get_server_connectivity_status,
            commands::clients::get_clients,
            commands::clients::get_client_details,
            commands::clients::launch_client,
            commands::clients::get_running_client_ids,
            commands::clients::stop_client,
            commands::clients::get_client_logs,
            commands::clients::get_app_logs,
            commands::clients::download_client_only,
            commands::clients::reinstall_client,
            commands::clients::open_client_folder,
            commands::clients::get_latest_client_logs,
            commands::clients::update_client_installed_status,
            commands::clients::delete_client,
            commands::clients::increment_client_counter,
            commands::clients::get_custom_clients,
            commands::clients::add_custom_client,
            commands::clients::remove_custom_client,
            commands::clients::update_custom_client,
            commands::clients::launch_custom_client,
            commands::clients::validate_custom_clients,
            commands::settings::get_settings,
            commands::settings::save_settings,
            commands::settings::reset_settings,
            commands::settings::mark_disclaimer_shown,
            commands::settings::mark_first_run_shown,
            commands::settings::set_optional_telemetry,
            commands::settings::mark_telemetry_consent_shown,
            commands::settings::is_telemetry_consent_shown,
            commands::settings::get_flags,
            commands::settings::get_accounts,
            commands::settings::add_account,
            commands::settings::remove_account,
            commands::settings::set_active_account,
            commands::settings::update_account,
            commands::settings::get_active_account,
            commands::settings::reset_flags,
            commands::settings::get_favorite_clients,
            commands::settings::add_favorite_client,
            commands::settings::remove_favorite_client,
            commands::settings::is_client_favorite,
            commands::utils::get_version,
            commands::utils::get_auth_url,
            commands::utils::open_data_folder,
            commands::utils::reset_requirements,
            commands::utils::reset_cache,
            commands::utils::decode_base64,
            commands::utils::encode_base64,
            commands::analytics::send_client_analytics,
            commands::discord_rpc::update_presence,
            commands::plugins::get_plugins_manifest,
            commands::plugins::get_plugin_data,
            commands::plugins::save_plugin_data,
            commands::plugins::delete_plugin,
            commands::plugins::update_plugin_enabled_status,
            commands::plugins::get_plugin_code,
            commands::plugins::save_plugin_code,
            commands::plugins::create_plugin_from_text,
            commands::updater::check_for_updates,
            commands::updater::download_and_install_update,
            commands::updater::get_changelog,
        ])
        .setup(|app| {
            let app_handle = app.handle();
            *core::storage::data::APP_HANDLE.lock().unwrap() = Some(app_handle.clone());

            let version = env!("CARGO_PKG_VERSION");
            let window_title = format!("CollapseLoader v{version}");

            if let Some(window) = app_handle.get_webview_window("main") {
                let _ = window.set_title(&window_title);
            }

            Analytics::send_start_analytics();

            Ok(())
        })
        .on_window_event(|_window, event| {
            if let tauri::WindowEvent::CloseRequested { .. } = event {
                core::utils::discord_rpc::shutdown();
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
