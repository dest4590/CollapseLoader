use crate::core::storage::common::JsonStorage;
use crate::core::storage::data::DATA;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::{LazyLock, Mutex};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ModEntry {
    pub slug: String,
    pub title: String,
    pub description: String,
    pub icon_url: String,
    pub modrinth_url: String,
    pub version_id: String,
    pub version_number: String,
    pub filename: String,
    pub download_url: String,
    pub file_size: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ModBuild {
    pub id: String,
    pub name: String,
    pub description: String,
    pub mc_version: String,
    pub loader: String,
    pub mods: Vec<ModEntry>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ModBuildManager {
    pub builds: Vec<ModBuild>,
    #[serde(skip)]
    pub file_path: PathBuf,
}

impl ModBuildManager {
    pub fn load(path: PathBuf) -> Self {
        <Self as JsonStorage>::load_from_disk_with(path.clone(), |loaded| {
            loaded.file_path = path;
        })
    }

    pub fn create(&mut self, build: ModBuild) {
        self.builds.push(build);
        self.save_to_disk();
    }

    pub fn update(&mut self, build: ModBuild) {
        if let Some(existing) = self.builds.iter_mut().find(|b| b.id == build.id) {
            *existing = build;
            self.save_to_disk();
        }
    }

    pub fn delete(&mut self, id: &str) {
        self.builds.retain(|b| b.id != id);
        self.save_to_disk();
    }

    pub fn get(&self, id: &str) -> Option<&ModBuild> {
        self.builds.iter().find(|b| b.id == id)
    }

    pub fn get_all(&self) -> &[ModBuild] {
        &self.builds
    }
}

impl JsonStorage for ModBuildManager {
    fn file_path(&self) -> &PathBuf {
        &self.file_path
    }

    fn resource_name() -> &'static str {
        "mod_builds"
    }

    fn create_default() -> Self {
        Self::default()
    }
}

impl Default for ModBuildManager {
    fn default() -> Self {
        Self {
            builds: Vec::new(),
            file_path: DATA.get_local("mod_builds.json"),
        }
    }
}

pub static MOD_BUILDS: LazyLock<Mutex<ModBuildManager>> = LazyLock::new(|| {
    Mutex::new(ModBuildManager::load(
        DATA.get_local("mod_builds.json"),
    ))
});
