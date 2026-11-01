use serde::Serialize;
use std::net::IpAddr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, LazyLock, Mutex};
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::task::JoinHandle;

#[derive(Clone)]
pub struct GameProxyConfig {
    pub proxy_type: String,
    pub proxy_host: String,
    pub proxy_port: u16,
    pub username: String,
    pub password: String,
}

pub struct GameProxyTunnel {
    local_port: u16,
    task: JoinHandle<()>,
    used: Arc<AtomicBool>,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GameProxyStatus {
    pub status: String,
    pub server: Option<String>,
    pub error: Option<String>,
}

static GAME_PROXY_STATUS: LazyLock<Mutex<GameProxyStatus>> = LazyLock::new(|| {
    Mutex::new(GameProxyStatus {
        status: "off".to_string(),
        server: None,
        error: None,
    })
});

pub fn game_proxy_status() -> GameProxyStatus {
    GAME_PROXY_STATUS
        .lock()
        .map(|status| status.clone())
        .unwrap_or(GameProxyStatus {
            status: "error".to_string(),
            server: None,
            error: Some("Proxy status is unavailable.".to_string()),
        })
}

fn set_game_proxy_status(status: &str, server: Option<String>, error: Option<String>) {
    if let Ok(mut current) = GAME_PROXY_STATUS.lock() {
        current.status = status.to_string();
        current.server = server;
        current.error = error;
    }
}

impl GameProxyTunnel {
    pub fn local_port(&self) -> u16 {
        self.local_port
    }

    pub fn was_used(&self) -> bool {
        self.used.load(Ordering::Acquire)
    }
}

impl Drop for GameProxyTunnel {
    fn drop(&mut self) {
        self.task.abort();
        set_game_proxy_status("off", None, None);
    }
}

pub async fn start_game_proxy_tunnel(
    config: GameProxyConfig,
    client_id: u32,
) -> Result<GameProxyTunnel, String> {
    if config.proxy_host.trim().is_empty() || config.proxy_port == 0 {
        return Err("Enter a valid SOCKS proxy address and port.".to_string());
    }
    if config.proxy_type == "socks5" && (config.username.is_empty() != config.password.is_empty()) {
        return Err("Enter both SOCKS5 username and password, or leave both empty.".to_string());
    }
    if config.proxy_type == "socks4" && !config.password.is_empty() {
        return Err("SOCKS4 does not support a password.".to_string());
    }
    if config.proxy_type != "socks4" && config.proxy_type != "socks5" {
        return Err("Unsupported SOCKS proxy type.".to_string());
    }
    if config.username.len() > u8::MAX as usize || config.password.len() > u8::MAX as usize {
        return Err("Proxy username and password must be at most 255 bytes.".to_string());
    }

    let listener = TcpListener::bind(("127.0.0.1", 0))
        .await
        .map_err(|error| format!("Failed to start the Minecraft proxy tunnel: {error}"))?;
    let local_port = listener
        .local_addr()
        .map_err(|error| format!("Failed to read the Minecraft proxy tunnel address: {error}"))?
        .port();
    let used = Arc::new(AtomicBool::new(false));
    let task_used = Arc::clone(&used);
    set_game_proxy_status("waiting", None, None);
    let task = tokio::spawn(async move {
        loop {
            let (mut client, _) = match listener.accept().await {
                Ok(connection) => connection,
                Err(error) => {
                    crate::log_warn!("Minecraft proxy tunnel accept failed: {}", error);
                    break;
                }
            };
            let config = config.clone();
            let used = Arc::clone(&task_used);
            tokio::spawn(async move {
                if let Err(error) = relay_connection(&mut client, &config, client_id, &used).await {
                    set_game_proxy_status("error", None, Some(error.clone()));
                    crate::core::clients::client::add_log_line(
                        client_id,
                        format!("Minecraft proxy tunnel error: {error}"),
                    );
                    crate::log_warn!("Minecraft proxy tunnel connection failed: {}", error);
                }
            });
        }
    });

    Ok(GameProxyTunnel {
        local_port,
        task,
        used,
    })
}

pub async fn test_proxy_connection(
    config: GameProxyConfig,
    server_host: &str,
    server_port: u16,
) -> Result<(), String> {
    validate_proxy_config(&config)?;
    if server_host.trim().is_empty() || server_port == 0 {
        return Err("Enter a valid Minecraft server address and port for the test.".to_string());
    }

    tokio::time::timeout(Duration::from_secs(10), async {
        let mut proxy = TcpStream::connect((
            config.proxy_host.trim_matches(['[', ']']),
            config.proxy_port,
        ))
        .await
        .map_err(|error| format!("Could not connect to the SOCKS proxy: {error}"))?;
        match config.proxy_type.as_str() {
            "socks4" => connect_socks4(&mut proxy, &config, server_host, server_port).await,
            "socks5" => connect_socks5(&mut proxy, &config, server_host, server_port).await,
            _ => Err("Unsupported SOCKS proxy type.".to_string()),
        }
    })
    .await
    .map_err(|_| "SOCKS proxy test timed out after 10 seconds.".to_string())?
}

fn validate_proxy_config(config: &GameProxyConfig) -> Result<(), String> {
    if config.proxy_host.trim().is_empty() || config.proxy_port == 0 {
        return Err("Enter a valid SOCKS proxy address and port.".to_string());
    }
    if config.proxy_type == "socks5" && (config.username.is_empty() != config.password.is_empty()) {
        return Err("Enter both SOCKS5 username and password, or leave both empty.".to_string());
    }
    if config.proxy_type == "socks4" && !config.password.is_empty() {
        return Err("SOCKS4 does not support a password.".to_string());
    }
    if config.proxy_type != "socks4" && config.proxy_type != "socks5" {
        return Err("Unsupported SOCKS proxy type.".to_string());
    }
    if config.username.len() > u8::MAX as usize || config.password.len() > u8::MAX as usize {
        return Err("Proxy username and password must be at most 255 bytes.".to_string());
    }
    Ok(())
}

async fn relay_connection(
    client: &mut TcpStream,
    config: &GameProxyConfig,
    client_id: u32,
    used: &AtomicBool,
) -> Result<(), String> {
    negotiate_local_socks5(client).await?;
    let (server_host, server_port) = read_local_destination(client).await?;
    if !used.swap(true, Ordering::AcqRel) {
        crate::core::clients::client::add_log_line(
            client_id,
            format!("Minecraft requested a proxy connection to {server_host}:{server_port}."),
        );
    }
    let mut proxy = match TcpStream::connect((
        config.proxy_host.trim_matches(['[', ']']),
        config.proxy_port,
    ))
    .await
    {
        Ok(proxy) => proxy,
        Err(error) => {
            let message = format!("Failed to connect to the SOCKS proxy: {error}");
            send_local_failure(client, 1).await?;
            return Err(message);
        }
    };

    let upstream_result = match config.proxy_type.as_str() {
        "socks4" => connect_socks4(&mut proxy, config, &server_host, server_port).await,
        "socks5" => connect_socks5(&mut proxy, config, &server_host, server_port).await,
        _ => Err("Unsupported SOCKS proxy type.".to_string()),
    };
    if let Err(error) = upstream_result {
        let response = upstream_socks5_reply_code(&error);
        send_local_failure(client, response).await?;
        return Err(error);
    }

    set_game_proxy_status(
        "connected",
        Some(format!("{server_host}:{server_port}")),
        None,
    );
    crate::core::clients::client::add_log_line(
        client_id,
        format!(
            "Minecraft proxy connected through SOCKS{} to {server_host}:{server_port}.",
            if config.proxy_type == "socks4" {
                "4"
            } else {
                "5"
            }
        ),
    );

    client
        .write_all(&[5, 0, 0, 1, 0, 0, 0, 0, 0, 0])
        .await
        .map_err(|error| format!("Failed to respond to Minecraft SOCKS request: {error}"))?;
    tokio::io::copy_bidirectional(client, &mut proxy)
        .await
        .map_err(|error| format!("Minecraft proxy tunnel relay failed: {error}"))?;
    Ok(())
}

async fn send_local_failure(client: &mut TcpStream, reply_code: u8) -> Result<(), String> {
    client
        .write_all(&[5, reply_code, 0, 1, 0, 0, 0, 0, 0, 0])
        .await
        .map_err(|error| format!("Failed to return SOCKS5 failure to Minecraft: {error}"))
}

fn upstream_socks5_reply_code(error: &str) -> u8 {
    error
        .rsplit_once("(code ")
        .and_then(|(_, code)| code.split(')').next())
        .and_then(|code| code.parse().ok())
        .unwrap_or(1)
}

async fn negotiate_local_socks5(client: &mut TcpStream) -> Result<(), String> {
    let mut greeting = [0; 2];
    client
        .read_exact(&mut greeting)
        .await
        .map_err(|error| format!("Failed to read Minecraft SOCKS greeting: {error}"))?;
    if greeting[0] != 5 {
        return Err("Minecraft attempted to use an unsupported local proxy protocol.".to_string());
    }
    let mut methods = vec![0; greeting[1] as usize];
    client
        .read_exact(&mut methods)
        .await
        .map_err(|error| format!("Failed to read Minecraft SOCKS methods: {error}"))?;
    if !methods.contains(&0) {
        client
            .write_all(&[5, 0xff])
            .await
            .map_err(|error| format!("Failed to reject Minecraft SOCKS methods: {error}"))?;
        return Err("Minecraft did not offer unauthenticated local SOCKS access.".to_string());
    }
    client
        .write_all(&[5, 0])
        .await
        .map_err(|error| format!("Failed to select Minecraft SOCKS method: {error}"))
}

async fn read_local_destination(client: &mut TcpStream) -> Result<(String, u16), String> {
    let mut request = [0; 4];
    client
        .read_exact(&mut request)
        .await
        .map_err(|error| format!("Failed to read Minecraft SOCKS request: {error}"))?;
    if request[0] != 5 || request[1] != 1 {
        client.write_all(&[5, 7, 0, 1, 0, 0, 0, 0, 0, 0]).await.ok();
        return Err("Only TCP connections are supported by the Minecraft proxy.".to_string());
    }

    let host = match request[3] {
        1 => {
            let mut address = [0; 4];
            client
                .read_exact(&mut address)
                .await
                .map_err(|error| format!("Failed to read Minecraft IPv4 target: {error}"))?;
            std::net::Ipv4Addr::from(address).to_string()
        }
        4 => {
            let mut address = [0; 16];
            client
                .read_exact(&mut address)
                .await
                .map_err(|error| format!("Failed to read Minecraft IPv6 target: {error}"))?;
            std::net::Ipv6Addr::from(address).to_string()
        }
        3 => {
            let mut length = [0];
            client
                .read_exact(&mut length)
                .await
                .map_err(|error| format!("Failed to read Minecraft domain length: {error}"))?;
            let mut domain = vec![0; length[0] as usize];
            client
                .read_exact(&mut domain)
                .await
                .map_err(|error| format!("Failed to read Minecraft domain target: {error}"))?;
            String::from_utf8(domain)
                .map_err(|error| format!("Minecraft server domain is invalid: {error}"))?
        }
        _ => return Err("Minecraft requested an invalid SOCKS target type.".to_string()),
    };

    let mut port = [0; 2];
    client
        .read_exact(&mut port)
        .await
        .map_err(|error| format!("Failed to read Minecraft server port: {error}"))?;
    Ok((host, u16::from_be_bytes(port)))
}

async fn connect_socks5(
    proxy: &mut TcpStream,
    config: &GameProxyConfig,
    server_host: &str,
    server_port: u16,
) -> Result<(), String> {
    let needs_auth = !config.username.is_empty();

    if needs_auth {
        proxy
            .write_all(&[5, 1, 2])
            .await
            .map_err(|error| format!("Failed to negotiate SOCKS5 authentication: {error}"))?;
    } else {
        proxy
            .write_all(&[5, 1, 0])
            .await
            .map_err(|error| format!("Failed to negotiate SOCKS5 authentication: {error}"))?;
    }

    let mut method = [0; 2];
    proxy
        .read_exact(&mut method)
        .await
        .map_err(|error| format!("Failed to read SOCKS5 authentication response: {error}"))?;
    let expected_method = if needs_auth { 2 } else { 0 };
    if method != [5, expected_method] {
        return Err("The SOCKS5 proxy rejected the available authentication method.".to_string());
    }

    if needs_auth {
        proxy
            .write_all(&[1, config.username.len() as u8])
            .await
            .map_err(|error| format!("Failed to send SOCKS5 username: {error}"))?;
        proxy
            .write_all(config.username.as_bytes())
            .await
            .map_err(|error| format!("Failed to send SOCKS5 username: {error}"))?;
        proxy
            .write_all(&[config.password.len() as u8])
            .await
            .map_err(|error| format!("Failed to send SOCKS5 password: {error}"))?;
        proxy
            .write_all(config.password.as_bytes())
            .await
            .map_err(|error| format!("Failed to send SOCKS5 password: {error}"))?;

        let mut auth_response = [0; 2];
        proxy
            .read_exact(&mut auth_response)
            .await
            .map_err(|error| format!("Failed to read SOCKS5 authentication response: {error}"))?;
        if auth_response != [1, 0] {
            return Err("SOCKS5 authentication failed.".to_string());
        }
    }

    let mut request = vec![5, 1, 0];
    match server_host.parse::<IpAddr>() {
        Ok(IpAddr::V4(address)) => {
            request.push(1);
            request.extend_from_slice(&address.octets());
        }
        Ok(IpAddr::V6(address)) => {
            request.push(4);
            request.extend_from_slice(&address.octets());
        }
        Err(_) => {
            let host = server_host.as_bytes();
            if host.is_empty() || host.len() > u8::MAX as usize {
                return Err("Minecraft server address must be at most 255 bytes.".to_string());
            }
            request.extend_from_slice(&[3, host.len() as u8]);
            request.extend_from_slice(host);
        }
    }
    request.extend_from_slice(&server_port.to_be_bytes());
    proxy.write_all(&request).await.map_err(|error| {
        format!("Failed to request the Minecraft server through SOCKS5: {error}")
    })?;

    let mut response = [0; 4];
    proxy
        .read_exact(&mut response)
        .await
        .map_err(|error| format!("Failed to read SOCKS5 connection response: {error}"))?;
    if response[0] != 5 || response[1] != 0 {
        let reason = match response[1] {
            1 => "general proxy failure",
            2 => "connection denied by the proxy's ruleset",
            3 => "destination network is unreachable",
            4 => "destination host is unreachable",
            5 => "destination refused the connection",
            6 => "proxy connection TTL expired",
            7 => "proxy does not support this command",
            8 => "proxy does not support this address type",
            _ => "unknown proxy error",
        };
        return Err(format!(
            "SOCKS5 proxy could not connect to the server: {reason} (code {}).",
            response[1]
        ));
    }
    let address_length = match response[3] {
        1 => 4,
        4 => 16,
        3 => {
            let mut length = [0];
            proxy
                .read_exact(&mut length)
                .await
                .map_err(|error| format!("Failed to read SOCKS5 response address: {error}"))?;
            length[0] as usize
        }
        _ => return Err("SOCKS5 proxy returned an invalid address type.".to_string()),
    };
    let mut remainder = vec![0; address_length + 2];
    proxy
        .read_exact(&mut remainder)
        .await
        .map_err(|error| format!("Failed to finish reading SOCKS5 response: {error}"))?;
    Ok(())
}

async fn connect_socks4(
    proxy: &mut TcpStream,
    config: &GameProxyConfig,
    server_host: &str,
    server_port: u16,
) -> Result<(), String> {
    let username = config.username.as_bytes();
    if username.contains(&0) {
        return Err("SOCKS4 username cannot contain a null byte.".to_string());
    }

    let address = match server_host.parse::<IpAddr>() {
        Ok(IpAddr::V4(address)) => address,
        Ok(IpAddr::V6(_)) => {
            return Err("SOCKS4 cannot connect to an IPv6 server address.".to_string());
        }
        Err(_) => tokio::net::lookup_host((server_host, server_port))
            .await
            .map_err(|error| format!("Failed to resolve the Minecraft server for SOCKS4: {error}"))?
            .find_map(|address| match address.ip() {
                IpAddr::V4(ipv4) => Some(ipv4),
                IpAddr::V6(_) => None,
            })
            .ok_or_else(|| "SOCKS4 requires an IPv4 Minecraft server address.".to_string())?,
    };

    let mut request = vec![4, 1];
    request.extend_from_slice(&server_port.to_be_bytes());
    request.extend_from_slice(&address.octets());
    request.extend_from_slice(username);
    request.push(0);
    proxy.write_all(&request).await.map_err(|error| {
        format!("Failed to request the Minecraft server through SOCKS4: {error}")
    })?;

    let mut response = [0; 8];
    proxy
        .read_exact(&mut response)
        .await
        .map_err(|error| format!("Failed to read SOCKS4 connection response: {error}"))?;
    if response[1] != 0x5a {
        return Err(format!(
            "SOCKS4 proxy could not connect to the server (code {}).",
            response[1]
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::upstream_socks5_reply_code;

    #[test]
    fn forwards_upstream_socks5_reply_code() {
        assert_eq!(
            upstream_socks5_reply_code("SOCKS5 proxy could not connect to the server (code 2)."),
            2
        );
        assert_eq!(
            upstream_socks5_reply_code("SOCKS5 proxy could not connect to the server (code 8)."),
            8
        );
    }

    #[test]
    fn uses_general_failure_for_non_socks_errors() {
        assert_eq!(upstream_socks5_reply_code("Could not connect to proxy."), 1);
    }
}
