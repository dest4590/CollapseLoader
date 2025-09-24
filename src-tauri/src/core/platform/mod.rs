#[cfg(target_os = "windows")]
pub(crate) mod windows;
#[cfg(target_os = "windows")]
pub use self::windows::check_platform_dependencies;

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
pub use self::linux::{check_platform_dependencies, check_webkit_environment};

#[cfg(not(any(target_os = "windows", target_os = "linux")))]
pub fn check_platform_dependencies() -> Result<(), StartupError> {
    // No specific checks for other platforms like macOS for now
    Ok(())
}
