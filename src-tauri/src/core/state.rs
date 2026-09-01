use crate::core::clients::manager::ClientManager;
use crate::core::storage::accounts::ACCOUNT_MANAGER;
use crate::core::storage::favorites::FAVORITE_MANAGER;
use crate::core::storage::flags::FLAGS_MANAGER;
use crate::core::storage::launch_history::LAUNCH_HISTORY;
use crate::core::storage::mod_builds::MOD_BUILDS;
use crate::core::storage::presets::PRESET_MANAGER;
use crate::core::storage::settings::SETTINGS;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

pub struct ClientState {
    pub manager: Arc<Mutex<ClientManager>>,
}

impl ClientState {
    pub fn new(manager: Arc<Mutex<ClientManager>>) -> Self {
        Self { manager }
    }
}

#[derive(Default)]
pub struct CustomClientsState;

impl CustomClientsState {
    pub fn new() -> Self {
        Self
    }

    pub fn lock(
        &self,
    ) -> MutexGuard<'static, crate::core::storage::custom_clients::CustomClientManager> {
        crate::core::storage::custom_clients::CUSTOM_CLIENT_MANAGER
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
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

    pub fn settings(&self) -> MutexGuard<'static, crate::core::storage::settings::Settings> {
        SETTINGS.lock().unwrap_or_else(PoisonError::into_inner)
    }

    pub fn accounts(&self) -> MutexGuard<'static, crate::core::storage::accounts::AccountManager> {
        ACCOUNT_MANAGER
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
    }

    pub fn favorites(
        &self,
    ) -> MutexGuard<'static, crate::core::storage::favorites::FavoriteManager> {
        FAVORITE_MANAGER
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
    }

    pub fn flags(&self) -> MutexGuard<'static, crate::core::storage::flags::Flags> {
        FLAGS_MANAGER.lock().unwrap_or_else(PoisonError::into_inner)
    }

    pub fn presets(&self) -> MutexGuard<'static, crate::core::storage::presets::PresetManager> {
        PRESET_MANAGER
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
    }

    pub fn mod_builds(
        &self,
    ) -> MutexGuard<'static, crate::core::storage::mod_builds::ModBuildManager> {
        MOD_BUILDS.lock().unwrap_or_else(PoisonError::into_inner)
    }

    pub fn launch_history(
        &self,
    ) -> MutexGuard<'static, crate::core::storage::launch_history::LaunchHistoryManager> {
        LAUNCH_HISTORY
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
    }

    pub fn data(&self) -> MutexGuard<'static, std::path::PathBuf> {
        crate::core::storage::data::DATA
            .root_dir
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
    }
}
