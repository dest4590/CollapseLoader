use serde::de::DeserializeOwned;
use std::sync::LazyLock;
use std::time::Duration;

use crate::core::storage::data::DATA;
use crate::{log_debug, log_error, log_warn};

use super::servers::{Server, SERVERS};
use super::cache;

pub const API_CACHE_DIR: &str = "cache/";

pub struct Api {
    pub api_server: Server,
}

impl Api {
    pub fn json<T: DeserializeOwned>(&self, path: &str) -> Result<T, Box<dyn std::error::Error>> {
        let cache_dir = DATA.root_dir.join(API_CACHE_DIR);
        cache::ensure_cache_dir(&cache_dir);

        let cache_file_path = cache::cache_file_path(&cache_dir, path);

        let cached_data: Option<serde_json::Value> = cache::read_cached_json(&cache_file_path);

        let url = format!("{}{}", self.api_server.url, path);

        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|e| {
                log_error!("Failed to create HTTP client for API request: {}", e);
                Box::new(e) as Box<dyn std::error::Error>
            })?;

        match client.get(&url).send() {
            Ok(response) => {
                if !response.status().is_success() {
                    log_warn!("API returned non-success status: {}", response.status());
                    if let Some(cached) = cached_data {
                        log_debug!(
                            "Using cached data due to API error status for path: {}",
                            path
                        );
                        let result: T = serde_json::from_value(cached)?;
                        return Ok(result);
                    }
                    return Err(format!(
                        "API returned status {} and no cache available",
                        response.status()
                    )
                    .into());
                }

                match response.text() {
                    Ok(body) => {
                        if body.is_empty() {
                            log_warn!("API returned empty response body");
                            if let Some(cached) = cached_data {
                                log_debug!(
                                    "Using cached data due to empty API response for path: {}",
                                    path
                                );
                                let result: T = serde_json::from_value(cached)?;
                                return Ok(result);
                            }
                            return Err("API returned empty response and no cache available".into());
                        }

                        match serde_json::from_str::<serde_json::Value>(&body) {
                            Ok(api_data) => {
                                cache::write_cache_if_changed(&cache_file_path, &api_data, &cached_data);

                                let result: T = serde_json::from_value(api_data)?;
                                Ok(result)
                            }
                            Err(e) => {
                                log_warn!(
                                    "Failed to parse API response as JSON for path {}: {}",
                                    path,
                                    e
                                );
                                log_debug!(
                                    "API response body that failed to parse: {}",
                                    &body[..std::cmp::min(body.len(), 500)]
                                );
                                if let Some(cached) = cached_data {
                                    log_debug!(
                                        "Using cached data due to API JSON parse error for path: {}",
                                        path
                                    );
                                    let result: T = serde_json::from_value(cached)?;
                                    Ok(result)
                                } else {
                                    Err(Box::new(e))
                                }
                            }
                        }
                    }
                    Err(e) => {
                        log_warn!("Failed to read API response body for path {}: {}", path, e);
                        if let Some(cached) = cached_data {
                            log_debug!(
                                "Using cached data due to API response read error for path: {}",
                                path
                            );
                            let result: T = serde_json::from_value(cached)?;
                            Ok(result)
                        } else {
                            Err(Box::new(e))
                        }
                    }
                }
            }
            Err(e) => {
                log_warn!("Failed to fetch from API path {}: {}", path, e);
                if let Some(cached) = cached_data {
                    log_debug!(
                        "Using cached data due to API fetch error for path: {}",
                        path
                    );
                    let result: T = serde_json::from_value(cached)?;
                    Ok(result)
                } else {
                    Err(Box::new(e))
                }
            }
        }
    }
}

pub static API: LazyLock<Option<Api>> = LazyLock::new(|| {
    SERVERS
        .selected_auth
        .read()
        .unwrap()
        .clone()
        .map_or_else(
            || {
                log_warn!("Required Auth server or CDN server is not available. API functionality will be disabled.");
                None
            },
            |auth_s| Some(Api { api_server: auth_s }),
        )
});
