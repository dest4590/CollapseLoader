use std::{path::PathBuf, sync::Mutex};

use serde::{Deserialize, Serialize};
use std::sync::LazyLock;

use crate::core::storage::data::DATA;

use super::common::JsonStorage;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FavoriteManager {
    pub favorites: Vec<u32>,
    #[serde(skip)]
    pub favorites_path: PathBuf,
}

impl FavoriteManager {
    pub fn load_from_disk(path: PathBuf) -> Self {
        let mut loaded = <Self as JsonStorage>::load_from_disk(path.clone());
        loaded.favorites_path = path;
        loaded
    }

    pub fn add_favorite(&mut self, client_id: u32) {
        if !self.favorites.contains(&client_id) {
            self.favorites.push(client_id);
        }
    }

    pub fn remove_favorite(&mut self, client_id: u32) {
        self.favorites.retain(|&id| id != client_id);
    }

    pub fn is_favorite(&self, client_id: u32) -> bool {
        self.favorites.contains(&client_id)
    }
}

impl JsonStorage for FavoriteManager {
    fn file_path(&self) -> &PathBuf {
        &self.favorites_path
    }

    fn resource_name() -> &'static str {
        "favorites"
    }

    fn create_default() -> Self {
        Self::default()
    }
}

impl Default for FavoriteManager {
    fn default() -> Self {
        Self {
            favorites: Vec::new(),
            favorites_path: DATA.get_local("favorites.json"),
        }
    }
}

pub static FAVORITE_MANAGER: LazyLock<Mutex<FavoriteManager>> = LazyLock::new(|| {
    Mutex::new(FavoriteManager::load_from_disk(
        DATA.get_local("favorites.json"),
    ))
});
