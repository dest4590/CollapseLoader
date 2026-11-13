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
use tokio::io::{AsyncSeekExt, AsyncWriteExt, BufWriter};
use tokio::sync::watch;

const MAX_RETRIES: u32 = 40;
const BASE_RETRY_DELAY: Duration = Duration::from_secs(2);
const MAX_RETRY_DELAY: Duration = Duration::from_secs(30);
const PROGRESS_EMIT_INTERVAL: Duration = Duration::from_millis(200);
const WAIT_HEARTBEAT: Duration = Duration::from_secs(1);

// Steam-style segmented downloading: one file is split into fixed-size chunks
// fetched over several connections at once. Small files and servers that
// ignore Range fall back to a single sequential stream.
const SEGMENT_SIZE: u64 = 4 * 1024 * 1024;
const PARALLEL_CONNECTIONS: usize = 8;
const MIN_SEGMENTED_SIZE: u64 = 8 * 1024 * 1024;
const MANIFEST_FLUSH_INTERVAL: Duration = Duration::from_secs(1);

static HTTP_CLIENT: LazyLock<Client> = LazyLock::new(|| {
    super::create_client_builder()
        .timeout(Duration::from_secs(600))
        // HTTP/2 would multiplex every worker onto a single TCP connection,
        // so a per-connection server cap would throttle the whole download.
        .http1_only()
        .pool_max_idle_per_host(PARALLEL_CONNECTIONS)
        .build()
        .unwrap_or_else(|e| {
            log_error!("Failed to build download client: {e}");
            super::create_client(Duration::from_secs(600))
        })
});

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
        let meta_path = meta_path_for(&path);
        Self {
            path,
            meta_path,
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

fn meta_path_for(path: &Path) -> std::path::PathBuf {
    let mut meta = path.to_path_buf().into_os_string();
    meta.push(".meta");
    std::path::PathBuf::from(meta)
}

/// Persisted next to the .part: ETag/Last-Modified for If-Range validation
/// plus a per-segment progress grid so segmented downloads survive pauses,
/// outages and process restarts. Legacy sidecars (validator only) deserialize
/// with an empty grid and are seeded from the .part size.
#[derive(Default, Clone, serde::Serialize, serde::Deserialize)]
struct DownloadManifest {
    etag: Option<String>,
    last_modified: Option<String>,
    #[serde(default)]
    total: u64,
    #[serde(default)]
    segment_size: u64,
    #[serde(default)]
    done: Vec<u64>,
}

impl DownloadManifest {
    fn if_range_value(&self) -> Option<&str> {
        self.etag.as_deref().or(self.last_modified.as_deref())
    }

    fn segment_len(&self, index: usize) -> u64 {
        let start = self.segment_size * index as u64;
        self.segment_size.min(self.total.saturating_sub(start))
    }

    fn sum(&self) -> u64 {
        self.done.iter().sum()
    }

    fn complete(&self) -> bool {
        self.total > 0 && self.sum() >= self.total
    }

    fn init_grid(&mut self, total: u64, segment_size: u64) {
        self.total = total;
        self.segment_size = segment_size;
        let count = total.div_ceil(segment_size) as usize;
        self.done = vec![0; count];
    }

    fn reset_grid(&mut self) {
        self.total = 0;
        self.segment_size = 0;
        self.done.clear();
    }

    /// Bytes valid from offset 0: full segments plus the head of the first
    /// incomplete one (workers write each segment front-to-back).
    fn contiguous_prefix(&self) -> u64 {
        if self.done.is_empty() || self.segment_size == 0 {
            return 0;
        }
        let mut prefix = 0u64;
        for (i, &written) in self.done.iter().enumerate() {
            let len = self.segment_len(i);
            prefix += written.min(len);
            if written < len {
                break;
            }
        }
        prefix
    }

    fn seed_from_prefix(&mut self, prefix: u64) {
        if self.total == 0 || self.done.is_empty() {
            return;
        }
        let mut remaining = prefix.min(self.total);
        for i in 0..self.done.len() {
            let len = self.segment_len(i);
            let take = remaining.min(len);
            self.done[i] = take;
            remaining -= take;
        }
    }

    fn refresh_validators(&mut self, response: &reqwest::Response) {
        if let Some(etag) = response
            .headers()
            .get(reqwest::header::ETAG)
            .and_then(|v| v.to_str().ok())
        {
            self.etag = Some(etag.to_string());
        }
        if let Some(lm) = response
            .headers()
            .get(reqwest::header::LAST_MODIFIED)
            .and_then(|v| v.to_str().ok())
        {
            self.last_modified = Some(lm.to_string());
        }
    }
}

async fn load_manifest(meta_path: &Path) -> DownloadManifest {
    fs::read_to_string(meta_path)
        .await
        .ok()
        .and_then(|raw| serde_json::from_str(&raw).ok())
        .unwrap_or_default()
}

async fn save_manifest(manifest: &DownloadManifest, meta_path: &Path) {
    if let Ok(raw) = serde_json::to_string(manifest) {
        let _ = fs::write(meta_path, raw).await;
    }
}

/// Parses "bytes start-end/total".
fn parse_content_range(value: &str) -> Option<(u64, u64, u64)> {
    let value = value.strip_prefix("bytes ")?;
    let (range, total) = value.split_once('/')?;
    let (start, end) = range.split_once('-')?;
    Some((start.parse().ok()?, end.parse().ok()?, total.parse().ok()?))
}

enum SessionOutcome {
    Done(u64),
    Paused(u64),
    Cancelled,
    Retry(String),
    Fatal(String),
    Fallback(String),
}

/// Loop decision produced by the outcome handler; the actual continue/break
/// happens at the call site so the `'session` label stays in scope.
enum SessionFlow {
    Continue,
    Break,
}

enum WorkerExit {
    Drained,
    Paused,
    Cancelled,
    Retry(String),
    Fatal(String),
    Fallback(String),
}

enum SendOutcome {
    Response(reqwest::Response),
    Paused,
    Cancelled,
    Error(String),
}

/// Sends a request while staying responsive to pause/cancel. A Run signal
/// during the send simply re-issues the request.
async fn send_with_control<F>(
    mut build: F,
    control_rx: &mut watch::Receiver<DownloadControl>,
) -> SendOutcome
where
    F: FnMut() -> reqwest::RequestBuilder,
{
    loop {
        let request = build();
        tokio::select! {
            _ = control_rx.changed() => {
                match *control_rx.borrow_and_update() {
                    DownloadControl::Cancel => return SendOutcome::Cancelled,
                    DownloadControl::Pause => return SendOutcome::Paused,
                    DownloadControl::Run => continue,
                }
            }
            res = request.send() => {
                return match res {
                    Ok(response) => SendOutcome::Response(response),
                    Err(e) => SendOutcome::Error(format!("Network request failed: {e}")),
                };
            }
        }
    }
}

fn is_retryable_status(status: reqwest::StatusCode) -> bool {
    status.is_server_error()
        || status == reqwest::StatusCode::TOO_MANY_REQUESTS
        || status == reqwest::StatusCode::REQUEST_TIMEOUT
}

#[allow(clippy::too_many_arguments)]
async fn finalize_download(
    temp_guard: &mut TempFileGuard,
    temp_path: &Path,
    dest_path: &Path,
    emit_name: &str,
    winner: &AtomicBool,
    downloaded: u64,
    elapsed: Duration,
) -> Result<(), String> {
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
        let control_tx = control_tx.clone();
        let temp_path = candidate_temp_path(dest_path.as_path(), url_idx);

        attempts.push(async move {
            perform_download(
                &url,
                &dest_path,
                &temp_path,
                &emit_name,
                app_handle.as_ref(),
                control_tx,
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

fn check_disk_space(dest_path: &Path, required: u64) -> Result<(), String> {
    let Some(parent) = dest_path.parent() else {
        return Ok(());
    };
    let disks = sysinfo::Disks::new_with_refreshed_list();
    if let Some(disk) = disks
        .list()
        .iter()
        .find(|d| parent.starts_with(d.mount_point()))
    {
        let needed = (required as f64 * 1.1) as u64;
        if disk.available_space() < needed {
            let err = format!(
                "Insufficient disk space on {}. Required: {}, Available: {}",
                disk.mount_point().display(),
                human_size(needed),
                human_size(disk.available_space())
            );
            log_error!("{}", err);
            return Err(err);
        }
    }
    Ok(())
}

async fn perform_download(
    url: &str,
    dest_path: &Path,
    temp_path: &Path,
    emit_name: &str,
    app_handle: Option<&tauri::AppHandle>,
    control_tx: watch::Sender<DownloadControl>,
    winner: Arc<AtomicBool>,
) -> Result<(), String> {
    let mut control_rx = control_tx.subscribe();
    let mut last_error = String::from("Unknown error");
    let mut temp_guard = TempFileGuard::new(temp_path.to_path_buf());
    let mut manifest = load_manifest(&temp_guard.meta_path).await;
    let mut known_total: Option<u64> = None;
    // Bytes already on disk from a previous session, pause or process restart.
    let mut progress_bytes: u64 = fs::metadata(temp_path).await.map(|m| m.len()).unwrap_or(0);
    let mut retryable = true;
    let mut segmented_disabled = false;
    let download_start = Instant::now();

    macro_rules! handle_outcome {
        ($outcome:expr) => {
            match $outcome {
                SessionOutcome::Done(bytes) => {
                    finalize_download(
                        &mut temp_guard,
                        temp_path,
                        dest_path,
                        emit_name,
                        &winner,
                        bytes,
                        download_start.elapsed(),
                    )
                    .await?;
                    return Ok(());
                }
                SessionOutcome::Paused(bytes) => {
                    progress_bytes = bytes;
                    log_info!(
                        "Download for {} paused at {} ({:.1}%)",
                        emit_name,
                        human_size(bytes),
                        percentage_of(bytes, known_total)
                    );
                    emit_progress(
                        app_handle,
                        emit_name,
                        bytes,
                        known_total,
                        percentage_of(bytes, known_total),
                        0,
                        download_start.elapsed().as_millis() as u64,
                        "paused",
                    );
                    if let Err(e) = wait_for_resume(&mut control_rx).await {
                        temp_guard.discard();
                        return Err(e);
                    }
                    SessionFlow::Continue
                }
                SessionOutcome::Cancelled => {
                    log_warn!("Download for {} was cancelled by user", emit_name);
                    temp_guard.discard();
                    return Err("Cancelled".to_string());
                }
                SessionOutcome::Retry(e) => {
                    if manifest.total > 0 {
                        progress_bytes = manifest.sum();
                    }
                    last_error = e;
                    retryable = true;
                    SessionFlow::Break
                }
                SessionOutcome::Fatal(e) => {
                    last_error = e;
                    retryable = false;
                    SessionFlow::Break
                }
                SessionOutcome::Fallback(e) => {
                    log_warn!("Segmented mode disabled for {}: {}", emit_name, e);
                    segmented_disabled = true;
                    SessionFlow::Continue
                }
            }
        };
    }

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
                    progress_bytes,
                    known_total,
                    percentage_of(progress_bytes, known_total),
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
                                emit_progress(app_handle, emit_name, progress_bytes, known_total, percentage_of(progress_bytes, known_total), 0, 0, "paused");
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

        // One session = one probe plus either a segmented fan-out or a single
        // sequential stream. Pausing ends the session but keeps the task (and
        // the .part + manifest) alive; resuming starts a new probe.
        'session: loop {
            let control = *control_rx.borrow_and_update();
            match control {
                DownloadControl::Cancel => {
                    temp_guard.discard();
                    return Err("Cancelled".to_string());
                }
                DownloadControl::Pause => {
                    emit_progress(
                        app_handle,
                        emit_name,
                        progress_bytes,
                        known_total,
                        percentage_of(progress_bytes, known_total),
                        0,
                        0,
                        "paused",
                    );
                    if let Err(e) = wait_for_resume(&mut control_rx).await {
                        temp_guard.discard();
                        return Err(e);
                    }
                }
                DownloadControl::Run => {}
            }

            let part_len = fs::metadata(temp_path).await.map(|m| m.len()).unwrap_or(0);
            let has_data = part_len > 0;

            // A 1-byte probe reveals the total size and Range support without
            // pulling the body. If-Range makes a changed remote file arrive as
            // a full 200 instead of being appended to stale bytes.
            let probe = send_with_control(
                || {
                    let mut request = HTTP_CLIENT
                        .get(url)
                        .header(reqwest::header::RANGE, "bytes=0-0")
                        // Never pool the probe connection: the server closes it
                        // right after the 1-byte body, and a stale pooled socket
                        // would fail the follow-up request.
                        .header(reqwest::header::CONNECTION, "close");
                    if has_data {
                        if let Some(value) = manifest.if_range_value() {
                            request = request.header(reqwest::header::IF_RANGE, value);
                        }
                    }
                    request
                },
                &mut control_rx,
            )
            .await;

            let response = match probe {
                SendOutcome::Response(response) => response,
                SendOutcome::Cancelled => {
                    temp_guard.discard();
                    return Err("Cancelled".to_string());
                }
                SendOutcome::Paused => {
                    emit_progress(
                        app_handle,
                        emit_name,
                        progress_bytes,
                        known_total,
                        percentage_of(progress_bytes, known_total),
                        0,
                        0,
                        "paused",
                    );
                    if let Err(e) = wait_for_resume(&mut control_rx).await {
                        temp_guard.discard();
                        return Err(e);
                    }
                    continue 'session;
                }
                SendOutcome::Error(e) => {
                    log_error!("Network request failed for {}: {}", emit_name, e);
                    last_error = e;
                    retryable = true;
                    break 'session;
                }
            };

            let status = response.status();
            if status == reqwest::StatusCode::RANGE_NOT_SATISFIABLE {
                // bytes=0-0 is unsatisfiable only for an empty resource.
                drop(response);
                if let Err(e) = fs::File::create(temp_path).await {
                    last_error = format!("Temp file creation error: {e}");
                    retryable = false;
                    break 'session;
                }
                finalize_download(
                    &mut temp_guard,
                    temp_path,
                    dest_path,
                    emit_name,
                    &winner,
                    0,
                    download_start.elapsed(),
                )
                .await?;
                return Ok(());
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
                retryable = is_retryable_status(status);
                break 'session;
            }

            manifest.refresh_validators(&response);

            if status == reqwest::StatusCode::OK {
                // Server ignored Range (or the validator mismatched): the
                // probe response is the whole file — consume it from zero.
                if part_len > 0 {
                    log_warn!(
                        "Server ignored Range for {}, restarting from scratch",
                        emit_name
                    );
                }
                manifest.reset_grid();
                let total = response.content_length();
                if let Some(total) = total {
                    manifest.init_grid(total, SEGMENT_SIZE);
                    if let Err(e) = check_disk_space(dest_path, total) {
                        last_error = e;
                        retryable = false;
                        break 'session;
                    }
                }
                known_total = total.or(known_total);
                progress_bytes = 0;
                save_manifest(&manifest, &temp_guard.meta_path).await;
                log_info!(
                    "Downloading {} ({})",
                    emit_name,
                    total
                        .map(human_size)
                        .unwrap_or_else(|| "unknown size".into())
                );
                let outcome = stream_sequential(
                    response,
                    temp_path,
                    &temp_guard.meta_path,
                    0,
                    total,
                    &mut manifest,
                    &mut control_rx,
                    app_handle,
                    emit_name,
                    download_start,
                )
                .await;
                match handle_outcome!(outcome) {
                    SessionFlow::Continue => continue 'session,
                    SessionFlow::Break => break 'session,
                }
            }

            // 206: the probe told us the real total.
            let probe_total = response
                .headers()
                .get(reqwest::header::CONTENT_RANGE)
                .and_then(|v| v.to_str().ok())
                .and_then(parse_content_range)
                .map(|(_, _, total)| total);
            drop(response);

            let had_grid = !manifest.done.is_empty() && manifest.total > 0;
            if let Some(total) = probe_total {
                known_total = Some(total);
                if !had_grid {
                    // First sight or a legacy sidecar: trust the contiguous
                    // bytes already on disk (they were written sequentially).
                    manifest.init_grid(total, SEGMENT_SIZE);
                    manifest.seed_from_prefix(part_len.min(total));
                } else if manifest.total != total || manifest.segment_size != SEGMENT_SIZE {
                    // Remote size changed — old bytes can no longer be trusted.
                    manifest.init_grid(total, SEGMENT_SIZE);
                } else if part_len < manifest.sum() {
                    // The .part lost data (truncated/removed) — trust the disk.
                    manifest.seed_from_prefix(part_len.min(total));
                }
            }
            save_manifest(&manifest, &temp_guard.meta_path).await;

            if probe_total.is_some_and(|total| total >= MIN_SEGMENTED_SIZE) && !segmented_disabled {
                let total = probe_total.unwrap();
                let remaining = total.saturating_sub(manifest.sum());
                if let Err(e) = check_disk_space(dest_path, remaining) {
                    last_error = e;
                    retryable = false;
                    break 'session;
                }
                log_info!(
                    "Downloading {} ({} in {} segments, up to {} connections)",
                    emit_name,
                    human_size(total),
                    manifest.done.len(),
                    PARALLEL_CONNECTIONS
                );
                let shared = Arc::new(Mutex::new(manifest.clone()));
                let outcome = run_segmented(
                    url,
                    temp_path,
                    &temp_guard.meta_path,
                    Arc::clone(&shared),
                    &control_tx,
                    app_handle,
                    emit_name,
                    download_start,
                )
                .await;
                manifest = shared.lock().unwrap().clone();
                match handle_outcome!(outcome) {
                    SessionFlow::Continue => continue 'session,
                    SessionFlow::Break => break 'session,
                }
            }

            // Sequential mode: small file, unknown total, or a server that
            // cannot serve closed ranges. Resume from the contiguous prefix.
            let prefix = if manifest.total > 0 {
                manifest.contiguous_prefix()
            } else {
                part_len
            };
            if let Some(total) = known_total {
                if let Err(e) = check_disk_space(dest_path, total.saturating_sub(prefix)) {
                    last_error = e;
                    retryable = false;
                    break 'session;
                }
            }

            let resume_from = prefix;
            let sent = send_with_control(
                || {
                    let mut request = HTTP_CLIENT.get(url);
                    if resume_from > 0 {
                        request =
                            request.header(reqwest::header::RANGE, format!("bytes={resume_from}-"));
                        if let Some(value) = manifest.if_range_value() {
                            request = request.header(reqwest::header::IF_RANGE, value);
                        }
                    }
                    request
                },
                &mut control_rx,
            )
            .await;

            let response = match sent {
                SendOutcome::Response(response) => response,
                SendOutcome::Cancelled => {
                    temp_guard.discard();
                    return Err("Cancelled".to_string());
                }
                SendOutcome::Paused => {
                    emit_progress(
                        app_handle,
                        emit_name,
                        progress_bytes,
                        known_total,
                        percentage_of(progress_bytes, known_total),
                        0,
                        0,
                        "paused",
                    );
                    if let Err(e) = wait_for_resume(&mut control_rx).await {
                        temp_guard.discard();
                        return Err(e);
                    }
                    continue 'session;
                }
                SendOutcome::Error(e) => {
                    log_error!("Network request failed for {}: {}", emit_name, e);
                    last_error = e;
                    retryable = true;
                    break 'session;
                }
            };

            let status = response.status();
            if status == reqwest::StatusCode::RANGE_NOT_SATISFIABLE {
                log_warn!(
                    "Range not satisfiable for {} (prefix {}), restarting from scratch",
                    emit_name,
                    prefix
                );
                drop(response);
                manifest.reset_grid();
                progress_bytes = 0;
                if let Err(e) = fs::File::create(temp_path).await {
                    last_error = format!("Temp file creation error: {e}");
                    retryable = false;
                    break 'session;
                }
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
                retryable = is_retryable_status(status);
                break 'session;
            }

            manifest.refresh_validators(&response);
            let (prefix, total) = if status == reqwest::StatusCode::PARTIAL_CONTENT {
                let total = response
                    .content_length()
                    .map(|r| r + prefix)
                    .or(known_total);
                (prefix, total)
            } else {
                if prefix > 0 {
                    log_warn!(
                        "Server ignored Range for {}, restarting from scratch",
                        emit_name
                    );
                }
                manifest.reset_grid();
                let total = response.content_length();
                if let Some(total) = total {
                    manifest.init_grid(total, SEGMENT_SIZE);
                }
                (0, total)
            };
            known_total = total.or(known_total);
            progress_bytes = prefix;
            save_manifest(&manifest, &temp_guard.meta_path).await;
            log_info!(
                "Downloading {} ({}{})",
                emit_name,
                total
                    .map(human_size)
                    .unwrap_or_else(|| "unknown size".into()),
                if prefix > 0 {
                    format!(", resuming from {}", human_size(prefix))
                } else {
                    String::new()
                }
            );
            let outcome = stream_sequential(
                response,
                temp_path,
                &temp_guard.meta_path,
                prefix,
                total,
                &mut manifest,
                &mut control_rx,
                app_handle,
                emit_name,
                download_start,
            )
            .await;
            match handle_outcome!(outcome) {
                SessionFlow::Continue => continue 'session,
                SessionFlow::Break => break 'session,
            }
        }

        if !retryable {
            return Err(last_error);
        }
    }

    Err(last_error)
}

struct SegmentJob {
    url: String,
    temp_path: std::path::PathBuf,
    manifest: Arc<Mutex<DownloadManifest>>,
    dirty: Arc<AtomicBool>,
    index: usize,
    if_range: Option<String>,
}

/// Fetches one segment with closed Range requests, writing at its absolute
/// offset. The shared grid only records fully written chunks, so dropping
/// this future at any point (pause, cancel, retry) stays safe.
async fn download_segment(
    job: SegmentJob,
    mut control_rx: watch::Receiver<DownloadControl>,
) -> WorkerExit {
    let (seg_start, seg_len, mut written) = {
        let m = job.manifest.lock().unwrap();
        (
            m.segment_size * job.index as u64,
            m.segment_len(job.index),
            m.done[job.index],
        )
    };
    if written >= seg_len {
        return WorkerExit::Drained;
    }

    let mut file = match fs::OpenOptions::new()
        .write(true)
        .open(&job.temp_path)
        .await
    {
        Ok(file) => file,
        Err(e) => return WorkerExit::Fatal(format!("Segment file open error: {e}")),
    };

    while written < seg_len {
        let control = *control_rx.borrow_and_update();
        match control {
            DownloadControl::Cancel => return WorkerExit::Cancelled,
            DownloadControl::Pause => {
                let _ = file.flush().await;
                return WorkerExit::Paused;
            }
            DownloadControl::Run => {}
        }

        let start = seg_start + written;
        let end = seg_start + seg_len - 1;
        let expected = seg_len - written;
        let url = job.url.clone();
        let if_range = job.if_range.clone();
        let sent = send_with_control(
            move || {
                let mut request = HTTP_CLIENT
                    .get(&url)
                    .header(reqwest::header::RANGE, format!("bytes={start}-{end}"));
                if let Some(value) = &if_range {
                    request = request.header(reqwest::header::IF_RANGE, value);
                }
                request
            },
            &mut control_rx,
        )
        .await;

        let response = match sent {
            SendOutcome::Response(response) => response,
            SendOutcome::Cancelled => return WorkerExit::Cancelled,
            SendOutcome::Paused => {
                let _ = file.flush().await;
                return WorkerExit::Paused;
            }
            SendOutcome::Error(e) => {
                let _ = file.flush().await;
                return WorkerExit::Retry(e);
            }
        };

        let status = response.status();
        if status == reqwest::StatusCode::OK {
            let _ = file.flush().await;
            return WorkerExit::Fallback("Server ignored segment Range request".to_string());
        }
        if status == reqwest::StatusCode::RANGE_NOT_SATISFIABLE {
            let _ = file.flush().await;
            return WorkerExit::Retry(
                "HTTP 416 for segment (remote file may have changed)".to_string(),
            );
        }
        if status != reqwest::StatusCode::PARTIAL_CONTENT {
            let _ = file.flush().await;
            let msg = format!(
                "HTTP {} ({}) downloading segment",
                status.as_u16(),
                status.canonical_reason().unwrap_or("Unknown")
            );
            return if is_retryable_status(status) {
                WorkerExit::Retry(msg)
            } else {
                WorkerExit::Fatal(msg)
            };
        }

        let range_ok = response
            .headers()
            .get(reqwest::header::CONTENT_RANGE)
            .and_then(|v| v.to_str().ok())
            .and_then(parse_content_range)
            .is_some_and(|(s, e, _)| s == start && e == end);
        if !range_ok || response.content_length() != Some(expected) {
            let _ = file.flush().await;
            return WorkerExit::Fallback(
                "Unexpected Content-Range in segment response".to_string(),
            );
        }

        if let Err(e) = file.seek(std::io::SeekFrom::Start(start)).await {
            return WorkerExit::Fatal(format!("Segment seek error: {e}"));
        }

        let mut stream = response.bytes_stream();
        let mut received = 0u64;
        loop {
            let chunk = tokio::select! {
                _ = control_rx.changed() => {
                    let control = *control_rx.borrow_and_update();
                    match control {
                        DownloadControl::Cancel => {
                            let _ = file.flush().await;
                            return WorkerExit::Cancelled;
                        }
                        DownloadControl::Pause => {
                            let _ = file.flush().await;
                            return WorkerExit::Paused;
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
                    let _ = file.flush().await;
                    return WorkerExit::Retry(format!("Network stream error: {e}"));
                }
            };
            let take = (chunk.len() as u64).min(expected - received) as usize;
            if let Err(e) = file.write_all(&chunk[..take]).await {
                return WorkerExit::Fatal(format!("Segment write error: {e}"));
            }
            received += take as u64;
            written += take as u64;
            {
                let mut m = job.manifest.lock().unwrap();
                m.done[job.index] = written;
            }
            job.dirty.store(true, Ordering::Relaxed);
            if written >= seg_len || take < chunk.len() {
                break;
            }
        }
        let _ = file.flush().await;
        if written < seg_len {
            return WorkerExit::Retry(
                "Segment stream ended before the range was complete".to_string(),
            );
        }
    }

    WorkerExit::Drained
}

/// Keeps up to PARALLEL_CONNECTIONS segment workers alive, emits aggregated
/// progress on a heartbeat and flushes the manifest grid periodically so a
/// hard crash loses at most ~1s of recorded progress.
#[allow(clippy::too_many_arguments)]
async fn run_segmented(
    url: &str,
    temp_path: &Path,
    meta_path: &Path,
    manifest: Arc<Mutex<DownloadManifest>>,
    control_tx: &watch::Sender<DownloadControl>,
    app_handle: Option<&tauri::AppHandle>,
    emit_name: &str,
    download_start: Instant,
) -> SessionOutcome {
    let (total, if_range) = {
        let m = manifest.lock().unwrap();
        (m.total, m.if_range_value().map(str::to_string))
    };

    // Preallocate so workers can write at arbitrary offsets.
    match fs::OpenOptions::new()
        .write(true)
        .create(true)
        .open(temp_path)
        .await
    {
        Ok(file) => {
            if let Err(e) = file.set_len(total).await {
                return SessionOutcome::Fatal(format!("Preallocation error: {e}"));
            }
        }
        Err(e) => return SessionOutcome::Fatal(format!("Temp file open error: {e}")),
    }

    let dirty = Arc::new(AtomicBool::new(false));
    let mut workers = FuturesUnordered::new();
    let mut next_index = 0usize;
    let mut heartbeat = tokio::time::interval(PROGRESS_EMIT_INTERVAL);
    let mut last_flush = Instant::now();
    let mut window_start = Instant::now();
    let mut window_base = manifest.lock().unwrap().sum();
    let mut speed_bps = 0f64;
    let mut outcome: Option<SessionOutcome> = None;

    loop {
        while workers.len() < PARALLEL_CONNECTIONS {
            let next = {
                let m = manifest.lock().unwrap();
                (next_index..m.done.len()).find(|&i| m.done[i] < m.segment_len(i))
            };
            let Some(index) = next else { break };
            next_index = index + 1;
            workers.push(download_segment(
                SegmentJob {
                    url: url.to_string(),
                    temp_path: temp_path.to_path_buf(),
                    manifest: Arc::clone(&manifest),
                    dirty: Arc::clone(&dirty),
                    index,
                    if_range: if_range.clone(),
                },
                control_tx.subscribe(),
            ));
        }
        if workers.is_empty() {
            break;
        }

        tokio::select! {
            exit = workers.next() => {
                match exit {
                    Some(WorkerExit::Drained) => {}
                    Some(WorkerExit::Paused) => {
                        outcome = Some(SessionOutcome::Paused(manifest.lock().unwrap().sum()));
                        break;
                    }
                    Some(WorkerExit::Cancelled) => {
                        outcome = Some(SessionOutcome::Cancelled);
                        break;
                    }
                    Some(WorkerExit::Retry(e)) => {
                        outcome = Some(SessionOutcome::Retry(e));
                        break;
                    }
                    Some(WorkerExit::Fatal(e)) => {
                        outcome = Some(SessionOutcome::Fatal(e));
                        break;
                    }
                    Some(WorkerExit::Fallback(e)) => {
                        outcome = Some(SessionOutcome::Fallback(e));
                        break;
                    }
                    None => break,
                }
            }
            _ = heartbeat.tick() => {
                let sum = manifest.lock().unwrap().sum();
                let elapsed = window_start.elapsed();
                if elapsed >= Duration::from_millis(500) {
                    speed_bps = sum.saturating_sub(window_base) as f64 / elapsed.as_secs_f64();
                    window_base = sum;
                    window_start = Instant::now();
                }
                emit_progress(
                    app_handle,
                    emit_name,
                    sum,
                    Some(total),
                    percentage_of(sum, Some(total)),
                    speed_bps as u64,
                    download_start.elapsed().as_millis() as u64,
                    "downloading",
                );
                if dirty.load(Ordering::Relaxed) && last_flush.elapsed() >= MANIFEST_FLUSH_INTERVAL {
                    let snapshot = manifest.lock().unwrap().clone();
                    save_manifest(&snapshot, meta_path).await;
                    dirty.store(false, Ordering::Relaxed);
                    last_flush = Instant::now();
                }
            }
        }
    }
    drop(workers);

    let snapshot = manifest.lock().unwrap().clone();
    save_manifest(&snapshot, meta_path).await;

    match outcome {
        Some(outcome) => outcome,
        None if snapshot.complete() => SessionOutcome::Done(snapshot.sum()),
        None => {
            SessionOutcome::Retry("Segment grid incomplete after all workers drained".to_string())
        }
    }
}

/// Single-stream fallback (small files, servers without Range support).
/// Writes append after the contiguous prefix; the grid is synced at every
/// session boundary so a later attempt can switch back to segmented mode.
#[allow(clippy::too_many_arguments)]
async fn stream_sequential(
    response: reqwest::Response,
    temp_path: &Path,
    meta_path: &Path,
    prefix: u64,
    total: Option<u64>,
    manifest: &mut DownloadManifest,
    control_rx: &mut watch::Receiver<DownloadControl>,
    app_handle: Option<&tauri::AppHandle>,
    emit_name: &str,
    download_start: Instant,
) -> SessionOutcome {
    let temp_file = if prefix > 0 {
        match fs::OpenOptions::new()
            .write(true)
            .create(true)
            .open(temp_path)
            .await
        {
            Ok(mut file) => {
                if let Err(e) = file.set_len(prefix).await {
                    return SessionOutcome::Retry(format!("Temp file truncate error: {e}"));
                }
                if let Err(e) = file.seek(std::io::SeekFrom::Start(prefix)).await {
                    return SessionOutcome::Retry(format!("Temp file seek error: {e}"));
                }
                file
            }
            Err(e) => return SessionOutcome::Retry(format!("Temp file open error: {e}")),
        }
    } else {
        match fs::File::create(temp_path).await {
            Ok(file) => file,
            Err(e) => return SessionOutcome::Fatal(format!("Temp file creation error: {e}")),
        }
    };
    let mut dest = BufWriter::new(temp_file);
    let mut downloaded = prefix;
    let mut last_emitted_pct: u8 = 0;
    let mut last_emit_time = Instant::now();
    let mut speed_window_bytes: u64 = 0;
    let mut speed_window_start = Instant::now();
    let mut current_speed_bps: f64 = 0.0;
    let mut first_chunk = true;
    let mut stream = response.bytes_stream();

    manifest.seed_from_prefix(prefix);
    save_manifest(manifest, meta_path).await;

    if prefix > 0 {
        emit_progress(
            app_handle,
            emit_name,
            downloaded,
            total,
            percentage_of(downloaded, total),
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
                        let _ = dest.flush().await;
                        return SessionOutcome::Cancelled;
                    }
                    DownloadControl::Pause => {
                        if let Err(e) = dest.flush().await {
                            log_warn!("Flush on pause failed for {}: {}", emit_name, e);
                        }
                        manifest.seed_from_prefix(downloaded);
                        save_manifest(manifest, meta_path).await;
                        return SessionOutcome::Paused(downloaded);
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
                log_warn!(
                    "Stream read error for {} at {}: {} — will resume",
                    emit_name,
                    human_size(downloaded),
                    e
                );
                let _ = dest.flush().await;
                manifest.seed_from_prefix(downloaded);
                save_manifest(manifest, meta_path).await;
                return SessionOutcome::Retry(format!("Network stream error: {e}"));
            }
        };

        if let Err(e) = dest.write_all(&chunk).await {
            log_error!("Write error for {}: {}", temp_path.display(), e);
            let _ = dest.flush().await;
            manifest.seed_from_prefix(downloaded);
            save_manifest(manifest, meta_path).await;
            return SessionOutcome::Fatal(format!("File write error: {e}"));
        }

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

        let percentage = percentage_of(downloaded, total);
        let now = Instant::now();
        if percentage != last_emitted_pct
            || now.duration_since(last_emit_time) >= PROGRESS_EMIT_INTERVAL
        {
            last_emitted_pct = percentage;
            last_emit_time = now;
            emit_progress(
                app_handle,
                emit_name,
                downloaded,
                total,
                percentage,
                current_speed_bps as u64,
                download_start.elapsed().as_millis() as u64,
                "downloading",
            );
        }
    }

    if let Err(e) = dest.flush().await {
        log_error!("Flush error for {}: {}", temp_path.display(), e);
        return SessionOutcome::Fatal(format!("File flush error: {e}"));
    }
    drop(dest);
    manifest.seed_from_prefix(downloaded);
    SessionOutcome::Done(downloaded)
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
        ignore_range: bool,
        ignore_closed_ranges: bool,
        max_concurrent: Option<Arc<std::sync::atomic::AtomicUsize>>,
    }

    struct ConcurrencyGuard(Arc<std::sync::atomic::AtomicUsize>);
    impl Drop for ConcurrencyGuard {
        fn drop(&mut self) {
            self.0.fetch_sub(1, Ordering::SeqCst);
        }
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
        let current = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let handle = tokio::spawn(async move {
            loop {
                let Ok((mut socket, _)) = listener.accept().await else {
                    break;
                };
                let payload = payload.clone();
                let behavior = behavior.clone();
                let requests = Arc::clone(&requests);
                let connections = Arc::clone(&connections);
                let current = Arc::clone(&current);
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
                    let total = payload.len();
                    let mut range: Option<(usize, Option<usize>)> = None;
                    let mut if_range: Option<String> = None;
                    for line in request.lines() {
                        let lower = line.to_ascii_lowercase();
                        if let Some(value) = lower.strip_prefix("range: bytes=") {
                            if let Some((s, e)) = value.trim().split_once('-') {
                                let start = s.parse().unwrap_or(0);
                                let end = if e.is_empty() { None } else { e.parse().ok() };
                                range = Some((start, end));
                            }
                        }
                        if let Some(value) = lower.strip_prefix("if-range: ") {
                            if_range = Some(value.trim().to_string());
                        }
                    }

                    let mut start = 0usize;
                    let mut end = total.saturating_sub(1);
                    let mut partial = false;
                    if let Some((s, e)) = range {
                        // The 1-byte probe (0-0) stays servable even when the
                        // "closed ranges" mode rejects multi-byte ranges.
                        let closed_multi = e.is_some_and(|end| end > s);
                        if !behavior.ignore_range
                            && !(behavior.ignore_closed_ranges && closed_multi)
                        {
                            start = s;
                            end = e
                                .unwrap_or(total.saturating_sub(1))
                                .min(total.saturating_sub(1));
                            partial = true;
                        }
                    }
                    requests.lock().unwrap().push(start);

                    if partial {
                        if let Some(etag) = &behavior.etag {
                            // If-Range forces a full 200 only when present and
                            // stale; without it the Range is served normally.
                            if if_range.as_deref().is_some_and(|v| v != etag.as_str()) {
                                start = 0;
                                end = total.saturating_sub(1);
                                partial = false;
                            }
                        }
                    }

                    let conn = connections.fetch_add(1, Ordering::SeqCst);
                    let active = current.fetch_add(1, Ordering::SeqCst) + 1;
                    if let Some(max) = &behavior.max_concurrent {
                        max.fetch_max(active, Ordering::SeqCst);
                    }
                    let _guard = ConcurrencyGuard(Arc::clone(&current));

                    if start >= total || start > end {
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
                            let limit_end = limit.min(total);
                            let _ = socket.write_all(&payload[..limit_end]).await;
                            return;
                        }
                    }

                    let head = if partial {
                        format!(
                            "HTTP/1.1 206 Partial Content\r\nContent-Length: {}\r\nContent-Range: bytes {}-{}/{}\r\n{etag_header}\r\n",
                            end - start + 1,
                            start,
                            end,
                            total
                        )
                    } else {
                        format!("HTTP/1.1 200 OK\r\nContent-Length: {total}\r\n{etag_header}\r\n")
                    };
                    if socket.write_all(head.as_bytes()).await.is_err() {
                        return;
                    }
                    let mut offset = start;
                    while offset <= end {
                        let chunk_end = (offset + 64 * 1024).min(end + 1);
                        if socket.write_all(&payload[offset..chunk_end]).await.is_err() {
                            return;
                        }
                        offset = chunk_end;
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
                    ..Default::default()
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
                seen,
                vec![0, 512 * 1024],
                "expected a probe then a Range-resume from disk: {seen:?}"
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
                    ..Default::default()
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
                vec![0],
                "expected one probe answered with a full re-download: {seen:?}"
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
                    drop_first_after: Some(256 * 1024),
                    ..Default::default()
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
            assert!(
                seen.contains(&(256 * 1024)),
                "retry must Range-resume from the bytes kept on disk: {seen:?}"
            );

            server.abort();
            let _ = tokio::fs::remove_dir_all(&dir).await;
        });
    }

    async fn wait_snapshot<F>(name: &str, timeout: Duration, pred: F) -> DownloadSnapshot
    where
        F: Fn(&DownloadSnapshot) -> bool,
    {
        let deadline = Instant::now() + timeout;
        loop {
            if let Some(s) = active_download_states()
                .into_iter()
                .find(|s| s.file == name)
            {
                if pred(&s) {
                    return s;
                }
            }
            assert!(Instant::now() < deadline, "condition never met for {name}");
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
    }

    #[test]
    fn segmented_parallel_download() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let payload: Vec<u8> = (0..24 * 1024 * 1024).map(|i| (i % 251) as u8).collect();
            let requests = Arc::new(Mutex::new(Vec::new()));
            let max_concurrent = Arc::new(std::sync::atomic::AtomicUsize::new(0));
            let (url, server) = spawn_range_server(
                payload.clone(),
                Duration::from_millis(1),
                ServerBehavior {
                    max_concurrent: Some(Arc::clone(&max_concurrent)),
                    ..Default::default()
                },
                Arc::clone(&requests),
            )
            .await;

            let dir = std::env::temp_dir().join(format!("cl-dl-seg-{}", std::process::id()));
            tokio::fs::create_dir_all(&dir).await.unwrap();
            let dest = dir.join("file.bin");

            download_file(&[url], &dest, "e2e-segmented", None)
                .await
                .unwrap();

            let got = tokio::fs::read(&dest).await.unwrap();
            assert_eq!(got, payload, "segmented file differs from original");
            let peak = max_concurrent.load(Ordering::SeqCst);
            assert!(
                peak >= 4,
                "expected several parallel segment connections, peak was {peak}"
            );
            let seen = requests.lock().unwrap().clone();
            assert!(
                seen.len() >= 7,
                "expected a probe plus six segment requests: {seen:?}"
            );

            server.abort();
            let _ = tokio::fs::remove_dir_all(&dir).await;
        });
    }

    #[test]
    fn segmented_pause_resume() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let payload: Vec<u8> = (0..12 * 1024 * 1024).map(|i| (i % 251) as u8).collect();
            let (url, server) = spawn_range_server(
                payload.clone(),
                Duration::from_millis(5),
                ServerBehavior::default(),
                Arc::new(Mutex::new(Vec::new())),
            )
            .await;

            let dir = std::env::temp_dir().join(format!("cl-dl-seg-pause-{}", std::process::id()));
            tokio::fs::create_dir_all(&dir).await.unwrap();
            let dest = dir.join("file.bin");
            let part = candidate_temp_path(&dest, 0);
            let meta = meta_path_for(&part);
            let name = "e2e-seg-pause";

            let dest_for_task = dest.clone();
            let download =
                tokio::spawn(
                    async move { download_file(&[url], &dest_for_task, name, None).await },
                );

            wait_snapshot(name, Duration::from_secs(10), |s| {
                s.downloaded >= 2 * 1024 * 1024
            })
            .await;
            assert!(pause_download(name));
            let paused = wait_snapshot(name, Duration::from_secs(5), |s| s.paused).await;
            assert!(tokio::fs::try_exists(&part).await.unwrap());
            assert!(tokio::fs::try_exists(&meta).await.unwrap());

            tokio::time::sleep(Duration::from_millis(300)).await;
            let still = active_download_states()
                .into_iter()
                .find(|s| s.file == name)
                .expect("snapshot while paused");
            assert!(still.paused);
            assert_eq!(
                still.downloaded, paused.downloaded,
                "bytes kept arriving while paused"
            );

            assert!(resume_download(name));
            download.await.unwrap().unwrap();

            let got = tokio::fs::read(&dest).await.unwrap();
            assert_eq!(got, payload, "resumed segmented file differs from original");
            assert!(!tokio::fs::try_exists(&part).await.unwrap());
            assert!(!tokio::fs::try_exists(&meta).await.unwrap());

            server.abort();
            let _ = tokio::fs::remove_dir_all(&dir).await;
        });
    }

    #[test]
    fn segmented_restart_from_manifest() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let payload: Vec<u8> = (0..12 * 1024 * 1024).map(|i| (i % 251) as u8).collect();
            let requests = Arc::new(Mutex::new(Vec::new()));
            let (url, server) = spawn_range_server(
                payload.clone(),
                Duration::from_millis(5),
                ServerBehavior {
                    etag: Some("\"v1\"".to_string()),
                    ..Default::default()
                },
                Arc::clone(&requests),
            )
            .await;

            let dir =
                std::env::temp_dir().join(format!("cl-dl-seg-restart-{}", std::process::id()));
            tokio::fs::create_dir_all(&dir).await.unwrap();
            let dest = dir.join("file.bin");
            let part = candidate_temp_path(&dest, 0);
            let meta = meta_path_for(&part);
            let name = "e2e-seg-restart";

            let dest_for_task = dest.clone();
            let url_for_task = url.clone();
            let download = tokio::spawn(async move {
                download_file(&[url_for_task], &dest_for_task, name, None).await
            });

            wait_snapshot(name, Duration::from_secs(10), |s| {
                s.downloaded >= 6 * 1024 * 1024
            })
            .await;
            assert!(pause_download(name));
            wait_snapshot(name, Duration::from_secs(5), |s| s.paused).await;
            // Let any in-flight request land so the request log is stable.
            tokio::time::sleep(Duration::from_millis(300)).await;

            let raw = tokio::fs::read_to_string(&meta).await.unwrap();
            let m: DownloadManifest = serde_json::from_str(&raw).unwrap();
            assert!(m.sum() > 0, "manifest must record segment progress");
            let mut expected: Vec<usize> = m
                .done
                .iter()
                .enumerate()
                .filter(|(i, &d)| d < m.segment_len(*i))
                .map(|(i, &d)| (m.segment_size * i as u64 + d) as usize)
                .collect();
            expected.sort_unstable();
            let before = requests.lock().unwrap().len();

            // Simulate the process dying while paused: .part + manifest survive.
            download.abort();
            let _ = download.await;
            assert!(tokio::fs::try_exists(&part).await.unwrap());
            assert!(tokio::fs::try_exists(&meta).await.unwrap());

            download_file(&[url], &dest, "e2e-seg-restart2", None)
                .await
                .unwrap();

            let got = tokio::fs::read(&dest).await.unwrap();
            assert_eq!(
                got, payload,
                "restarted segmented file differs from original"
            );
            let seen = requests.lock().unwrap().clone();
            let after = &seen[before..];
            assert_eq!(
                after.first().copied(),
                Some(0),
                "restart begins with a probe: {after:?}"
            );
            let mut actual: Vec<usize> = after[1..].to_vec();
            actual.sort_unstable();
            assert_eq!(
                actual, expected,
                "restart must re-fetch exactly the unfinished segment tails: {after:?}"
            );

            server.abort();
            let _ = tokio::fs::remove_dir_all(&dir).await;
        });
    }

    #[test]
    fn ignore_range_falls_back_to_sequential() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let payload: Vec<u8> = (0..12 * 1024 * 1024).map(|i| (i % 251) as u8).collect();
            let requests = Arc::new(Mutex::new(Vec::new()));
            let (url, server) = spawn_range_server(
                payload.clone(),
                Duration::from_millis(1),
                ServerBehavior {
                    ignore_range: true,
                    ..Default::default()
                },
                Arc::clone(&requests),
            )
            .await;

            let dir = std::env::temp_dir().join(format!("cl-dl-norange-{}", std::process::id()));
            tokio::fs::create_dir_all(&dir).await.unwrap();
            let dest = dir.join("file.bin");

            download_file(&[url], &dest, "e2e-norange", None)
                .await
                .unwrap();

            let got = tokio::fs::read(&dest).await.unwrap();
            assert_eq!(got, payload, "fallback file differs from original");
            let seen = requests.lock().unwrap().clone();
            assert!(
                seen.iter().all(|&s| s == 0),
                "a server ignoring Range must be streamed from zero: {seen:?}"
            );

            server.abort();
            let _ = tokio::fs::remove_dir_all(&dir).await;
        });
    }

    #[test]
    fn closed_range_rejection_falls_back() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let payload: Vec<u8> = (0..12 * 1024 * 1024).map(|i| (i % 251) as u8).collect();
            let requests = Arc::new(Mutex::new(Vec::new()));
            let (url, server) = spawn_range_server(
                payload.clone(),
                Duration::from_millis(1),
                ServerBehavior {
                    etag: Some("\"v1\"".to_string()),
                    ignore_closed_ranges: true,
                    ..Default::default()
                },
                Arc::clone(&requests),
            )
            .await;

            let dir = std::env::temp_dir().join(format!("cl-dl-closed-{}", std::process::id()));
            tokio::fs::create_dir_all(&dir).await.unwrap();
            let dest = dir.join("file.bin");

            download_file(&[url], &dest, "e2e-closed", None)
                .await
                .unwrap();

            let got = tokio::fs::read(&dest).await.unwrap();
            assert_eq!(got, payload, "closed-range fallback file differs");
            let seen = requests.lock().unwrap().clone();
            // probe (206) + at least one rejected segment + sequential re-fetch.
            assert!(
                seen.len() >= 3,
                "expected probe, rejected segment and sequential fallback: {seen:?}"
            );

            server.abort();
            let _ = tokio::fs::remove_dir_all(&dir).await;
        });
    }
}
