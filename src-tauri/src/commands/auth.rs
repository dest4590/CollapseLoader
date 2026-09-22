use crate::core::auth::microsoft;
use crate::core::auth::microsoft::MinecraftAccountData;
use crate::core::storage::accounts::PublicAccount;
use crate::AppState;
use tauri::{AppHandle, State};

fn save_microsoft_account(
    state: State<'_, AppState>,
    data: MinecraftAccountData,
) -> Result<PublicAccount, String> {
    let mut manager = state.accounts();
    let id = manager.add_microsoft_account(data)?;
    if !manager.set_active_account(&id) {
        return Err("Microsoft account was saved but could not be activated.".to_string());
    }
    manager
        .accounts
        .iter()
        .find(|account| account.id == id)
        .map(PublicAccount::from)
        .ok_or_else(|| "Microsoft account was saved but could not be loaded.".to_string())
}

#[tauri::command]
pub async fn login_microsoft_account(
    app_handle: AppHandle,
    state: State<'_, AppState>,
) -> Result<PublicAccount, String> {
    let data = microsoft::login(app_handle).await?;
    save_microsoft_account(state, data)
}

#[tauri::command]
pub async fn login_microsoft_account_with_access_token(
    access_token: String,
    state: State<'_, AppState>,
) -> Result<PublicAccount, String> {
    let data = microsoft::login_with_access_token(&access_token).await?;
    save_microsoft_account(state, data)
}

#[tauri::command]
pub async fn login_microsoft_account_with_refresh_token(
    refresh_token: String,
    state: State<'_, AppState>,
) -> Result<PublicAccount, String> {
    let data = microsoft::login_with_refresh_token(&refresh_token).await?;
    save_microsoft_account(state, data)
}

#[tauri::command]
pub async fn login_microsoft_account_with_cookies(
    cookies: String,
    state: State<'_, AppState>,
) -> Result<PublicAccount, String> {
    let data = microsoft::login_with_cookies(&cookies).await?;
    save_microsoft_account(state, data)
}
