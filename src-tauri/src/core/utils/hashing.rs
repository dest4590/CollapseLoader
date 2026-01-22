use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

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

    let digest = context.finalize();
    Ok(format!("{digest:x}"))
}

fn bytes_to_lower_hex(bytes: &[u8]) -> String {
    const LUT: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        out.push(LUT[(b >> 4) as usize] as char);
        out.push(LUT[(b & 0x0f) as usize] as char);
    }
    out
}

pub fn calculate_hash(path: &Path) -> Result<String, String> {
    let file = File::open(path).map_err(|e| format!("Failed to open file for hashing: {e}"))?;
    let mut reader = BufReader::new(file);
    let mut hasher = Sha256::new();
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

    let digest = hasher.finalize();
    Ok(bytes_to_lower_hex(digest.as_slice()))
}
