use std::{fs, path::PathBuf, sync::LazyLock};

use crate::{core::network::servers::Server, log_debug, log_info};

pub static CODENAME: &str = "REBORN";
pub static API_VERSION: &str = "v1";

pub static GITHUB_REPO_OWNER: &str = "dest4590";
pub static GITHUB_REPO_NAME: &str = "CollapseLoader";
pub static GITHUB_REPO_BRANCH: &str = "dev";

pub static IS_LINUX: bool = cfg!(target_os = "linux");
pub static IS_MACOS: bool = cfg!(target_os = "macos");
pub static IS_WINDOWS: bool = cfg!(target_os = "windows");
pub static IS_AARCH64: bool = cfg!(target_arch = "aarch64");

pub static FILE_EXTENSION: &str = if IS_WINDOWS { ".exe" } else { "" };
pub static PATH_SEPARATOR: &str = if IS_WINDOWS { ";" } else { ":" };

pub static LINUX_SUFFIX: &str = "-linux";
pub static MACOS_SUFFIX: &str = "-macos";
pub static ARM64_SUFFIX: &str = "-arm64";
pub static LEGACY_SUFFIX: &str = "-legacy";

pub static JDK21_FOLDER: &str = if IS_LINUX {
    "jdk-21.0.2-linux"
} else if IS_MACOS {
    if IS_AARCH64 {
        "jdk-21.0.9-macos-aarch64"
    } else {
        "jdk-21.0.9-macos-x64"
    }
} else {
    "jdk-21.0.2"
};

pub static JDK8_FOLDER: &str = if IS_LINUX {
    "jdk8-linux"
} else if IS_MACOS {
    if IS_AARCH64 {
        "jdk8-macos-aarch64"
    } else {
        "jdk8-macos-x64"
    }
} else {
    "jdk8"
};

pub static JDK_FOLDERS: LazyLock<Vec<&str>> = LazyLock::new(|| vec![JDK21_FOLDER, JDK8_FOLDER]);

pub static ASSETS_FOLDER: &str = "assets";
pub static ASSETS_FABRIC_FOLDER: &str = "assets-fabric";
pub static LIBRARIES_FOLDER: &str = "libraries";
pub static LIBRARIES_FABRIC_FOLDER: &str = "libraries-fabric";
pub static LIBRARIES_LEGACY_FOLDER: &str = "libraries-legacy";
pub static NATIVES_FOLDER: &str = "natives";
pub static NATIVES_LINUX_FOLDER: &str = "natives-linux";
pub static NATIVES_MACOS_FOLDER: &str = "natives-macos-x64";
pub static NATIVES_MACOS_ARM64_FOLDER: &str = "natives-macos-arm64";
pub static NATIVES_LEGACY_FOLDER: &str = "natives-legacy";
pub static NATIVES_LEGACY_LINUX_FOLDER: &str = "natives-legacy-linux";
pub static NATIVES_FABRIC_FOLDER: &str = "natives-fabric";

pub static ASSETS_ZIP: &str = "misc/assets.zip";
pub static ASSETS_FABRIC_ZIP: &str = "misc/assets-fabric.zip";
pub static LIBRARIES_ZIP: &str = "misc/libraries.zip";
pub static LIBRARIES_FABRIC_ZIP: &str = "misc/libraries-fabric.zip";
pub static LIBRARIES_LEGACY_ZIP: &str = "misc/libraries-legacy.zip";
pub static NATIVES_ZIP: &str = "misc/natives.zip";
pub static NATIVES_LINUX_ZIP: &str = "misc/natives-linux.zip";
pub static NATIVES_MACOS_ZIP: &str = "misc/natives-macos-x64.zip";
pub static NATIVES_MACOS_ARM64_ZIP: &str = "misc/natives-macos-arm64.zip";
pub static NATIVES_LEGACY_ZIP: &str = "misc/natives-legacy.zip";
pub static NATIVES_LEGACY_LINUX_ZIP: &str = "misc/natives-legacy-linux.zip";

pub static MINECRAFT_VERSIONS_FOLDER: &str = "minecraft-versions";
pub static AGENT_OVERLAY_FOLDER: &str = "agent_overlay";
pub static CUSTOM_CLIENTS_FOLDER: &str = "custom_clients";
pub static MODS_FOLDER: &str = "mods";

pub static AGENT_FILE: &str = "CollapseAgent.jar";
pub static OVERLAY_FILE: &str = if IS_LINUX {
    "libCollapseOverlay.so"
} else {
    "CollapseOverlay.dll"
};

pub static IRC_HOST: LazyLock<String> = LazyLock::new(|| {
    if let Ok(url) = std::env::var("FORCE_IRC") {
        if !url.is_empty() {
            log_info!("Using forced IRC server: {}", url);
            return url.to_string();
        }
    }

    "irc.collapseloader.org:1338".to_string()
});

pub static FABRIC_DEPS_URL: &str = "clients/fabric/deps/jars";
pub static FORGE_DEPS_URL: &str = "clients/forge/deps/jars";

pub static VANILLA_CLIENTS_URL: &str = "clients.json";
pub static FABRIC_CLIENTS_URL: &str = "fabric-clients.json";
pub static FORGE_CLIENTS_URL: &str = "forge-clients.json";

fn parse_env_bool(var: &str) -> bool {
    std::env::var(var).ok().is_some_and(|s| {
        let s = s.trim().to_ascii_lowercase();
        matches!(s.as_str(), "1" | "true" | "yes" | "on")
    })
}

pub static SKIP_AGENT_OVERLAY_VERIFICATION: LazyLock<bool> =
    LazyLock::new(|| parse_env_bool("SKIP_AGENT_OVERLAY_VERIFICATION"));

pub static MOCK_CLIENTS: LazyLock<bool> = LazyLock::new(|| parse_env_bool("MOCK_CLIENTS"));

pub static API_SERVERS: LazyLock<Vec<Server>> = LazyLock::new(|| {
    if let Ok(url) = std::env::var("FORCE_API") {
        if !url.is_empty() {
            log_info!("Using forced API server: {}", url);
            return vec![Server::new(&url)];
        }
    }

    vec![
        Server::new("https://huggingface.co/datasets/Collapsecdn/collapsecdn/resolve/main/"),
    ]
});

pub static CDN_SERVERS: LazyLock<Vec<Server>> = LazyLock::new(|| {
    if let Ok(url) = std::env::var("FORCE_CDN") {
        if !url.is_empty() {
            log_info!("Using forced CDN server: {}", url);
            return vec![Server::new(&url)];
        }
    }
    vec![
        Server::new("https://huggingface.co/datasets/Collapsecdn/collapsecdn/resolve/main/"),
    ]
});

pub static ROOT_DIR: LazyLock<String> = LazyLock::new(|| {
    let base_dir = if IS_LINUX {
        std::env::var("XDG_DATA_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                std::env::var("HOME")
                    .map(|h| PathBuf::from(h).join(".local").join("share"))
                    .unwrap_or_else(|_| PathBuf::from("."))
            })
    } else if IS_MACOS {
        std::env::var("HOME")
            .map(|h| PathBuf::from(h).join("Library").join("Application Support"))
            .unwrap_or_else(|_| PathBuf::from("."))
    } else {
        std::env::var("APPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                std::env::var("HOME")
                    .map(PathBuf::from)
                    .unwrap_or_else(|_| PathBuf::from("."))
            })
    };

    let override_file = base_dir.join("CollapseLoaderRoot.txt");
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

    if IS_LINUX {
        let legacy_dir = std::env::var("HOME")
            .map(|h| PathBuf::from(h).join("CollapseLoader"))
            .unwrap_or_else(|_| PathBuf::from("CollapseLoader"));

        let xdg_dir = base_dir.join("CollapseLoader");

        if !xdg_dir.exists() && legacy_dir.exists() {
            return legacy_dir.to_string_lossy().to_string();
        }
    }

    let collapse_dir = base_dir.join("CollapseLoader");
    collapse_dir.to_string_lossy().to_string()
});
