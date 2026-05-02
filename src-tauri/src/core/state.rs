//! Application state management for Tauri.

use crate::core::clients::manager::ClientManager;
use std::sync::{Arc, Mutex};
use std::sync::{MutexGuard, PoisonError};

/// State related to game clients.
///
/// This structure holds the shared manager that tracks all official game clients.
pub struct ClientState {
    /// Thread-safe access to the client manager.
    ///
    /// The manager is wrapped in an `Arc<Mutex<...>>` to allow concurrent access
    /// from multiple Tauri commands.
    pub manager: Arc<Mutex<ClientManager>>,
}

impl ClientState {
    /// Creates a new client state with the given manager.
    pub fn new(manager: Arc<Mutex<ClientManager>>) -> Self {
        Self { manager }
    }
}

/// State related to custom (user-added) clients.
///
/// This is a marker struct that provides access to the global custom client manager.
#[derive(Default)]
pub struct CustomClientsState;

impl CustomClientsState {
    /// Creates a new custom clients state.
    pub fn new() -> Self {
        Self
    }

    /// Acquires a lock on the global custom client manager.
    ///
    /// This method handles potential mutex poisoning by returning the inner value.
    pub fn lock(
        &self,
    ) -> MutexGuard<'static, crate::core::storage::custom_clients::CustomClientManager> {
        crate::core::storage::custom_clients::CUSTOM_CLIENT_MANAGER
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
    }
}

/// The main application state shared across Tauri commands.
///
/// This structure is managed by Tauri and can be injected into commands using `State<'_, AppState>`.
pub struct AppState {
    /// State for official clients.
    pub clients: ClientState,
    /// State for custom clients.
    pub custom_clients: CustomClientsState,
}

impl AppState {
    /// Creates a new application state.
    pub fn new(client_manager: Arc<Mutex<ClientManager>>) -> Self {
        Self {
            clients: ClientState::new(client_manager),
            custom_clients: CustomClientsState::new(),
        }
    }
}
