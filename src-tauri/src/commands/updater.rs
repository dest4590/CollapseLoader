use crate::{
    core::{
        storage::data::Data,
        utils::globals::{GITHUB_REPO_NAME, GITHUB_REPO_OWNER},
    },
    log_debug, log_error, log_info, log_warn,
};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::cmp::Ordering;

#[derive(Debug, Serialize, Deserialize)]
pub struct GitHubRelease {
    pub tag_name: String,
    pub name: String,
    pub body: String,
    pub html_url: String,
    pub assets: Vec<GitHubAsset>,
    pub published_at: String,
    pub prerelease: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GitHubAsset {
    pub name: String,
    pub browser_download_url: String,
    pub size: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChangelogEntry {
    pub version: String,
    pub changes: Vec<ChangeItem>,
    pub date: String,
    pub highlights: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Category {
    Feature,
    Improvement,
    Bugfix,
    Other,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChangeItem {
    pub category: Category,
    pub description_key: String,
    pub icon: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateInfo {
    pub available: bool,
    pub current_version: String,
    pub latest_version: String,
    pub release_notes: String,
    pub download_url: String,
    pub changelog: Vec<ChangelogEntry>,
    pub translations: Option<JsonValue>,
    pub release_date: String,
    pub is_critical: bool,
}

fn parse_version(version: &str) -> Result<(u32, u32, u32), String> {
    let version = version.trim_start_matches('v');
    let parts: Vec<&str> = version.split('.').collect();

    if parts.len() != 3 {
        return Err("Invalid version format".to_string());
    }

    let major = parts[0]
        .parse::<u32>()
        .map_err(|_| "Invalid major version")?;
    let minor = parts[1]
        .parse::<u32>()
        .map_err(|_| "Invalid minor version")?;
    let patch = parts[2]
        .parse::<u32>()
        .map_err(|_| "Invalid patch version")?;

    Ok((major, minor, patch))
}

fn compare_versions(v1: &str, v2: &str) -> Result<Ordering, String> {
    let (major1, minor1, patch1) = parse_version(v1)?;
    let (major2, minor2, patch2) = parse_version(v2)?;

    match major1.cmp(&major2) {
        Ordering::Equal => match minor1.cmp(&minor2) {
            Ordering::Equal => Ok(patch1.cmp(&patch2)),
            other => Ok(other),
        },
        other => Ok(other),
    }
}

#[tauri::command]
pub async fn check_for_updates() -> Result<UpdateInfo, String> {
    let current_version = env!("CARGO_PKG_VERSION");
    log_info!("Checking for updates. Current version: {}", current_version);

    let client = reqwest::Client::new();
    let url = if std::env::var("LOCAL_UPDATER_URL").unwrap_or_default() == "true" {
        log_debug!("Using local updater URL for development");
        "http://127.0.0.1:8000/repos/dest4590/CollapseLoader/releases/latest".to_string()
    } else {
        format!(
            "https://api.github.com/repos/{GITHUB_REPO_OWNER}/{GITHUB_REPO_NAME}/releases/latest"
        )
    };
    log_debug!("Fetching latest release from: {}", url);

    let response = client
        .get(&url)
        .header("User-Agent", "CollapseLoader-Updater")
        .header("Accept", "application/vnd.github+json")
        .header("X-GitHub-Api-Version", "2022-11-28")
        .send()
        .await
        .map_err(|e| {
            log_warn!("Failed to fetch releases from GitHub API: {}", e);
            format!("Failed to fetch releases: {e}")
        })?;

    if !response.status().is_success() {
        log_warn!(
            "GitHub API returned non-success status: {}",
            response.status()
        );
        return Err(format!("GitHub API error: {}", response.status()));
    }

    let release: GitHubRelease = response.json().await.map_err(|e| {
        log_warn!("Failed to parse release data from GitHub API: {}", e);
        format!("Failed to parse release data: {e}")
    })?;

    log_debug!(
        "Successfully fetched and parsed latest release: '{}'",
        release.name
    );

    if release.prerelease {
        log_info!("Latest release is a pre-release, skipping update check.");
        return Ok(UpdateInfo {
            available: false,
            current_version: current_version.to_string(),
            latest_version: release.tag_name.clone(),
            release_notes: "Latest release is a prerelease".to_string(),
            download_url: String::new(),
            changelog: get_changelog(),
            translations: None,
            release_date: release.published_at,
            is_critical: false,
        });
    }

    let latest_version = release.tag_name.trim_start_matches('v');
    log_info!("Latest stable version found: {}", latest_version);

    let is_newer = match compare_versions(current_version, latest_version) {
        Ok(Ordering::Less) => true,
        Ok(_) => false,
        Err(e) => {
            log_warn!("Failed to compare versions: {}", e);
            false
        }
    };

    if is_newer {
        log_info!("A new version is available: {}", latest_version);
    } else {
        log_info!("Application is up to date.");
    }

    let download_url = if cfg!(target_os = "windows") {
        release
            .assets
            .iter()
            .find(|asset| Data::has_extension(&asset.name, ".msi"))
            .or_else(|| {
                release
                    .assets
                    .iter()
                    .find(|asset| Data::has_extension(&asset.name, ".exe"))
            })
            .map(|asset| asset.browser_download_url.clone())
            .unwrap_or_else(|| {
                log_warn!("No MSI or EXE asset found for release");
                String::new()
            })
    } else {
        log_warn!("Auto-update not supported on this platform, no download URL will be provided.");
        String::new()
    };

    if download_url.is_empty() {
        log_warn!("No suitable installer found for the current platform.");
        return Err(format!(
            "No suitable installer found. Please download manually from {}",
            release.html_url
        ));
    }
    log_debug!("Found suitable installer URL: {}", download_url);

    let (parsed_changelog, parsed_translations) = extract_changelog_json_block(&release.body)
        .and_then(|content| {
            log_debug!("Found changelog JSON block in release notes");
            parse_changelog_and_translations(&content).ok()
        })
        .unwrap_or_else(|| {
            log_warn!("No valid changelog JSON block found in release notes");
            Default::default()
        });

    let is_critical = release.body.to_lowercase().contains("security")
        || release.body.to_lowercase().contains("critical");
    if is_critical {
        log_warn!("Update is marked as critical.");
    }

    Ok(UpdateInfo {
        available: is_newer,
        current_version: current_version.to_string(),
        latest_version: release.tag_name,
        release_notes: release.body.clone(),
        download_url,
        changelog: parsed_changelog,
        translations: parsed_translations,
        release_date: release.published_at,
        is_critical,
    })
}

#[tauri::command]
pub async fn download_and_install_update(download_url: String) -> Result<(), String> {
    log_info!("Starting update download and installation process.");
    if download_url.is_empty() {
        log_warn!("Update process aborted: No download URL provided.");
        return Err("No download URL provided".to_string());
    }

    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::limited(10))
        .build()
        .map_err(|e| format!("Failed to build HTTP client: {e}"))?;

    log_debug!("Downloading update from: {}", download_url);

    let response = client.get(&download_url).send().await.map_err(|e| {
        log_error!("Failed to download update: {}", e);
        format!("Failed to download update: {e}")
    })?;

    if !response.status().is_success() {
        log_error!("Update download failed with status: {}", response.status());
        return Err(format!(
            "Download failed with status: {}",
            response.status()
        ));
    }

    let bytes = response.bytes().await.map_err(|e| {
        log_error!("Failed to read update data from response: {}", e);
        format!("Failed to read update data: {e}")
    })?;

    log_debug!("Downloaded {} MB", bytes.len() / (1024 * 1024));

    let temp_dir = std::env::temp_dir();
    let file_name = download_url.split('/').next_back().unwrap_or("update.msi");
    let temp_file = temp_dir.join(file_name);

    log_debug!("Writing update to temp file: {:?}", temp_file);

    if !file_name.ends_with(".msi") {
        log_error!("Downloaded file is not an MSI installer: {}", file_name);
        return Err(format!(
            "Downloaded file is not an MSI. Please download manually from {download_url}"
        ));
    }

    std::fs::write(&temp_file, bytes).map_err(|e| {
        log_error!("Failed to write update file to temp directory: {}", e);
        format!("Failed to write update file: {e}")
    })?;

    #[cfg(target_os = "windows")]
    {
        use std::io::Write;
        use std::os::windows::process::CommandExt;

        let current_exe_name = std::env::current_exe()
            .ok()
            .and_then(|p| p.file_name().map(|s| s.to_string_lossy().to_string()))
            .unwrap_or_else(|| "collapseloader.exe".to_string());
        log_debug!("Current executable name: {}", current_exe_name);

        let msi_path = temp_file.to_string_lossy().to_string();
        let script_path = std::env::temp_dir().join("cl_update_and_restart.bat");
        log_debug!("Updater script path: {:?}", script_path);

        let quoted_msi = msi_path;

        let script_content = format!(
            r#"@echo off
setlocal enabledelayedexpansion
echo Waiting for process {current_exe_name} to exit...
:waitloop
tasklist /FI "IMAGENAME eq {current_exe_name}" | find /I "{current_exe_name}" >nul
if %ERRORLEVEL%==0 (
    timeout /t 1 >nul
    goto waitloop
)
echo Installing update silently...
msiexec /i "{quoted_msi}" /qn /norestart >nul 2>&1
set "TP1=%ProgramFiles%\collapseloader\collapseloader.exe"
call set "TP2=%%ProgramFiles(x86)%%\collapseloader\collapseloader.exe"
set "TP3=%LocalAppData%\Programs\collapseloader\collapseloader.exe"

set "EXE_PATH=%ProgramFiles%\collapseloader\collapseloader.exe"
if not exist "%EXE_PATH%" call set "EXE_PATH=%%ProgramFiles(x86)%%\collapseloader\collapseloader.exe"
if not exist "%EXE_PATH%" set "EXE_PATH=%LocalAppData%\Programs\collapseloader\collapseloader.exe"

if exist "%EXE_PATH%" (
    echo Launching updated application...
    start "" "%EXE_PATH%"
) else (
    echo Could not locate installed application. Please start it manually.
    echo Tried paths:
    echo   !TP1!
    echo   !TP2!
    echo   !TP3!
    timeout /t 5 >nul
)

del "{quoted_msi}" >nul 2>&1
exit
"#
        );

        {
            let mut file = std::fs::File::create(&script_path)
                .map_err(|e| format!("Failed to create updater script: {e}"))?;
            file.write_all(script_content.as_bytes())
                .map_err(|e| format!("Failed to write updater script: {e}"))?;
            log_debug!("Updater script created successfully.");
        }

        let mut cmd = std::process::Command::new("cmd.exe");
        cmd.args(["/C", "start", "", &script_path.to_string_lossy()]);
        const DETACHED_PROCESS: u32 = 0x0000_0008;
        cmd.creation_flags(DETACHED_PROCESS);
        cmd.spawn().map_err(|e| {
            log_error!("Failed to launch updater script: {}", e);
            format!("Failed to launch updater script: {e}")
        })?;

        log_info!("Updater script launched. Exiting application to allow update.");
        std::process::exit(0);
    }

    #[cfg(not(target_os = "windows"))]
    {
        Err("Auto-update not supported on this platform".to_string())
    }
}

#[tauri::command]
pub const fn get_changelog() -> Vec<ChangelogEntry> {
    Vec::new()
}

fn extract_changelog_json_block(body: &str) -> Option<String> {
    let marker = if let Some(idx) = body.find("```changelog") {
        (idx, "```changelog")
    } else if let Some(idx) = body.find("``` changelog") {
        (idx, "``` changelog")
    } else {
        log_debug!("No changelog JSON block marker found in release body.");
        return None;
    };

    let start_idx = marker.0;

    let after_marker = &body[start_idx..];
    let first_newline = after_marker.find('\n')?;
    let content_start = start_idx + first_newline + 1;

    let rest = &body[content_start..];
    if let Some(closing_rel) = rest.find("```") {
        let closing_idx = content_start + closing_rel;
        let content = &body[content_start..closing_idx];
        log_debug!("Extracted changelog content block.");
        return Some(content.trim().to_string());
    }

    log_warn!("Found changelog block marker but no closing '```'.");
    None
}

fn parse_changelog_and_translations(
    content: &str,
) -> Result<(Vec<ChangelogEntry>, Option<JsonValue>), String> {
    if let Ok(v) = serde_json::from_str::<Vec<ChangelogEntry>>(content) {
        log_debug!("Parsed changelog as a direct array of entries.");
        return Ok((v, None));
    }

    if let Ok(entry) = serde_json::from_str::<ChangelogEntry>(content) {
        log_debug!("Parsed changelog as a single entry object.");
        return Ok((vec![entry], None));
    }

    let root: JsonValue = serde_json::from_str(content)
        .map_err(|e| format!("Failed to parse changelog JSON root: {e}"))?;

    if root.is_object() {
        let entries_val = root.get("entries");
        let translations_val = root.get("translations").cloned();

        if let Some(ev) = entries_val {
            let entries_json = serde_json::to_string(ev)
                .map_err(|e| format!("Failed to serialize entries node: {e}"))?;
            let entries: Vec<ChangelogEntry> = serde_json::from_str(&entries_json)
                .map_err(|e| format!("Failed to parse entries array: {e}"))?;
            log_debug!(
                "Parsed changelog from a root object with 'entries' and 'translations' keys."
            );
            return Ok((entries, translations_val));
        }
    }

    log_warn!("Changelog JSON is not in a recognized format.");
    Err("Changelog JSON is not in a recognized format".to_string())
}
