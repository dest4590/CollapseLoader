use crate::{
    core::utils::globals::{AUTH_SERVERS, CDN_SERVERS},
    log_info, log_warn,
};
use reqwest::blocking::Client;
use std::sync::LazyLock;
use std::{sync::Mutex, time::Duration};

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
    pub selected_cdn: Option<Server>,
    pub selected_auth: Option<Server>,
    pub connectivity_status: Mutex<ServerConnectivityStatus>,
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
        Self {
            cdns: CDN_SERVERS.to_vec(),
            auths: AUTH_SERVERS.to_vec(),
            selected_cdn: None,
            selected_auth: None,
            connectivity_status: Mutex::new(ServerConnectivityStatus {
                cdn_online: false,
                auth_online: false,
            }),
        }
    }

    pub fn check_servers(&mut self) {
        let client = Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .unwrap_or_default();

        for server in &self.cdns {
            let response_result = client.head(&server.url).send();
            match response_result {
                Ok(response) => {
                    log_info!(
                        "CDN Server {} responded with: {}",
                        server.url,
                        response.status()
                    );
                    self.selected_cdn = Some(server.clone());
                    break;
                }
                Err(e) => {
                    log_warn!("Failed to connect to CDN Server {}: {}", server.url, e);
                }
            }
        }

        for server in &self.auths {
            let response_result = client.head(&server.url).send();
            match response_result {
                Ok(response) => {
                    log_info!(
                        "Auth Server {} responded with: {}",
                        server.url,
                        response.status()
                    );
                    self.selected_auth = Some(server.clone());
                    break;
                }
                Err(e) => {
                    log_warn!("Failed to connect to Auth Server {}: {}", server.url, e);
                }
            }
        }

        self.set_status();
    }

    pub fn set_status(&self) -> ServerConnectivityStatus {
        let mut status = self.connectivity_status.lock().unwrap();
        status.cdn_online = self.selected_cdn.is_some();
        status.auth_online = self.selected_auth.is_some();
        status.clone()
    }

    pub fn get_auth_server_url(&self) -> Option<String> {
        self.selected_auth.as_ref().map(|server| server.url.clone())
    }
}

impl Default for Servers {
    fn default() -> Self {
        Self::new()
    }
}

pub static SERVERS: LazyLock<Servers> = LazyLock::new(|| {
    let mut servers = Servers::new();
    servers.check_servers();
    servers
});
