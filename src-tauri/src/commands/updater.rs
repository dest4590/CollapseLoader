use crate::{
    core::utils::globals::{GITHUB_REPO_NAME, GITHUB_REPO_OWNER},
    log_debug, log_warn,
};
use serde::{Deserialize, Serialize};
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

fn get_changelog_entries() -> Vec<ChangelogEntry> {
    vec![ChangelogEntry {
        version: "0.1.7".to_string(),
        changes: vec![ChangeItem {
            category: Category::Feature,
            description_key: "added_custom_clients".to_string(),
            icon: "🎮".to_string(),
        }],
        date: "27.07.2025".to_string(),
        highlights: vec![
            "Public alpha release".to_string(),
            "Core client management".to_string(),
        ],
    }]
}

#[tauri::command]
pub async fn check_for_updates() -> Result<UpdateInfo, String> {
    let current_version = env!("CARGO_PKG_VERSION");

    let client = reqwest::Client::new();
    let url = format!(
        "https://api.github.com/repos/{}/{}/releases/latest",
        GITHUB_REPO_OWNER, GITHUB_REPO_NAME
    );

    let response = client
        .get(&url)
        .header("User-Agent", "CollapseLoader-Updater")
        .header("Accept", "application/vnd.github+json")
        .header("X-GitHub-Api-Version", "2022-11-28")
        .send()
        .await
        .map_err(|e| format!("Failed to fetch releases: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("GitHub API error: {}", response.status()));
    }

    let release: GitHubRelease = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse release data: {}", e))?;

    if release.prerelease {
        return Ok(UpdateInfo {
            available: false,
            current_version: current_version.to_string(),
            latest_version: release.tag_name.clone(),
            release_notes: "Latest release is a prerelease".to_string(),
            download_url: String::new(),
            changelog: get_changelog(),
            release_date: release.published_at,
            is_critical: false,
        });
    }

    let latest_version = release.tag_name.trim_start_matches('v');

    let is_newer = match compare_versions(current_version, latest_version) {
        Ok(Ordering::Less) => true,
        Ok(_) => false,
        Err(e) => {
            log_warn!("Failed to compare versions: {}", e);
            false
        }
    };

    let download_url = if cfg!(target_os = "windows") {
        release
            .assets
            .iter()
            .find(|asset| asset.name.ends_with(".msi"))
            .or_else(|| {
                release
                    .assets
                    .iter()
                    .find(|asset| asset.name.ends_with(".exe"))
            })
            .map(|asset| asset.browser_download_url.clone())
            .unwrap_or_else(|| {
                log_warn!("No MSI or EXE asset found for release");
                String::new()
            })
    } else {
        String::new()
    };

    if download_url.is_empty() {
        return Err(format!(
            "No suitable installer found. Please download manually from {}",
            release.html_url
        ));
    }

    Ok(UpdateInfo {
        available: is_newer,
        current_version: current_version.to_string(),
        latest_version: release.tag_name,
        release_notes: release.body.clone(),
        download_url,
        changelog: get_changelog(),
        release_date: release.published_at,
        is_critical: release.body.to_lowercase().contains("security")
            || release.body.to_lowercase().contains("critical"),
    })
}

#[tauri::command]
pub async fn download_and_install_update(download_url: String) -> Result<(), String> {
    if download_url.is_empty() {
        return Err("No download URL provided".to_string());
    }

    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::limited(10))
        .build()
        .map_err(|e| format!("Failed to build HTTP client: {}", e))?;

    log_debug!("Downloading from: {}", download_url);

    let response = client
        .get(&download_url)
        .send()
        .await
        .map_err(|e| format!("Failed to download update: {}", e))?;

    if !response.status().is_success() {
        return Err(format!(
            "Download failed with status: {}",
            response.status()
        ));
    }

    let bytes = response
        .bytes()
        .await
        .map_err(|e| format!("Failed to read update data: {}", e))?;

    log_debug!("Downloaded {} bytes", bytes.len());

    let temp_dir = std::env::temp_dir();
    let file_name = download_url.split('/').last().unwrap_or("update.msi");
    let temp_file = temp_dir.join(file_name);

    log_debug!("Writing to temp file: {:?}", temp_file);

    if !file_name.ends_with(".msi") {
        return Err(format!(
            "Downloaded file is not an MSI. Please download manually from {}",
            download_url
        ));
    }

    std::fs::write(&temp_file, bytes).map_err(|e| format!("Failed to write update file: {}", e))?;

    #[cfg(target_os = "windows")]
    {
        let mut command = std::process::Command::new("msiexec");
        command.arg("/i");
        command.arg(&temp_file);
        command.arg("/norestart");

        command
            .spawn()
            .map_err(|e| format!("Failed to start installer: {}", e))?;

        std::thread::sleep(std::time::Duration::from_secs(3));
        std::process::exit(0);
    }

    #[cfg(not(target_os = "windows"))]
    {
        Err("Auto-update not supported on this platform".to_string())
    }
}

#[tauri::command]
pub fn get_changelog() -> Vec<ChangelogEntry> {
    get_changelog_entries()
}
