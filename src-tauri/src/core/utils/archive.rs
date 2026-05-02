use crate::core::utils::helpers::emit_to_main_window;
use crate::{log_debug, log_error};
use std::fs;
use std::io;
use std::path::Path;

pub fn unzip(
    zip_path: &Path,
    unzip_path: &Path,
    emit_name: &str,
    app_handle: Option<&tauri::AppHandle>,
) -> Result<(), String> {
    if let Some(handle) = app_handle {
        emit_to_main_window(handle, "unzip-start", emit_name);
    }

    if unzip_path.exists() {
        log_debug!(
            "Directory {} exists, will overwrite contents",
            unzip_path.display()
        );
    } else {
        log_debug!("Creating unzip directory: {}", unzip_path.display());
        fs::create_dir_all(unzip_path).map_err(|e| e.to_string())?;
    }

    if !zip_path.exists() {
        log_error!(
            "Zip file not found at expected path: {}",
            zip_path.display()
        );
        return Err(format!("Zip file not found: {}", zip_path.display()));
    }

    let file = fs::File::open(zip_path).map_err(|e| {
        log_error!("Failed to open zip file {}: {}", zip_path.display(), e);
        e.to_string()
    })?;

    let mut archive = zip::ZipArchive::new(file).map_err(|e| {
        log_error!("Failed to read zip archive {}: {}", zip_path.display(), e);
        e.to_string()
    })?;

    let total_files = archive.len() as u64;
    let mut files_extracted: u64 = 0;
    let mut last_percentage: u8 = 0;

    for i in 0..archive.len() {
        let mut file_entry = archive.by_index(i).map_err(|e| e.to_string())?;
        let outpath = unzip_path.join(file_entry.mangled_name());

        if file_entry.name().ends_with('/') {
            fs::create_dir_all(&outpath).map_err(|e| e.to_string())?;
        } else {
            if let Some(parent) = outpath.parent() {
                if !parent.exists() {
                    fs::create_dir_all(parent).map_err(|e| {
                        log_error!("Failed to create parent dir {}: {}", parent.display(), e);
                        e.to_string()
                    })?;
                }
            }
            let mut outfile = fs::File::create(&outpath).map_err(|e| {
                log_error!("Failed to create output file {}: {}", outpath.display(), e);
                e.to_string()
            })?;
            io::copy(&mut file_entry, &mut outfile).map_err(|e| {
                log_error!(
                    "Failed to write extracted file {}: {}",
                    outpath.display(),
                    e
                );
                e.to_string()
            })?;
        }

        files_extracted += 1;
        let percentage = ((files_extracted as f64 / total_files as f64) * 100.0) as u8;

        if percentage != last_percentage {
            last_percentage = percentage;
            if let Some(handle) = app_handle {
                emit_to_main_window(
                    handle,
                    "unzip-progress",
                    serde_json::json!({
                        "file": emit_name,
                        "percentage": percentage,
                        "action": "extracting",
                        "files_extracted": files_extracted,
                        "total_files": total_files
                    }),
                );
            }
        }
    }

    let _ = std::fs::File::create(unzip_path.join(".valid"));
    let _ = fs::remove_file(zip_path);

    if let Some(handle) = app_handle {
        emit_to_main_window(handle, "unzip-complete", emit_name);
    }

    Ok(())
}
