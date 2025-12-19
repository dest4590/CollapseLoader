use super::common::JsonStorage;
use super::data::DATA;
use serde::{Deserialize, Serialize};
use std::sync::LazyLock;
use std::{path::PathBuf, sync::Mutex};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Flag<T> {
    pub value: T,
}

impl<T> Flag<T> {
    pub const fn new(value: T) -> Self {
        Self { value }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Flags {
    pub disclaimer_shown: Flag<bool>,
    pub first_run: Flag<bool>,
    pub telemetry_consent_shown: Flag<bool>,
    pub custom_clients_display: Flag<String>,
    #[serde(skip)]
    pub flags_path: PathBuf,
}

impl Flags {
    pub fn load_from_disk(path: PathBuf) -> Self {
        let mut loaded = <Self as JsonStorage>::load_from_disk(path.clone());
        loaded.flags_path = path;
        loaded
    }

    pub fn set_custom_clients_display(&mut self, display: String) {
        self.custom_clients_display.value = display;
    }
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

impl Default for Flags {
    fn default() -> Self {
        Self {
            disclaimer_shown: Flag::new(false),
            first_run: Flag::new(true),
            telemetry_consent_shown: Flag::new(false),
            custom_clients_display: Flag::new("separate".to_string()),
            flags_path: DATA.get_local("flags.json"),
        }
    }
}

pub static FLAGS_MANAGER: LazyLock<Mutex<Flags>> =
    LazyLock::new(|| Mutex::new(Flags::load_from_disk(DATA.get_local("flags.json"))));
