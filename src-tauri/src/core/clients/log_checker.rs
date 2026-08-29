use regex::Regex;
use serde::Serialize;
use std::collections::HashSet;
use std::sync::{Arc, LazyLock, Mutex};
use tauri::AppHandle;

use crate::{
    core::{
        clients::client::{Client, CLIENT_LOGS},
        network::servers::SERVERS,
        storage::data::DATA,
        utils::globals::API_VERSION,
        utils::helpers::emit_to_main_window,
    },
    log_debug, log_error, log_info, log_warn,
};

static OPTIONS_BLACKLIST: LazyLock<Arc<Mutex<HashSet<String>>>> =
    LazyLock::new(|| Arc::new(Mutex::new(HashSet::new())));

pub fn is_options_sync_blocked(client_base: &str) -> bool {
    OPTIONS_BLACKLIST
        .lock()
        .map(|s| s.contains(client_base))
        .unwrap_or(false)
}

pub fn block_options_sync(client_base: &str) {
    if let Ok(mut s) = OPTIONS_BLACKLIST.lock() {
        s.insert(client_base.to_string());
    }
}

pub struct LogChecker {
    pub client: Client,
    user_token: String,
}

#[derive(Debug, Clone, Copy)]
enum CrashType {
    MissingMainClass,
    OutOfMemory,
    GameCrashed,
    OptionsCorrupted,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CrashLogPayload {
    client_name: String,
    client_version: String,
    crash_type: String,
    loader_version: String,
    os_name: String,
    os_version: String,
    line_count: usize,
    log_content: String,
}

const MAX_UPLOAD_LINES: usize = 500;
const MAX_UPLOAD_CHARS: usize = 32_000;

static MASK_PATTERNS: LazyLock<Vec<(Regex, &'static str)>> = LazyLock::new(|| {
    vec![
        (
            Regex::new(r"([A-Za-z]:\\Users\\)[^\\]+").expect("valid windows backslash path regex"),
            "$1********",
        ),
        (
            Regex::new(r"([A-Za-z]:\\\\Users\\\\)[^\\]+")
                .expect("valid escaped windows backslash path regex"),
            "$1********",
        ),
        (
            Regex::new(r"([A-Za-z]:/Users/)[^/]+").expect("valid windows slash path regex"),
            "$1********",
        ),
        (
            Regex::new(r"(^|[^\w])(/home/)[^/\s]+").expect("valid linux home regex"),
            "$1$2********",
        ),
        (
            Regex::new(r"(^|[^\w])(/Users/)[^/\s]+").expect("valid mac users regex"),
            "$1$2********",
        ),
        (
            Regex::new(r"USERNAME=\w+").expect("valid username env regex"),
            "USERNAME=********",
        ),
        (
            Regex::new(r"(?i)\b(USERPROFILE|HOME|HOMEPATH)=\S+").expect("valid home env regex"),
            "$1=********",
        ),
        (
            Regex::new(r#"(?i)(--accessToken\s+\"?)[^\"\s]+(\"?)"#)
                .expect("valid access token arg regex"),
            "$1********$2",
        ),
        (
            Regex::new(r"(?i)(accessToken=)\S+").expect("valid access token kv regex"),
            "$1********",
        ),
        (
            Regex::new(r#"(?i)(--uuid\s+\"?)[^\"\s]+(\"?)"#).expect("valid uuid arg regex"),
            "$1********$2",
        ),
        (
            Regex::new(r#"(?i)(--username\s+\"?)[^\"\s]+(\"?)"#).expect("valid username arg regex"),
            "$1********$2",
        ),
    ]
});

impl LogChecker {
    pub fn new(client: Client, user_token: String) -> Self {
        Self { user_token, client }
    }

    pub fn check(&self, app_handle_clone_for_crash_handling: &AppHandle) {
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
        } else if self.is_options_corruption(log_string) {
            log_debug!("Detected options.txt corruption crash type");
            Some(CrashType::OptionsCorrupted)
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

    fn is_options_corruption(&self, log_string: &str) -> bool {
        log_string.contains("NumberFormatException: For input string:")
            && log_string.contains("GameSettings.loadOptions")
            || log_string.contains("java.lang.NoSuchMethodError")
                && log_string.contains("ITransformation")
            || log_string.contains("NoClassDefFoundError")
                && log_string.contains("Reflector")
    }

    fn handle_crash(&self, crash_type: CrashType, client_logs: &[String], app_handle: &AppHandle) {
        log_warn!(
            "Client {} crashed! Detected reason: {:?}",
            self.client.name,
            crash_type
        );

        self.send_crash_log(crash_type, client_logs);

        match crash_type {
            CrashType::MissingMainClass => {
                emit_to_main_window(
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
                emit_to_main_window(
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
                emit_to_main_window(
                    app_handle,
                    "client-crashed",
                    serde_json::json!({
                        "id": self.client.id,
                        "name": self.client.name.clone(),
                        "error": "JVM crash or uncaught exception"
                    }),
                );
            }
            CrashType::OptionsCorrupted => {
                self.handle_options_corruption(app_handle);
            }
        }
    }

    fn handle_options_corruption(&self, app_handle: &AppHandle) {
        let client_base = crate::core::storage::data::Data::get_filename(&self.client.filename);
        log_warn!(
            "Detected corrupted options for client '{}' (base={}), cleaning up and disabling sync",
            self.client.name,
            client_base
        );

        let client_dir = DATA
            .root_dir
            .lock()
            .ok()
            .map(|root| root.join(&client_base));

        if let Some(dir) = client_dir {
            if dir.exists() {
                for name in ["options.txt", "optionsof.txt"] {
                    let path = dir.join(name);
                    if path.exists() {
                        if let Err(e) = std::fs::remove_file(&path) {
                            log_warn!("Failed to remove {}: {}", path.display(), e);
                        } else {
                            log_info!("Removed corrupted {}", path.display());
                        }
                    }
                }
            }
        }

        block_options_sync(&client_base);

        let _ = std::fs::remove_file(DATA.root_dir.lock().ok()
            .map(|r| r.join("synced_options").join("options.txt")).unwrap_or_default());
        let _ = std::fs::remove_file(DATA.root_dir.lock().ok()
            .map(|r| r.join("synced_options").join("optionsof.txt")).unwrap_or_default());

        emit_to_main_window(
            app_handle,
            "client-options-reset",
            serde_json::json!({
                "id": self.client.id,
                "name": self.client.name.clone(),
                "reason": "Corrupted options.txt detected; files removed and sync disabled"
            }),
        );
    }

    fn emit_crash_details(&self, client_logs: &[String], app_handle: &AppHandle) {
        log_debug!(
            "Emitting client-crash-details for client '{}'",
            self.client.name
        );

        let sanitized_logs: Vec<String> = client_logs
            .iter()
            .map(|line| sanitize_log_line(line))
            .collect();

        emit_to_main_window(
            app_handle,
            "client-crash-details",
            serde_json::json!({
                "id": self.client.id,
                "name": self.client.name.clone(),
                "logs": sanitized_logs
            }),
        );
    }

    // PROJECT CLOSED
    #[allow(dead_code, unused)]
    fn send_crash_log(&self, crash_type: CrashType, client_logs: &[String]) {
        return;
        let api_base = {
            let selected = SERVERS.selected_api.read().ok().and_then(|g| g.clone());
            selected.map(|s| s.url)
        };

        let Some(api_base) = api_base else {
            log_warn!(
                "Skipping crash log upload for '{}' because no API server is selected",
                self.client.name
            );
            return;
        };

        let crash_type_str = match crash_type {
            CrashType::MissingMainClass => "MissingMainClass",
            CrashType::OutOfMemory => "OutOfMemory",
            CrashType::GameCrashed => "GameCrashed",
            CrashType::OptionsCorrupted => "OptionsCorrupted",
        }
        .to_string();

        let (log_content, line_count) = prepare_log_payload(client_logs);
        if log_content.is_empty() {
            return;
        }

        let endpoint = format!("{}api/{}/crash-logs", api_base, API_VERSION);

        let payload = CrashLogPayload {
            client_name: self.client.name.clone(),
            client_version: self.client.version.clone(),
            crash_type: crash_type_str,
            loader_version: env!("CARGO_PKG_VERSION").to_string(),
            os_name: std::env::consts::OS.to_string(),
            os_version: std::env::consts::ARCH.to_string(),
            line_count,
            log_content,
        };

        let token = self.user_token.clone();
        let client_name = self.client.name.clone();
        tokio::spawn(async move {
            let mut request = reqwest::Client::new().post(&endpoint).json(&payload);
            if !token.trim().is_empty() {
                request = request.bearer_auth(token);
            }
            let result = request.send().await;

            match result {
                Ok(response) if response.status().is_success() => {
                    log_info!("Crash logs uploaded for client '{}'", client_name);
                }
                Ok(response) => {
                    log_warn!(
                        "Crash log upload failed for '{}' with status {}",
                        client_name,
                        response.status()
                    );
                }
                Err(err) => {
                    log_error!(
                        "Crash log upload request failed for '{}': {}",
                        client_name,
                        err
                    );
                }
            }
        });
    }
}

fn prepare_log_payload(client_logs: &[String]) -> (String, usize) {
    let total_lines = client_logs.len();

    let mut selected = if total_lines > MAX_UPLOAD_LINES {
        let head_count = MAX_UPLOAD_LINES / 2;
        let tail_count = MAX_UPLOAD_LINES - head_count;
        let mut lines = Vec::with_capacity(MAX_UPLOAD_LINES + 1);
        lines.extend_from_slice(&client_logs[..head_count]);
        lines.push("...[TRIMMED MIDDLE LINES]...".to_string());
        lines.extend_from_slice(&client_logs[total_lines - tail_count..]);
        lines
    } else {
        client_logs.to_vec()
    };

    for line in &mut selected {
        *line = sanitize_log_line(line);
    }

    let mut combined = selected.join("\n");
    if combined.len() > MAX_UPLOAD_CHARS {
        let head_size = MAX_UPLOAD_CHARS / 2;
        let tail_size = MAX_UPLOAD_CHARS - head_size;
        let head = &combined[..head_size];
        let tail = &combined[combined.len() - tail_size..];
        combined = format!("{}\n...[TRIMMED MIDDLE]...\n{}", head, tail);
    }

    (combined, total_lines)
}

fn sanitize_log_line(input: &str) -> String {
    let mut output = input.replace('\0', "");
    for (pattern, replacement) in MASK_PATTERNS.iter() {
        output = pattern.replace_all(&output, *replacement).to_string();
    }
    output
}
