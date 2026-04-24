use std::path::{Path, MAIN_SEPARATOR};

use tauri::AppHandle;

use super::{
    collect_jars_recursive, dir_has_any_jars, sanitize_version_for_paths, Client, ClientType,
    REQUIREMENTS_DOWNLOADING, REQUIREMENTS_SEMAPHORE,
};
use crate::core::storage::{
    data::{Data, DATA},
    settings::SETTINGS,
};
use crate::core::utils::globals::{
    AGENT_OVERLAY_FOLDER, ASSETS_FABRIC_FOLDER, ASSETS_FABRIC_ZIP, ASSETS_FOLDER, ASSETS_ZIP,
    FABRIC_DEPS_URL, FORGE_DEPS_URL, IS_AARCH64, IS_LINUX, IS_MACOS, IS_WINDOWS, JDK21_FOLDER,
    JDK8_FOLDER, LIBRARIES_FABRIC_FOLDER, LIBRARIES_FABRIC_ZIP, LIBRARIES_FOLDER,
    LIBRARIES_LEGACY_FOLDER, LIBRARIES_LEGACY_ZIP, LIBRARIES_ZIP,
    MINECRAFT_VERSIONS_FOLDER, MODS_FOLDER, NATIVES_FOLDER, NATIVES_LEGACY_FOLDER,
    NATIVES_LEGACY_LINUX_FOLDER, NATIVES_LEGACY_LINUX_ZIP, NATIVES_LEGACY_ZIP, NATIVES_LINUX_FOLDER,
    NATIVES_LINUX_ZIP, NATIVES_MACOS_ARM64_FOLDER, NATIVES_MACOS_ARM64_ZIP, NATIVES_MACOS_FOLDER,
    NATIVES_MACOS_ZIP, NATIVES_ZIP, PATH_SEPARATOR, CUSTOM_CLIENTS_FOLDER,
};
use crate::core::utils::{hashing::calculate_md5_hash, helpers::emit_to_main_window};
use crate::{log_debug, log_error, log_info, log_warn};

#[derive(serde::Deserialize)]
struct ModrinthFile {
    url: String,
    filename: String,
}

#[derive(serde::Deserialize)]
struct ModrinthVersion {
    files: Vec<ModrinthFile>,
}

impl Client {
    pub async fn download(
        &self,
        manager: &std::sync::Arc<std::sync::Mutex<crate::core::clients::manager::ClientManager>>,
    ) -> Result<(), String> {
        log_debug!(
            "Starting download for client '{}' (type: {:?})",
            self.name,
            self.client_type
        );

        let download_result = match self.client_type {
            ClientType::Forge => {
                let client_base = Data::get_filename(&self.filename);
                let mods_folder = format!("{client_base}{MAIN_SEPARATOR}{MODS_FOLDER}");
                let safe_ver = sanitize_version_for_paths(&self.version);
                let forge_jar_name =
                    format!("misc/{MINECRAFT_VERSIONS_FOLDER}/forge_{}.jar", safe_ver);

                let remote_path = if self.filename.contains('/') {
                    let parts: Vec<&str> = self.filename.split('/').collect();
                    format!("clients/{}/jars/{}", parts[0], parts[1])
                } else {
                    format!("clients/forge/jars/{}", self.filename)
                };

                DATA.download_to_folder(&remote_path, &mods_folder)
                    .await
                    .map_err(|e| format!("Forge Mod download failed: {e}"))?;

                DATA.download_to_folder(&forge_jar_name, MINECRAFT_VERSIONS_FOLDER)
                    .await
                    .map_err(|e| format!("Forge MC Jar download failed: {e}"))
            }
            ClientType::Fabric => {
                let client_base = Data::get_filename(&self.filename);
                let mods_folder = format!("{client_base}{MAIN_SEPARATOR}{MODS_FOLDER}");
                let remote_path = if self.filename.contains('/') {
                    let parts: Vec<&str> = self.filename.split('/').collect();
                    format!("clients/{}/jars/{}", parts[0], parts[1])
                } else {
                    format!("clients/fabric/jars/{}", self.filename)
                };
                DATA.download_to_folder(&remote_path, &mods_folder).await
            }
            ClientType::Default => {
                let remote_path = if self.filename.contains('/') {
                    let parts: Vec<&str> = self.filename.split('/').collect();
                    format!("clients/{}/jars/{}", parts[0], parts[1])
                } else {
                    format!("clients/vanilla/jars/{}", self.filename)
                };
                DATA.download(&remote_path).await
            }
        };

        if let Err(e) = download_result {
            log_error!("Download failed for {}: {}", self.name, e);
            self.mark_installed(manager, false);
            return Err(e);
        }

        if let Err(e) = self.verify_hash().await {
            self.mark_installed(manager, false);
            return Err(e);
        }

        if let Err(e) = self.download_fabric_mods().await {
            self.mark_installed(manager, false);
            return Err(e);
        }

        log_debug!("Client '{}' download procedure finished", self.name);

        Ok(())
    }

    async fn verify_hash(&self) -> Result<(), String> {
        let verify = SETTINGS.lock().map(|s| s.hash_verify.value).unwrap_or(true);

        if !verify {
            return Ok(());
        }

        let (_, jar_path) = self.get_launch_paths()?;
        if !jar_path.exists() {
            return Err("File not found for hash verification".to_string());
        }

        let calculated = tokio::task::spawn_blocking(move || calculate_md5_hash(&jar_path))
            .await
            .map_err(|e| e.to_string())??;

        if calculated != self.md5_hash {
            return Err(format!(
                "Hash mismatch. Expected: {}, Got: {}",
                self.md5_hash, calculated
            ));
        }

        Ok(())
    }

    async fn download_dependency(
        &self,
        remote_path: &str,
        local_folder_rel: &str,
        local_file_abs: &Path,
        expected_md5: Option<&String>,
        name_for_logs: &str,
    ) -> Result<(), String> {
        let mut need_download = true;

        if local_file_abs.exists() {
            if let Some(expected) = expected_md5 {
                let ok = Data::verify_file_md5(local_file_abs, expected)
                    .await
                    .map_err(|e| format!("Failed to verify MD5 for {name_for_logs}: {e}"))?;
                if ok {
                    need_download = false;
                } else {
                    log_warn!(
                        "MD5 mismatch for {}. Expected: {}. Redownloading...",
                        name_for_logs,
                        expected
                    );
                    let _ = std::fs::remove_file(local_file_abs);
                    need_download = true;
                }
            } else {
                need_download = false;
            }
        }

        if need_download {
            log_info!("Downloading dependency: {}", remote_path);
            DATA.download_to_folder(remote_path, local_folder_rel)
                .await
                .map_err(|e| format!("Failed to download dependency {name_for_logs}: {e}"))?;

            if let Some(expected) = expected_md5 {
                let ok = Data::verify_file_md5(local_file_abs, expected)
                    .await
                    .map_err(|e| format!("Failed to verify MD5 for {name_for_logs}: {e}"))?;
                if !ok {
                    log_warn!(
                        "MD5 mismatch after download for {}. Retrying...",
                        name_for_logs
                    );
                    let _ = std::fs::remove_file(local_file_abs);
                    DATA.download_to_folder(remote_path, local_folder_rel)
                        .await
                        .map_err(|e| {
                            format!("Failed to redownload dependency {name_for_logs}: {e}")
                        })?;

                    let ok2 = Data::verify_file_md5(local_file_abs, expected)
                        .await
                        .map_err(|e| {
                            format!("Failed to verify MD5 for {name_for_logs} after retry: {e}")
                        })?;
                    if !ok2 {
                        let _ = std::fs::remove_file(local_file_abs);
                        return Err(format!(
                            "MD5 mismatch for {name_for_logs} after retry. Aborting."
                        ));
                    }
                }
            }
        }
        Ok(())
    }

    async fn download_fabric_mods(&self) -> Result<(), String> {
        if self.client_type != ClientType::Fabric {
            return Ok(());
        }

        self.ensure_fabric_api().await?;

        if let Some(mods) = &self.dependencies {
            let client_base = Data::get_filename(&self.filename);
            let mods_folder_rel = format!("{client_base}{MAIN_SEPARATOR}{MODS_FOLDER}");
            let mods_folder_abs = DATA
                .root_dir
                .lock()
                .unwrap()
                .join(&client_base)
                .join(MODS_FOLDER);

            for req in mods {
                let name = req.name.clone();
                let mod_basename = name.trim_end_matches(".jar");
                let dest = mods_folder_abs.join(format!("{mod_basename}.jar"));
                let remote_path = format!("{FABRIC_DEPS_URL}/{mod_basename}.jar");

                self.download_dependency(
                    &remote_path,
                    &mods_folder_rel,
                    &dest,
                    req.md5_hash.as_ref(),
                    mod_basename,
                )
                .await?;
            }
        }
        Ok(())
    }

    async fn ensure_fabric_api(&self) -> Result<(), String> {
        if self.client_type != ClientType::Fabric {
            return Ok(());
        }

        let client_folder = self.get_client_folder()?;
        let mods_folder = client_folder.join(MODS_FOLDER);

        if !mods_folder.exists() {
            std::fs::create_dir_all(&mods_folder)
                .map_err(|e| format!("Failed to create mods folder: {e}"))?;
        }

        if let Ok(entries) = std::fs::read_dir(&mods_folder) {
            for entry in entries.flatten() {
                if let Some(name) = entry.file_name().to_str() {
                    if name.to_lowercase().contains("fabric-api") {
                        log_debug!("Fabric API already present in mods folder: {}", name);
                        return Ok(());
                    }
                }
            }
        }

        log_info!("Fetching Fabric API for version {} from Modrinth...", self.version);

        let url = format!(
            "https://api.modrinth.com/v2/project/P7dR8mSH/version?game_versions=[\"{}\"]&loaders=[\"fabric\"]",
            self.version
        );

        let client = reqwest::Client::new();
        let response = client
            .get(&url)
            .header("User-Agent", "CollapseLauncher-Reborn (github.com/dest4590/CollapseLoader)")
            .send()
            .await
            .map_err(|e| format!("Failed to fetch Fabric API info from Modrinth: {e}"))?;

        if !response.status().is_success() {
            return Err(format!("Modrinth API returned error: {}", response.status()));
        }

        let versions: Vec<ModrinthVersion> = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse Modrinth API response: {e}"))?;

        let best_version = versions.first().ok_or_else(|| {
            format!("No Fabric API version found on Modrinth for Minecraft {}", self.version)
        })?;

        let file = best_version.files.first().ok_or_else(|| {
            format!("No files found for the latest Fabric API on Modrinth")
        })?;

        let api_path = mods_folder.join(&file.filename);
        
        log_info!("Downloading Fabric API: {}", file.filename);
        
        let mods_folder_rel = if self.meta.is_custom {
            format!("{}/{}/{}", CUSTOM_CLIENTS_FOLDER, self.name, MODS_FOLDER)
        } else {
            let client_base = Data::get_filename(&self.filename);
            format!("{client_base}{MAIN_SEPARATOR}{MODS_FOLDER}")
        };

        self.download_dependency(
            &file.url,
            &mods_folder_rel,
            &api_path,
            None,
            &file.filename,
        ).await?;

        Ok(())
    }

    async fn download_forge_mods(&self) -> Result<(), String> {
        if self.client_type != ClientType::Forge {
            return Ok(());
        }

        if let Some(mods) = &self.dependencies {
            let client_base = Data::get_filename(&self.filename);
            let mods_folder_rel = format!("{client_base}{MAIN_SEPARATOR}{MODS_FOLDER}");
            let mods_folder_abs = DATA
                .root_dir
                .lock()
                .unwrap()
                .join(&client_base)
                .join(MODS_FOLDER);

            for req in mods {
                let name = req.name.clone();
                let mod_basename = name.trim_end_matches(".jar");
                let dest = mods_folder_abs.join(format!("{mod_basename}.jar"));
                let remote_path = format!("{FORGE_DEPS_URL}/{mod_basename}.jar");

                self.download_dependency(
                    &remote_path,
                    &mods_folder_rel,
                    &dest,
                    req.md5_hash.as_ref(),
                    mod_basename,
                )
                .await?;
            }
        }
        Ok(())
    }

    async fn download_type_mods(&self) -> Result<(), String> {
        match self.client_type {
            ClientType::Fabric => self.download_fabric_mods().await,
            ClientType::Forge => self.download_forge_mods().await,
            ClientType::Default => Ok(()),
        }
    }

    fn verify_or_queue_requirement(files_to_download: &mut Vec<String>, folder: &str, zip: &str) {
        if folder == ASSETS_FOLDER
            || folder == ASSETS_FABRIC_FOLDER
            || folder == JDK8_FOLDER
            || folder == JDK21_FOLDER
        {
            let path = DATA.root_dir.lock().unwrap().join(folder);
            if !path.exists() {
                log_info!("Folder '{}' missing. Queuing {} for download.", folder, zip);
                files_to_download.push(zip.to_string());
            }
            return;
        }

        if !DATA.verify_folder_integrity(folder) {
            log_info!(
                "Integrity check failed for '{}'. Wiping folder for clean redownload.",
                folder
            );
            let path = DATA.root_dir.lock().unwrap().join(folder);
            if path.exists() {
                let _ = std::fs::remove_dir_all(&path);
            }
            files_to_download.push(zip.to_string());
        }
    }

    fn resolve_assets_requirement(&self) -> (&'static str, &'static str) {
        if self.client_type == ClientType::Fabric {
            (ASSETS_FABRIC_FOLDER, ASSETS_FABRIC_ZIP)
        } else {
            (ASSETS_FOLDER, ASSETS_ZIP)
        }
    }

    fn resolve_libraries_and_natives_requirement(
        &self,
    ) -> (&'static str, &'static str, &'static str, &'static str) {
        if self.is_legacy_client() {
            (
                LIBRARIES_LEGACY_ZIP,
                LIBRARIES_LEGACY_FOLDER,
                NATIVES_LEGACY_ZIP,
                NATIVES_LEGACY_FOLDER,
            )
        } else {
            (LIBRARIES_ZIP, LIBRARIES_FOLDER, NATIVES_ZIP, NATIVES_FOLDER)
        }
    }

    fn resolve_platform_natives_requirement(
        &self,
        natives_zip: &'static str,
        natives_folder: &'static str,
    ) -> (&'static str, &'static str) {
        if IS_LINUX {
            if self.is_legacy_client() {
                (NATIVES_LEGACY_LINUX_ZIP, NATIVES_LEGACY_LINUX_FOLDER)
            } else {
                (NATIVES_LINUX_ZIP, NATIVES_LINUX_FOLDER)
            }
        } else if IS_MACOS {
            if IS_AARCH64 {
                (NATIVES_MACOS_ARM64_ZIP, NATIVES_MACOS_ARM64_FOLDER)
            } else {
                (NATIVES_MACOS_ZIP, NATIVES_MACOS_FOLDER)
            }
        } else {
            (natives_zip, natives_folder)
        }
    }

    fn collect_missing_requirements(&self) -> Vec<String> {
        let mut files_to_download = Vec::new();

        Self::verify_or_queue_requirement(
            &mut files_to_download,
            self.jdk_folder_name(),
            &self.jdk_zip_name(),
        );

        let (assets_folder, assets_zip) = self.resolve_assets_requirement();
        Self::verify_or_queue_requirement(&mut files_to_download, assets_folder, assets_zip);

        let (libs_zip, libs_folder, natives_zip, natives_folder) =
            self.resolve_libraries_and_natives_requirement();
        Self::verify_or_queue_requirement(&mut files_to_download, libs_folder, libs_zip);

        let (actual_natives_zip, actual_natives_folder) =
            self.resolve_platform_natives_requirement(natives_zip, natives_folder);
        Self::verify_or_queue_requirement(
            &mut files_to_download,
            actual_natives_folder,
            actual_natives_zip,
        );

        files_to_download
    }

    fn minecraft_remote_path(&self) -> Option<String> {
        match self.client_type {
            ClientType::Fabric => Some(format!(
                "misc/{MINECRAFT_VERSIONS_FOLDER}/fabric_{}.jar",
                self.version
            )),
            ClientType::Forge => Some(format!(
                "misc/{MINECRAFT_VERSIONS_FOLDER}/forge_{}.jar",
                self.version
            )),
            ClientType::Default => None,
        }
    }

    async fn ensure_minecraft_version_jar(&self) -> Result<(), String> {
        let Some(remote) = self.minecraft_remote_path() else {
            return Ok(());
        };

        let dest_filename = remote.rsplit('/').next().unwrap_or(&remote);
        let local_path = DATA
            .root_dir
            .lock()
            .unwrap()
            .join(MINECRAFT_VERSIONS_FOLDER)
            .join(dest_filename);

        self.download_dependency(
            &remote,
            MINECRAFT_VERSIONS_FOLDER,
            &local_path,
            None,
            dest_filename,
        )
        .await
    }

    pub async fn download_requirements(&self, app_handle: &AppHandle) -> Result<(), String> {
        let _permit = REQUIREMENTS_SEMAPHORE
            .acquire()
            .await
            .map_err(|_| "Failed to acquire requirement lock".to_string())?;

        let files_to_download = self.collect_missing_requirements();

        if !files_to_download.is_empty() {
            self.batch_download_requirements(app_handle, files_to_download)
                .await?;
        }

        if self.client_type == ClientType::Fabric {
            self.ensure_fabric_libraries().await?;
        }

        self.ensure_minecraft_version_jar().await?;
        self.download_type_mods().await?;

        Ok(())
    }

    async fn batch_download_requirements(
        &self,
        app_handle: &AppHandle,
        files: Vec<String>,
    ) -> Result<(), String> {
        {
            let mut downloading = REQUIREMENTS_DOWNLOADING.lock().unwrap();
            *downloading = true;
        }
        emit_to_main_window(app_handle, "requirements-status", true);

        for file in files {
            log_info!("Downloading requirement: {}", file);
            DATA.download(&file)
                .await
                .map_err(|e| format!("Failed {file}: {e}"))?;

            if (IS_LINUX || IS_MACOS) && file.starts_with(self.jdk_folder_name()) {
                self.fix_java_permissions();
            }
        }

        self.download_type_mods().await?;

        if self.client_type == ClientType::Fabric {
            self.clean_fabric_libraries();
        }

        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

        {
            let mut downloading = REQUIREMENTS_DOWNLOADING.lock().unwrap();
            *downloading = false;
        }
        emit_to_main_window(app_handle, "requirements-status", false);

        Ok(())
    }

    fn fix_java_permissions(&self) {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let bin_dir = DATA
                .root_dir
                .lock()
                .unwrap()
                .join(self.jdk_folder_name())
                .join("bin");
            if bin_dir.exists() {
                if let Ok(entries) = std::fs::read_dir(&bin_dir) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.is_file() {
                            if let Ok(mut perms) = std::fs::metadata(&path).map(|m| m.permissions())
                            {
                                perms.set_mode(0o755);
                                if let Err(e) = std::fs::set_permissions(&path, perms) {
                                    log_warn!(
                                        "Failed to set exec perm on {}: {}",
                                        path.display(),
                                        e
                                    );
                                } else {
                                    log_debug!("Set exec perm on {}", path.display());
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    fn clean_fabric_libraries(&self) {
        let fabric_libs_dir = DATA
            .root_dir
            .lock()
            .unwrap()
            .join(LIBRARIES_FABRIC_FOLDER);

        if !fabric_libs_dir.exists() {
            return;
        }

        Self::clean_jars_in_dir_recursive(&fabric_libs_dir, 0, 5);
    }

    fn clean_jars_in_dir_recursive(dir: &std::path::Path, depth: usize, max_depth: usize) {
        if depth >= max_depth {
            return;
        }
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                Self::clean_jars_in_dir_recursive(&path, depth + 1, max_depth);
                continue;
            }
            let Some(ext) = path.extension() else { continue; };
            if !ext.eq_ignore_ascii_case("jar") {
                continue;
            }
            let filename = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("");

            let is_empty = std::fs::metadata(&path)
                .map(|m| m.len() == 0)
                .unwrap_or(false);

            let is_foreign_native = filename.contains("natives") && ({
                if IS_WINDOWS {
                    filename.contains("-linux") || filename.contains("-macos")
                } else if IS_LINUX {
                    filename.contains("-windows") || filename.contains("-macos")
                } else if IS_MACOS {
                    filename.contains("-windows") || filename.contains("-linux")
                } else {
                    false
                }
            });

            if is_empty || is_foreign_native {
                log_info!("Removing incompatible library: {}", filename);
                let _ = std::fs::remove_file(&path);
            }
        }
    }

    async fn ensure_fabric_libraries(&self) -> Result<(), String> {
        let common_dir = DATA.root_dir.lock().unwrap().join(LIBRARIES_FABRIC_FOLDER);

        if !dir_has_any_jars(&common_dir, true) {
            log_info!("Downloading common Fabric libraries");
            DATA.download(LIBRARIES_FABRIC_ZIP).await?;
        }

        let versioned_zip = format!(
            "misc/{}/{}.zip",
            LIBRARIES_FABRIC_FOLDER,
            sanitize_version_for_paths(&self.version)
        );

        let versioned_dir = DATA.root_dir.lock().unwrap().join(
            versioned_zip
                .strip_prefix("misc/")
                .unwrap_or(&versioned_zip)
                .replace(".zip", ""),
        );

        if !dir_has_any_jars(&versioned_dir, false) {
            log_info!("Downloading versioned Fabric libraries: {}", versioned_zip);
            DATA.download(&versioned_zip).await?;
        }

        self.ensure_slf4j().await?;

        Ok(())
    }

    async fn ensure_slf4j(&self) -> Result<(), String> {
        let fabric_libs_dir = DATA
            .root_dir
            .lock()
            .unwrap()
            .join(LIBRARIES_FABRIC_FOLDER);

        let already_present = {
            let mut found = false;
            if let Ok(entries) = std::fs::read_dir(&fabric_libs_dir) {
                for entry in entries.flatten() {
                    let name = entry
                        .file_name()
                        .to_string_lossy()
                        .to_lowercase();
                    if name.contains("slf4j-api") && name.ends_with(".jar") {
                        let size = std::fs::metadata(entry.path())
                            .map(|m| m.len())
                            .unwrap_or(0);
                        if size > 0 {
                            found = true;
                            break;
                        }
                    }
                }
            }
            found
        };

        if already_present {
            log_debug!("SLF4J already present in libraries-fabric, skipping download");
            return Ok(());
        }

        log_info!("SLF4J not found, downloading from Maven Central...");

        let slf4j_url = "https://repo1.maven.org/maven2/org/slf4j/slf4j-api/2.0.9/slf4j-api-2.0.9.jar";
        let dest = fabric_libs_dir.join("slf4j-api-2.0.9.jar");

        let client = reqwest::Client::new();
        let response = client
            .get(slf4j_url)
            .header("User-Agent", "CollapseLoader-Reborn")
            .send()
            .await
            .map_err(|e| format!("Failed to download slf4j-api: {e}"))?;

        if !response.status().is_success() {
            return Err(format!(
                "Maven Central returned {} for slf4j-api",
                response.status()
            ));
        }

        let bytes = response
            .bytes()
            .await
            .map_err(|e| format!("Failed to read slf4j-api response: {e}"))?;

        std::fs::write(&dest, &bytes)
            .map_err(|e| format!("Failed to write slf4j-api: {e}"))?;

        log_info!("Downloaded slf4j-api-2.0.9.jar ({} bytes)", bytes.len());
        Ok(())
    }

    pub async fn ensure_java_available(&self, app_handle: &AppHandle) -> Result<(), String> {
        if self.java_executable_path().exists() {
            return Ok(());
        }

        log_warn!("Java executable missing. Redownloading requirements...");

        let jdk_dir = DATA.root_dir.lock().unwrap().join(self.jdk_folder_name());
        let _ = tokio::fs::remove_dir_all(jdk_dir).await;
        let jdk_zip = DATA.root_dir.lock().unwrap().join(self.jdk_zip_name());
        let _ = tokio::fs::remove_file(jdk_zip).await;

        self.download_requirements(app_handle).await?;

        if !self.java_executable_path().exists() {
            return Err("Java still missing after download".to_string());
        }
        Ok(())
    }

    pub(super) fn build_classpath(&self) -> Result<String, String> {
        let (_, client_jar) = self.get_launch_paths()?;
        let agent_overlay = DATA.root_dir.lock().unwrap().join(AGENT_OVERLAY_FOLDER);

        let mut cp_parts = Vec::new();

        match self.client_type {
            ClientType::Fabric => {
                cp_parts.push(self.get_minecraft_jar_path());

                let safe_ver = sanitize_version_for_paths(&self.version);
                let fabric_libs_root = DATA.root_dir.lock().unwrap().join(LIBRARIES_FABRIC_FOLDER);

                let v_libs = fabric_libs_root.join(&safe_ver);
                cp_parts.extend(collect_jars_recursive(&v_libs, false));

                cp_parts.extend(collect_jars_recursive(&fabric_libs_root, true));

                let common_mc_libs = DATA.root_dir.lock().unwrap().join(LIBRARIES_FOLDER);
                let mc_jars = collect_jars_recursive(&common_mc_libs, false);
                for jar in mc_jars {
                    if let Some(filename) = jar.file_name().and_then(|n| n.to_str()) {
                        if !filename.to_lowercase().contains("lwjgl") {
                            cp_parts.push(jar);
                        }
                    }
                }

                cp_parts.push(client_jar);
            }
            ClientType::Forge => {
                cp_parts.push(self.get_minecraft_jar_path());
                let libs = DATA.root_dir.lock().unwrap().join(LIBRARIES_LEGACY_FOLDER);
                cp_parts.extend(collect_jars_recursive(&libs, false));

                cp_parts.push(client_jar);
            }
            ClientType::Default => {
                let libs = if self.is_legacy_client() {
                    DATA.root_dir.lock().unwrap().join(LIBRARIES_LEGACY_FOLDER)
                } else {
                    DATA.root_dir.lock().unwrap().join(LIBRARIES_FOLDER)
                };

                return Ok(format!(
                    "{}{}*{}{}{}{}",
                    libs.display(),
                    MAIN_SEPARATOR,
                    PATH_SEPARATOR,
                    client_jar.display(),
                    PATH_SEPARATOR,
                    agent_overlay.display()
                ));
            }
        }

        cp_parts.push(agent_overlay);

        log_info!("Built classpath with {} elements", cp_parts.len());
        if cp_parts.len() < 10 {
             for (i, p) in cp_parts.iter().enumerate() {
                 log_debug!("CP [{}]: {:?}", i, p);
             }
        }

        Ok(cp_parts
            .iter()
            .map(|p: &std::path::PathBuf| p.display().to_string())
            .collect::<Vec<_>>()
            .join(PATH_SEPARATOR))
    }
}
