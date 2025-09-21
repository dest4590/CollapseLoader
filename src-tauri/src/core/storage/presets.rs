use super::common::JsonStorage;
use super::data::DATA;
use serde::{Deserialize, Serialize};
use std::sync::LazyLock;
use std::{collections::HashMap, path::PathBuf, sync::Mutex as StdMutex};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ThemePreset {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub created_at: String,
    pub custom_css: String,
    pub enable_custom_css: bool,

    pub base100: Option<String>,
    pub base200: Option<String>,
    pub base300: Option<String>,
    pub base_content: Option<String>,

    pub primary: Option<String>,
    pub primary_content: Option<String>,
    pub secondary: Option<String>,
    pub secondary_content: Option<String>,
    pub accent: Option<String>,
    pub accent_content: Option<String>,
    pub neutral: Option<String>,
    pub neutral_content: Option<String>,
    pub info: Option<String>,
    pub info_content: Option<String>,
    pub success: Option<String>,
    pub success_content: Option<String>,
    pub warning: Option<String>,
    pub warning_content: Option<String>,
    pub error: Option<String>,
    pub error_content: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PresetManager {
    pub presets: HashMap<String, ThemePreset>,
    pub config_path: PathBuf,
}

impl Default for PresetManager {
    fn default() -> Self {
        Self {
            presets: HashMap::new(),
            config_path: DATA.get_local("presets.json"),
        }
    }
}

impl JsonStorage for PresetManager {
    fn file_path(&self) -> &PathBuf {
        &self.config_path
    }

    fn resource_name() -> &'static str {
        "presets"
    }

    fn create_default() -> Self {
        Self::default()
    }
}

impl PresetManager {
    pub fn add_preset(&mut self, preset: ThemePreset) -> Result<(), String> {
        if self.presets.contains_key(&preset.id) {
            return Err(format!(
                "Preset with ID '{id}' already exists",
                id = preset.id
            ));
        }

        self.presets.insert(preset.id.clone(), preset);
        self.save_to_disk();
        Ok(())
    }

    pub fn update_preset(&mut self, preset: ThemePreset) -> Result<(), String> {
        if !self.presets.contains_key(&preset.id) {
            return Err(format!("Preset with ID '{id}' not found", id = preset.id));
        }

        self.presets.insert(preset.id.clone(), preset);
        self.save_to_disk();
        Ok(())
    }

    pub fn delete_preset(&mut self, id: &str) -> Result<(), String> {
        if self.presets.remove(id).is_none() {
            return Err(format!("Preset with ID '{id}' not found"));
        }

        self.save_to_disk();
        Ok(())
    }

    pub fn get_preset(&self, id: &str) -> Option<&ThemePreset> {
        self.presets.get(id)
    }

    pub fn get_all_presets(&self) -> Vec<ThemePreset> {
        self.presets.values().cloned().collect()
    }

    pub fn preset_exists(&self, id: &str) -> bool {
        self.presets.contains_key(id)
    }
}

pub static PRESET_MANAGER: LazyLock<StdMutex<PresetManager>> = LazyLock::new(|| {
    StdMutex::new(PresetManager::load_from_disk(
        DATA.get_local("presets.json"),
    ))
});
