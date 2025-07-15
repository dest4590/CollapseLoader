use base64::{engine::general_purpose, Engine};

use crate::api::{core::data::DATA, globals::CODENAME, network::servers::SERVERS};

#[tauri::command]
pub fn get_version() -> Result<serde_json::Value, String> {
    let result = serde_json::json!({
      "version":  env!("CARGO_PKG_VERSION").to_string(),
      "codename": CODENAME,
      "commitHash": env!("GIT_HASH").to_string(),
    });

    Ok(result)
}

#[tauri::command]
pub fn open_data_folder() -> Result<String, String> {
    let path = DATA.root_dir.to_string_lossy().to_string();

    if let Err(e) = open::that(&path) {
        return Err(format!("Failed to open data folder: {}", e));
    }

    Ok(path)
}

#[tauri::command]
pub fn reset_requirements() -> Result<(), String> {
    if let Err(e) = DATA.reset_requirements() {
        return Err(format!("Failed to reset requirements: {}", e));
    }
    Ok(())
}

#[tauri::command]
pub fn reset_cache() -> Result<(), String> {
    if let Err(e) = DATA.reset_cache() {
        return Err(format!("Failed to reset cache: {}", e));
    }
    Ok(())
}

#[tauri::command]
pub fn get_auth_url() -> Result<String, String> {
    if let Some(auth_url) = SERVERS.get_auth_server_url() {
        Ok(auth_url)
    } else {
        Ok("https://auth.collapseloader.org".to_string())
    }
}

#[tauri::command]
pub async fn get_api_url() -> Result<String, String> {
    if let Some(api_url) = SERVERS.get_api_server_url() {
        Ok(api_url)
    } else {
        Ok("https://api.collapseloader.org".to_string())
    }
}

#[tauri::command]
pub async fn encode_base64(input: String) -> Result<String, String> {
    let encoded = general_purpose::STANDARD.encode(input);
    Ok(encoded)
}

#[tauri::command]
pub async fn decode_base64(input: String) -> Result<String, String> {
    match general_purpose::STANDARD.decode(&input) {
        Ok(decoded) => match String::from_utf8(decoded) {
            Ok(decoded_str) => Ok(decoded_str),
            Err(_) => Err("Failed to decode base64 to UTF-8 string".to_string()),
        },
        Err(_) => Err("Failed to decode base64".to_string()),
    }
}
