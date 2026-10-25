use crate::core::auth::microsoft::ensure_valid;
use crate::core::auth::skin::{self, SkinProfile};
use crate::core::storage::accounts::{Account, AccountType};
use crate::core::storage::skin_library::{self, SavedSkin};
use crate::{log_info, log_warn, AppState};
use std::path::Path;
use tauri::State;

fn find_account(state: &AppState, account_id: &str) -> Result<Account, String> {
    let manager = state.accounts();
    let account = manager
        .accounts
        .iter()
        .find(|account| account.id == account_id)
        .cloned()
        .ok_or_else(|| "Account not found.".to_string())?;
    if account.account_type != AccountType::Microsoft {
        return Err("Skin management is only available for Microsoft accounts.".to_string());
    }
    Ok(account)
}

async fn access_token(state: &AppState, account_id: &str) -> Result<String, String> {
    let account = find_account(state, account_id)?;
    let data = account
        .minecraft_data()?
        .ok_or_else(|| "Skin management is only available for Microsoft accounts.".to_string())?;
    let refreshed = ensure_valid(&data).await?;
    let credentials_changed = refreshed.access_token != data.access_token
        || refreshed.refresh_token != data.refresh_token
        || refreshed.expires_at != data.expires_at;
    if credentials_changed {
        let mut manager = state.accounts();
        manager.update_microsoft_account(account_id, refreshed.clone())?;
    }
    Ok(refreshed.access_token)
}

#[tauri::command]
pub async fn get_minecraft_skin_profile(
    state: State<'_, AppState>,
    account_id: String,
) -> Result<SkinProfile, String> {
    log_info!("Loading Minecraft skin profile for account {}", account_id);
    let token = access_token(&state, &account_id).await?;
    skin::fetch_profile(&token).await
}

#[tauri::command]
pub async fn upload_minecraft_skin(
    state: State<'_, AppState>,
    account_id: String,
    file_path: String,
    variant: String,
) -> Result<SkinProfile, String> {
    log_info!("Uploading Minecraft skin for account {}", account_id);
    let variant = skin::normalize_variant(&variant)?;
    let token = access_token(&state, &account_id).await?;
    let mut profile = skin::upload_skin(&token, &file_path, &variant).await?;
    profile.skin_variant = variant.clone();

    if let Ok(bytes) = std::fs::read(&file_path) {
        let name = Path::new(&file_path)
            .file_stem()
            .map(|value| value.to_string_lossy().to_string())
            .unwrap_or_else(|| "My skin".to_string());
        if let Err(error) = skin_library::add_skin(&name, &variant, &bytes) {
            log_warn!("Failed to save uploaded skin: {}", error);
        }
    }

    Ok(profile)
}

#[tauri::command]
pub async fn reset_minecraft_skin(
    state: State<'_, AppState>,
    account_id: String,
) -> Result<SkinProfile, String> {
    log_info!("Resetting Minecraft skin for account {}", account_id);
    let token = access_token(&state, &account_id).await?;
    skin::reset_skin(&token).await
}

#[tauri::command]
pub async fn set_minecraft_cape(
    state: State<'_, AppState>,
    account_id: String,
    cape_id: Option<String>,
) -> Result<SkinProfile, String> {
    log_info!("Changing Minecraft cape for account {}", account_id);
    let token = access_token(&state, &account_id).await?;
    skin::set_cape(&token, cape_id.as_deref()).await
}

#[tauri::command]
pub fn list_saved_minecraft_skins() -> Result<Vec<SavedSkin>, String> {
    skin_library::list_skins()
}

#[tauri::command]
pub fn delete_saved_minecraft_skin(id: String) -> Result<(), String> {
    log_info!("Deleting saved Minecraft skin {}", id);
    skin_library::delete_skin(&id)
}

#[tauri::command]
pub async fn apply_saved_minecraft_skin(
    state: State<'_, AppState>,
    account_id: String,
    id: String,
) -> Result<SkinProfile, String> {
    log_info!(
        "Applying saved Minecraft skin {} for account {}",
        id,
        account_id
    );
    let (bytes, variant) = skin_library::skin_texture(&id)?;
    let token = access_token(&state, &account_id).await?;
    let mut profile = skin::upload_skin_bytes(&token, bytes, &variant).await?;
    profile.skin_variant = variant;
    Ok(profile)
}

#[tauri::command]
pub async fn apply_default_minecraft_skin(
    state: State<'_, AppState>,
    account_id: String,
    texture_key: Option<String>,
    texture_data: Option<String>,
    variant: String,
) -> Result<SkinProfile, String> {
    log_info!("Applying default Minecraft skin for account {}", account_id);
    let variant = skin::normalize_variant(&variant)?;
    let bytes = match (texture_key, texture_data) {
        (Some(texture_key), None) => skin::download_texture(&texture_key).await?,
        (None, Some(texture_data)) => skin::decode_texture_data(&texture_data)?,
        _ => return Err("Provide either skin texture data or a skin texture key.".to_string()),
    };
    let token = access_token(&state, &account_id).await?;
    let mut profile = skin::upload_skin_bytes(&token, bytes, &variant).await?;
    profile.skin_variant = variant;
    Ok(profile)
}
