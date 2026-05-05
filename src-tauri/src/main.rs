// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use collapseloader_lib::core::utils::args::Args;

fn main() {
    let args = Args::parse();
    args.process();

    let _ = dotenvy::dotenv();

    if let Err(e) = collapseloader_lib::prepare_startup() {
        collapseloader_lib::log_error!("Startup preparation failed: {}", e);
        collapseloader_lib::handle_startup_error(&e);
    }

    collapseloader_lib::run()
}
