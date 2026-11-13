use crate::core::utils::helpers::emit_to_main_window;
use crate::core::utils::taskbar;
use crate::log_error;
use crate::log_info;
use crate::log_warn;
use futures_util::stream::FuturesUnordered;
use futures_util::StreamExt;
use reqwest::Client;
use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, LazyLock, Mutex,
};
use std::time::{Duration, Instant};
use tokio::fs;
use tokio::io::{AsyncWriteExt, BufWriter};
use tokio::sync::watch;

const MAX_RETRIES: u32 = 40;
const BASE_RETRY_DELAY: Duration = Duration::from_secs(2);
const MAX_RETRY_DELAY: Duration = Duration::from_secs(30);
const PROGRESS_EMIT_INTERVAL: Duration = Duration::from_millis(200);
const WAIT_HEARTBEAT: Duration = Duration::from_secs(1);

static HTTP_CLIENT: LazyLock<Client> =
    LazyLock::new(|| super::create_client(Duration::from_secs(600)));

static ACTIVE_DOWNLOADS: LazyLock<Mutex<HashSet<String>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DownloadControl {
    Run,
    Pause,
    Cancel,
}

static CONTROL_CHANNELS: LazyLock<Mutex<HashMap<String, watch::Sender<DownloadControl>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

#[derive(Clone, serde::Serialize)]
pub struct DownloadSnapshot {
    pub file: String,
    pub percentage: u8,
    pub downloaded: u64,
    pub total: u64,
    pub speed_bps: u64,
    pub paused: bool,
    pub waiting: bool,
}

static DOWNLOAD_SNAPSHOTS: LazyLock<Mutex<HashMap<String, DownloadSnapshot>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

pub fn active_download_states() -> Vec<DownloadSnapshot> {
    DOWNLOAD_SNAPSHOTS
        .lock()
        .unwrap()
        .values()
        .cloned()
        .collect()
}

fn clear_snapshot(name: &str) {
    DOWNLOAD_SNAPSHOTS.lock().unwrap().remove(name);
}

#[allow(clippy::too_many_arguments)]
fn emit_progress(
    app_handle: Option<&tauri::AppHandle>,
    emit_name: &str,
    downloaded: u64,
    total: Option<u64>,
    percentage: u8,
    speed_bps: u64,
    elapsed_ms: u64,
    action: &str,
) {
    DOWNLOAD_SNAPSHOTS.lock().unwrap().insert(
        emit_name.to_string(),
        DownloadSnapshot {
            file: emit_name.to_string(),
            percentage,
            downloaded,
            total: total.unwrap_or(0),
            speed_bps,
            paused: action == "paused",
            waiting: action == "waiting",
        },
    );
    taskbar::set_progress(percentage);
    if let Some(handle) = app_handle {
        let progress_data = serde_json::json!({
            "file": emit_name,
            "percentage": percentage,
            "downloaded": downloaded,
            "total": total.unwrap_or(0),
            "speed_bps": speed_bps,
            "elapsed_ms": elapsed_ms,
            "action": action
        });
        emit_to_main_window(handle, "download-progress", progress_data);
    }
}

pub fn cancel_download(name: &str) -> bool {
    let mut lock = CONTROL_CHANNELS.lock().unwrap();
    if let Some(tx) = lock.remove(name) {
        tx.send_replace(DownloadControl::Cancel);
        clear_snapshot(name);
        return true;
    }
    false
}

pub fn pause_download(name: &str) -> bool {
    let lock = CONTROL_CHANNELS.lock().unwrap();
    if let Some(tx) = lock.get(name) {
        tx.send_replace(DownloadControl::Pause);
        return true;
    }
    false
}

pub fn resume_download(name: &str) -> bool {
    let lock = CONTROL_CHANNELS.lock().unwrap();
    if let Some(tx) = lock.get(name) {
        tx.send_replace(DownloadControl::Run);
        return true;
    }
    false
}

/// Blocks until the control channel leaves the Pause state.
/// Returns Err on cancel (or if the task owning the channel is gone).
async fn wait_for_resume(control_rx: &mut watch::Receiver<DownloadControl>) -> Result<(), String> {
    loop {
        match *control_rx.borrow_and_update() {
            DownloadControl::Cancel => return Err("Cancelled".to_string()),
            DownloadControl::Run => return Ok(()),
            DownloadControl::Pause => {}
        }
        if control_rx.changed().await.is_err() {
            return Err("Cancelled".to_string());
        }
    }
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

fn candidate_temp_path(dest_path: &Path, candidate_index: usize) -> std::path::PathBuf {
    let temp_extension = match dest_path.extension().and_then(|ext| ext.to_str()) {
        Some(ext) => format!("{ext}.{candidate_index}.part"),
        None => format!("part.{candidate_index}"),
    };

    let mut temp_path = dest_path.to_path_buf();
    temp_path.set_extension(temp_extension);
    temp_path
}

struct TempFileGuard {
    path: std::path::PathBuf,
    meta_path: std::path::PathBuf,
    keep: bool,
}

impl TempFileGuard {
    fn new(path: std::path::PathBuf) -> Self {
        let mut meta = path.clone().into_os_string();
        meta.push(".meta");
        Self {
            path,
            meta_path: std::path::PathBuf::from(meta),
            // Partial bytes survive failures and process restarts so the next
            // attempt can resume; only an explicit cancel throws them away.
            keep: true,
        }
    }

    fn discard(&mut self) {
        self.keep = false;
    }
}

impl Drop for TempFileGuard {
    fn drop(&mut self) {
        if !self.keep {
            let _ = std::fs::remove_file(&self.path);
            let _ = std::fs::remove_file(&self.meta_path);
        }
    }
}

/// Validator persisted next to the .part so a resume across app restarts can
/// send If-Range: a changed remote file then arrives as a full 200 response
/// instead of being appended to stale bytes.
#[derive(Default, serde::Serialize, serde::Deserialize)]
struct ResumeValidator {
    etag: Option<String>,
    last_modified: Option<String>,
}

impl ResumeValidator {
    fn if_range_value(&self) -> Option<&str> {
        self.etag.as_deref().or(self.last_modified.as_deref())
    }
}

async fn load_validator(meta_path: &Path) -> ResumeValidator {
    fs::read_to_string(meta_path)
        .await
        .ok()
        .and_then(|raw| serde_json::from_str(&raw).ok())
        .unwrap_or_default()
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

    if urls.is_empty() {
        return Err(format!(
            "Failed to download {} because no URLs were provided",
            emit_name
        ));
    }

    let (control_tx, _) = watch::channel(DownloadControl::Run);
    {
        let mut lock = CONTROL_CHANNELS.lock().unwrap();
        lock.insert(emit_name.to_string(), control_tx.clone());
    }

    struct ControlGuard(String);
    impl Drop for ControlGuard {
        fn drop(&mut self) {
            let mut lock = CONTROL_CHANNELS.lock().unwrap();
            lock.remove(&self.0);
            clear_snapshot(&self.0);
        }
    }
    let _control_guard = ControlGuard(emit_name.to_string());

    let mut attempts = FuturesUnordered::new();
    let winner = Arc::new(AtomicBool::new(false));
    let app_handle = app_handle.cloned();

    for (url_idx, url) in urls.iter().enumerate() {
        let url = url.clone();
        let dest_path = dest_path.to_path_buf();
        let emit_name = emit_name.to_string();
        let app_handle = app_handle.clone();
        let winner = Arc::clone(&winner);
        let control_rx = control_tx.subscribe();
        let temp_path = candidate_temp_path(dest_path.as_path(), url_idx);

        attempts.push(async move {
            perform_download(
                &url,
                &dest_path,
                &temp_path,
                &emit_name,
                app_handle.as_ref(),
                control_rx,
                winner,
            )
            .await
        });
    }

    let mut last_error = String::from("Unknown error");
    while let Some(result) = attempts.next().await {
        match result {
            Ok(_) => {
                control_tx.send_replace(DownloadControl::Cancel);
                return Ok(());
            }
            Err(e) => {
                if e != "Cancelled" {
                    log_warn!("Parallel download attempt for {} failed: {}", emit_name, e);
                    last_error = e;
                } else if last_error == "Unknown error" {
                    last_error = e;
                }
            }
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
    temp_path: &Path,
    emit_name: &str,
    app_handle: Option<&tauri::AppHandle>,
    mut control_rx: watch::Receiver<DownloadControl>,
    winner: Arc<AtomicBool>,
) -> Result<(), String> {
    let mut last_error = String::from("Unknown error");
    let mut temp_guard = TempFileGuard::new(temp_path.to_path_buf());
    // Bytes already on disk from a previous session, pause or process restart.
    let mut offset: u64 = fs::metadata(temp_path).await.map(|m| m.len()).unwrap_or(0);
    let mut known_total: Option<u64> = None;
    let mut retryable = true;

    for attempt in 1..=MAX_RETRIES {
        if attempt > 1 {
            let delay = backoff_delay(attempt.min(8));
            log_warn!(
                "Retrying download for {} (attempt {}/{}, waiting {:.1}s) ...",
                emit_name,
                attempt,
                MAX_RETRIES,
                delay.as_secs_f64()
            );

            let deadline = Instant::now() + delay;
            loop {
                let remaining = deadline.saturating_duration_since(Instant::now());
                if remaining.is_zero() {
                    break;
                }
                emit_progress(
                    app_handle,
                    emit_name,
                    offset,
                    known_total,
                    percentage_of(offset, known_total),
                    0,
                    0,
                    "waiting",
                );
                tokio::select! {
                    _ = control_rx.changed() => {
                        let control = *control_rx.borrow_and_update();
                        match control {
                            DownloadControl::Cancel => {
                                temp_guard.discard();
                                return Err("Cancelled".to_string());
                            }
                            DownloadControl::Pause => {
                                emit_progress(app_handle, emit_name, offset, known_total, percentage_of(offset, known_total), 0, 0, "paused");
                                if let Err(e) = wait_for_resume(&mut control_rx).await {
                                    temp_guard.discard();
                                    return Err(e);
                                }
                            }
                            DownloadControl::Run => {}
                        }
                    }
                    _ = tokio::time::sleep(remaining.min(WAIT_HEARTBEAT)) => {}
                }
            }
        }

        // One session = one HTTP response. Pausing ends the session but keeps
        // the task (and the .part) alive; resuming starts a new Range request.
        'session: loop {
            let control = *control_rx.borrow_and_update();
            match control {
                DownloadControl::Cancel => {
                    temp_guard.discard();
                    return Err("Cancelled".to_string());
                }
                DownloadControl::Pause => {
                    if let Err(e) = wait_for_resume(&mut control_rx).await {
                        temp_guard.discard();
                        return Err(e);
                    }
                }
                DownloadControl::Run => {}
            }

            let mut request = HTTP_CLIENT.get(url);
            if offset > 0 {
                request = request.header(reqwest::header::RANGE, format!("bytes={offset}-"));
                let validator = load_validator(&temp_guard.meta_path).await;
                if let Some(value) = validator.if_range_value() {
                    request = request.header(reqwest::header::IF_RANGE, value);
                }
            }

            let response = tokio::select! {
                _ = control_rx.changed() => {
                    let control = *control_rx.borrow_and_update();
                    match control {
                        DownloadControl::Cancel => {
                            temp_guard.discard();
                            return Err("Cancelled".to_string());
                        }
                        DownloadControl::Pause => {
                            emit_progress(app_handle, emit_name, offset, known_total, percentage_of(offset, known_total), 0, 0, "paused");
                            if let Err(e) = wait_for_resume(&mut control_rx).await {
                                temp_guard.discard();
                                return Err(e);
                            }
                            continue 'session;
                        }
                        DownloadControl::Run => continue 'session,
                    }
                }
                res = request.send() => match res {
                    Ok(response) => response,
                    Err(e) => {
                        log_error!("Network request failed for {}: {}", emit_name, e);
                        last_error = format!("Network request failed: {e}");
                        retryable = true;
                        break 'session;
                    }
                },
            };

            let status = response.status();
            if status == reqwest::StatusCode::RANGE_NOT_SATISFIABLE {
                log_warn!(
                    "Range not satisfiable for {} (offset {}), restarting from scratch",
                    emit_name,
                    offset
                );
                offset = 0;
                continue 'session;
            }
            if !status.is_success() {
                let error_msg = format!(
                    "HTTP {} ({}) downloading {}",
                    status.as_u16(),
                    status.canonical_reason().unwrap_or("Unknown"),
                    emit_name,
                );
                log_error!("{}", error_msg);
                last_error = error_msg;
                retryable = status.is_server_error()
                    || status == reqwest::StatusCode::TOO_MANY_REQUESTS
                    || status == reqwest::StatusCode::REQUEST_TIMEOUT;
                break 'session;
            }

            let resuming = status == reqwest::StatusCode::PARTIAL_CONTENT && offset > 0;
            if !resuming {
                if offset > 0 {
                    log_warn!(
                        "Server ignored Range for {}, restarting from scratch",
                        emit_name
                    );
                }
                offset = 0;
            }

            let validator = ResumeValidator {
                etag: response
                    .headers()
                    .get(reqwest::header::ETAG)
                    .and_then(|v| v.to_str().ok())
                    .map(str::to_string),
                last_modified: response
                    .headers()
                    .get(reqwest::header::LAST_MODIFIED)
                    .and_then(|v| v.to_str().ok())
                    .map(str::to_string),
            };
            if let Ok(raw) = serde_json::to_string(&validator) {
                let _ = fs::write(&temp_guard.meta_path, raw).await;
            }

            let remaining = response.content_length();
            let total_size = remaining.map(|r| r + offset);
            known_total = total_size.or(known_total);

            if let Some(remaining_bytes) = remaining {
                log_info!(
                    "Downloading {} ({}{})",
                    emit_name,
                    human_size(remaining_bytes),
                    if resuming {
                        format!(", resuming from {}", human_size(offset))
                    } else {
                        String::new()
                    }
                );

                // Disk space check
                if let Some(parent) = dest_path.parent() {
                    let disks = sysinfo::Disks::new_with_refreshed_list();
                    if let Some(disk) = disks
                        .list()
                        .iter()
                        .find(|d| parent.starts_with(d.mount_point()))
                    {
                        if disk.available_space() < (remaining_bytes as f64 * 1.1) as u64 {
                            let err = format!(
                                "Insufficient disk space on {}. Required: {}, Available: {}",
                                disk.mount_point().display(),
                                human_size((remaining_bytes as f64 * 1.1) as u64),
                                human_size(disk.available_space())
                            );
                            log_error!("{}", err);
                            last_error = err;
                            retryable = false;
                            break 'session;
                        }
                    }
                }
            }

            let temp_file = if resuming {
                match fs::OpenOptions::new().append(true).open(temp_path).await {
                    Ok(file) => file,
                    Err(e) => {
                        log_warn!(
                            "Failed to reopen {} for resume ({}), restarting from scratch",
                            temp_path.display(),
                            e
                        );
                        offset = 0;
                        continue 'session;
                    }
                }
            } else {
                fs::File::create(temp_path).await.map_err(|e| {
                    log_error!("Failed to create temp file {}: {}", temp_path.display(), e);
                    format!("Temp file creation error: {e}")
                })?
            };
            let mut dest = BufWriter::new(temp_file);
            let mut downloaded: u64 = offset;
            let mut last_emitted_pct: u8 = 0;
            let mut last_emit_time = Instant::now();
            let download_start = Instant::now();
            let mut speed_window_bytes: u64 = 0;
            let mut speed_window_start = Instant::now();
            let mut current_speed_bps: f64 = 0.0;
            let mut first_chunk = true;
            let mut stream = response.bytes_stream();

            if resuming {
                emit_progress(
                    app_handle,
                    emit_name,
                    downloaded,
                    total_size,
                    percentage_of(downloaded, total_size),
                    0,
                    0,
                    "downloading",
                );
            }

            loop {
                let chunk = tokio::select! {
                    _ = control_rx.changed() => {
                        let control = *control_rx.borrow_and_update();
                        match control {
                            DownloadControl::Cancel => {
                                log_warn!("Download for {} was cancelled by user", emit_name);
                                temp_guard.discard();
                                return Err("Cancelled".to_string());
                            }
                            DownloadControl::Pause => {
                                if let Err(e) = dest.flush().await {
                                    log_warn!("Flush on pause failed for {}: {}", emit_name, e);
                                }
                                drop(dest);
                                log_info!(
                                    "Download for {} paused at {} ({:.1}%)",
                                    emit_name,
                                    human_size(downloaded),
                                    percentage_of(downloaded, total_size)
                                );
                                emit_progress(
                                    app_handle,
                                    emit_name,
                                    downloaded,
                                    total_size,
                                    percentage_of(downloaded, total_size),
                                    0,
                                    download_start.elapsed().as_millis() as u64,
                                    "paused",
                                );
                                if let Err(e) = wait_for_resume(&mut control_rx).await {
                                    temp_guard.discard();
                                    return Err(e);
                                }
                                offset = downloaded;
                                continue 'session;
                            }
                            DownloadControl::Run => continue,
                        }
                    }
                    res = stream.next() => res,
                };

                let Some(chunk) = chunk else { break };

                let chunk = match chunk {
                    Ok(chunk) => chunk,
                    Err(e) => {
                        // Keep what we have and retry with a Range request
                        // instead of starting over.
                        log_warn!(
                            "Stream read error for {} at {}: {} — will resume",
                            emit_name,
                            human_size(downloaded),
                            e
                        );
                        last_error = format!("Network stream error: {e}");
                        retryable = true;
                        if let Err(e) = dest.flush().await {
                            log_warn!("Flush after stream error failed for {}: {}", emit_name, e);
                        }
                        drop(dest);
                        offset = downloaded;
                        break 'session;
                    }
                };

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

                let percentage = percentage_of(downloaded, total_size);

                let now = Instant::now();
                let should_emit = percentage != last_emitted_pct
                    || now.duration_since(last_emit_time) >= PROGRESS_EMIT_INTERVAL;

                if should_emit {
                    last_emitted_pct = percentage;
                    last_emit_time = now;
                    emit_progress(
                        app_handle,
                        emit_name,
                        downloaded,
                        total_size,
                        percentage,
                        current_speed_bps as u64,
                        download_start.elapsed().as_millis() as u64,
                        "downloading",
                    );
                }
            }

            dest.flush().await.map_err(|e| {
                log_error!("Flush error for {}: {}", temp_path.display(), e);
                format!("File flush error: {e}")
            })?;

            drop(dest);

            if winner
                .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
                .is_err()
            {
                temp_guard.discard();
                return Err("Another download finished first".to_string());
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

            fs::rename(temp_path, dest_path).await.map_err(|e| {
                log_error!(
                    "Failed to finalize {} ({} -> {}): {}",
                    emit_name,
                    temp_path.display(),
                    dest_path.display(),
                    e
                );
                format!("Failed to finalize downloaded file: {e}")
            })?;

            let _ = fs::remove_file(&temp_guard.meta_path).await;

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

            return Ok(());
        }

        if !retryable {
            return Err(last_error);
        }
    }

    Err(last_error)
}

fn percentage_of(downloaded: u64, total: Option<u64>) -> u8 {
    total.map_or_else(
        || std::cmp::min(99, (downloaded / 1024 / 1024) as u8),
        |total| ((downloaded as f64 / total as f64) * 100.0).min(99.0) as u8,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn control_signals_for_unknown_download() {
        assert!(!pause_download("no-such-download"));
        assert!(!resume_download("no-such-download"));
        assert!(!cancel_download("no-such-download"));
    }

    #[test]
    fn pause_resume_cancel_signals() {
        let (tx, mut rx) = watch::channel(DownloadControl::Run);
        CONTROL_CHANNELS
            .lock()
            .unwrap()
            .insert("test-signals".to_string(), tx);

        assert!(pause_download("test-signals"));
        assert_eq!(*rx.borrow_and_update(), DownloadControl::Pause);
        assert!(resume_download("test-signals"));
        assert_eq!(*rx.borrow_and_update(), DownloadControl::Run);
        assert!(cancel_download("test-signals"));
        assert_eq!(*rx.borrow_and_update(), DownloadControl::Cancel);
        assert!(!pause_download("test-signals"));
    }

    #[test]
    fn percentage_saturates_at_99() {
        assert_eq!(percentage_of(0, Some(100)), 0);
        assert_eq!(percentage_of(50, Some(100)), 50);
        assert_eq!(percentage_of(100, Some(100)), 99);
        assert_eq!(percentage_of(200, Some(100)), 99);
    }

    #[test]
    fn wait_for_resume_returns_on_run_and_cancel() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let (tx, mut rx) = watch::channel(DownloadControl::Pause);
            let tx2 = tx.clone();
            tokio::spawn(async move {
                tokio::time::sleep(Duration::from_millis(20)).await;
                tx2.send_replace(DownloadControl::Run);
            });
            assert!(wait_for_resume(&mut rx).await.is_ok());

            let (tx, mut rx) = watch::channel(DownloadControl::Pause);
            let tx2 = tx.clone();
            tokio::spawn(async move {
                tokio::time::sleep(Duration::from_millis(20)).await;
                tx2.send_replace(DownloadControl::Cancel);
            });
            assert_eq!(
                wait_for_resume(&mut rx).await.unwrap_err(),
                "Cancelled".to_string()
            );
        });
    }

    #[derive(Default, Clone)]
    struct ServerBehavior {
        etag: Option<String>,
        drop_first_after: Option<usize>,
    }

    async fn spawn_range_server(
        payload: Vec<u8>,
        chunk_delay: Duration,
        behavior: ServerBehavior,
        requests: Arc<Mutex<Vec<usize>>>,
    ) -> (String, tokio::task::JoinHandle<()>) {
        use tokio::io::AsyncReadExt;

        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let connections = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let handle = tokio::spawn(async move {
            loop {
                let Ok((mut socket, _)) = listener.accept().await else {
                    break;
                };
                let payload = payload.clone();
                let behavior = behavior.clone();
                let requests = Arc::clone(&requests);
                let connections = Arc::clone(&connections);
                tokio::spawn(async move {
                    let mut buf = Vec::new();
                    let mut byte = [0u8; 1];
                    loop {
                        match socket.read(&mut byte).await {
                            Ok(0) | Err(_) => return,
                            Ok(_) => {
                                buf.push(byte[0]);
                                if buf.ends_with(b"\r\n\r\n") {
                                    break;
                                }
                            }
                        }
                    }
                    let request = String::from_utf8_lossy(&buf);
                    let mut start = 0usize;
                    let mut if_range: Option<String> = None;
                    for line in request.lines() {
                        let lower = line.to_ascii_lowercase();
                        if let Some(value) = lower.strip_prefix("range: bytes=") {
                            start = value.trim_end_matches('-').parse().unwrap_or(0);
                        }
                        if let Some(value) = lower.strip_prefix("if-range: ") {
                            if_range = Some(value.trim().to_string());
                        }
                    }
                    requests.lock().unwrap().push(start);
                    let conn = connections.fetch_add(1, Ordering::SeqCst);
                    let total = payload.len();
                    if let Some(etag) = &behavior.etag {
                        if if_range.as_deref() != Some(etag.as_str()) {
                            start = 0;
                        }
                    }
                    if start >= total {
                        let _ = socket
                            .write_all(
                                b"HTTP/1.1 416 Range Not Satisfiable\r\nContent-Length: 0\r\n\r\n",
                            )
                            .await;
                        return;
                    }
                    let etag_header = behavior
                        .etag
                        .as_ref()
                        .map(|e| format!("ETag: {e}\r\n"))
                        .unwrap_or_default();
                    if conn == 0 {
                        if let Some(limit) = behavior.drop_first_after {
                            let head = format!(
                                "HTTP/1.1 200 OK\r\nContent-Length: {total}\r\n{etag_header}\r\n"
                            );
                            if socket.write_all(head.as_bytes()).await.is_err() {
                                return;
                            }
                            let end = limit.min(total);
                            let _ = socket.write_all(&payload[..end]).await;
                            return;
                        }
                    }
                    let head = if start > 0 {
                        format!(
                            "HTTP/1.1 206 Partial Content\r\nContent-Length: {}\r\nContent-Range: bytes {}-{}/{}\r\n{etag_header}\r\n",
                            total - start,
                            start,
                            total - 1,
                            total
                        )
                    } else {
                        format!("HTTP/1.1 200 OK\r\nContent-Length: {total}\r\n{etag_header}\r\n")
                    };
                    if socket.write_all(head.as_bytes()).await.is_err() {
                        return;
                    }
                    let mut offset = start;
                    while offset < total {
                        let end = (offset + 64 * 1024).min(total);
                        if socket.write_all(&payload[offset..end]).await.is_err() {
                            return;
                        }
                        offset = end;
                        tokio::time::sleep(chunk_delay).await;
                    }
                });
            }
        });
        (format!("http://{addr}/file.bin"), handle)
    }

    #[test]
    fn pause_resume_roundtrip_over_http() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let payload: Vec<u8> = (0..2 * 1024 * 1024).map(|i| (i % 251) as u8).collect();
            let (url, server) = spawn_range_server(
                payload.clone(),
                Duration::from_millis(30),
                ServerBehavior::default(),
                Arc::new(Mutex::new(Vec::new())),
            )
            .await;

            let dir = std::env::temp_dir().join(format!("cl-dl-test-{}", std::process::id()));
            tokio::fs::create_dir_all(&dir).await.unwrap();
            let dest = dir.join("file.bin");
            let part = candidate_temp_path(&dest, 0);
            let name = "e2e-pause-resume";

            let dest_for_task = dest.clone();
            let download =
                tokio::spawn(
                    async move { download_file(&[url], &dest_for_task, name, None).await },
                );

            let snapshot_of = |name: &str| {
                active_download_states()
                    .into_iter()
                    .find(|s| s.file == name)
            };
            let deadline = Instant::now() + Duration::from_secs(10);
            let frozen = loop {
                if let Some(s) = snapshot_of(name) {
                    if s.downloaded >= 256 * 1024 {
                        break s.downloaded;
                    }
                }
                assert!(Instant::now() < deadline, "download made no progress");
                tokio::time::sleep(Duration::from_millis(20)).await;
            };

            assert!(pause_download(name));
            let deadline = Instant::now() + Duration::from_secs(5);
            let paused_at = loop {
                if let Some(s) = snapshot_of(name) {
                    if s.paused {
                        break s.downloaded;
                    }
                }
                assert!(Instant::now() < deadline, "pause was not applied");
                tokio::time::sleep(Duration::from_millis(20)).await;
            };
            assert!(paused_at >= frozen);
            assert!(tokio::fs::try_exists(&part).await.unwrap());

            tokio::time::sleep(Duration::from_millis(400)).await;
            let still = snapshot_of(name).expect("snapshot while paused");
            assert!(still.paused);
            assert_eq!(
                still.downloaded, paused_at,
                "bytes kept arriving while paused"
            );

            assert!(resume_download(name));
            download.await.unwrap().unwrap();

            let got = tokio::fs::read(&dest).await.unwrap();
            assert_eq!(got, payload, "resumed file differs from original");
            assert!(!tokio::fs::try_exists(&part).await.unwrap());
            assert!(snapshot_of(name).is_none());

            server.abort();
            let _ = tokio::fs::remove_dir_all(&dir).await;
        });
    }

    #[test]
    fn resume_from_existing_part_over_http() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let payload: Vec<u8> = (0..1024 * 1024).map(|i| (i % 251) as u8).collect();
            let requests = Arc::new(Mutex::new(Vec::new()));
            let (url, server) = spawn_range_server(
                payload.clone(),
                Duration::from_millis(5),
                ServerBehavior {
                    etag: Some("\"v1\"".to_string()),
                    drop_first_after: None,
                },
                Arc::clone(&requests),
            )
            .await;

            let dir = std::env::temp_dir().join(format!("cl-dl-restart-{}", std::process::id()));
            tokio::fs::create_dir_all(&dir).await.unwrap();
            let dest = dir.join("file.bin");
            let part = candidate_temp_path(&dest, 0);
            // Simulate a previous process: half the file on disk plus validator.
            tokio::fs::write(&part, &payload[..512 * 1024])
                .await
                .unwrap();
            let mut meta = part.clone().into_os_string();
            meta.push(".meta");
            let meta = std::path::PathBuf::from(meta);
            tokio::fs::write(&meta, r#"{"etag":"\"v1\"","last_modified":null}"#)
                .await
                .unwrap();

            download_file(&[url], &dest, "e2e-restart", None)
                .await
                .unwrap();

            let got = tokio::fs::read(&dest).await.unwrap();
            assert_eq!(got, payload, "resumed file differs from original");
            let seen = requests.lock().unwrap().clone();
            assert_eq!(
                seen.first().copied(),
                Some(512 * 1024),
                "first request must Range-resume from disk: {seen:?}"
            );
            assert!(!tokio::fs::try_exists(&part).await.unwrap());
            assert!(!tokio::fs::try_exists(&meta).await.unwrap());

            server.abort();
            let _ = tokio::fs::remove_dir_all(&dir).await;
        });
    }

    #[test]
    fn changed_remote_restarts_from_scratch() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let payload: Vec<u8> = (0..1024 * 1024).map(|i| (i % 251) as u8).collect();
            let requests = Arc::new(Mutex::new(Vec::new()));
            let (url, server) = spawn_range_server(
                payload.clone(),
                Duration::from_millis(5),
                ServerBehavior {
                    etag: Some("\"v2\"".to_string()),
                    drop_first_after: None,
                },
                Arc::clone(&requests),
            )
            .await;

            let dir = std::env::temp_dir().join(format!("cl-dl-changed-{}", std::process::id()));
            tokio::fs::create_dir_all(&dir).await.unwrap();
            let dest = dir.join("file.bin");
            let part = candidate_temp_path(&dest, 0);
            // Stale partial from an older remote version must not be appended to.
            tokio::fs::write(&part, vec![0xAAu8; 300 * 1024])
                .await
                .unwrap();
            let mut meta = part.clone().into_os_string();
            meta.push(".meta");
            tokio::fs::write(
                std::path::PathBuf::from(meta),
                r#"{"etag":"\"v1\"","last_modified":null}"#,
            )
            .await
            .unwrap();

            download_file(&[url], &dest, "e2e-changed", None)
                .await
                .unwrap();

            let got = tokio::fs::read(&dest).await.unwrap();
            assert_eq!(got, payload, "stale partial corrupted the download");
            let seen = requests.lock().unwrap().clone();
            assert_eq!(
                seen,
                vec![300 * 1024],
                "expected one full re-download: {seen:?}"
            );

            server.abort();
            let _ = tokio::fs::remove_dir_all(&dir).await;
        });
    }

    #[test]
    fn outage_auto_resumes_over_http() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let payload: Vec<u8> = (0..2 * 1024 * 1024).map(|i| (i % 251) as u8).collect();
            let requests = Arc::new(Mutex::new(Vec::new()));
            let (url, server) = spawn_range_server(
                payload.clone(),
                Duration::from_millis(20),
                ServerBehavior {
                    etag: None,
                    drop_first_after: Some(256 * 1024),
                },
                Arc::clone(&requests),
            )
            .await;

            let dir = std::env::temp_dir().join(format!("cl-dl-outage-{}", std::process::id()));
            tokio::fs::create_dir_all(&dir).await.unwrap();
            let dest = dir.join("file.bin");
            let name = "e2e-outage";

            let dest_for_task = dest.clone();
            let download =
                tokio::spawn(
                    async move { download_file(&[url], &dest_for_task, name, None).await },
                );

            let snapshot_of = |name: &str| {
                active_download_states()
                    .into_iter()
                    .find(|s| s.file == name)
            };
            let deadline = Instant::now() + Duration::from_secs(20);
            let saw_waiting = loop {
                if let Some(s) = snapshot_of(name) {
                    if s.waiting {
                        break true;
                    }
                }
                assert!(Instant::now() < deadline, "waiting state never emitted");
                tokio::time::sleep(Duration::from_millis(20)).await;
            };
            assert!(saw_waiting);

            download.await.unwrap().unwrap();

            let got = tokio::fs::read(&dest).await.unwrap();
            assert_eq!(got, payload, "auto-resumed file differs from original");
            let seen = requests.lock().unwrap().clone();
            assert!(seen.len() >= 2, "expected a retry connection: {seen:?}");
            assert!(seen[1] > 0, "retry must resume with Range: {seen:?}");

            server.abort();
            let _ = tokio::fs::remove_dir_all(&dir).await;
        });
    }
}
