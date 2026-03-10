use crate::core::network::create_client;
use crate::core::network::servers::SERVERS;
use crate::core::storage::data::DATA;
use crate::core::utils::globals::{API_SERVERS, CDN_SERVERS};
use serde::{Deserialize, Serialize};
use std::env;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::time::Duration;
use tauri::{AppHandle, Emitter};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NetworkRequest {
    pub id: String,
    pub method: String,
    pub url: String,
    pub status: Option<u16>,
    pub duration: Option<u64>,
    pub timestamp: u64,
    pub request_headers: Option<std::collections::HashMap<String, String>>,
    pub request_body: Option<serde_json::Value>,
    pub response_headers: Option<std::collections::HashMap<String, String>>,
    pub response_size: Option<u64>,
    pub response_body: Option<serde_json::Value>,
    pub response_text: Option<String>,
    pub error_message: Option<String>,
}

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
    pub arch: String,
    pub version: String,
}

#[derive(Debug, Serialize)]
pub struct SystemNetworkInfo {
    pub local_dns_servers: Vec<String>,
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

fn get_local_dns_servers() -> Vec<String> {
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
}

#[tauri::command]
pub async fn generate_network_report(app_handle: AppHandle) -> Result<NetworkReport, String> {
    let mut pings = Vec::new();
    let mut dns_results = Vec::new();
    let client = create_client(Duration::from_secs(5));

    let mut all_servers = Vec::new();
    for s in API_SERVERS.iter() {
        all_servers.push(s.url.clone());
    }
    for s in CDN_SERVERS.iter() {
        all_servers.push(s.url.clone());
    }
    all_servers.sort();
    all_servers.dedup();

    for url in all_servers {
        let start = std::time::Instant::now();
        match client.get(&url).send().await {
            Ok(resp) => {
                let latency = start.elapsed().as_millis() as u64;
                let status = resp.status().as_u16();

                let mut headers_map: std::collections::HashMap<String, String> =
                    std::collections::HashMap::new();
                for (k, v) in resp.headers().iter() {
                    headers_map.insert(k.to_string(), v.to_str().unwrap_or_default().to_string());
                }

                let header_content_len = resp.content_length();
                let bytes_vec: Vec<u8> = match resp.bytes().await {
                    Ok(b) => b.to_vec(),
                    Err(_) => Vec::new(),
                };
                let content_len = if !bytes_vec.is_empty() {
                    Some(bytes_vec.len() as u64)
                } else {
                    header_content_len
                };
                let snippet = if !bytes_vec.is_empty() {
                    Some(
                        String::from_utf8_lossy(&bytes_vec[..std::cmp::min(bytes_vec.len(), 1024)])
                            .to_string(),
                    )
                } else {
                    None
                };

                pings.push(PingResult {
                    url: url.clone(),
                    latency_ms: Some(latency),
                    status_code: Some(status),
                    headers: Some(headers_map),
                    content_length: content_len,
                    response_snippet: snippet,
                    error: None,
                });
            }
            Err(e) => {
                pings.push(PingResult {
                    url: url.clone(),
                    latency_ms: None,
                    status_code: None,
                    headers: None,
                    content_length: None,
                    response_snippet: None,
                    error: Some(e.to_string()),
                });
            }
        }

        if let Ok(parsed_url) = reqwest::Url::parse(&url) {
            if let Some(host) = parsed_url.host_str() {
                if !dns_results.iter().any(|d: &DnsResult| d.host == host) {
                    let dns_start = std::time::Instant::now();
                    match tokio::net::lookup_host(format!("{}:443", host)).await {
                        Ok(addrs) => {
                            let ips: Vec<String> = addrs.map(|a| a.ip().to_string()).collect();

                            let mut tcp_reachable = false;
                            let mut tcp_latency_ms = None;
                            let mut ip_latencies: Vec<IpLatency> = Vec::new();

                            for ip in &ips {
                                let connect_start = std::time::Instant::now();
                                let res = tokio::time::timeout(
                                    Duration::from_secs(3),
                                    tokio::net::TcpStream::connect(format!("{}:443", ip)),
                                )
                                .await;

                                let lat = match res {
                                    Ok(Ok(_)) => {
                                        let l = connect_start.elapsed().as_millis() as u64;
                                        tcp_reachable = true;
                                        Some(l)
                                    }
                                    _ => None,
                                };
                                if tcp_latency_ms.is_none() {
                                    tcp_latency_ms = lat;
                                }
                                ip_latencies.push(IpLatency {
                                    ip: ip.clone(),
                                    tcp_latency_ms: lat,
                                });
                            }

                            let dns_lookup_ms = Some(dns_start.elapsed().as_millis() as u64);

                            dns_results.push(DnsResult {
                                host: host.to_string(),
                                resolved_ips: ips,
                                tcp_port_443_reachable: tcp_reachable,
                                tcp_latency_ms,
                                ip_latencies,
                                dns_lookup_ms,
                                error: None,
                            });
                        }
                        Err(e) => {
                            let dns_lookup_ms = Some(dns_start.elapsed().as_millis() as u64);
                            dns_results.push(DnsResult {
                                host: host.to_string(),
                                resolved_ips: Vec::new(),
                                tcp_port_443_reachable: false,
                                tcp_latency_ms: None,
                                ip_latencies: Vec::new(),
                                dns_lookup_ms,
                                error: Some(e.to_string()),
                            });
                        }
                    }
                }
            }
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
            arch: env::consts::ARCH.to_string(),
            version: app_handle.package_info().version.to_string(),
        },
        system_network: SystemNetworkInfo {
            local_dns_servers: get_local_dns_servers(),
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

    let mut txt = String::new();

    txt.push_str("==================================================\n");
    txt.push_str("            NETWORK DIAGNOSTIC REPORT             \n");
    txt.push_str("==================================================\n\n");

    txt.push_str(&format!(
        "Generated at: {} (Timestamp: {})\n\n",
        report.date, report.timestamp
    ));

    txt.push_str("--- SYSTEM & APP ENVIRONMENT ---\n");

    txt.push_str(&format!("OS: {}\n", report.environment.os));
    txt.push_str(&format!("Architecture: {}\n", report.environment.arch));
    txt.push_str(&format!("App Version: {}\n\n", report.environment.version));

    txt.push_str("--- LOCAL NETWORK SETTINGS ---\n");

    if report.system_network.local_dns_servers.is_empty() {
        txt.push_str("Local DNS Servers: Unknown/Failed to parse\n\n");
    } else {
        txt.push_str(&format!(
            "Local DNS Servers: {}\n\n",
            report.system_network.local_dns_servers.join(", ")
        ));
    }

    txt.push_str("--- CURRENT CONFIGURATION ---\n");

    txt.push_str(&format!(
        "Selected API Server: {}\n",
        report.selected_api.as_deref().unwrap_or("None")
    ));

    txt.push_str(&format!(
        "Selected CDN Server: {}\n\n",
        report.selected_cdn.as_deref().unwrap_or("None")
    ));

    txt.push_str("--- HTTP SERVER REACHABILITY (PING) ---\n");

    for p in &report.pings {
        txt.push_str(&format!("URL: {}\n", p.url));

        if let Some(latency) = p.latency_ms {
            txt.push_str(&format!("  HTTP Latency: {} ms\n", latency));
        }

        if let Some(status) = p.status_code {
            txt.push_str(&format!("  HTTP Status: {}\n", status));
        }

        if let Some(len) = p.content_length {
            txt.push_str(&format!("  Content Length: {} bytes\n", len));
        }

        if let Some(headers) = &p.headers {
            txt.push_str("  Response Headers:\n");
            for (k, v) in headers {
                txt.push_str(&format!("    {}: {}\n", k, v));
            }
        }

        if let Some(snip) = &p.response_snippet {
            txt.push_str("  Response Snippet:\n");
            let snippet_preview = if snip.len() > 1024 {
                &snip[..1024]
            } else {
                &snip
            };
            txt.push_str(&format!("    {}\n", snippet_preview.replace('\n', "\\n")));
        }

        if let Some(err) = &p.error {
            txt.push_str(&format!("  HTTP Error: {}\n", err));
        }

        txt.push_str("\n");
    }

    txt.push_str("\n");

    txt.push_str("--- DNS RESOLUTION & TCP CHECK ---\n");

    for d in &report.dns {
        txt.push_str(&format!("Host: {}\n", d.host));

        if d.resolved_ips.is_empty() {
            txt.push_str("  IPs: None resolved (Blocked or DNS down)\n");
        } else {
            txt.push_str(&format!("  IPs: {}\n", d.resolved_ips.join(", ")));
        }

        txt.push_str(&format!(
            "  TCP 443 Reachable: {}\n",
            if d.tcp_port_443_reachable {
                "YES"
            } else {
                "NO"
            }
        ));

        if let Some(lat) = d.tcp_latency_ms {
            txt.push_str(&format!("  TCP Latency (first reachable): {} ms\n", lat));
        }

        if let Some(dns_ms) = d.dns_lookup_ms {
            txt.push_str(&format!("  DNS Lookup Time: {} ms\n", dns_ms));
        }

        if !d.ip_latencies.is_empty() {
            txt.push_str("  Per-IP TCP Latencies:\n");
            for ip in &d.ip_latencies {
                txt.push_str(&format!(
                    "    {} - {}\n",
                    ip.ip,
                    ip.tcp_latency_ms
                        .map(|l| format!("{} ms", l))
                        .unwrap_or_else(|| "unreachable".to_string())
                ));
            }
        }

        if let Some(err) = &d.error {
            txt.push_str(&format!("  Error: {}\n", err));
        }

        txt.push_str("\n");
    }

    std::fs::write(&export_path, txt).map_err(|e| e.to_string())?;

    let path_str = export_path.to_string_lossy().to_string();

    if let Err(e) = open::that(&path_str) {
        crate::log_error!("Failed to open network report: {}", e);
    }

    Ok(path_str)
}

#[tauri::command]
pub async fn api_request(
    method: String,
    url: String,
    headers: Option<std::collections::HashMap<String, String>>,
    body: Option<serde_json::Value>,
    app_handle: AppHandle,
) -> Result<serde_json::Value, String> {
    let client = create_client(Duration::from_secs(30));
    let start = std::time::Instant::now();
    let id = uuid::Uuid::new_v4().to_string();
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64;

    let initial_request = NetworkRequest {
        id: id.clone(),
        method: method.clone(),
        url: url.clone(),
        status: None,
        duration: None,
        timestamp,
        request_headers: headers.clone(),
        request_body: body.clone(),
        response_headers: None,
        response_size: None,
        response_body: None,
        response_text: None,
        error_message: None,
    };

    let _ = app_handle.emit("network-request", initial_request);

    let mut rb = match method.to_uppercase().as_str() {
        "GET" => client.get(&url),
        "POST" => client.post(&url),
        "PUT" => client.put(&url),
        "PATCH" => client.patch(&url),
        "DELETE" => client.delete(&url),
        _ => return Err(format!("Unsupported method: {}", method)),
    };

    if let Some(ref h) = headers {
        for (k, v) in h {
            rb = rb.header(k, v);
        }
    }

    if let Some(ref b) = body {
        rb = rb.json(&b);
    }

    let response = rb.send().await.map_err(|e| {
        let rec = NetworkRequest {
            id: id.clone(),
            method: method.clone(),
            url: url.clone(),
            status: Some(0),
            duration: Some(start.elapsed().as_millis() as u64),
            timestamp,
            request_headers: headers.clone(),
            request_body: body.clone(),
            response_headers: None,
            response_size: None,
            response_body: None,
            response_text: None,
            error_message: Some(e.to_string()),
        };
        let _ = app_handle.emit("network-response", rec.clone());
        let _ = save_request_history(&rec);
        e.to_string()
    })?;

    let status = response.status().as_u16();
    let duration = start.elapsed().as_millis() as u64;
    let mut resp_headers_map: std::collections::HashMap<String, String> =
        std::collections::HashMap::new();

    for (k, v) in response.headers().iter() {
        resp_headers_map.insert(k.to_string(), v.to_str().unwrap_or_default().to_string());
    }

    let text = response.text().await.map_err(|e| e.to_string())?;
    let response_size = Some(text.as_bytes().len() as u64);
    let mut response_json: Option<serde_json::Value> = None;

    if !text.is_empty() {
        if let Ok(j) = serde_json::from_str::<serde_json::Value>(&text) {
            response_json = Some(j.clone());
        }
    }

    let rec = NetworkRequest {
        id: id.clone(),
        method: method.clone(),
        url: url.clone(),
        status: Some(status),
        duration: Some(duration),
        timestamp,
        request_headers: headers.clone(),
        request_body: body.clone(),
        response_headers: Some(resp_headers_map),
        response_size: response_size,
        response_body: response_json.clone(),
        response_text: if response_json.is_none() {
            Some(text.clone())
        } else {
            None
        },
        error_message: None,
    };

    let _ = app_handle.emit("network-response", rec.clone());

    let _ = save_request_history(&rec);

    if let Some(j) = response_json {
        Ok(j)
    } else {
        Ok(serde_json::json!(text))
    }
}

fn history_file_path() -> Option<PathBuf> {
    Some(
        DATA.root_dir
            .lock()
            .unwrap()
            .join("network")
            .join("network_history.jsonl"),
    )
}

fn save_request_history(rec: &NetworkRequest) -> Result<(), String> {
    let path = history_file_path().ok_or_else(|| "Failed to resolve app dir".to_string())?;
    let json = serde_json::to_string(rec).map_err(|e| e.to_string())?;
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .map_err(|e| e.to_string())?;

    file.write_all(json.as_bytes()).map_err(|e| e.to_string())?;
    file.write_all(b"\n").map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub fn get_network_history() -> Result<Vec<NetworkRequest>, String> {
    let path = history_file_path().ok_or_else(|| "Failed to resolve app dir".to_string())?;

    if !path.exists() {
        return Ok(vec![]);
    }

    let file = File::open(&path).map_err(|e| e.to_string())?;
    let reader = BufReader::new(file);
    let mut out = Vec::new();

    for line in reader.lines() {
        if let Ok(l) = line {
            if l.trim().is_empty() {
                continue;
            }
            if let Ok(rec) = serde_json::from_str::<NetworkRequest>(&l) {
                out.push(rec);
            }
        }
    }

    Ok(out)
}
