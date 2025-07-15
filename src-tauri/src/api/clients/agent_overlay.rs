use crate::api::core::data::DATA;
use crate::api::network::servers::SERVERS;
use crate::{log_debug, log_error, log_info, log_warn};
use base64::Engine;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AgentOverlayInfo {
    pub agent_hash: String,
    pub overlay_hash: String,
}

#[derive(Serialize)]
pub struct AgentArguments {
    token: String,
    client_name: String,
    analytics: bool,
    cordshare: bool,
    ircchat: bool,
}

impl AgentArguments {
    pub fn new(
        token: String,
        client_name: String,
        analytics: bool,
        cordshare: bool,
        ircchat: bool,
    ) -> Self {
        Self {
            token,
            client_name,
            analytics,
            cordshare,
            ircchat,
        }
    }
    pub fn encrypt(&self) -> String {
        let json = serde_json::to_string(self).unwrap_or_else(|e| {
            log_error!("Failed to serialize AgentArguments: {}", e);
            "{}".to_string()
        });
        base64::engine::general_purpose::STANDARD.encode(json)
    }

    pub fn log_info(&self) {
        log_info!(
            "Running client with this agent arguments: Token: {}, Client Name: {}, Analytics: {}, Cordshare: {}, IRC Chat: {}",
            "*".repeat(self.token.len() / 2),
            self.client_name,
            self.analytics,
            self.cordshare,
            self.ircchat
        );
    }
}

pub struct AgentOverlayManager;

impl AgentOverlayManager {
    fn get_api_base_url() -> Result<String, String> {
        SERVERS
            .selected_auth_server
            .as_ref()
            .map(|server| server.url.clone())
            .ok_or_else(|| "No API server available".to_string())
    }

    pub async fn download_agent_overlay_files() -> Result<(), String> {
        log_debug!("Starting download of agent and overlay files...");

        let info = Self::get_agent_overlay_info().await?;

        let folder = DATA.root_dir.join("agent_overlay");
        if !folder.exists() {
            fs::create_dir_all(&folder)
                .map_err(|e| format!("Failed to create directory: {}", e))?;
        }

        let agent_path = folder.join("CollapseAgent.jar");
        let overlay_path = folder.join("CollapseOverlay.dll");

        let base_url = Self::get_api_base_url()?;

        log_debug!("Downloading agent file...");
        Self::download_file(&format!("{}api/agent/download/", base_url), &agent_path).await?;

        let downloaded_hash = Self::calculate_md5_hash(&agent_path)?;
        if downloaded_hash != info.agent_hash {
            return Err(format!(
                "Agent file hash mismatch. Expected: {}, Got: {}",
                info.agent_hash, downloaded_hash
            ));
        }

        log_debug!("Downloading overlay file...");
        Self::download_file(&format!("{}api/overlay/download/", base_url), &overlay_path).await?;

        let downloaded_overlay_hash = Self::calculate_md5_hash(&overlay_path)?;
        if downloaded_overlay_hash != info.overlay_hash {
            return Err(format!(
                "Overlay file hash mismatch. Expected: {}, Got: {}",
                info.overlay_hash, downloaded_overlay_hash
            ));
        }

        log_debug!("Agent and overlay files download completed successfully");
        Ok(())
    }

    async fn get_agent_overlay_info() -> Result<AgentOverlayInfo, String> {
        let base_url = Self::get_api_base_url()?;
        let url = format!("{}api/agent-overlay/", base_url);

        let client = reqwest::Client::new();
        let response = client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("Failed to get agent/overlay info: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("Backend returned error: {}", response.status()));
        }

        let info: AgentOverlayInfo = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse agent/overlay info: {}", e))?;

        Ok(info)
    }

    async fn download_file(url: &str, path: &PathBuf) -> Result<(), String> {
        let client = reqwest::Client::new();
        let response = client
            .get(url)
            .send()
            .await
            .map_err(|e| format!("Failed to download file: {}", e))?;

        if !response.status().is_success() {
            return Err(format!(
                "Download failed with status: {}",
                response.status()
            ));
        }

        let bytes = response
            .bytes()
            .await
            .map_err(|e| format!("Failed to read file bytes: {}", e))?;

        fs::write(path, bytes).map_err(|e| format!("Failed to write file to disk: {}", e))?;

        Ok(())
    }

    fn calculate_md5_hash(path: &PathBuf) -> Result<String, String> {
        let bytes =
            fs::read(path).map_err(|e| format!("Failed to read file for hashing: {}", e))?;

        let digest = md5::compute(&bytes);
        Ok(format!("{:x}", digest))
    }

    pub async fn verify_agent_overlay_files() -> Result<bool, String> {
        log_debug!("Verifying agent and overlay files...");

        let folder = DATA.root_dir.join("agent_overlay");
        if !folder.exists() {
            fs::create_dir_all(&folder)
                .map_err(|e| format!("Failed to create directory: {}", e))?;
        }

        let agent_path = folder.join("CollapseAgent.jar");
        let overlay_path = folder.join("CollapseOverlay.dll");

        if !agent_path.exists() || !overlay_path.exists() {
            log_warn!("Agent or overlay files are missing");
            return Ok(false);
        }

        let info = Self::get_agent_overlay_info().await?;

        let agent_hash = Self::calculate_md5_hash(&agent_path)?;
        if agent_hash != info.agent_hash {
            log_error!(
                "Agent file hash verification failed. Expected: {}, Got: {}",
                info.agent_hash,
                agent_hash
            );
            return Ok(false);
        }

        let overlay_hash = Self::calculate_md5_hash(&overlay_path)?;
        if overlay_hash != info.overlay_hash {
            log_error!(
                "Overlay file hash verification failed. Expected: {}, Got: {}",
                info.overlay_hash,
                overlay_hash
            );
            return Ok(false);
        }

        log_debug!("Agent and overlay files verified successfully");
        Ok(true)
    }
}
