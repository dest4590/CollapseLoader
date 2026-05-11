use crate::{
    core::utils::globals::{API_SERVERS, CDN_SERVERS},
    log_info, log_warn,
};
use futures_util::StreamExt;
use reqwest::Client;
use serde::Serialize;
use std::sync::{LazyLock, Mutex, RwLock};
use std::time::Duration;
use tokio::sync::watch;

const REQUEST_TIMEOUT: Duration = Duration::from_secs(10);
const MAX_SERVER_CHECK_RETRIES: usize = 5;
const SERVER_CHECK_RETRY_DELAY: Duration = Duration::from_millis(300);

/// Represents a remote server (API or CDN).
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Server {
    /// The base URL of the server.
    pub url: String,
}

/// Represents the connectivity status of the application's infrastructure.
#[derive(Debug, Clone, Serialize)]
pub struct ServerConnectivityStatus {
    /// Whether at least one CDN server is reachable.
    pub cdn_online: bool,
    /// Whether at least one API server is reachable.
    pub api_online: bool,
}

/// Manages the collection of API and CDN servers, including health checks and selection.
#[derive(Debug)]
pub struct Servers {
    /// List of available CDN servers.
    pub cdns: Vec<Server>,
    /// List of available API servers.
    pub apis: Vec<Server>,
    /// The currently selected CDN server for downloads.
    pub selected_cdn: RwLock<Option<Server>>,
    /// The currently selected API server for requests.
    pub selected_api: RwLock<Option<Server>>,
    /// The overall connectivity status.
    pub connectivity_status: Mutex<ServerConnectivityStatus>,
    /// Sender for notifying when the initial server check is complete.
    pub check_complete_tx: watch::Sender<bool>,
    /// Receiver for waiting on the initial server check.
    pub check_complete_rx: watch::Receiver<bool>,
}

impl Server {
    /// Creates a new server instance with the given URL.
    pub fn new(url: &str) -> Self {
        Self {
            url: url.to_string(),
        }
    }
}

impl Servers {
    /// Creates a new `Servers` instance and initializes it with default servers.
    pub fn new() -> Self {
        let cdns = CDN_SERVERS.to_vec();
        let apis = API_SERVERS.to_vec();

        let initial_cdn = if std::env::var("FORCE_CDN").is_ok() && !cdns.is_empty() {
            Some(cdns[0].clone())
        } else {
            None
        };

        let initial_api = if std::env::var("FORCE_API").is_ok() && !apis.is_empty() {
            Some(apis[0].clone())
        } else {
            None
        };

        let (tx, rx) = watch::channel(false);

        Self {
            cdns,
            apis,
            selected_cdn: RwLock::new(initial_cdn),
            selected_api: RwLock::new(initial_api),
            connectivity_status: Mutex::new(ServerConnectivityStatus {
                cdn_online: false,
                api_online: false,
            }),
            check_complete_tx: tx,
            check_complete_rx: rx,
        }
    }

    /// Performs health checks on all servers and selects the best available ones.
    pub async fn check_servers(&self) {
        let client = super::create_client(REQUEST_TIMEOUT);

        tokio::join!(
            self.check_group(&client, &self.cdns, &self.selected_cdn, "CDN"),
            self.check_group(&client, &self.apis, &self.selected_api, "API")
        );

        self.set_status();
        let _ = self.check_complete_tx.send(true);
    }

    /// Waits until the initial server health check has completed.
    pub async fn wait_for_initial_check(&self) {
        let mut rx = self.check_complete_rx.clone();
        if *rx.borrow_and_update() {
            return;
        }
        let _ = rx.changed().await;
    }

    async fn check_group(
        &self,
        client: &Client,
        servers: &[Server],
        selected: &RwLock<Option<Server>>,
        name: &str,
    ) {
        let mut set = futures_util::stream::FuturesUnordered::new();

        for server in servers {
            let url = if name == "CDN" {
                if let Some(api_server) = self.apis.iter().find(|s| server.url.starts_with(&s.url))
                {
                    api_server.url.clone()
                } else {
                    server.url.clone()
                }
            } else {
                server.url.clone()
            };

            let client = client.clone();
            let server = server.clone();
            let name = name.to_string();

            set.push(async move {
                if Self::probe_server(&client, &url, "HEAD", &server.url, &name).await {
                    return Some(server);
                }
                if Self::probe_server(&client, &url, "GET", &server.url, &name).await {
                    return Some(server);
                }
                None
            });
        }

        while let Some(res) = set.next().await {
            if let Some(server) = res {
                let mut lock = selected.write().unwrap();
                *lock = Some(server);
                return;
            }
        }

        let mut lock = selected.write().unwrap();
        *lock = None;
        log_warn!(
            "{} server unreachable – all {} servers failed",
            name,
            servers.len()
        );
    }

    async fn probe_server(
        client: &Client,
        url: &str,
        method: &str,
        display_url: &str,
        name: &str,
    ) -> bool {
        let mut last_err: Option<String> = None;

        for attempt in 0..MAX_SERVER_CHECK_RETRIES {
            let result = if method == "HEAD" {
                client.head(url).send().await
            } else {
                client.get(url).send().await
            };

            match result {
                Ok(resp) => {
                    let status = resp.status();
                    if status.is_success() || status == reqwest::StatusCode::NOT_FOUND {
                        log_info!("{} [{}] {} – online", name, method, display_url);
                        return true;
                    }
                    last_err = Some(format!("HTTP {}", status.as_u16()));
                }
                Err(e) => {
                    last_err = Some(e.to_string());
                }
            }

            if attempt + 1 < MAX_SERVER_CHECK_RETRIES {
                tokio::time::sleep(SERVER_CHECK_RETRY_DELAY).await;
            }
        }

        log_warn!(
            "{} [{}] {} – unreachable after {} attempts: {}",
            name,
            method,
            display_url,
            MAX_SERVER_CHECK_RETRIES,
            last_err.as_deref().unwrap_or("unknown error")
        );
        false
    }

    pub fn set_status(&self) -> ServerConnectivityStatus {
        let mut status = self.connectivity_status.lock().unwrap();
        status.cdn_online = self.selected_cdn.read().unwrap().is_some();
        status.api_online = self.selected_api.read().unwrap().is_some();
        status.clone()
    }

    pub fn get_api_server_url(&self) -> Option<String> {
        self.selected_api
            .read()
            .unwrap()
            .as_ref()
            .map(|server| server.url.clone())
    }

    pub fn get_cdn_server_url(&self) -> Option<String> {
        self.selected_cdn
            .read()
            .unwrap()
            .as_ref()
            .map(|server| server.url.clone())
    }
}

impl Default for Servers {
    fn default() -> Self {
        Self::new()
    }
}

pub static SERVERS: LazyLock<Servers> = LazyLock::new(|| {
    // since 0.2.4 server checking moved to lib.rs init
    Servers::new()
});
