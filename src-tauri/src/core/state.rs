use crate::core::clients::manager::ClientManager;
use std::sync::{Arc, Mutex};

pub struct ClientState {
    pub manager: Arc<Mutex<ClientManager>>,
}

impl ClientState {
    pub fn new(manager: Arc<Mutex<ClientManager>>) -> Self {
        Self { manager }
    }
}

pub struct CustomClientsState;

impl CustomClientsState {
    pub fn new() -> Self {
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
    pub fn new(client_manager: Arc<Mutex<ClientManager>>) -> Self {
        Self {
            clients: ClientState::new(client_manager),
            custom_clients: CustomClientsState::new(),
        }
    }
}
