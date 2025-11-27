use crate::{
    core::utils::globals::{AUTH_SERVERS, CDN_SERVERS},
    log_info, log_warn,
};
use backoff::{future::retry, ExponentialBackoff};
use reqwest::Client;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, LazyLock, Mutex, RwLock};
use std::time::{Duration, Instant};

const REQUEST_TIMEOUT: Duration = Duration::from_secs(5);
const CB_MAX_FAILURES: usize = 3;
const CB_RESET_WINDOW: Duration = Duration::from_secs(60);
const BACKOFF_MAX_ELAPSED: Duration = Duration::from_secs(10);

#[derive(Debug)]
pub struct CircuitBreaker {
    failures: AtomicUsize,
    last_failure: Mutex<Option<Instant>>,
}

impl CircuitBreaker {
    fn new() -> Self {
        Self {
            failures: AtomicUsize::new(0),
            last_failure: Mutex::new(None),
        }
    }

    fn record_failure(&self) {
        self.failures.fetch_add(1, Ordering::SeqCst);
        let mut last = self.last_failure.lock().unwrap();
        *last = Some(Instant::now());
    }

    fn record_success(&self) {
        self.failures.store(0, Ordering::SeqCst);
    }

    fn is_open(&self) -> bool {
        if self.failures.load(Ordering::SeqCst) < CB_MAX_FAILURES {
            return false;
        }
        let last = self.last_failure.lock().unwrap();
        if let Some(time) = *last {
            if time.elapsed() < CB_RESET_WINDOW {
                return true;
            }
        }
        false
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct Server {
    pub url: String,
    #[serde(skip)]
    pub circuit_breaker: Arc<CircuitBreaker>,
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
}

impl Server {
    pub fn new(url: &str) -> Self {
        Self {
            url: url.to_string(),
            circuit_breaker: Arc::new(CircuitBreaker::new()),
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

        Self {
            cdns,
            auths,
            selected_cdn: RwLock::new(initial_cdn),
            selected_auth: RwLock::new(initial_auth),
            connectivity_status: Mutex::new(ServerConnectivityStatus {
                cdn_online: false,
                auth_online: false,
            }),
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
    }

    async fn check_group(
        &self,
        client: &Client,
        servers: &[Server],
        selected: &RwLock<Option<Server>>,
        name: &str,
    ) {
        for server in servers {
            if server.circuit_breaker.is_open() {
                log_warn!(
                    "Skipping {} Server {} (Circuit Breaker Open)",
                    name,
                    server.url
                );
                continue;
            }

            let op = || async {
                let resp = client.head(&server.url).send().await.map_err(|e| {
                    backoff::Error::<Box<dyn std::error::Error + Send + Sync>>::transient(Box::new(
                        e,
                    ))
                })?;
                if !resp.status().is_success() {
                    return Err(
                        backoff::Error::<Box<dyn std::error::Error + Send + Sync>>::transient(
                            format!("Status not success: {}", resp.status()).into(),
                        ),
                    );
                }
                Ok(resp)
            };

            let backoff = ExponentialBackoff {
                max_elapsed_time: Some(BACKOFF_MAX_ELAPSED),
                ..Default::default()
            };

            match retry(backoff, op).await {
                Ok(response) => {
                    log_info!(
                        "{} Server {} responded with: {}",
                        name,
                        server.url,
                        response.status()
                    );
                    server.circuit_breaker.record_success();
                    let mut lock = selected.write().unwrap();
                    *lock = Some(server.clone());
                    return;
                }
                Err(e) => {
                    log_warn!("Failed to connect to {} Server {}: {}", name, server.url, e);
                    server.circuit_breaker.record_failure();
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
    let servers = Servers::new();
    tauri::async_runtime::spawn(async {
        SERVERS.check_servers().await;
    });
    servers
});
