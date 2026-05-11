use crate::core::storage::common::JsonStorage;
use crate::core::storage::data::DATA;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::{LazyLock, Mutex};

const MAX_HISTORY_ENTRIES: usize = 50;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LaunchEntry {
    pub client_id: u32,
    pub client_name: String,
    pub client_version: String,
    pub launched_at: String,
    pub account_name: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LaunchHistoryManager {
    pub entries: Vec<LaunchEntry>,
    #[serde(skip)]
    pub history_path: PathBuf,
}

impl LaunchHistoryManager {
    pub fn load(path: PathBuf) -> Self {
        <Self as JsonStorage>::load_from_disk_with(path.clone(), |loaded| {
            loaded.history_path = path;
        })
    }

    pub fn record(&mut self, entry: LaunchEntry) {
        self.entries.insert(0, entry);
        if self.entries.len() > MAX_HISTORY_ENTRIES {
            self.entries.truncate(MAX_HISTORY_ENTRIES);
        }
        self.save_to_disk();
    }

    pub fn clear(&mut self) {
        self.entries.clear();
        self.save_to_disk();
    }
}

impl JsonStorage for LaunchHistoryManager {
    fn file_path(&self) -> &PathBuf {
        &self.history_path
    }

    fn resource_name() -> &'static str {
        "launch_history"
    }

    fn create_default() -> Self {
        Self::default()
    }
}

impl Default for LaunchHistoryManager {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
            history_path: DATA.get_local("launch_history.json"),
        }
    }
}

pub static LAUNCH_HISTORY: LazyLock<Mutex<LaunchHistoryManager>> = LazyLock::new(|| {
    Mutex::new(LaunchHistoryManager::load(
        DATA.get_local("launch_history.json"),
    ))
});
