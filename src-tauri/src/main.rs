// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use collapseloader_lib::core::utils::args::Args;
#[cfg(target_os = "windows")]
use windows::Win32::UI::HiDpi::{SetProcessDpiAwareness, PROCESS_SYSTEM_DPI_AWARE};

fn main() {
    let args = Args::parse();
    args.process();

    let _ = dotenvy::dotenv();

    #[cfg(target_os = "windows")]
    unsafe {
        if let Err(e) = SetProcessDpiAwareness(PROCESS_SYSTEM_DPI_AWARE) {
            use collapseloader_lib::core::platform::error::StartupError;

            collapseloader_lib::log_error!("Failed to set DPI awareness: {}", e);
            collapseloader_lib::handle_startup_error(&StartupError::DpiAwarenessFailed(
                e.to_string(),
            ));
        }
    }

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
