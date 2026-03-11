use crate::commands::network::history_file_path;
use crate::core::network::create_client;
use crate::core::network::servers::SERVERS;
use crate::core::utils::globals::{API_SERVERS, CDN_SERVERS};
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

fn get_local_ip() -> String {
    std::net::UdpSocket::bind("0.0.0.0:0")
        .and_then(|s| {
            s.connect("8.8.8.8:80")?;
            s.local_addr()
        })
        .map(|addr| addr.ip().to_string())
        .unwrap_or_else(|_| "Unknown".to_string())
}

fn get_hostname() -> String {
    if let Ok(name) = std::env::var("COMPUTERNAME") {
        return name;
    }
    if let Ok(name) = std::env::var("HOSTNAME") {
        return name;
    }
    if let Ok(name) = std::fs::read_to_string("/etc/hostname") {
        return name.trim().to_string();
    }
    "Unknown".to_string()
}

fn get_proxy_env_vars() -> Option<std::collections::HashMap<String, String>> {
    let mut proxies = std::collections::HashMap::new();
    for key in &["http_proxy", "https_proxy", "all_proxy", "no_proxy"] {
        if let Ok(val) =
            std::env::var(key.to_lowercase()).or_else(|_| std::env::var(key.to_uppercase()))
        {
            proxies.insert(key.to_string(), val);
        }
    }

    if proxies.is_empty() {
        None
    } else {
        Some(proxies)
    }
}

pub async fn get_local_dns_servers() -> Vec<String> {
    tokio::task::spawn_blocking(|| {
        let mut dns_servers = Vec::new();

        #[cfg(target_os = "windows")]
        {
            if let Ok(output) = std::process::Command::new("powershell")
                .args([
                    "-NoProfile",
                    "-Command",
                    "(Get-DnsClientServerAddress -AddressFamily IPv4).ServerAddresses",
                ])
                .output()
            {
                let stdout = String::from_utf8_lossy(&output.stdout);
                for line in stdout.lines() {
                    let ip = line.trim();
                    if !ip.is_empty() {
                        dns_servers.push(ip.to_string());
                    }
                }
            }
        }

        #[cfg(not(target_os = "windows"))]
        {
            if let Ok(content) = std::fs::read_to_string("/etc/resolv.conf") {
                for line in content.lines() {
                    if line.starts_with("nameserver ") {
                        let parts: Vec<&str> = line.split_whitespace().collect();
                        if parts.len() > 1 {
                            dns_servers.push(parts[1].to_string());
                        }
                    }
                }
            }
        }

        dns_servers.dedup();
        dns_servers
    })
    .await
    .unwrap_or_default()
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
    use std::fmt::Write as _;

    writeln!(txt, "==================================================").unwrap();
    writeln!(txt, "            NETWORK DIAGNOSTIC REPORT             ").unwrap();
    writeln!(txt, "==================================================\n").unwrap();

    writeln!(
        txt,
        "Generated at: {} (Timestamp: {})\n",
        report.date, report.timestamp
    )
    .unwrap();

    writeln!(txt, "--- SYSTEM & APP ENVIRONMENT ---").unwrap();
    writeln!(txt, "Hostname: {}", report.system_network.hostname).unwrap();
    writeln!(
        txt,
        "OS: {} ({})",
        report.environment.os, report.environment.os_family
    )
    .unwrap();
    writeln!(txt, "Architecture: {}", report.environment.arch).unwrap();
    writeln!(txt, "App Version: {}", report.environment.version).unwrap();
    writeln!(txt, "Executable Path: {}\n", report.environment.exec_path).unwrap();

    writeln!(txt, "--- LOCAL NETWORK SETTINGS ---").unwrap();
    writeln!(txt, "Local LAN IP: {}", report.system_network.local_ip).unwrap();

    if let Some(proxies) = &report.system_network.proxy_settings {
        writeln!(txt, "System Proxies Detected:").unwrap();
        for (k, v) in proxies {
            writeln!(txt, "  {}: {}", k.to_uppercase(), v).unwrap();
        }
    } else {
        writeln!(txt, "System Proxies Detected: None").unwrap();
    }

    if report.system_network.local_dns_servers.is_empty() {
        writeln!(txt, "Local DNS Servers: Unknown/Failed to parse\n").unwrap();
    } else {
        writeln!(
            txt,
            "Local DNS Servers: {}\n",
            report.system_network.local_dns_servers.join(", ")
        )
        .unwrap();
    }

    writeln!(txt, "--- CURRENT CONFIGURATION ---").unwrap();
    writeln!(
        txt,
        "Selected API Server: {}",
        report.selected_api.as_deref().unwrap_or("None")
    )
    .unwrap();
    writeln!(
        txt,
        "Selected CDN Server: {}\n",
        report.selected_cdn.as_deref().unwrap_or("None")
    )
    .unwrap();

    writeln!(txt, "--- HTTP SERVER REACHABILITY (PING) ---").unwrap();
    for p in &report.pings {
        writeln!(txt, "URL: {}", p.url).unwrap();
        if let Some(latency) = p.latency_ms {
            writeln!(txt, "  HTTP Latency: {} ms", latency).unwrap();
        }
        if let Some(status) = p.status_code {
            writeln!(txt, "  HTTP Status: {}", status).unwrap();
        }
        if let Some(len) = p.content_length {
            writeln!(txt, "  Content Length: {} bytes", len).unwrap();
        }

        if let Some(headers) = &p.headers {
            writeln!(txt, "  Response Headers:").unwrap();
            for (k, v) in headers {
                writeln!(txt, "    {}: {}", k, v).unwrap();
            }
        }

        if let Some(snip) = &p.response_snippet {
            writeln!(txt, "  Response Snippet:").unwrap();
            writeln!(txt, "    {}", snip.replace('\n', "\\n")).unwrap();
        }

        if let Some(err) = &p.error {
            writeln!(txt, "  HTTP Error: {}", err).unwrap();
        }
        writeln!(txt).unwrap();
    }

    writeln!(txt, "--- DNS RESOLUTION & TCP CHECK ---").unwrap();
    for d in &report.dns {
        writeln!(txt, "Host: {}", d.host).unwrap();

        if d.resolved_ips.is_empty() {
            writeln!(txt, "  IPs: None resolved (Blocked or DNS down)").unwrap();
        } else {
            writeln!(txt, "  IPs: {}", d.resolved_ips.join(", ")).unwrap();
        }

        writeln!(
            txt,
            "  TCP 443 Reachable: {}",
            if d.tcp_port_443_reachable {
                "YES"
            } else {
                "NO"
            }
        )
        .unwrap();

        if let Some(lat) = d.tcp_latency_ms {
            writeln!(txt, "  TCP Latency (best reachable): {} ms", lat).unwrap();
        }
        if let Some(dns_ms) = d.dns_lookup_ms {
            writeln!(txt, "  DNS Lookup Time: {} ms", dns_ms).unwrap();
        }

        if !d.ip_latencies.is_empty() {
            writeln!(txt, "  Per-IP TCP Latencies:").unwrap();
            for ip in &d.ip_latencies {
                writeln!(
                    txt,
                    "    {} - {}",
                    ip.ip,
                    ip.tcp_latency_ms
                        .map(|l| format!("{} ms", l))
                        .unwrap_or_else(|| "unreachable".to_string())
                )
                .unwrap();
            }
        }

        if let Some(err) = &d.error {
            writeln!(txt, "  Error: {}", err).unwrap();
        }
        writeln!(txt).unwrap();
    }

    tokio::task::spawn_blocking(move || -> Result<String, String> {
        let path = history_file_path().ok_or_else(|| "Failed to resolve app dir".to_string())?;
        let export_dir = path.parent().unwrap_or(&path).join("exports");

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
