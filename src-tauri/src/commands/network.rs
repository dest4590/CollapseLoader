use crate::core::network::get_api_client;
use crate::core::storage::data::DATA;
use crate::log_error;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::{Mutex, OnceLock};
use tauri::{AppHandle, Emitter};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NetworkRequest {
    pub id: String,
    pub method: String,
    pub url: String,
    pub status: Option<u16>,
    pub duration: Option<u64>,
    pub timestamp: u64,
    pub request_headers: Option<std::collections::HashMap<String, String>>,
    pub request_body: Option<serde_json::Value>,
    pub response_headers: Option<std::collections::HashMap<String, String>>,
    pub response_size: Option<u64>,
    pub response_body: Option<serde_json::Value>,
    pub response_text: Option<String>,
    pub error_message: Option<String>,
}

static NETWORK_HISTORY: OnceLock<Mutex<VecDeque<NetworkRequest>>> = OnceLock::new();

fn get_history() -> &'static Mutex<VecDeque<NetworkRequest>> {
    NETWORK_HISTORY.get_or_init(|| Mutex::new(VecDeque::new()))
}

fn with_network_history<R>(operation: impl FnOnce(&mut VecDeque<NetworkRequest>) -> R) -> R {
    let mut history = get_history().lock().unwrap();
    operation(&mut history)
}

fn emit_network_event(app_handle: &AppHandle, event: &str, record: &NetworkRequest) {
    let _ = app_handle.emit(event, record);
}

fn mask_auth_headers(
    mut headers: Option<std::collections::HashMap<String, String>>,
) -> Option<std::collections::HashMap<String, String>> {
    if let Some(ref mut h) = headers {
        for (k, v) in h.iter_mut() {
            if k.eq_ignore_ascii_case("authorization") {
                if v.len() >= 7 && v[..7].eq_ignore_ascii_case("bearer ") {
                    *v = "Bearer ****".to_string();
                } else {
                    *v = "****".to_string();
                }
            }
        }
    }
    headers
}

const MAX_HISTORY_TEXT_LEN: usize = 4_000;

fn truncate_for_history(text: &str) -> String {
    if text.len() <= MAX_HISTORY_TEXT_LEN {
        text.to_string()
    } else {
        let mut end = MAX_HISTORY_TEXT_LEN;
        while !text.is_char_boundary(end) {
            end -= 1;
        }
        format!("{}… [truncated {} chars]", &text[..end], text.len() - end)
    }
}

#[tauri::command]
pub async fn api_request(
    method: String,
    url: String,
    headers: Option<std::collections::HashMap<String, String>>,
    body: Option<serde_json::Value>,
    app_handle: AppHandle,
) -> Result<serde_json::Value, String> {
    let client = get_api_client();

    let start = std::time::Instant::now();
    let id = uuid::Uuid::new_v4().to_string();
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64;

    let masked_headers = mask_auth_headers(headers.clone());

    let initial_request = NetworkRequest {
        id: id.clone(),
        method: method.clone(),
        url: url.clone(),
        status: None,
        duration: None,
        timestamp,
        request_headers: masked_headers.clone(),
        request_body: body.clone(),
        response_headers: None,
        response_size: None,
        response_body: None,
        response_text: None,
        error_message: None,
    };

    emit_network_event(&app_handle, "network-request", &initial_request);

    let req_method = reqwest::Method::from_bytes(method.to_ascii_uppercase().as_bytes())
        .map_err(|_| format!("Unsupported method: {}", method))?;
    let mut rb = client.request(req_method, &url);

    if let Some(ref h) = headers {
        for (k, v) in h {
            rb = rb.header(k, v);
        }
    }

    if let Some(ref b) = body {
        if !b.is_null() {
            let mut is_form_data = false;
            if let Some(obj) = b.as_object() {
                for (_, val) in obj {
                    if let Some(val_obj) = val.as_object() {
                        if val_obj.get("__type").and_then(|v| v.as_str()) == Some("file") {
                            is_form_data = true;
                            break;
                        }
                    }
                }
            }

            if is_form_data {
                let mut form = reqwest::multipart::Form::new();
                if let Some(obj) = b.as_object() {
                    for (key, val) in obj {
                        if let Some(val_obj) = val.as_object() {
                            if val_obj.get("__type").and_then(|v| v.as_str()) == Some("file") {
                                let name = val_obj
                                    .get("name")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("file");
                                let mime = val_obj
                                    .get("type")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("application/octet-stream");
                                let data_b64 =
                                    val_obj.get("data").and_then(|v| v.as_str()).unwrap_or("");
                                let data = base64::Engine::decode(
                                    &base64::engine::general_purpose::STANDARD,
                                    data_b64,
                                )
                                .map_err(|e| format!("Failed to decode base64: {}", e))?;

                                let part = reqwest::multipart::Part::bytes(data)
                                    .file_name(name.to_string())
                                    .mime_str(mime)
                                    .map_err(|e| format!("Invalid mime type: {}", e))?;
                                form = form.part(key.clone(), part);
                            } else {
                                form = form.text(key.clone(), val.to_string());
                            }
                        } else {
                            form = form.text(key.clone(), val.to_string());
                        }
                    }
                }
                rb = rb.multipart(form);
            } else {
                rb = rb.json(b);
            }
        }
    }

    let response = match rb.send().await {
        Ok(res) => res,
        Err(e) => {
            let rec = NetworkRequest {
                id,
                method,
                url,
                status: Some(0),
                duration: Some(start.elapsed().as_millis() as u64),
                timestamp,
                request_headers: masked_headers,
                request_body: body,
                response_headers: None,
                response_size: None,
                response_body: None,
                response_text: None,
                error_message: Some(e.to_string()),
            };
            emit_network_event(&app_handle, "network-response", &rec);
            save_request_history(rec);
            return Err(e.to_string());
        }
    };

    let status = response.status().as_u16();
    let duration = start.elapsed().as_millis() as u64;
    let mut resp_headers_map = std::collections::HashMap::with_capacity(response.headers().len());

    for (k, v) in response.headers().iter() {
        resp_headers_map.insert(k.to_string(), v.to_str().unwrap_or_default().to_string());
    }

    let text = response.text().await.map_err(|e| e.to_string())?;
    let response_size = Some(text.len() as u64);
    let mut response_json: Option<serde_json::Value> = None;

    if !text.is_empty() {
        if let Ok(j) = serde_json::from_str::<serde_json::Value>(&text) {
            response_json = Some(j);
        }
    }

    let rec = NetworkRequest {
        id,
        method,
        url,
        status: Some(status),
        duration: Some(duration),
        timestamp,
        request_headers: masked_headers,
        request_body: body,
        response_headers: Some(resp_headers_map),
        response_size,
        response_body: response_json.clone(),
        response_text: if response_json.is_none() {
            Some(truncate_for_history(&text))
        } else {
            None
        },
        error_message: None,
    };

    emit_network_event(&app_handle, "network-response", &rec);
    save_request_history(rec);

    if let Some(j) = response_json {
        Ok(j)
    } else {
        Ok(serde_json::json!(text))
    }
}

const MAX_CACHED_IMAGE_BYTES: usize = 20 * 1024 * 1024;

fn marketplace_image_cache_dir() -> std::path::PathBuf {
    let root = DATA
        .root_dir
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clone();
    root.join("cache").join("marketplace-images")
}

#[tauri::command]
pub async fn fetch_cached_image(url: String) -> Result<String, String> {
    use base64::Engine;
    use sha2::{Digest, Sha256};

    let parsed = reqwest::Url::parse(&url).map_err(|e| format!("Invalid image URL: {e}"))?;
    if !matches!(parsed.scheme(), "http" | "https") {
        return Err("Only http(s) image URLs are supported".to_string());
    }

    let cache_dir = marketplace_image_cache_dir();
    let cache_key = Sha256::digest(url.as_bytes())
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect::<String>();
    let cache_path = cache_dir.join(cache_key);

    if let Ok(cached) = tokio::fs::read_to_string(&cache_path).await {
        if !cached.is_empty() {
            return Ok(cached);
        }
    }

    tokio::fs::create_dir_all(&cache_dir)
        .await
        .map_err(|e| format!("Failed to create image cache directory: {e}"))?;

    let response = get_api_client()
        .get(&url)
        .header(reqwest::header::ACCEPT, "image/*")
        .send()
        .await
        .map_err(|e| format!("Failed to download image: {e}"))?;

    if !response.status().is_success() {
        return Err(format!(
            "Failed to download image: HTTP {}",
            response.status().as_u16()
        ));
    }

    if response
        .content_length()
        .is_some_and(|length| length as usize > MAX_CACHED_IMAGE_BYTES)
    {
        return Err("Image is too large to cache".to_string());
    }

    let content_type = response
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.split(';').next())
        .map(str::trim)
        .filter(|value| value.starts_with("image/"))
        .unwrap_or("image/jpeg")
        .to_string();

    let bytes = response
        .bytes()
        .await
        .map_err(|e| format!("Failed to read image data: {e}"))?;

    if bytes.len() > MAX_CACHED_IMAGE_BYTES {
        return Err("Image is too large to cache".to_string());
    }

    let data_url = format!(
        "data:{content_type};base64,{}",
        base64::engine::general_purpose::STANDARD.encode(&bytes)
    );

    if let Err(e) = tokio::fs::write(&cache_path, &data_url).await {
        log_error!("Failed to write marketplace image cache: {}", e);
    }

    Ok(data_url)
}

#[tauri::command]
pub fn clear_network_history() {
    with_network_history(|history| history.clear());
}

#[tauri::command]
pub fn get_network_history() -> Result<Vec<NetworkRequest>, String> {
    Ok(with_network_history(|history| {
        history.iter().cloned().collect()
    }))
}

fn save_request_history(rec: NetworkRequest) {
    with_network_history(|history| {
        history.push_back(rec);
        if history.len() > 1000 {
            history.pop_front();
        }
    });
}
