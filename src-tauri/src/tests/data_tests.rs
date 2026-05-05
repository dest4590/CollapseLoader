use std::fs;
use std::path::{Path, PathBuf};

use crate::core::storage::data::{Data, FileInfo, LocalFileKind};

// ── helpers ───────────────────────────────────────────────────────────────────

struct TempDir(PathBuf);

impl TempDir {
    fn new(name: &str) -> Self {
        let path =
            std::env::temp_dir().join(format!("collapseloader_test_{name}_{}", std::process::id()));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).expect("failed to create temp dir");
        Self(path)
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[cfg(target_family = "windows")]
#[test]
fn remove_path_deletes_directory_with_readonly_file() {
    let tmp = TempDir::new("remove_path_readonly_dir");
    let target_dir = tmp.path().join("resourcepacks");
    let readonly_file = target_dir.join("pack.mcmeta");

    fs::create_dir_all(&target_dir).unwrap();
    fs::write(&readonly_file, "test").unwrap();

    let mut permissions = fs::metadata(&readonly_file).unwrap().permissions();
    permissions.set_readonly(true);
    fs::set_permissions(&readonly_file, permissions).unwrap();

    crate::core::utils::fs::remove_path(&target_dir).unwrap();

    assert!(!target_dir.exists());
}

// ── Data::has_extension ───────────────────────────────────────────────────────

#[test]
fn has_extension_matches_exact() {
    assert!(Data::has_extension("client.jar", "jar"));
}

#[test]
fn has_extension_is_case_insensitive() {
    assert!(Data::has_extension("archive.ZIP", "zip"));
    assert!(Data::has_extension("archive.Zip", "ZIP"));
}

#[test]
fn has_extension_returns_false_for_wrong_ext() {
    assert!(!Data::has_extension("client.jar", "zip"));
}

#[test]
fn has_extension_returns_false_for_no_ext() {
    assert!(!Data::has_extension("client", "jar"));
}

// ── Data::get_filename ────────────────────────────────────────────────────────

#[test]
fn get_filename_strips_extension() {
    assert_eq!(Data::get_filename("1.16.5.jar"), "1.16.5");
}

#[test]
fn get_filename_works_with_path() {
    assert_eq!(
        Data::get_filename("fabric/jars/fabric-1.18.jar"),
        "fabric-1.18"
    );
}

#[test]
fn get_filename_no_extension_returns_full_name() {
    assert_eq!(Data::get_filename("somefile"), "somefile");
}

// ── Data::get_as_folder_string ────────────────────────────────────────────────

#[test]
fn get_as_folder_string_adds_separator() {
    let result = Data::get_as_folder_string("assets.zip");
    assert!(result.ends_with(std::path::MAIN_SEPARATOR));
    assert!(result.starts_with("assets"));
}

// ── Data::get_local ───────────────────────────────────────────────────────────

#[test]
fn get_local_joins_simple_path() {
    let tmp = TempDir::new("get_local");
    let data = Data::new(tmp.path().to_path_buf());
    assert_eq!(data.get_local("natives"), tmp.path().join("natives"));
}

#[test]
fn get_local_normalises_forward_and_back_slashes() {
    let tmp = TempDir::new("get_local_slashes");
    let data = Data::new(tmp.path().to_path_buf());
    let a = data.get_local("a/b/c");
    let b = data.get_local("a\\b\\c");
    assert_eq!(a, b);
}

#[test]
fn get_local_ignores_empty_segments() {
    let tmp = TempDir::new("get_local_empty");
    let data = Data::new(tmp.path().to_path_buf());
    let result = data.get_local("a//b");
    assert_eq!(result, tmp.path().join("a").join("b"));
}

// ── FileInfo classification ───────────────────────────────────────────────────

#[test]
fn file_info_classifies_zip() {
    let info = FileInfo::new("natives-linux.zip");
    assert!(matches!(info.kind, LocalFileKind::Zip));
    assert!(info.is_zip());
    assert!(!info.is_jar());
}

#[test]
fn file_info_classifies_plain_jar() {
    let info = FileInfo::new("1.16.5.jar");
    assert!(matches!(info.kind, LocalFileKind::Jar));
    assert!(info.is_jar());
    assert!(!info.is_zip());
}

#[test]
fn file_info_classifies_fabric_jar_by_prefix() {
    let info = FileInfo::new("fabric/jars/fabric-1.18.jar");
    assert!(matches!(info.kind, LocalFileKind::FabricJar));
}

#[test]
fn file_info_classifies_fabric_jar_by_path_segment() {
    let info = FileInfo::new("clients/fabric/jars/fabric-1.19.jar");
    assert!(matches!(info.kind, LocalFileKind::FabricJar));
}

#[test]
fn file_info_classifies_other() {
    let info = FileInfo::new("readme.txt");
    assert!(matches!(info.kind, LocalFileKind::Other));
}

#[test]
fn file_info_strips_misc_prefix_for_local_file() {
    let info = FileInfo::new("misc/optifine.jar");
    assert_eq!(info.local_file, "optifine.jar");
}

// ── FileInfo path helpers ─────────────────────────────────────────────────────

#[test]
fn file_info_destination_path_for_plain_jar() {
    let root = PathBuf::from("/root");
    let info = FileInfo::new("1.16.5.jar");
    assert_eq!(
        info.destination_path(&root),
        root.join("1.16.5").join("1.16.5.jar")
    );
}

#[test]
fn file_info_destination_path_for_fabric_jar() {
    let root = PathBuf::from("/root");
    let info = FileInfo::new("fabric/jars/fabric-1.18.jar");
    assert_eq!(
        info.destination_path(&root),
        root.join("fabric-1.18")
            .join("mods")
            .join("fabric-1.18.jar")
    );
}

#[test]
fn file_info_destination_path_for_zip() {
    let root = PathBuf::from("/root");
    let info = FileInfo::new("assets.zip");
    assert_eq!(info.destination_path(&root), root.join("assets.zip"));
}

#[test]
fn file_info_unzip_path_strips_zip_suffix() {
    let root = PathBuf::from("/root");
    let info = FileInfo::new("assets.zip");
    assert_eq!(info.unzip_path(&root), root.join("assets"));
}

#[test]
fn file_info_jar_path_for_fabric() {
    let root = PathBuf::from("/root");
    let info = FileInfo::new("fabric/jars/fabric-1.18.jar");
    assert_eq!(
        info.jar_path(&root),
        root.join("fabric-1.18")
            .join("mods")
            .join("fabric-1.18.jar")
    );
}

#[test]
fn file_info_jar_path_for_vanilla() {
    let root = PathBuf::from("/root");
    let info = FileInfo::new("1.8.jar");
    assert_eq!(info.jar_path(&root), root.join("1.8").join("1.8.jar"));
}

// ── Data::is_folder_healthy ───────────────────────────────────────────────────

#[test]
fn is_folder_healthy_returns_false_for_missing_folder() {
    let tmp = TempDir::new("healthy_missing");
    let data = Data::new(tmp.path().to_path_buf());
    assert!(!data.is_folder_healthy("nonexistent"));
}

#[test]
fn is_folder_healthy_returns_false_without_sentinel() {
    let tmp = TempDir::new("healthy_no_sentinel");
    fs::create_dir(tmp.path().join("folder")).unwrap();
    fs::write(tmp.path().join("folder").join("file.txt"), "hello").unwrap();
    let data = Data::new(tmp.path().to_path_buf());
    assert!(!data.is_folder_healthy("folder"));
}

#[test]
fn is_folder_healthy_returns_true_with_sentinel_and_content() {
    let tmp = TempDir::new("healthy_ok");
    let folder = tmp.path().join("assets");
    fs::create_dir(&folder).unwrap();
    fs::write(folder.join(".valid"), "").unwrap();
    fs::write(folder.join("pack.json"), "{}").unwrap();
    let data = Data::new(tmp.path().to_path_buf());
    assert!(data.is_folder_healthy("assets"));
}

// ── Data::verify_folder_integrity ────────────────────────────────────────────

#[test]
fn verify_folder_integrity_fails_without_manifest() {
    let tmp = TempDir::new("integrity_no_manifest");
    let folder = tmp.path().join("libs");
    fs::create_dir(&folder).unwrap();
    let data = Data::new(tmp.path().to_path_buf());
    assert!(!data.verify_folder_integrity("libs"));
}

#[test]
fn verify_folder_integrity_passes_with_correct_manifest() {
    let tmp = TempDir::new("integrity_ok");
    let folder = tmp.path().join("libs");
    fs::create_dir(&folder).unwrap();

    let file_content = b"test content";
    fs::write(folder.join("test.jar"), file_content).unwrap();

    let hash =
        crate::core::utils::hashing::calculate_hash(&folder.join("test.jar")).expect("hash failed");
    fs::write(folder.join("manifest.txt"), format!("test.jar:{hash}\n")).unwrap();

    let data = Data::new(tmp.path().to_path_buf());
    assert!(data.verify_folder_integrity("libs"));
}

#[test]
fn verify_folder_integrity_fails_with_wrong_hash() {
    let tmp = TempDir::new("integrity_bad_hash");
    let folder = tmp.path().join("libs");
    fs::create_dir(&folder).unwrap();
    fs::write(folder.join("test.jar"), b"content").unwrap();
    fs::write(
        folder.join("manifest.txt"),
        "test.jar:00000000000000000000000000000000\n",
    )
    .unwrap();
    let data = Data::new(tmp.path().to_path_buf());
    assert!(!data.verify_folder_integrity("libs"));
}

#[test]
fn verify_folder_integrity_fails_with_missing_file() {
    let tmp = TempDir::new("integrity_missing_file");
    let folder = tmp.path().join("libs");
    fs::create_dir(&folder).unwrap();
    fs::write(folder.join("manifest.txt"), "ghost.jar:abc123\n").unwrap();
    let data = Data::new(tmp.path().to_path_buf());
    assert!(!data.verify_folder_integrity("libs"));
}
