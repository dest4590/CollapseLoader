use super::get_client_by_id;
use crate::core::clients::client::Client;
use crate::AppState;
use serde::Serialize;
use sysinfo::{MemoryRefreshKind, Pid, ProcessRefreshKind, ProcessesToUpdate, RefreshKind, System};
use tauri::State;

#[derive(Debug, Clone, Serialize)]
pub struct ClientRamUsage {
    pub client_id: u32,
    pub is_running: bool,
    pub process_count: usize,
    pub pids: Vec<u32>,
    pub total_memory_bytes: u64,
    pub total_memory_mib: f64,
    pub system_total_memory_bytes: u64,
    pub system_total_memory_mib: f64,
    pub system_memory_percent: f64,
}

fn collect_client_ram_usage(client: &Client) -> ClientRamUsage {
    let pids = crate::core::utils::process::find_processes_by_filename(&client.filename)
        .into_iter()
        .filter_map(|pid| pid.parse::<u32>().ok())
        .collect::<Vec<_>>();

    let mut system = System::new_with_specifics(
        RefreshKind::nothing()
            .with_memory(MemoryRefreshKind::nothing().with_ram())
            .with_processes(ProcessRefreshKind::nothing().with_memory()),
    );
    system.refresh_memory();
    let _ = system.refresh_processes(ProcessesToUpdate::All, true);

    let total_memory_bytes = pids
        .iter()
        .filter_map(|pid| system.process(Pid::from_u32(*pid)))
        .map(|process| process.memory())
        .sum::<u64>();

    let system_total_memory_bytes = system.total_memory();
    let total_memory_mib = total_memory_bytes as f64 / 1024.0 / 1024.0;
    let system_total_memory_mib = system_total_memory_bytes as f64 / 1024.0 / 1024.0;
    let system_memory_percent = if system_total_memory_bytes > 0 {
        (total_memory_bytes as f64 / system_total_memory_bytes as f64) * 100.0
    } else {
        0.0
    };

    ClientRamUsage {
        client_id: client.id,
        is_running: !pids.is_empty(),
        process_count: pids.len(),
        pids,
        total_memory_bytes,
        total_memory_mib,
        system_total_memory_bytes,
        system_total_memory_mib,
        system_memory_percent,
    }
}

#[tauri::command]
pub async fn get_client_ram_usage(
    id: u32,
    state: State<'_, AppState>,
) -> Result<ClientRamUsage, String> {
    let client = get_client_by_id(id, &state.clients.manager)?;
    let client_clone = client.clone();
    tokio::task::spawn_blocking(move || collect_client_ram_usage(&client_clone))
        .await
        .map_err(|e| format!("Failed to get client RAM usage: {e}"))
}
