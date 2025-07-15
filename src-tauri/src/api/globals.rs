use lazy_static::lazy_static;
use std::path::PathBuf;

use crate::api::network::servers::Server;

pub static CODENAME: &str = "Dioxide";

lazy_static! {
    // pub static ref WEB_SERVERS: Vec<Server> = vec![Server::new("https://web.collapseloader.org/"), Server::new("https://collapse.ttfdk.lol/")];
    pub static ref API_SERVERS: Vec<Server> = vec![
        Server::new("https://api.collapseloader.org/"),
    ];
    pub static ref CDN_SERVERS: Vec<Server> = vec![
        Server::new("https://cdn.collapseloader.org/"),
        Server::new("https://collapse.ttfdk.lol/cdn/"),
        Server::new(
            "https://axkanxneklh7.objectstorage.eu-amsterdam-1.oci.customer-oci.com/n/axkanxneklh7/b/collapse/o/",
        ),
    ];
    pub static ref AUTH_SERVERS: Vec<Server> = vec![Server::new("https://auth.collapseloader.org/"), Server::new("https://collapse.ttfdk.lol/auth/")];
    pub static ref ROOT_DIR: String = {
        let roaming_dir = std::env::var("APPDATA").unwrap_or_else(|_| {
            std::env::var("HOME").unwrap_or_else(|_| ".".to_string())
        });
        let collapse_dir = PathBuf::from(roaming_dir).join("CollapseLoader");
        collapse_dir.to_string_lossy().to_string()
    };
}
