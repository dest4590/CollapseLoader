use lazy_static::lazy_static;
use serde::de::DeserializeOwned;
use std::{
    fs::{self, File},
    io::{BufReader, BufWriter},
};

use crate::api::core::data::DATA;
use crate::{log_debug, log_error, log_warn};

use super::servers::{Server, SERVERS};

pub const API_CACHE_DIR: &str = "cache/";

pub struct CAPI {
    pub api_server: Server,
}

fn sanitize_path_for_filename(path: &str) -> String {
    path.replace(['/', '\\'], "_") + ".json"
}

impl CAPI {
    pub fn json<T: DeserializeOwned>(&self, path: &str) -> Result<T, Box<dyn std::error::Error>> {
        let cache_dir = DATA.root_dir.join(API_CACHE_DIR);
        if !cache_dir.exists() {
            if let Err(e) = fs::create_dir_all(&cache_dir) {
                log_warn!(
                    "Failed to create API cache directory at {:?}: {}",
                    cache_dir,
                    e
                );
            }
        }

        let cache_file_name = sanitize_path_for_filename(path);
        let cache_file_path = cache_dir.join(cache_file_name);

        let cached_data: Option<serde_json::Value> = if cache_file_path.exists() {
            match File::open(&cache_file_path) {
                Ok(file) => {
                    let reader = BufReader::new(file);
                    match serde_json::from_reader(reader) {
                        Ok(data) => Some(data),
                        Err(e) => {
                            log_warn!(
                                "Failed to deserialize cached API response from {:?}: {}",
                                cache_file_path,
                                e
                            );
                            None
                        }
                    }
                }
                Err(e) => {
                    log_warn!(
                        "Failed to open cached API response file {:?}: {}",
                        cache_file_path,
                        e
                    );
                    None
                }
            }
        } else {
            None
        };

        let url = format!("{}{}", self.api_server.url, path);

        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
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
                    } else {
                        return Err(format!(
                            "API returned status {} and no cache available",
                            response.status()
                        )
                        .into());
                    }
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
                            } else {
                                return Err(
                                    "API returned empty response and no cache available".into()
                                );
                            }
                        }

                        match serde_json::from_str::<serde_json::Value>(&body) {
                            Ok(api_data) => {
                                let should_update_cache = match &cached_data {
                                    Some(cached) => *cached != api_data,
                                    None => true,
                                };

                                if should_update_cache {
                                    if cache_dir.exists() {
                                        match File::create(&cache_file_path) {
                                            Ok(file) => {
                                                let writer = BufWriter::new(file);
                                                if let Err(e) =
                                                    serde_json::to_writer_pretty(writer, &api_data)
                                                {
                                                    log_warn!(
                                                        "Failed to write API response to cache at {:?}: {}",
                                                        cache_file_path,
                                                        e
                                                    );
                                                } else {
                                                    log_debug!(
                                                        "Cache updated for API path: {} (data changed)",
                                                        path
                                                    );
                                                }
                                            }
                                            Err(e) => {
                                                log_warn!(
                                                    "Failed to create API cache file at {:?}: {}",
                                                    cache_file_path,
                                                    e
                                                );
                                            }
                                        }
                                    }
                                }

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

lazy_static! {
    pub static ref API: Option<CAPI> = {
        match SERVERS.selected_api_server.clone() {
            Some(api_s) => Some(CAPI { api_server: api_s }),
            _ => {
                log_warn!("Required API server or CDN server is not available. API functionality will be disabled.");
                None
            }
        }
    };
}
