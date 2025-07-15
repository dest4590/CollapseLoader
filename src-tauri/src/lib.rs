use tauri::Manager;

use crate::api::analytics::Analytics;

mod api;
mod commands;

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_opener::init())
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
            commands::utils::get_api_url,
            commands::utils::open_data_folder,
            commands::utils::reset_requirements,
            commands::utils::reset_cache,
            commands::utils::decode_base64,
            commands::utils::encode_base64,
            commands::analytics::send_client_analytics,
            commands::discord_rpc::update_presence
        ])
        .setup(|app| {
            let app_handle = app.handle();
            *api::core::data::APP_HANDLE.lock().unwrap() = Some(app_handle.clone());

            let version = env!("CARGO_PKG_VERSION");
            let window_title = format!("CollapseLoader v{}", version);

            if let Some(window) = app_handle.get_webview_window("main") {
                let _ = window.set_title(&window_title);
            }

            Analytics::send_start_analytics();

            Ok(())
        })
        .on_window_event(|_window, event| {
            if let tauri::WindowEvent::CloseRequested { .. } = event {
                let _ = api::discord_rpc::shutdown();
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
