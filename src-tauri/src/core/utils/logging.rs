use chrono::Local;
use colored::*;
use lazy_static::lazy_static;
use std::collections::VecDeque;
use std::fmt;
use std::sync::Mutex;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    Error = 1,
    Warn = 2,
    Info = 3,
    Debug = 4,
}

pub struct Logger;

lazy_static! {
    pub static ref APP_LOGS: Mutex<VecDeque<String>> = Mutex::new(VecDeque::new());
    pub static ref LOG_LEVEL: Mutex<LogLevel> = Mutex::new(LogLevel::Debug);
}

impl Logger {
    pub fn new() -> Self {
        Logger
    }

    pub fn log_with_module(&self, level: LogLevel, tag: &str, message: &str) {
        let timestamp = Local::now().format("%H:%M:%S").to_string();

        let short = tag
            .rsplit(|c: char| c == '.' || c == ':' || c == '/')
            .next()
            .unwrap_or(tag)
            .to_uppercase();

        let (level_name, level_colored) = match level {
            LogLevel::Info => ("INFO", format!("{:<5}", "INFO").green().bold()),
            LogLevel::Warn => ("WARN", format!("{:<5}", "WARN").yellow().bold()),
            LogLevel::Error => ("ERROR", format!("{:<5}", "ERROR").red().bold()),
            LogLevel::Debug => ("DEBUG", format!("{:<5}", "DEBUG").cyan().bold()),
        };

        if let Ok(gl) = LOG_LEVEL.lock() {
            if (level as i32) > (*gl as i32) {
                return;
            }
        }

        let shorted_tag = tag.rsplit("collapseloader_lib.").next().unwrap_or(tag);

        let ts_colored = timestamp.dimmed();
        let tag_colored = format!("{}", shorted_tag).white();

        fn emoji_for_module(tag: &str) -> Option<&'static str> {
            if tag.contains("core.network") {
                Some("📶")
            } else if tag.contains("core.clients") {
                Some("🎮")
            } else if tag.contains("core.storage") {
                Some("💾")
            } else if tag.contains("core.utils") {
                Some("🧰")
            } else if tag.contains("commands") {
                Some("📜")
            } else {
                None
            }
        }

        let emoji = emoji_for_module(tag)
            .map(|e| format!(" {} |", e))
            .unwrap_or_default();

        println!(
            "{} [{} #{} {}] {}",
            ts_colored,
            level_colored,
            emoji,
            format!("{}", tag_colored).bold(),
            message
        );

        let plain = format!("{} [{}] [{}] {}", timestamp, level_name, short, message);
        const MAX_APP_LOGS: usize = 1000;
        if let Ok(mut app_logs) = APP_LOGS.lock() {
            app_logs.push_back(plain);
            if app_logs.len() > MAX_APP_LOGS {
                app_logs.pop_front();
            }
        }
    }
}

lazy_static! {
    pub static ref LOGGER: Logger = Logger::new();
}

impl LogLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            LogLevel::Info => "INFO",
            LogLevel::Warn => "WARN",
            LogLevel::Error => "ERROR",
            LogLevel::Debug => "DEBUG",
        }
    }
}

impl fmt::Display for LogLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl Logger {
    /// Change global log level at runtime
    // TODO: use it somewhere
    #[allow(dead_code)]
    pub fn set_level(&self, level: LogLevel) {
        if let Ok(mut gl) = LOG_LEVEL.lock() {
            *gl = level;
        }
    }
    /// Read current global log level
    // TODO: use it somewhere
    #[allow(dead_code)]
    pub fn get_level(&self) -> LogLevel {
        if let Ok(gl) = LOG_LEVEL.lock() {
            *gl
        } else {
            LogLevel::Info
        }
    }
}

#[macro_export]
macro_rules! log_info {
    ($($arg:tt)*) => {
        {
            let __clp_tag = $crate::collapse_tag!();
            $crate::core::utils::logging::LOGGER.log_with_module(
                $crate::core::utils::logging::LogLevel::Info,
                &__clp_tag,
                &format!($($arg)*)
            )
        }
    };
}

#[macro_export]
macro_rules! log_warn {
    ($($arg:tt)*) => {
        {
            let __clp_tag = $crate::collapse_tag!();
            $crate::core::utils::logging::LOGGER.log_with_module(
                $crate::core::utils::logging::LogLevel::Warn,
                &__clp_tag,
                &format!($($arg)*)
            )
        }
    };
}

#[macro_export]
macro_rules! log_error {
    ($($arg:tt)*) => {
        {
            let __clp_tag = $crate::collapse_tag!();
            $crate::core::utils::logging::LOGGER.log_with_module(
                $crate::core::utils::logging::LogLevel::Error,
                &__clp_tag,
                &format!($($arg)*)
            )
        }
    };
}

#[macro_export]
macro_rules! log_debug {
    ($($arg:tt)*) => {
        {
            let __clp_tag = $crate::collapse_tag!();
            $crate::core::utils::logging::LOGGER.log_with_module(
                $crate::core::utils::logging::LogLevel::Debug,
                &__clp_tag,
                &format!($($arg)*)
            )
        }
    };
}
