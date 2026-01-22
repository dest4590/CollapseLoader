use crate::core::platform::error::StartupError;
use std::process::Command;

fn has_pkg_config_package(name: &str) -> bool {
    Command::new("pkg-config")
        .args(["--exists", name])
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

fn has_library(name: &str) -> bool {
    let output = Command::new("ldconfig")
        .arg("-p")
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
        .unwrap_or_default();

    output.contains(name)
}

pub fn check_platform_dependencies() -> Result<(), StartupError> {
    let mut ok =
        has_pkg_config_package("webkit2gtk-4.1") || has_pkg_config_package("webkit2gtk-4.0");

    if !ok {
        ok = has_library("libwebkit2gtk-4.1.so") || has_library("libwebkit2gtk-4.0.so");
    }

    if !ok {
        return Err(StartupError::LinuxDependenciesMissing);
    }

    Ok(())
}

pub fn check_webkit_environment() -> Result<(), StartupError> {
    let is_wayland = std::env::var("XDG_SESSION_TYPE")
        .map(|v| v == "wayland")
        .unwrap_or(false);

    match std::env::var("WEBKIT_DISABLE_DMABUF_RENDERER") {
        Ok(value) => {
            if value != "1" {
                if is_wayland {
                    return Err(StartupError::LinuxWebKitWaylandWarning);
                }
                return Err(StartupError::LinuxWebKitWarning);
            }
        }
        Err(_) => {
            if is_wayland {
                return Err(StartupError::LinuxWebKitWaylandWarning);
            }
            return Err(StartupError::LinuxWebKitWarning);
        }
    }

    Ok(())
}
