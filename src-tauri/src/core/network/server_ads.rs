use serde::Deserialize;
use std::path::Path;

use crate::core::network::api::API;
use crate::{log_error, log_info, log_warn};

const SERVER_ADS_URL: &str = "server-ads";

#[derive(Debug, Clone, Deserialize)]
pub struct ServerAdData {
    pub name: String,
    pub ip: String,
}

pub async fn fetch_server_ads() -> Vec<ServerAdData> {
    let Some(api) = API.as_ref() else {
        log_warn!("API not available, skipping server ads fetch");
        return vec![];
    };

    match api.json_async::<Vec<ServerAdData>>(SERVER_ADS_URL).await {
        Ok(ads) => {
            log_info!("Fetched {} server ad(s)", ads.len());
            ads
        }
        Err(e) => {
            log_warn!("Failed to fetch server ads: {}", e);
            vec![]
        }
    }
}

pub fn inject_servers_dat(path: &Path, ads: &[ServerAdData]) {
    if ads.is_empty() {
        return;
    }

    let existing = if path.exists() {
        read_existing_servers(path)
    } else {
        vec![]
    };

    let ad_ips: std::collections::HashSet<&str> = ads.iter().map(|a| a.ip.as_str()).collect();

    let user_servers: Vec<(String, String)> = existing
        .into_iter()
        .filter(|(_, ip)| !ad_ips.contains(ip.as_str()))
        .collect();

    let mut all_servers: Vec<(String, String)> =
        ads.iter().map(|a| (a.name.clone(), a.ip.clone())).collect();
    all_servers.extend(user_servers);

    match write_servers_dat(path, &all_servers) {
        Ok(_) => log_info!("Injected {} server(s) into servers.dat", ads.len()),
        Err(e) => log_error!("Failed to write servers.dat: {}", e),
    }
}

fn read_existing_servers(path: &Path) -> Vec<(String, String)> {
    use flate2::read::GzDecoder;
    use std::io::Read;

    let file = match std::fs::read(path) {
        Ok(b) => b,
        Err(e) => {
            log_warn!("Could not read existing servers.dat: {}", e);
            return vec![];
        }
    };

    // Check if the file starts with the GZIP magic number (0x1f, 0x8b)
    // This allows us to recover data if it was incorrectly compressed by previous runs
    if file.len() >= 2 && file[0] == 0x1f && file[1] == 0x8b {
        let mut decoder = GzDecoder::new(file.as_slice());
        let mut nbt = Vec::new();
        if decoder.read_to_end(&mut nbt).is_ok() {
            return parse_nbt_servers(&nbt);
        } else {
            log_warn!("servers.dat has gzip magic but failed to decode, will overwrite");
            return vec![];
        }
    }

    // Minecraft natively uses uncompressed NBT for servers.dat
    parse_nbt_servers(&file)
}

fn parse_nbt_servers(data: &[u8]) -> Vec<(String, String)> {
    let mut pos = 0usize;

    if data.len() < 3 || data[pos] != 10 {
        return vec![];
    }
    pos += 1;
    let root_name_len = u16::from_be_bytes([data[pos], data[pos + 1]]) as usize;
    pos += 2 + root_name_len;

    loop {
        if pos >= data.len() {
            return vec![];
        }
        let tag_type = data[pos];
        pos += 1;

        if tag_type == 0 {
            return vec![];
        }
        if pos + 2 > data.len() {
            return vec![];
        }
        let name_len = u16::from_be_bytes([data[pos], data[pos + 1]]) as usize;
        pos += 2;
        if pos + name_len > data.len() {
            return vec![];
        }
        let name = std::str::from_utf8(&data[pos..pos + name_len]).unwrap_or("");
        pos += name_len;

        if tag_type == 9 && name == "servers" {
            if pos + 5 > data.len() {
                return vec![];
            }
            let _elem_type = data[pos];
            pos += 1;
            let count = i32::from_be_bytes([data[pos], data[pos + 1], data[pos + 2], data[pos + 3]])
                as usize;
            pos += 4;

            let mut servers = Vec::with_capacity(count);
            for _ in 0..count {
                let (entry, next_pos) = parse_compound_entry(data, pos);
                pos = next_pos;
                if let Some((n, ip)) = entry {
                    servers.push((n, ip));
                }
            }
            return servers;
        }

        pos = skip_tag_payload(data, pos, tag_type);
    }
}

fn parse_compound_entry(data: &[u8], mut pos: usize) -> (Option<(String, String)>, usize) {
    let mut name: Option<String> = None;
    let mut ip: Option<String> = None;

    loop {
        if pos >= data.len() {
            break;
        }
        let tag_type = data[pos];
        pos += 1;

        if tag_type == 0 {
            break;
        }

        if pos + 2 > data.len() {
            break;
        }
        let name_len = u16::from_be_bytes([data[pos], data[pos + 1]]) as usize;
        pos += 2;
        if pos + name_len > data.len() {
            break;
        }
        let key = std::str::from_utf8(&data[pos..pos + name_len])
            .unwrap_or("")
            .to_string();
        pos += name_len;

        if tag_type == 8 {
            if pos + 2 > data.len() {
                break;
            }
            let val_len = u16::from_be_bytes([data[pos], data[pos + 1]]) as usize;
            pos += 2;
            if pos + val_len > data.len() {
                break;
            }
            let val = std::str::from_utf8(&data[pos..pos + val_len])
                .unwrap_or("")
                .to_string();
            pos += val_len;

            match key.as_str() {
                "name" => name = Some(val),
                "ip" => ip = Some(val),
                _ => {}
            }
        } else {
            pos = skip_tag_payload(data, pos, tag_type);
        }
    }

    let entry = match (name, ip) {
        (Some(n), Some(i)) => Some((n, i)),
        _ => None,
    };
    (entry, pos)
}

fn skip_tag_payload(data: &[u8], mut pos: usize, tag_type: u8) -> usize {
    match tag_type {
        1 => pos + 1,
        2 => pos + 2,
        3 => pos + 4,
        4 => pos + 8,
        5 => pos + 4,
        6 => pos + 8,
        7 => {
            if pos + 4 <= data.len() {
                let len =
                    i32::from_be_bytes([data[pos], data[pos + 1], data[pos + 2], data[pos + 3]])
                        as usize;
                pos + 4 + len
            } else {
                data.len()
            }
        }
        8 => {
            if pos + 2 <= data.len() {
                let len = u16::from_be_bytes([data[pos], data[pos + 1]]) as usize;
                pos + 2 + len
            } else {
                data.len()
            }
        }
        9 => {
            if pos + 5 <= data.len() {
                let elem_type = data[pos];
                let count = i32::from_be_bytes([
                    data[pos + 1],
                    data[pos + 2],
                    data[pos + 3],
                    data[pos + 4],
                ]) as usize;
                pos += 5;
                for _ in 0..count {
                    pos = skip_tag_payload(data, pos, elem_type);
                }
                pos
            } else {
                data.len()
            }
        }
        10 => {
            loop {
                if pos >= data.len() {
                    break;
                }
                let inner_type = data[pos];
                pos += 1;
                if inner_type == 0 {
                    break;
                }
                if pos + 2 > data.len() {
                    break;
                }
                let name_len = u16::from_be_bytes([data[pos], data[pos + 1]]) as usize;
                pos += 2 + name_len;
                pos = skip_tag_payload(data, pos, inner_type);
            }
            pos
        }
        11 => {
            if pos + 4 <= data.len() {
                let len =
                    i32::from_be_bytes([data[pos], data[pos + 1], data[pos + 2], data[pos + 3]])
                        as usize;
                pos + 4 + len * 4
            } else {
                data.len()
            }
        }
        12 => {
            if pos + 4 <= data.len() {
                let len =
                    i32::from_be_bytes([data[pos], data[pos + 1], data[pos + 2], data[pos + 3]])
                        as usize;
                pos + 4 + len * 8
            } else {
                data.len()
            }
        }
        _ => data.len(),
    }
}

fn write_servers_dat(path: &Path, servers: &[(String, String)]) -> std::io::Result<()> {
    let nbt = build_nbt(servers);

    // Writes the file as uncompressed NBT which Minecraft demands
    std::fs::write(path, nbt)
}

fn build_nbt(servers: &[(String, String)]) -> Vec<u8> {
    let mut buf = Vec::new();

    push_tag_header(&mut buf, 10, "");

    push_tag_header(&mut buf, 9, "servers");
    buf.push(10);
    let count = servers.len() as i32;
    buf.extend_from_slice(&count.to_be_bytes());

    for (name, ip) in servers {
        push_tag_header(&mut buf, 8, "name");
        push_nbt_string(&mut buf, name);

        push_tag_header(&mut buf, 8, "ip");
        push_nbt_string(&mut buf, ip);

        buf.push(0);
    }

    buf.push(0);

    buf
}

fn push_tag_header(buf: &mut Vec<u8>, tag_type: u8, name: &str) {
    buf.push(tag_type);
    push_nbt_string(buf, name);
}

fn push_nbt_string(buf: &mut Vec<u8>, s: &str) {
    let bytes = s.as_bytes();
    let len = bytes.len() as u16;
    buf.extend_from_slice(&len.to_be_bytes());
    buf.extend_from_slice(bytes);
}
