use crate::core::utils::helpers::emit_to_main_window;
use crate::core::utils::taskbar;
use crate::log_error;
use crate::log_info;
use crate::log_warn;
use futures_util::StreamExt;
use reqwest::Client;
use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::sync::{LazyLock, Mutex};
use std::time::{Duration, Instant};
use tokio::fs;
use tokio::io::{AsyncWriteExt, BufWriter};
use tokio::sync::oneshot;

const MAX_RETRIES: u32 = 3;
const BASE_RETRY_DELAY: Duration = Duration::from_secs(2);
const MAX_RETRY_DELAY: Duration = Duration::from_secs(16);
const PROGRESS_EMIT_INTERVAL: Duration = Duration::from_millis(200);

static HTTP_CLIENT: LazyLock<Client> =
    LazyLock::new(|| super::create_client(Duration::from_secs(600)));

static ACTIVE_DOWNLOADS: LazyLock<Mutex<HashSet<String>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));

static CANCEL_CHANNELS: LazyLock<Mutex<HashMap<String, oneshot::Sender<()>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

pub fn cancel_download(name: &str) -> bool {
    let mut lock = CANCEL_CHANNELS.lock().unwrap();
    if let Some(tx) = lock.remove(name) {
        let _ = tx.send(());
        return true;
    }
    false
}

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
        format!("{bytes} B")
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
    {
        let mut active = ACTIVE_DOWNLOADS.lock().unwrap();
        if active.contains(emit_name) {
            log_info!("Download already in progress for {}, skipping", emit_name);
            return Ok(());
        }
        active.insert(emit_name.to_string());
    }

    struct ActiveGuard(String);
    impl Drop for ActiveGuard {
        fn drop(&mut self) {
            let mut active = ACTIVE_DOWNLOADS.lock().unwrap();
            active.remove(&self.0);
        }
    }
    let _active_guard = ActiveGuard(emit_name.to_string());

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
        "Failed to download {} after {} attempts across {} URL(s): {}",
        emit_name,
        MAX_RETRIES * urls.len() as u32,
        urls.len(),
        last_error
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
        log_info!("Downloading {} ({})", emit_name, human_size(total));

        // Disk space check
        if let Some(parent) = dest_path.parent() {
            let disks = sysinfo::Disks::new_with_refreshed_list();
            if let Some(disk) = disks
                .list()
                .iter()
                .find(|d| parent.starts_with(d.mount_point()))
            {
                if disk.available_space() < (total as f64 * 1.1) as u64 {
                    let err = format!(
                        "Insufficient disk space on {}. Required: {}, Available: {}",
                        disk.mount_point().display(),
                        human_size((total as f64 * 1.1) as u64),
                        human_size(disk.available_space())
                    );
                    log_error!("{}", err);
                    return Err(err);
                }
            }
        }
    }

    let mut temp_path = dest_path.to_path_buf();
    let temp_extension = match dest_path.extension().and_then(|ext| ext.to_str()) {
        Some(ext) => format!("{ext}.part"),
        None => "part".to_string(),
    };
    temp_path.set_extension(temp_extension);

    let dest_file = fs::File::create(&temp_path).await.map_err(|e| {
        log_error!("Failed to create temp file {}: {}", temp_path.display(), e);
        format!("Temp file creation error: {e}")
    })?;
    let mut dest = BufWriter::new(dest_file);

    let mut downloaded: u64 = 0;
    let mut last_emitted_pct: u8 = 0;
    let mut last_emit_time = Instant::now();
    let download_start = Instant::now();
    let mut speed_window_bytes: u64 = 0;
    let mut speed_window_start = Instant::now();
    let mut current_speed_bps: f64 = 0.0;
    let mut first_chunk = true;
    let mut stream = response.bytes_stream();

    let (tx, mut rx) = oneshot::channel();
    {
        let mut lock = CANCEL_CHANNELS.lock().unwrap();
        lock.insert(emit_name.to_string(), tx);
    }

    struct CancelGuard(String);
    impl Drop for CancelGuard {
        fn drop(&mut self) {
            let mut lock = CANCEL_CHANNELS.lock().unwrap();
            lock.remove(&self.0);
        }
    }
    let _guard = CancelGuard(emit_name.to_string());

    let write_result: Result<(), String> = async {
        loop {
            let chunk = tokio::select! {
                _ = &mut rx => {
                    log_warn!("Download for {} was cancelled by user", emit_name);
                    return Err("Cancelled".to_string());
                }
                res = stream.next() => res,
            };

            let Some(chunk) = chunk else { break };

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

            if first_chunk {
                first_chunk = false;
                speed_window_start = Instant::now();
            }

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
                taskbar::set_progress(percentage);
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
    taskbar::clear_progress();
    log_info!(
        "Downloaded {} – {} in {:.1}s (avg {}ps)",
        emit_name,
        human_size(downloaded),
        elapsed.as_secs_f64(),
        human_size(avg_speed as u64)
    );

    Ok(())
}
