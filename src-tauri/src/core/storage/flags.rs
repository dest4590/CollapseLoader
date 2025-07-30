use super::common::JsonStorage;
use super::data::DATA;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::{path::PathBuf, sync::Mutex};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Flag<T> {
    pub value: T,
}

impl<T> Flag<T> {
    pub fn new(value: T) -> Self {
        Self { value }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Flags {
    pub disclaimer_shown: Flag<bool>,
    pub first_run: Flag<bool>,
    pub telemetry_consent_shown: Flag<bool>,
    pub flags_path: PathBuf,
}

impl JsonStorage for Flags {
    fn file_path(&self) -> &PathBuf {
        &self.flags_path
    }

    fn resource_name() -> &'static str {
        "flags"
    }

    fn create_default() -> Self {
        Self::default()
    }
}

impl Clone for Flags {
    fn clone(&self) -> Self {
        Self {
            disclaimer_shown: self.disclaimer_shown.clone(),
            first_run: self.first_run.clone(),
            telemetry_consent_shown: self.telemetry_consent_shown.clone(),
            flags_path: self.flags_path.clone(),
        }
    }
}

impl Default for Flags {
    fn default() -> Self {
        Self {
            disclaimer_shown: Flag::new(false),
            first_run: Flag::new(true),
            telemetry_consent_shown: Flag::new(false),
            flags_path: DATA.get_local("flags.json"),
        }
    }
}

lazy_static! {
    pub static ref FLAGS_MANAGER: Mutex<Flags> =
        Mutex::new(Flags::load_from_disk(DATA.get_local("flags.json")));
}
