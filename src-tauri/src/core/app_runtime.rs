use crate::core::platform::{check_platform_dependencies, error::StartupError};
#[cfg(target_os = "linux")]
use crate::core::platform::check_webkit_environment;
use crate::core::storage::settings::SETTINGS;
use crate::core::utils::globals::CODENAME;
use crate::logging::Logger;
use std::sync::{Mutex, OnceLock, PoisonError};
use std::time::{Duration, Instant};
use tauri::Manager;
#[cfg(target_os = "windows")]
use windows::Win32::UI::HiDpi::{SetProcessDpiAwareness, PROCESS_SYSTEM_DPI_AWARE};

pub struct StartupRuntime;

impl StartupRuntime {
    pub fn prepare() -> Result<(), StartupError> {
        configure_dpi_awareness()?;
        check_platform_dependencies()?;

        #[cfg(target_os = "linux")]
        {
            crate::log_info!("Checking WebKit environment variables...");
            check_webkit_environment()?;
        }

        Ok(())
    }
}

pub struct StartupMetadata {
    version: &'static str,
    codename: String,
    is_dev: bool,
    git_hash: String,
    git_branch: String,
}

impl StartupMetadata {
    pub fn from_env() -> Self {
        Self {
            version: env!("CARGO_PKG_VERSION"),
            codename: CODENAME.to_string().to_uppercase(),
            is_dev: env!("DEVELOPMENT") == "true",
            git_hash: env!("GIT_HASH").chars().take(7).collect(),
            git_branch: env!("GIT_BRANCH").to_string(),
        }
    }

    pub fn configure_main_window(&self, app_handle: &tauri::AppHandle) {
        if let Some(window) = app_handle.get_webview_window("main") {
            if let Err(error) = window.set_title(&self.window_title()) {
                crate::log_warn!("Failed to set window title: {}", error);
            }

            #[cfg(target_os = "macos")]
            if let Err(error) = window.set_decorations(true) {
                crate::log_warn!("Failed to enable window decorations: {}", error);
            }

            let start_minimized = SETTINGS
                .lock()
                .map(|settings| settings.start_minimized.value)
                .unwrap_or(false);

            if start_minimized {
                let _ = window.hide();
            }
        }
    }

    pub fn print_banner(&self) {
        Logger::print_startup_banner(
            self.version,
            &self.codename,
            self.is_dev,
            &self.git_hash,
            &self.git_branch,
        );
    }

    fn window_title(&self) -> String {
        let build_label = if self.is_dev {
            format!(
                "(development build, {}, {} branch)",
                self.git_hash, self.git_branch
            )
        } else {
            String::new()
        };

        format!(
            "CollapseLoader v{} ({}) {}",
            self.version, self.codename, build_label
        )
    }
}

pub enum TrayMenuAction {
    Show,
    Quit,
    LaunchClient(u32),
    Ignore,
}

impl TrayMenuAction {
    pub fn parse(action_id: &str) -> Self {
        match action_id {
            "show" => Self::Show,
            "quit" => Self::Quit,
            _ => action_id
                .strip_prefix("launch_")
                .and_then(|id| id.parse::<u32>().ok())
                .map(Self::LaunchClient)
                .unwrap_or(Self::Ignore),
        }
    }
}

pub enum DeepLinkAction {
    VerifyEmail { code: String, email: String },
    LaunchClient { client_id: String },
}

impl DeepLinkAction {
    pub fn parse(url: &str) -> Option<Self> {
        if url.contains("verify") {
            let code = query_value(url, "code")?;
            let email = query_value(url, "email").unwrap_or_default();

            return Some(Self::VerifyEmail { code, email });
        }

        query_value(url, "client").map(|client_id| Self::LaunchClient { client_id })
    }
}

pub struct DeepLinkDeduplicator;

impl DeepLinkDeduplicator {
    pub fn should_handle(url: &str) -> bool {
        static LAST_HANDLED: OnceLock<Mutex<Option<(String, Instant)>>> = OnceLock::new();

        let last_handled = LAST_HANDLED.get_or_init(|| Mutex::new(None));
        let mut guard = last_handled
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        let normalized = url.trim();

        if let Some((previous_url, previous_time)) = guard.as_ref() {
            if previous_url == normalized && previous_time.elapsed() < Duration::from_secs(2) {
                return false;
            }
        }

        *guard = Some((normalized.to_string(), Instant::now()));
        true
    }
}

pub(crate) fn query_value(url: &str, key: &str) -> Option<String> {
    let (_, query) = url.split_once('?')?;

    query.split('&').find_map(|pair| {
        let (candidate_key, value) = pair.split_once('=')?;
        (candidate_key == key).then(|| value.to_string())
    })
}

#[cfg(target_os = "windows")]
fn configure_dpi_awareness() -> Result<(), StartupError> {
    unsafe {
        SetProcessDpiAwareness(PROCESS_SYSTEM_DPI_AWARE)
            .map_err(|error| StartupError::DpiAwarenessFailed(error.to_string()))?;
    }

    Ok(())
}

#[cfg(not(target_os = "windows"))]
fn configure_dpi_awareness() -> Result<(), StartupError> {
    Ok(())
}