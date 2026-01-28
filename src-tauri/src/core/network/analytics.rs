use crate::core::network::servers::SERVERS;
use crate::core::utils::globals::API_VERSION;
use crate::log_debug;
use std::thread;

pub struct Analytics;

impl Analytics {
    pub fn send_start_analytics() {
        Self::send_analytics(
            &format!("api/{API_VERSION}/loader/launch"),
            "start analytics",
            None,
        );
    }

    pub fn send_client_analytics(client_id: u32, token: &str) {
        let endpoint = format!("api/{API_VERSION}/clients/launch/{client_id}");
        Self::send_analytics(&endpoint, "client analytics", Some(token));
    }

    pub fn send_client_download_analytics(client_id: u32, token: &str) {
        let endpoint = format!("api/{API_VERSION}/clients/download/{client_id}");
        Self::send_analytics(&endpoint, "client download analytics", Some(token));
    }

    fn send_analytics(endpoint: &str, analytics_type: &str, token: Option<&str>) {
        let endpoint = endpoint.to_string();
        let analytics_type = analytics_type.to_string();
        let token = token.map(|t| t.to_string());

        thread::spawn(move || {
            let Some(server_url) = Self::get_server_url(&analytics_type) else {
                return;
            };

            let client = Self::create_client();
            let url = format!("{server_url}{endpoint}");

            log_debug!("Sending {} to {}", analytics_type, url);

            match client.post(&url).bearer_auth(token.as_deref().unwrap_or("")).send() {
                Ok(_) => {}
                Err(e) => {
                    log_debug!("Failed to send {}: {}", analytics_type, e);
                }
            }
        });
    }

    fn get_server_url(analytics_type: &str) -> Option<String> {
        match SERVERS.selected_api.read().unwrap().clone() {
            Some(server) => Some(server.url),
            None => {
                log_debug!("No API server selected for {}", analytics_type);
                None
            }
        }
    }

    fn create_client() -> reqwest::blocking::Client {
        reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .unwrap_or_default()
    }
}
