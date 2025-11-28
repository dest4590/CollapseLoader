use rand::Rng;
use std::sync::{Arc, Mutex};
use std::{fs::File, io::BufReader};
use tauri::AppHandle;

use super::client::Client;
use crate::core::clients::client::Meta;
use crate::core::utils::globals::MOCK_CLIENTS;
use crate::core::utils::helpers::emit_to_main_window;
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
                meta: Meta {
                    asset_index: "1.16".to_string(),
                    is_new: false,
                    is_fabric: false,
                    installed: rng.random_bool(1.0 / 3.0),
                    is_custom: false,
                    size: rng.random_range(50..=100),
                },
                ..Default::default()
            });
        }

        Ok(clients)
    }

    pub fn get_client<F>(manager: &Arc<Mutex<ClientManager>>, client_id: u32, f: F)
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
            let api_option = API.as_ref();
            if let Some(api_instance) = api_option {
                match api_instance.json::<Vec<Client>>("clients") {
                    Ok(clients) => Ok(clients),
                    Err(e) => {
                        log_warn!(
                            "Failed to fetch clients from API: {}. Attempting to load from cache.",
                            e
                        );
                        Self::load_from_cache("clients.json")
                    }
                }
            } else {
                log_warn!("API instance not available. Attempting to load clients from cache.");
                Self::load_from_cache("clients.json")
            }
        });

        let fabric_clients_task = tokio::task::spawn_blocking(
            || -> Result<Vec<Client>, Box<dyn std::error::Error + Send + Sync>> {
                let api_option = API.as_ref();
                if let Some(api_instance) = api_option {
                    match api_instance.json::<Vec<Client>>("fabric-clients") {
                        Ok(clients) => Ok(clients),
                        Err(e) => {
                            log_warn!("Failed to fetch fabric clients: {}", e);
                            Ok(Vec::new())
                        }
                    }
                } else {
                    Ok(Vec::new())
                }
            },
        );

        let (clients_res, fabric_res) = tokio::join!(clients_task, fabric_clients_task);

        let mut clients =
            clients_res.map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)??;
        let mut fabric_clients =
            fabric_res.map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)??;

        if !fabric_clients.is_empty() {
            clients.append(&mut fabric_clients);
            clients.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        } else {
            log_debug!("API returned 0 fabric clients or failed to fetch");
        }

        for client in &mut clients {
            if client.meta.is_new != (semver::Version::parse(&client.version).unwrap().minor > 6) {
                client.meta = super::client::Meta::new(&client.version, &client.filename);
            }

            client.meta.size = client.size;
        }

        log_debug!(
            "Initialized ClientManager with {} clients ({} fabric, {} vanilla)",
            clients.len(),
            clients.iter().filter(|c| c.meta.is_fabric).count(),
            clients.iter().filter(|c| !c.meta.is_fabric).count()
        );
        Ok(clients)
    }

    fn load_from_cache(
        filename: &str,
    ) -> Result<Vec<Client>, Box<dyn std::error::Error + Send + Sync>> {
        let cache_path = DATA.root_dir.join(API_CACHE_DIR).join(filename);
        if cache_path.exists() {
            let file = File::open(cache_path)
                .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;
            let reader = BufReader::new(file);
            let cached_clients: Vec<Client> = serde_json::from_reader(reader).map_err(|e| {
                log_warn!("Failed to deserialize cached clients: {}", e);
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

        Ok(())
    }
}
