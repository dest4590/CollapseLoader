use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use discord_rich_presence::{activity, DiscordIpc, DiscordIpcClient};
use lazy_static::lazy_static;

use crate::api::core::settings::SETTINGS;
use crate::{log_debug, log_warn};

const DISCORD_APP_ID: &str = "1225803664204234772";

lazy_static! {
    static ref DISCORD_CLIENT: Mutex<Option<DiscordIpcClient>> = Mutex::new(None);
}

pub fn initialize() -> Result<(), String> {
    std::thread::spawn(|| {
        let mut client = match DiscordIpcClient::new(DISCORD_APP_ID) {
            Ok(client) => client,
            Err(e) => {
                log_debug!("Failed to create Discord client: {}", e);
                return;
            }
        };

        if let Err(e) = client.connect() {
            log_debug!("Could not connect to Discord: {}", e);
            return;
        }

        log_debug!("Connected to Discord");

        if let Ok(mut discord_client) = DISCORD_CLIENT.try_lock() {
            *discord_client = Some(client);
        } else {
            log_warn!("Could not acquire Discord client lock during initialization");
        }
    });

    Ok(())
}

pub fn update_activity(details: String, state: String) -> Result<(), String> {
    if let Ok(settings) = SETTINGS.try_lock() {
        if !settings.discord_rpc_enabled.value {
            log_debug!("Discord RPC is disabled in settings, skipping activity update");
            return Ok(());
        }
    } else {
        log_debug!("Could not acquire settings lock, skipping Discord activity update");
        return Ok(());
    }

    let mut discord_client_lock = match DISCORD_CLIENT.try_lock() {
        Ok(lock) => lock,
        Err(_) => {
            log_debug!("Could not acquire Discord client lock, skipping update");
            return Ok(());
        }
    };

    let discord_client = match &mut *discord_client_lock {
        Some(client) => client,
        None => return Err("Discord client not initialized".to_string()),
    };

    let start_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let large_text = format!("Version {}", env!("CARGO_PKG_VERSION"));

    let assets = activity::Assets::new()
        .large_image("https://i.imgur.com/ZpWg110.gif")
        .large_text(&large_text);

    let activity = activity::Activity::new()
        .details(&details)
        .state(&state)
        .assets(assets)
        .timestamps(activity::Timestamps::new().start(start_time as i64));

    if let Err(e) = discord_client.set_activity(activity.clone()) {
        log_debug!("Failed to update Discord activity: {}", e);

        if let Err(e) = discord_client.connect() {
            log_debug!("Failed to reconnect to Discord: {}", e);
        } else {
            if let Err(e) = discord_client.set_activity(activity) {
                log_debug!(
                    "Failed to update Discord activity after reconnection: {}",
                    e
                );
            }
        }
    }

    Ok(())
}

pub fn update_activity_async(details: String, state: String) {
    std::thread::spawn(move || {
        if let Err(e) = update_activity(details, state) {
            log_debug!("Failed to update Discord activity asynchronously: {}", e);
        }
    });
}

pub fn shutdown() {
    if let Ok(mut discord_client_lock) = DISCORD_CLIENT.try_lock() {
        if let Some(mut client) = discord_client_lock.take() {
            let _ = client.close();
            log_debug!("Discord RPC connection closed");
        }
    }
}

pub fn toggle_rpc(enabled: bool) -> Result<(), String> {
    if enabled {
        initialize()
    } else {
        shutdown();
        Ok(())
    }
}
