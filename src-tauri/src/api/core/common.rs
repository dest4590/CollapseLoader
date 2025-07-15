use crate::{log_debug, log_error, log_warn};
use serde::{de::DeserializeOwned, Serialize};
use std::{fs, path::PathBuf};

pub trait JsonStorage: Sized + Serialize + DeserializeOwned {
    fn file_path(&self) -> &PathBuf;

    fn resource_name() -> &'static str;

    fn create_default() -> Self;

    fn save_to_disk(&self) {
        log_debug!(
            "Saving {} to {}",
            Self::resource_name(),
            self.file_path().display()
        );
        match serde_json::to_string_pretty(&self) {
            Ok(data) => {
                let file_path = self.file_path();

                if let Some(parent_dir) = file_path.parent() {
                    if !parent_dir.exists() {
                        if let Err(e) = fs::create_dir_all(parent_dir) {
                            log_warn!(
                                "Failed to create directory for {} file {}: {}",
                                Self::resource_name(),
                                parent_dir.display(),
                                e
                            );
                            return;
                        }
                    }
                }

                if let Err(e) = fs::write(file_path, data) {
                    log_warn!(
                        "Failed to write {} file to {}: {}",
                        Self::resource_name(),
                        file_path.display(),
                        e
                    );
                    return;
                }
                log_debug!(
                    "Successfully saved {} to {}",
                    Self::resource_name(),
                    file_path.display()
                );
            }
            Err(e) => {
                log_error!(
                    "Failed to serialize {} to JSON: {}",
                    Self::resource_name(),
                    e
                );
            }
        }
    }

    fn load_from_disk(file_path: PathBuf) -> Self {
        if file_path.exists() {
            match fs::read_to_string(&file_path) {
                Ok(data) => match serde_json::from_str::<Self>(&data) {
                    Ok(loaded) => {
                        return loaded;
                    }
                    Err(e) => {
                        log_warn!(
                            "Failed to parse {} file at {}, attempting to merge with defaults: {}",
                            Self::resource_name(),
                            file_path.display(),
                            e
                        );

                        if let Ok(partial_value) = serde_json::from_str::<serde_json::Value>(&data)
                        {
                            let default = Self::create_default();
                            if let Ok(default_value) = serde_json::to_value(&default) {
                                if let Ok(merged) =
                                    Self::merge_json_values(default_value, partial_value)
                                {
                                    if let Ok(merged_instance) =
                                        serde_json::from_value::<Self>(merged)
                                    {
                                        log_debug!(
                                            "Successfully merged {} with defaults",
                                            Self::resource_name()
                                        );
                                        merged_instance.save_to_disk();
                                        return merged_instance;
                                    }
                                }
                            }
                        }

                        log_warn!(
                            "Failed to recover {} at {}, using defaults",
                            Self::resource_name(),
                            file_path.display()
                        );
                    }
                },
                Err(e) => {
                    log_warn!(
                        "Failed to read {} file at {}, using default: {}",
                        Self::resource_name(),
                        file_path.display(),
                        e
                    );
                }
            }
        } else {
            log_debug!(
                "No {} file found at {}, creating a new one with defaults.",
                Self::resource_name(),
                file_path.display()
            );
        }

        let default = Self::create_default();
        let _ = default.save_to_disk();
        default
    }

    fn merge_json_values(
        default_value: serde_json::Value,
        partial_value: serde_json::Value,
    ) -> serde_json::Result<serde_json::Value> {
        use serde_json::Value;

        match (default_value, partial_value) {
            (Value::Object(mut default_map), Value::Object(partial_map)) => {
                for (key, value) in partial_map {
                    match default_map.get_mut(&key) {
                        Some(default_field) => {
                            if default_field.is_object() && value.is_object() {
                                let merged = Self::merge_json_values(default_field.clone(), value)?;
                                *default_field = merged;
                            } else {
                                *default_field = value;
                            }
                        }
                        None => {
                            default_map.insert(key, value);
                        }
                    }
                }

                Ok(Value::Object(default_map))
            }
            (_, partial) => Ok(partial),
        }
    }
}
