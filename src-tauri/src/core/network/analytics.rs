use crate::core::network::servers::SERVERS;
use crate::log_debug;
use std::thread;

pub struct Analytics;

impl Analytics {
    pub fn send_start_analytics() {
        Self::send_analytics("api/loader/launch", "start analytics");
    }

    pub fn send_client_analytics(client_id: u32) {
        let endpoint = format!("api/client/{client_id}/launch");
        Self::send_analytics(&endpoint, "client analytics");
    }

    pub fn send_client_download_analytics(client_id: u32) {
        let endpoint = format!("api/client/{client_id}/download");
        Self::send_analytics(&endpoint, "client download analytics");
    }

    fn send_analytics(endpoint: &str, analytics_type: &str) {
        let endpoint = endpoint.to_string();
        let analytics_type = analytics_type.to_string();

        thread::spawn(move || {
            let Some(server_url) = Self::get_server_url(&analytics_type) else {
                return;
            };

            let client = Self::create_client();
            let url = format!("{server_url}{endpoint}");

            match client.post(&url).send() {
                Ok(response) => {
                    log_debug!(
                        "{} sent successfully. Status: {}",
                        analytics_type
                            .chars()
                            .next()
                            .unwrap()
                            .to_uppercase()
                            .collect::<String>()
                            + &analytics_type[1..],
                        response.status()
                    );
                }
                Err(e) => {
                    log_debug!("Failed to send {}: {}", analytics_type, e);
                }
            }
        });
    }

    fn get_server_url(analytics_type: &str) -> Option<String> {
        match SERVERS.selected_auth.clone() {
            Some(server) => Some(server.url),
            None => {
                log_debug!("No Auth server selected for {}", analytics_type);
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
