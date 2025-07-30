use crate::log_error;
use serde::Serialize;
use tauri::{AppHandle, Emitter, EventTarget, Manager};

pub fn emit_to_main_window<S: Serialize + Clone>(app_handle: &AppHandle, event: &str, payload: S) {
    if let Some(window) = app_handle.get_webview_window("main") {
        if let Err(e) = window.emit(event, payload) {
            log_error!("Failed to emit event '{}': {}", event, e);
        }
    } else {
        log_error!("Main window not found for emitting event '{}'", event);
    }
}

pub fn emit_to_main_window_filtered<S: Serialize + Clone>(
    app_handle: &AppHandle,
    event: &str,
    payload: S,
) {
    if let Err(e) = app_handle.emit_filter(event, payload, |target| match target {
        EventTarget::Window { label } => label == "main",
        _ => false,
    }) {
        log_error!(
            "Failed to emit filtered event '{}' to main window: {}",
            event,
            e
        );
    }
}
