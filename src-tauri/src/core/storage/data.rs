use crate::core::network::downloader::download_file;
use crate::core::network::servers::SERVERS;
use crate::core::storage::settings::SETTINGS;
use crate::core::utils::archive::unzip;
use crate::core::utils::fs as fs_utils;
use crate::core::utils::globals::{
    ASSETS_FABRIC_FOLDER, ASSETS_FOLDER, JDK21_FOLDER, JDK8_FOLDER, LIBRARIES_FABRIC_FOLDER,
    LIBRARIES_FOLDER, LIBRARIES_LEGACY_FOLDER, MINECRAFT_VERSIONS_FOLDER, NATIVES_FABRIC_FOLDER,
    NATIVES_FOLDER, NATIVES_LEGACY_FOLDER, ROOT_DIR,
};
use crate::core::utils::helpers::emit_to_main_window;
use crate::{log_debug, log_error, log_info, log_warn};
use std::fs;
use std::path::{Path, PathBuf, MAIN_SEPARATOR};
use std::sync::Mutex;
use tokio::fs as tokio_fs;
use tokio::task;

pub struct Data {
    pub root_dir: Mutex<PathBuf>,
}

pub static APP_HANDLE: std::sync::LazyLock<Mutex<Option<tauri::AppHandle>>> =
    std::sync::LazyLock::new(|| Mutex::new(None));

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum LocalFileKind {
    Zip,
    Jar,
    FabricJar,
    Other,
}

#[derive(Clone, Copy)]
struct SyncItem {
    name: &'static str,
    is_dir: bool,
}

const SYNC_ITEMS: &[SyncItem] = &[
    SyncItem {
        name: "resourcepacks",
        is_dir: true,
    },
    SyncItem {
        name: "shaderpacks",
        is_dir: true,
    },
    SyncItem {
        name: "options.txt",
        is_dir: false,
    },
    SyncItem {
        name: "optionsof.txt",
        is_dir: false,
    },
];

pub(crate) struct FileInfo {
    pub(crate) local_file: String,
    pub(crate) file_name: String,
    pub(crate) kind: LocalFileKind,
}

impl LocalFileKind {
    pub(crate) fn is_jar(self) -> bool {
        matches!(self, Self::Jar | Self::FabricJar)
    }
}

impl FileInfo {
    pub(crate) fn new(file: &str) -> Self {
        let local_file = if file.starts_with("misc/") {
            file.strip_prefix("misc/").unwrap().to_string()
        } else {
            file.rsplit(['/', '\\']).next().unwrap_or(file).to_string()
        };

        let file_name = Data::get_filename(&local_file);
        let kind = if Data::has_extension(&local_file, "zip") {
            LocalFileKind::Zip
        } else if (file.starts_with("fabric/") || file.contains("/fabric/jars/"))
            && Data::has_extension(&local_file, "jar")
        {
            LocalFileKind::FabricJar
        } else if Data::has_extension(&local_file, "jar") {
            LocalFileKind::Jar
        } else {
            LocalFileKind::Other
        };

        Self {
            local_file,
            file_name,
            kind,
        }
    }

    pub(crate) fn is_zip(&self) -> bool {
        matches!(self.kind, LocalFileKind::Zip)
    }

    pub(crate) fn is_jar(&self) -> bool {
        self.kind.is_jar()
    }

    pub(crate) fn jar_path(&self, root_dir: &Path) -> PathBuf {
        if matches!(self.kind, LocalFileKind::FabricJar) {
            root_dir
                .join(&self.file_name)
                .join("mods")
                .join(&self.local_file)
        } else {
            root_dir.join(&self.file_name).join(&self.local_file)
        }
    }

    pub(crate) fn unzip_path(&self, root_dir: &Path) -> PathBuf {
        root_dir.join(self.local_file.trim_end_matches(".zip"))
    }

    pub(crate) fn destination_path(&self, root_dir: &Path) -> PathBuf {
        match self.kind {
            LocalFileKind::FabricJar => self.jar_path(root_dir),
            LocalFileKind::Jar => root_dir.join(&self.file_name).join(&self.local_file),
            LocalFileKind::Zip | LocalFileKind::Other => root_dir.join(&self.local_file),
        }
    }
}

impl Data {
    pub fn new(root_dir: PathBuf) -> Self {
        if !root_dir.exists() {
            log_debug!(
                "Root data directory does not exist, creating: {}",
                root_dir.display()
            );
            if let Err(e) = fs::create_dir_all(&root_dir) {
                log_error!("Failed to create root directory: {}", e);
                panic!("Failed to create root directory: {}", e);
            }
            log_info!("Created root data directory: {}", root_dir.display());
        }

        Self {
            root_dir: Mutex::new(root_dir),
        }
    }

    pub fn has_extension(file_path: &str, extension: &str) -> bool {
        Path::new(file_path)
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case(extension))
    }

    fn root_dir_snapshot(&self) -> PathBuf {
        self.root_dir.lock().unwrap().clone()
    }

    fn get_local_with_root(root_dir: &Path, relative_path: &str) -> PathBuf {
        let parts: Vec<&str> = relative_path.split(|c| ['/', '\\'].contains(&c)).collect();
        let mut path = root_dir.to_path_buf();
        for part in parts {
            if part.is_empty() {
                continue;
            }
            path = path.join(part);
        }
        path
    }

    pub fn get_local(&self, relative_path: &str) -> PathBuf {
        let root_dir = self.root_dir_snapshot();
        Self::get_local_with_root(&root_dir, relative_path)
    }

    pub async fn unzip(&self, file: &str) -> Result<(), String> {
        let info = Self::resolve_local_file_info(file);
        let root_dir = self.root_dir_snapshot();
        let zip_path = Self::get_local_with_root(&root_dir, &info.local_file);
        let unzip_path = info.unzip_path(&root_dir);

        let app_handle = APP_HANDLE.lock().unwrap().clone();
        let emit_name = info.local_file.clone();

        task::spawn_blocking(move || {
            unzip(&zip_path, &unzip_path, &emit_name, app_handle.as_ref())
        })
        .await
        .map_err(|e| e.to_string())??;

        Ok(())
    }

    pub fn get_as_folder(&self, file: &str) -> PathBuf {
        let file_name = Path::new(file)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or(file);
        self.root_dir_snapshot().join(file_name)
    }

    pub fn get_as_folder_string(file: &str) -> String {
        let file_name = Path::new(file)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or(file);
        format!("{file_name}{MAIN_SEPARATOR}")
    }

    pub fn get_filename(file: &str) -> String {
        Path::new(file)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or(file)
            .to_string()
    }

    fn resolve_local_file_info(file: &str) -> FileInfo {
        FileInfo::new(file)
    }

    fn get_destination_path(root_dir: &Path, info: &FileInfo) -> PathBuf {
        info.destination_path(root_dir)
    }

    fn is_file_usable(path: &Path) -> bool {
        path.metadata()
            .map(|m| m.is_file() && m.len() > 0)
            .unwrap_or(false)
    }

    fn is_extracted_folder_usable(path: &Path) -> bool {
        let sentinel = path.join(".valid");
        path.exists()
            && path.is_dir()
            && sentinel.exists()
            && std::fs::read_dir(path).is_ok_and(|mut entries| entries.next().is_some())
    }

    pub async fn download_to_folder(&self, file: &str, dest_folder: &str) -> Result<(), String> {
        let info = Self::resolve_local_file_info(file);
        let root_dir = self.root_dir_snapshot();
        if let Some(app_handle) = APP_HANDLE.lock().unwrap().as_ref() {
            emit_to_main_window(app_handle, "download-start", &info.local_file);
        }

        let download_urls = Self::get_download_urls(file)?;

        let dest_dir = root_dir.join(dest_folder);
        if let Err(e) = tokio_fs::create_dir_all(&dest_dir).await {
            log_error!(
                "Failed to create destination folder {}: {}",
                dest_dir.display(),
                e
            );
            return Err(format!("Failed to create destination folder: {e}"));
        }

        let dest_filename = file.rsplit(['/', '\\']).next().unwrap_or(file);
        let dest_filename = dest_filename
            .replace("%2B", "+")
            .replace("%20", " ")
            .replace("%2b", "+");
        let dest_path = dest_dir.join(&dest_filename);

        let app_handle = APP_HANDLE.lock().unwrap().clone();
        download_file(
            &download_urls,
            &dest_path,
            &info.local_file,
            app_handle.as_ref(),
        )
        .await?;

        if let Some(handle) = app_handle.as_ref() {
            emit_to_main_window(handle, "download-complete", &info.local_file);
        }

        Ok(())
    }
    fn should_skip_download(&self, root_dir: &Path, info: &FileInfo) -> bool {
        match info.kind {
            LocalFileKind::Zip => Self::is_extracted_folder_usable(&info.unzip_path(root_dir)),
            LocalFileKind::Jar | LocalFileKind::FabricJar => {
                Self::is_file_usable(&info.jar_path(root_dir))
            }
            LocalFileKind::Other => false,
        }
    }

    async fn prepare_download_dirs(&self, root_dir: &Path, info: &FileInfo) -> Result<(), String> {
        if let Some(parent) = Path::new(&info.local_file).parent() {
            if !parent.as_os_str().is_empty() {
                let parent_path = root_dir.join(parent);
                if let Err(e) = tokio_fs::create_dir_all(&parent_path).await {
                    log_error!(
                        "Failed to create parent directory {}: {}",
                        parent_path.display(),
                        e
                    );
                    return Err(format!("Failed to create directory: {e}"));
                }
            }
        }

        if info.is_jar() {
            let client_dir = root_dir.join(&info.file_name);
            let target_dir = if matches!(info.kind, LocalFileKind::FabricJar) {
                client_dir.join("mods")
            } else {
                client_dir.clone()
            };

            if let Err(e) = tokio_fs::create_dir_all(&target_dir).await {
                log_error!("Failed to create directory {}: {}", target_dir.display(), e);
                return Err(if matches!(info.kind, LocalFileKind::FabricJar) {
                    format!("Failed to create mods directory: {e}")
                } else {
                    format!("Failed to create directory: {e}")
                });
            }

            if !matches!(info.kind, LocalFileKind::FabricJar) {
                if SETTINGS
                    .lock()
                    .map(|s| s.sync_client_settings.value)
                    .unwrap_or(false)
                {
                    if let Err(e) = self.ensure_client_synced(&info.file_name).await {
                        log_warn!("Failed to ensure client sync for {}: {}", info.file_name, e);
                    }
                }
            }
        }
        Ok(())
    }

    fn get_download_urls(file: &str) -> Result<Vec<String>, String> {
        let is_url = file.starts_with("http://") || file.starts_with("https://");
        if is_url {
            Ok(vec![file.to_string()])
        } else {
            let mut urls = Vec::new();

            if let Some(selected) = SERVERS.selected_cdn.read().unwrap().as_ref() {
                urls.push(format!("{}{}", selected.url, file));
            }

            for cdn in &SERVERS.cdns {
                let url = format!("{}{}", cdn.url, file);
                if !urls.contains(&url) {
                    urls.push(url);
                }
            }

            if urls.is_empty() {
                log_error!("No CDN server available for download");
                return Err("No CDN server available for download.".to_string());
            }

            Ok(urls)
        }
    }

    pub async fn download(&self, file: &str) -> Result<(), String> {
        let info = Self::resolve_local_file_info(file);
        let root_dir = self.root_dir_snapshot();

        if self.should_skip_download(&root_dir, &info) {
            log_debug!("File {} already exists, skipping download", file);
            return Ok(());
        }

        if let Some(app_handle) = APP_HANDLE.lock().unwrap().as_ref() {
            emit_to_main_window(app_handle, "download-start", &info.local_file);
        }

        self.prepare_download_dirs(&root_dir, &info).await?;

        let download_urls = Self::get_download_urls(file)?;
        let dest_path = Self::get_destination_path(&root_dir, &info);

        let app_handle = APP_HANDLE.lock().unwrap().clone();
        download_file(
            &download_urls,
            &dest_path,
            &info.local_file,
            app_handle.as_ref(),
        )
        .await?;

        if let Some(handle) = app_handle.as_ref() {
            emit_to_main_window(handle, "download-complete", &info.local_file);
        }

        if info.is_zip() {
            self.unzip(file).await.map_err(|e| {
                log_error!("Failed to extract {}: {}", file, e);
                if let Some(app_handle) = APP_HANDLE.lock().unwrap().as_ref() {
                    let err_data = serde_json::json!({
                        "file": info.local_file,
                        "error": e
                    });
                    emit_to_main_window(app_handle, "unzip-error", err_data);
                }
                e
            })?;
        }

        Ok(())
    }

    pub async fn sync_all_installed_clients(&self) -> Result<(), String> {
        let root_dir = self.root_dir_snapshot();
        let global_options_dir = root_dir.join("synced_options");

        if !global_options_dir.exists() {
            return Ok(());
        }

        let entries = match std::fs::read_dir(&root_dir) {
            Ok(e) => e,
            Err(e) => return Err(format!("Failed to read root dir: {e}")),
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            let name = match path.file_name().and_then(|n| n.to_str()) {
                Some(n) => n.to_string(),
                None => continue,
            };

            if fs_utils::SYSTEM_DIRS.contains(&name.as_str()) {
                continue;
            }

            if name.to_lowercase().starts_with("jdk") || name.to_lowercase().starts_with("java") {
                continue;
            }

            if let Err(e) = self.ensure_client_synced(&name).await {
                if !e.contains("5") {
                    // ignore access denied 5 (when file locked or not exist)
                    log_warn!("Failed to sync client {}: {}", name, e);
                }
            }
        }

        let custom_clients_dir = root_dir.join("custom_clients");
        if custom_clients_dir.exists() {
            if let Ok(entries) = std::fs::read_dir(&custom_clients_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if !path.is_dir() {
                        continue;
                    }
                    let name = match path.file_name().and_then(|n| n.to_str()) {
                        Some(n) => n.to_string(),
                        None => continue,
                    };
                    let client_base =
                        format!("custom_clients{}{}", std::path::MAIN_SEPARATOR, name);
                    if let Err(e) = self.ensure_client_synced(&client_base).await {
                        log_warn!("Failed to sync custom client {}: {}", name, e);
                    } else {
                        log_info!("Synced custom client: {}", name);
                    }
                }
            }
        }

        //log_info!("Synced {} client directories", synced);
        Ok(())
    }

    pub async fn ensure_client_synced(&self, client_base: &str) -> Result<(), String> {
        let root_dir = self.root_dir_snapshot();
        let global_options_dir = root_dir.join("synced_options");
        if !global_options_dir.exists() {
            tokio_fs::create_dir_all(&global_options_dir)
                .await
                .map_err(|e| format!("Failed to create global options dir: {e}"))?;
        }

        let client_dir = root_dir.join(client_base);
        if !client_dir.exists() {
            tokio_fs::create_dir_all(&client_dir)
                .await
                .map_err(|e| format!("Failed to create client dir: {e}"))?;
        }

        for item in SYNC_ITEMS {
            let target = global_options_dir.join(item.name);

            // process all sync folders (like resourcepacks and shaderpacks)
            if !target.exists() {
                if item.is_dir {
                    tokio_fs::create_dir_all(&target).await.map_err(|e| {
                        format!(
                            "Failed to create global {} dir: {}: {}",
                            item.name,
                            target.display(),
                            e
                        )
                    })?;
                } else {
                    tokio_fs::write(&target, "").await.map_err(|e| {
                        format!(
                            "Failed to create global {} file at {}: {}",
                            item.name,
                            target.display(),
                            e
                        )
                    })?;
                }
            }

            let client_target = client_dir.join(item.name);

            #[cfg(target_family = "windows")]
            if item.is_dir {
                if let Ok(true) = junction::exists(&client_target) {
                    continue;
                }
            }

            #[cfg(target_family = "windows")]
            if !item.is_dir {
                let _ = fs::remove_file(&client_target);
                if target.exists() {
                    if let Err(e) = fs_utils::copy_file(&target, &client_target) {
                        log_warn!("Failed to copy {} for {}: {}", item.name, client_base, e);
                    }
                }
                continue;
            }

            // why we need to deleting it? w1xced???
            // if let Err(e) = fs_utils::remove_path(&client_target) {
            //     log_warn!(
            //         "Failed to remove existing client {} at {}: {}",
            //         item.name,
            //         client_target.display(),
            //         e
            //     );
            // }

            // do not sync if folder/file already exists in client dir
            if client_target.exists() {
                continue;
            }

            if let Err(e) = fs_utils::create_link(&target, &client_target, item.is_dir) {
                log_warn!(
                    "Failed to link {} for {}: {} -> {}: {}",
                    item.name,
                    client_base,
                    target.display(),
                    client_target.display(),
                    e
                );
                match if item.is_dir {
                    fs_utils::copy_dir_recursive(&target, &client_target, false)
                } else {
                    fs_utils::copy_file(&target, &client_target)
                } {
                    Ok(_) => {
                        log_warn!(
                            "Used copy fallback for {} sync: {} -> {}",
                            item.name,
                            target.display(),
                            client_target.display()
                        );
                    }
                    Err(copy_err) => {
                        log_warn!(
                            "Fallback copy also failed for {}: {} -> {}: {}",
                            item.name,
                            target.display(),
                            client_target.display(),
                            copy_err
                        );
                    }
                }
            }
        }

        Ok(())
    }

    pub async fn sync_options_back(&self, client_base: &str) -> Result<(), String> {
        let root_dir = self.root_dir_snapshot();
        let global_options_dir = root_dir.join("synced_options");
        let client_dir = root_dir.join(client_base);

        if !client_dir.exists() || !global_options_dir.exists() {
            return Ok(());
        }

        let file_items = ["options.txt", "optionsof.txt"];

        for name in file_items {
            let client_file = client_dir.join(name);
            let global_file = global_options_dir.join(name);

            if !client_file.exists() {
                continue;
            }

            let should_copy = if global_file.exists() {
                let client_modified = fs::metadata(&client_file).and_then(|m| m.modified()).ok();
                let global_modified = fs::metadata(&global_file).and_then(|m| m.modified()).ok();

                match (client_modified, global_modified) {
                    (Some(c), Some(g)) => c > g,
                    _ => true,
                }
            } else {
                true
            };

            if should_copy {
                if let Err(e) = fs_utils::copy_file(&client_file, &global_file) {
                    log_warn!("Failed to sync {} back from {}: {}", name, client_base, e);
                } else {
                    log_info!(
                        "Synced {} back from {} to synced_options",
                        name,
                        client_base
                    );
                }
            }
        }

        Ok(())
    }

    pub async fn verify_file_md5(path: &Path, expected: &str) -> Result<bool, String> {
        let path = path.to_path_buf();
        let expected = expected.to_lowercase();
        let calc = tokio::task::spawn_blocking(move || {
            crate::core::utils::hashing::calculate_md5_hash(&path)
        })
        .await
        .map_err(|e| e.to_string())??;

        Ok(calc.eq_ignore_ascii_case(&expected))
    }

    pub async fn reset_requirements(&self) -> Result<(), String> {
        let root_dir = self.root_dir_snapshot();
        let base_requirements = [
            JDK21_FOLDER,
            JDK8_FOLDER,
            ASSETS_FOLDER,
            NATIVES_FOLDER,
            LIBRARIES_FOLDER,
            LIBRARIES_LEGACY_FOLDER,
            ASSETS_FABRIC_FOLDER,
            LIBRARIES_FABRIC_FOLDER,
            NATIVES_FABRIC_FOLDER,
            NATIVES_LEGACY_FOLDER,
            MINECRAFT_VERSIONS_FOLDER,
        ];

        let mut requirements = Vec::new();
        for req in base_requirements {
            requirements.push(req.to_string());
            requirements.push(format!("{}.zip", req));
        }

        for requirement in &requirements {
            let path = root_dir.join(requirement);
            if path.exists() {
                if path.is_dir() {
                    tokio_fs::remove_dir_all(&path)
                        .await
                        .map_err(|e| e.to_string())?;
                } else {
                    tokio_fs::remove_file(&path)
                        .await
                        .map_err(|e| e.to_string())?;
                }
            }
        }

        Ok(())
    }

    pub async fn reset_cache(&self) -> Result<(), String> {
        let root_dir = self.root_dir_snapshot();
        let cache_dir = root_dir.join("cache");
        if cache_dir.exists() {
            tokio_fs::remove_dir_all(&cache_dir)
                .await
                .map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    pub fn is_folder_healthy(&self, relative_path: &str) -> bool {
        let path = self.get_local(relative_path);
        let sentinel = path.join(".valid");

        if !path.exists() || !path.is_dir() {
            return false;
        }

        if !sentinel.exists() {
            log_warn!(
                "Folder {} exists but no sentinel found. Mark as unhealthy.",
                relative_path
            );
            return false;
        }

        std::fs::read_dir(&path).is_ok_and(|mut entries| entries.next().is_some())
    }

    pub fn verify_folder_integrity(&self, folder_name: &str) -> bool {
        let folder_path = self.root_dir.lock().unwrap().join(folder_name);
        let manifest_path = folder_path.join("manifest.txt");

        if !manifest_path.exists() {
            log_warn!("Manifest missing for folder: {}", folder_name);
            return false;
        }

        let content = match std::fs::read_to_string(&manifest_path) {
            Ok(c) => c,
            Err(e) => {
                log_error!("Failed to read manifest in {}: {}", folder_name, e);
                return false;
            }
        };

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || !line.contains(':') {
                continue;
            }

            let parts: Vec<&str> = line.splitn(2, ':').collect();
            let rel_path = parts[0];
            let expected_hash = parts[1];
            let full_path = folder_path.join(rel_path);

            if !full_path.exists() {
                log_warn!(
                    "Integrity Check Failed: File missing -> {}",
                    full_path.display()
                );
                return false;
            }

            match crate::core::utils::hashing::calculate_hash(&full_path) {
                Ok(actual_hash) => {
                    if actual_hash != expected_hash {
                        log_warn!("Integrity Check Failed: Hash mismatch for {}", rel_path);
                        return false;
                    }
                }
                Err(_) => return false,
            }
        }

        true
    }
}

pub static DATA: std::sync::LazyLock<Data> =
    std::sync::LazyLock::new(|| Data::new(ROOT_DIR.clone().into()));
