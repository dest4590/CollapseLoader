use lazy_static::lazy_static;
use std::{fs::File, io::BufReader, sync::Mutex};
use tauri::AppHandle;

use crate::{
    api::{
        core::data::DATA,
        network::api::{API, API_CACHE_DIR},
        utils,
    },
    log_debug, log_error, log_info, log_warn,
};

use super::client::Client;

pub struct ClientManager {
    pub clients: Vec<Client>,
}

impl ClientManager {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let api_option = API.as_ref();

        if let Some(api_instance) = api_option {
            let mut clients: Vec<Client> = match api_instance.json::<Vec<Client>>("clients") {
                Ok(clients) => {
                    log_info!("Successfully fetched {} clients from API", clients.len());
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
                        return Ok(ClientManager {
                            clients: Vec::new(),
                        });
                    }
                }
            };

            for client in &mut clients {
                if client.meta.is_new
                    != (semver::Version::parse(&client.version).unwrap().minor > 6)
                {
                    client.meta =
                        super::client::Meta::new(&client.version, &client.filename.clone());
                }

                client.meta.size = client.size;
            }

            log_debug!("ClientManager initialized with {} clients", clients.len());
            Ok(ClientManager { clients })
        } else {
            log_warn!("API instance not available. Attempting to load clients from cache.");
            let clients_cache_path = DATA.root_dir.join(API_CACHE_DIR).join("clients.json");

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
                    client.meta =
                        super::client::Meta::new(&client.version, &client.filename.clone());
                }

                client.meta.size = client.size;

                log_debug!(
                    "Client {} has cached size: {} MB",
                    client.name,
                    client.meta.size
                );
            }

            log_info!(
                "ClientManager initialized from cache with {} clients",
                clients.len()
            );
            Ok(ClientManager { clients })
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
        utils::emit_to_main_window(
            app_handle,
            "update-user-status",
            serde_json::json!({
                "status": "online",
                "currentClient": null,
                "clientVersion": null
            }),
        );

        Ok(())
    }
}

lazy_static! {
    pub static ref CLIENT_MANAGER: Mutex<Option<ClientManager>> = Mutex::new(None);
}

pub async fn initialize_client_manager() -> Result<(), String> {
    match ClientManager::new_async().await {
        Ok(manager) => {
            if let Ok(mut client_manager) = CLIENT_MANAGER.lock() {
                *client_manager = Some(manager);
                Ok(())
            } else {
                Err("Failed to acquire lock on CLIENT_MANAGER".to_string())
            }
        }
        Err(e) => {
            log_error!("Failed to initialize ClientManager: {}", e);

            if let Ok(mut client_manager) = CLIENT_MANAGER.lock() {
                *client_manager = Some(ClientManager {
                    clients: Vec::new(),
                });
            }
            Err(format!("Failed to initialize ClientManager: {e}"))
        }
    }
}
