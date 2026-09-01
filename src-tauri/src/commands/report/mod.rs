mod helpers;

use crate::core::network::create_client;
use crate::core::network::servers::SERVERS;
use crate::core::storage::data::DATA;
use crate::core::utils::globals::{API_SERVERS, CDN_SERVERS};
use helpers::*;
use serde::Serialize;
use std::env;
use std::sync::OnceLock;
use std::time::Duration;
use tauri::AppHandle;

#[derive(Debug, Serialize)]
pub struct PingResult {
    pub url: String,
    pub latency_ms: Option<u64>,
    pub status_code: Option<u16>,
    pub headers: Option<std::collections::HashMap<String, String>>,
    pub content_length: Option<u64>,
    pub response_snippet: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct DnsResult {
    pub host: String,
    pub resolved_ips: Vec<String>,
    pub tcp_port_443_reachable: bool,
    pub tcp_latency_ms: Option<u64>,
    pub ip_latencies: Vec<IpLatency>,
    pub dns_lookup_ms: Option<u64>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct IpLatency {
    pub ip: String,
    pub tcp_latency_ms: Option<u64>,
}

#[derive(Debug, Serialize)]
pub struct AppEnvironment {
    pub os: String,
    pub os_family: String,
    pub arch: String,
    pub version: String,
    pub exec_path: String,
}

#[derive(Debug, Serialize)]
pub struct SystemNetworkInfo {
    pub hostname: String,
    pub local_ip: String,
    pub local_dns_servers: Vec<String>,
    pub proxy_settings: Option<std::collections::HashMap<String, String>>,
}

#[derive(Debug, Serialize)]
pub struct NetworkReport {
    pub timestamp: u64,
    pub date: String,
    pub environment: AppEnvironment,
    pub system_network: SystemNetworkInfo,
    pub pings: Vec<PingResult>,
    pub dns: Vec<DnsResult>,
    pub selected_api: Option<String>,
    pub selected_cdn: Option<String>,
}

#[tauri::command]
pub async fn generate_network_report(app_handle: AppHandle) -> Result<NetworkReport, String> {
    static REPORT_CLIENT: OnceLock<reqwest::Client> = OnceLock::new();
    let client = REPORT_CLIENT
        .get_or_init(|| create_client(Duration::from_secs(5)))
        .clone();

    let mut all_servers: Vec<String> = API_SERVERS
        .iter()
        .chain(CDN_SERVERS.iter())
        .map(|s| s.url.clone())
        .collect();

    all_servers.sort_unstable();
    all_servers.dedup();

    let mut ping_set = tokio::task::JoinSet::new();
    let mut dns_set = tokio::task::JoinSet::new();
    let mut seen_hosts = std::collections::HashSet::new();

    for url in all_servers {
        let url_clone = url.clone();
        let client_clone = client.clone();

        ping_set.spawn(async move {
            let start = std::time::Instant::now();
            match client_clone.get(&url_clone).send().await {
                Ok(resp) => {
                    let latency = start.elapsed().as_millis() as u64;
                    let status = resp.status().as_u16();

                    let mut headers_map =
                        std::collections::HashMap::with_capacity(resp.headers().len());
                    for (k, v) in resp.headers().iter() {
                        headers_map
                            .insert(k.to_string(), v.to_str().unwrap_or_default().to_string());
                    }

                    let header_content_len = resp.content_length();
                    let b = resp.bytes().await.unwrap_or_default();

                    let content_len = if !b.is_empty() {
                        Some(b.len() as u64)
                    } else {
                        header_content_len
                    };

                    let snippet = if !b.is_empty() {
                        Some(
                            String::from_utf8_lossy(&b[..std::cmp::min(b.len(), 1024)]).to_string(),
                        )
                    } else {
                        None
                    };

                    PingResult {
                        url: url_clone,
                        latency_ms: Some(latency),
                        status_code: Some(status),
                        headers: Some(headers_map),
                        content_length: content_len,
                        response_snippet: snippet,
                        error: None,
                    }
                }
                Err(e) => PingResult {
                    url: url_clone,
                    latency_ms: None,
                    status_code: None,
                    headers: None,
                    content_length: None,
                    response_snippet: None,
                    error: Some(e.to_string()),
                },
            }
        });

        if let Ok(parsed_url) = reqwest::Url::parse(&url) {
            if let Some(host) = parsed_url.host_str() {
                if seen_hosts.insert(host.to_string()) {
                    let host_clone = host.to_string();
                    dns_set.spawn(async move {
                        let dns_start = std::time::Instant::now();
                        match tokio::net::lookup_host(format!("{}:443", host_clone)).await {
                            Ok(addrs) => {
                                let ips: Vec<String> = addrs.map(|a| a.ip().to_string()).collect();
                                let dns_lookup_ms = Some(dns_start.elapsed().as_millis() as u64);

                                let mut ip_set = tokio::task::JoinSet::new();
                                for ip in &ips {
                                    let ip_clone = ip.clone();
                                    ip_set.spawn(async move {
                                        let connect_start = std::time::Instant::now();
                                        let res = tokio::time::timeout(
                                            Duration::from_secs(3),
                                            tokio::net::TcpStream::connect(format!(
                                                "{}:443",
                                                ip_clone
                                            )),
                                        )
                                        .await;

                                        let lat = match res {
                                            Ok(Ok(_)) => {
                                                Some(connect_start.elapsed().as_millis() as u64)
                                            }
                                            _ => None,
                                        };
                                        IpLatency {
                                            ip: ip_clone,
                                            tcp_latency_ms: lat,
                                        }
                                    });
                                }

                                let mut ip_latencies = Vec::with_capacity(ips.len());
                                let mut tcp_reachable = false;
                                let mut tcp_latency_ms = None;

                                while let Some(res) = ip_set.join_next().await {
                                    if let Ok(ip_lat) = res {
                                        if let Some(lat) = ip_lat.tcp_latency_ms {
                                            tcp_reachable = true;
                                            tcp_latency_ms = Some(
                                                tcp_latency_ms
                                                    .map_or(lat, |m| std::cmp::min(m, lat)),
                                            );
                                        }
                                        ip_latencies.push(ip_lat);
                                    }
                                }

                                DnsResult {
                                    host: host_clone,
                                    resolved_ips: ips,
                                    tcp_port_443_reachable: tcp_reachable,
                                    tcp_latency_ms,
                                    ip_latencies,
                                    dns_lookup_ms,
                                    error: None,
                                }
                            }
                            Err(e) => {
                                let dns_lookup_ms = Some(dns_start.elapsed().as_millis() as u64);
                                DnsResult {
                                    host: host_clone,
                                    resolved_ips: Vec::new(),
                                    tcp_port_443_reachable: false,
                                    tcp_latency_ms: None,
                                    ip_latencies: Vec::new(),
                                    dns_lookup_ms,
                                    error: Some(e.to_string()),
                                }
                            }
                        }
                    });
                }
            }
        }
    }

    let mut pings = Vec::new();
    while let Some(res) = ping_set.join_next().await {
        if let Ok(ping) = res {
            pings.push(ping);
        }
    }

    let mut dns_results = Vec::new();
    while let Some(res) = dns_set.join_next().await {
        if let Ok(dns) = res {
            dns_results.push(dns);
        }
    }

    Ok(NetworkReport {
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64,
        date: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        environment: AppEnvironment {
            os: env::consts::OS.to_string(),
            os_family: env::consts::FAMILY.to_string(),
            arch: env::consts::ARCH.to_string(),
            version: app_handle.package_info().version.to_string(),
            exec_path: env::current_exe()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|_| "Unknown".to_string()),
        },
        system_network: SystemNetworkInfo {
            hostname: get_hostname(),
            local_ip: get_local_ip(),
            local_dns_servers: get_local_dns_servers().await,
            proxy_settings: get_proxy_env_vars(),
        },
        pings,
        dns: dns_results,
        selected_api: SERVERS.get_api_server_url(),
        selected_cdn: SERVERS.get_cdn_server_url(),
    })
}

#[tauri::command]
pub async fn export_network_report(app_handle: AppHandle) -> Result<String, String> {
    let report = generate_network_report(app_handle).await?;

    let mut txt = String::with_capacity(4096);
    write_report_header(&mut txt, &report);
    write_environment_section(&mut txt, &report);
    write_local_network_section(&mut txt, &report);
    write_current_configuration_section(&mut txt, &report);
    write_ping_section(&mut txt, &report);
    write_dns_section(&mut txt, &report);

    tokio::task::spawn_blocking(move || -> Result<String, String> {
        let export_dir = DATA
            .root_dir
            .lock()
            .unwrap()
            .join("network")
            .join("exports");

        if !export_dir.exists() {
            std::fs::create_dir_all(&export_dir).map_err(|e| e.to_string())?;
        }

        let filename = format!(
            "network_report_{}.txt",
            chrono::Local::now().format("%Y%m%d_%H%M%S")
        );
        let export_path = export_dir.join(&filename);

        std::fs::write(&export_path, txt).map_err(|e| e.to_string())?;

        let path_str = export_path.to_string_lossy().to_string();

        if let Err(e) = open::that(&path_str) {
            eprintln!("Failed to open network report: {}", e);
        }

        Ok(path_str)
    })
    .await
    .map_err(|e| format!("Blocking task failed: {}", e))?
}
