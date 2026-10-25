use super::common::JsonStorage;
use super::data::DATA;
use crate::core::auth::skin;
use base64::{engine::general_purpose::STANDARD, Engine as _};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::PathBuf;
use std::sync::{LazyLock, Mutex as StdMutex};
use uuid::Uuid;

const MAX_SAVED_SKINS: usize = 60;
const TEXTURE_PREFIX: &str = "data:image/png;base64,";

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SavedSkin {
    pub id: String,
    pub name: String,
    pub variant: String,
    pub texture_url: String,
    pub texture_hash: String,
    pub created_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SkinLibrary {
    pub skins: Vec<SavedSkin>,
    #[serde(skip)]
    pub config_path: PathBuf,
}

impl Default for SkinLibrary {
    fn default() -> Self {
        Self {
            skins: Vec::new(),
            config_path: DATA.get_local("skins.json"),
        }
    }
}

impl JsonStorage for SkinLibrary {
    fn file_path(&self) -> &PathBuf {
        &self.config_path
    }

    fn resource_name() -> &'static str {
        "skin library"
    }

    fn create_default() -> Self {
        Self::default()
    }
}

impl SkinLibrary {
    pub fn load_from_disk(path: PathBuf) -> Self {
        <Self as JsonStorage>::load_from_disk_with(path.clone(), |loaded| {
            loaded.config_path = path;
        })
    }

    pub fn list(&self) -> Vec<SavedSkin> {
        self.skins.clone()
    }

    pub fn upsert(&mut self, name: &str, variant: &str, bytes: &[u8]) -> Result<SavedSkin, String> {
        skin::validate_skin_bytes(bytes)?;
        let variant = skin::normalize_variant(variant)?;
        let texture_hash = hash_texture(bytes);
        let texture_url = format!("{TEXTURE_PREFIX}{}", STANDARD.encode(bytes));
        let name = normalize_name(name);

        if let Some(existing) = self
            .skins
            .iter_mut()
            .find(|saved| saved.texture_hash == texture_hash)
        {
            existing.name = name;
            existing.variant = variant;
            existing.texture_url = texture_url;
            let saved = existing.clone();
            self.save_to_disk();
            return Ok(saved);
        }

        let saved = SavedSkin {
            id: Uuid::new_v4().to_string(),
            name,
            variant,
            texture_url,
            texture_hash,
            created_at: chrono::Utc::now().to_rfc3339(),
        };
        self.skins.insert(0, saved.clone());
        self.skins.truncate(MAX_SAVED_SKINS);
        self.save_to_disk();
        Ok(saved)
    }

    pub fn delete(&mut self, id: &str) -> Result<(), String> {
        let before = self.skins.len();
        self.skins.retain(|saved| saved.id != id);
        if self.skins.len() == before {
            return Err("Saved skin not found.".to_string());
        }
        self.save_to_disk();
        Ok(())
    }

    pub fn texture(&self, id: &str) -> Result<(Vec<u8>, String), String> {
        let saved = self
            .skins
            .iter()
            .find(|saved| saved.id == id)
            .ok_or_else(|| "Saved skin not found.".to_string())?;
        let encoded = saved
            .texture_url
            .strip_prefix(TEXTURE_PREFIX)
            .ok_or_else(|| "Saved skin texture is corrupted.".to_string())?;
        let bytes = STANDARD
            .decode(encoded)
            .map_err(|error| format!("Failed to decode the saved skin texture: {error}"))?;
        skin::validate_skin_bytes(&bytes)?;
        Ok((bytes, saved.variant.clone()))
    }
}

fn normalize_name(name: &str) -> String {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return "My skin".to_string();
    }
    trimmed.chars().take(48).collect()
}

fn hash_texture(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

pub static SKIN_LIBRARY: LazyLock<StdMutex<SkinLibrary>> =
    LazyLock::new(|| StdMutex::new(SkinLibrary::load_from_disk(DATA.get_local("skins.json"))));

pub fn list_skins() -> Result<Vec<SavedSkin>, String> {
    let library = SKIN_LIBRARY
        .lock()
        .map_err(|_| "Skin library is unavailable.".to_string())?;
    Ok(library.list())
}

pub fn add_skin(name: &str, variant: &str, bytes: &[u8]) -> Result<SavedSkin, String> {
    let mut library = SKIN_LIBRARY
        .lock()
        .map_err(|_| "Skin library is unavailable.".to_string())?;
    library.upsert(name, variant, bytes)
}

pub fn delete_skin(id: &str) -> Result<(), String> {
    let mut library = SKIN_LIBRARY
        .lock()
        .map_err(|_| "Skin library is unavailable.".to_string())?;
    library.delete(id)
}

pub fn skin_texture(id: &str) -> Result<(Vec<u8>, String), String> {
    let library = SKIN_LIBRARY
        .lock()
        .map_err(|_| "Skin library is unavailable.".to_string())?;
    library.texture(id)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn skin_bytes() -> Vec<u8> {
        let mut bytes = vec![0u8; 24 + 16];
        bytes[..8].copy_from_slice(&[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A]);
        bytes[16..20].copy_from_slice(&64u32.to_be_bytes());
        bytes[20..24].copy_from_slice(&64u32.to_be_bytes());
        bytes
    }

    #[test]
    fn upsert_deduplicates_and_updates_variant() {
        let path = std::env::temp_dir().join(format!(
            "collapseloader-skin-library-test-{}.json",
            Uuid::new_v4()
        ));
        let mut library = SkinLibrary {
            skins: Vec::new(),
            config_path: path.clone(),
        };
        let first = library.upsert("My skin", "classic", &skin_bytes()).unwrap();
        let second = library.upsert("My skin", "slim", &skin_bytes()).unwrap();
        assert_eq!(library.skins.len(), 1);
        assert_eq!(first.id, second.id);
        assert_eq!(second.variant, "slim");
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn delete_removes_saved_skin() {
        let path = std::env::temp_dir().join(format!(
            "collapseloader-skin-library-test-{}.json",
            Uuid::new_v4()
        ));
        let mut library = SkinLibrary {
            skins: Vec::new(),
            config_path: path.clone(),
        };
        let saved = library.upsert("My skin", "classic", &skin_bytes()).unwrap();
        library.delete(&saved.id).unwrap();
        assert!(library.skins.is_empty());
        assert!(library.delete(&saved.id).is_err());
        let _ = std::fs::remove_file(path);
    }
}
