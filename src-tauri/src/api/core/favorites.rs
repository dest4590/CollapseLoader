use std::{path::PathBuf, sync::Mutex};

use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};

use crate::api::core::data::DATA;

use super::common::JsonStorage;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FavoriteManager {
    pub favorites: Vec<u32>,
    pub favorites_path: PathBuf,
}

impl FavoriteManager {
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

lazy_static! {
    pub static ref FAVORITE_MANAGER: Mutex<FavoriteManager> = Mutex::new(
        FavoriteManager::load_from_disk(DATA.get_local("favorites.json"))
    );
}
