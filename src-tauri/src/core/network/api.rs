use serde::{de::DeserializeOwned, Deserialize};
use std::sync::LazyLock;
use std::time::Duration;

use crate::core::storage::data::DATA;
use crate::core::utils::globals::API_VERSION;
use crate::{log_info, log_warn};

use super::cache;
use super::servers::{Server, SERVERS};

pub const API_CACHE_DIR: &str = "cache";
pub const API_MAX_RETRIES: usize = 5;

#[derive(Deserialize)]
struct ApiResponse {
    success: Option<bool>,
    data: Option<serde_json::Value>,
    error: Option<String>,
}

fn extract_api_response(body: &str) -> Result<serde_json::Value, String> {
    match serde_json::from_str::<ApiResponse>(body) {
        Ok(api_data) => {
            if api_data.success.is_none() || api_data.data.is_none() {
                serde_json::from_str::<serde_json::Value>(body).map_err(|e| e.to_string())
            } else if api_data.success.unwrap_or(false) {
                Ok(api_data.data.unwrap())
            } else {
                let err_msg = api_data
                    .error
                    .unwrap_or_else(|| "Unknown API error".to_string());
                Err(format!("API error: {}", err_msg))
            }
        }
        Err(_) => serde_json::from_str::<serde_json::Value>(body).map_err(|e| e.to_string()),
    }
}

pub struct Api {
    pub api_server: Server,
}

impl Api {
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
                // let url = format!("{}api/{}/{}", server.url, API_VERSION, path);

                let url = format!("{}{}", server.url, path);

                // println!("url is {}", url);

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

                    let body = response.text().unwrap_or_default();
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

        match fetch_network() {
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
