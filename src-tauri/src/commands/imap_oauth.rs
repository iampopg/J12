use serde_json::{json, Value};
use crate::imap_acquisition::oauth::{
    start_google_device_flow, start_microsoft_device_flow,
    poll_google_device_token, poll_microsoft_device_token,
};

#[tauri::command]
pub async fn imap_device_flow_start(input: Value) -> Result<Value, String> {
    let provider = input["provider"].as_str().unwrap_or("google");
    let client_id = input["client_id"].as_str()
        .or_else(|| input["clientId"].as_str())
        .unwrap_or("")
        .to_string();

    let client_secret = input["client_secret"].as_str()
        .or_else(|| input["clientSecret"].as_str())
        .unwrap_or("")
        .to_string();

    let tenant_id = input["tenant_id"].as_str()
        .or_else(|| input["tenantId"].as_str());

    if provider == "google" {
        let cid = if client_id.is_empty() {
            // Built-in public desktop test credential
            "1072944624458-7cjhghj6l7j22m1b2g8r8f0t744v4a2u.apps.googleusercontent.com"
        } else {
            &client_id
        };
        let csec = if client_secret.is_empty() {
            "GOCSPX-uJk94l82m9z7"
        } else {
            &client_secret
        };

        let flow = start_google_device_flow(cid, csec).await?;
        Ok(json!(flow))
    } else {
        let cid = if client_id.is_empty() {
            // Standard Microsoft Graph / IMAP public client id
            "d3590ed6-52b3-4102-aeff-aad2292ab01c"
        } else {
            &client_id
        };

        let flow = start_microsoft_device_flow(cid, tenant_id).await?;
        Ok(json!(flow))
    }
}

#[tauri::command]
pub async fn imap_device_flow_poll(input: Value) -> Result<Value, String> {
    let provider = input["provider"].as_str().unwrap_or("google");
    let client_id = input["client_id"].as_str()
        .or_else(|| input["clientId"].as_str())
        .unwrap_or("")
        .to_string();

    let client_secret = input["client_secret"].as_str()
        .or_else(|| input["clientSecret"].as_str())
        .unwrap_or("")
        .to_string();

    let device_code = input["device_code"].as_str()
        .or_else(|| input["deviceCode"].as_str())
        .ok_or_else(|| "device_code is required".to_string())?;

    let tenant_id = input["tenant_id"].as_str()
        .or_else(|| input["tenantId"].as_str());

    if provider == "google" {
        let cid = if client_id.is_empty() {
            "1072944624458-7cjhghj6l7j22m1b2g8r8f0t744v4a2u.apps.googleusercontent.com"
        } else {
            &client_id
        };
        let csec = if client_secret.is_empty() {
            "GOCSPX-uJk94l82m9z7"
        } else {
            &client_secret
        };

        let token = poll_google_device_token(cid, csec, device_code).await?;
        Ok(json!(token))
    } else {
        let cid = if client_id.is_empty() {
            "d3590ed6-52b3-4102-aeff-aad2292ab01c"
        } else {
            &client_id
        };

        let token = poll_microsoft_device_token(cid, tenant_id, device_code).await?;
        Ok(json!(token))
    }
}
