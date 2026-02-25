use crate::core::utils::helpers::emit_to_main_window;
use crate::log_error;
use crate::log_warn;
use futures_util::StreamExt;
use reqwest::Client;
use std::path::Path;
use std::sync::LazyLock;
use std::time::Duration;
use tokio::fs;
use tokio::io::AsyncWriteExt;

const MAX_RETRIES: u32 = 3;
const RETRY_DELAY: Duration = Duration::from_secs(2);
static HTTP_CLIENT: LazyLock<Client> = LazyLock::new(|| {
    Client::builder()
        .timeout(Duration::from_secs(600))
        .build()
        .unwrap_or_default()
});

pub async fn download_file(
    urls: &[String],
    dest_path: &Path,
    emit_name: &str,
    app_handle: Option<&tauri::AppHandle>,
) -> Result<(), String> {
    let mut last_error = String::from("Unknown error");

    for url in urls {
        for attempt in 1..=MAX_RETRIES {
            if attempt > 1 {
                log_warn!(
                    "Retrying download for {} (Attempt {}/{})",
                    emit_name,
                    attempt,
                    MAX_RETRIES
                );
                tokio::time::sleep(RETRY_DELAY).await;
            }

            match perform_download(url, dest_path, emit_name, app_handle).await {
                Ok(_) => return Ok(()),
                Err(e) => {
                    last_error = e;
                    log_error!(
                        "Download failed for {} (Attempt {}/{}): {}",
                        emit_name,
                        attempt,
                        MAX_RETRIES,
                        last_error
                    );
                }
            }
        }
        log_warn!(
            "Failed to download {} from {} after {} attempts. Trying fallback if available...",
            emit_name,
            url,
            MAX_RETRIES
        );
    }

    Err(format!(
        "Failed to download {} after {} attempts: {}",
        emit_name, MAX_RETRIES, last_error
    ))
}

async fn perform_download(
    url: &str,
    dest_path: &Path,
    emit_name: &str,
    app_handle: Option<&tauri::AppHandle>,
) -> Result<(), String> {
    let response = HTTP_CLIENT.get(url).send().await.map_err(|e| {
        log_error!("Failed to make HTTP request to {}: {}", url, e);
        format!("Network request failed: {e}")
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

    let mut temp_path = dest_path.to_path_buf();
    let temp_extension = match dest_path.extension().and_then(|ext| ext.to_str()) {
        Some(ext) => format!("{ext}.part"),
        None => "part".to_string(),
    };
    temp_path.set_extension(temp_extension);

    let mut dest = fs::File::create(&temp_path).await.map_err(|e| {
        log_error!("Failed to create temp file {}: {}", temp_path.display(), e);
        format!("Temp file creation error: {e}")
    })?;

    let mut downloaded: u64 = 0;
    let mut last_percentage: u8 = 0;
    let mut stream = response.bytes_stream();

    let write_result: Result<(), String> = async {
        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|e| {
                log_error!("Failed to read response data for {}: {}", emit_name, e);
                format!("Network stream error: {e}")
            })?;

            dest.write_all(&chunk).await.map_err(|e| {
                log_error!(
                    "Failed to write data to temp file {}: {}",
                    temp_path.display(),
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
            log_error!("Failed to flush temp file {}: {}", temp_path.display(), e);
            format!("File flush error: {e}")
        })?;

        Ok(())
    }
    .await;

    if let Err(e) = write_result {
        let _ = fs::remove_file(&temp_path).await;
        return Err(e);
    }

    if fs::try_exists(dest_path).await.unwrap_or(false) {
        fs::remove_file(dest_path).await.map_err(|e| {
            log_error!(
                "Failed to replace destination file {}: {}",
                dest_path.display(),
                e
            );
            format!("Failed to replace destination file: {e}")
        })?;
    }

    fs::rename(&temp_path, dest_path).await.map_err(|e| {
        log_error!(
            "Failed to finalize download from {} to {}: {}",
            temp_path.display(),
            dest_path.display(),
            e
        );
        format!("Failed to finalize downloaded file: {e}")
    })?;

    Ok(())
}
