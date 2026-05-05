use crate::log_error;
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};

pub fn emit_to_main_window<S: Serialize + Clone>(app_handle: &AppHandle, event: &str, payload: S) {
    if let Some(window) = app_handle.get_webview_window("main") {
        if let Err(e) = window.emit(event, payload) {
            log_error!("Failed to emit event '{}': {}", event, e);
        }
    } else {
        log_error!("Main window not found for emitting event '{}'", event);
    }
}

pub fn show_main_window(app_handle: &AppHandle) {
    if let Some(window) = app_handle.get_webview_window("main") {
        let _ = window.show();
        let _ = window.set_focus();
    }
}

pub fn hide_main_window(app_handle: &AppHandle) {
    if let Some(window) = app_handle.get_webview_window("main") {
        let _ = window.hide();
    }
}

pub fn is_development_enabled() -> bool {
    env!("DEVELOPMENT").eq_ignore_ascii_case("true")
}
