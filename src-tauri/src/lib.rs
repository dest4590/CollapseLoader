use tauri::Manager;

use crate::core::utils::globals::CODENAME;

use self::core::network::analytics::Analytics;

mod commands;
mod core;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
#[allow(clippy::large_stack_frames)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
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
            commands::clients::get_running_custom_client_ids,
            commands::clients::stop_custom_client,
            commands::presets::get_all_presets,
            commands::presets::get_preset,
            commands::presets::create_preset,
            commands::presets::update_preset,
            commands::presets::delete_preset,
            commands::presets::duplicate_preset,
            commands::settings::get_settings,
            commands::settings::save_settings,
            commands::settings::reset_settings,
            commands::settings::mark_disclaimer_shown,
            commands::settings::mark_first_run_shown,
            commands::settings::set_optional_telemetry,
            commands::settings::mark_telemetry_consent_shown,
            commands::settings::is_telemetry_consent_shown,
            commands::settings::set_custom_clients_display,
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
            commands::utils::is_development,
            commands::utils::get_auth_url,
            commands::utils::open_data_folder,
            commands::utils::reset_requirements,
            commands::utils::reset_cache,
            commands::utils::get_data_folder,
            commands::utils::change_data_folder,
            commands::utils::decode_base64,
            commands::utils::encode_base64,
            commands::discord_rpc::update_presence,
            commands::updater::check_for_updates,
            commands::updater::download_and_install_update,
            commands::updater::get_changelog,
        ])
        .setup(|app| {
            let app_handle = app.handle();
            *core::storage::data::APP_HANDLE.lock().unwrap() = Some(app_handle.clone());

            crate::log_debug!("Tauri setup: application handle stored");

            // dev info
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
                let _ = window.set_title(&window_title);
            }

            crate::log_info!("Starting CollapseLoader: {}", window_title);
            Analytics::send_start_analytics();
            crate::log_debug!("Analytics start event sent");

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
