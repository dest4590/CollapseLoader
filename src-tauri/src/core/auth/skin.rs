use base64::{engine::general_purpose::STANDARD, Engine as _};
use serde::{Deserialize, Serialize};

const PROFILE_URL: &str = "https://api.minecraftservices.com/minecraft/profile";
const SKIN_UPLOAD_URL: &str = "https://api.minecraftservices.com/minecraft/profile/skins";
const ACTIVE_SKIN_URL: &str = "https://api.minecraftservices.com/minecraft/profile/skins/active";
const ACTIVE_CAPE_URL: &str = "https://api.minecraftservices.com/minecraft/profile/capes/active";
const USER_AGENT: &str = concat!("CollapseLoader/", env!("CARGO_PKG_VERSION"));
const MAX_SKIN_BYTES: usize = 4 * 1024 * 1024;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SkinProfile {
    pub id: String,
    pub name: String,
    pub skin_url: Option<String>,
    pub skin_variant: String,
    pub capes: Vec<SkinCape>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SkinCape {
    pub id: String,
    pub url: String,
    pub alias: Option<String>,
    pub active: bool,
}

#[derive(Deserialize)]
struct RawProfile {
    id: String,
    name: String,
    #[serde(default)]
    skins: Vec<RawSkin>,
    #[serde(default)]
    capes: Vec<RawCape>,
}

#[derive(Deserialize)]
struct RawSkin {
    url: String,
    #[serde(default)]
    variant: String,
    #[serde(default)]
    state: String,
}

#[derive(Deserialize)]
struct RawCape {
    id: String,
    url: String,
    #[serde(default)]
    alias: Option<String>,
    #[serde(default)]
    state: String,
}

fn http_client() -> Result<reqwest::Client, String> {
    reqwest::Client::builder()
        .user_agent(USER_AGENT)
        .build()
        .map_err(|error| format!("Failed to create HTTP client: {error}"))
}

fn limited_body(body: &str) -> String {
    const MAX_CHARS: usize = 500;
    if body.chars().count() <= MAX_CHARS {
        return body.to_string();
    }
    let truncated: String = body.chars().take(MAX_CHARS).collect();
    format!("{truncated}...")
}

async fn parse_profile_response(
    response: reqwest::Response,
    access_token: &str,
    action: &str,
) -> Result<SkinProfile, String> {
    let status = response.status();
    let body = response.text().await.map_err(|error| {
        format!("Failed to read the response after trying to {action}: {error}")
    })?;
    if !status.is_success() {
        return Err(format!(
            "Failed to {action} (HTTP {status}): {}",
            limited_body(&body)
        ));
    }
    if let Ok(raw) = serde_json::from_str::<RawProfile>(&body) {
        return Ok(profile_from_raw(raw));
    }
    fetch_profile(access_token).await
}

pub fn normalize_variant(variant: &str) -> Result<String, String> {
    match variant.trim().to_ascii_lowercase().as_str() {
        "classic" => Ok("classic".to_string()),
        "slim" => Ok("slim".to_string()),
        _ => Err("Skin model must be either classic or slim.".to_string()),
    }
}

pub fn validate_skin_bytes(bytes: &[u8]) -> Result<(), String> {
    if bytes.is_empty() {
        return Err("The selected skin file is empty.".to_string());
    }
    if bytes.len() > MAX_SKIN_BYTES {
        return Err("The selected skin file is too large. Maximum size is 4 MB.".to_string());
    }
    if !bytes.starts_with(&[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A]) {
        return Err("The selected file is not a valid PNG image.".to_string());
    }
    if bytes.len() < 24 {
        return Err("The selected PNG image is corrupted.".to_string());
    }
    let width = u32::from_be_bytes([bytes[16], bytes[17], bytes[18], bytes[19]]);
    let height = u32::from_be_bytes([bytes[20], bytes[21], bytes[22], bytes[23]]);
    if width != 64 || height != 64 {
        return Err("Skin must be a 64x64 PNG image.".to_string());
    }
    Ok(())
}

fn read_skin_file(path: &str) -> Result<Vec<u8>, String> {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return Err("Skin file path cannot be empty.".to_string());
    }
    let bytes = std::fs::read(trimmed)
        .map_err(|error| format!("Failed to read the selected skin file: {error}"))?;
    validate_skin_bytes(&bytes)?;
    Ok(bytes)
}

fn secure_texture_url(url: &str) -> String {
    url.replacen(
        "http://textures.minecraft.net",
        "https://textures.minecraft.net",
        1,
    )
}

fn profile_from_raw(raw: RawProfile) -> SkinProfile {
    let active_skin = raw
        .skins
        .iter()
        .find(|skin| skin.state.eq_ignore_ascii_case("ACTIVE"))
        .or_else(|| raw.skins.first());
    let skin_url = active_skin.map(|skin| secure_texture_url(&skin.url));
    let skin_variant = active_skin
        .map(|skin| skin.variant.to_ascii_lowercase())
        .filter(|variant| variant == "slim")
        .unwrap_or_else(|| "classic".to_string());
    let capes = raw
        .capes
        .iter()
        .map(|cape| SkinCape {
            id: cape.id.clone(),
            url: secure_texture_url(&cape.url),
            alias: cape.alias.clone(),
            active: cape.state.eq_ignore_ascii_case("ACTIVE"),
        })
        .collect();

    SkinProfile {
        id: raw.id,
        name: raw.name,
        skin_url,
        skin_variant,
        capes,
    }
}

pub async fn download_texture(texture_key: &str) -> Result<Vec<u8>, String> {
    if texture_key.len() != 64 || !texture_key.chars().all(|value| value.is_ascii_hexdigit()) {
        return Err("Invalid skin texture key.".to_string());
    }
    let url = format!("https://textures.minecraft.net/texture/{texture_key}");
    let response = http_client()?
        .get(&url)
        .send()
        .await
        .map_err(|error| format!("Failed to download the skin texture: {error}"))?;
    let status = response.status();
    if !status.is_success() {
        return Err(format!(
            "Failed to download the skin texture (HTTP {status})."
        ));
    }
    let bytes = response
        .bytes()
        .await
        .map_err(|error| format!("Failed to read the skin texture: {error}"))?
        .to_vec();
    validate_skin_bytes(&bytes)?;
    Ok(bytes)
}

pub fn decode_texture_data(texture_data: &str) -> Result<Vec<u8>, String> {
    let encoded = texture_data
        .strip_prefix("data:image/png;base64,")
        .ok_or_else(|| "Invalid skin texture data.".to_string())?;
    let bytes = STANDARD
        .decode(encoded)
        .map_err(|error| format!("Failed to decode the skin texture: {error}"))?;
    validate_skin_bytes(&bytes)?;
    Ok(bytes)
}

pub async fn fetch_profile(access_token: &str) -> Result<SkinProfile, String> {
    let client = http_client()?;
    let response = client
        .get(PROFILE_URL)
        .bearer_auth(access_token)
        .header(reqwest::header::ACCEPT, "application/json")
        .send()
        .await
        .map_err(|error| format!("Failed to load the Minecraft profile: {error}"))?;
    let status = response.status();
    let body = response
        .text()
        .await
        .map_err(|error| format!("Failed to read the Minecraft profile response: {error}"))?;
    if !status.is_success() {
        return Err(format!(
            "Minecraft services rejected the profile request (HTTP {status}): {}",
            limited_body(&body)
        ));
    }
    let raw: RawProfile = serde_json::from_str(&body)
        .map_err(|error| format!("Invalid Minecraft profile response: {error}"))?;
    Ok(profile_from_raw(raw))
}

pub async fn upload_skin(
    access_token: &str,
    file_path: &str,
    variant: &str,
) -> Result<SkinProfile, String> {
    let variant = normalize_variant(variant)?;
    let bytes = read_skin_file(file_path)?;
    upload_skin_bytes(access_token, bytes, &variant).await
}

pub async fn upload_skin_bytes(
    access_token: &str,
    bytes: Vec<u8>,
    variant: &str,
) -> Result<SkinProfile, String> {
    validate_skin_bytes(&bytes)?;
    let variant = normalize_variant(variant)?;
    let client = http_client()?;
    let part = reqwest::multipart::Part::bytes(bytes)
        .file_name("skin.png")
        .mime_str("image/png")
        .map_err(|error| format!("Failed to prepare the skin file: {error}"))?;
    let form = reqwest::multipart::Form::new()
        .text("variant", variant)
        .part("file", part);
    let response = client
        .post(SKIN_UPLOAD_URL)
        .bearer_auth(access_token)
        .header(reqwest::header::ACCEPT, "application/json")
        .multipart(form)
        .send()
        .await
        .map_err(|error| format!("Failed to upload the skin: {error}"))?;
    parse_profile_response(response, access_token, "upload the skin").await
}

pub async fn reset_skin(access_token: &str) -> Result<SkinProfile, String> {
    let client = http_client()?;
    let response = client
        .delete(ACTIVE_SKIN_URL)
        .bearer_auth(access_token)
        .send()
        .await
        .map_err(|error| format!("Failed to reset the skin: {error}"))?;
    parse_profile_response(response, access_token, "reset the skin").await
}

pub async fn set_cape(access_token: &str, cape_id: Option<&str>) -> Result<SkinProfile, String> {
    let client = http_client()?;
    let response = match cape_id {
        Some(cape_id) => {
            let cape_id = cape_id.trim();
            if cape_id.is_empty() {
                return Err("Cape ID cannot be empty.".to_string());
            }
            client
                .put(ACTIVE_CAPE_URL)
                .bearer_auth(access_token)
                .json(&serde_json::json!({ "capeId": cape_id }))
                .send()
                .await
                .map_err(|error| format!("Failed to change the cape: {error}"))?
        }
        None => client
            .delete(ACTIVE_CAPE_URL)
            .bearer_auth(access_token)
            .send()
            .await
            .map_err(|error| format!("Failed to remove the cape: {error}"))?,
    };
    parse_profile_response(response, access_token, "change the cape").await
}

#[cfg(test)]
mod tests {
    use super::*;

    fn png_header(width: u32, height: u32) -> Vec<u8> {
        let mut bytes = vec![0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];
        bytes.extend_from_slice(&13_u32.to_be_bytes());
        bytes.extend_from_slice(b"IHDR");
        bytes.extend_from_slice(&width.to_be_bytes());
        bytes.extend_from_slice(&height.to_be_bytes());
        bytes
    }

    #[test]
    fn accepts_64_pixel_skins() {
        assert!(validate_skin_bytes(&png_header(64, 64)).is_ok());
    }

    #[test]
    fn rejects_unsupported_sizes() {
        assert!(validate_skin_bytes(&png_header(128, 128)).is_err());
        assert!(validate_skin_bytes(&png_header(64, 32)).is_err());
        assert!(validate_skin_bytes(&png_header(32, 32)).is_err());
    }

    #[test]
    fn rejects_non_png_files() {
        assert!(validate_skin_bytes(b"not a png").is_err());
    }

    #[test]
    fn decodes_skin_texture_data() {
        let encoded = STANDARD.encode(png_header(64, 64));
        let data = format!("data:image/png;base64,{encoded}");
        assert!(decode_texture_data(&data).is_ok());
        assert!(decode_texture_data(&encoded).is_err());
        assert!(decode_texture_data("data:image/png;base64,not base64").is_err());
    }

    #[test]
    fn upgrades_texture_urls_to_https() {
        assert_eq!(
            secure_texture_url("http://textures.minecraft.net/texture/abc"),
            "https://textures.minecraft.net/texture/abc"
        );
        assert_eq!(
            secure_texture_url("https://textures.minecraft.net/texture/abc"),
            "https://textures.minecraft.net/texture/abc"
        );
    }

    #[test]
    fn normalizes_skin_variants() {
        assert_eq!(normalize_variant("SLIM").unwrap(), "slim");
        assert_eq!(normalize_variant(" classic ").unwrap(), "classic");
        assert!(normalize_variant("wide").is_err());
    }
}
