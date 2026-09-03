use tauri::State;
use crate::AppState;
use super::types::KiloAIModel;

/// Fetch free models from kilo.ai
#[tauri::command]
pub async fn fetch_kiloai_models() -> Result<Vec<KiloAIModel>, String> {
    let client = reqwest::Client::new();
    
    let response = client
        .get("https://kilo.ai/api/models")
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;
    
    if !response.status().is_success() {
        return Err(format!("API error: {}", response.status()));
    }
    
    let models: Vec<serde_json::Value> = response
        .json()
        .await
        .map_err(|e| format!("Parse error: {}", e))?;
    
    let mut free_models = Vec::new();
    
    for model in &models {
        let price_input_str = model
            .get("priceInput")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        
        let price_output_str = model
            .get("priceOutput")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        
        let input_price: f64 = if price_input_str.is_empty() {
            0.0
        } else {
            price_input_str.parse().unwrap_or(1.0)
        };
        
        let output_price: f64 = if price_output_str.is_empty() {
            0.0
        } else {
            price_output_str.parse().unwrap_or(1.0)
        };
        
        if input_price == 0.0 && output_price == 0.0 {
            let model_id = model
                .get("openrouterId")
                .or_else(|| model.get("slug"))
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            
            let name = model
                .get("name")
                .or_else(|| model.get("openrouterData").and_then(|v| v.get("name")))
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown")
                .to_string();
            
            let description = model
                .get("description")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            
            let context_length = model
                .get("contextLength")
                .or_else(|| model.get("openrouterData").and_then(|v| v.get("contextLength")))
                .and_then(|v| v.as_i64());
            
            let is_recommended = model
                .get("isRecommended")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            
            if !model_id.is_empty() && model_id != "null" {
                free_models.push(KiloAIModel {
                    id: model_id,
                    name,
                    description: description.chars().take(100).collect(),
                    context_length,
                    is_recommended,
                });
            }
        }
    }
    
    Ok(free_models)
}

/// Fetch models from OpenRouter
#[tauri::command]
pub async fn fetch_openrouter_models() -> Result<Vec<KiloAIModel>, String> {
    let client = reqwest::Client::new();
    
    let response = client
        .get("https://openrouter.ai/api/v1/models")
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;
    
    if !response.status().is_success() {
        return Err(format!("API error: {}", response.status()));
    }
    
    let json: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("Parse error: {}", e))?;
    
    let models = json
        .get("data")
        .and_then(|v| v.as_array())
        .map(|arr| arr.as_slice())
        .unwrap_or(&[]);
    
    let mut free_models = Vec::new();
    
    for model in models {
        let pricing = model.get("pricing");
        let prompt_price = pricing
            .and_then(|p| p.get("prompt"))
            .and_then(|v| v.as_str())
            .unwrap_or("0");
        let completion_price = pricing
            .and_then(|p| p.get("completion"))
            .and_then(|v| v.as_str())
            .unwrap_or("0");
        
        let prompt_f: f64 = prompt_price.parse().unwrap_or(0.0);
        let completion_f: f64 = completion_price.parse().unwrap_or(0.0);
        
        if prompt_f == 0.0 && completion_f == 0.0 {
            let model_id = model.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let name = model.get("name").and_then(|v| v.as_str()).unwrap_or("Unknown").to_string();
            let description = model.get("description").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let context_length = model.get("context_length").and_then(|v| v.as_i64());
            
            if !model_id.is_empty() {
                free_models.push(KiloAIModel {
                    id: model_id,
                    name,
                    description: description.chars().take(100).collect(),
                    context_length,
                    is_recommended: false,
                });
            }
        }
    }
    
    Ok(free_models)
}

/// AI Chat command
#[tauri::command]
pub async fn ai_chat(input: serde_json::Value) -> Result<String, String> {
    let provider = input["provider"].as_str().unwrap_or("local");
    let api_key = input["api_key"].as_str().unwrap_or("");
    let model = input["model"].as_str().unwrap_or("llama3.2");
    let endpoint = input["endpoint"].as_str().unwrap_or("http://localhost:11434");
    let raw_prompt = input["prompt"].as_str().unwrap_or("");
    let (prompt_sanitized, has_injection, redacted_count) = super::guard::prepare_ai_prompt(raw_prompt, provider);
    if has_injection {
        eprintln!("[AI SECURITY WARNING] Heuristic prompt injection detected in prompt for provider: {}", provider);
    }
    if redacted_count > 0 {
        eprintln!("[AI PRIVACY GUARD] Redacted {} PII items before external transmission to: {}", redacted_count, provider);
    }
    let prompt = prompt_sanitized.as_str();

    let client = reqwest::Client::new();
    let system_prompt = "You are a forensic investigation assistant helping investigators analyze email evidence. Always cite evidence references when making claims. Be concise and factual.";
    
    match provider {
        "local" => {
            let response = client
                .post(format!("{}/api/chat", endpoint))
                .json(&serde_json::json!({
                    "model": model,
                    "messages": [
                        {"role": "system", "content": system_prompt},
                        {"role": "user", "content": prompt}
                    ],
                    "stream": false
                }))
                .timeout(std::time::Duration::from_secs(60))
                .send()
                .await
                .map_err(|e| format!("Local AI error: {}", e))?;
            
            if !response.status().is_success() {
                return Err(format!("Local AI error: {}", response.status()));
            }
            
            let data: serde_json::Value = response.json().await.map_err(|e| e.to_string())?;
            Ok(data["message"]["content"].as_str().unwrap_or("No response").to_string())
        }
        "openrouter" => {
            let response = client
                .post("https://openrouter.ai/api/v1/chat/completions")
                .header("Authorization", format!("Bearer {}", api_key))
                .header("HTTP-Referer", "https://j12-forensic.app")
                .header("X-Title", "J12 Forensic")
                .json(&serde_json::json!({
                    "model": model,
                    "messages": [
                        {"role": "system", "content": system_prompt},
                        {"role": "user", "content": prompt}
                    ]
                }))
                .timeout(std::time::Duration::from_secs(60))
                .send()
                .await
                .map_err(|e| format!("OpenRouter error: {}", e))?;
            
            if !response.status().is_success() {
                let status = response.status();
                let error_text = response.text().await.unwrap_or_default();
                return Err(format!("OpenRouter error {}: {}", status, error_text));
            }
            
            let data: serde_json::Value = response.json().await.map_err(|e| e.to_string())?;
            Ok(data["choices"][0]["message"]["content"].as_str().unwrap_or("No response").to_string())
        }
        "kiloai" => {
            let response = client
                .post("https://api.kilo.ai/api/gateway")
                .header("Authorization", format!("Bearer {}", api_key))
                .header("HTTP-Referer", "https://j12-forensic.app")
                .header("X-Title", "J12 Forensic")
                .json(&serde_json::json!({
                    "model": model,
                    "messages": [
                        {"role": "system", "content": system_prompt},
                        {"role": "user", "content": prompt}
                    ]
                }))
                .timeout(std::time::Duration::from_secs(60))
                .send()
                .await
                .map_err(|e| format!("kilo.ai error: {}", e))?;
            
            if !response.status().is_success() {
                let status = response.status();
                let error_text = response.text().await.unwrap_or_default();
                return Err(format!("kilo.ai error {}: {}", status, error_text));
            }
            
            let data: serde_json::Value = response.json().await.map_err(|e| e.to_string())?;
            Ok(data["choices"][0]["message"]["content"].as_str().unwrap_or("No response").to_string())
        }
        "gemini" => {
            let response = client
                .post(format!("https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}", model, api_key))
                .json(&serde_json::json!({
                    "contents": [{"parts": [{"text": format!("{}\n\n{}", system_prompt, prompt)}]}]
                }))
                .timeout(std::time::Duration::from_secs(60))
                .send()
                .await
                .map_err(|e| format!("Gemini error: {}", e))?;
            
            if !response.status().is_success() {
                return Err(format!("Gemini error: {}", response.status()));
            }
            
            let data: serde_json::Value = response.json().await.map_err(|e| e.to_string())?;
            Ok(data["candidates"][0]["content"]["parts"][0]["text"].as_str().unwrap_or("No response").to_string())
        }
        "chatgpt" => {
            let response = client
                .post("https://api.openai.com/v1/chat/completions")
                .header("Authorization", format!("Bearer {}", api_key))
                .json(&serde_json::json!({
                    "model": model,
                    "messages": [
                        {"role": "system", "content": system_prompt},
                        {"role": "user", "content": prompt}
                    ]
                }))
                .timeout(std::time::Duration::from_secs(60))
                .send()
                .await
                .map_err(|e| format!("ChatGPT error: {}", e))?;
            
            if !response.status().is_success() {
                return Err(format!("ChatGPT error: {}", response.status()));
            }
            
            let data: serde_json::Value = response.json().await.map_err(|e| e.to_string())?;
            Ok(data["choices"][0]["message"]["content"].as_str().unwrap_or("No response").to_string())
        }
        "claude" => {
            let response = client
                .post("https://api.anthropic.com/v1/messages")
                .header("x-api-key", api_key)
                .header("anthropic-version", "2023-06-01")
                .json(&serde_json::json!({
                    "model": model,
                    "max_tokens": 1024,
                    "system": system_prompt,
                    "messages": [{"role": "user", "content": prompt}]
                }))
                .timeout(std::time::Duration::from_secs(60))
                .send()
                .await
                .map_err(|e| format!("Claude error: {}", e))?;
            
            if !response.status().is_success() {
                return Err(format!("Claude error: {}", response.status()));
            }
            
            let data: serde_json::Value = response.json().await.map_err(|e| e.to_string())?;
            Ok(data["content"][0]["text"].as_str().unwrap_or("No response").to_string())
        }
        _ => Err(format!("Unknown provider: {}", provider))
    }
}

/// Create a new AI session
#[tauri::command]
pub async fn ai_create_session(state: State<'_, AppState>, case_id: String, provider: String, model: String) -> Result<String, String> {
    let db = state.db.lock().await;
    let session_id = format!("ses_{}", crate::db::generate_id());
    let now = chrono::Utc::now().to_rfc3339();
    
    db.conn.execute(
        "INSERT INTO ai_sessions (id, case_id, provider, model, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params![&session_id, &case_id, &provider, &model, &now],
    ).map_err(|e| e.to_string())?;
    
    db.conn.execute(
        "INSERT INTO ai_audit_log (id, case_id, action, provider, timestamp) VALUES (?1, ?2, 'session_created', ?3, ?4)",
        rusqlite::params![format!("aud_{}", crate::db::generate_id()), &case_id, &provider, &now],
    ).ok();
    
    Ok(session_id)
}

/// Get AI session history
#[tauri::command]
pub async fn ai_get_session_history(state: State<'_, AppState>, session_id: String) -> Result<Vec<serde_json::Value>, String> {
    let db = state.db.lock().await;
    
    let mut stmt = db.conn.prepare(
        "SELECT id, role, content, evidence_refs, timestamp FROM ai_messages WHERE session_id = ?1 ORDER BY timestamp ASC"
    ).map_err(|e| e.to_string())?;
    
    let messages = stmt.query_map([&session_id], |row| {
        Ok(serde_json::json!({
            "id": row.get::<_, String>(0)?,
            "role": row.get::<_, String>(1)?,
            "content": row.get::<_, String>(2)?,
            "evidence_refs": row.get::<_, Option<String>>(3)?.unwrap_or_default(),
            "timestamp": row.get::<_, String>(4)?,
        }))
    }).map_err(|e| e.to_string())?.collect::<Result<Vec<_>,_>>().map_err(|e| e.to_string())?;
    
    Ok(messages)
}

/// Clear AI session
#[tauri::command]
pub async fn ai_clear_session(state: State<'_, AppState>, session_id: String) -> Result<(), String> {
    let db = state.db.lock().await;
    
    db.conn.execute("DELETE FROM ai_messages WHERE session_id = ?1", [&session_id]).ok();
    db.conn.execute("DELETE FROM ai_tool_calls WHERE session_id = ?1", [&session_id]).ok();
    db.conn.execute("DELETE FROM ai_context_snapshots WHERE session_id = ?1", [&session_id]).ok();
    
    let now = chrono::Utc::now().to_rfc3339();
    db.conn.execute("UPDATE ai_sessions SET ended_at = ?1 WHERE id = ?2", [&now, &session_id]).ok();
    
    Ok(())
}
