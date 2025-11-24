use std::fs;
use std::path::Path;

pub fn calculate_md5_hash(path: &Path) -> Result<String, String> {
    let bytes = fs::read(path).map_err(|e| format!("Failed to read file for hashing: {e}"))?;
    let digest = md5::compute(&bytes);
    Ok(format!("{digest:x}"))
}
