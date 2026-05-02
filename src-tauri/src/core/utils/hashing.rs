use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

fn calculate_file_hash<D: Digest>(path: &Path) -> Result<String, String> {
    let file = File::open(path).map_err(|e| format!("Failed to open file for hashing: {e}"))?;
    let mut reader = BufReader::new(file);
    let mut hasher = D::new();
    let mut buffer = [0; 8192];

    loop {
        let count = reader
            .read(&mut buffer)
            .map_err(|e| format!("Failed to read file: {e}"))?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
    }

    Ok(hasher
        .finalize()
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect::<String>())
}

pub fn calculate_md5_hash(path: &Path) -> Result<String, String> {
    let file = File::open(path).map_err(|e| format!("Failed to open file for hashing: {e}"))?;
    let mut reader = BufReader::new(file);
    let mut context = md5::Context::new();
    let mut buffer = [0; 8192];

    loop {
        let count = reader
            .read(&mut buffer)
            .map_err(|e| format!("Failed to read file: {e}"))?;
        if count == 0 {
            break;
        }
        context.consume(&buffer[..count]);
    }

    Ok(format!("{:x}", context.finalize()))
}

pub fn calculate_hash(path: &Path) -> Result<String, String> {
    calculate_file_hash::<Sha256>(path)
}
