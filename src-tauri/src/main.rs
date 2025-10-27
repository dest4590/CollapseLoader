// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    let _ = dotenvy::dotenv();

    if let Err(e) = collapseloader_lib::check_dependencies() {
        collapseloader_lib::log_error!("Dependency check failed: {}", e);
        collapseloader_lib::handle_startup_error(&e);
    }

    #[cfg(target_os = "linux")]
    if let Err(e) = collapseloader_lib::check_webkit_warning() {
        collapseloader_lib::handle_startup_error(&e);
    }

    collapseloader_lib::run()
}
