use crate::core::network::game_proxy::{self, GameProxyConfig, GameProxyStatus};
use crate::core::storage::settings::SETTINGS;

fn current_proxy_config() -> Result<GameProxyConfig, String> {
    let settings = SETTINGS
        .lock()
        .map_err(|_| "Failed to read Minecraft proxy settings.".to_string())?;
    Ok(GameProxyConfig {
        proxy_type: settings.game_proxy_type.value.clone(),
        proxy_host: settings.game_proxy_host.value.clone(),
        proxy_port: settings.game_proxy_port.value,
        username: settings.game_proxy_username.value.clone(),
        password: settings.game_proxy_password.value.clone(),
    })
}

#[tauri::command]
pub fn get_game_proxy_status() -> GameProxyStatus {
    game_proxy::game_proxy_status()
}

#[tauri::command]
pub async fn test_game_proxy(server_host: String, server_port: u16) -> Result<(), String> {
    game_proxy::test_proxy_connection(current_proxy_config()?, &server_host, server_port).await
}
