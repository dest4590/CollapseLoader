use crate::commands::report::NetworkReport;

pub(crate) fn get_local_ip() -> String {
    std::net::UdpSocket::bind("0.0.0.0:0")
        .and_then(|s| {
            s.connect("8.8.8.8:80")?;
            s.local_addr()
        })
        .map(|addr| addr.ip().to_string())
        .unwrap_or_else(|_| "Unknown".to_string())
}

pub(crate) fn get_hostname() -> String {
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

pub(crate) fn get_proxy_env_vars() -> Option<std::collections::HashMap<String, String>> {
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

pub(crate) async fn get_local_dns_servers() -> Vec<String> {
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

pub(crate) fn push_line(buffer: &mut String, args: std::fmt::Arguments<'_>) {
    use std::fmt::Write;

    let _ = buffer.write_fmt(args);
    let _ = buffer.write_char('\n');
}

pub(crate) fn write_report_header(buffer: &mut String, report: &NetworkReport) {
    push_line(
        buffer,
        format_args!("=================================================="),
    );
    push_line(
        buffer,
        format_args!("            NETWORK DIAGNOSTIC REPORT             "),
    );
    push_line(
        buffer,
        format_args!("=================================================="),
    );
    push_line(
        buffer,
        format_args!(
            "Generated at: {} (Timestamp: {})",
            report.date, report.timestamp
        ),
    );
    push_line(buffer, format_args!(""));
}

pub(crate) fn write_environment_section(buffer: &mut String, report: &NetworkReport) {
    push_line(buffer, format_args!("--- SYSTEM & APP ENVIRONMENT ---"));
    push_line(
        buffer,
        format_args!("Hostname: {}", report.system_network.hostname),
    );
    push_line(
        buffer,
        format_args!(
            "OS: {} ({})",
            report.environment.os, report.environment.os_family
        ),
    );
    push_line(
        buffer,
        format_args!("Architecture: {}", report.environment.arch),
    );
    push_line(
        buffer,
        format_args!("App Version: {}", report.environment.version),
    );
    push_line(
        buffer,
        format_args!("Executable Path: {}", report.environment.exec_path),
    );
    push_line(buffer, format_args!(""));
}

pub(crate) fn write_local_network_section(buffer: &mut String, report: &NetworkReport) {
    push_line(buffer, format_args!("--- LOCAL NETWORK SETTINGS ---"));
    push_line(
        buffer,
        format_args!("Local LAN IP: {}", report.system_network.local_ip),
    );

    if let Some(proxies) = &report.system_network.proxy_settings {
        push_line(buffer, format_args!("System Proxies Detected:"));
        for (key, value) in proxies {
            push_line(buffer, format_args!("  {}: {}", key.to_uppercase(), value));
        }
    } else {
        push_line(buffer, format_args!("System Proxies Detected: None"));
    }

    if report.system_network.local_dns_servers.is_empty() {
        push_line(
            buffer,
            format_args!("Local DNS Servers: Unknown/Failed to parse"),
        );
        push_line(buffer, format_args!(""));
    } else {
        push_line(
            buffer,
            format_args!(
                "Local DNS Servers: {}",
                report.system_network.local_dns_servers.join(", ")
            ),
        );
        push_line(buffer, format_args!(""));
    }
}

pub(crate) fn write_current_configuration_section(buffer: &mut String, report: &NetworkReport) {
    push_line(buffer, format_args!("--- CURRENT CONFIGURATION ---"));
    push_line(
        buffer,
        format_args!(
            "Selected API Server: {}",
            report.selected_api.as_deref().unwrap_or("None")
        ),
    );
    push_line(
        buffer,
        format_args!(
            "Selected CDN Server: {}",
            report.selected_cdn.as_deref().unwrap_or("None")
        ),
    );
    push_line(buffer, format_args!(""));
}

pub(crate) fn write_ping_section(buffer: &mut String, report: &NetworkReport) {
    push_line(
        buffer,
        format_args!("--- HTTP SERVER REACHABILITY (PING) ---"),
    );

    for ping in &report.pings {
        push_line(buffer, format_args!("URL: {}", ping.url));

        if let Some(latency) = ping.latency_ms {
            push_line(buffer, format_args!("  HTTP Latency: {} ms", latency));
        }

        if let Some(status) = ping.status_code {
            push_line(buffer, format_args!("  HTTP Status: {}", status));
        }

        if let Some(length) = ping.content_length {
            push_line(buffer, format_args!("  Content Length: {} bytes", length));
        }

        if let Some(headers) = &ping.headers {
            push_line(buffer, format_args!("  Response Headers:"));
            for (key, value) in headers {
                push_line(buffer, format_args!("    {}: {}", key, value));
            }
        }

        if let Some(snippet) = &ping.response_snippet {
            push_line(buffer, format_args!("  Response Snippet:"));
            push_line(buffer, format_args!("    {}", snippet.replace('\n', "\\n")));
        }

        if let Some(error) = &ping.error {
            push_line(buffer, format_args!("  HTTP Error: {}", error));
        }

        push_line(buffer, format_args!(""));
    }
}

pub(crate) fn write_dns_section(buffer: &mut String, report: &NetworkReport) {
    push_line(buffer, format_args!("--- DNS RESOLUTION & TCP CHECK ---"));

    for dns in &report.dns {
        push_line(buffer, format_args!("Host: {}", dns.host));

        if dns.resolved_ips.is_empty() {
            push_line(
                buffer,
                format_args!("  IPs: None resolved (Blocked or DNS down)"),
            );
        } else {
            push_line(
                buffer,
                format_args!("  IPs: {}", dns.resolved_ips.join(", ")),
            );
        }

        push_line(
            buffer,
            format_args!(
                "  TCP 443 Reachable: {}",
                if dns.tcp_port_443_reachable {
                    "YES"
                } else {
                    "NO"
                }
            ),
        );

        if let Some(latency) = dns.tcp_latency_ms {
            push_line(
                buffer,
                format_args!("  TCP Latency (best reachable): {} ms", latency),
            );
        }

        if let Some(dns_ms) = dns.dns_lookup_ms {
            push_line(buffer, format_args!("  DNS Lookup Time: {} ms", dns_ms));
        }

        if !dns.ip_latencies.is_empty() {
            push_line(buffer, format_args!("  Per-IP TCP Latencies:"));
            for ip in &dns.ip_latencies {
                push_line(
                    buffer,
                    format_args!(
                        "    {} - {}",
                        ip.ip,
                        ip.tcp_latency_ms
                            .map(|latency| format!("{} ms", latency))
                            .unwrap_or_else(|| "unreachable".to_string())
                    ),
                );
            }
        }

        if let Some(error) = &dns.error {
            push_line(buffer, format_args!("  Error: {}", error));
        }

        push_line(buffer, format_args!(""));
    }
}
