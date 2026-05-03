use crate::core::storage::presets::{ThemePreset, PRESET_MANAGER};
use crate::{log_debug, log_info, log_warn};
use chrono::Utc;
use uuid::Uuid;

#[derive(Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PresetFields {
    pub name: String,
    pub description: Option<String>,
    #[serde(rename = "customCSS", alias = "customCss")]
    pub custom_css: String,
    #[serde(rename = "enableCustomCSS", alias = "enableCustomCss")]
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
    #[serde(rename = "backgroundImage")]
    pub background_image: Option<String>,
    #[serde(rename = "backgroundBlur")]
    pub background_blur: Option<f64>,
    #[serde(rename = "backgroundOpacity")]
    pub background_opacity: Option<f64>,
}

impl PresetFields {
    fn from_preset(preset: &ThemePreset, name: String) -> Self {
        Self {
            name,
            description: preset.description.clone(),
            custom_css: preset.custom_css.clone(),
            enable_custom_css: preset.enable_custom_css,
            base100: preset.base100.clone(),
            base200: preset.base200.clone(),
            base300: preset.base300.clone(),
            base_content: preset.base_content.clone(),
            primary: preset.primary.clone(),
            primary_content: preset.primary_content.clone(),
            secondary: preset.secondary.clone(),
            secondary_content: preset.secondary_content.clone(),
            accent: preset.accent.clone(),
            accent_content: preset.accent_content.clone(),
            neutral: preset.neutral.clone(),
            neutral_content: preset.neutral_content.clone(),
            info: preset.info.clone(),
            info_content: preset.info_content.clone(),
            success: preset.success.clone(),
            success_content: preset.success_content.clone(),
            warning: preset.warning.clone(),
            warning_content: preset.warning_content.clone(),
            error: preset.error.clone(),
            error_content: preset.error_content.clone(),
            background_image: preset.background_image.clone(),
            background_blur: preset.background_blur,
            background_opacity: preset.background_opacity,
        }
    }
}

fn build_preset(id: String, created_at: String, fields: PresetFields) -> ThemePreset {
    ThemePreset {
        id,
        name: fields.name,
        description: fields.description,
        created_at,
        custom_css: fields.custom_css,
        enable_custom_css: fields.enable_custom_css,
        base100: fields.base100,
        base200: fields.base200,
        base300: fields.base300,
        base_content: fields.base_content,
        primary: fields.primary,
        primary_content: fields.primary_content,
        secondary: fields.secondary,
        secondary_content: fields.secondary_content,
        accent: fields.accent,
        accent_content: fields.accent_content,
        neutral: fields.neutral,
        neutral_content: fields.neutral_content,
        info: fields.info,
        info_content: fields.info_content,
        success: fields.success,
        success_content: fields.success_content,
        warning: fields.warning,
        warning_content: fields.warning_content,
        error: fields.error,
        error_content: fields.error_content,
        background_image: fields.background_image,
        background_blur: fields.background_blur,
        background_opacity: fields.background_opacity,
    }
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreatePresetInput {
    #[serde(flatten)]
    pub preset: PresetFields,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdatePresetInput {
    pub id: String,
    #[serde(flatten)]
    pub preset: PresetFields,
}

#[tauri::command]
pub fn get_all_presets() -> Result<Vec<ThemePreset>, String> {
    log_debug!("Fetching all theme presets");
    PRESET_MANAGER
        .lock()
        .map(|p| p.get_all_presets())
        .map_err(|e| {
            log_warn!("Failed to get presets: {}", e);
            "Failed to get presets".to_string()
        })
}

#[tauri::command]
pub fn get_preset(id: String) -> Result<Option<ThemePreset>, String> {
    log_debug!("Fetching theme preset with ID: {}", id);
    let preset_manager = PRESET_MANAGER.lock().unwrap();
    Ok(preset_manager.get_preset(&id).cloned())
}

#[tauri::command]
pub fn create_preset(input: CreatePresetInput) -> Result<ThemePreset, String> {
    log_info!(
        "Creating new theme preset with name: '{}'",
        input.preset.name
    );
    let mut preset_manager = PRESET_MANAGER.lock().unwrap();

    let preset = build_preset(
        Uuid::new_v4().to_string(),
        Utc::now().to_rfc3339(),
        input.preset,
    );

    preset_manager.add_preset(preset.clone())?;
    log_info!(
        "Successfully created and saved new preset with ID: {}",
        preset.id
    );
    Ok(preset)
}

#[tauri::command]
pub fn update_preset(input: UpdatePresetInput) -> Result<ThemePreset, String> {
    log_info!("Updating theme preset with ID: {}", input.id);
    let mut preset_manager = PRESET_MANAGER.lock().unwrap();

    if !preset_manager.preset_exists(&input.id) {
        log_warn!("Update failed: Preset with ID '{}' not found", input.id);
        return Err(format!("Preset with ID '{}' not found", input.id));
    }

    let created_at = preset_manager
        .get_preset(&input.id)
        .unwrap()
        .created_at
        .clone();

    let preset = build_preset(input.id, created_at, input.preset);

    preset_manager.update_preset(preset.clone())?;
    log_info!("Successfully updated preset with ID: {}", preset.id);
    Ok(preset)
}

#[tauri::command]
pub fn delete_preset(id: String) -> Result<(), String> {
    log_info!("Deleting theme preset with ID: {}", id);
    let mut preset_manager = PRESET_MANAGER.lock().unwrap();
    preset_manager.delete_preset(&id)
}

#[tauri::command]
pub fn duplicate_preset(id: String, new_name: String) -> Result<ThemePreset, String> {
    log_info!("Duplicating theme preset with ID: {} as '{}'", id, new_name);
    let mut preset_manager = PRESET_MANAGER.lock().unwrap();

    let existing_preset = preset_manager.get_preset(&id).ok_or_else(|| {
        log_warn!("Duplication failed: Preset with ID '{}' not found", id);
        format!("Preset with ID '{id}' not found")
    })?;

    let new_preset = build_preset(
        Uuid::new_v4().to_string(),
        Utc::now().to_rfc3339(),
        PresetFields::from_preset(existing_preset, new_name),
    );

    preset_manager.add_preset(new_preset.clone())?;
    log_info!(
        "Successfully duplicated preset. New preset ID: {}",
        new_preset.id
    );
    Ok(new_preset)
}
