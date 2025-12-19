use crate::{
    core::utils::globals::{AUTH_SERVERS, CDN_SERVERS},
    log_info, log_warn,
};
use reqwest::Client;
use std::sync::{LazyLock, Mutex, RwLock};
use std::time::Duration;
use tokio::sync::watch;

const REQUEST_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Debug, Clone, serde::Serialize)]
pub struct Server {
    pub url: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ServerConnectivityStatus {
    pub cdn_online: bool,
    pub auth_online: bool,
}

#[derive(Debug)]
pub struct Servers {
    pub cdns: Vec<Server>,
    pub auths: Vec<Server>,
    pub selected_cdn: RwLock<Option<Server>>,
    pub selected_auth: RwLock<Option<Server>>,
    pub connectivity_status: Mutex<ServerConnectivityStatus>,
    pub check_complete_tx: watch::Sender<bool>,
    pub check_complete_rx: watch::Receiver<bool>,
}

impl Server {
    pub fn new(url: &str) -> Self {
        Self {
            url: url.to_string(),
        }
    }
}

impl Servers {
    pub fn new() -> Self {
        let cdns = CDN_SERVERS.to_vec();
        let auths = AUTH_SERVERS.to_vec();

        let initial_cdn = if std::env::var("FORCE_CDN").is_ok() && !cdns.is_empty() {
            Some(cdns[0].clone())
        } else {
            None
        };

        let initial_auth = if std::env::var("FORCE_AUTH").is_ok() && !auths.is_empty() {
            Some(auths[0].clone())
        } else {
            None
        };

        let (tx, rx) = watch::channel(false);

        Self {
            cdns,
            auths,
            selected_cdn: RwLock::new(initial_cdn),
            selected_auth: RwLock::new(initial_auth),
            connectivity_status: Mutex::new(ServerConnectivityStatus {
                cdn_online: false,
                auth_online: false,
            }),
            check_complete_tx: tx,
            check_complete_rx: rx,
        }
    }

    pub async fn check_servers(&self) {
        let client = Client::builder()
            .timeout(REQUEST_TIMEOUT)
            .build()
            .unwrap_or_default();

        self.check_group(&client, &self.cdns, &self.selected_cdn, "CDN")
            .await;
        self.check_group(&client, &self.auths, &self.selected_auth, "Auth")
            .await;

        self.set_status();
        let _ = self.check_complete_tx.send(true);
    }

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
        for server in servers {
            match client.head(&server.url).send().await {
                Ok(resp) => {
                    if resp.status().is_success() {
                        log_info!(
                            "{} Server {} responded with: {}",
                            name,
                            server.url,
                            resp.status()
                        );
                        let mut lock = selected.write().unwrap();
                        *lock = Some(server.clone());
                        return;
                    } else {
                        log_warn!(
                            "{} Server {} returned status: {}",
                            name,
                            server.url,
                            resp.status()
                        );
                    }
                }
                Err(e) => {
                    log_warn!("Failed to connect to {} Server {}: {}", name, server.url, e);
                }
            }
        }

        let mut lock = selected.write().unwrap();
        *lock = None;
    }

    pub fn set_status(&self) -> ServerConnectivityStatus {
        let mut status = self.connectivity_status.lock().unwrap();
        status.cdn_online = self.selected_cdn.read().unwrap().is_some();
        status.auth_online = self.selected_auth.read().unwrap().is_some();
        status.clone()
    }

    pub fn get_auth_server_url(&self) -> Option<String> {
        self.selected_auth
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
