#[cfg(target_os = "windows")]
use crate::core::storage::data::{APP_HANDLE, DATA};
#[cfg(target_os = "windows")]
use crate::core::storage::settings::SETTINGS;
#[cfg(target_os = "windows")]
use crate::core::utils::helpers::emit_to_main_window;
#[cfg(target_os = "windows")]
use crate::core::utils::helpers::is_development_enabled;
#[cfg(target_os = "windows")]
use crate::messagebox;

use crate::log_info;

#[cfg(target_os = "windows")]
use crate::{log_debug, log_error, log_warn};

#[cfg(target_os = "windows")]
const DPI_RELEASE_API: &str =
    "https://api.github.com/repos/dest4590/ZapretCollapseLoader/releases/latest";
#[cfg(target_os = "windows")]
const DPI_ZIP_FALLBACK_URL: &str = "https://github.com/dest4590/ZapretCollapseLoader/releases/download/1.0.0/ZapretCollapseLoader.zip";
#[cfg(target_os = "windows")]
const DPI_ZIP_NAME: &str = "ZapretCollapseLoader.zip";
#[cfg(target_os = "windows")]
const DPI_FOLDER_NAME: &str = "ZapretCollapseLoader";

#[cfg(target_os = "windows")]
pub fn kill_winws() {
    use std::os::windows::process::CommandExt;
    use std::process::Command;

    log_info!("Attempting to kill any existing winws.exe processes");

    let mut cmd = Command::new("taskkill");
    cmd.args(["/F", "/IM", "winws.exe", "/T"]);

    if !is_development_enabled() {
        cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
    }

    match cmd.output() {
        Ok(output) => {
            if output.status.success() {
                log_info!("Successfully killed existing winws.exe processes");
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                if stderr.contains("not found") || stderr.contains("не найден") {
                    log_debug!("No winws.exe processes were found to kill");
                } else {
                    log_warn!("taskkill returned non-zero status: {}", stderr);
                }
            }
        }
        Err(e) => {
            log_error!("Failed to execute taskkill: {}", e);
        }
    }
}

#[cfg(target_os = "windows")]
pub fn enable_dpi_bypass_async() -> Result<(), String> {
    let app_handle_clone = APP_HANDLE.lock().unwrap().clone();

    std::thread::spawn(move || {
        if let Err(e) = enable_dpi_bypass_inner() {
            log_error!("DPI bypass setup failed: {}", e);

            if let Some(app_handle) = app_handle_clone.as_ref() {
                if e.contains("operation requires elevation")
                    || e.contains("Запрошенная операция требует повышения")
                {
                    emit_to_main_window(
                        app_handle,
                        "toast-error",
                        "Failed to enable DPI bypass due to insufficient privileges. Please run the application as administrator and try again.",
                    );
                } else {
                    emit_to_main_window(
                        app_handle,
                        "toast-error",
                        format!("Failed to enable DPI bypass: {}", e),
                    );
                }
            }
        } else if let Some(app_handle) = app_handle_clone.as_ref() {
            emit_to_main_window(
                app_handle,
                "toast-success",
                "DPI bypass enabled successfully",
            );
        }
    });
    Ok(())
}

#[cfg(not(target_os = "windows"))]
pub fn enable_dpi_bypass_async() {
    log_info!("DPI bypass is only supported on Windows; skipping");
}
#[cfg(target_os = "windows")]
pub fn download_dpi_bypass() -> Result<(), String> {
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
                        json.get("assets").and_then(|v| v.as_array()).map_or_else(|| {
                            log_warn!("No assets field in GitHub release JSON; falling back to hardcoded URL");
                            DPI_ZIP_FALLBACK_URL.to_string()
                        }, |assets| {
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
                            found.map_or_else(|| {
                                log_warn!("Asset {} not found in latest release; falling back to hardcoded URL", DPI_ZIP_NAME);
                                DPI_ZIP_FALLBACK_URL.to_string()
                            }, |u| {
                                log_info!(
                                    "Resolved latest DPI package URL from GitHub releases API"
                                );
                                u
                            })
                        })
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

    Ok(())
}

#[cfg(target_os = "windows")]
fn enable_dpi_bypass_inner() -> Result<(), String> {
    download_dpi_bypass()?;
    start_winws_background_inner()?;
    Ok(())
}

#[cfg(target_os = "windows")]
pub fn start_winws_background_if_configured() {
    std::thread::spawn(|| {
        if let Err(e) = start_winws_background_inner() {
            log_error!("winws background start failed: {}", e);

            if e.contains("operation requires elevation")
                || e.contains("Запрошенная операция требует повышения")
            {
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
        return Ok(());
    }

    kill_winws();

    use std::path::PathBuf;
    let base_dir: PathBuf = DATA.root_dir.lock().unwrap().join(DPI_FOLDER_NAME);
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

    fn extend_with_game_filter(base: &str, game_filter: &Option<String>) -> String {
        game_filter
            .as_ref()
            .map_or_else(|| base.to_string(), |filter| format!("{},{}", base, filter))
    }

    let game_filter = std::env::var("GameFilter")
        .ok()
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty());

    let mut args: Vec<String> = Vec::new();

    args.push(format!(
        "--wf-tcp={}",
        extend_with_game_filter("80,443,2053,2083,2087,2096,8443", &game_filter)
    ));
    args.push(format!(
        "--wf-udp={}",
        extend_with_game_filter("443,19294-19344,50000-50100", &game_filter)
    ));
    args.push("--filter-udp=443".to_string());
    args.push(format!(
        "--hostlist={}",
        p(lists_dir.join("list-general.txt"))
    ));
    args.push(format!(
        "--hostlist-exclude={}",
        p(lists_dir.join("list-exclude.txt"))
    ));
    args.push(format!(
        "--ipset-exclude={}",
        p(lists_dir.join("ipset-exclude.txt"))
    ));
    args.push("--dpi-desync=fake".to_string());
    args.push("--dpi-desync-repeats=11".to_string());
    args.push(format!(
        "--dpi-desync-fake-quic={}",
        p(bin_dir.join("quic_initial_www_google_com.bin"))
    ));
    args.push("--new".to_string());

    args.push("--filter-udp=19294-19344,50000-50100".to_string());
    args.push("--filter-l7=discord,stun".to_string());
    args.push("--dpi-desync=fake".to_string());
    args.push("--dpi-desync-repeats=6".to_string());
    args.push("--new".to_string());

    args.push("--filter-tcp=2053,2083,2087,2096,8443".to_string());
    args.push("--hostlist-domains=discord.media".to_string());
    args.push("--dpi-desync=fake,multisplit".to_string());
    args.push("--dpi-desync-split-seqovl=681".to_string());
    args.push("--dpi-desync-split-pos=1".to_string());
    args.push("--dpi-desync-fooling=ts".to_string());
    args.push("--dpi-desync-repeats=8".to_string());
    args.push(format!(
        "--dpi-desync-split-seqovl-pattern={}",
        p(bin_dir.join("tls_clienthello_www_google_com.bin"))
    ));
    args.push(format!(
        "--dpi-desync-fake-tls={}",
        p(bin_dir.join("tls_clienthello_www_google_com.bin"))
    ));
    args.push("--new".to_string());

    args.push("--filter-tcp=443".to_string());
    args.push(format!(
        "--hostlist={}",
        p(lists_dir.join("list-google.txt"))
    ));
    args.push("--ip-id=zero".to_string());
    args.push("--dpi-desync=fake,multisplit".to_string());
    args.push("--dpi-desync-split-seqovl=681".to_string());
    args.push("--dpi-desync-split-pos=1".to_string());
    args.push("--dpi-desync-fooling=ts".to_string());
    args.push("--dpi-desync-repeats=8".to_string());
    args.push(format!(
        "--dpi-desync-split-seqovl-pattern={}",
        p(bin_dir.join("tls_clienthello_www_google_com.bin"))
    ));
    args.push(format!(
        "--dpi-desync-fake-tls={}",
        p(bin_dir.join("tls_clienthello_www_google_com.bin"))
    ));
    args.push("--new".to_string());

    args.push("--filter-tcp=80,443".to_string());
    args.push(format!(
        "--hostlist={}",
        p(lists_dir.join("list-general.txt"))
    ));
    args.push(format!(
        "--hostlist-exclude={}",
        p(lists_dir.join("list-exclude.txt"))
    ));
    args.push(format!(
        "--ipset-exclude={}",
        p(lists_dir.join("ipset-exclude.txt"))
    ));
    args.push("--dpi-desync=fake,multisplit".to_string());
    args.push("--dpi-desync-split-seqovl=654".to_string());
    args.push("--dpi-desync-split-pos=1".to_string());
    args.push("--dpi-desync-fooling=ts".to_string());
    args.push("--dpi-desync-repeats=8".to_string());
    args.push(format!(
        "--dpi-desync-split-seqovl-pattern={}",
        p(bin_dir.join("tls_clienthello_max_ru.bin"))
    ));
    args.push(format!(
        "--dpi-desync-fake-tls={}",
        p(bin_dir.join("tls_clienthello_max_ru.bin"))
    ));
    args.push("--new".to_string());

    args.push("--filter-udp=443".to_string());
    args.push(format!("--ipset={}", p(lists_dir.join("ipset-all.txt"))));
    args.push(format!(
        "--hostlist-exclude={}",
        p(lists_dir.join("list-exclude.txt"))
    ));
    args.push(format!(
        "--ipset-exclude={}",
        p(lists_dir.join("ipset-exclude.txt"))
    ));
    args.push("--dpi-desync=fake".to_string());
    args.push("--dpi-desync-repeats=11".to_string());
    args.push(format!(
        "--dpi-desync-fake-quic={}",
        p(bin_dir.join("quic_initial_www_google_com.bin"))
    ));
    args.push("--new".to_string());

    args.push(format!(
        "--filter-tcp={}",
        extend_with_game_filter("80,443", &game_filter)
    ));
    args.push(format!("--ipset={}", p(lists_dir.join("ipset-all.txt"))));
    args.push(format!(
        "--hostlist-exclude={}",
        p(lists_dir.join("list-exclude.txt"))
    ));
    args.push(format!(
        "--ipset-exclude={}",
        p(lists_dir.join("ipset-exclude.txt"))
    ));
    args.push("--dpi-desync=fake,multisplit".to_string());
    args.push("--dpi-desync-split-seqovl=654".to_string());
    args.push("--dpi-desync-split-pos=1".to_string());
    args.push("--dpi-desync-fooling=ts".to_string());
    args.push("--dpi-desync-repeats=8".to_string());
    args.push(format!(
        "--dpi-desync-split-seqovl-pattern={}",
        p(bin_dir.join("tls_clienthello_max_ru.bin"))
    ));
    args.push(format!(
        "--dpi-desync-fake-tls={}",
        p(bin_dir.join("tls_clienthello_max_ru.bin"))
    ));
    args.push("--new".to_string());

    if let Some(filter) = &game_filter {
        args.push(format!("--filter-udp={}", filter));
        args.push(format!("--ipset={}", p(lists_dir.join("ipset-all.txt"))));
        args.push(format!(
            "--ipset-exclude={}",
            p(lists_dir.join("ipset-exclude.txt"))
        ));
        args.push("--dpi-desync=fake".to_string());
        args.push("--dpi-desync-autottl=2".to_string());
        args.push("--dpi-desync-repeats=10".to_string());
        args.push("--dpi-desync-any-protocol=1".to_string());
        args.push(format!(
            "--dpi-desync-fake-unknown-udp={}",
            p(bin_dir.join("quic_initial_www_google_com.bin"))
        ));
        args.push("--dpi-desync-cutoff=n2".to_string());
    }

    use std::os::windows::process::CommandExt;
    use std::process::Command;

    let create_no_window = if is_development_enabled() {
        0
    } else {
        0x08000000
    };

    match Command::new(&winws_path)
        .current_dir(&bin_dir)
        .creation_flags(create_no_window)
        .args(&args)
        .spawn()
    {
        Ok(_child) => {
            log_info!("winws.exe started in background (spawn ok)");
        }
        Err(e) => {
            log_error!(
                "Failed to spawn winws.exe at {}: {}",
                winws_path.display(),
                e
            );

            if let Some(app_handle) = APP_HANDLE.lock().unwrap().as_ref() {
                emit_to_main_window(
                    app_handle,
                    "toast-error",
                    format!("Failed to start DPI bypass process: {}", e),
                );
            }

            return Err(format!("Failed to start winws.exe: {}", e));
        }
    }
    Ok(())
}
