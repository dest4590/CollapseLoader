use serde_json::Value;
use std::fs::{self, File};
use std::io::{BufReader, BufWriter};
use std::path::{Path, PathBuf};

use crate::log_debug;
use crate::log_warn;

pub fn sanitize_path_for_filename(path: &str) -> String {
    path.replace(['/', '\\'], "_") + ".json"
}

pub fn ensure_cache_dir(cache_dir: &Path) {
    if !cache_dir.exists() {
        if let Err(e) = fs::create_dir_all(cache_dir) {
            log_warn!("Failed to create API cache directory at {:?}: {}", cache_dir, e);
        } else {
            log_debug!("Created cache directory at {:?}", cache_dir);
        }
    }
}

pub fn cache_file_path(cache_dir: &Path, path: &str) -> PathBuf {
    let file_name = sanitize_path_for_filename(path);
    cache_dir.join(file_name)
}

pub fn read_cached_json(cache_file_path: &Path) -> Option<Value> {
    if !cache_file_path.exists() {
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

pub fn write_cache_if_changed(cache_file_path: &Path, api_data: &Value, prev_cached: &Option<Value>) {
    let should_update_cache = prev_cached.as_ref().is_none_or(|cached| *cached != *api_data);

    if should_update_cache && cache_file_path.parent().map(|p| p.exists()).unwrap_or(false) {
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
                    log_debug!("Cache updated");
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
