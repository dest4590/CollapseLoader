use serde_json::Value;
use std::fs::{self, File};
use std::io::{BufReader, BufWriter, ErrorKind};
use std::path::{Path, PathBuf};

use crate::log_debug;
use crate::log_warn;

/// Sanitizes an API path to be used as a filename for caching.
pub fn sanitize_path_for_filename(path: &str) -> String {
    let sanitized = path.replace(['/', '\\'], "_");
    if sanitized.to_lowercase().ends_with(".json") {
        sanitized
    } else {
        format!("{}.json", sanitized)
    }
}

/// Ensures that the API cache directory exists on disk.
pub fn ensure_cache_dir(cache_dir: &Path) {
    match fs::create_dir(cache_dir) {
        Ok(()) => {
            log_debug!("Created cache directory at {:?}", cache_dir);
        }
        Err(e) => match e.kind() {
            ErrorKind::AlreadyExists => {}
            ErrorKind::NotFound => {
                if let Err(e2) = fs::create_dir_all(cache_dir) {
                    log_warn!(
                        "Failed to create API cache directory at {:?}: {}",
                        cache_dir,
                        e2
                    );
                } else {
                    log_debug!("Created cache directory at {:?}", cache_dir);
                }
            }
            _ => {
                log_warn!(
                    "Failed to create API cache directory at {:?}: {}",
                    cache_dir,
                    e
                );
            }
        },
    }
}

/// Returns the full path to a cached API response file.
pub fn cache_file_path(cache_dir: &Path, path: &str) -> PathBuf {
    let file_name = sanitize_path_for_filename(path);
    cache_dir.join(file_name)
}

/// Reads and deserializes a cached JSON response from disk.
pub fn read_cached_json(cache_file_path: &Path) -> Option<Value> {
    if !cache_file_path.exists() {
        log_debug!("API cache miss: {:?}", cache_file_path);
        return None;
    }

    match File::open(cache_file_path) {
        Ok(file) => {
            let reader = BufReader::new(file);
            match serde_json::from_reader(reader) {
                Ok(data) => Some(data),
                Err(e) => {
                    log_warn!(
                        "Failed to deserialize cached API response from {:?}: {}",
                        cache_file_path,
                        e
                    );
                    None
                }
            }
        }
        Err(e) => {
            log_warn!(
                "Failed to open cached API response file {:?}: {}",
                cache_file_path,
                e
            );
            None
        }
    }
}

/// Writes an API response to the cache if it has changed since the last fetch.
pub fn write_cache_if_changed(
    cache_file_path: &Path,
    api_data: &Value,
    prev_cached: &Option<Value>,
) {
    let should_update_cache = prev_cached
        .as_ref()
        .is_none_or(|cached| *cached != *api_data);

    if should_update_cache
        && cache_file_path
            .parent()
            .map(|p| p.exists())
            .unwrap_or(false)
    {
        match File::create(cache_file_path) {
            Ok(file) => {
                let writer = BufWriter::new(file);
                if let Err(e) = serde_json::to_writer_pretty(writer, api_data) {
                    log_warn!(
                        "Failed to write API response to cache at {:?}: {}",
                        cache_file_path,
                        e
                    );
                } else {
                    log_debug!("Wrote API cache: {:?}", cache_file_path);
                }
            }
            Err(e) => {
                log_warn!(
                    "Failed to create API cache file at {:?}: {}",
                    cache_file_path,
                    e
                );
            }
        }
    }
}
