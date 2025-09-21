use crate::core::storage::presets::{ThemePreset, PRESET_MANAGER};
use chrono::Utc;
use uuid::Uuid;

#[derive(serde::Deserialize)]
pub struct CreatePresetInput {
    pub name: String,
    pub description: Option<String>,
    pub custom_css: String,
    pub enable_custom_css: bool,

    // daisyui
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

#[derive(serde::Deserialize)]
pub struct UpdatePresetInput {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub custom_css: String,
    pub enable_custom_css: bool,

    // daisyui
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

#[tauri::command]
pub fn get_all_presets() -> Result<Vec<ThemePreset>, String> {
    PRESET_MANAGER
        .lock()
        .map(|p| p.get_all_presets())
        .map_err(|_| "Failed to get presets".to_string())
}

#[tauri::command]
pub fn get_preset(id: String) -> Result<Option<ThemePreset>, String> {
    let preset_manager = PRESET_MANAGER.lock().unwrap();
    Ok(preset_manager.get_preset(&id).cloned())
}

#[tauri::command]
pub fn create_preset(input: CreatePresetInput) -> Result<ThemePreset, String> {
    let mut preset_manager = PRESET_MANAGER.lock().unwrap();

    let preset = ThemePreset {
        id: Uuid::new_v4().to_string(),
        name: input.name,
        description: input.description,
        created_at: Utc::now().to_rfc3339(),
        custom_css: input.custom_css,
        enable_custom_css: input.enable_custom_css,

        base100: input.base100,
        base200: input.base200,
        base300: input.base300,
        base_content: input.base_content,

        primary: input.primary,
        primary_content: input.primary_content,
        secondary: input.secondary,
        secondary_content: input.secondary_content,
        accent: input.accent,
        accent_content: input.accent_content,
        neutral: input.neutral,
        neutral_content: input.neutral_content,
        info: input.info,
        info_content: input.info_content,
        success: input.success,
        success_content: input.success_content,
        warning: input.warning,
        warning_content: input.warning_content,
        error: input.error,
        error_content: input.error_content,
    };

    preset_manager.add_preset(preset.clone())?;
    drop(preset_manager);
    Ok(preset)
}

#[tauri::command]
pub fn update_preset(input: UpdatePresetInput) -> Result<ThemePreset, String> {
    let mut preset_manager = PRESET_MANAGER.lock().unwrap();

    if !preset_manager.preset_exists(&input.id) {
        return Err(format!("Preset with ID '{}' not found", input.id));
    }

    let existing_preset = preset_manager.get_preset(&input.id).unwrap();
    let created_at = existing_preset.created_at.clone();

    let preset = ThemePreset {
        id: input.id,
        name: input.name,
        description: input.description,
        created_at,
        custom_css: input.custom_css,
        enable_custom_css: input.enable_custom_css,

        base100: input.base100,
        base200: input.base200,
        base300: input.base300,
        base_content: input.base_content,

        primary: input.primary,
        primary_content: input.primary_content,
        secondary: input.secondary,
        secondary_content: input.secondary_content,
        accent: input.accent,
        accent_content: input.accent_content,
        neutral: input.neutral,
        neutral_content: input.neutral_content,
        info: input.info,
        info_content: input.info_content,
        success: input.success,
        success_content: input.success_content,
        warning: input.warning,
        warning_content: input.warning_content,
        error: input.error,
        error_content: input.error_content,
    };

    preset_manager.update_preset(preset.clone())?;
    drop(preset_manager);
    Ok(preset)
}

#[tauri::command]
pub fn delete_preset(id: String) -> Result<(), String> {
    let mut preset_manager = PRESET_MANAGER.lock().unwrap();
    preset_manager.delete_preset(&id)
}

#[tauri::command]
pub fn duplicate_preset(id: String, new_name: String) -> Result<ThemePreset, String> {
    let mut preset_manager = PRESET_MANAGER.lock().unwrap();

    let existing_preset = preset_manager
        .get_preset(&id)
        .ok_or_else(|| format!("Preset with ID '{id}' not found"))?;

    let new_preset = ThemePreset {
        id: Uuid::new_v4().to_string(),
        name: new_name,
        description: existing_preset.description.clone(),
        created_at: Utc::now().to_rfc3339(),
        custom_css: existing_preset.custom_css.clone(),
        enable_custom_css: existing_preset.enable_custom_css,

        base100: existing_preset.base100.clone(),
        base200: existing_preset.base200.clone(),
        base300: existing_preset.base300.clone(),
        base_content: existing_preset.base_content.clone(),

        primary: existing_preset.primary.clone(),
        primary_content: existing_preset.primary_content.clone(),
        secondary: existing_preset.secondary.clone(),
        secondary_content: existing_preset.secondary_content.clone(),
        accent: existing_preset.accent.clone(),
        accent_content: existing_preset.accent_content.clone(),
        neutral: existing_preset.neutral.clone(),
        neutral_content: existing_preset.neutral_content.clone(),
        info: existing_preset.info.clone(),
        info_content: existing_preset.info_content.clone(),
        success: existing_preset.success.clone(),
        success_content: existing_preset.success_content.clone(),
        warning: existing_preset.warning.clone(),
        warning_content: existing_preset.warning_content.clone(),
        error: existing_preset.error.clone(),
        error_content: existing_preset.error_content.clone(),
    };

    preset_manager.add_preset(new_preset.clone())?;
    drop(preset_manager);
    Ok(new_preset)
}
