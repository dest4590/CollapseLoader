use std::{
    fs::File,
    io::BufReader,
    sync::{LazyLock, Mutex},
};
use tauri::AppHandle;

use super::client::Client;
use crate::core::utils::helpers::emit_to_main_window;
use crate::{
    core::{
        network::api::{API, API_CACHE_DIR},
        storage::data::DATA,
    },
    log_debug, log_error, log_info, log_warn,
};

pub struct ClientManager {
    pub clients: Vec<Client>,
}

impl ClientManager {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        log_debug!("ClientManager::new() starting initialization");
        let api_option = API.as_ref();

        if let Some(api_instance) = api_option {
            log_debug!("API instance available — fetching clients from API");
            let mut clients: Vec<Client> = match api_instance.json::<Vec<Client>>("clients") {
                Ok(clients) => {
                    log_info!("Fetched {} clients from API", clients.len());
                    clients
                }
                Err(e) => {
                    log_warn!(
                        "Failed to fetch clients from API: {}. Attempting to load from cache.",
                        e
                    );
                    let cache_path = DATA.root_dir.join(API_CACHE_DIR).join("clients.json");
                    if cache_path.exists() {
                        let file = File::open(cache_path)?;
                        let reader = BufReader::new(file);
                        let cached_clients: Vec<Client> =
                            serde_json::from_reader(reader).map_err(|e| {
                                log_warn!("Failed to deserialize cached clients: {}", e);
                                Box::new(e) as Box<dyn std::error::Error>
                            })?;
                        log_debug!("Loaded {} clients from cache", cached_clients.len());
                        cached_clients
                    } else {
                        log_warn!("Clients cache not found. Returning empty client list.");
                        return Ok(Self {
                            clients: Vec::new(),
                        });
                    }
                }
            };

            match api_instance.json::<Vec<Client>>("fabric-clients") {
                Ok(mut fabric_clients) => {
                    let fabric_count = fabric_clients.len();
                    if fabric_count > 0 {
                        log_info!("Fetched {} fabric clients from API", fabric_count);
                        clients.append(&mut fabric_clients);
                        clients.sort_by(|a, b| b.created_at.cmp(&a.created_at));
                    } else {
                        log_debug!("API returned 0 fabric clients");
                    }
                }
                Err(e) => {
                    log_warn!("Failed to fetch fabric clients: {}", e);
                }
            }

            for client in &mut clients {
                if client.meta.is_new
                    != (semver::Version::parse(&client.version).unwrap().minor > 6)
                {
                    client.meta = super::client::Meta::new(&client.version, &client.filename);
                }

                client.meta.size = client.size;
            }

            log_debug!("ClientManager initialized with {} clients", clients.len());

            Ok(Self { clients })
        } else {
            log_warn!("API instance not available. Attempting to load clients from cache.");
            let clients_cache_path = DATA.root_dir.join(API_CACHE_DIR).join("clients.json");
            log_debug!(
                "Looking for cached clients at {}",
                clients_cache_path.display()
            );

            let mut clients: Vec<Client> = if clients_cache_path.exists() {
                let file = File::open(clients_cache_path)?;
                let reader = BufReader::new(file);
                let cached_clients = serde_json::from_reader(reader).unwrap_or_else(|e| {
                    log_warn!(
                        "Error deserializing cached clients: {}. Returning empty list.",
                        e
                    );
                    Vec::new()
                });
                log_info!(
                    "Loaded {} clients from cache (API offline)",
                    cached_clients.len()
                );
                cached_clients
            } else {
                log_warn!("Clients cache not found when API offline. Returning empty list.");
                Vec::new()
            };

            for client in &mut clients {
                if client.meta.is_new
                    != (semver::Version::parse(&client.version).unwrap().minor > 6)
                {
                    client.meta = super::client::Meta::new(&client.version, &client.filename);
                }

                client.meta.size = client.size;
            }

            log_info!(
                "ClientManager initialized from cache with {} clients — operating offline mode",
                clients.len()
            );
            Ok(Self { clients })
        }
    }

    pub async fn new_async() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        tokio::task::spawn_blocking(|| {
            Self::new().map_err(|e| -> Box<dyn std::error::Error + Send + Sync> {
                Box::new(std::io::Error::other(e.to_string()))
            })
        })
        .await?
    }

    pub fn update_status_on_client_exit(&self, app_handle: &AppHandle) -> Result<(), String> {
        log_debug!("Updating user status on client exit to 'online' and clearing currentClient");
        emit_to_main_window(
            app_handle,
            "update-user-status",
            serde_json::json!({
                "status": "online",
                "currentClient": null,
            }),
        );

        Ok(())
    }
}

pub static CLIENT_MANAGER: LazyLock<Mutex<Option<ClientManager>>> =
    LazyLock::new(|| Mutex::new(None));

pub async fn initialize_client_manager() -> Result<(), String> {
    match ClientManager::new_async().await {
        Ok(manager) => {
            log_info!("ClientManager async initialization succeeded — setting global manager");
            CLIENT_MANAGER.lock().map_or_else(
                |_| {
                    log_error!("Failed to acquire lock on CLIENT_MANAGER during initialization");
                    Err("Failed to acquire lock on CLIENT_MANAGER".to_string())
                },
                |mut client_manager| {
                    *client_manager = Some(manager);
                    Ok(())
                },
            )
        }
        Err(e) => {
            log_error!("Failed to initialize ClientManager: {}", e);
            log_warn!("Falling back to empty ClientManager instance");

            if let Ok(mut client_manager) = CLIENT_MANAGER.lock() {
                *client_manager = Some(ClientManager {
                    clients: Vec::new(),
                });
                log_debug!("CLIENT_MANAGER set to empty instance after failure");
            }
            Err(format!("Failed to initialize ClientManager: {e}"))
        }
    }
}
