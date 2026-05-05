use crate::commands::utils::{decode_base64, dir_size, encode_base64, get_version};
use std::fs;
use std::path::{Path, PathBuf};

struct TempDir(PathBuf);

impl TempDir {
    fn new(name: &str) -> Self {
        let path = std::env::temp_dir()
            .join(format!("collapseloader_utils_test_{name}_{}", std::process::id()));
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

#[tokio::test]
async fn encode_and_decode_base64_round_trip() {
    let encoded = encode_base64("CollapseLoader".to_string())
        .await
        .expect("encode should succeed");
    let decoded = decode_base64(encoded)
        .await
        .expect("decode should succeed");

    assert_eq!(decoded, "CollapseLoader");
}

#[tokio::test]
async fn decode_base64_rejects_invalid_payload() {
    let error = decode_base64("not-base64!!!".to_string())
        .await
        .expect_err("invalid payload should fail");

    assert_eq!(error, "Failed to decode base64");
}

#[tokio::test]
async fn decode_base64_rejects_invalid_utf8() {
    let error = decode_base64("//4=".to_string())
        .await
        .expect_err("invalid UTF-8 should fail");

    assert_eq!(error, "Failed to decode base64 to UTF-8 string");
}

#[test]
fn get_version_returns_expected_metadata_keys() {
    let version = get_version().expect("version command should succeed");

    assert!(version.get("version").is_some());
    assert!(version.get("codename").is_some());
    assert!(version.get("commitHash").is_some());
    assert!(version.get("commitMessage").is_some());
    assert!(version.get("branch").is_some());
    assert!(version.get("development").is_some());
}

#[test]
fn dir_size_sums_nested_files() {
    let temp_dir = TempDir::new("dir_size");
    let nested_dir = temp_dir.path().join("nested");

    fs::create_dir_all(&nested_dir).expect("nested dir should be created");
    fs::write(temp_dir.path().join("root.txt"), [1u8; 5]).expect("root file should exist");
    fs::write(nested_dir.join("child.txt"), [2u8; 7]).expect("nested file should exist");

    assert_eq!(dir_size(temp_dir.path()), 12);
}