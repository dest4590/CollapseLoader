use tauri::AppHandle;

use crate::core::utils::helpers::emit_to_main_window_filtered;
use crate::{
    core::clients::client::{Client, CLIENT_LOGS},
    log_debug, log_warn,
};

pub struct LogChecker {
    pub client: Client,
}

#[derive(Debug)]
enum CrashType {
    MissingMainClass,
    OutOfMemory,
    GameCrashed,
}

impl LogChecker {
    pub const fn new(client: Client) -> Self {
        Self { client }
    }

    pub fn check(&self, app_handle_clone_for_crash_handling: &AppHandle) {
        log_debug!("Checking logs for client '{}'", self.client.name);
        if let Ok(logs_guard) = CLIENT_LOGS.lock() {
            if let Some(client_logs) = logs_guard.get(&self.client.id) {
                let full_log_string = client_logs.join("\\\\n");

                if let Some(crash_type) = self.detect_crash_type(&full_log_string) {
                    self.handle_crash(crash_type, client_logs, app_handle_clone_for_crash_handling);
                } else {
                    log_debug!(
                        "No crash detected in logs for client '{}'",
                        self.client.name
                    );
                }
            } else {
                log_warn!(
                    "Could not find logs for client ID {} during crash check",
                    self.client.id
                );
            }
        } else {
            log_warn!("Failed to acquire lock on CLIENT_LOGS for crash check");
        }
    }

    fn detect_crash_type(&self, log_string: &str) -> Option<CrashType> {
        if log_string.contains("Could not find or load main class") {
            log_debug!("Detected MissingMainClass crash type");
            Some(CrashType::MissingMainClass)
        } else if log_string.contains("java.lang.OutOfMemoryError") {
            log_debug!("Detected OutOfMemory crash type");
            Some(CrashType::OutOfMemory)
        } else if log_string.contains("#@!@# Game crashed!")
            || log_string.contains("Error occurred during initialization of VM")
            || log_string.contains("java.lang.UnsupportedClassVersionError")
        {
            log_debug!("Detected GameCrashed crash type");
            Some(CrashType::GameCrashed)
        } else {
            None
        }
    }

    fn handle_crash(&self, crash_type: CrashType, client_logs: &[String], app_handle: &AppHandle) {
        log_warn!(
            "Client {} crashed! Detected reason: {:?}",
            self.client.name,
            crash_type
        );
        match crash_type {
            CrashType::MissingMainClass => {
                emit_to_main_window_filtered(
                    app_handle,
                    "client-needs-reinstall",
                    serde_json::json!({
                        "id": self.client.id,
                        "name": self.client.name.clone()
                    }),
                );
            }
            CrashType::OutOfMemory => {
                self.emit_crash_details(client_logs, app_handle);
                emit_to_main_window_filtered(
                    app_handle,
                    "client-crashed",
                    serde_json::json!({
                        "id": self.client.id,
                        "name": self.client.name.clone(),
                        "error": "OutOfMemoryError"
                    }),
                );
            }
            CrashType::GameCrashed => {
                self.emit_crash_details(client_logs, app_handle);
                emit_to_main_window_filtered(
                    app_handle,
                    "client-crashed",
                    serde_json::json!({
                        "id": self.client.id,
                        "name": self.client.name.clone(),
                        "error": "JVM crash or uncaught exception"
                    }),
                );
            }
        }
    }

    fn emit_crash_details(&self, client_logs: &[String], app_handle: &AppHandle) {
        log_debug!(
            "Emitting client-crash-details for client '{}'",
            self.client.name
        );
        emit_to_main_window_filtered(
            app_handle,
            "client-crash-details",
            serde_json::json!({
                "id": self.client.id,
                "name": self.client.name.clone(),
                "logs": client_logs.to_owned()
            }),
        );
    }
}
