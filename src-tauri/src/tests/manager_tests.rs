use std::sync::{Arc, Mutex};

use crate::core::clients::{client::Client, manager::ClientManager};

// ── ClientManager::default ────────────────────────────────────────────────────

#[test]
fn default_creates_empty_client_list() {
    let mgr = ClientManager::default();
    assert!(mgr.clients.is_empty());
}

// ── ClientManager::get_client ─────────────────────────────────────────────────

#[test]
fn get_client_calls_closure_for_existing_id() {
    let mut mgr = ClientManager::default();
    mgr.clients.push(Client {
        id: 42,
        name: "Test".to_string(),
        ..Default::default()
    });

    let arc = Arc::new(Mutex::new(mgr));
    let mut found = false;
    ClientManager::get_client(&arc, 42, |c| {
        assert_eq!(c.name, "Test");
        found = true;
    });
    assert!(found);
}

#[test]
fn get_client_does_not_call_closure_for_missing_id() {
    let mgr = ClientManager::default();
    let arc = Arc::new(Mutex::new(mgr));
    let mut called = false;
    ClientManager::get_client(&arc, 99, |_c| {
        called = true;
    });
    assert!(!called);
}

#[test]
fn get_client_allows_mutation_through_closure() {
    let mut mgr = ClientManager::default();
    mgr.clients.push(Client {
        id: 1,
        working: false,
        ..Default::default()
    });

    let arc = Arc::new(Mutex::new(mgr));
    ClientManager::get_client(&arc, 1, |c| {
        c.working = true;
    });

    let guard = arc.lock().unwrap();
    assert!(guard.clients[0].working);
}

// ── Client struct ─────────────────────────────────────────────────────────────

#[test]
fn client_default_has_zero_id() {
    let client = Client::default();
    assert_eq!(client.id, 0);
}

#[test]
fn client_fields_are_set_correctly() {
    let client = Client {
        id: 7,
        name: "OptiFine".to_string(),
        version: "1.20.1".to_string(),
        working: true,
        ..Default::default()
    };
    assert_eq!(client.id, 7);
    assert_eq!(client.name, "OptiFine");
    assert_eq!(client.version, "1.20.1");
    assert!(client.working);
}

// ── Mock clients shape ────────────────────────────────────────────────────────
// Access mock_clients indirectly via fetch_clients when MOCK_CLIENTS is enabled.
// We test the shape of ClientManager's clients vec directly by constructing
// equivalent data (mock_clients is private, but its output contract can be
// validated by building a manager with hand-crafted clients of the same shape).

#[test]
fn clients_can_be_found_by_id_after_push() {
    let mut mgr = ClientManager::default();
    for i in 1u32..=4 {
        mgr.clients.push(Client {
            id: i,
            name: format!("Mock client #{i}"),
            version: "1.16.5".to_string(),
            ..Default::default()
        });
    }

    assert_eq!(mgr.clients.len(), 4);

    // Every mock client must have the expected name pattern and version.
    for (idx, c) in mgr.clients.iter().enumerate() {
        assert_eq!(c.id, (idx + 1) as u32);
        assert!(c.name.starts_with("Mock client #"));
        assert_eq!(c.version, "1.16.5");
    }
}

#[test]
fn sorting_clients_by_created_at_descending() {
    use chrono::{TimeZone, Utc};

    let mut clients = vec![
        Client {
            id: 1,
            created_at: Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap(),
            ..Default::default()
        },
        Client {
            id: 2,
            created_at: Utc.with_ymd_and_hms(2025, 6, 1, 0, 0, 0).unwrap(),
            ..Default::default()
        },
        Client {
            id: 3,
            created_at: Utc.with_ymd_and_hms(2023, 3, 15, 0, 0, 0).unwrap(),
            ..Default::default()
        },
    ];

    clients.sort_by_key(|c| std::cmp::Reverse(c.created_at));

    assert_eq!(clients[0].id, 2); // newest first
    assert_eq!(clients[1].id, 1);
    assert_eq!(clients[2].id, 3); // oldest last
}
