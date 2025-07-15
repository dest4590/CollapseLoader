// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[cfg(target_os = "windows")]
use win_msgbox::Okay;

pub fn check_webview2() -> Result<bool, String> {
    #[cfg(target_os = "windows")]
    {
        use winreg::{enums::HKEY_CURRENT_USER, enums::HKEY_LOCAL_MACHINE, RegKey};

        let is_webview2_installed = RegKey::predef(HKEY_LOCAL_MACHINE)
            .open_subkey("SOFTWARE\\WOW6432Node\\Microsoft\\EdgeUpdate\\Clients\\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}")
            .is_ok()
            || RegKey::predef(HKEY_LOCAL_MACHINE)
                .open_subkey("SOFTWARE\\Microsoft\\EdgeUpdate\\Clients\\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}")
                .is_ok()
            || RegKey::predef(HKEY_CURRENT_USER)
                .open_subkey("SOFTWARE\\Microsoft\\EdgeUpdate\\Clients\\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}")
                .is_ok();

        if !is_webview2_installed {
            eprintln!("WebView2 is not installed. Attempting to download and install it...");

            let installer =
                reqwest::blocking::get("https://go.microsoft.com/fwlink/p/?LinkId=2124703")
                    .map_err(|e| e.to_string())?
                    .bytes()
                    .map_err(|e| e.to_string())?;

            let path = std::env::temp_dir().join("MicrosoftEdgeWebview2Setup.exe");

            if std::fs::write(&path, installer).is_ok() {
                let mut command = std::process::Command::new(&path);
                command.arg("/silent");
                command.arg("/install");

                match command.output() {
                    Ok(_) => {
                        eprintln!("WebView2 installer executed successfully.");
                        if let Err(msgbox_err) = win_msgbox::show::<Okay>(
                            "WebView2 has been installed successfully. Please restart the application.",
                        ) {
                            eprintln!("Failed to display restart message box: {:?}", msgbox_err);
                        }
                        Ok(true)
                    }
                    Err(e) => {
                        eprintln!("Failed to execute WebView2 installer: {:?}", e);
                        Ok(false)
                    }
                }
            } else {
                eprintln!("Failed to write WebView2 installer to temporary directory.");
                Ok(false)
            }
        } else {
            Ok(true)
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        Ok(true)
    }
}

fn main() {
    #[cfg(target_os = "windows")]
    if let Err(e) = check_webview2() {
        if let Err(msgbox_err) =
            win_msgbox::show::<Okay>(&format!("Error checking WebView2: {}", e))
        {
            eprintln!("Failed to display error message box: {:?}", msgbox_err);
        }
    }

    collapseloader_lib::run()
}
