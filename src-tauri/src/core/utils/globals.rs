use std::{fs, path::PathBuf, sync::LazyLock};

use crate::{core::network::servers::Server, log_debug, log_info};

pub static CODENAME: &str = "Agent";
pub static GITHUB_REPO_OWNER: &str = "dest4590";
pub static GITHUB_REPO_NAME: &str = "CollapseLoader";

pub static IS_LINUX: bool = cfg!(target_os = "linux");
pub static FILE_EXTENSION: &str = if IS_LINUX { "" } else { ".exe" };
pub static LINUX_SUFFIX: &str = "-linux";
pub static LEGACY_SUFFIX: &str = "-1.12";
pub static PATH_SEPARATOR: &str = if IS_LINUX { ":" } else { ";" };

pub static JDK_FOLDER: &str = if IS_LINUX {
    "jdk-21.0.2_linux"
} else {
    "jdk-21.0.2"
};

// Asset/library file names (without .zip extension)
pub static ASSETS_FOLDER: &str = "assets";
pub static ASSETS_FABRIC_FOLDER: &str = "assets_fabric";
pub static LIBRARIES_FOLDER: &str = "libraries";
pub static LIBRARIES_LEGACY_FOLDER: &str = "libraries-1.12";
pub static LIBRARIES_FABRIC_FOLDER: &str = "libraries_fabric";
pub static NATIVES_FOLDER: &str = "natives";
pub static NATIVES_LINUX_FOLDER: &str = "natives-linux";
pub static NATIVES_LEGACY_FOLDER: &str = "natives-1.12";
pub static NATIVES_FABRIC_FOLDER: &str = "natives_fabric";

// Zip file names
pub static ASSETS_ZIP: &str = "assets.zip";
pub static ASSETS_FABRIC_ZIP: &str = "assets_fabric.zip";
pub static LIBRARIES_ZIP: &str = "libraries.zip";
pub static LIBRARIES_LEGACY_ZIP: &str = "libraries-1.12.zip";
pub static LIBRARIES_FABRIC_ZIP: &str = "libraries_fabric.zip";
pub static NATIVES_ZIP: &str = "natives.zip";
pub static NATIVES_LINUX_ZIP: &str = "natives-linux.zip";
pub static NATIVES_LEGACY_ZIP: &str = "natives-1.12.zip";

// Folder names
pub static MINECRAFT_VERSIONS_FOLDER: &str = "minecraft_versions";
pub static AGENT_OVERLAY_FOLDER: &str = "agent_overlay";
pub static CUSTOM_CLIENTS_FOLDER: &str = "custom_clients";
pub static MODS_FOLDER: &str = "mods";

// IRC Chat Host
pub static IRC_HOST: &str = "irc.collapseloader.org:1338";
//pub static IRC_HOST: &str = "127.0.0.1:1338";

fn parse_env_bool(var: &str) -> bool {
    std::env::var(var).ok().is_some_and(|s| {
        let s = s.trim().to_ascii_lowercase();
        matches!(s.as_str(), "1" | "true" | "yes" | "on")
    })
}

pub static USE_LOCAL_SERVER: LazyLock<bool> = LazyLock::new(|| {
    let val = parse_env_bool("USE_LOCAL_SERVER");
    if val {
        log_info!("Using local server: {}", val);
    }
    val
});

pub static SKIP_AGENT_OVERLAY_VERIFICATION: LazyLock<bool> =
    LazyLock::new(|| parse_env_bool("SKIP_AGENT_OVERLAY_VERIFICATION"));

pub static MOCK_CLIENTS: LazyLock<bool> = LazyLock::new(|| parse_env_bool("MOCK_CLIENTS"));

pub static CDN_SERVERS: LazyLock<Vec<Server>> = LazyLock::new(|| {
    if let Ok(url) = std::env::var("FORCE_CDN") {
        if !url.is_empty() {
            log_info!("Using forced CDN server: {}", url);
            return vec![Server::new(&url)];
        }
    }
    vec![
        Server::new("https://cdn.collapseloader.org/"),
        Server::new("https://collapse.ttfdk.lol/cdn/"),
        Server::new(
            "https://axkanxneklh7.objectstorage.eu-amsterdam-1.oci.customer-oci.com/n/axkanxneklh7/b/collapse/o/",
        ),
    ]
});

pub static AUTH_SERVERS: LazyLock<Vec<Server>> = LazyLock::new(|| {
    if let Ok(url) = std::env::var("FORCE_AUTH") {
        if !url.is_empty() {
            log_info!("Using forced Auth server: {}", url);
            return vec![Server::new(&url)];
        }
    }
    if *USE_LOCAL_SERVER {
        vec![
            Server::new("http://localhost:8000/"),
            Server::new("https://collapse.ttfdk.lol/auth/"),
        ]
    } else {
        vec![
            Server::new("https://auth.collapseloader.org/"),
            Server::new("https://collapse.ttfdk.lol/auth/"),
        ]
    }
});

pub static ROOT_DIR: LazyLock<String> = LazyLock::new(|| {
    let roaming_dir = std::env::var("APPDATA").unwrap_or_else(|_| {
        // fallback for non-windows systems (aka linux/mac)
        std::env::var("HOME").unwrap_or_else(|_| ".".to_string())
    });

    let override_file = PathBuf::from(&roaming_dir).join("CollapseLoaderRoot.txt");
    if let Ok(contents) = fs::read_to_string(&override_file) {
        let override_path = contents.trim_matches(['\n', '\r', '"', '\'']).trim();
        if !override_path.is_empty() {
            let path = PathBuf::from(override_path);
            if path.exists() {
                log_debug!("Using override path: {}", path.display());
                return path.to_string_lossy().to_string();
            }
        }
    }

    let collapse_dir = PathBuf::from(roaming_dir).join("CollapseLoader");
    collapse_dir.to_string_lossy().to_string()
});
