use tauri::AppHandle;

use crate::{
    api::{
        clients::client::{Client, CLIENT_LOGS},
        utils,
    },
    log_info,
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
    pub fn new(client: Client) -> Self {
        LogChecker { client }
    }

    pub fn check(&self, app_handle_clone_for_crash_handling: &AppHandle) {
        if let Ok(logs_guard) = CLIENT_LOGS.lock() {
            if let Some(client_logs) = logs_guard.get(&self.client.id) {
                let full_log_string = client_logs.join("\\\\n");

                if let Some(crash_type) = self.detect_crash_type(&full_log_string) {
                    self.handle_crash(crash_type, client_logs, app_handle_clone_for_crash_handling);
                }
            }
        }
    }

    fn detect_crash_type(&self, log_string: &str) -> Option<CrashType> {
        if log_string.contains("Could not find or load main class") {
            Some(CrashType::MissingMainClass)
        } else if log_string.contains("java.lang.OutOfMemoryError") {
            Some(CrashType::OutOfMemory)
        } else if log_string.contains("#@!@# Game crashed!")
            || log_string.contains("Error occurred during initialization of VM")
        {
            Some(CrashType::GameCrashed)
        } else {
            None
        }
    }

    fn handle_crash(
        &self,
        crash_type: CrashType,
        client_logs: &Vec<String>,
        app_handle: &AppHandle,
    ) {
        match crash_type {
            CrashType::MissingMainClass => {
                log_info!(
                    "Client {} (ID: {}) crash likely due to missing main class. Triggering reinstall.",
                    self.client.name, self.client.id
                );
                utils::emit_to_main_window_filtered(
                    app_handle,
                    "client-needs-reinstall",
                    serde_json::json!({
                        "id": self.client.id,
                        "name": self.client.name.clone()
                    }),
                );
            }
            CrashType::OutOfMemory => {
                log_info!(
                    "Client {} (ID: {}) crash likely due to OutOfMemoryError.",
                    self.client.name,
                    self.client.id
                );
                self.emit_crash_details(client_logs, app_handle);
            }
            CrashType::GameCrashed => {
                self.emit_crash_details(client_logs, app_handle);
            }
        }
    }

    fn emit_crash_details(&self, client_logs: &Vec<String>, app_handle: &AppHandle) {
        utils::emit_to_main_window_filtered(
            app_handle,
            "client-crash-details",
            serde_json::json!({
                "id": self.client.id,
                "name": self.client.name.clone(),
                "logs": client_logs.clone()
            }),
        );
    }
}
