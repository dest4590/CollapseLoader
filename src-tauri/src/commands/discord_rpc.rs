use crate::api::discord_rpc;

#[tauri::command]
pub fn update_presence(details: String, state: String) -> Result<(), String> {
    discord_rpc::update_activity_async(details, state);
    Ok(())
}
