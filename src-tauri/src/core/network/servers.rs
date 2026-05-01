use crate::{
    core::utils::globals::{API_SERVERS, CDN_SERVERS},
    log_info, log_warn,
};
use reqwest::Client;
use serde::Serialize;
use std::sync::{LazyLock, Mutex, RwLock};
use std::time::Duration;
use tokio::sync::watch;

const REQUEST_TIMEOUT: Duration = Duration::from_secs(10);
const MAX_SERVER_CHECK_RETRIES: usize = 5;
const SERVER_CHECK_RETRY_DELAY: Duration = Duration::from_millis(300);

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Server {
    pub url: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ServerConnectivityStatus {
    pub cdn_online: bool,
    pub api_online: bool,
}

#[derive(Debug)]
pub struct Servers {
    pub cdns: Vec<Server>,
    pub apis: Vec<Server>,
    pub selected_cdn: RwLock<Option<Server>>,
    pub selected_api: RwLock<Option<Server>>,
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

    pub async fn check_servers(&self) {
        let client = super::create_client(REQUEST_TIMEOUT);

        tokio::join!(
            self.check_group(&client, &self.cdns, &self.selected_cdn, "CDN"),
            self.check_group(&client, &self.apis, &self.selected_api, "API")
        );

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
            // if CDN located on same server as API we should check API server instead of CDN, to reduce unused requests
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

            let mut ok;

            ok = self
                .probe_server(client, &url, "HEAD", &server.url, name)
                .await;

            if !ok {
                ok = self
                    .probe_server(client, &url, "GET", &server.url, name)
                    .await;
            }

            if ok {
                let mut lock = selected.write().unwrap();
                *lock = Some(server.clone());
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
        &self,
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
