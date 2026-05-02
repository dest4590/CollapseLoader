//! Client management logic for fetching and handling game clients.

use rand::RngExt;
use std::sync::{Arc, Mutex};
use std::{fs::File, io::BufReader};
use tauri::{AppHandle, Manager};

use super::client::Client;
use crate::core::clients::client::Meta;
use crate::core::utils::globals::{
    FABRIC_CLIENTS_URL, FORGE_CLIENTS_URL, MOCK_CLIENTS, VANILLA_CLIENTS_URL,
};
use crate::core::utils::helpers::emit_to_main_window;
use crate::log_error;
use crate::{
    core::{
        network::api::{API, API_CACHE_DIR},
        storage::data::DATA,
    },
    log_debug, log_info, log_warn,
};

type DynError = Box<dyn std::error::Error + Send + Sync>;

/// Manages the collection of game clients, including fetching from remote APIs and local caching.
#[derive(Default)]
pub struct ClientManager {
    /// The list of currently loaded clients.
    pub clients: Vec<Client>,
}

impl ClientManager {
    /// Generates a list of mock clients for testing purposes.
    fn mock_clients() -> Vec<Client> {
        let mut rng = rand::rng();

        (1..5)
            .map(|i| Client {
                id: i,
                name: format!("Mock client #{i}"),
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
            })
            .collect()
    }

    /// Fetches clients from a specific API endpoint, falling back to local cache on failure.
    fn fetch_clients_from_endpoint(
        endpoint: &str,
        cache_file: &str,
    ) -> Result<Vec<Client>, DynError> {
        let Some(api_instance) = API.as_ref() else {
            log_warn!(
                "API instance not available. Attempting to load {} from cache.",
                endpoint
            );
            return Self::load_from_cache(cache_file);
        };

        match api_instance.json::<Vec<Client>>(endpoint) {
            Ok(clients) => Ok(clients),
            Err(e) => {
                log_warn!(
                    "Failed to fetch {} from API: {}. Attempting to load from cache.",
                    endpoint,
                    e
                );
                Self::load_from_cache(cache_file)
            }
        }
    }

    /// Spawns an asynchronous task to fetch clients from an endpoint.
    fn spawn_clients_fetch_task(
        endpoint: &'static str,
        cache_file: &'static str,
    ) -> tokio::task::JoinHandle<Result<Vec<Client>, DynError>> {
        tokio::task::spawn_blocking(move || Self::fetch_clients_from_endpoint(endpoint, cache_file))
    }

    /// Resolves the result of a client fetch task.
    fn resolve_fetch_task_result(
        result: Result<Result<Vec<Client>, DynError>, tokio::task::JoinError>,
        label: &str,
    ) -> Result<Vec<Client>, DynError> {
        match result {
            Ok(inner) => inner,
            Err(e) => {
                log_error!("{} task failed to join: {}", label, e);
                Err(Box::new(e))
            }
        }
    }

    fn apply_client_metadata(clients: &mut [Client]) {
        let dev_enabled = crate::core::utils::helpers::is_development_enabled();

        for client in clients {
            client.meta = Meta::new(&client.version, &client.filename, &client.client_type);
            client.meta.size = client.size;

            if dev_enabled {
                client.working = true;
            }
        }
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

    /// Fetches all types of clients (Vanilla, Fabric, Forge) in parallel.
    ///
    /// This method orchestrates multiple asynchronous fetch tasks and merges their results.
    /// If all remote fetches fail, it returns the combined local cache.
    pub async fn fetch_clients() -> Result<Vec<Client>, DynError> {
        if *MOCK_CLIENTS {
            log_info!("Skipping client manager initialization, mock clients enabled, generating client list...");
            return Ok(Self::mock_clients());
        }

        let vanilla_task = Self::spawn_clients_fetch_task(VANILLA_CLIENTS_URL, "clients.json");
        let fabric_task = Self::spawn_clients_fetch_task(FABRIC_CLIENTS_URL, "fabric-clients.json");
        let forge_task = Self::spawn_clients_fetch_task(FORGE_CLIENTS_URL, "forge-clients.json");

        let (vanilla_res, fabric_res, forge_res) =
            tokio::join!(vanilla_task, fabric_task, forge_task);

        let mut all_clients = Vec::new();
        let mut any_success = false;

        let results = [
            (vanilla_res, "Vanilla"),
            (fabric_res, "Fabric"),
            (forge_res, "Forge"),
        ];

        for (res, label) in results {
            match Self::resolve_fetch_task_result(res, label) {
                Ok(clients) => {
                    all_clients.extend(clients);
                    any_success = true;
                }
                Err(e) => log_error!("Failed to fetch {} clients: {}", label, e),
            }
        }

        if !any_success {
            return Err("Failed to fetch any clients from API or cache".into());
        }

        all_clients.sort_by_key(|b| std::cmp::Reverse(b.created_at));
        Self::apply_client_metadata(&mut all_clients);

        log_debug!(
            "Initialized ClientManager with {} clients",
            all_clients.len()
        );

        Ok(all_clients)
    }


    fn load_from_cache(filename: &str) -> Result<Vec<Client>, DynError> {
        let cache_path = {
            let root_dir = DATA
                .root_dir
                .lock()
                .map_err(|e| std::io::Error::other(format!("Failed to lock root_dir: {e}")))?;
            root_dir.join(API_CACHE_DIR).join(filename)
        };

        if !cache_path.exists() {
            log_warn!("Clients cache not found. Returning empty client list.");
            return Ok(Vec::new());
        }

        let file = File::open(&cache_path)?;
        let reader = BufReader::new(file);
        let cached_clients: Vec<Client> = serde_json::from_reader(reader).map_err(|e| {
            log_warn!(
                "Failed to deserialize cached clients from {}: {}",
                cache_path.display(),
                e
            );
            DynError::from(e)
        })?;

        log_debug!("Loaded {} clients from cache", cached_clients.len());
        Ok(cached_clients)
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

        let minimize_to_tray_on_launch = crate::core::storage::settings::SETTINGS
            .lock()
            .map(|settings| settings.minimize_to_tray_on_launch.value)
            .map_err(|e| format!("Failed to lock settings: {e}"))?;

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
