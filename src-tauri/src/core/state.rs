use crate::core::clients::manager::ClientManager;
use std::sync::{Arc, Mutex};

pub struct ClientState {
    pub manager: Arc<Mutex<ClientManager>>,
}

impl ClientState {
    pub const fn new(manager: Arc<Mutex<ClientManager>>) -> Self {
        Self { manager }
    }
}

pub struct CustomClientsState;

impl Default for CustomClientsState {
    fn default() -> Self {
        Self::new()
    }
}

impl CustomClientsState {
    pub const fn new() -> Self {
        Self
    }

    pub fn lock(
        &self,
    ) -> std::sync::MutexGuard<'static, crate::core::storage::custom_clients::CustomClientManager>
    {
        crate::core::storage::custom_clients::CUSTOM_CLIENT_MANAGER
            .lock()
            .unwrap()
    }
}

pub struct AppState {
    pub clients: ClientState,
    pub custom_clients: CustomClientsState,
}

impl AppState {
    pub const fn new(client_manager: Arc<Mutex<ClientManager>>) -> Self {
        Self {
            clients: ClientState::new(client_manager),
            custom_clients: CustomClientsState::new(),
        }
    }
}
