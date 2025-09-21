use super::common::JsonStorage;
use paste::paste;
use serde::{Deserialize, Serialize};
use std::{fmt, path::PathBuf, sync::Mutex as StdMutex};

use crate::core::utils::globals::ROOT_DIR;
use std::sync::LazyLock;

const fn default_show_true() -> bool {
    true
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Setting<T> {
    pub value: T,
    #[serde(default = "default_show_true")]
    pub show: bool,
}

impl<T> Setting<T> {
    pub const fn new(value: T, show: bool) -> Self {
        Self { value, show }
    }
}

impl<T> std::ops::Deref for Setting<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T> std::ops::DerefMut for Setting<T> {
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
            $( $field:ident: Setting<$field_type:ty> = ($default_val:expr, $show_val:expr) ),* $(,)?
        }
    ) => {
        paste! {
            $(#[$attr])*
            #[derive(Clone, Debug, Serialize, Deserialize)]
            pub struct $name {
                $(pub $field: Setting<$field_type>,)*
                pub config_path: PathBuf,
            }

            #[derive(Deserialize, Debug)]
            pub struct [<Input $name>] {
                $(pub $field: Setting<$field_type>,)*
            }

            impl Default for $name {
                fn default() -> Self {
                    Self {
                        $(
                            $field: Setting::new($default_val, $show_val),
                        )*
                        config_path: PathBuf::from(&*ROOT_DIR).join("config.json"),
                    }
                }
            }

            impl $name {
                pub fn from_input(input: [<Input $name>], config_path: PathBuf) -> Self {
                    Self {
                        $(
                            $field: Setting {
                                value: input.$field.value,
                                show: input.$field.show,
                            },
                        )*
                        config_path,
                    }
                }

                pub fn load_from_disk(path: PathBuf) -> Self {
                    // update config_path here, because value is computed at runtime
                    let mut loaded = <Self as JsonStorage>::load_from_disk(path.clone());
                    loaded.config_path = path;
                    loaded
                }
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
        ram: Setting<u32> = (2048, true),
        theme: Setting<String> = ("dark".to_string(), false),
        language: Setting<String> = ("en".to_string(), true),
        discord_rpc_enabled: Setting<bool> = (true, true),
        optional_telemetry: Setting<bool> = (true, true),
        cordshare: Setting<bool> = (true, true),
        irc_chat: Setting<bool> = (true, true),
        hash_verify: Setting<bool> = (true, true),
        sync_client_settings: Setting<bool> = (true, true),
    }
}

pub static SETTINGS: LazyLock<StdMutex<Settings>> = LazyLock::new(|| {
    StdMutex::new(Settings::load_from_disk(
        PathBuf::from(&*ROOT_DIR).join("config.json"),
    ))
});
