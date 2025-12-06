use crate::core::utils::globals::IRC_HOST;
use crate::{log_error, log_info};
use serde_json::json;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, State};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio::sync::Mutex;

pub struct IrcState {
    pub writer: Arc<Mutex<Option<tokio::net::tcp::OwnedWriteHalf>>>,
}

impl Default for IrcState {
    fn default() -> Self {
        Self {
            writer: Arc::new(Mutex::new(None)),
        }
    }
}

#[tauri::command]
pub async fn connect_irc(
    app: AppHandle,
    state: State<'_, IrcState>,
    token: String,
) -> Result<(), String> {
    log_info!("Connecting to IRC server at {}", IRC_HOST);

    match TcpStream::connect(IRC_HOST).await {
        Ok(stream) => {
            if let Err(e) = (|| -> std::io::Result<()> {
                let sock_ref = socket2::SockRef::from(&stream);
                let mut ka = socket2::TcpKeepalive::new();
                ka = ka.with_time(std::time::Duration::from_secs(20));
                ka = ka.with_interval(std::time::Duration::from_secs(20));
                sock_ref.set_tcp_keepalive(&ka)
            })() {
                log_error!("Failed to set TCP keepalive: {}", e);
                return Err(format!("Failed to set TCP keepalive: {}", e));
            }

            let (reader, mut writer) = stream.into_split();
            let writer_handle = state.writer.clone();

            let auth_packet = json!({
                "op": "auth",
                "token": token,
                "type": "loader",
                "client": "CollapseLoader"
            });

            let auth_str = format!("{}\n", auth_packet.to_string());

            if let Err(e) = writer.write_all(auth_str.as_bytes()).await {
                log_error!("Failed to send auth to IRC: {}", e);
                return Err(format!("Failed to send auth: {}", e));
            }

            *writer_handle.lock().await = Some(writer);

            let app_clone = app.clone();
            let writer_for_task = writer_handle.clone();

            tokio::spawn(async move {
                let mut reader = BufReader::new(reader);
                let mut line = String::new();

                app_clone.emit("irc-connected", ()).unwrap_or_default();

                loop {
                    line.clear();
                    match reader.read_line(&mut line).await {
                        Ok(0) => {
                            log_info!("IRC connection closed by server");
                            app_clone.emit("irc-disconnected", ()).unwrap_or_default();
                            let mut writer_guard = writer_for_task.lock().await;
                            *writer_guard = None;
                            break;
                        }
                        Ok(_) => {
                            let msg = line.trim().to_string();
                            app_clone.emit("irc-message", msg).unwrap_or_default();
                        }
                        Err(e) => {
                            log_error!("Error reading from IRC: {}", e);
                            app_clone
                                .emit("irc-error", e.to_string())
                                .unwrap_or_default();
                            app_clone.emit("irc-disconnected", ()).unwrap_or_default();
                            let mut writer_guard = writer_for_task.lock().await;
                            *writer_guard = None;
                            break;
                        }
                    }
                }
            });

            Ok(())
        }
        Err(e) => {
            log_error!("Failed to connect to IRC: {}", e);
            Err(format!("Failed to connect: {}", e))
        }
    }
}

#[tauri::command]
pub async fn disconnect_irc(state: State<'_, IrcState>) -> Result<(), String> {
    let mut writer = state.writer.lock().await;
    if let Some(mut w) = writer.take() {
        let _ = w.shutdown().await;
    }
    Ok(())
}

#[tauri::command]
pub async fn send_irc_message(state: State<'_, IrcState>, message: String) -> Result<(), String> {
    let mut writer_guard = state.writer.lock().await;
    if let Some(writer) = writer_guard.as_mut() {
        let packet = if message.starts_with('/') || message.starts_with('@') {
            json!({
                "op": "chat",
                "content": message
            })
        } else {
            json!({
                "op": "chat",
                "content": message
            })
        };

        let msg_str = format!("{}\n", packet.to_string());

        if let Err(e) = writer.write_all(msg_str.as_bytes()).await {
            log_error!("Failed to send IRC message: {}", e);
            return Err(format!("Failed to send message: {}", e));
        }
        Ok(())
    } else {
        Err("Not connected".to_string())
    }
}
