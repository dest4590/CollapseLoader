use base64::prelude::{Engine, BASE64_STANDARD, BASE64_URL_SAFE_NO_PAD};
use chrono::{DateTime, Duration as ChronoDuration, Utc};
use p256::ecdsa::signature::Signer;
use p256::ecdsa::{Signature, SigningKey, VerifyingKey};
use p256::elliptic_curve::rand_core::OsRng;
use rand::Rng;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::Arc;
use tauri::{Manager, WebviewUrl, WebviewWindowBuilder};
use tokio::time::{sleep, Duration};

const MICROSOFT_CLIENT_ID: &str = "00000000402b5328";
const AUTH_REPLY_HOST: &str = "login.live.com";
const AUTH_REPLY_PATH: &str = "/oauth20_desktop.srf";
const REQUESTED_SCOPE: &str = "service::user.auth.xboxlive.com::MBI_SSL";
const SIGN_IN_WINDOW_LABEL: &str = "microsoft-signin";
const MINECRAFT_SERVICES_USER_AGENT: &str = "CollapseLoader/1.3.0";

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MinecraftAccountData {
    pub username: String,
    pub uuid: String,
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_at: String,
}

#[derive(Deserialize)]
struct OAuthTokenResponse {
    access_token: String,
    refresh_token: String,
    expires_in: u64,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
struct DeviceToken {
    token: String,
    display_claims: HashMap<String, serde_json::Value>,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct SisuRedirect {
    msa_oauth_redirect: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct SisuAuthorizeResponse {
    title_token: DeviceToken,
    user_token: DeviceToken,
}

#[derive(Deserialize)]
struct MinecraftTokenResponse {
    access_token: String,
}

#[derive(Deserialize)]
struct MinecraftProfile {
    id: String,
    name: String,
}

struct DeviceKey {
    id: uuid::Uuid,
    signing_key: SigningKey,
    x: String,
    y: String,
}

struct DatedResponse<T> {
    date: DateTime<Utc>,
    value: T,
}

struct SignedResponse<T> {
    headers: reqwest::header::HeaderMap,
    date: DateTime<Utc>,
    value: T,
}

fn endpoint(host: &str, path: &str) -> String {
    ["https://", host, path].concat()
}

fn auth_reply_url() -> String {
    endpoint(AUTH_REPLY_HOST, AUTH_REPLY_PATH)
}

fn random_url_safe(byte_count: usize) -> String {
    let mut bytes = vec![0_u8; byte_count];
    rand::rng().fill_bytes(&mut bytes);
    BASE64_URL_SAFE_NO_PAD.encode(bytes)
}

fn generate_device_key() -> Result<DeviceKey, String> {
    let signing_key = SigningKey::random(&mut OsRng);
    let public_key = VerifyingKey::from(&signing_key);
    let point = public_key.to_encoded_point(false);
    let x = point
        .x()
        .ok_or_else(|| "Failed to generate Xbox proof key.".to_string())?;
    let y = point
        .y()
        .ok_or_else(|| "Failed to generate Xbox proof key.".to_string())?;

    Ok(DeviceKey {
        id: uuid::Uuid::new_v4(),
        signing_key,
        x: BASE64_URL_SAFE_NO_PAD.encode(x),
        y: BASE64_URL_SAFE_NO_PAD.encode(y),
    })
}

fn proof_key_json(key: &DeviceKey) -> serde_json::Value {
    serde_json::json!({
        "kty": "EC",
        "x": key.x,
        "y": key.y,
        "crv": "P-256",
        "alg": "ES256",
        "use": "sig"
    })
}

pub async fn login(app: tauri::AppHandle) -> Result<MinecraftAccountData, String> {
    let http = reqwest::Client::new();
    let key = generate_device_key()?;
    let device = request_device_token(&http, &key, Utc::now()).await?;

    let verifier = random_url_safe(64);
    let challenge = BASE64_URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes()));
    let expected_state = random_url_safe(32);
    let (session_id, authorization_url) = sisu_authenticate(
        &http,
        &device.value.token,
        &challenge,
        &expected_state,
        "select_account",
        &key,
        device.date,
    )
    .await?;

    if let Some(existing) = app.get_webview_window(SIGN_IN_WINDOW_LABEL) {
        let _ = existing.close();
    }

    let window = WebviewWindowBuilder::new(
        &app,
        SIGN_IN_WINDOW_LABEL,
        WebviewUrl::External(authorization_url),
    )
    .title("Sign in to Microsoft")
    .always_on_top(true)
    .min_inner_size(500.0, 500.0)
    .inner_size(1000.0, 700.0)
    .focused(true)
    .center()
    .build()
    .map_err(|error| format!("Failed to open Microsoft login window: {error}"))?;

    let started = std::time::Instant::now();
    let timeout = Duration::from_secs(10 * 60);
    let authorization_code = loop {
        if started.elapsed() >= timeout {
            let _ = window.close();
            return Err("Microsoft login timed out. Please try again.".to_string());
        }
        if window.title().is_err() {
            return Err("Microsoft login was cancelled.".to_string());
        }

        let current_url = window
            .url()
            .map_err(|error| format!("Failed to read Microsoft login URL: {error}"))?;
        if current_url.host_str() == Some(AUTH_REPLY_HOST) && current_url.path() == AUTH_REPLY_PATH
        {
            let parameters: HashMap<_, _> = current_url.query_pairs().into_owned().collect();
            if let Some(error) = parameters.get("error") {
                let description = parameters
                    .get("error_description")
                    .cloned()
                    .unwrap_or_else(|| error.clone());
                let _ = window.close();
                return Err(format!("Microsoft login failed: {description}"));
            }
            if parameters.get("state") != Some(&expected_state) {
                let _ = window.close();
                return Err("Microsoft login state validation failed.".to_string());
            }
            if let Some(code) = parameters.get("code") {
                break code.clone();
            }
        }

        sleep(Duration::from_millis(75)).await;
    };
    let _ = window.close();

    let oauth = exchange_authorization_code(&http, &authorization_code, &verifier).await?;
    let sisu = sisu_authorize(
        &http,
        Some(&session_id),
        &oauth.value.access_token,
        &device.value.token,
        &key,
        oauth.date,
    )
    .await?;
    let xsts = xsts_authorize(&http, sisu.value, &device.value.token, &key, sisu.date).await?;

    exchange_for_minecraft_account(&http, oauth.value, xsts.value).await
}

pub async fn login_with_cookies(cookies: &str) -> Result<MinecraftAccountData, String> {
    let cookies = cookies.trim();
    if cookies.is_empty() {
        return Err("Microsoft cookies cannot be empty.".to_string());
    }

    let jar = Arc::new(reqwest::cookie::Jar::default());
    populate_cookie_jar(&jar, cookies)?;
    let http = reqwest::Client::builder()
        .cookie_provider(jar)
        .redirect(reqwest::redirect::Policy::limited(12))
        .build()
        .map_err(|error| format!("Failed to create Microsoft cookie client: {error}"))?;

    let key = generate_device_key()?;
    let device = request_device_token(&http, &key, Utc::now()).await?;
    let verifier = random_url_safe(64);
    let challenge = BASE64_URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes()));
    let expected_state = random_url_safe(32);
    let (session_id, authorization_url) = sisu_authenticate(
        &http,
        &device.value.token,
        &challenge,
        &expected_state,
        "none",
        &key,
        device.date,
    )
    .await?;

    let response = http
        .get(authorization_url)
        .header("Accept", "text/html,application/xhtml+xml,application/json")
        .send()
        .await
        .map_err(|error| format!("Microsoft cookie sign-in failed: {error}"))?;
    let callback = response.url().clone();
    let parameters: HashMap<_, _> = callback.query_pairs().into_owned().collect();

    if let Some(error) = parameters.get("error") {
        let description = parameters
            .get("error_description")
            .cloned()
            .unwrap_or_else(|| error.clone());
        return Err(format!("Microsoft cookie sign-in failed: {description}"));
    }
    if callback.host_str() != Some(AUTH_REPLY_HOST) || callback.path() != AUTH_REPLY_PATH {
        return Err(
            "Microsoft cookies are invalid or expired. Export fresh login.live.com cookies and try again."
                .to_string(),
        );
    }
    if parameters.get("state") != Some(&expected_state) {
        return Err("Microsoft cookie sign-in state validation failed.".to_string());
    }
    let authorization_code = parameters.get("code").ok_or_else(|| {
        "Microsoft cookie sign-in did not return an authorization code.".to_string()
    })?;

    let oauth = exchange_authorization_code(&http, authorization_code, &verifier).await?;
    let sisu = sisu_authorize(
        &http,
        Some(&session_id),
        &oauth.value.access_token,
        &device.value.token,
        &key,
        oauth.date,
    )
    .await?;
    let xsts = xsts_authorize(&http, sisu.value, &device.value.token, &key, sisu.date).await?;
    exchange_for_minecraft_account(&http, oauth.value, xsts.value).await
}

fn populate_cookie_jar(jar: &reqwest::cookie::Jar, input: &str) -> Result<(), String> {
    let default_origins = [
        "https://login.live.com/",
        "https://account.live.com/",
        "https://login.microsoftonline.com/",
    ];
    let mut added = 0_usize;

    let parsed_json = serde_json::from_str::<serde_json::Value>(input).ok();
    let json_cookies = parsed_json.as_ref().and_then(|value| {
        value
            .as_array()
            .or_else(|| value.get("cookies").and_then(serde_json::Value::as_array))
    });
    if let Some(cookies) = json_cookies {
        for cookie in cookies {
            let Some(name) = cookie.get("name").and_then(serde_json::Value::as_str) else {
                continue;
            };
            let Some(value) = cookie.get("value").and_then(serde_json::Value::as_str) else {
                continue;
            };
            let domain = cookie
                .get("domain")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("login.live.com")
                .trim_start_matches('.');
            let path = cookie
                .get("path")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("/");
            let origin = url::Url::parse(&["https://", domain, path].concat())
                .map_err(|error| format!("Invalid cookie domain: {error}"))?;
            jar.add_cookie_str(
                &format!("{name}={value}; Domain={domain}; Path={path}; Secure"),
                &origin,
            );
            added += 1;
        }
    } else if input.lines().any(|line| line.split('\t').count() >= 7) {
        for line in input.lines().map(str::trim) {
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let columns: Vec<_> = line.split('\t').collect();
            if columns.len() < 7 {
                continue;
            }
            let domain = columns[0].trim_start_matches('.');
            let path = columns[2];
            let name = columns[5];
            let value = columns[6];
            let origin = url::Url::parse(&["https://", domain, path].concat())
                .map_err(|error| format!("Invalid cookie domain: {error}"))?;
            jar.add_cookie_str(
                &format!("{name}={value}; Domain={domain}; Path={path}; Secure"),
                &origin,
            );
            added += 1;
        }
    } else {
        let raw = input
            .strip_prefix("Cookie:")
            .or_else(|| input.strip_prefix("cookie:"))
            .unwrap_or(input)
            .trim();
        for pair in raw
            .split(';')
            .map(str::trim)
            .filter(|pair| pair.contains('='))
        {
            for origin in default_origins {
                let origin = url::Url::parse(origin).expect("static Microsoft cookie URL");
                jar.add_cookie_str(&format!("{pair}; Path=/; Secure"), &origin);
            }
            added += 1;
        }
    }

    if added == 0 {
        return Err(
            "No Microsoft cookies were found. Paste a Cookie header, Netscape cookies, or a JSON cookie export."
                .to_string(),
        );
    }
    Ok(())
}

pub async fn login_with_refresh_token(refresh_token: &str) -> Result<MinecraftAccountData, String> {
    let refresh_token = refresh_token.trim();
    if refresh_token.is_empty() {
        return Err("Microsoft refresh token cannot be empty.".to_string());
    }

    let http = reqwest::Client::new();
    let oauth = refresh_microsoft_token(&http, refresh_token).await?;
    complete_microsoft_exchange(&http, oauth).await
}

pub async fn login_with_access_token(access_token: &str) -> Result<MinecraftAccountData, String> {
    let access_token = access_token.trim();
    if access_token.is_empty() {
        return Err("Microsoft access token cannot be empty.".to_string());
    }

    let http = reqwest::Client::new();

    if verify_entitlement(&http, access_token).await.is_ok() {
        let profile = fetch_profile(&http, access_token).await?;
        return Ok(MinecraftAccountData {
            username: profile.name,
            uuid: profile.id,
            access_token: access_token.to_string(),
            refresh_token: None,
            expires_at: token_expiration(access_token),
        });
    }

    
    let oauth = OAuthTokenResponse {
        access_token: access_token.to_string(),
        refresh_token: String::new(),
        expires_in: 3600,
    };
    complete_microsoft_exchange(
        &http,
        DatedResponse {
            date: Utc::now(),
            value: oauth,
        },
    )
    .await
}

async fn complete_microsoft_exchange(
    http: &reqwest::Client,
    oauth: DatedResponse<OAuthTokenResponse>,
) -> Result<MinecraftAccountData, String> {
    let key = generate_device_key()?;
    let device = request_device_token(http, &key, oauth.date).await?;
    let sisu = sisu_authorize(
        http,
        None,
        &oauth.value.access_token,
        &device.value.token,
        &key,
        device.date,
    )
    .await?;
    let xsts = xsts_authorize(http, sisu.value, &device.value.token, &key, sisu.date).await?;
    exchange_for_minecraft_account(http, oauth.value, xsts.value).await
}

fn token_expiration(token: &str) -> String {
    let expiration = token
        .split('.')
        .nth(1)
        .and_then(|payload| BASE64_URL_SAFE_NO_PAD.decode(payload).ok())
        .and_then(|payload| serde_json::from_slice::<serde_json::Value>(&payload).ok())
        .and_then(|payload| payload.get("exp").and_then(serde_json::Value::as_i64))
        .and_then(|timestamp| DateTime::<Utc>::from_timestamp(timestamp, 0))
        .unwrap_or_else(|| Utc::now() + ChronoDuration::minutes(30));
    expiration.to_rfc3339()
}

pub async fn ensure_valid(account: &MinecraftAccountData) -> Result<MinecraftAccountData, String> {
    let expires_at = account
        .expires_at
        .parse::<DateTime<Utc>>()
        .unwrap_or_else(|_| Utc::now());
    if expires_at > Utc::now() + ChronoDuration::minutes(5) {
        return Ok(account.clone());
    }

    let refresh_token = account.refresh_token.as_deref().ok_or_else(|| {
        "This Microsoft session has expired and has no refresh token. Please sign in again."
            .to_string()
    })?;
    let http = reqwest::Client::new();
    let oauth = refresh_microsoft_token(&http, refresh_token).await?;
    let key = generate_device_key()?;
    let device = request_device_token(&http, &key, oauth.date).await?;
    let sisu = sisu_authorize(
        &http,
        None,
        &oauth.value.access_token,
        &device.value.token,
        &key,
        device.date,
    )
    .await?;
    let xsts = xsts_authorize(&http, sisu.value, &device.value.token, &key, sisu.date).await?;

    exchange_for_minecraft_account(&http, oauth.value, xsts.value).await
}

async fn request_device_token(
    http: &reqwest::Client,
    key: &DeviceKey,
    date: DateTime<Utc>,
) -> Result<DatedResponse<DeviceToken>, String> {
    let response = send_signed_request(
        http,
        "Xbox device authentication",
        "https://device.auth.xboxlive.com/device/authenticate",
        "/device/authenticate",
        serde_json::json!({
            "Properties": {
                "AuthMethod": "ProofOfPossession",
                "Id": format!("{{{}}}", key.id.to_string().to_uppercase()),
                "DeviceType": "Win32",
                "Version": "10.16.0",
                "ProofKey": proof_key_json(key)
            },
            "RelyingParty": "http://auth.xboxlive.com",
            "TokenType": "JWT"
        }),
        key,
        date,
    )
    .await?;

    Ok(DatedResponse {
        date: response.date,
        value: response.value,
    })
}

async fn sisu_authenticate(
    http: &reqwest::Client,
    device_token: &str,
    challenge: &str,
    state: &str,
    prompt: &str,
    key: &DeviceKey,
    date: DateTime<Utc>,
) -> Result<(String, url::Url), String> {
    let response: SignedResponse<SisuRedirect> = send_signed_request(
        http,
        "Xbox Sisu authentication",
        "https://sisu.xboxlive.com/authenticate",
        "/authenticate",
        serde_json::json!({
            "AppId": MICROSOFT_CLIENT_ID,
            "DeviceToken": device_token,
            "Offers": [REQUESTED_SCOPE],
            "Query": {
                "code_challenge": challenge,
                "code_challenge_method": "S256",
                "state": state,
                "prompt": prompt
            },
            "RedirectUri": auth_reply_url(),
            "Sandbox": "RETAIL",
            "TokenType": "code",
            "TitleId": "1794566092"
        }),
        key,
        date,
    )
    .await?;

    let session_id = response
        .headers
        .get("X-SessionId")
        .and_then(|value| value.to_str().ok())
        .ok_or_else(|| "Xbox Sisu authentication did not return a session ID.".to_string())?
        .to_string();
    let authorization_url = url::Url::parse(&response.value.msa_oauth_redirect)
        .map_err(|error| format!("Invalid Microsoft login URL: {error}"))?;

    Ok((session_id, authorization_url))
}

async fn exchange_authorization_code(
    http: &reqwest::Client,
    code: &str,
    verifier: &str,
) -> Result<DatedResponse<OAuthTokenResponse>, String> {
    request_oauth_token(
        http,
        &[
            ("client_id", MICROSOFT_CLIENT_ID),
            ("code", code),
            ("code_verifier", verifier),
            ("grant_type", "authorization_code"),
            ("redirect_uri", &auth_reply_url()),
            ("scope", REQUESTED_SCOPE),
        ],
    )
    .await
}

async fn refresh_microsoft_token(
    http: &reqwest::Client,
    refresh_token: &str,
) -> Result<DatedResponse<OAuthTokenResponse>, String> {
    request_oauth_token(
        http,
        &[
            ("client_id", MICROSOFT_CLIENT_ID),
            ("refresh_token", refresh_token),
            ("grant_type", "refresh_token"),
            ("redirect_uri", &auth_reply_url()),
            ("scope", REQUESTED_SCOPE),
        ],
    )
    .await
}

async fn request_oauth_token(
    http: &reqwest::Client,
    form: &[(&str, &str)],
) -> Result<DatedResponse<OAuthTokenResponse>, String> {
    let response = http
        .post(endpoint("login.live.com", "/oauth20_token.srf"))
        .header("Accept", "application/json")
        .form(form)
        .send()
        .await
        .map_err(|error| format!("Microsoft token request failed: {error}"))?;
    let date = response_date(response.headers());
    let status = response.status();
    let body = response
        .text()
        .await
        .map_err(|error| format!("Failed to read Microsoft token response: {error}"))?;
    if !status.is_success() {
        return Err(format!(
            "Microsoft token request failed (HTTP {status}): {}",
            limited_body(&body)
        ));
    }
    let value = serde_json::from_str(&body)
        .map_err(|error| format!("Invalid Microsoft token response: {error}"))?;
    Ok(DatedResponse { date, value })
}

async fn sisu_authorize(
    http: &reqwest::Client,
    session_id: Option<&str>,
    access_token: &str,
    device_token: &str,
    key: &DeviceKey,
    date: DateTime<Utc>,
) -> Result<DatedResponse<SisuAuthorizeResponse>, String> {
    let response = send_signed_request(
        http,
        "Xbox Sisu authorization",
        "https://sisu.xboxlive.com/authorize",
        "/authorize",
        serde_json::json!({
            "AccessToken": format!("t={access_token}"),
            "AppId": MICROSOFT_CLIENT_ID,
            "DeviceToken": device_token,
            "ProofKey": proof_key_json(key),
            "Sandbox": "RETAIL",
            "SessionId": session_id,
            "SiteName": "user.auth.xboxlive.com",
            "RelyingParty": "http://xboxlive.com",
            "UseModernGamertag": true
        }),
        key,
        date,
    )
    .await?;

    Ok(DatedResponse {
        date: response.date,
        value: response.value,
    })
}

async fn xsts_authorize(
    http: &reqwest::Client,
    sisu: SisuAuthorizeResponse,
    device_token: &str,
    key: &DeviceKey,
    date: DateTime<Utc>,
) -> Result<DatedResponse<DeviceToken>, String> {
    let response = send_signed_request(
        http,
        "Xbox security authorization",
        "https://xsts.auth.xboxlive.com/xsts/authorize",
        "/xsts/authorize",
        serde_json::json!({
            "RelyingParty": "rp://api.minecraftservices.com/",
            "TokenType": "JWT",
            "Properties": {
                "SandboxId": "RETAIL",
                "UserTokens": [sisu.user_token.token],
                "DeviceToken": device_token,
                "TitleToken": sisu.title_token.token
            }
        }),
        key,
        date,
    )
    .await?;

    Ok(DatedResponse {
        date: response.date,
        value: response.value,
    })
}

async fn exchange_for_minecraft_account(
    http: &reqwest::Client,
    oauth: OAuthTokenResponse,
    xsts: DeviceToken,
) -> Result<MinecraftAccountData, String> {
    let user_hash = xsts
        .display_claims
        .get("xui")
        .and_then(|claims| claims.get(0))
        .and_then(|claim| claim.get("uhs"))
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| "Xbox security authorization did not return a user hash.".to_string())?;

    let response = http
        .post(endpoint("api.minecraftservices.com", "/launcher/login"))
        .header("Accept", "application/json")
        .header("User-Agent", MINECRAFT_SERVICES_USER_AGENT)
        .json(&serde_json::json!({
            "platform": "PC_LAUNCHER",
            "xtoken": format!("XBL3.0 x={user_hash};{}", xsts.token)
        }))
        .send()
        .await
        .map_err(|error| format!("Minecraft authentication failed: {error}"))?;
    let status = response.status();
    let body = response
        .text()
        .await
        .map_err(|error| format!("Failed to read Minecraft authentication response: {error}"))?;
    if !status.is_success() {
        return Err(format!(
            "Minecraft authentication failed (HTTP {status}): {}",
            limited_body(&body)
        ));
    }
    let minecraft_token: MinecraftTokenResponse = serde_json::from_str(&body)
        .map_err(|error| format!("Invalid Minecraft authentication response: {error}"))?;

    verify_entitlement(http, &minecraft_token.access_token).await?;
    let profile = fetch_profile(http, &minecraft_token.access_token).await?;

    Ok(MinecraftAccountData {
        username: profile.name,
        uuid: profile.id,
        access_token: minecraft_token.access_token,
        refresh_token: (!oauth.refresh_token.is_empty()).then_some(oauth.refresh_token),
        expires_at: (Utc::now() + ChronoDuration::seconds(oauth.expires_in as i64)).to_rfc3339(),
    })
}

async fn send_signed_request<T: DeserializeOwned>(
    http: &reqwest::Client,
    step: &str,
    url: &str,
    url_path: &str,
    raw_body: serde_json::Value,
    key: &DeviceKey,
    date: DateTime<Utc>,
) -> Result<SignedResponse<T>, String> {
    let body = serde_json::to_vec(&raw_body)
        .map_err(|error| format!("Failed to serialize {step} request: {error}"))?;
    let windows_time = ((date.timestamp() as i128 + 11_644_473_600) * 10_000_000) as u64;

    let mut signature_input = Vec::new();
    signature_input.extend_from_slice(&1_u32.to_be_bytes());
    signature_input.push(0);
    signature_input.extend_from_slice(&windows_time.to_be_bytes());
    signature_input.push(0);
    signature_input.extend_from_slice(b"POST");
    signature_input.push(0);
    signature_input.extend_from_slice(url_path.as_bytes());
    signature_input.push(0);
    signature_input.push(0);
    signature_input.extend_from_slice(&body);
    signature_input.push(0);

    let signature: Signature = key.signing_key.sign(&signature_input);
    let mut signature_header = Vec::new();
    signature_header.extend_from_slice(&1_i32.to_be_bytes());
    signature_header.extend_from_slice(&windows_time.to_be_bytes());
    signature_header.extend_from_slice(&signature.r().to_bytes());
    signature_header.extend_from_slice(&signature.s().to_bytes());
    let signature_header = BASE64_STANDARD.encode(signature_header);

    let mut request = http
        .post(url)
        .header("Content-Type", "application/json; charset=utf-8")
        .header("Accept", "application/json")
        .header("Signature", signature_header);
    if url != "https://sisu.xboxlive.com/authorize" {
        request = request.header("x-xbl-contract-version", "1");
    }

    let response = request
        .body(body)
        .send()
        .await
        .map_err(|error| format!("{step} request failed: {error}"))?;
    let status = response.status();
    let headers = response.headers().clone();
    let response_date = response_date(&headers);
    let response_body = response
        .text()
        .await
        .map_err(|error| format!("Failed to read {step} response: {error}"))?;
    if !status.is_success() {
        return Err(format!(
            "{step} failed (HTTP {status}): {}",
            limited_body(&response_body)
        ));
    }
    let value = serde_json::from_str(&response_body)
        .map_err(|error| format!("Invalid {step} response: {error}"))?;

    Ok(SignedResponse {
        headers,
        date: response_date,
        value,
    })
}

fn response_date(headers: &reqwest::header::HeaderMap) -> DateTime<Utc> {
    headers
        .get(reqwest::header::DATE)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| DateTime::parse_from_rfc2822(value).ok())
        .map(|value| value.with_timezone(&Utc))
        .unwrap_or_else(Utc::now)
}

fn limited_body(body: &str) -> String {
    const MAX_CHARS: usize = 500;
    let mut chars = body.chars();
    let limited: String = chars.by_ref().take(MAX_CHARS).collect();
    if chars.next().is_some() {
        format!("{limited}…")
    } else {
        limited
    }
}

async fn verify_entitlement(http: &reqwest::Client, token: &str) -> Result<(), String> {
    let response = http
        .get(endpoint(
            "api.minecraftservices.com",
            "/entitlements/license",
        ))
        .header("User-Agent", MINECRAFT_SERVICES_USER_AGENT)
        .bearer_auth(token)
        .send()
        .await
        .map_err(|error| format!("Minecraft license check failed: {error}"))?;
    if response.status().is_success() {
        Ok(())
    } else {
        Err("This Microsoft account does not own Minecraft Java Edition.".to_string())
    }
}

async fn fetch_profile(http: &reqwest::Client, token: &str) -> Result<MinecraftProfile, String> {
    let response = http
        .get(endpoint("api.minecraftservices.com", "/minecraft/profile"))
        .header("User-Agent", MINECRAFT_SERVICES_USER_AGENT)
        .bearer_auth(token)
        .send()
        .await
        .map_err(|error| format!("Minecraft profile request failed: {error}"))?;
    if !response.status().is_success() {
        return Err("This account does not have a Minecraft Java Edition profile.".to_string());
    }
    response
        .json()
        .await
        .map_err(|error| format!("Invalid Minecraft profile response: {error}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use reqwest::cookie::CookieStore;

    fn cookie_header(jar: &reqwest::cookie::Jar) -> String {
        let origin = url::Url::parse("https://login.live.com/").unwrap();
        jar.cookies(&origin)
            .expect("cookie header")
            .to_str()
            .unwrap()
            .to_string()
    }

    #[test]
    fn cookie_parser_accepts_raw_header() {
        let jar = reqwest::cookie::Jar::default();
        populate_cookie_jar(&jar, "Cookie: MSPAuth=test; MSPOK=value").unwrap();
        let cookies = cookie_header(&jar);
        assert!(cookies.contains("MSPAuth=test"));
        assert!(cookies.contains("MSPOK=value"));
    }

    #[test]
    fn cookie_parser_accepts_json_export() {
        let jar = reqwest::cookie::Jar::default();
        populate_cookie_jar(
            &jar,
            r#"{"cookies":[{"name":"MSPAuth","value":"test","domain":".login.live.com","path":"/"}]}"#,
        )
        .unwrap();
        assert!(cookie_header(&jar).contains("MSPAuth=test"));
    }

    #[test]
    fn cookie_parser_accepts_netscape_export() {
        let jar = reqwest::cookie::Jar::default();
        populate_cookie_jar(&jar, ".login.live.com\tTRUE\t/\tTRUE\t0\tMSPAuth\ttest").unwrap();
        assert!(cookie_header(&jar).contains("MSPAuth=test"));
    }

    #[test]
    fn cookie_parser_rejects_empty_input() {
        let jar = reqwest::cookie::Jar::default();
        assert!(populate_cookie_jar(&jar, "").is_err());
    }

    #[test]
    fn token_expiration_uses_jwt_exp_claim() {
        let payload = BASE64_URL_SAFE_NO_PAD.encode(br#"{"exp":1893456000}"#);
        let token = format!("header.{payload}.signature");
        let expiration = token_expiration(&token).parse::<DateTime<Utc>>().unwrap();
        assert_eq!(expiration.timestamp(), 1_893_456_000);
    }
}
