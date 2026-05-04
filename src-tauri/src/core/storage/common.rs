use crate::core::utils::{fs as fs_utils, misc};
use crate::{log_error, log_warn};
use serde::{de::DeserializeOwned, Serialize};
use std::{
    fs::{self, File},
    io::BufReader,
    path::PathBuf,
};
/// A trait for types that can be persisted to disk as JSON.
///
/// This trait provides default implementations for saving and loading data,
/// including atomic writes using temporary files and automatic backups on corruption.
pub trait JsonStorage: Sized + Serialize + DeserializeOwned {
    /// Returns the path to the file on disk.
    fn file_path(&self) -> &PathBuf;
    /// Returns a human-readable name for the resource (used in logs).
    fn resource_name() -> &'static str;
    /// Creates a default instance of the resource.
    fn create_default() -> Self;

    /// Serializes the resource to JSON and saves it to disk atomically.
    fn save_to_disk(&self) {
        let file_path = self.file_path();

        let data = match serde_json::to_string_pretty(&self) {
            Ok(d) => d,
            Err(e) => {
                log_error!("Failed to serialize {}: {}", Self::resource_name(), e);
                return;
            }
        };

        if let Some(parent) = file_path.parent() {
            if !parent.exists() {
                if let Err(e) = fs_utils::ensure_dir(parent) {
                    log_warn!("Failed to create dir for {}: {}", Self::resource_name(), e);
                    return;
                }
            }
        }

        if let Err(e) = fs_utils::atomic_write(file_path, data.as_bytes()) {
            log_error!(
                "Failed to write temp file for {}: {}",
                Self::resource_name(),
                e
            );
            return;
        }
    }

    /// Loads the resource from disk, falling back to defaults if the file is missing or corrupted.
    fn load_from_disk(file_path: PathBuf) -> Self {
        if !file_path.exists() {
            log_warn!(
                "{} not found at {}, creating defaults",
                misc::capitalize_first(Self::resource_name()),
                file_path.display()
            );
            let default = Self::create_default();
            default.save_to_disk();
            return default;
        }

        let file = match File::open(&file_path) {
            Ok(f) => f,
            Err(e) => {
                log_error!("Failed to open {}: {}", Self::resource_name(), e);
                let default = Self::create_default();
                default.save_to_disk();
                return default;
            }
        };

        let reader = BufReader::new(file);

        match serde_json::from_reader(reader) {
            Ok(loaded) => loaded,
            Err(e) => {
                log_warn!(
                    "Failed to parse {} ({}). Backing up and resetting.",
                    Self::resource_name(),
                    e
                );

                let backup_path = file_path.with_extension("bak");
                let _ = fs::copy(&file_path, backup_path);

                let default = Self::create_default();
                default.save_to_disk();
                default
            }
        }
    }

    /// Loads the resource and then restores transient fields such as file paths.
    fn load_from_disk_with(file_path: PathBuf, post_load: impl FnOnce(&mut Self)) -> Self {
        let mut loaded = Self::load_from_disk(file_path);
        post_load(&mut loaded);
        loaded
    }
}
