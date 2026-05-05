use crate::commands::clients::{get_app_logs, get_client_by_id};
use crate::core::clients::{client::Client, manager::ClientManager};
use crate::core::utils::logging::APP_LOGS;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

fn make_manager_with_clients(clients: Vec<Client>) -> Arc<Mutex<ClientManager>> {
    Arc::new(Mutex::new(ClientManager { clients }))
}

#[test]
fn get_client_by_id_returns_matching_client() {
    let manager = make_manager_with_clients(vec![
        Client {
            id: 1,
            name: "Alpha".to_string(),
            ..Default::default()
        },
        Client {
            id: 7,
            name: "Target".to_string(),
            ..Default::default()
        },
    ]);

    let client = get_client_by_id(7, &manager).expect("client should exist");

    assert_eq!(client.id, 7);
    assert_eq!(client.name, "Target");
}

#[test]
fn get_client_by_id_returns_error_for_missing_client() {
    let manager = make_manager_with_clients(vec![Client {
        id: 1,
        name: "Alpha".to_string(),
        ..Default::default()
    }]);

    let error = get_client_by_id(99, &manager).expect_err("client should be missing");

    assert!(error.contains("Client with ID 99 not found"));
}

#[test]
fn get_app_logs_returns_log_buffer_snapshot() {
    let original = APP_LOGS.lock().expect("log buffer lock").clone();

    {
        let mut logs = APP_LOGS.lock().expect("log buffer lock");
        *logs = VecDeque::from([
            "first entry".to_string(),
            "second entry".to_string(),
        ]);
    }

    let collected = get_app_logs();

    assert_eq!(collected, vec!["first entry", "second entry"]);

    let mut logs = APP_LOGS.lock().expect("log buffer lock");
    *logs = original;
}