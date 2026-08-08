pub mod analytics;
pub mod api;
pub mod cache;
pub mod downloader;
pub mod server_ads;
pub mod servers;

use crate::log_error;
use reqwest::{Client, ClientBuilder};
use std::sync::OnceLock;
use std::time::Duration;

/// Returns a lazily-initialized shared HTTP client with a 30-second timeout.
/// Used by both `commands/network.rs` and `core/network/server_ads.rs`.
pub fn get_api_client() -> &'static Client {
    static API_CLIENT: OnceLock<Client> = OnceLock::new();
    API_CLIENT.get_or_init(|| create_client(Duration::from_secs(30)))
}

pub fn user_agent() -> &'static str {
    static USER_AGENT: OnceLock<String> = OnceLock::new();

    USER_AGENT.get_or_init(|| format!("CollapseLoader_{}/tauri", env!("CARGO_PKG_VERSION")))
}

const CONNECT_TIMEOUT: Duration = Duration::from_secs(15);
const POOL_IDLE_PER_HOST: usize = 4;
const POOL_IDLE_TIMEOUT: Duration = Duration::from_secs(90);
const TCP_KEEPALIVE: Duration = Duration::from_secs(30);

pub fn create_client_builder() -> ClientBuilder {
    Client::builder()
        .user_agent(user_agent())
        .use_rustls_tls()
        .min_tls_version(reqwest::tls::Version::TLS_1_2)
        .connect_timeout(CONNECT_TIMEOUT)
        .tcp_keepalive(TCP_KEEPALIVE)
        .pool_max_idle_per_host(POOL_IDLE_PER_HOST)
        .pool_idle_timeout(POOL_IDLE_TIMEOUT)
}

pub fn create_blocking_client_builder() -> reqwest::blocking::ClientBuilder {
    reqwest::blocking::Client::builder()
        .user_agent(user_agent())
        .use_rustls_tls()
        .min_tls_version(reqwest::tls::Version::TLS_1_2)
        .connect_timeout(CONNECT_TIMEOUT)
        .tcp_keepalive(TCP_KEEPALIVE)
        .pool_max_idle_per_host(POOL_IDLE_PER_HOST)
        .pool_idle_timeout(POOL_IDLE_TIMEOUT)
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
