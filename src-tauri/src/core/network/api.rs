//! API client for interacting with remote servers, including caching and retry logic.

use serde::de::DeserializeOwned;
use std::sync::LazyLock;
use std::time::Duration;

use crate::core::storage::data::APP_HANDLE;
use crate::core::storage::data::DATA;
use crate::core::utils::globals::API_VERSION;
use crate::core::utils::helpers::emit_to_main_window;
use crate::{log_info, log_warn};

use super::cache;
use super::servers::{Server, SERVERS};

/// Directory name for API response caching.
pub const API_CACHE_DIR: &str = "cache";
/// Maximum number of retries for a failed API request.
pub const API_MAX_RETRIES: usize = 5;

/// Extracts the data payload from a raw API response string.
///
/// This function handles both the standard wrapped response format and
/// direct JSON payloads for backward compatibility.
fn extract_api_response(body: &str) -> Result<serde_json::Value, String> {
    let parsed: serde_json::Value = serde_json::from_str(body).map_err(|e| e.to_string())?;

    if let Some(object) = parsed.as_object() {
        if let Some(success) = object.get("success") {
            match success.as_bool() {
                Some(true) => {
                    return object
                        .get("data")
                        .cloned()
                        .ok_or_else(|| "API response marked successful but missing data".to_string());
                }
                Some(false) => {
                    let err_msg = object
                        .get("error")
                        .and_then(|value| value.as_str())
                        .unwrap_or("Unknown API error");
                    return Err(format!("API error: {err_msg}"));
                }
                None => {
                    return Err("API response contains a non-boolean success field".to_string());
                }
            }
        }
    }

    Ok(parsed)
}

/// A client for making requests to the application's API servers.
pub struct Api {
    /// The primary API server to use.
    pub api_server: Server,
}

impl Api {
    /// Performs a GET request to the specified path and deserializes the response.
    ///
    /// This method handles caching, server failover, and retries.
    pub fn json<T: DeserializeOwned>(&self, path: &str) -> Result<T, Box<dyn std::error::Error>> {
        let cache_dir = DATA.root_dir.lock().unwrap().join(API_CACHE_DIR);
        cache::ensure_cache_dir(&cache_dir);

        let cache_file_path = cache::cache_file_path(&cache_dir, path);
        let cached_data: Option<serde_json::Value> = cache::read_cached_json(&cache_file_path);

        let fetch_network = || -> Result<serde_json::Value, String> {
            let client = super::create_blocking_client(Duration::from_secs(5));

            let mut apis = SERVERS.apis.clone();
            if apis.is_empty() {
                apis.push(self.api_server.clone());
            }

            let preferred = SERVERS.selected_api.read().unwrap().clone();
            let start_index = preferred
                .as_ref()
                .and_then(|ps| apis.iter().position(|s| s.url == ps.url))
                .or_else(|| apis.iter().position(|s| s.url == self.api_server.url))
                .unwrap_or(0);

            for server in apis.iter().cycle().skip(start_index).take(apis.len()) {
                let url = format!("{}{}", server.url, path);

                for attempt in 1..=API_MAX_RETRIES {
                    if let Some(app_handle) = APP_HANDLE.lock().unwrap().as_ref() {
                        let initial = serde_json::json!({
                            "id": uuid::Uuid::new_v4().to_string(),
                            "method": "GET",
                            "url": url,
                            "status": null,
                            "duration": null,
                            "timestamp": chrono::Utc::now().timestamp_millis() as u64,
                            "request_headers": null,
                            "request_body": null,
                            "response_headers": null,
                            "response_size": null,
                            "response_body": null,
                            "response_text": null,
                            "error_message": null,
                        });
                        emit_to_main_window(app_handle, "network-request", initial);
                    }
                    if attempt > 1 {
                        log_info!(
                            "Retrying API request (attempt {}/{}) for path: {} on server {}",
                            attempt,
                            API_MAX_RETRIES,
                            path,
                            server.url
                        );
                    }

                    let response = match client.get(&url).send() {
                        Ok(res) => res,
                        Err(e) => {
                            if attempt < API_MAX_RETRIES {
                                std::thread::sleep(Duration::from_secs(attempt as u64));
                                continue;
                            }
                            log_warn!(
                                "Failed to reach API server {} for path: {} after {} attempts: {}",
                                server.url,
                                path,
                                API_MAX_RETRIES,
                                e
                            );
                            break;
                        }
                    };

                    let status = response.status();
                    if !status.is_success() {
                        log_warn!(
                            "API returned non-success status {} for path: {} (attempt {}/{})",
                            status,
                            path,
                            attempt,
                            API_MAX_RETRIES
                        );
                        if (status.is_server_error() || status.as_u16() == 429)
                            && attempt < API_MAX_RETRIES
                        {
                            std::thread::sleep(Duration::from_secs(attempt as u64));
                            continue;
                        }
                        return Err(format!("API returned status {}", status));
                    }

                    let status_uint = response.status().as_u16();
                    let body = response.text().unwrap_or_default();
                    if body.is_empty() {
                        return Err("API returned empty response".to_string());
                    }

                    if let Some(app_handle) = APP_HANDLE.lock().unwrap().as_ref() {
                        let parsed = serde_json::from_str::<serde_json::Value>(&body).ok();
                        let rec = serde_json::json!({
                            "id": uuid::Uuid::new_v4().to_string(),
                            "method": "GET",
                            "url": url,
                            "status": status_uint,
                            "duration": 0,
                            "timestamp": chrono::Utc::now().timestamp_millis() as u64,
                            "request_headers": null,
                            "request_body": null,
                            "response_headers": null,
                            "response_size": body.len() as u64,
                            "response_body": parsed,
                            "response_text": if parsed.is_none() { Some(body.clone()) } else { None },
                            "error_message": null,
                        });
                        emit_to_main_window(app_handle, "network-response", rec);
                    }

                    match extract_api_response(&body) {
                        Ok(data_value) => {
                            *SERVERS.selected_api.write().unwrap() = Some(server.clone());
                            return Ok(data_value);
                        }
                        Err(e) => return Err(e),
                    }
                }
            }

            Err("Exceeded maximum API retry attempts across all servers".to_string())
        };

        match fetch_network() {
            Ok(data_value) => {
                cache::write_cache_if_changed(&cache_file_path, &data_value, &cached_data);
                Ok(serde_json::from_value(data_value)?)
            }
            Err(err_msg) => {
                if let Some(cached) = cached_data {
                    log_warn!(
                        "Using cached API response for path {} after network failure: {}",
                        path,
                        err_msg
                    );
                    Ok(serde_json::from_value(cached)?)
                } else {
                    Err(format!("{} and no cache available", err_msg).into())
                }
            }
        }
    }

    pub async fn json_async<T: DeserializeOwned>(
        &self,
        path: &str,
    ) -> Result<T, Box<dyn std::error::Error>> {
        let cache_dir = DATA.root_dir.lock().unwrap().join(API_CACHE_DIR);
        cache::ensure_cache_dir(&cache_dir);

        let cache_file_path = cache::cache_file_path(&cache_dir, path);
        let cached_data: Option<serde_json::Value> = cache::read_cached_json(&cache_file_path);

        let fetch_network = async {
            let client = super::create_client(Duration::from_secs(5));

            let mut apis = SERVERS.apis.clone();
            if apis.is_empty() {
                apis.push(self.api_server.clone());
            }

            let preferred = SERVERS.selected_api.read().unwrap().clone();
            let start_index = preferred
                .as_ref()
                .and_then(|ps| apis.iter().position(|s| s.url == ps.url))
                .or_else(|| apis.iter().position(|s| s.url == self.api_server.url))
                .unwrap_or(0);

            for server in apis.iter().cycle().skip(start_index).take(apis.len()) {
                let url = format!("{}api/{}/{}", server.url, API_VERSION, path);

                for attempt in 1..=API_MAX_RETRIES {
                    if attempt > 1 {
                        log_info!(
                            "Retrying API request (attempt {}/{}) for path: {} on server {}",
                            attempt,
                            API_MAX_RETRIES,
                            path,
                            server.url
                        );
                    }

                    let response = match client.get(&url).send().await {
                        Ok(res) => res,
                        Err(e) => {
                            if attempt < API_MAX_RETRIES {
                                tokio::time::sleep(Duration::from_secs(attempt as u64)).await;
                                continue;
                            }
                            log_warn!(
                                "Failed to reach API server {} for path: {} after {} attempts: {}",
                                server.url,
                                path,
                                API_MAX_RETRIES,
                                e
                            );
                            break;
                        }
                    };

                    let status = response.status();
                    if !status.is_success() {
                        log_warn!(
                            "API returned non-success status {} for path: {} (attempt {}/{})",
                            status,
                            path,
                            attempt,
                            API_MAX_RETRIES
                        );
                        if (status.is_server_error() || status.as_u16() == 429)
                            && attempt < API_MAX_RETRIES
                        {
                            tokio::time::sleep(Duration::from_secs(attempt as u64)).await;
                            continue;
                        }
                        return Err(format!("API returned status {}", status));
                    }

                    let body = response.text().await.unwrap_or_default();
                    if body.is_empty() {
                        return Err("API returned empty response".to_string());
                    }

                    match extract_api_response(&body) {
                        Ok(data_value) => {
                            *SERVERS.selected_api.write().unwrap() = Some(server.clone());
                            return Ok(data_value);
                        }
                        Err(e) => return Err(e),
                    }
                }
            }

            Err("Exceeded maximum API retry attempts across all servers".to_string())
        };

        match fetch_network.await {
            Ok(data_value) => {
                cache::write_cache_if_changed(&cache_file_path, &data_value, &cached_data);
                Ok(serde_json::from_value(data_value)?)
            }
            Err(err_msg) => {
                if let Some(cached) = cached_data {
                    Ok(serde_json::from_value(cached)?)
                } else {
                    Err(format!("{} and no cache available", err_msg).into())
                }
            }
        }
    }
}

pub static API: LazyLock<Option<Api>> = LazyLock::new(|| {
    SERVERS
        .selected_api
        .read()
        .unwrap()
        .clone()
        .map(|api_server| Api { api_server })
});
