use crate::log_error;
use native_dialog::DialogBuilder;

#[derive(Debug, thiserror::Error)]
pub enum StartupError {
    #[error("WebView2 is not installed. Please install it from https://developer.microsoft.com/en-us/microsoft-edge/webview2/")]
    WebView2NotInstalled,
    #[error("Failed to install WebView2. Please install it manually from https://developer.microsoft.com/en-us/microsoft-edge/webview2/")]
    WebView2InstallFailed,
    #[error("Failed to check for WebView2: {0}")]
    WebView2CheckFailed(String),

    #[cfg(target_os = "linux")]
    #[error("webkit2gtk dependencies are missing. Please install them. For example: \n\nDebian/Ubuntu: sudo apt-get install libwebkit2gtk-4.0-37\nArch: sudo pacman -S webkit2gtk3\nFedora: sudo dnf install webkit2gtk3-devel")]
    LinuxDependenciesMissing,

    #[cfg(target_os = "linux")]
    #[error("Warning: WEBKIT_DISABLE_DMABUF_RENDERER environment variable is not set to 1.\n\nIf you experience a white screen or rendering issues, please set this variable before launching the application:\n\nWEBKIT_DISABLE_DMABUF_RENDERER=1 collapseloader\n\nOr add it to your shell profile for permanent use.")]
    LinuxWebKitWarning,

    #[error("Failed to set DPI awareness: {0}")]
    DpiAwarenessFailed(String),
}

impl StartupError {
    pub fn show_and_exit(&self) {
        let title = "Startup Error";
        let message = self.to_string();

        log_error!("Startup Error: {}", message);

        let _ = DialogBuilder::message().set_text(&message).set_title(title);

        std::process::exit(1);
    }

    pub fn show_warning(&self) {
        let title = "Warning";
        let message = self.to_string();

        eprintln!("\n==== WARNING ====");
        eprintln!("{}", message);
        eprintln!("================\n");

        let _ = std::panic::catch_unwind(|| {
            let _ = DialogBuilder::message().set_text(&message).set_title(title);
        });
    }
}
