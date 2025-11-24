use crate::core::utils::helpers::emit_to_main_window;
use crate::{log_error, log_info};
use futures_util::StreamExt;
use std::path::Path;
use std::time::Duration;
use tokio::io::AsyncWriteExt;

pub async fn download_file(
    url: &str,
    dest_path: &Path,
    emit_name: &str,
    app_handle: Option<&tauri::AppHandle>,
) -> Result<(), String> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(600))
        .build()
        .map_err(|e| {
            log_error!("Failed to create HTTP client: {}", e);
            format!("Failed to create HTTP client: {e}")
        })?;

    let response = client.get(url).send().await.map_err(|e| {
        log_error!("Failed to make HTTP request to {}: {}", url, e);
        format!("Failed to download file {emit_name}: {e}")
    })?;

    if !response.status().is_success() {
        let error_msg = format!(
            "Failed to download file {}: HTTP {} - {}",
            emit_name,
            response.status().as_u16(),
            response
                .status()
                .canonical_reason()
                .unwrap_or("Unknown error")
        );
        log_error!("{}", error_msg);
        return Err(error_msg);
    }

    let total_size = response.content_length();

    let mut dest = tokio::fs::File::create(dest_path).await.map_err(|e| {
        log_error!(
            "Failed to create destination file {}: {}",
            dest_path.display(),
            e
        );
        format!("Failed to create file: {e}")
    })?;

    log_info!("Created destination file: {}", dest_path.display());

    let mut downloaded: u64 = 0;
    let mut last_percentage: u8 = 0;
    let mut stream = response.bytes_stream();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| {
            log_error!("Failed to read response data for {}: {}", emit_name, e);
            format!("Network read error: {e}")
        })?;

        dest.write_all(&chunk).await.map_err(|e| {
            log_error!(
                "Failed to write data to file {}: {}",
                dest_path.display(),
                e
            );
            format!("File write error: {e}")
        })?;

        downloaded += chunk.len() as u64;

        let percentage = total_size.map_or_else(
            || std::cmp::min(99, (downloaded / 1024 / 1024) as u8),
            |total| ((downloaded as f64 / total as f64) * 100.0) as u8,
        );

        if percentage != last_percentage {
            last_percentage = percentage;
            if let Some(handle) = app_handle {
                let progress_data = serde_json::json!({
                    "file": emit_name,
                    "percentage": percentage,
                    "downloaded": downloaded,
                    "total": total_size.unwrap_or(0),
                    "action": "downloading"
                });
                emit_to_main_window(handle, "download-progress", progress_data);
            }
        }
    }

    dest.flush().await.map_err(|e| {
        log_error!("Failed to flush file {}: {}", dest_path.display(), e);
        format!("File flush error: {e}")
    })?;

    Ok(())
}
