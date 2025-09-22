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
}

impl StartupError {
    pub fn show_and_exit(&self) {
        let title = "Startup Error";
        let message = self.to_string();

        log_error!("Startup Error: {}", message);

        DialogBuilder::message().set_text(&message).set_title(title);

        std::process::exit(1);
    }
}
