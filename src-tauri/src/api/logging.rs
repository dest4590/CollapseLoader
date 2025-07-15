use colored::*;
use lazy_static::lazy_static;
use std::sync::Mutex;

pub enum LogLevel {
    Info,
    Warn,
    Error,
    Debug,
}

pub struct Logger;

lazy_static! {
    pub static ref APP_LOGS: Mutex<Vec<String>> = Mutex::new(Vec::new());
}

impl Logger {
    pub fn new() -> Self {
        Logger
    }

    pub fn log_with_module(&self, level: LogLevel, module_path: &str, message: &str) {
        let module_name = module_path
            .split("::")
            .last()
            .unwrap_or(module_path)
            .to_uppercase();

        let formatted_log = match level {
            LogLevel::Info => format!("[INFO # {module_name}] {message}"),
            LogLevel::Warn => format!("[WARN # {module_name}] {message}"),
            LogLevel::Error => format!("[ERROR # {module_name}] {message}"),
            LogLevel::Debug => format!("[DEBUG # {module_name}] {message}"),
        };

        match level {
            LogLevel::Info => {
                println!("{} {}", format!("[INFO # {module_name}]").green(), message);
            }
            LogLevel::Warn => {
                println!("{} {}", format!("[WARN # {module_name}]").yellow(), message);
            }
            LogLevel::Error => {
                println!("{} {}", format!("[ERROR # {module_name}]").red(), message);
            }
            LogLevel::Debug => {
                println!("{} {}", format!("[DEBUG # {module_name}]").cyan(), message);
            }
        }

        if let Ok(mut app_logs) = APP_LOGS.lock() {
            app_logs.push(formatted_log);
        }
    }
}

lazy_static! {
    pub static ref LOGGER: Logger = Logger::new();
}

#[macro_export]
macro_rules! log_info {
    ($($arg:tt)*) => {
        $crate::api::logging::LOGGER.log_with_module(
            $crate::api::logging::LogLevel::Info,
            module_path!(),
            &format!($($arg)*)
        )
    };
}

#[macro_export]
macro_rules! log_warn {
    ($($arg:tt)*) => {
        $crate::api::logging::LOGGER.log_with_module(
            $crate::api::logging::LogLevel::Warn,
            module_path!(),
            &format!($($arg)*)
        )
    };
}

#[macro_export]
macro_rules! log_error {
    ($($arg:tt)*) => {
        $crate::api::logging::LOGGER.log_with_module(
            $crate::api::logging::LogLevel::Error,
            module_path!(),
            &format!($($arg)*)
        )
    };
}

#[macro_export]
macro_rules! log_debug {
    ($($arg:tt)*) => {
        $crate::api::logging::LOGGER.log_with_module(
            $crate::api::logging::LogLevel::Debug,
            module_path!(),
            &format!($($arg)*)
        )
    };
}
