use crate::core::storage::data::DATA;
use crate::core::storage::settings::SETTINGS;
use crate::core::utils::helpers::is_development_enabled;
use crate::{log_error, log_info, log_warn};

use crate::core::platform::messagebox;

const DPI_RELEASE_API: &str =
    "https://api.github.com/repos/dest4590/ZapretCollapseLoader/releases/latest";
const DPI_ZIP_FALLBACK_URL: &str = "https://github.com/dest4590/ZapretCollapseLoader/releases/download/1.0.0/ZapretCollapseLoader.zip";
const DPI_ZIP_NAME: &str = "ZapretCollapseLoader.zip";
const DPI_FOLDER_NAME: &str = "ZapretCollapseLoader";

#[cfg(target_os = "windows")]
pub fn enable_dpi_bypass_async() -> Result<(), String> {
    std::thread::spawn(|| {
        if let Err(e) = enable_dpi_bypass_inner() {
            log_error!("DPI bypass setup failed: {}", e);
            return Err(e);
        }
        Ok(())
    });
    Ok(())
}

#[cfg(not(target_os = "windows"))]
pub fn enable_dpi_bypass_async() {
    log_info!("DPI bypass is only supported on Windows; skipping");
}

#[cfg(target_os = "windows")]
fn enable_dpi_bypass_inner() -> Result<(), String> {
    let download_url = match reqwest::blocking::Client::new()
        .get(DPI_RELEASE_API)
        .header(reqwest::header::USER_AGENT, "CollapseLoader")
        .header(reqwest::header::ACCEPT, "application/vnd.github.v3+json")
        .send()
    {
        Ok(r) => {
            if r.status().is_success() {
                match r.json::<serde_json::Value>() {
                    Ok(json) => {
                        if let Some(assets) = json.get("assets").and_then(|v| v.as_array()) {
                            let mut found: Option<String> = None;
                            for asset in assets {
                                if asset.get("name").and_then(|n| n.as_str()) == Some(DPI_ZIP_NAME)
                                {
                                    if let Some(url) =
                                        asset.get("browser_download_url").and_then(|u| u.as_str())
                                    {
                                        found = Some(url.to_string());
                                        break;
                                    }
                                }
                            }
                            if let Some(u) = found {
                                log_info!(
                                    "Resolved latest DPI package URL from GitHub releases API"
                                );
                                u
                            } else {
                                log_warn!("Asset {} not found in latest release; falling back to hardcoded URL", DPI_ZIP_NAME);
                                DPI_ZIP_FALLBACK_URL.to_string()
                            }
                        } else {
                            log_warn!("No assets field in GitHub release JSON; falling back to hardcoded URL");
                            DPI_ZIP_FALLBACK_URL.to_string()
                        }
                    }
                    Err(e) => {
                        log_warn!("Failed to parse GitHub release JSON: {}. Falling back to hardcoded URL", e);
                        DPI_ZIP_FALLBACK_URL.to_string()
                    }
                }
            } else {
                log_warn!("GitHub releases API returned non-success status: {}. Falling back to hardcoded URL", r.status());
                DPI_ZIP_FALLBACK_URL.to_string()
            }
        }
        Err(e) => {
            log_warn!(
                "Failed to fetch GitHub releases API: {}. Falling back to hardcoded URL",
                e
            );
            DPI_ZIP_FALLBACK_URL.to_string()
        }
    };

    log_info!("Downloading DPI bypass package from {}", download_url);

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| format!("Failed to create Tokio runtime: {}", e))?;

    rt.block_on(async { DATA.download(&download_url).await })
        .map_err(|e| format!("Failed to download DPI package: {}", e))?;

    DATA.unzip(DPI_ZIP_NAME)?;

    //if let Err(e) = std::fs::remove_file(DATA.root_dir.join(DPI_ZIP_NAME)) {
    //    log_warn!("Failed to remove DPI zip file after extraction: {}", e);
    //}

    start_winws_background_inner()?;

    Ok(())
}

#[cfg(target_os = "windows")]
pub fn start_winws_background_if_configured() {
    std::thread::spawn(|| {
        if let Err(e) = start_winws_background_inner() {
            log_error!("winws background start failed: {}", e);

            if e.contains("operation requires elevation") {
                messagebox::show_error("Failed to start DPI bypass", "Starting the DPI bypass process requires administrator privileges. Please run CollapseLoader as administrator and try again.");
            }
        }
    });
}

#[cfg(not(target_os = "windows"))]
pub fn start_winws_background_if_configured() {
    log_info!("winws is Windows-only; skipping background start");
}

#[cfg(target_os = "windows")]
fn start_winws_background_inner() -> Result<(), String> {
    let settings = SETTINGS.lock().map_err(|e| e.to_string())?.clone();
    if !settings.dpi_bypass.value {
        log_info!("DPI bypass disabled; skipping winws start");
        return Ok(());
    }

    use std::path::PathBuf;
    let base_dir: PathBuf = DATA.root_dir.join(DPI_FOLDER_NAME);
    let bin_dir = base_dir.join("bin");
    let lists_dir = base_dir.join("lists");
    let winws_path = bin_dir.join("winws.exe");

    if !winws_path.exists() {
        log_warn!(
            "winws.exe not found at {}. Triggering DPI package setup",
            winws_path.display()
        );
        if let Err(e) = enable_dpi_bypass_async() {
            return Err(format!("Failed to setup DPI bypass package: {}", e));
        }
        return Ok(());
    }

    fn p<P: AsRef<std::path::Path>>(path: P) -> String {
        path.as_ref().display().to_string()
    }

    let mut args: Vec<String> = Vec::new();

    args.push("--wf-tcp=80,443".to_string());
    args.push("--wf-udp=443".to_string());
    args.push("--filter-udp=443".to_string());
    args.push(format!(
        "--hostlist={}",
        p(lists_dir.join("list-general.txt"))
    ));
    args.push("--dpi-desync=fake".to_string());
    args.push("--dpi-desync-repeats=6".to_string());
    args.push(format!(
        "--dpi-desync-fake-quic={}",
        p(bin_dir.join("quic_initial_www_google_com.bin"))
    ));
    args.push("--new".to_string());

    use std::os::windows::process::CommandExt;
    use std::process::Command;

    let create_no_window = if is_development_enabled() {
        0
    } else {
        0x08000000
    };

    Command::new(&winws_path)
        .current_dir(&bin_dir)
        .creation_flags(create_no_window)
        .args(args)
        .spawn()
        .map_err(|e| format!("Failed to start winws.exe: {e}"))?;

    log_info!("winws.exe started in background");
    Ok(())
}
