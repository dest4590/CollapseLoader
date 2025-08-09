use crate::core::storage::data::DATA;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PluginFileInfo {
    pub id: String,
    pub name: String,
    pub enabled: bool,
    pub installed_at: String,
    pub size: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PluginManifest {
    pub plugins: Vec<PluginFileInfo>,
    pub version: String,
    pub last_modified: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PluginMetadata {
    pub id: String,
    pub name: String,
    pub version: String,
    pub author: String,
    pub description: String,
    pub icon: Option<String>,
    pub website: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PluginData {
    pub metadata: PluginMetadata,
    pub code: String,
    pub config: serde_json::Value,
    pub enabled: bool,
    pub installed_at: String,
}

fn get_plugins_dir() -> PathBuf {
    DATA.root_dir.join("plugins")
}

fn get_manifest_path() -> PathBuf {
    get_plugins_dir().join("manifest.json")
}

fn get_plugin_path(plugin_id: &str) -> PathBuf {
    get_plugins_dir().join(format!("{plugin_id}.json"))
}

fn ensure_plugins_dir() -> Result<(), String> {
    let plugins_dir = get_plugins_dir();
    if !plugins_dir.exists() {
        fs::create_dir_all(&plugins_dir)
            .map_err(|e| format!("Failed to create plugins directory: {e}"))?;
    }
    Ok(())
}

fn load_manifest() -> Result<PluginManifest, String> {
    let manifest_path = get_manifest_path();
    if !manifest_path.exists() {
        return Ok(PluginManifest {
            plugins: Vec::new(),
            version: "1.0.0".to_string(),
            last_modified: chrono::Utc::now().to_rfc3339(),
        });
    }

    let content =
        fs::read_to_string(&manifest_path).map_err(|e| format!("Failed to read manifest: {e}"))?;

    serde_json::from_str(&content).map_err(|e| format!("Failed to parse manifest: {e}"))
}

fn save_manifest(manifest: &PluginManifest) -> Result<(), String> {
    ensure_plugins_dir()?;
    let manifest_path = get_manifest_path();
    let content = serde_json::to_string_pretty(manifest)
        .map_err(|e| format!("Failed to serialize manifest: {e}"))?;

    fs::write(&manifest_path, content).map_err(|e| format!("Failed to write manifest: {e}"))
}

#[tauri::command]
pub fn get_plugins_manifest() -> Result<PluginManifest, String> {
    load_manifest()
}

#[tauri::command]
pub fn get_plugin_data(plugin_id: String) -> Result<PluginData, String> {
    let plugin_path = get_plugin_path(&plugin_id);
    if !plugin_path.exists() {
        return Err("Plugin not found".to_string());
    }

    let content =
        fs::read_to_string(&plugin_path).map_err(|e| format!("Failed to read plugin file: {e}"))?;

    serde_json::from_str(&content).map_err(|e| format!("Failed to parse plugin data: {e}"))
}

#[tauri::command]
pub fn save_plugin_data(plugin_data: PluginData) -> Result<(), String> {
    ensure_plugins_dir()?;

    let plugin_path = get_plugin_path(&plugin_data.metadata.id);
    let content = serde_json::to_string_pretty(&plugin_data)
        .map_err(|e| format!("Failed to serialize plugin data: {e}"))?;

    fs::write(&plugin_path, &content).map_err(|e| format!("Failed to write plugin file: {e}"))?;

    let mut manifest = load_manifest()?;
    let plugin_info = PluginFileInfo {
        id: plugin_data.metadata.id.clone(),
        name: plugin_data.metadata.name.clone(),
        enabled: plugin_data.enabled,
        installed_at: plugin_data.installed_at,
        size: content.len() as u64,
    };

    if let Some(existing) = manifest
        .plugins
        .iter_mut()
        .find(|p| p.id == plugin_data.metadata.id)
    {
        *existing = plugin_info;
    } else {
        manifest.plugins.push(plugin_info);
    }

    manifest.last_modified = chrono::Utc::now().to_rfc3339();
    save_manifest(&manifest)?;

    Ok(())
}

#[tauri::command]
pub fn delete_plugin(plugin_id: String) -> Result<(), String> {
    let plugin_path = get_plugin_path(&plugin_id);
    if plugin_path.exists() {
        fs::remove_file(&plugin_path).map_err(|e| format!("Failed to delete plugin file: {e}"))?;
    }

    let mut manifest = load_manifest()?;
    manifest.plugins.retain(|p| p.id != plugin_id);
    manifest.last_modified = chrono::Utc::now().to_rfc3339();
    save_manifest(&manifest)?;

    Ok(())
}

#[tauri::command]
pub fn update_plugin_enabled_status(plugin_id: String, enabled: bool) -> Result<(), String> {
    let mut plugin_data = get_plugin_data(plugin_id.clone())?;
    plugin_data.enabled = enabled;
    save_plugin_data(plugin_data)?;

    Ok(())
}

#[tauri::command]
pub fn get_plugin_code(plugin_id: String) -> Result<String, String> {
    let plugin_data = get_plugin_data(plugin_id)?;
    Ok(plugin_data.code)
}

#[tauri::command]
pub fn save_plugin_code(plugin_id: String, code: String) -> Result<(), String> {
    let mut plugin_data = get_plugin_data(plugin_id)?;
    plugin_data.code = code;
    save_plugin_data(plugin_data)?;
    Ok(())
}

#[tauri::command]
pub fn create_plugin_from_text(
    code: String,
    metadata: PluginMetadata,
    config: serde_json::Value,
) -> Result<(), String> {
    let plugin_data = PluginData {
        metadata,
        code,
        config,
        enabled: false,
        installed_at: chrono::Utc::now().to_rfc3339(),
    };

    save_plugin_data(plugin_data)
}
