use crate::core::error::StartupError;

pub fn check_platform_dependencies() -> Result<(), StartupError> {
    let result = std::process::Command::new("pkg-config")
        .args(["--print-errors", "webkit2gtk-4.1"])
        .output();

    match result {
        Ok(output) => {
            if !output.status.success() {
                return Err(StartupError::LinuxDependenciesMissing);
            }
        }
        Err(_) => {
            return Err(StartupError::LinuxDependenciesMissing);
        }
    }

    Ok(())
}

pub fn check_webkit_environment() -> Result<(), StartupError> {
    match std::env::var("WEBKIT_DISABLE_DMABUF_RENDERER") {
        Ok(value) => {
            if value != "1" {
                return Err(StartupError::LinuxWebKitWarning);
            }
        }
        Err(_) => {
            return Err(StartupError::LinuxWebKitWarning);
        }
    }

    Ok(())
}
