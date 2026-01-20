use std::process::{Command, Output};

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

use crate::core::storage::data::DATA;
use crate::core::utils::globals::{FILE_EXTENSION, JDK21_FOLDER, JDK_FOLDERS};

pub fn get_jps_path() -> std::path::PathBuf {
    for folder in JDK_FOLDERS.iter() {
        let path = DATA
            .root_dir
            .join(folder)
            .join("bin")
            .join("jps".to_owned() + FILE_EXTENSION);
        if path.exists() {
            return path;
        }
    }

    DATA.root_dir
        .join(JDK21_FOLDER)
        .join("bin")
        .join("jps".to_owned() + FILE_EXTENSION)
}
use crate::{log_debug, log_error, log_info, log_warn};

pub fn is_java_installed() -> bool {
    let jps_path = get_jps_path();
    if jps_path.exists() {
        return true;
    }

    if Command::new("jps").arg("-l").output().is_ok() {
        return true;
    }

    if Command::new("java").arg("-version").output().is_ok() {
        return true;
    }

    false
}

pub fn execute_jps() -> Result<Output, std::io::Error> {
    let embedded = get_jps_path();
    if !embedded.exists() {
        let mut command = Command::new("jps");
        #[cfg(target_os = "windows")]
        command.creation_flags(0x0800_0000);
        return command.arg("-m").output();
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Some(bin_dir) = embedded.parent() {
            if bin_dir.exists() {
                if let Ok(entries) = std::fs::read_dir(bin_dir) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.is_file() {
                            if let Ok(mut perms) = std::fs::metadata(&path).map(|m| m.permissions())
                            {
                                let mode = perms.mode() & 0o777;
                                if mode != 0o755 {
                                    perms.set_mode(0o755);
                                    if let Err(e) = std::fs::set_permissions(&path, perms) {
                                        log_warn!(
                                            "Failed to set exec perm on {}: {}",
                                            path.display(),
                                            e
                                        );
                                    } else {
                                        log_debug!("Set exec perm on {}", path.display());
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    let mut command = Command::new(embedded);

    #[cfg(target_os = "windows")]
    command.creation_flags(0x0800_0000);

    command.arg("-m").output()
}

pub fn get_jps_output_lines() -> Vec<String> {
    match execute_jps() {
        Ok(output) => {
            let binding = String::from_utf8_lossy(&output.stdout);
            binding.lines().map(|s| s.to_string()).collect()
        }
        Err(e) => {
            log_warn!("Failed to execute jps command: {}", e);
            Vec::new()
        }
    }
}

pub fn find_processes_by_filename(filename: &str) -> Vec<String> {
    let lines = get_jps_output_lines();
    lines
        .iter()
        .filter(|line| line.contains(filename))
        .filter_map(|line| line.split_whitespace().next().map(|s| s.to_string()))
        .collect()
}

pub fn kill_process(pid: &str, client_name: &str) -> Result<bool, String> {
    log_debug!(
        "Attempting to kill process with PID: {} for '{}'",
        pid,
        client_name
    );

    #[cfg(target_os = "windows")]
    {
        let mut kill_command = Command::new("taskkill");
        kill_command.creation_flags(0x0800_0000);

        let kill_output = kill_command
            .arg("/PID")
            .arg(pid)
            .arg("/F")
            .output()
            .map_err(|e| {
                log_error!("Failed to execute taskkill for PID {}: {}", pid, e);
                format!("Failed to kill process: {e}")
            })?;

        if kill_output.status.success() {
            log_info!("Successfully killed process {} for '{}'", pid, client_name);
            return Ok(true);
        } else {
            log_error!(
                "taskkill failed for PID {}: {}",
                pid,
                String::from_utf8_lossy(&kill_output.stderr)
            );
            return Ok(false);
        }
    }

    #[cfg(unix)]
    {
        let kill_output = Command::new("kill")
            .arg("-9")
            .arg(pid)
            .output()
            .map_err(|e| {
                log_error!("Failed to execute kill for PID {}: {}", pid, e);
                format!("Failed to kill process: {e}")
            })?;

        if kill_output.status.success() {
            log_info!("Successfully killed process {} for '{}'", pid, client_name);
            return Ok(true);
        } else {
            log_error!(
                "kill failed for PID {}: {}",
                pid,
                String::from_utf8_lossy(&kill_output.stderr)
            );
            return Ok(false);
        }
    }

    log_error!("No supported kill command for this OS");
    Err("No supported kill command for this OS".to_string())
}

pub fn stop_process_by_filename(filename: &str, client_name: &str) -> Result<(), String> {
    log_info!("Attempting to stop process for '{}'", client_name);

    let lines = match execute_jps() {
        Ok(output) => {
            let binding = String::from_utf8_lossy(&output.stdout);
            binding.lines().map(|s| s.to_string()).collect::<Vec<_>>()
        }
        Err(e) => {
            log_error!("Failed to execute jps command for stopping: {}", e);
            return Err(format!("Failed to execute jps command: {e}"));
        }
    };

    let mut process_found = false;
    for line in &lines {
        if line.contains(filename) {
            process_found = true;
            let pid = line.split_whitespace().next().unwrap_or_default();
            kill_process(pid, client_name)?;
        }
    }

    if !process_found {
        log_info!("No process found for: {}", client_name);
    }

    Ok(())
}

pub fn filter_running<T, F>(items: Vec<T>, get_filename: F) -> Vec<T>
where
    F: Fn(&T) -> &str,
{
    let lines = get_jps_output_lines();
    items
        .into_iter()
        .filter(|item| lines.iter().any(|line| line.contains(get_filename(item))))
        .collect()
}
