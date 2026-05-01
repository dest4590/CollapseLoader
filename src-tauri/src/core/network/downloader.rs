use crate::core::utils::helpers::emit_to_main_window;
use crate::log_error;
use crate::log_info;
use crate::log_warn;
use futures_util::StreamExt;
use reqwest::Client;
use std::path::Path;
use std::sync::LazyLock;
use std::time::{Duration, Instant};
use tokio::fs;
use tokio::io::AsyncWriteExt;

const MAX_RETRIES: u32 = 3;
const BASE_RETRY_DELAY: Duration = Duration::from_secs(2);
const MAX_RETRY_DELAY: Duration = Duration::from_secs(16);
const PROGRESS_EMIT_INTERVAL: Duration = Duration::from_millis(200);

static HTTP_CLIENT: LazyLock<Client> =
    LazyLock::new(|| super::create_client(Duration::from_secs(600)));

fn human_size(bytes: u64) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = KB * 1024.0;
    const GB: f64 = MB * 1024.0;
    let b = bytes as f64;
    if b >= GB {
        format!("{:.2} GB", b / GB)
    } else if b >= MB {
        format!("{:.1} MB", b / MB)
    } else if b >= KB {
        format!("{:.0} KB", b / KB)
    } else {
        format!("{b} B")
    }
}

fn backoff_delay(attempt: u32) -> Duration {
    let secs = BASE_RETRY_DELAY.as_secs_f64() * 2f64.powi(attempt as i32 - 1);
    let capped = secs.min(MAX_RETRY_DELAY.as_secs_f64());
    let jitter = capped * 0.1 * ((attempt % 3) as f64 - 1.0);
    Duration::from_secs_f64((capped + jitter).max(0.1))
}

pub async fn download_file(
    urls: &[String],
    dest_path: &Path,
    emit_name: &str,
    app_handle: Option<&tauri::AppHandle>,
) -> Result<(), String> {
    let mut last_error = String::from("Unknown error");

    for (url_idx, url) in urls.iter().enumerate() {
        for attempt in 1..=MAX_RETRIES {
            if attempt > 1 {
                let delay = backoff_delay(attempt);
                log_warn!(
                    "Retrying download for {} (attempt {}/{}, waiting {:.1}s) ...",
                    emit_name,
                    attempt,
                    MAX_RETRIES,
                    delay.as_secs_f64()
                );
                tokio::time::sleep(delay).await;
            }

            match perform_download(url, dest_path, emit_name, app_handle).await {
                Ok(_) => return Ok(()),
                Err(e) => {
                    last_error = e;
                    log_error!(
                        "Download failed for {} (attempt {}/{}): {}",
                        emit_name,
                        attempt,
                        MAX_RETRIES,
                        last_error
                    );
                }
            }
        }

        if url_idx + 1 < urls.len() {
            log_warn!(
                "All {} attempts failed for {} on URL {}. Trying next URL...",
                MAX_RETRIES,
                emit_name,
                url
            );
        } else {
            log_warn!(
                "All {} attempts failed for {} on URL {}. No more URLs available.",
                MAX_RETRIES,
                emit_name,
                url
            );
        }
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
        log_error!("Network request failed for {}: {}", emit_name, e);
        format!("Network request failed: {e}")
    })?;

    if !response.status().is_success() {
        let status = response.status();
        let error_msg = format!(
            "HTTP {} ({}) downloading {}",
            status.as_u16(),
            status.canonical_reason().unwrap_or("Unknown"),
            emit_name,
        );
        log_error!("{}", error_msg);
        return Err(error_msg);
    }

    let total_size = response.content_length();

    if let Some(total) = total_size {
        log_info!("Downloading {} ({}) ...", emit_name, human_size(total));
    }

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
    let mut last_emitted_pct: u8 = 0;
    let mut last_emit_time = Instant::now();
    let download_start = Instant::now();
    let mut speed_window_bytes: u64 = 0;
    let mut speed_window_start = Instant::now();
    let mut current_speed_bps: f64 = 0.0;
    let mut stream = response.bytes_stream();

    let write_result: Result<(), String> = async {
        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|e| {
                log_error!("Stream read error for {}: {}", emit_name, e);
                format!("Network stream error: {e}")
            })?;

            dest.write_all(&chunk).await.map_err(|e| {
                log_error!("Write error for {}: {}", temp_path.display(), e);
                format!("File write error: {e}")
            })?;

            let chunk_len = chunk.len() as u64;
            downloaded += chunk_len;
            speed_window_bytes += chunk_len;

            let speed_elapsed = speed_window_start.elapsed();
            if speed_elapsed >= Duration::from_millis(500) {
                current_speed_bps = speed_window_bytes as f64 / speed_elapsed.as_secs_f64();
                speed_window_bytes = 0;
                speed_window_start = Instant::now();
            }

            let percentage = total_size.map_or_else(
                || std::cmp::min(99, (downloaded / 1024 / 1024) as u8),
                |total| ((downloaded as f64 / total as f64) * 100.0).min(99.0) as u8,
            );

            let now = Instant::now();
            let should_emit = percentage != last_emitted_pct
                || now.duration_since(last_emit_time) >= PROGRESS_EMIT_INTERVAL;

            if should_emit {
                last_emitted_pct = percentage;
                last_emit_time = now;
                if let Some(handle) = app_handle {
                    let progress_data = serde_json::json!({
                        "file": emit_name,
                        "percentage": percentage,
                        "downloaded": downloaded,
                        "total": total_size.unwrap_or(0),
                        "speed_bps": current_speed_bps as u64,
                        "elapsed_ms": download_start.elapsed().as_millis() as u64,
                        "action": "downloading"
                    });
                    emit_to_main_window(handle, "download-progress", progress_data);
                }
            }
        }

        dest.flush().await.map_err(|e| {
            log_error!("Flush error for {}: {}", temp_path.display(), e);
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
            "Failed to finalize {} ({} -> {}): {}",
            emit_name,
            temp_path.display(),
            dest_path.display(),
            e
        );
        format!("Failed to finalize downloaded file: {e}")
    })?;

    let elapsed = download_start.elapsed();
    let avg_speed = if elapsed.as_secs_f64() > 0.0 {
        downloaded as f64 / elapsed.as_secs_f64()
    } else {
        0.0
    };
    log_info!(
        "Downloaded {} – {} in {:.1}s (avg {}ps)",
        emit_name,
        human_size(downloaded),
        elapsed.as_secs_f64(),
        human_size(avg_speed as u64)
    );

    Ok(())
}
