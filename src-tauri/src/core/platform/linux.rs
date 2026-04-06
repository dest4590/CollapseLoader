use crate::core::platform::error::StartupError;
use std::process::Command;

fn has_pkg_config_binary() -> bool {
    Command::new("pkg-config")
        .arg("--version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

fn has_pkg_config_package(name: &str) -> bool {
    if !has_pkg_config_binary() {
        return false;
    }

    Command::new("pkg-config")
        .args(["--exists", name])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

pub fn check_platform_dependencies() -> Result<(), StartupError> {
    if !has_pkg_config_binary() {
        return Err(StartupError::LinuxDependenciesMissing);
    }

    let ok = has_pkg_config_package("webkit2gtk-4.1") || has_pkg_config_package("webkit2gtk-4.0");

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
