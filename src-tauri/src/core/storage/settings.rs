use super::common::JsonStorage;
use super::data::DATA;
use paste::paste;
use serde::{Deserialize, Serialize};
use std::{fmt, path::PathBuf, sync::Mutex as StdMutex};

use lazy_static::lazy_static;

fn default_show_true() -> bool {
    true
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Setting<T> {
    pub value: T,
    #[serde(default = "default_show_true")]
    pub show: bool,
}

impl<T> Setting<T> {
    pub fn new(value: T, show: bool) -> Self {
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
                        config_path: DATA.get_local("config.json"),
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
                    <Self as JsonStorage>::load_from_disk(path)
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
    }
}

lazy_static! {
    pub static ref SETTINGS: StdMutex<Settings> =
        StdMutex::new(Settings::load_from_disk(DATA.get_local("config.json")));
}
