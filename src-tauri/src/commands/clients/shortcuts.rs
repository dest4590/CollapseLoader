use super::{get_client_by_id, sanitize_filename};
use crate::log_info;
use crate::AppState;
use tauri::State;

fn resolve_client_id(id: u32, custom_id: Option<u32>, is_custom: bool) -> u32 {
    if is_custom {
        custom_id.unwrap_or(id)
    } else {
        id
    }
}

#[tauri::command]
pub fn create_client_shortcut(
    id: u32,
    custom_id: Option<u32>,
    is_custom: bool,
    shortcut_name: Option<String>,
    icon_path: Option<String>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let (client_name, _client_path) = if is_custom {
        let cid = custom_id.unwrap_or(id);
        let manager = state.custom_clients.lock();
        let c = manager
            .get_client(cid)
            .cloned()
            .ok_or_else(|| "Custom client not found".to_string())?;
        drop(manager);
        (c.name.clone(), c.file_path.clone())
    } else {
        let c = get_client_by_id(id, &state.clients.manager)?;
        let (folder, _) = c.get_launch_paths()?;
        (c.name.clone(), folder)
    };

    let display_name = shortcut_name
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| client_name.clone());

    let exe_path =
        std::env::current_exe().map_err(|e| format!("Failed to get executable path: {e}"))?;

    log_info!(
        "Creating shortcut '{}' for client '{}' (exe: {})",
        display_name,
        client_name,
        exe_path.display()
    );

    create_shortcut_platform(
        &display_name,
        &exe_path,
        id,
        custom_id,
        is_custom,
        icon_path.as_deref(),
    )
}

#[cfg(target_os = "windows")]
fn create_shortcut_platform(
    display_name: &str,
    exe_path: &std::path::Path,
    id: u32,
    custom_id: Option<u32>,
    is_custom: bool,
    icon_path: Option<&str>,
) -> Result<(), String> {
    let client_id = resolve_client_id(id, custom_id, is_custom);
    let args = format!("collapseloader://launch-client/{}", client_id);

    let exe_str = exe_path.to_string_lossy().into_owned();
    let icon_str = icon_path.unwrap_or(&exe_str).to_string();
    let safe_name = sanitize_filename(display_name);

    let script = format!(
        r#"
$desktop = [Environment]::GetFolderPath('Desktop');
$lnk = Join-Path $desktop '{safe_name}.lnk';
$exe = '{exe}';
$ico = '{ico}';
$ws = New-Object -ComObject WScript.Shell;
$s = $ws.CreateShortcut($lnk);
$s.TargetPath = $exe;
$s.Arguments = '{link_args}';
$s.Description = 'Launch {desc} via CollapseLoader';
$s.IconLocation = $ico;
$s.Save();
Write-Output $lnk
"#,
        safe_name = safe_name.replace('\'', "''"),
        exe = exe_str.replace('\'', "''"),
        ico = icon_str.replace('\'', "''"),
        link_args = args,
        desc = display_name.replace('\'', "''"),
    );

    let output = std::process::Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", &script])
        .output()
        .map_err(|e| format!("Failed to run PowerShell: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("PowerShell shortcut creation failed: {stderr}"));
    }

    let lnk_out = String::from_utf8_lossy(&output.stdout).trim().to_string();
    log_info!("Shortcut created at: {}", lnk_out);
    Ok(())
}

#[cfg(target_os = "linux")]
fn create_shortcut_platform(
    display_name: &str,
    exe_path: &std::path::Path,
    id: u32,
    custom_id: Option<u32>,
    is_custom: bool,
    icon_path: Option<&str>,
) -> Result<(), String> {
    let client_id = resolve_client_id(id, custom_id, is_custom);
    let deep_link = format!("collapseloader://launch-client/{}", client_id);

    let home = std::env::var("HOME").map_err(|_| "Cannot find HOME".to_string())?;
    let desktop = std::path::PathBuf::from(&home).join("Desktop");

    let target_dir = if desktop.exists() {
        desktop
    } else {
        std::path::PathBuf::from(&home)
    };
    let desktop_file = target_dir.join(format!("{}.desktop", sanitize_filename(display_name)));

    let icon_line = if let Some(ip) = icon_path {
        format!("Icon={}", ip)
    } else {
        format!("Icon={}", exe_path.to_string_lossy())
    };

    let content = format!(
        "[Desktop Entry]\nVersion=1.0\nType=Application\nName={name}\nExec={exe} {link}\n{icon}\nTerminal=false\nComment=Launch {name} via CollapseLoader\n",
        name = display_name,
        exe = exe_path.to_string_lossy(),
        link = deep_link,
        icon = icon_line,
    );

    std::fs::write(&desktop_file, &content)
        .map_err(|e| format!("Failed to write .desktop file: {e}"))?;

    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(&desktop_file, std::fs::Permissions::from_mode(0o755))
        .map_err(|e| format!("Failed to set permissions: {e}"))?;

    log_info!("Desktop shortcut created at: {}", desktop_file.display());
    Ok(())
}

#[cfg(target_os = "macos")]
fn create_shortcut_platform(
    display_name: &str,
    _exe_path: &std::path::Path,
    id: u32,
    custom_id: Option<u32>,
    is_custom: bool,
    _icon_path: Option<&str>,
) -> Result<(), String> {
    let client_id = resolve_client_id(id, custom_id, is_custom);
    let deep_link = format!("collapseloader://launch-client/{}", client_id);

    let home = std::env::var("HOME").map_err(|_| "Cannot find HOME".to_string())?;
    let desktop = std::path::PathBuf::from(&home).join("Desktop");
    let target_dir = if desktop.exists() {
        desktop
    } else {
        std::path::PathBuf::from(&home)
    };

    let app_bundle = target_dir.join(format!("{}.app", sanitize_filename(display_name)));
    let contents = app_bundle.join("Contents");
    let macos_dir = contents.join("MacOS");

    std::fs::create_dir_all(&macos_dir)
        .map_err(|e| format!("Failed to create .app bundle: {e}"))?;

    let plist = format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
         <!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">\n\
         <plist version=\"1.0\"><dict>\n\
         <key>CFBundleName</key><string>{name}</string>\n\
         <key>CFBundleExecutable</key><string>launch</string>\n\
         <key>CFBundleIdentifier</key><string>com.collapseloader.shortcut.{id}</string>\n\
         </dict></plist>\n",
        name = display_name,
        id = client_id,
    );
    std::fs::write(contents.join("Info.plist"), plist)
        .map_err(|e| format!("Failed to write Info.plist: {e}"))?;

    let script = format!("#!/bin/sh\nopen '{}'\n", deep_link);
    let script_path = macos_dir.join("launch");
    std::fs::write(&script_path, script)
        .map_err(|e| format!("Failed to write launcher script: {e}"))?;

    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(&script_path, std::fs::Permissions::from_mode(0o755))
        .map_err(|e| format!("Failed to set permissions: {e}"))?;

    log_info!("macOS app shortcut created at: {}", app_bundle.display());
    Ok(())
}
