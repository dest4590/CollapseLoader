use crate::{
    api::globals::{API_SERVERS, AUTH_SERVERS, CDN_SERVERS},
    log_info, log_warn,
};
use lazy_static::lazy_static;
use reqwest::blocking::Client;
use std::{sync::Mutex, time::Duration};

#[derive(Debug, Clone, serde::Serialize)]
pub struct Server {
    pub url: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ServerConnectivityStatus {
    pub cdn_online: bool,
    pub api_online: bool,
    pub auth_online: bool,
}

#[derive(Debug)]
pub struct Servers {
    pub cdn_servers: Vec<Server>,
    pub api_servers: Vec<Server>,
    pub auth_server: Vec<Server>,
    pub selected_cdn_server: Option<Server>,
    pub selected_api_server: Option<Server>,
    pub selected_auth_server: Option<Server>,
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
            cdn_servers: CDN_SERVERS.to_vec(),
            api_servers: API_SERVERS.to_vec(),
            auth_server: AUTH_SERVERS.to_vec(),
            selected_cdn_server: None,
            selected_api_server: None,
            selected_auth_server: None,
            connectivity_status: Mutex::new(ServerConnectivityStatus {
                cdn_online: false,
                api_online: false,
                auth_online: false,
            }),
        }
    }

    pub fn check_servers(&mut self) {
        let client = Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .unwrap_or_default();

        for server in &self.cdn_servers {
            let response_result = client.head(&server.url).send();
            match response_result {
                Ok(response) => {
                    log_info!(
                        "CDN Server {} responded with: {}",
                        server.url,
                        response.status()
                    );
                    self.selected_cdn_server = Some(server.clone());
                    break;
                }
                Err(e) => {
                    log_warn!("Failed to connect to CDN Server {}: {}", server.url, e);
                }
            }
        }

        for server in &self.api_servers {
            let response_result = client.head(&server.url).send();
            match response_result {
                Ok(response) => {
                    log_info!(
                        "API Server {} responded with: {}",
                        server.url,
                        response.status()
                    );
                    self.selected_api_server = Some(server.clone());
                    break;
                }
                Err(e) => {
                    log_warn!("Failed to connect to API Server {}: {}", server.url, e);
                }
            }
        }

        for server in &self.auth_server {
            let response_result = client.head(&server.url).send();
            match response_result {
                Ok(response) => {
                    log_info!(
                        "Auth Server {} responded with: {}",
                        server.url,
                        response.status()
                    );
                    self.selected_auth_server = Some(server.clone());
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
        status.cdn_online = self.selected_cdn_server.is_some();
        status.api_online = self.selected_api_server.is_some();
        status.auth_online = self.selected_auth_server.is_some();
        status.clone()
    }

    pub fn get_auth_server_url(&self) -> Option<String> {
        self.selected_auth_server
            .as_ref()
            .map(|server| server.url.clone())
    }

    pub fn get_api_server_url(&self) -> Option<String> {
        self.selected_api_server
            .as_ref()
            .map(|server| server.url.clone())
    }
}

lazy_static! {
    #[derive(Debug)]
    pub static ref SERVERS: Servers = {
        let mut servers = Servers::new();
        servers.check_servers();
        servers
    };
}
