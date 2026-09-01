use crate::core::platform::error::StartupError;
use crate::log_warn;
use std::path::Path;

/// Checks if the WebKitGTK shared library is available on the system.
/// new since 18.08.2026
/// on user pc old pkg-config variant isn't worked, so now we check for the lib in folders
fn has_webkit2gtk_library() -> bool {
    let search_dirs = ["/usr/lib", "/usr/lib64", "/usr/local/lib"];

    for dir in &search_dirs {
        let Ok(entries) = std::fs::read_dir(dir) else {
            continue;
        };

        for entry in entries.flatten() {
            let name = entry.file_name();
            let name = name.to_string_lossy();

            if entry.path().is_dir() {
                if let Ok(sub) = std::fs::read_dir(entry.path()) {
                    for sub_entry in sub.flatten() {
                        let sub_name = sub_entry.file_name();
                        let sub_name = sub_name.to_string_lossy();
                        if (sub_name.starts_with("libwebkit2gtk-4.1.so")
                            || sub_name.starts_with("libwebkit2gtk-4.0.so"))
                            && sub_entry.path().is_file()
                        {
                            return true;
                        }
                    }
                }
            }

            if (name.starts_with("libwebkit2gtk-4.1.so")
                || name.starts_with("libwebkit2gtk-4.0.so"))
                && entry.path().is_file()
            {
                return true;
            }
        }
    }

    if let Ok(conf) = std::fs::read_to_string("/etc/ld.so.conf") {
        for line in conf.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if line.starts_with("include ") {
                let pattern = line.strip_prefix("include ").unwrap_or(line);
                if let Some((dir_part, glob_part)) = pattern.rsplit_once('/') {
                    if glob_part == "*" {
                        if let Ok(entries) = std::fs::read_dir(dir_part) {
                            for entry in entries.flatten() {
                                if let Ok(include_conf) = std::fs::read_to_string(entry.path()) {
                                    for inc_line in include_conf.lines() {
                                        let inc_line = inc_line.trim();
                                        if !inc_line.is_empty() && !inc_line.starts_with('#') {
                                            if check_dir_for_webkit(Path::new(inc_line)) {
                                                return true;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            } else if check_dir_for_webkit(Path::new(line)) {
                return true;
            }
        }
    }

    false
}

fn check_dir_for_webkit(dir: &Path) -> bool {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return false;
    };

    for entry in entries.flatten() {
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if (name.starts_with("libwebkit2gtk-4.1.so") || name.starts_with("libwebkit2gtk-4.0.so"))
            && entry.path().is_file()
        {
            return true;
        }
    }

    false
}

pub fn check_platform_dependencies() -> Result<(), StartupError> {
    if !has_webkit2gtk_library() {
        log_warn!(
            "WebKitGTK shared library (libwebkit2gtk-4.1.so or libwebkit2gtk-4.0.so) was not found. \
             The app may fail to start if WebKitGTK is not installed."
        );
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
