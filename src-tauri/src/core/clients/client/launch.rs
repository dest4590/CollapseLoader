use std::{
    path::{Path, PathBuf},
    process::Stdio,
    sync::{Arc, Mutex},
};

use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::Command,
};

use super::{add_log_line, Client, ClientType, LaunchOptions, CLIENT_LOGS};

use crate::core::{
    clients::{
        internal::agent_overlay::AgentArguments, log_checker::LogChecker, manager::ClientManager,
    },
    network::{analytics::Analytics, server_ads},
    storage::{accounts::ACCOUNT_MANAGER, data::DATA, settings::SETTINGS},
    utils::{
        globals::{
            AGENT_FILE, AGENT_OVERLAY_FOLDER, ARM64_SUFFIX, ASSETS_FABRIC_FOLDER, ASSETS_FOLDER,
            IS_AARCH64, IS_LINUX, IS_MACOS, LEGACY_SUFFIX, LINUX_SUFFIX, MACOS_SUFFIX,
            NATIVES_FOLDER, NATIVES_LEGACY_LINUX_FOLDER, NATIVES_MACOS_ARM64_FOLDER,
            NATIVES_MACOS_FOLDER, PATH_SEPARATOR,
        },
        helpers::emit_to_main_window,
    },
};
use crate::{log_debug, log_error, log_info};

impl Client {
    fn append_new_instance_separator(&self) {
        let mut logs = CLIENT_LOGS.lock().unwrap();
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
        if self.client_type == ClientType::Fabric {
            DATA.root_dir.lock().unwrap().join(ASSETS_FABRIC_FOLDER)
        } else {
            DATA.root_dir.lock().unwrap().join(ASSETS_FOLDER)
        }
    }

    fn resolve_natives_path(&self) -> PathBuf {
        let root = DATA.root_dir.lock().unwrap();

        if self.is_legacy_client() {
            if IS_LINUX {
                root.join(NATIVES_LEGACY_LINUX_FOLDER)
            } else if IS_MACOS {
                if IS_AARCH64 {
                    root.join(format!(
                        "{}{}{}{}",
                        NATIVES_FOLDER, LEGACY_SUFFIX, MACOS_SUFFIX, ARM64_SUFFIX
                    ))
                } else {
                    root.join(format!(
                        "{}{}{}",
                        NATIVES_FOLDER, LEGACY_SUFFIX, MACOS_SUFFIX
                    ))
                }
            } else {
                root.join(format!("{}{}", NATIVES_FOLDER, LEGACY_SUFFIX))
            }
        } else if self.meta.is_new {
            if IS_LINUX {
                root.join(format!("{}{}", NATIVES_FOLDER, LINUX_SUFFIX))
            } else if IS_MACOS {
                if IS_AARCH64 {
                    root.join(NATIVES_MACOS_ARM64_FOLDER)
                } else {
                    root.join(NATIVES_MACOS_FOLDER)
                }
            } else {
                root.join(NATIVES_FOLDER)
            }
        } else {
            if IS_LINUX {
                root.join(format!("{}{}", NATIVES_FOLDER, LINUX_SUFFIX))
            } else if IS_MACOS {
                if IS_AARCH64 {
                    root.join(format!(
                        "{}{}{}",
                        NATIVES_FOLDER, MACOS_SUFFIX, ARM64_SUFFIX
                    ))
                } else {
                    root.join(NATIVES_MACOS_FOLDER)
                }
            } else {
                root.join(format!("{}{}", NATIVES_FOLDER, LEGACY_SUFFIX))
            }
        }
    }

    fn get_launch_settings(&self) -> (bool, bool, String, u32) {
        let s = SETTINGS.lock().unwrap();
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
        cmd.arg("--username")
            .arg(username)
            .arg("--gameDir")
            .arg(client_folder)
            .arg("--assetsDir")
            .arg(assets_dir)
            .arg("--assetIndex")
            .arg(&self.meta.asset_index)
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

        let (client_folder, _) = self.get_launch_paths()?;
        let assets_dir = self.resolve_assets_dir();
        let natives_path = self.resolve_natives_path();

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

        let agent_overlay_path = DATA.root_dir.lock().unwrap().join(AGENT_OVERLAY_FOLDER);

        let mut cmd = Command::new(java_bin);

        #[cfg(windows)]
        cmd.creation_flags(0x0800_0000);

        cmd.current_dir(&client_folder);

        cmd.arg("-Xverify:none");

        #[cfg(target_os = "macos")]
        cmd.arg("-XstartOnFirstThread");

        let is_legacy_vanilla = self.client_type == ClientType::Default && !self.meta.is_new;
        if self.client_type != ClientType::Forge && !is_legacy_vanilla {
            cmd.arg(format!(
                "-javaagent:{}={}",
                agent_overlay_path.join(AGENT_FILE).display(),
                agent_args.encode()
            ));
        }

        self.apply_java_args(&mut cmd);

        cmd.arg(format!("-Xmx{ram_mb}M"));

        cmd.arg(format!(
            "-Djava.library.path={}{}{}",
            natives_path.display(),
            PATH_SEPARATOR,
            agent_overlay_path.display()
        ));

        cmd.arg("-cp").arg(classpath).arg(&self.main_class);

        if self.client_type == ClientType::Forge {
            cmd.arg("--tweakClass")
                .arg("net.minecraftforge.fml.common.launcher.FMLTweaker");
        }

        self.append_game_launch_args(&mut cmd, &username, &client_folder, &assets_dir);

        cmd.stdout(Stdio::piped()).stderr(Stdio::piped());

        let servers_dat_path = client_folder.join("servers.dat");
        let ads = server_ads::fetch_server_ads().await;
        server_ads::inject_servers_dat(&servers_dat_path, &ads);

        log_debug!("Spawning client process: {}", self.name);

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
}
