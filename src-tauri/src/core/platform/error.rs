use crate::log_error;
use native_dialog::{DialogBuilder, MessageLevel};

/// Errors that can occur during the application's startup phase.
#[derive(Debug, thiserror::Error)]
pub enum StartupError {
    /// WebView2 is not installed on Windows.
    #[error("WebView2 is not installed. Please install it from https://developer.microsoft.com/en-us/microsoft-edge/webview2/")]
    WebView2NotInstalled,
    /// Automatic installation of WebView2 failed.
    #[error("Failed to install WebView2. Please install it manually from https://developer.microsoft.com/en-us/microsoft-edge/webview2/")]
    WebView2InstallFailed,
    /// Failed to check if WebView2 is installed.
    #[error("Failed to check for WebView2: {0}")]
    WebView2CheckFailed(String),

    /// Required Linux dependencies (WebKitGTK) are missing.
    #[cfg(target_os = "linux")]
    #[error(
        "Linux dependencies are missing (WebKitGTK).\n\n\
        CollapseLoader needs WebKitGTK and pkg-config to run.\n\n\
        Try one of these (depending on your distro):\n\
        • Debian/Ubuntu: sudo apt-get install pkg-config libwebkit2gtk-4.1-0\n\
          (older: sudo apt-get install pkg-config libwebkit2gtk-4.0-37)\n\
        • Fedora: sudo dnf install pkgconf-pkg-config webkit2gtk4.1-devel\n\
        • Arch: sudo pacman -S pkgconf webkit2gtk-4.1\n\
        • openSUSE: sudo zypper install pkg-config webkit2gtk4.1-devel\n\n\
        Tip: after installing, log out/in (or reboot) if your desktop session still behaves weird."
    )]
    LinuxDependenciesMissing,

    /// Warning about WebKit rendering issues on Linux.
    #[cfg(target_os = "linux")]
    #[error(
        "Linux rendering tip: WEBKIT_DISABLE_DMABUF_RENDERER is not set to 1.\n\n\
        If you get a white screen / flickering / GPU rendering issues, launch like:\n\
        WEBKIT_DISABLE_DMABUF_RENDERER=1 collapseloader\n\n\
        Extra tips if it still happens:\n\
        • Make sure your Mesa/NVIDIA drivers are up-to-date."
    )]
    LinuxWebKitWarning,

    /// Warning about WebKit rendering issues on Linux with Wayland.
    #[cfg(target_os = "linux")]
    #[error(
        "Linux Wayland rendering tip: WEBKIT_DISABLE_DMABUF_RENDERER is not set to 1.\n\n\
        On Wayland, you might need to force X11 if you experience flickering or a white screen:\n\
        GDK_BACKEND=x11 WEBKIT_DISABLE_DMABUF_RENDERER=1 collapseloader\n\n\
        Alternatively, try setting the environment variable manually:\n\
        WEBKIT_DISABLE_DMABUF_RENDERER=1 collapseloader"
    )]
    LinuxWebKitWaylandWarning,

    /// Failed to set DPI awareness on Windows.
    #[error("Failed to set DPI awareness: {0}")]
    DpiAwarenessFailed(String),
}

impl StartupError {
    /// Displays the error in a native dialog and exits the application.
    pub fn show_and_exit(&self) {
        let title = "Startup Error";
        let message = self.to_string();

        log_error!("Startup Error: {}", message);

        let _ = DialogBuilder::message()
            .set_level(MessageLevel::Error)
            .set_title(title)
            .set_text(&message)
            .alert()
            .show();

        std::process::exit(1);
    }

    /// Displays the error as a warning in a native dialog without exiting.
    pub fn show_warning(&self) {
        let title = "Warning";
        let message = self.to_string();

        eprintln!("\n==== WARNING ====");
        eprintln!("{}", message);
        eprintln!("================\n");

        let _ = std::panic::catch_unwind(|| {
            let _ = DialogBuilder::message()
                .set_level(MessageLevel::Warning)
                .set_title(title)
                .set_text(&message)
                .alert()
                .show();
        });
    }
}
