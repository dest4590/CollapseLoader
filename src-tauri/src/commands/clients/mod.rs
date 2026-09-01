pub mod custom;
pub mod general;
pub mod mods;
pub mod ram;
pub mod shortcuts;

pub use custom::*;
pub use general::*;
pub use mods::*;
pub use ram::*;
pub use shortcuts::*;

pub(crate) fn get_client_by_id(
    id: u32,
    manager: &std::sync::Arc<std::sync::Mutex<crate::core::clients::manager::ClientManager>>,
) -> Result<crate::core::clients::client::Client, String> {
    manager
        .lock()
        .map_err(|_| "Failed to acquire lock on client manager".to_string())?
        .clients
        .iter()
        .find(|c| c.id == id)
        .cloned()
        .ok_or_else(|| format!("Client with ID {id} not found"))
}

pub(crate) fn with_client_manager<R>(
    state: &tauri::State<'_, crate::AppState>,
    operation: impl FnOnce(&mut crate::core::clients::manager::ClientManager) -> Result<R, String>,
) -> Result<R, String> {
    let mut manager = state
        .clients
        .manager
        .lock()
        .map_err(|_| "Failed to acquire lock on client manager".to_string())?;
    operation(&mut manager)
}

pub(crate) fn with_custom_client_manager<R>(
    state: &tauri::State<'_, crate::AppState>,
    operation: impl FnOnce(
        &mut crate::core::storage::custom_clients::CustomClientManager,
    ) -> Result<R, String>,
) -> Result<R, String> {
    let mut manager = state.custom_clients.lock();
    operation(&mut manager)
}

pub(crate) fn refresh_tray_menu_after_client_change(
    state: tauri::State<'_, crate::AppState>,
    result: Result<(), String>,
) -> Result<(), String> {
    if result.is_ok() {
        crate::commands::utils::refresh_tray_menu(state);
    }
    result
}

pub(crate) fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            c => c,
        })
        .collect()
}
