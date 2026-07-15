use crate::core::storage::mod_builds::{ModBuild, MOD_BUILDS};

#[tauri::command]
pub async fn get_all_mod_builds() -> Result<Vec<ModBuild>, String> {
    let builds = MOD_BUILDS.lock().map_err(|e| e.to_string())?;
    Ok(builds.get_all().to_vec())
}

#[tauri::command]
pub async fn get_mod_build(id: String) -> Result<Option<ModBuild>, String> {
    let builds = MOD_BUILDS.lock().map_err(|e| e.to_string())?;
    Ok(builds.get(&id).cloned())
}

#[tauri::command]
pub async fn create_mod_build(build: ModBuild) -> Result<ModBuild, String> {
    let mut builds = MOD_BUILDS.lock().map_err(|e| e.to_string())?;
    let created = build.clone();
    builds.create(build);
    Ok(created)
}

#[tauri::command]
pub async fn update_mod_build(build: ModBuild) -> Result<ModBuild, String> {
    let mut builds = MOD_BUILDS.lock().map_err(|e| e.to_string())?;
    let updated = build.clone();
    builds.update(build);
    Ok(updated)
}

#[tauri::command]
pub async fn delete_mod_build(id: String) -> Result<(), String> {
    let mut builds = MOD_BUILDS.lock().map_err(|e| e.to_string())?;
    builds.delete(&id);
    Ok(())
}

#[tauri::command]
pub async fn export_mod_build(id: String) -> Result<ModBuild, String> {
    let builds = MOD_BUILDS.lock().map_err(|e| e.to_string())?;
    builds
        .get(&id)
        .cloned()
        .ok_or_else(|| "Build not found".to_string())
}

#[tauri::command]
pub async fn import_mod_build(build: ModBuild) -> Result<ModBuild, String> {
    let mut builds = MOD_BUILDS.lock().map_err(|e| e.to_string())?;
    let imported = build.clone();
    builds.create(build);
    Ok(imported)
}
