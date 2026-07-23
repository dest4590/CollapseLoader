use crate::AppState;
use tauri::{AppHandle, State};

#[tauri::command]
pub fn update_tray_menu(app: AppHandle, state: State<'_, AppState>) -> Result<(), String> {
    use tauri::menu::PredefinedMenuItem;
    use tauri::menu::{Menu, MenuItem};

    #[allow(clippy::type_complexity)]
    let (fav_clients, popular_clients): (Vec<(u32, String)>, Vec<(u32, String)>) = state
        .clients
        .manager
        .lock()
        .map(|m| {
            let favorites = state.favorites().favorites.clone();

            let installed: Vec<_> = m
                .clients
                .iter()
                .filter(|c| c.show && c.working && c.meta.installed)
                .collect();

            let mut favs = Vec::new();
            let mut others = Vec::new();

            for c in installed {
                let ver = c
                    .version
                    .replace('_', ".")
                    .trim_start_matches('V')
                    .to_string();
                if favorites.contains(&c.id) {
                    favs.push((c.id, format!("⭐  {} {}", c.name, ver)));
                } else {
                    others.push(c);
                }
            }

            others.sort_by_key(|b| std::cmp::Reverse(b.launches));
            let popular: Vec<_> = others
                .into_iter()
                .take(10)
                .map(|c| {
                    let ver = c
                        .version
                        .replace('_', ".")
                        .trim_start_matches('V')
                        .to_string();
                    (c.id, format!("⚡  {} {}", c.name, ver))
                })
                .collect();

            (favs, popular)
        })
        .unwrap_or_default();

    let show = MenuItem::with_id(&app, "show", "▶  Open CollapseLoader", true, None::<&str>)
        .map_err(|e| e.to_string())?;
    let quit = MenuItem::with_id(&app, "quit", "✕  Quit", true, None::<&str>)
        .map_err(|e| e.to_string())?;

    let sep1 = PredefinedMenuItem::separator(&app).map_err(|e| e.to_string())?;
    let sep2 = PredefinedMenuItem::separator(&app).map_err(|e| e.to_string())?;
    let sep3 = PredefinedMenuItem::separator(&app).map_err(|e| e.to_string())?;

    let fav_items: Vec<MenuItem<tauri::Wry>> = fav_clients
        .iter()
        .map(|(id, label)| {
            MenuItem::with_id(
                &app,
                format!("launch_{id}"),
                label.as_str(),
                true,
                None::<&str>,
            )
            .expect("Failed to create client menu item")
        })
        .collect();

    let popular_items: Vec<MenuItem<tauri::Wry>> = popular_clients
        .iter()
        .map(|(id, label)| {
            MenuItem::with_id(
                &app,
                format!("launch_{id}"),
                label.as_str(),
                true,
                None::<&str>,
            )
            .expect("Failed to create client menu item")
        })
        .collect();

    let fav_header = MenuItem::with_id(
        &app,
        "_fav_header",
        "── Favorited clients ──",
        false,
        None::<&str>,
    )
    .map_err(|e| e.to_string())?;

    let popular_header = MenuItem::with_id(
        &app,
        "_popular_header",
        "── Popular clients ──",
        false,
        None::<&str>,
    )
    .map_err(|e| e.to_string())?;

    let mut item_refs: Vec<&dyn tauri::menu::IsMenuItem<tauri::Wry>> = vec![&show, &sep1];

    if fav_items.is_empty() && popular_items.is_empty() {
        item_refs = vec![&show, &sep1, &quit];
    } else {
        if !fav_items.is_empty() {
            item_refs.push(&fav_header);
            for item in &fav_items {
                item_refs.push(item);
            }
            if !popular_items.is_empty() {
                item_refs.push(&sep2);
            }
        }

        if !popular_items.is_empty() {
            item_refs.push(&popular_header);
            for item in &popular_items {
                item_refs.push(item);
            }
        }
        item_refs.push(&sep3);
        item_refs.push(&quit);
    }

    let new_menu = Menu::with_items(&app, &item_refs).map_err(|e| e.to_string())?;

    if let Some(tray) = app.tray_by_id("0").or_else(|| app.tray_by_id("main")) {
        tray.set_menu(Some(new_menu)).map_err(|e| e.to_string())?;
    }

    Ok(())
}

pub fn refresh_tray_menu(state: State<'_, AppState>) {
    if let Some(app) = crate::core::storage::data::APP_HANDLE
        .lock()
        .unwrap()
        .clone()
    {
        let _ = update_tray_menu(app, state);
    }
}
