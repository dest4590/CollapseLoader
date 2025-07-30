use crate::core::network::analytics::Analytics;

#[tauri::command]
pub fn send_client_analytics(client_id: u32) -> Result<(), String> {
    Analytics::send_client_analytics(client_id);
    Ok(())
}
