use super::common::JsonStorage;
use crate::core::utils::globals::ROOT_DIR;
use paste::paste;
use serde::{Deserialize, Serialize};
use std::{
    fmt,
    ops::{Deref, DerefMut},
    path::PathBuf,
    sync::{LazyLock, Mutex as StdMutex},
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Setting<T> {
    pub value: T,
    #[serde(default = "default_show_true")]
    pub show: bool,
}

const fn default_show_true() -> bool {
    true
}

impl<T> Setting<T> {
    pub fn new(value: T, show: bool) -> Self {
        Self { value, show }
    }
}

impl<T: Default> Default for Setting<T> {
    fn default() -> Self {
        Self {
            value: T::default(),
            show: true,
        }
    }
}

impl<T> Deref for Setting<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T> DerefMut for Setting<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

impl<T: fmt::Display> fmt::Display for Setting<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

macro_rules! define_settings {
    (
        $(#[$attr:meta])*
        pub struct $name:ident {
            $(
                $(#[doc = $doc:expr])?
                $field:ident: $type:ty = ($default_val:expr, $show_val:expr)
            ),* $(,)?
        }
    ) => {
        paste! {
            $(#[$attr])*
            #[derive(Clone, Debug, Serialize, Deserialize)]
            pub struct $name {
                $(
                    $(#[doc = $doc])?
                    #[serde(default)]
                    pub $field: Setting<$type>,
                )*
                #[serde(skip, default)]
                pub config_path: PathBuf,
            }

            impl Default for $name {
                fn default() -> Self {
                    Self {
                        $(
                            $field: Setting::new($default_val, $show_val),
                        )*
                        config_path: Self::config_path(),
                    }
                }
            }

            impl $name {
                pub fn config_path() -> PathBuf {
                    PathBuf::from(&*ROOT_DIR).join("config.json")
                }

                fn apply_visibility_defaults(&mut self) {
                    $( self.$field.show = $show_val; )*
                }

                pub fn from_input(mut input: Self, config_path: PathBuf) -> Self {
                    input.config_path = config_path;
                    input.apply_visibility_defaults();
                    input
                }

                pub fn load_from_disk(path: PathBuf) -> Self {
                    <Self as JsonStorage>::load_from_disk_with(path.clone(), |loaded| {
                        loaded.config_path = path;
                        loaded.apply_visibility_defaults();
                    })
                }

                pub fn save(&self) {
                    <Self as JsonStorage>::save_to_disk(self);
                }

                $(
                    pub fn [<get_ $field>]() -> $type {
                        SETTINGS.lock().unwrap().$field.value.clone()
                    }

                    pub fn [<set_ $field>](val: $type) {
                        let mut lock = SETTINGS.lock().unwrap();
                        lock.$field.value = val;
                        lock.save();
                    }
                )*
            }

            impl JsonStorage for $name {
                fn file_path(&self) -> &PathBuf {
                    &self.config_path
                }
                fn resource_name() -> &'static str {
                    "config"
                }
                fn create_default() -> Self {
                    Self::default()
                }
            }
        }
    };
}

define_settings! {
    pub struct Settings {
        ram: u32 = (2048, true),
        theme: String = ("dark".to_string(), false),
        language: String = ("en".to_string(), true),
        discord_rpc_enabled: bool = (true, true),
        optional_telemetry: bool = (true, true),
        irc_chat: bool = (true, true),
        hash_verify: bool = (true, true),
        sync_client_settings: bool = (true, true),
        dpi_bypass: bool = (false, true),
        minimize_to_tray_on_launch: bool = (false, true),
        close_to_tray: bool = (false, true),
        java_path: String = ("".to_string(), true),
        java_args: String = ("".to_string(), true),
        auto_update: bool = (true, true),
        autostart: bool = (false, true),
        start_minimized: bool = (false, true),
    }
}

pub static SETTINGS: LazyLock<StdMutex<Settings>> =
    LazyLock::new(|| StdMutex::new(Settings::load_from_disk(Settings::config_path())));

#[rustfmt::skip]
pub fn settings_schema() -> Vec<(String, String)> {
    vec![
        ("ram".to_string(), "settings.ram".to_string()),
        ("language".to_string(), "settings.language".to_string()),
        ("discord_rpc_enabled".to_string(), "settings.discord_rpc_enabled".to_string()),
        ("optional_telemetry".to_string(), "settings.optional_telemetry".to_string()),
        ("irc_chat".to_string(), "settings.irc_chat".to_string()),
        ("hash_verify".to_string(), "settings.hash_verify".to_string()),
        ("sync_client_settings".to_string(), "settings.sync_client_settings".to_string()),
        ("dpi_bypass".to_string(), "settings.dpi_bypass".to_string()),
        ("minimize_to_tray_on_launch".to_string(), "settings.minimize_to_tray_on_launch".to_string()),
        ("close_to_tray".to_string(), "settings.close_to_tray".to_string()),
        ("auto_update".to_string(), "settings.auto_update".to_string()),
        ("autostart".to_string(), "settings.autostart".to_string()),
        ("start_minimized".to_string(), "settings.start_minimized".to_string()),
        ("java_path".to_string(), "settings.java_path".to_string()),
        ("java_args".to_string(), "settings.java_args".to_string()),
    ]
}
