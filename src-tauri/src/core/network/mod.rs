pub mod analytics;
pub mod api;
pub mod cache;
pub mod downloader;
pub mod servers;

use crate::log_error;
use reqwest::{Client, ClientBuilder};
use std::time::Duration;

use std::sync::OnceLock;

pub fn user_agent() -> &'static str {
    static USER_AGENT: OnceLock<String> = OnceLock::new();

    USER_AGENT.get_or_init(|| format!("CollapseLoader_{}/tauri", env!("CARGO_PKG_VERSION")))
}

pub fn create_client_builder() -> ClientBuilder {
    Client::builder()
        .user_agent(user_agent())
        .use_rustls_tls()
        .min_tls_version(reqwest::tls::Version::TLS_1_2)
        .max_tls_version(reqwest::tls::Version::TLS_1_2)
}

pub fn create_blocking_client_builder() -> reqwest::blocking::ClientBuilder {
    reqwest::blocking::Client::builder()
        .user_agent(user_agent())
        .use_rustls_tls()
        .min_tls_version(reqwest::tls::Version::TLS_1_2)
        .max_tls_version(reqwest::tls::Version::TLS_1_2)
}

pub fn create_client(timeout: Duration) -> Client {
    create_client_builder()
        .timeout(timeout)
        .build()
        .unwrap_or_else(|e| {
            log_error!("Failed to build async HTTP client: {}", e);
            Client::new()
        })
}

pub fn create_blocking_client(timeout: Duration) -> reqwest::blocking::Client {
    create_blocking_client_builder()
        .timeout(timeout)
        .build()
        .unwrap_or_else(|e| {
            log_error!("Failed to build blocking HTTP client: {}", e);
            reqwest::blocking::Client::new()
        })
}
