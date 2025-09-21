use crate::core::error::StartupError;

pub fn check_platform_dependencies() -> Result<(), StartupError> {
    // On Linux, we need to check for webkit2gtk dependencies.
    // This is a simple check, a more robust one might be needed.
    let result = std::process::Command::new("pkg-config")
        .args(["--print-errors", "webkit2gtk-4.0"])
        .output();

    match result {
        Ok(output) => {
            if !output.status.success() {
                return Err(StartupError::LinuxDependenciesMissing);
            }
        }
        Err(_) => {
            // pkg-config might not be installed, which is a problem itself.
            return Err(StartupError::LinuxDependenciesMissing);
        }
    }

    Ok(())
}
