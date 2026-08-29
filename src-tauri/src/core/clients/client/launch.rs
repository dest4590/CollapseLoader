use std::{
    path::{Path, PathBuf},
    process::Stdio,
    sync::{Arc, Mutex},
};

#[cfg(unix)]
use std::os::unix::process::CommandExt;

use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::Command,
};

use super::{add_log_line, Client, ClientType, LaunchOptions, CLIENT_LOGS};

#[allow(unused)]
use crate::core::{
    clients::{
        internal::{
            agent_overlay::AgentArguments,
            titlebar_branding::TitlebarBrandingManager,
        },
        log_checker::LogChecker,
        manager::ClientManager,
    },
    network::{analytics::Analytics, server_ads},
    storage::{accounts::ACCOUNT_MANAGER, data::DATA, settings::SETTINGS},
    utils::{
        globals::{
            AGENT_FILE, AGENT_OVERLAY_FOLDER, ARM64_SUFFIX, ASSETS_FABRIC_FOLDER, ASSETS_FOLDER,
            IS_AARCH64, IS_LINUX, IS_MACOS, IS_WINDOWS, LEGACY_SUFFIX, LINUX_SUFFIX, MACOS_SUFFIX,
            NATIVES_FOLDER, NATIVES_LEGACY_LINUX_FOLDER, NATIVES_MACOS_ARM64_FOLDER,
            NATIVES_MACOS_FOLDER, PATH_SEPARATOR, SKIP_TITLEBAR_BRANDING,
            SUBNATIVES_1_8_9_LINUX_FOLDER, SUBNATIVES_1_8_9_MACOS_FOLDER,
            SUBNATIVES_1_8_9_WINDOWS_FOLDER, TITLEBAR_FILE,
        },
        helpers::emit_to_main_window,
        process::force_high_performance_gpu,
    },
};
use crate::{log_debug, log_error, log_info, log_warn};

impl Client {
    #[cfg(target_os = "linux")]
    fn has_nvidia_gpu() -> bool {
        std::process::Command::new("lspci")
            .output()
            .map(|o| {
                let stdout = String::from_utf8_lossy(&o.stdout);
                stdout.to_lowercase().contains("nvidia")
            })
            .unwrap_or(false)
    }

    fn append_new_instance_separator(&self) {
        let mut logs = CLIENT_LOGS.lock().unwrap_or_else(|e| e.into_inner());
        let client_logs = logs.entry(self.id).or_default();
        if !client_logs.is_empty() {
            client_logs.push("-------------------------------------------".to_string());
            client_logs.push("--- New Instance Started ---".to_string());
            client_logs.push("-------------------------------------------".to_string());
        }
    }

    async fn prepare_launch_prerequisites(
        &self,
        app_handle: &tauri::AppHandle,
    ) -> Result<(), String> {
        self.download_requirements(app_handle).await?;
        self.ensure_java_available(app_handle).await
    }

    fn resolve_java_bin(&self) -> PathBuf {
        if let Some(path) = self.java_path.as_deref().filter(|p| !p.is_empty()) {
            return PathBuf::from(path);
        }

        if let Ok(s) = SETTINGS.lock() {
            if !s.java_path.value.is_empty() {
                return PathBuf::from(&s.java_path.value);
            }
        }

        self.java_executable_path()
    }

    fn resolve_assets_dir(&self) -> PathBuf {
        let root = DATA.root_dir.lock().unwrap_or_else(|e| e.into_inner());
        if self.client_type == ClientType::Fabric {
            root.join(ASSETS_FABRIC_FOLDER)
        } else {
            root.join(ASSETS_FOLDER)
        }
    }

    fn resolve_linux_natives_path(root: &Path) -> PathBuf {
        root.join(format!("{}{}", NATIVES_FOLDER, LINUX_SUFFIX))
    }

    fn resolve_macos_natives_path(root: &Path, use_legacy_layout: bool) -> PathBuf {
        if IS_AARCH64 {
            if use_legacy_layout {
                root.join(format!(
                    "{}{}{}{}",
                    NATIVES_FOLDER, LEGACY_SUFFIX, MACOS_SUFFIX, ARM64_SUFFIX
                ))
            } else {
                root.join(NATIVES_MACOS_ARM64_FOLDER)
            }
        } else if use_legacy_layout {
            root.join(format!(
                "{}{}{}",
                NATIVES_FOLDER, LEGACY_SUFFIX, MACOS_SUFFIX
            ))
        } else {
            root.join(NATIVES_MACOS_FOLDER)
        }
    }

    fn resolve_default_natives_path(root: &Path, use_legacy_layout: bool) -> PathBuf {
        if use_legacy_layout {
            root.join(format!("{}{}", NATIVES_FOLDER, LEGACY_SUFFIX))
        } else {
            root.join(NATIVES_FOLDER)
        }
    }

    fn resolve_natives_path(&self) -> PathBuf {
        if let Some(path) = self
            .natives_path
            .as_deref()
            .filter(|p| !p.trim().is_empty())
        {
            return PathBuf::from(path);
        }

        let root = DATA.root_dir.lock().unwrap_or_else(|e| e.into_inner());
        let use_legacy_layout = self.is_legacy_client() || (!self.meta.is_new && IS_WINDOWS);

        if IS_LINUX {
            if self.uses_sub_libraries() {
                root.join(SUBNATIVES_1_8_9_LINUX_FOLDER)
            } else if self.is_legacy_client() {
                root.join(NATIVES_LEGACY_LINUX_FOLDER)
            } else {
                Self::resolve_linux_natives_path(&root)
            }
        } else if IS_MACOS {
            if self.uses_sub_libraries() {
                root.join(SUBNATIVES_1_8_9_MACOS_FOLDER)
            } else {
                Self::resolve_macos_natives_path(&root, use_legacy_layout)
            }
        } else {
            if self.uses_sub_libraries() {
                root.join(SUBNATIVES_1_8_9_WINDOWS_FOLDER)
            } else {
                Self::resolve_default_natives_path(&root, use_legacy_layout)
            }
        }
    }

    fn get_launch_settings(&self) -> (bool, bool, String, u32) {
        let s = SETTINGS.lock().unwrap_or_else(|e| e.into_inner());
        (
            s.optional_telemetry.value,
            s.irc_chat.value,
            s.language.value.clone(),
            s.ram.value,
        )
    }

    fn resolve_username(&self) -> String {
        ACCOUNT_MANAGER
            .lock()
            .ok()
            .and_then(|m| m.get_active_account().map(|a| a.username.clone()))
            .unwrap_or_else(|| {
                let rnd = rand::random::<u32>() % 100_000;
                format!("Collapse{rnd:05}")
            })
    }

    fn append_java_args(cmd: &mut Command, args: &str) {
        for arg in args.split_whitespace() {
            cmd.arg(arg);
        }
    }

    fn apply_java_args(&self, cmd: &mut Command) {
        if let Ok(s) = SETTINGS.lock() {
            if !s.java_args.value.is_empty() {
                Self::append_java_args(cmd, &s.java_args.value);
            }
        }

        if let Some(args) = self.java_args.as_deref().filter(|a| !a.is_empty()) {
            Self::append_java_args(cmd, args);
        }
    }

    fn append_game_launch_args(
        &self,
        cmd: &mut Command,
        username: &str,
        client_folder: &Path,
        assets_dir: &Path,
    ) {
        let effective_asset_index = self.effective_asset_index();

        cmd.arg("--username")
            .arg(username)
            .arg("--gameDir")
            .arg(client_folder)
            .arg("--assetsDir")
            .arg(assets_dir)
            .arg("--assetIndex")
            .arg(effective_asset_index)
            .arg("--uuid")
            .arg("N/A")
            .arg("--accessToken")
            .arg("0")
            .arg("--userType")
            .arg("legacy")
            .arg("--version")
            .arg(&self.version)
            .arg("--client")
            .arg(&self.filename);
    }

    fn effective_asset_index(&self) -> String {
        if !self.meta.asset_index.is_empty() {
            return self.meta.asset_index.clone();
        }

        if self.version.contains("1.21") {
            "1.21".to_string()
        } else if self.version.contains("1.16") {
            "1.16".to_string()
        } else if self.version.contains("1.8.9") {
            "1.8".to_string()
        } else {
            "1.16".to_string()
        }
    }

    fn redact_sensitive_command(command: &str) -> String {
        let mut secure_command = command.to_owned();

        if let Some(start) = secure_command.find("-javaagent:") {
            if let Some(end) = secure_command[start..].find(" -") {
                let actual_end = start + end;
                secure_command.replace_range(start..actual_end, "-javaagent:[HIDDEN]");
            } else if let Some(end) = secure_command[start..].find('"') {
                let actual_end = start + end;
                secure_command.replace_range(start..actual_end, "-javaagent:[HIDDEN]");
            }
        }

        secure_command
    }

    pub async fn run(
        self,
        options: LaunchOptions,
        manager: Arc<Mutex<ClientManager>>,
    ) -> Result<(), String> {
        if !options.is_custom && SETTINGS.lock().is_ok_and(|s| s.optional_telemetry.value) {
            Analytics::send_client_analytics(self.id, &options.user_token);
        }

        self.append_new_instance_separator();
        let crash_report_token = options.user_token.clone();

        let app_handle = options.app_handle.clone();
        let client_id = self.id;

        if let Err(e) = self.prepare_launch_prerequisites(&app_handle).await {
            self.emit_crash(&app_handle, &e);
            return Err(e);
        }

        let java_bin = self.resolve_java_bin();

        #[cfg(windows)]
        force_high_performance_gpu(&java_bin);

        let (client_folder, _) = self.get_launch_paths()?;
        let assets_dir = self.resolve_assets_dir();
        let natives_path = self.resolve_natives_path();

        let is_legacy_vanilla_for_natives = self.client_type == ClientType::Default
            && self.is_legacy_client();
        if is_legacy_vanilla_for_natives {
            let natives_link = client_folder.join("natives");
            if !natives_link.exists() {
                #[cfg(unix)]
                {
                    let _ = std::os::unix::fs::symlink(&natives_path, &natives_link);
                }
                #[cfg(windows)]
                {
                    let _ = std::os::windows::fs::symlink_dir(&natives_path, &natives_link);
                }
            }
        }

        let classpath = self.build_classpath()?;

        let (analytics, irc, lang, ram_mb) = self.get_launch_settings();

        let username = self.resolve_username();

        let agent_args = AgentArguments::new(
            options.user_token,
            self.name.clone(),
            if self.meta.is_custom {
                false
            } else {
                analytics
            },
            irc,
            lang,
        );

        let agent_overlay_path = DATA
            .root_dir
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .join(AGENT_OVERLAY_FOLDER);

        let is_legacy_vanilla = self.client_type == ClientType::Default && !self.meta.is_new;

        let should_apply_titlebar = !*SKIP_TITLEBAR_BRANDING
            && !self.meta.is_custom
            && self.client_type != ClientType::Forge
            && !is_legacy_vanilla
            && !TitlebarBrandingManager::has_branding_in_jar(&client_folder.join(&self.filename));

        if should_apply_titlebar {
            let titlebar_path = agent_overlay_path.join(TITLEBAR_FILE);
            if titlebar_path.exists() {
                log_info!(
                    "Titlebar branding will be applied for client: {}",
                    self.name
                );
            } else {
                log_debug!(
                    "Titlebar branding file not found, skipping for: {}",
                    self.name
                );
            }
        } else if !*SKIP_TITLEBAR_BRANDING && !self.meta.is_custom {
            log_info!(
                "Skipping titlebar branding for {} (already branded or excluded)",
                self.name
            );
        }

        let mut cmd = Command::new(java_bin);

        #[cfg(windows)]
        cmd.creation_flags(0x0800_0000);

        cmd.current_dir(&client_folder);

        cmd.arg("-Xverify:none");

        if self.is_legacy_client() && self.client_type == ClientType::Default {
            cmd.arg("-XX:+UseG1GC");
            cmd.arg("-XX:MaxPermSize=256m");
            cmd.arg("-Dorg.lwjgl.system.stacksize=16384");
            cmd.arg("-Dorg.lwjgl.system.nojni=false");
        }

        #[cfg(target_os = "macos")]
        cmd.arg("-XstartOnFirstThread");

        #[cfg(target_os = "linux")]
        if Self::has_nvidia_gpu() {
            cmd.env("__NV_PRIME_RENDER_OFFLOAD", "1");
            cmd.env("__GLX_VENDOR_LIBRARY_NAME", "nvidia");
            cmd.env("__VK_LAYER_NV_optimus", "NVIDIA_only");
            cmd.env("DRI_PRIME", "1");
        }

        if self.is_legacy_client() && self.client_type == ClientType::Default {
            #[cfg(target_os = "linux")]
            {
                cmd.env("MALLOC_TRIM_THRESHOLD_", "131072");
                cmd.env("MALLOC_TOP_PAD_", "131072");
                cmd.env("MALLOC_MMAP_THRESHOLD_", "131072");
                cmd.env("MALLOC_ARENA_MAX", "1");
            }
        }

        if !should_apply_titlebar {
            cmd.env("COLLAPSE_SKIP_TITLEBAR", "1");
        }

        #[cfg(target_os = "linux")]
        {
            if is_legacy_vanilla {
                let jemalloc_path = DATA
                    .root_dir
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .join("natives-linux")
                    .join("libjemalloc.so");
                if jemalloc_path.exists() {
                    cmd.env("LD_PRELOAD", &jemalloc_path);
                    cmd.env(
                        "MALLOC_CONF",
                        "background_thread:true,metadata_thp:auto,dirty_decay_ms:9000000000,muzzy_decay_ms:9000000000",
                    );
                    log_info!(
                        "Loaded libjemalloc.so via LD_PRELOAD for legacy vanilla client: {}",
                        self.name
                    );
                } else {
                    log_warn!("libjemalloc.so not found at {}", jemalloc_path.display());
                }
            }
        }

        #[cfg(target_os = "linux")]
        if should_apply_titlebar && agent_overlay_path.join(TITLEBAR_FILE).exists() {
            let jemalloc_path = DATA
                .root_dir
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .join("natives-linux")
                .join("libjemalloc.so");
            let titlebar = agent_overlay_path.join(TITLEBAR_FILE);
            let preload = if jemalloc_path.exists() {
                format!("{}:{}", jemalloc_path.display(), titlebar.display())
            } else {
                titlebar.display().to_string()
            };
            cmd.env("LD_PRELOAD", &preload);
        }

        #[cfg(target_os = "windows")]
        if should_apply_titlebar {
            let titlebar_path = agent_overlay_path.join(TITLEBAR_FILE);
            if titlebar_path.exists() {
                cmd.arg(format!("-agentpath:{}", titlebar_path.display()));
            }
        }

        #[cfg(target_os = "macos")]
        if should_apply_titlebar {
            let titlebar_path = agent_overlay_path.join(TITLEBAR_FILE);
            if titlebar_path.exists() {
                cmd.arg(format!("-agentpath:{}", titlebar_path.display()));
            }
        }

        if !self.meta.is_custom && self.client_type != ClientType::Forge && !is_legacy_vanilla {
            cmd.arg(format!(
                "-javaagent:{}={}",
                agent_overlay_path.join(AGENT_FILE).display(),
                agent_args.encode()
            ));
        }

        self.apply_java_args(&mut cmd);

        if is_legacy_vanilla {
            cmd.arg("-XX:+UseG1GC");
            cmd.arg("-Dorg.lwjgl.system.stacksize=16384");
            cmd.arg("-Dorg.lwjgl.system.nojni=false");
            #[cfg(target_os = "linux")]
            {
                cmd.arg("-Dcom.sun.java.util.jar.disableSHA1=true");
            }
        }

        if is_legacy_vanilla {
            cmd.arg("-Xss256k");
        }

        cmd.arg(format!("-Xmx{ram_mb}M"));
        if self.meta.is_custom {
            cmd.arg(format!(
                "-Djava.library.path={}",
                natives_path.display(),
            ));
        } else {
            cmd.arg(format!(
                "-Djava.library.path={}{}{}",
                natives_path.display(),
                PATH_SEPARATOR,
                agent_overlay_path.display()
            ));
        }

        let actual_main_class = if self.client_type == ClientType::Forge {
            "net.minecraft.launchwrapper.Launch".to_string()
        } else if self.meta.is_custom && self.client_type == ClientType::Fabric {
            "net.fabricmc.loader.impl.launch.knot.KnotClient".to_string()
        } else {
            self.main_class.clone()
        };

        cmd.arg("-cp").arg(classpath).arg(actual_main_class);

        if self.client_type == ClientType::Forge {
            cmd.arg("--tweakClass")
                .arg("net.minecraftforge.fml.common.launcher.FMLTweaker");
        }

        self.append_game_launch_args(&mut cmd, &username, &client_folder, &assets_dir);

        cmd.stdout(Stdio::piped()).stderr(Stdio::piped());

        let servers_dat_path = client_folder.join("servers.dat");
        let server_result = server_ads::fetch_server_ads().await;
        server_ads::inject_servers_dat(&servers_dat_path, &server_result);

        log_debug!("Spawning client process: {}", self.name);

        #[cfg(target_os = "linux")]
        {
            lower_nofile_in_parent();
            let is_legacy = self.is_legacy_client();
            unsafe {
                cmd.pre_exec(move || {
                    if is_legacy {
                        lower_nofile_in_parent();
                    }
                    Ok(())
                });
            }
        }

        let mut child = cmd
            .spawn()
            .map_err(|e| format!("Failed to spawn process: {e}"))?;

        let secure_command = Self::redact_sensitive_command(&format!("{cmd:#?}"));

        add_log_line(client_id, secure_command);

        let self_clone = self.clone();

        emit_to_main_window(
            &app_handle,
            "client-launched",
            serde_json::json!({
                "id": client_id,
                "name": self.name,
                "version": self.version
            }),
        );

        if let Some(stdout) = child.stdout.take() {
            let id = client_id;
            tokio::spawn(async move {
                let mut reader = BufReader::new(stdout).lines();
                while let Ok(Some(line)) = reader.next_line().await {
                    add_log_line(id, line);
                }
            });
        }

        if let Some(stderr) = child.stderr.take() {
            let id = client_id;
            tokio::spawn(async move {
                let mut reader = BufReader::new(stderr).lines();
                while let Ok(Some(line)) = reader.next_line().await {
                    add_log_line(id, line);
                }
            });
        }

        tokio::spawn(async move {
            match child.wait().await {
                Ok(status) => {
                    let msg = format!("Process finished with status: {status}");
                    log_info!("{}", msg);
                    add_log_line(client_id, msg);

                    let checker = LogChecker::new(self_clone.clone(), crash_report_token.clone());
                    checker.check(&app_handle);

                    if let Ok(m) = manager.lock() {
                        let _ = m.update_status_on_client_exit(&app_handle);
                    }

                    let sync_enabled = SETTINGS
                        .lock()
                        .map(|s| s.sync_client_settings.value)
                        .unwrap_or(false);

                    if sync_enabled {
                        #[cfg(target_family = "windows")]
                        {
                            let client_base = crate::core::storage::data::Data::get_filename(
                                &self_clone.filename,
                            );

                            if let Err(e) = DATA.sync_options_back(&client_base).await {
                                log_info!("Failed to sync options back for {}: {}", client_base, e);
                            }
                        }
                    }

                    emit_to_main_window(
                        &app_handle,
                        "client-exited",
                        serde_json::json!({
                            "id": client_id,
                            "name": self_clone.name,
                            "exitCode": status.code().unwrap_or(-1)
                        }),
                    );
                }
                Err(e) => {
                    let msg = format!("Error waiting for process: {e}");
                    log_error!("{}", msg);
                    add_log_line(client_id, msg.clone());
                    emit_to_main_window(
                        &app_handle,
                        "client-crashed",
                        serde_json::json!({
                            "id": client_id,
                            "name": self_clone.name,
                            "error": msg
                        }),
                    );
                }
            }
        });

        Ok(())
    }

    fn emit_crash(&self, app_handle: &tauri::AppHandle, error: &str) {
        emit_to_main_window(
            app_handle,
            "client-crashed",
            serde_json::json!({
                "id": self.id,
                "name": self.name,
                "error": error
            }),
        );
    }

    #[cfg(target_os = "linux")]
    pub fn lower_nofile_in_child() {
        use std::os::raw::{c_int, c_ulong};

        const RLIMIT_NOFILE: c_int = 7;
        const TARGET_SOFT: c_ulong = 8192;

        #[repr(C)]
        struct Rlimit {
            rlim_cur: c_ulong,
            rlim_max: c_ulong,
        }

        extern "C" {
            fn prlimit(
                pid: c_int,
                resource: c_int,
                new_limit: *const Rlimit,
                old_limit: *mut Rlimit,
            ) -> c_int;
        }

        let mut current = Rlimit { rlim_cur: 0, rlim_max: 0 };
        let ret = unsafe {
            prlimit(0, RLIMIT_NOFILE, std::ptr::null(), &mut current)
        };
        if ret != 0 {
            return;
        }

        if current.rlim_cur > TARGET_SOFT || current.rlim_max > TARGET_SOFT {
            let target = if TARGET_SOFT < current.rlim_max {
                TARGET_SOFT
            } else {
                current.rlim_max
            };
            let new_limit = Rlimit {
                rlim_cur: target,
                rlim_max: target,
            };
            unsafe {
                prlimit(0, RLIMIT_NOFILE, &new_limit, std::ptr::null_mut());
            }
        }
    }
}

#[cfg(target_os = "linux")]
pub fn lower_nofile_in_parent() {
    use std::os::raw::{c_int, c_ulong};

    const RLIMIT_NOFILE: c_int = 7;
    const TARGET_SOFT: c_ulong = 8192;

    #[repr(C)]
    struct Rlimit {
        rlim_cur: c_ulong,
        rlim_max: c_ulong,
    }

    extern "C" {
        fn prlimit(
            pid: c_int,
            resource: c_int,
            new_limit: *const Rlimit,
            old_limit: *mut Rlimit,
        ) -> c_int;
    }

    let mut current = Rlimit { rlim_cur: 0, rlim_max: 0 };
    let ret = unsafe {
        prlimit(0, RLIMIT_NOFILE, std::ptr::null(), &mut current)
    };
    if ret != 0 {
        log_warn!("Failed to read RLIMIT_NOFILE");
        return;
    }

    log_debug!(
        "Current RLIMIT_NOFILE: cur={} max={}",
        current.rlim_cur,
        current.rlim_max
    );

    if current.rlim_cur > TARGET_SOFT || current.rlim_max > TARGET_SOFT {
        let target = if TARGET_SOFT < current.rlim_max {
            TARGET_SOFT
        } else {
            current.rlim_max
        };
        let new_limit = Rlimit {
            rlim_cur: target,
            rlim_max: target,
        };
        let ret = unsafe {
            prlimit(0, RLIMIT_NOFILE, &new_limit, std::ptr::null_mut())
        };
        if ret == 0 {
            log_info!(
                "Lowered RLIMIT_NOFILE from {}/{} to {}/{} for legacy client",
                current.rlim_cur,
                current.rlim_max,
                target,
                target
            );
        } else {
            log_warn!(
                "Failed to set RLIMIT_NOFILE: {}",
                std::io::Error::last_os_error()
            );
        }
    }
}
