use crate::core::storage::mod_builds::ModBuild;
use crate::AppState;
use tauri::State;

#[tauri::command]
pub async fn get_all_mod_builds(state: State<'_, AppState>) -> Result<Vec<ModBuild>, String> {
    let builds = state.mod_builds();
    Ok(builds.get_all().to_vec())
}

#[tauri::command]
pub async fn get_mod_build(
    id: String,
    state: State<'_, AppState>,
) -> Result<Option<ModBuild>, String> {
    let builds = state.mod_builds();
    Ok(builds.get(&id).cloned())
}

#[tauri::command]
pub async fn create_mod_build(
    build: ModBuild,
    state: State<'_, AppState>,
) -> Result<ModBuild, String> {
    let mut builds = state.mod_builds();
    let created = build.clone();
    builds.create(build);
    Ok(created)
}

#[tauri::command]
pub async fn update_mod_build(
    build: ModBuild,
    state: State<'_, AppState>,
) -> Result<ModBuild, String> {
    let mut builds = state.mod_builds();
    let updated = build.clone();
    builds.update(build);
    Ok(updated)
}

#[tauri::command]
pub async fn delete_mod_build(id: String, state: State<'_, AppState>) -> Result<(), String> {
    let mut builds = state.mod_builds();
    builds.delete(&id);
    Ok(())
}

#[tauri::command]
pub async fn export_mod_build(id: String, state: State<'_, AppState>) -> Result<ModBuild, String> {
    let builds = state.mod_builds();
    builds
        .get(&id)
        .cloned()
        .ok_or_else(|| "Build not found".to_string())
}

#[tauri::command]
pub async fn import_mod_build(
    build: ModBuild,
    state: State<'_, AppState>,
) -> Result<ModBuild, String> {
    let mut builds = state.mod_builds();
    let imported = build.clone();
    builds.create(build);
    Ok(imported)
}
