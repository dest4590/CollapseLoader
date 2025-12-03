use chrono::Local;
use colored::Colorize;
use std::collections::VecDeque;
use std::fmt;
use std::sync::{LazyLock, Mutex};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    Error = 1,
    Warn = 2,
    Info = 3,
    Debug = 4,
}

pub struct Logger;

pub static APP_LOGS: LazyLock<Mutex<VecDeque<String>>> =
    LazyLock::new(|| Mutex::new(VecDeque::new()));
pub static LOG_LEVEL: LazyLock<Mutex<LogLevel>> = LazyLock::new(|| Mutex::new(LogLevel::Debug));
pub static STARTUP_PRINTED: LazyLock<Mutex<bool>> = LazyLock::new(|| Mutex::new(false));

const MAX_APP_LOGS: usize = 1000;

impl Logger {
    pub fn log_with_module(level: LogLevel, tag: &str, message: &str) {
        let timestamp = Local::now().format("%H:%M:%S").to_string();

        let short = tag
            .rsplit(|c: char| ['.', ':', '/'].contains(&c))
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

        let mut shorted_tag = {
            let slice = if let Some(pos) = tag.rfind("collapseloader_lib.") {
                &tag[pos + "collapseloader_lib.".len()..]
            } else {
                tag
            };
            slice.replace("collapse.module.collapseloader_lib", "core.init")
        };

        if shorted_tag.starts_with("commands.") {
            shorted_tag.insert_str(0, "tauri.");
        }

        let ts_colored = timestamp.dimmed();
        let tag_colored = shorted_tag.white();

        let emoji = Self::emoji_for_module(&shorted_tag)
            .map(|e| format!(" {e} |"))
            .unwrap_or_default();

        println!(
            "{} [{} #{} {}] {}",
            ts_colored,
            level_colored,
            emoji,
            tag_colored.bold(),
            message
        );

        let plain = format!("{timestamp} [{level_name}] [{short}] {message}");
        if let Ok(mut app_logs) = APP_LOGS.lock() {
            app_logs.push_back(plain);
            if app_logs.len() > MAX_APP_LOGS {
                app_logs.pop_front();
            }
        }
    }

    fn emoji_for_module(tag: &str) -> Option<&'static str> {
        if tag.contains("core.network") {
            Some("\u{2601}")
        } else if tag.contains("core.clients") {
            Some("\u{2609}")
        } else if tag.contains("core.storage") {
            Some("\u{26C3}")
        } else if tag.contains("core.utils") {
            Some("\u{2692}")
        } else if tag.contains("core.init") {
            Some("\u{2699}")
        } else if tag.contains("commands.") {
            Some("\u{25CF}")
        } else {
            None
        }
    }
}

impl Logger {
    pub fn print_startup_banner(
        version: &str,
        codename: &str,
        is_dev: bool,
        git_hash: &str,
        git_branch: &str,
    ) {
        if let Ok(mut printed) = STARTUP_PRINTED.lock() {
            if *printed {
                return;
            }
            *printed = true;
        }

        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

        let header = format!("CollapseLoader v{} ({})", version, codename);
        let dev_info = if is_dev {
            format!("development build - {} ({})", git_hash, git_branch)
        } else {
            String::new()
        };

        let os = std::env::consts::OS;
        let arch = std::env::consts::ARCH;

        println!();
        println!("{}", "========================================".dimmed());
        println!("{}", header.green().bold());
        if !dev_info.is_empty() {
            println!("{}", dev_info.yellow());
        }
        println!(
            "{} {} | {}:{}",
            "startup:".white(),
            timestamp.dimmed(),
            os,
            arch
        );
        println!("{}", "========================================".dimmed());
        println!();

        let plain = format!("{timestamp} [STARTUP] {header} {dev_info} {os}/{arch}");
        if let Ok(mut app_logs) = APP_LOGS.lock() {
            app_logs.push_back(plain);
            if app_logs.len() > MAX_APP_LOGS {
                app_logs.pop_front();
            }
        }
    }
}

impl LogLevel {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Info => "INFO",
            Self::Warn => "WARN",
            Self::Error => "ERROR",
            Self::Debug => "DEBUG",
        }
    }
}

impl fmt::Display for LogLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[macro_export]
macro_rules! log_info {
    ($($arg:tt)*) => {
        {
            let __clp_tag = $crate::collapse_tag!();
            $crate::core::utils::logging::Logger::log_with_module(
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
            $crate::core::utils::logging::Logger::log_with_module(
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
            $crate::core::utils::logging::Logger::log_with_module(
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
            $crate::core::utils::logging::Logger::log_with_module(
                $crate::core::utils::logging::LogLevel::Debug,
                &__clp_tag,
                &format!($($arg)*)
            )
        }
    };
}
