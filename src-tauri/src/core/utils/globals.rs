use lazy_static::lazy_static;
use std::{fs, path::PathBuf};

use crate::{core::network::servers::Server, log_debug, log_info};

pub static CODENAME: &str = "Fabric";
pub static GITHUB_REPO_OWNER: &str = "dest4590";
pub static GITHUB_REPO_NAME: &str = "CollapseLoader";

pub static JDK_FOLDER: &str = "jdk-21.0.2";

fn parse_env_bool(var: &str) -> bool {
    std::env::var(var)
        .ok()
        .map(|s| {
            let s = s.trim().to_ascii_lowercase();
            matches!(s.as_str(), "1" | "true" | "yes" | "on")
        })
        .unwrap_or(false)
}

lazy_static! {
    pub static ref LOCAL_DEVELOPMENT: bool = {
        let val = parse_env_bool("DEVELOPMENT");
        if val {
            log_info!("Local development mode: {}", val);
        }
        val
    };

    pub static ref USE_LOCAL_SERVER: bool = {
        let val = parse_env_bool("USE_LOCAL_SERVER");
        if val {
            log_info!("Using local server: {}", val);
        }
        val
    };

    pub static ref CDN_SERVERS: Vec<Server> = vec![
        Server::new("https://cdn.collapseloader.org/"),
        Server::new("https://collapse.ttfdk.lol/cdn/"),
        Server::new(
            "https://axkanxneklh7.objectstorage.eu-amsterdam-1.oci.customer-oci.com/n/axkanxneklh7/b/collapse/o/",
        ),
    ];
    pub static ref AUTH_SERVERS: Vec<Server> = vec![if *USE_LOCAL_SERVER {
        Server::new("http://localhost:8000/")
    } else {
        Server::new("https://auth.collapseloader.org/")
    },  Server::new("https://collapse.ttfdk.lol/auth/")];

    pub static ref ROOT_DIR: String = {
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
    };
}
