use base64::Engine;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub fn generate_xoauth2_string(username: &str, access_token: &str) -> String {
    let raw = format!("user={}\x01auth=Bearer {}\x01\x01", username.trim(), access_token.trim());
    base64::engine::general_purpose::STANDARD.encode(raw.as_bytes())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeviceCodeResponse {
    pub device_code: String,
    pub user_code: String,
    pub verification_uri: String,
    pub expires_in: u64,
    pub interval: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OAuthTokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: Option<u64>,
    pub refresh_token: Option<String>,
    pub scope: Option<String>,
}

pub async fn start_google_device_flow(client_id: &str, client_secret: &str) -> Result<DeviceCodeResponse, String> {
    let client = reqwest::Client::new();
    let mut params = HashMap::new();
    params.insert("client_id", client_id);
    params.insert("scope", "https://mail.google.com/");

    let res = client
        .post("https://oauth2.googleapis.com/device/code")
        .form(&params)
        .send()
        .await
        .map_err(|e| format!("Google Device Code request failed: {}", e))?;

    if !res.status().is_success() {
        let err_text = res.text().await.unwrap_or_default();
        return Err(format!("Google OAuth Error: {}", err_text));
    }

    let parsed: serde_json::Value = res.json().await.map_err(|e| e.to_string())?;
    Ok(DeviceCodeResponse {
        device_code: parsed["device_code"].as_str().unwrap_or_default().to_string(),
        user_code: parsed["user_code"].as_str().unwrap_or_default().to_string(),
        verification_uri: parsed["verification_url"].as_str().unwrap_or("https://google.com/device").to_string(),
        expires_in: parsed["expires_in"].as_u64().unwrap_or(1800),
        interval: parsed["interval"].as_u64().unwrap_or(5),
    })
}

pub async fn start_microsoft_device_flow(client_id: &str, tenant_id: Option<&str>) -> Result<DeviceCodeResponse, String> {
    let client = reqwest::Client::new();
    let tenant = tenant_id.unwrap_or("common");
    let url = format!("https://login.microsoftonline.com/{}/oauth2/v2.0/devicecode", tenant);

    let mut params = HashMap::new();
    params.insert("client_id", client_id);
    params.insert("scope", "https://outlook.office.com/IMAP.AccessAsUser.All offline_access");

    let res = client
        .post(&url)
        .form(&params)
        .send()
        .await
        .map_err(|e| format!("Microsoft Device Code request failed: {}", e))?;

    if !res.status().is_success() {
        let err_text = res.text().await.unwrap_or_default();
        return Err(format!("Microsoft OAuth Error: {}", err_text));
    }

    let parsed: serde_json::Value = res.json().await.map_err(|e| e.to_string())?;
    Ok(DeviceCodeResponse {
        device_code: parsed["device_code"].as_str().unwrap_or_default().to_string(),
        user_code: parsed["user_code"].as_str().unwrap_or_default().to_string(),
        verification_uri: parsed["verification_uri"].as_str().unwrap_or("https://microsoft.com/devicelogin").to_string(),
        expires_in: parsed["expires_in"].as_u64().unwrap_or(900),
        interval: parsed["interval"].as_u64().unwrap_or(5),
    })
}

pub async fn poll_google_device_token(client_id: &str, client_secret: &str, device_code: &str) -> Result<OAuthTokenResponse, String> {
    let client = reqwest::Client::new();
    let mut params = HashMap::new();
    params.insert("client_id", client_id);
    params.insert("client_secret", client_secret);
    params.insert("device_code", device_code);
    params.insert("grant_type", "urn:ietf:params:oauth:grant-type:device_code");

    let res = client
        .post("https://oauth2.googleapis.com/token")
        .form(&params)
        .send()
        .await
        .map_err(|e| format!("Google token request failed: {}", e))?;

    let parsed: serde_json::Value = res.json().await.map_err(|e| e.to_string())?;
    if let Some(err) = parsed.get("error") {
        return Err(err.as_str().unwrap_or("authorization_pending").to_string());
    }

    Ok(OAuthTokenResponse {
        access_token: parsed["access_token"].as_str().unwrap_or_default().to_string(),
        token_type: parsed["token_type"].as_str().unwrap_or("Bearer").to_string(),
        expires_in: parsed["expires_in"].as_u64(),
        refresh_token: parsed["refresh_token"].as_str().map(|s| s.to_string()),
        scope: parsed["scope"].as_str().map(|s| s.to_string()),
    })
}

pub async fn poll_microsoft_device_token(client_id: &str, tenant_id: Option<&str>, device_code: &str) -> Result<OAuthTokenResponse, String> {
    let client = reqwest::Client::new();
    let tenant = tenant_id.unwrap_or("common");
    let url = format!("https://login.microsoftonline.com/{}/oauth2/v2.0/token", tenant);

    let mut params = HashMap::new();
    params.insert("client_id", client_id);
    params.insert("device_code", device_code);
    params.insert("grant_type", "urn:ietf:params:oauth:grant-type:device_code");

    let res = client
        .post(&url)
        .form(&params)
        .send()
        .await
        .map_err(|e| format!("Microsoft token request failed: {}", e))?;

    let parsed: serde_json::Value = res.json().await.map_err(|e| e.to_string())?;
    if let Some(err) = parsed.get("error") {
        return Err(err.as_str().unwrap_or("authorization_pending").to_string());
    }

    Ok(OAuthTokenResponse {
        access_token: parsed["access_token"].as_str().unwrap_or_default().to_string(),
        token_type: parsed["token_type"].as_str().unwrap_or("Bearer").to_string(),
        expires_in: parsed["expires_in"].as_u64(),
        refresh_token: parsed["refresh_token"].as_str().map(|s| s.to_string()),
        scope: parsed["scope"].as_str().map(|s| s.to_string()),
    })
}
