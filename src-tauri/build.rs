use std::fs::OpenOptions;
use std::io;
use std::io::Write;
use std::process::Command;

fn execute_git_command(args: &[&str]) -> io::Result<String> {
    let output = Command::new("git").args(args).output()?;
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

#[cfg(unix)]
fn force_println(msg: &str) {
    if let Ok(mut tty) = OpenOptions::new().write(true).open("/dev/tty") {
        let _ = writeln!(tty, "{}", msg);
    }
}

#[cfg(windows)]
fn force_println(msg: &str) {
    if let Ok(mut tty) = OpenOptions::new().write(true).open("CONOUT$") {
        let _ = writeln!(tty, "{msg}");
    }
}

fn main() {
    const NAME: &str = "CollapseBuilder";

    fn banner() {
        force_println("\n");
        let width = NAME.len() + 4;
        force_println(&format!("+{}+", "-".repeat(width)));
        force_println(&format!("|  {NAME}  |"));
        force_println(&format!("+{}+", "-".repeat(width)));
    }

    fn info(msg: &str) {
        force_println(&format!("info: {msg}"));
    }

    fn warn(msg: &str) {
        force_println(&format!("warning: {msg}"));
    }

    fn fence() {
        force_println(&format!("+{}+", "-".repeat(NAME.len() + 4)));
    }

    let is_dev_like = tauri_build::is_dev();
    let profile = std::env::var("PROFILE").unwrap_or_default();
    let is_release_build = !is_dev_like && profile == "release";
    let npm_tauri_build = std::env::var_os("NPM_TAURI_BUILD").is_some();

    let build_kind = if is_release_build {
        if npm_tauri_build {
            "tauri_release_npm"
        } else {
            "tauri_release"
        }
    } else if is_dev_like {
        "tauri_dev"
    } else {
        "cargo"
    };

    let is_release_build = build_kind == "tauri_release" || build_kind == "tauri_release_npm";

    banner();

    let development: bool = std::env::var("DEVELOPMENT").map_or_else(
        |_| {
            std::fs::read_to_string("../.env")
                .map_err(|_| {})
                .ok()
                .and_then(|s| {
                    s.lines()
                        .map(str::trim)
                        .filter(|l| !l.is_empty() && !l.starts_with('#'))
                        .find_map(|l| {
                            let mut kv = l.splitn(2, '=');
                            let k = kv.next().map(str::trim);
                            let v = kv.next().map(str::trim);
                            if k == Some("DEVELOPMENT") {
                                v.map(str::to_string)
                            } else {
                                None
                            }
                        })
                })
                .is_some_and(|v| {
                    let lv = v.to_ascii_lowercase();
                    matches!(lv.as_str(), "1" | "true" | "yes" | "y" | "on")
                })
        },
        |v| {
            let lv = v.to_ascii_lowercase();
            matches!(lv.as_str(), "1" | "true" | "yes" | "y" | "on")
        },
    );
    println!("cargo:rustc-env=DEVELOPMENT={development}");
    println!("cargo:rerun-if-env-changed=DEVELOPMENT");
    println!("cargo:rerun-if-changed=../.env");

    if is_release_build && development {
        for _ in 1..5 {
            println!("cargo:warning=RELEASE build with DEVELOPMENT=true");
        }
    }

    let git_hash = execute_git_command(&["rev-parse", "HEAD"]).unwrap_or_else(|_| "unknown".into());
    let git_branch = execute_git_command(&["rev-parse", "--abbrev-ref", "HEAD"])
        .unwrap_or_else(|_| "unknown".into());
    println!("cargo:rustc-env=GIT_HASH={git_hash}");
    println!("cargo:rustc-env=GIT_BRANCH={git_branch}");

    info(&format!("Build kind: {build_kind}"));
    info(&format!("Profile: {profile}"));
    if development {
        warn("DEVELOPMENT mode: true");
    } else {
        info("DEVELOPMENT mode: false");
    }
    info(&format!("Git branch: {git_branch}"));
    info(&format!(
        "Git hash: {}",
        git_hash.chars().take(7).collect::<String>()
    ));

    fence();

    tauri_build::build();
}
