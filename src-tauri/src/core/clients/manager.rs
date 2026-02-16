use rand::Rng;
use std::sync::{Arc, Mutex};
use std::{fs::File, io::BufReader};
use tauri::{AppHandle, Manager};

use super::client::Client;
use crate::core::clients::client::Meta;
use crate::core::utils::globals::MOCK_CLIENTS;
use crate::core::utils::helpers::emit_to_main_window;
use crate::log_error;
use crate::{
    core::{
        network::api::{API, API_CACHE_DIR},
        storage::data::DATA,
    },
    log_debug, log_info, log_warn,
};

#[derive(Default)]
pub struct ClientManager {
    pub clients: Vec<Client>,
}

impl ClientManager {
    async fn mock_clients() -> Result<Vec<Client>, Box<dyn std::error::Error + Send + Sync>> {
        let mut clients = Vec::new();
        let mut rng = rand::rng();

        for i in 1..5 {
            clients.push(Client {
                id: i,
                name: "Mock client #".to_owned() + &i.to_string(),
                version: "1.16.5".to_string(),
                rating_avg: Some(rng.random_range(10..=50) as f32 / 10.0),
                rating_count: Some(rng.random_range(0..=250)),
                meta: Meta {
                    asset_index: "1.16".to_string(),
                    is_new: false,
                    is_fabric: false,
                    is_forge: false,
                    installed: rng.random_bool(1.0 / 3.0),
                    is_custom: false,
                    size: rng.random_range(50..=100),
                },
                ..Default::default()
            });
        }

        Ok(clients)
    }

    pub fn get_client<F>(manager: &Arc<Mutex<Self>>, client_id: u32, f: F)
    where
        F: FnOnce(&mut Client),
    {
        if let Ok(mut mgr) = manager.lock() {
            if let Some(client) = mgr.clients.iter_mut().find(|c| c.id == client_id) {
                f(client);
            }
        }
    }

    pub async fn fetch_clients() -> Result<Vec<Client>, Box<dyn std::error::Error + Send + Sync>> {
        if *MOCK_CLIENTS {
            log_info!("Skipping client manager initialization, mock clients enabled, generating client list...");
            return Self::mock_clients().await;
        }

        let clients_task = tokio::task::spawn_blocking(|| {
            API.as_ref().map_or_else(
                || {
                    log_warn!("API instance not available. Attempting to load clients from cache.");
                    Self::load_from_cache("clients.json")
                },
                |api_instance| match api_instance.json::<Vec<Client>>("clients") {
                    Ok(clients) => Ok(clients),
                    Err(e) => {
                        log_warn!(
                            "Failed to fetch clients from API: {}. Attempting to load from cache.",
                            e
                        );
                        Self::load_from_cache("clients.json")
                    }
                },
            )
        });

        let fabric_clients_task = tokio::task::spawn_blocking(
            || -> Result<Vec<Client>, Box<dyn std::error::Error + Send + Sync>> {
                API.as_ref().map_or_else(
                    || {
                        log_warn!("API instance not available. Attempting to load fabric-clients from cache.");
                        Self::load_from_cache("fabric-clients.json")
                    },
                    |api_instance| match api_instance.json::<Vec<Client>>("fabric-clients") {
                        Ok(clients) => Ok(clients),
                        Err(e) => {
                            log_warn!(
                                "Failed to fetch fabric-clients from API: {}. Attempting to load from cache.",
                                e
                            );
                            Self::load_from_cache("fabric-clients.json")
                        }
                    },
                )
            },
        );

        let forge_clients_task = tokio::task::spawn_blocking(
            || -> Result<Vec<Client>, Box<dyn std::error::Error + Send + Sync>> {
                API.as_ref().map_or_else(
                    || {
                        log_warn!("API instance not available. Attempting to load forge-clients from cache.");
                        Self::load_from_cache("forge-clients.json")
                    },
                    |api_instance| match api_instance.json::<Vec<Client>>("forge-clients") {
                        Ok(clients) => Ok(clients),
                        Err(e) => {
                            log_warn!(
                                "Failed to fetch forge-clients from API: {}. Attempting to load from cache.",
                                e
                            );
                            Self::load_from_cache("forge-clients.json")
                        }
                    },
                )
            },
        );

        let (clients_res, fabric_res, forge_res) =
            tokio::join!(clients_task, fabric_clients_task, forge_clients_task);

        if clients_res.is_err() {
            log_error!("clients task join error: {:?}", clients_res.as_ref().err());
        }
        if fabric_res.is_err() {
            log_error!(
                "fabric clients task join error: {:?}",
                fabric_res.as_ref().err()
            );
        }
        if forge_res.is_err() {
            log_error!(
                "forge clients task join error: {:?}",
                forge_res.as_ref().err()
            );
        }

        let mut clients = clients_res.map_err(|e| {
            log_error!("clients task failed to join: {}", e);
            Box::new(e) as Box<dyn std::error::Error + Send + Sync>
        })??;
        let mut fabric_clients = fabric_res.map_err(|e| {
            log_error!("fabric clients task failed to join: {}", e);
            Box::new(e) as Box<dyn std::error::Error + Send + Sync>
        })??;
        let mut forge_clients = forge_res.map_err(|e| {
            log_error!("forge clients task failed to join: {}", e);
            Box::new(e) as Box<dyn std::error::Error + Send + Sync>
        })??;

        if !fabric_clients.is_empty() {
            clients.append(&mut fabric_clients);
        }

        if !forge_clients.is_empty() {
            clients.append(&mut forge_clients);
        }

        clients.sort_by(|a, b| b.created_at.cmp(&a.created_at));

        for client in &mut clients {
            client.meta = Meta::new(&client.version, &client.filename, &client.client_type);

            client.meta.size = client.size;

            if crate::core::utils::helpers::is_development_enabled() {
                client.working = true;
            }
        }

        log_debug!(
            "Initialized ClientManager with {} clients ({} fabric, {} forge, {} vanilla)",
            clients.len(),
            clients.iter().filter(|c| c.meta.is_fabric).count(),
            clients.iter().filter(|c| c.meta.is_forge).count(),
            clients.iter().filter(|c| !c.meta.is_fabric).count()
        );
        Ok(clients)
    }

    fn load_from_cache(
        filename: &str,
    ) -> Result<Vec<Client>, Box<dyn std::error::Error + Send + Sync>> {
        let cache_path = DATA
            .root_dir
            .lock()
            .unwrap()
            .join(API_CACHE_DIR)
            .join(filename);
        if cache_path.exists() {
            let file = File::open(&cache_path)
                .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;
            let reader = BufReader::new(file);
            let cached_clients: Vec<Client> = serde_json::from_reader(reader).map_err(|e| {
                log_warn!(
                    "Failed to deserialize cached clients from {}: {}",
                    cache_path.display(),
                    e
                );
                log_error!(
                    "Cache deserialize error for {}: {}",
                    cache_path.display(),
                    e
                );
                Box::new(e) as Box<dyn std::error::Error + Send + Sync>
            })?;
            log_debug!("Loaded {} clients from cache", cached_clients.len());
            Ok(cached_clients)
        } else {
            log_warn!("Clients cache not found. Returning empty client list.");
            Ok(Vec::new())
        }
    }

    pub fn update_status_on_client_exit(&self, app_handle: &AppHandle) -> Result<(), String> {
        emit_to_main_window(
            app_handle,
            "update-user-status",
            serde_json::json!({
                "status": "online",
                "currentClient": null,
            }),
        );

        let minimize_to_tray_on_launch = {
            let settings = crate::core::storage::settings::SETTINGS.lock().unwrap();
            settings.minimize_to_tray_on_launch.value
        };

        if minimize_to_tray_on_launch {
            let running_clients = Client::get_running_clients(&Arc::new(Mutex::new(Self {
                clients: self.clients.clone(),
            })));

            let running_custom_clients =
                crate::core::clients::custom_clients::CustomClient::get_running_custom_clients();

            if running_clients.is_empty() && running_custom_clients.is_empty() {
                if let Some(window) = app_handle.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
        }

        Ok(())
    }
}
