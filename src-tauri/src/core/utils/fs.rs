use std::fs;
use std::io::Write;
use std::path::Path;

pub const SYSTEM_DIRS: &[&str] = &[
    "synced_options",
    "libraries",
    "libraries-fabric",
    "libraries-legacy",
    "natives",
    "natives-macos-x64",
    "natives-macos-arm64",
    "natives-linux",
    "natives-legacy",
    "natives-legacy-linux",
    "natives-fabric",
    "assets",
    "assets-fabric",
    "minecraft-versions",
    "custom_clients",
    "agent_overlay",
    "misc",
    "cache",
];

pub fn ensure_dir(path: &Path) -> Result<(), String> {
    fs::create_dir_all(path).map_err(|e| e.to_string())
}

pub fn ensure_parent_dir(path: &Path) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            ensure_dir(parent)?;
        }
    }
    Ok(())
}

pub fn atomic_write(path: &Path, contents: &[u8]) -> Result<(), String> {
    ensure_parent_dir(path)?;

    let mut temp_path = path.to_path_buf();
    let temp_extension = match path.extension().and_then(|ext| ext.to_str()) {
        Some(ext) => format!("{ext}.tmp"),
        None => "tmp".to_string(),
    };
    temp_path.set_extension(temp_extension);

    let mut file = fs::File::create(&temp_path).map_err(|e| e.to_string())?;
    file.write_all(contents).map_err(|e| e.to_string())?;
    file.sync_all().map_err(|e| e.to_string())?;

    if path.exists() {
        remove_path(path)?;
    }

    fs::rename(&temp_path, path).map_err(|e| e.to_string())
}

pub fn remove_path(path: &Path) -> Result<(), String> {
    if fs::symlink_metadata(path).is_err() {
        return Ok(());
    }

    #[cfg(target_family = "windows")]
    {
        if let Ok(true) = junction::exists(path) {
            return junction::delete(path).map_err(|e| e.to_string());
        }
    }

    match fs::symlink_metadata(path) {
        Ok(meta) => {
            if meta.file_type().is_symlink() || meta.is_file() {
                fs::remove_file(path).map_err(|e| e.to_string())
            } else {
                fs::remove_dir_all(path).map_err(|e| e.to_string())
            }
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(e.to_string()),
    }
}

pub fn copy_file(src: &Path, dst: &Path) -> Result<(), String> {
    ensure_parent_dir(dst)?;
    fs::copy(src, dst).map(|_| ()).map_err(|e| e.to_string())
}

pub fn copy_dir_recursive(src: &Path, dst: &Path, skip_symlinks: bool) -> Result<(), String> {
    ensure_dir(dst)?;

    for entry in fs::read_dir(src).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let source_path = entry.path();
        let target_path = dst.join(entry.file_name());
        let file_type = entry.file_type().map_err(|e| e.to_string())?;

        if file_type.is_symlink() && skip_symlinks {
            continue;
        }

        if file_type.is_dir() {
            copy_dir_recursive(&source_path, &target_path, skip_symlinks)?;
        } else {
            copy_file(&source_path, &target_path)?;
        }
    }

    Ok(())
}

pub fn create_link(src: &Path, dst: &Path, is_dir: bool) -> Result<(), String> {
    #[cfg(target_family = "unix")]
    {
        let _ = is_dir;
        std::os::unix::fs::symlink(src, dst).map_err(|e| e.to_string())
    }

    #[cfg(target_family = "windows")]
    {
        use std::os::windows::fs::{symlink_dir, symlink_file};

        if is_dir {
            match symlink_dir(src, dst) {
                Ok(()) => Ok(()),
                Err(symlink_err) => junction::create(src, dst).map_err(|junction_err| {
                    format!(
                        "{}; junction fallback failed: {}",
                        symlink_err, junction_err
                    )
                }),
            }
        } else {
            match symlink_file(src, dst) {
                Ok(()) => Ok(()),
                Err(symlink_err) => fs::hard_link(src, dst).map_err(|hard_link_err| {
                    format!(
                        "{}; hard-link fallback failed: {}",
                        symlink_err, hard_link_err
                    )
                }),
            }
        }
    }
}
