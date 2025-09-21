use crate::core::error::StartupError;
use winreg::{enums::HKEY_CURRENT_USER, enums::HKEY_LOCAL_MACHINE, RegKey};

fn is_webview2_installed() -> bool {
    let reg_subkeys = [
        "SOFTWARE\\WOW6432Node\\Microsoft\\EdgeUpdate\\Clients\\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}",
        "SOFTWARE\\Microsoft\\EdgeUpdate\\Clients\\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}",
    ];

    for subkey in &reg_subkeys {
        if RegKey::predef(HKEY_LOCAL_MACHINE)
            .open_subkey(subkey)
            .is_ok()
            || RegKey::predef(HKEY_CURRENT_USER)
                .open_subkey(subkey)
                .is_ok()
        {
            return true;
        }
    }

    let candidates = [
        r"Microsoft\EdgeWebView\Application",
        r"Microsoft\EdgeWebView2\Application",
        r"Microsoft\EdgeWebView2\Runner",
        r"Microsoft\EdgeWebView",
    ];

    let check_base = |base: &str| -> bool {
        let base_path = std::path::Path::new(base);
        for cand in &candidates {
            let p = base_path.join(cand);
            if p.exists() {
                if p.is_dir() {
                    if std::fs::read_dir(&p)
                        .map(|mut it| it.next().is_some())
                        .unwrap_or(false)
                    {
                        return true;
                    }
                } else {
                    return true;
                }
            }

            let exe = base_path.join(format!(r"{}\msedgewebview2.exe", cand));
            if exe.exists() {
                return true;
            }
        }
        false
    };

    if let Ok(pf) = std::env::var("ProgramFiles") {
        if check_base(&pf) {
            return true;
        }
    }
    if let Ok(pfx86) = std::env::var("ProgramFiles(x86)") {
        if check_base(&pfx86) {
            return true;
        }
    }

    if let Ok(temp) = std::env::var("TEMP") {
        let temp_path = std::path::Path::new(&temp).join("MicrosoftEdgeWebview2Setup.exe");
        if temp_path.exists() {
            return true;
        }
    }

    false
}

fn install_webview2() -> Result<(), StartupError> {
    eprintln!("WebView2 is not installed. Attempting to download and install it...");

    let installer = reqwest::blocking::get("https://go.microsoft.com/fwlink/p/?LinkId=2124703")
        .map_err(|e| StartupError::WebView2CheckFailed(e.to_string()))?
        .bytes()
        .map_err(|e| StartupError::WebView2CheckFailed(e.to_string()))?;

    let path = std::env::temp_dir().join("MicrosoftEdgeWebview2Setup.exe");

    if std::fs::write(&path, installer).is_ok() {
        let mut command = std::process::Command::new(&path);
        command.arg("/install");

        match command.status() {
            Ok(status) if status.success() => {
                eprintln!("WebView2 installer executed successfully.");
                Ok(())
            }
            _ => {
                eprintln!("Failed to execute WebView2 installer.");
                eprintln!("Command: {:?}", command);
                Err(StartupError::WebView2InstallFailed)
            }
        }
    } else {
        eprintln!("Failed to write WebView2 installer to temporary directory.");
        Err(StartupError::WebView2InstallFailed)
    }
}

pub fn check_platform_dependencies() -> Result<(), StartupError> {
    if !is_webview2_installed() {
        return Err(StartupError::WebView2NotInstalled);
    }
    Ok(())
}

pub fn attempt_install_webview2() -> Result<(), StartupError> {
    install_webview2()
}
