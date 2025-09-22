use crate::{core::utils::discord_rpc, log_debug};

#[tauri::command]
pub fn update_presence(details: String, state: String) -> Result<(), String> {
    log_debug!(
        "Updating Discord presence: details='{}', state='{}'",
        details,
        state
    );
    discord_rpc::update_activity_async(details, state);
    Ok(())
}
