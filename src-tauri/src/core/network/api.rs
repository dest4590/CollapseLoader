use serde::de::DeserializeOwned;
use std::sync::LazyLock;
use std::time::Duration;

use crate::core::storage::data::DATA;
use crate::core::utils::globals::API_VERSION;
use crate::{log_debug, log_error, log_info, log_warn};

use super::cache;
use super::servers::{Server, SERVERS};

pub const API_CACHE_DIR: &str = "cache";
pub const API_MAX_RETRIES: usize = 5;

pub struct Api {
    pub api_server: Server,
}

impl Api {
    pub fn json<T: DeserializeOwned>(&self, path: &str) -> Result<T, Box<dyn std::error::Error>> {
        let cache_dir = DATA.root_dir.lock().unwrap().join(API_CACHE_DIR);
        cache::ensure_cache_dir(&cache_dir);

        let cache_file_path = cache::cache_file_path(&cache_dir, path);
        let cached_data: Option<serde_json::Value> = cache::read_cached_json(&cache_file_path);

        let url = format!("{}api/{}/{}", self.api_server.url, API_VERSION, path);
        let client = super::create_blocking_client(Duration::from_secs(5));

        for attempt in 0..API_MAX_RETRIES {
            if attempt > 0 {
                log_info!(
                    "Retrying API request (attempt {}/{}) for path: {}",
                    attempt + 1,
                    API_MAX_RETRIES,
                    path
                );
            }

            match client.get(&url).send() {
                Ok(response) => {
                    let status = response.status();
                    if !status.is_success() {
                        log_warn!(
                            "API returned non-success status {} for path: {} (attempt {}/{})",
                            status,
                            path,
                            attempt + 1,
                            API_MAX_RETRIES
                        );
                        if (status.is_server_error() || status.as_u16() == 429)
                            && attempt + 1 < API_MAX_RETRIES
                        {
                            std::thread::sleep(Duration::from_secs((attempt + 1) as u64));
                            continue;
                        }
                        if let Some(cached) = &cached_data {
                            log_debug!(
                                "Using cached data due to API error status for path: {}",
                                path
                            );
                            let result: T = serde_json::from_value(cached.clone())?;
                            return Ok(result);
                        }
                        return Err(format!(
                            "API returned status {} and no cache available",
                            status
                        )
                        .into());
                    }

                    match response.text() {
                        Ok(body) => {
                            if body.is_empty() {
                                log_warn!("API returned empty response body for path: {}", path);
                                if let Some(cached) = &cached_data {
                                    log_debug!(
                                        "Using cached data due to empty API response for path: {}",
                                        path
                                    );
                                    let result: T = serde_json::from_value(cached.clone())?;
                                    return Ok(result);
                                }
                                return Err(
                                    "API returned empty response and no cache available".into()
                                );
                            }

                            match serde_json::from_str::<serde_json::Value>(&body) {
                                Ok(api_data) => {
                                    if let (Some(success), Some(data), Some(_timestamp)) = (
                                        api_data.get("success"),
                                        api_data.get("data"),
                                        api_data.get("timestamp"),
                                    ) {
                                        let success = success.as_bool().unwrap_or(false);
                                        if success {
                                            let data_value = data.clone();
                                            cache::write_cache_if_changed(
                                                &cache_file_path,
                                                &data_value,
                                                &cached_data,
                                            );
                                            let result: T = serde_json::from_value(data_value)?;
                                            return Ok(result);
                                        } else {
                                            let err_msg = api_data
                                                .get("error")
                                                .and_then(|v| v.as_str())
                                                .unwrap_or("API returned an error");
                                            log_warn!(
                                                "API returned error for path {}: {}",
                                                path,
                                                err_msg
                                            );
                                            if let Some(cached) = &cached_data {
                                                log_debug!("Using cached data due to API error wrapper for path: {}", path);
                                                let result: T =
                                                    serde_json::from_value(cached.clone())?;
                                                return Ok(result);
                                            }
                                            return Err(format!("API error: {}", err_msg).into());
                                        }
                                    } else {
                                        log_error!("API response does not match required ApiResponse<T> format for path: {}", path);
                                        if let Some(cached) = &cached_data {
                                            log_debug!("Using cached data due to invalid API response format for path: {}", path);
                                            let result: T = serde_json::from_value(cached.clone())?;
                                            return Ok(result);
                                        }
                                        return Err("API response does not match required ApiResponse<T> format and no cache available".into());
                                    }
                                }
                                Err(e) => {
                                    log_warn!(
                                        "Failed to parse API response as JSON for path {}: {}",
                                        path,
                                        e
                                    );
                                    log_debug!(
                                        "API response body that failed to parse (truncated): {}",
                                        &body[..std::cmp::min(body.len(), 500)]
                                    );
                                    if let Some(cached) = &cached_data {
                                        log_debug!("Using cached data due to API JSON parse error for path: {}", path);
                                        let result: T = serde_json::from_value(cached.clone())?;
                                        return Ok(result);
                                    }
                                    log_error!(
                                        "JSON parse error and no cache available for path {}: {}",
                                        path,
                                        e
                                    );
                                    return Err(Box::new(e));
                                }
                            }
                        }
                        Err(e) => {
                            log_warn!("Failed to read API response body for path {}: {}", path, e);
                            if attempt + 1 < API_MAX_RETRIES {
                                std::thread::sleep(Duration::from_secs((attempt + 1) as u64));
                                continue;
                            }
                            if let Some(cached) = &cached_data {
                                log_debug!(
                                    "Using cached data due to API response read error for path: {}",
                                    path
                                );
                                let result: T = serde_json::from_value(cached.clone())?;
                                return Ok(result);
                            }
                            log_error!(
                                "Failed to read response and no cache available for path {}: {}",
                                path,
                                e
                            );
                            return Err(Box::new(e));
                        }
                    }
                }
                Err(e) => {
                    log_warn!(
                        "Failed to fetch from API path {}: {} (attempt {}/{})",
                        path,
                        e,
                        attempt + 1,
                        API_MAX_RETRIES
                    );
                    if attempt + 1 < API_MAX_RETRIES {
                        std::thread::sleep(Duration::from_secs((attempt + 1) as u64));
                        continue;
                    }
                    if let Some(cached) = &cached_data {
                        log_debug!(
                            "Using cached data due to API fetch error for path: {}",
                            path
                        );
                        let result: T = serde_json::from_value(cached.clone())?;
                        return Ok(result);
                    }
                    log_error!(
                        "Failed to fetch API and no cache present for path {}: {}",
                        path,
                        e
                    );
                    return Err(Box::new(e));
                }
            }
        }

        Err("Exceeded maximum API retry attempts and no cache available".into())
    }
}

pub static API: LazyLock<Option<Api>> = LazyLock::new(|| {
    SERVERS
        .selected_api
        .read()
        .unwrap()
        .clone()
        .map_or_else(|| None, |api_s| Some(Api { api_server: api_s }))
});
