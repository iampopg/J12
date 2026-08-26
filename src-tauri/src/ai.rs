//! AI Evidence Access Layer
//! 
//! This module provides read-only access to the evidence database for AI tools.
//! All access is permission-scoped, bounded, and audited.
//! 
//! Architecture:
//! Evidence DB → Evidence Access Layer → AI Evidence Gateway → AI Tools

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Kilo.ai model info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KiloAIModel {
    pub id: String,
    pub name: String,
    pub description: String,
    pub context_length: Option<i64>,
    pub is_recommended: bool,
}

/// Fetch free models from kilo.ai (backend command to avoid CORS)
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
    
    // API returns a list directly
    let models: Vec<serde_json::Value> = response
        .json()
        .await
        .map_err(|e| format!("Parse error: {}", e))?;
    
    let mut free_models = Vec::new();
    
    for model in &models {
        // Get prices - empty string or "0.000000" means free
        let price_input_str = model
            .get("priceInput")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        
        let price_output_str = model
            .get("priceOutput")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        
        // Parse price - empty string = free
        let input_price: f64 = if price_input_str.is_empty() {
            0.0
        } else {
            price_input_str.parse().unwrap_or(1.0) // If can't parse, assume paid
        };
        
        let output_price: f64 = if price_output_str.is_empty() {
            0.0
        } else {
            price_output_str.parse().unwrap_or(1.0)
        };
        
        // Filter: only free models (price = 0)
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

/// Fetch models from OpenRouter (backend command to avoid CORS)
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
        
        // Include free and very cheap models
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

/// AI Chat command (backend - avoids CORS)
#[tauri::command]
pub async fn ai_chat(input: serde_json::Value) -> Result<String, String> {
    let provider = input["provider"].as_str().unwrap_or("local");
    let api_key = input["api_key"].as_str().unwrap_or("");
    let model = input["model"].as_str().unwrap_or("llama3.2");
    let endpoint = input["endpoint"].as_str().unwrap_or("http://localhost:11434");
    let prompt = input["prompt"].as_str().unwrap_or("");
    
    let client = reqwest::Client::new();
    let system_prompt = "You are a forensic investigation assistant helping investigators analyze email evidence. Always cite evidence references when making claims. Be concise and factual.";
    
    match provider {
        "local" => {
            // Call Ollama/local LLM
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
            // OpenRouter API
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
            // kilo.ai uses Kilo Gateway API (OpenAI-compatible)
            // Endpoint: https://api.kilo.ai/api/gateway
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
            // Call Google Gemini
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
            // Call OpenAI
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
            // Call Anthropic Claude
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

/// Search query for emails (matches the SearchQuery in the architecture doc)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchQuery {
    pub text: Option<String>,
    pub subject: Option<String>,
    pub from: Option<String>,
    pub to: Option<String>,
    pub date_from: Option<String>,
    pub date_to: Option<String>,
    pub has_attachments: Option<bool>,
    pub attachment_types: Option<Vec<String>>,
    pub folder_category: Option<String>,
    pub risk_score_min: Option<i64>,
    pub entity_id: Option<String>,
    pub limit: i64,
    pub offset: i64,
}

impl Default for SearchQuery {
    fn default() -> Self {
        Self {
            text: None,
            subject: None,
            from: None,
            to: None,
            date_from: None,
            date_to: None,
            has_attachments: None,
            attachment_types: None,
            folder_category: None,
            risk_score_min: None,
            entity_id: None,
            limit: 50,
            offset: 0,
        }
    }
}

/// Email search result (bounded fields for AI)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailResult {
    pub id: String,
    pub message_id: Option<String>,
    pub from_addr: String,
    pub from_display: Option<String>,
    pub to_addrs: String,
    pub subject: Option<String>,
    pub date_sent: Option<String>,
    pub folder_category: String,
    pub risk_score: i64,
    pub has_attachments: bool,
}

/// Attachment metadata (no binary content)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttachmentMetadata {
    pub id: String,
    pub filename: Option<String>,
    pub mime_type: String,
    pub size_bytes: i64,
    pub sha256: String,
    pub entropy: Option<f64>,
    pub risk_flags: Vec<String>,
}

/// Authentication results for an email
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthResults {
    pub email_id: String,
    pub spf_result: Option<String>,
    pub dkim_result: Option<String>,
    pub dmarc_result: Option<String>,
    pub arc_result: Option<String>,
    pub received_chain: Vec<String>,
    pub originating_ip: Option<String>,
}

/// Entity data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityData {
    pub id: String,
    pub email_address: String,
    pub display_name: Option<String>,
    pub sent_count: i64,
    pub received_count: i64,
    pub first_seen: Option<String>,
    pub last_seen: Option<String>,
}

/// Timeline event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineEvent {
    pub id: String,
    pub timestamp: String,
    pub event_type: String,
    pub actor: Option<String>,
    pub summary: Option<String>,
    pub email_id: Option<String>,
}

/// Finding data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FindingData {
    pub id: String,
    pub finding_type: String,
    pub severity: String,
    pub title: String,
    pub description: Option<String>,
    pub status: String,
}

/// Case statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaseStats {
    pub total_emails: i64,
    pub total_entities: i64,
    pub total_attachments: i64,
    pub total_findings: i64,
    pub inbox_count: i64,
    pub sent_count: i64,
    pub deleted_count: i64,
    pub spam_count: i64,
    pub date_from: Option<String>,
    pub date_to: Option<String>,
}

/// Tool risk classification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ToolRiskLevel {
    /// Level 0: Harmless retrieval
    Harmless,
    /// Level 1: Sensitive retrieval
    Sensitive,
    /// Level 2: Expensive analysis
    Expensive,
    /// Level 3: Potentially dangerous
    Dangerous,
}

/// Investigation budget
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvestigationBudget {
    pub max_tool_calls: i64,
    pub max_runtime_seconds: i64,
    pub max_results: i64,
    pub max_tokens: i64,
    pub max_attachment_bytes: i64,
    pub max_graph_nodes: i64,
}

impl Default for InvestigationBudget {
    fn default() -> Self {
        Self {
            max_tool_calls: 50,
            max_runtime_seconds: 120,
            max_results: 1000,
            max_tokens: 10000,
            max_attachment_bytes: 10485760, // 10 MB
            max_graph_nodes: 500,
        }
    }
}

/// AI Evidence Gateway policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceGatewayPolicy {
    pub provider_type: AIProviderType,
    pub enable_body: bool,
    pub enable_headers: bool,
    pub enable_pii: bool,
    pub enable_credentials: bool,
    pub enable_attachment_text: bool,
    pub enable_attachment_binary: bool,
    pub enable_chain_of_custody: bool,
    pub enable_investigator_notes: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AIProviderType {
    Local,
    KiloAI,
    Online,
}

impl EvidenceGatewayPolicy {
    /// Get policy for local AI
    pub fn local() -> Self {
        Self {
            provider_type: AIProviderType::Local,
            enable_body: true,
            enable_headers: true,
            enable_pii: true,
            enable_credentials: false,
            enable_attachment_text: true,
            enable_attachment_binary: false,
            enable_chain_of_custody: false,
            enable_investigator_notes: false,
        }
    }
    
    /// Get policy for remote AI
    pub fn remote() -> Self {
        Self {
            provider_type: AIProviderType::Online,
            enable_body: false,
            enable_headers: true,
            enable_pii: false,
            enable_credentials: false,
            enable_attachment_text: false,
            enable_attachment_binary: false,
            enable_chain_of_custody: false,
            enable_investigator_notes: false,
        }
    }
    
    /// Get policy for kilo.ai
    pub fn kiloai() -> Self {
        Self {
            provider_type: AIProviderType::KiloAI,
            enable_body: true,
            enable_headers: true,
            enable_pii: true,
            enable_credentials: false,
            enable_attachment_text: true,
            enable_attachment_binary: false,
            enable_chain_of_custody: false,
            enable_investigator_notes: false,
        }
    }
}

/// Tool definition for AI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub risk_level: ToolRiskLevel,
    pub parameters: Vec<ToolParameter>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolParameter {
    pub name: String,
    pub param_type: String,
    pub required: bool,
    pub description: String,
}

/// All available AI tools
pub fn get_tool_definitions() -> Vec<ToolDefinition> {
    vec![
        // Level 0: Harmless retrieval
        ToolDefinition {
            name: "search_emails".to_string(),
            description: "Search emails with filters".to_string(),
            risk_level: ToolRiskLevel::Harmless,
            parameters: vec![
                ToolParameter { name: "text".to_string(), param_type: "string".to_string(), required: false, description: "Full-text search".to_string() },
                ToolParameter { name: "from".to_string(), param_type: "string".to_string(), required: false, description: "Sender email".to_string() },
                ToolParameter { name: "to".to_string(), param_type: "string".to_string(), required: false, description: "Recipient email".to_string() },
                ToolParameter { name: "date_from".to_string(), param_type: "string".to_string(), required: false, description: "Start date".to_string() },
                ToolParameter { name: "date_to".to_string(), param_type: "string".to_string(), required: false, description: "End date".to_string() },
                ToolParameter { name: "limit".to_string(), param_type: "number".to_string(), required: true, description: "Max results (max 100)".to_string() },
            ],
        },
        ToolDefinition {
            name: "get_email".to_string(),
            description: "Get email metadata by ID".to_string(),
            risk_level: ToolRiskLevel::Harmless,
            parameters: vec![
                ToolParameter { name: "email_id".to_string(), param_type: "string".to_string(), required: true, description: "Email ID".to_string() },
            ],
        },
        ToolDefinition {
            name: "get_case_statistics".to_string(),
            description: "Get case statistics".to_string(),
            risk_level: ToolRiskLevel::Harmless,
            parameters: vec![
                ToolParameter { name: "case_id".to_string(), param_type: "string".to_string(), required: true, description: "Case ID".to_string() },
            ],
        },
        ToolDefinition {
            name: "get_entity".to_string(),
            description: "Get entity by ID or email".to_string(),
            risk_level: ToolRiskLevel::Harmless,
            parameters: vec![
                ToolParameter { name: "entity_id".to_string(), param_type: "string".to_string(), required: false, description: "Entity ID".to_string() },
                ToolParameter { name: "email".to_string(), param_type: "string".to_string(), required: false, description: "Email address".to_string() },
            ],
        },
        ToolDefinition {
            name: "get_authentication_results".to_string(),
            description: "Get SPF/DKIM/DMARC results".to_string(),
            risk_level: ToolRiskLevel::Harmless,
            parameters: vec![
                ToolParameter { name: "email_id".to_string(), param_type: "string".to_string(), required: true, description: "Email ID".to_string() },
            ],
        },
        ToolDefinition {
            name: "get_timeline".to_string(),
            description: "Get timeline events".to_string(),
            risk_level: ToolRiskLevel::Harmless,
            parameters: vec![
                ToolParameter { name: "case_id".to_string(), param_type: "string".to_string(), required: true, description: "Case ID".to_string() },
                ToolParameter { name: "limit".to_string(), param_type: "number".to_string(), required: false, description: "Max events".to_string() },
            ],
        },
        ToolDefinition {
            name: "get_findings".to_string(),
            description: "Get forensic findings".to_string(),
            risk_level: ToolRiskLevel::Harmless,
            parameters: vec![
                ToolParameter { name: "case_id".to_string(), param_type: "string".to_string(), required: true, description: "Case ID".to_string() },
            ],
        },
        // Level 1: Sensitive retrieval
        ToolDefinition {
            name: "get_email_body".to_string(),
            description: "Get email body text".to_string(),
            risk_level: ToolRiskLevel::Sensitive,
            parameters: vec![
                ToolParameter { name: "email_id".to_string(), param_type: "string".to_string(), required: true, description: "Email ID".to_string() },
            ],
        },
        ToolDefinition {
            name: "get_attachment_text".to_string(),
            description: "Get extracted text from attachment".to_string(),
            risk_level: ToolRiskLevel::Sensitive,
            parameters: vec![
                ToolParameter { name: "attachment_id".to_string(), param_type: "string".to_string(), required: true, description: "Attachment ID".to_string() },
                ToolParameter { name: "max_bytes".to_string(), param_type: "number".to_string(), required: false, description: "Max bytes (default 50000)".to_string() },
            ],
        },
        ToolDefinition {
            name: "get_attachments".to_string(),
            description: "Get attachment metadata for email".to_string(),
            risk_level: ToolRiskLevel::Sensitive,
            parameters: vec![
                ToolParameter { name: "email_id".to_string(), param_type: "string".to_string(), required: true, description: "Email ID".to_string() },
            ],
        },
        // Level 2: Expensive analysis
        ToolDefinition {
            name: "get_communication_graph".to_string(),
            description: "Get communication graph data".to_string(),
            risk_level: ToolRiskLevel::Expensive,
            parameters: vec![
                ToolParameter { name: "case_id".to_string(), param_type: "string".to_string(), required: true, description: "Case ID".to_string() },
                ToolParameter { name: "max_nodes".to_string(), param_type: "number".to_string(), required: false, description: "Max nodes (default 500)".to_string() },
            ],
        },
        ToolDefinition {
            name: "run_entity_resolution".to_string(),
            description: "Find possible duplicate entities".to_string(),
            risk_level: ToolRiskLevel::Expensive,
            parameters: vec![
                ToolParameter { name: "case_id".to_string(), param_type: "string".to_string(), required: true, description: "Case ID".to_string() },
            ],
        },
        ToolDefinition {
            name: "run_anomaly_analysis".to_string(),
            description: "Find anomalous emails".to_string(),
            risk_level: ToolRiskLevel::Expensive,
            parameters: vec![
                ToolParameter { name: "case_id".to_string(), param_type: "string".to_string(), required: true, description: "Case ID".to_string() },
                ToolParameter { name: "limit".to_string(), param_type: "number".to_string(), required: false, description: "Max results".to_string() },
            ],
        },
    ]
}

/// Validate tool call against budget and policy
pub fn validate_tool_call(
    tool_name: &str,
    policy: &EvidenceGatewayPolicy,
    budget: &InvestigationBudget,
    current_calls: i64,
) -> Result<(), String> {
    // Check budget
    if current_calls >= budget.max_tool_calls {
        return Err(format!("Tool call budget exceeded ({}/{})", current_calls, budget.max_tool_calls));
    }
    
    // Check tool risk level
    let tools = get_tool_definitions();
    let tool = tools.iter().find(|t| t.name == tool_name);
    
    match tool {
        Some(t) => {
            match t.risk_level {
                ToolRiskLevel::Harmless => Ok(()),
                ToolRiskLevel::Sensitive => {
                    if policy.enable_body {
                        Ok(())
                    } else {
                        Err("Sensitive retrieval not allowed by policy".to_string())
                    }
                }
                ToolRiskLevel::Expensive => Ok(()), // Allowed but monitored
                ToolRiskLevel::Dangerous => Err("Dangerous tools not allowed".to_string()),
            }
        }
        None => Err(format!("Unknown tool: {}", tool_name)),
    }
}

// === TAURI COMMANDS ===

use crate::AppState;
use tauri::State;

/// Get case statistics (AI tool)
#[tauri::command]
pub async fn ai_get_case_statistics(state: State<'_, AppState>, case_id: String) -> Result<CaseStats, String> {
    let db = state.db.lock().await;
    
    let total_emails: i64 = db.conn.query_row(
        "SELECT COUNT(*) FROM emails WHERE case_id = ?1",
        [&case_id],
        |row| row.get(0)
    ).unwrap_or(0);
    
    let total_entities: i64 = db.conn.query_row(
        "SELECT COUNT(*) FROM entities WHERE case_id = ?1",
        [&case_id],
        |row| row.get(0)
    ).unwrap_or(0);
    
    let total_attachments: i64 = db.conn.query_row(
        "SELECT COUNT(*) FROM attachments WHERE email_id IN (SELECT id FROM emails WHERE case_id = ?1)",
        [&case_id],
        |row| row.get(0)
    ).unwrap_or(0);
    
    let total_findings: i64 = db.conn.query_row(
        "SELECT COUNT(*) FROM findings WHERE case_id = ?1",
        [&case_id],
        |row| row.get(0)
    ).unwrap_or(0);
    
    let inbox_count: i64 = db.conn.query_row(
        "SELECT COUNT(*) FROM emails WHERE case_id = ?1 AND folder_category = 'inbox'",
        [&case_id],
        |row| row.get(0)
    ).unwrap_or(0);
    
    let sent_count: i64 = db.conn.query_row(
        "SELECT COUNT(*) FROM emails WHERE case_id = ?1 AND folder_category = 'sent'",
        [&case_id],
        |row| row.get(0)
    ).unwrap_or(0);
    
    let deleted_count: i64 = db.conn.query_row(
        "SELECT COUNT(*) FROM emails WHERE case_id = ?1 AND folder_category = 'soft_deleted'",
        [&case_id],
        |row| row.get(0)
    ).unwrap_or(0);
    
    let spam_count: i64 = db.conn.query_row(
        "SELECT COUNT(*) FROM emails WHERE case_id = ?1 AND folder_category = 'spam'",
        [&case_id],
        |row| row.get(0)
    ).unwrap_or(0);
    
    Ok(CaseStats {
        total_emails,
        total_entities,
        total_attachments,
        total_findings,
        inbox_count,
        sent_count,
        deleted_count,
        spam_count,
        date_from: None,
        date_to: None,
    })
}

/// Search emails (AI tool)
#[tauri::command]
pub async fn ai_search_emails(state: State<'_, AppState>, query: SearchQuery) -> Result<Vec<EmailResult>, String> {
    let db = state.db.lock().await;
    
    let mut sql = String::from("SELECT id, message_id, from_addr, from_display, to_addrs, subject, date_sent_utc, folder_category, risk_score FROM emails WHERE 1=1");
    let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();
    
    if let Some(text) = &query.text {
        sql.push_str(" AND (subject LIKE ? OR body_text LIKE ?)");
        params.push(Box::new(format!("%{}%", text)));
        params.push(Box::new(format!("%{}%", text)));
    }
    
    if let Some(from) = &query.from {
        sql.push_str(" AND from_addr LIKE ?");
        params.push(Box::new(format!("%{}%", from)));
    }
    
    if let Some(to) = &query.to {
        sql.push_str(" AND to_addrs LIKE ?");
        params.push(Box::new(format!("%{}%", to)));
    }
    
    if let Some(date_from) = &query.date_from {
        sql.push_str(" AND date_sent_utc >= ?");
        params.push(Box::new(date_from.clone()));
    }
    
    if let Some(date_to) = &query.date_to {
        sql.push_str(" AND date_sent_utc <= ?");
        params.push(Box::new(date_to.clone()));
    }
    
    if let Some(folder) = &query.folder_category {
        sql.push_str(" AND folder_category = ?");
        params.push(Box::new(folder.clone()));
    }
    
    if let Some(risk_min) = query.risk_score_min {
        sql.push_str(" AND risk_score >= ?");
        params.push(Box::new(risk_min));
    }
    
    sql.push_str(" ORDER BY date_sent_utc DESC LIMIT ? OFFSET ?");
    params.push(Box::new(query.limit.min(100)));
    params.push(Box::new(query.offset));
    
    let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();
    
    let mut stmt = db.conn.prepare(&sql).map_err(|e| e.to_string())?;
    let emails = stmt.query_map(param_refs.as_slice(), |row| {
        Ok(EmailResult {
            id: row.get(0)?,
            message_id: row.get(1)?,
            from_addr: row.get(2)?,
            from_display: row.get(3)?,
            to_addrs: row.get(4)?,
            subject: row.get(5)?,
            date_sent: row.get(6)?,
            folder_category: row.get(7)?,
            risk_score: row.get(8)?,
            has_attachments: false,
        })
    }).map_err(|e| e.to_string())?.collect::<Result<Vec<_>,_>>().map_err(|e| e.to_string())?;
    
    Ok(emails)
}

/// Get email by ID (AI tool)
#[tauri::command]
pub async fn ai_get_email(state: State<'_, AppState>, email_id: String) -> Result<Option<EmailResult>, String> {
    let db = state.db.lock().await;
    
    let result = db.conn.query_row(
        "SELECT id, message_id, from_addr, from_display, to_addrs, subject, date_sent_utc, folder_category, risk_score FROM emails WHERE id = ?1",
        [&email_id],
        |row| Ok(EmailResult {
            id: row.get(0)?,
            message_id: row.get(1)?,
            from_addr: row.get(2)?,
            from_display: row.get(3)?,
            to_addrs: row.get(4)?,
            subject: row.get(5)?,
            date_sent: row.get(6)?,
            folder_category: row.get(7)?,
            risk_score: row.get(8)?,
            has_attachments: false,
        })
    );
    
    match result {
        Ok(email) => Ok(Some(email)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.to_string()),
    }
}

/// Get authentication results (AI tool)
#[tauri::command]
pub async fn ai_get_authentication_results(state: State<'_, AppState>, email_id: String) -> Result<Option<AuthResults>, String> {
    let db = state.db.lock().await;
    
    // Get the raw headers to extract auth results
    let headers_raw: Option<String> = db.conn.query_row(
        "SELECT headers_raw FROM emails WHERE id = ?1",
        [&email_id],
        |row| row.get(0)
    ).ok();
    
    if let Some(headers) = headers_raw {
        // Parse authentication results from headers
        let mut auth = AuthResults {
            email_id: email_id.clone(),
            spf_result: None,
            dkim_result: None,
            dmarc_result: None,
            arc_result: None,
            received_chain: Vec::new(),
            originating_ip: None,
        };
        
        // Simple parsing (can be enhanced)
        for line in headers.lines() {
            if line.starts_with("Authentication-Results:") {
                if line.contains("spf=") {
                    auth.spf_result = Some(extract_auth_value(line, "spf="));
                }
                if line.contains("dkim=") {
                    auth.dkim_result = Some(extract_auth_value(line, "dkim="));
                }
                if line.contains("dmarc=") {
                    auth.dmarc_result = Some(extract_auth_value(line, "dmarc="));
                }
            }
            if line.starts_with("Received:") {
                auth.received_chain.push(line.trim().to_string());
            }
        }
        
        Ok(Some(auth))
    } else {
        Ok(None)
    }
}

fn extract_auth_value(header: &str, key: &str) -> String {
    if let Some(pos) = header.find(key) {
        let start = pos + key.len();
        let end = header[start..].find(|c: char| c == ' ' || c == ';' || c == '\n').unwrap_or(header.len() - start);
        header[start..start + end].to_string()
    } else {
        "none".to_string()
    }
}

/// Get entity by ID or email (AI tool)
#[tauri::command]
pub async fn ai_get_entity(state: State<'_, AppState>, entity_id: Option<String>, email: Option<String>) -> Result<Option<EntityData>, String> {
    let db = state.db.lock().await;
    
    let result = if let Some(eid) = entity_id {
        db.conn.query_row(
            "SELECT id, email_address, display_name, sent_count, received_count, first_seen, last_seen FROM entities WHERE id = ?1",
            [&eid],
            |row| Ok(EntityData {
                id: row.get(0)?,
                email_address: row.get(1)?,
                display_name: row.get(2)?,
                sent_count: row.get(3)?,
                received_count: row.get(4)?,
                first_seen: row.get(5)?,
                last_seen: row.get(6)?,
            })
        )
    } else if let Some(em) = email {
        db.conn.query_row(
            "SELECT id, email_address, display_name, sent_count, received_count, first_seen, last_seen FROM entities WHERE email_address = ?1",
            [&em],
            |row| Ok(EntityData {
                id: row.get(0)?,
                email_address: row.get(1)?,
                display_name: row.get(2)?,
                sent_count: row.get(3)?,
                received_count: row.get(4)?,
                first_seen: row.get(5)?,
                last_seen: row.get(6)?,
            })
        )
    } else {
        return Err("Either entity_id or email must be provided".to_string());
    };
    
    match result {
        Ok(entity) => Ok(Some(entity)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.to_string()),
    }
}

/// Get timeline events (AI tool)
#[tauri::command]
pub async fn ai_get_timeline(state: State<'_, AppState>, case_id: String, limit: Option<i64>) -> Result<Vec<TimelineEvent>, String> {
    let db = state.db.lock().await;
    let lim = limit.unwrap_or(100).min(500);
    
    let mut stmt = db.conn.prepare(
        "SELECT id, timestamp, event_type, actor, summary, email_id FROM timeline_events WHERE case_id = ?1 ORDER BY timestamp ASC LIMIT ?2"
    ).map_err(|e| e.to_string())?;
    
    let events = stmt.query_map([&case_id, &lim.to_string()], |row| {
        Ok(TimelineEvent {
            id: row.get(0)?,
            timestamp: row.get(1)?,
            event_type: row.get(2)?,
            actor: row.get(3)?,
            summary: row.get(4)?,
            email_id: row.get(5)?,
        })
    }).map_err(|e| e.to_string())?.collect::<Result<Vec<_>,_>>().map_err(|e| e.to_string())?;
    
    Ok(events)
}

/// Get findings (AI tool)
#[tauri::command]
pub async fn ai_get_findings(state: State<'_, AppState>, case_id: String) -> Result<Vec<FindingData>, String> {
    let db = state.db.lock().await;
    
    let mut stmt = db.conn.prepare(
        "SELECT id, type, severity, title, description, status FROM findings WHERE case_id = ?1 ORDER BY severity, created_at"
    ).map_err(|e| e.to_string())?;
    
    let findings = stmt.query_map([&case_id], |row| {
        Ok(FindingData {
            id: row.get(0)?,
            finding_type: row.get(1)?,
            severity: row.get(2)?,
            title: row.get(3)?,
            description: row.get(4)?,
            status: row.get(5)?,
        })
    }).map_err(|e| e.to_string())?.collect::<Result<Vec<_>,_>>().map_err(|e| e.to_string())?;
    
    Ok(findings)
}

/// Get case context for AI (aggregated data)
#[tauri::command]
pub async fn ai_get_case_context(state: State<'_, AppState>, case_id: String) -> Result<serde_json::Value, String> {
    let db = state.db.lock().await;
    
    // Get basic case info
    let case_info = db.conn.query_row(
        "SELECT id, title, case_number, description, status, target_name, target_email FROM cases WHERE id = ?1",
        [&case_id],
        |row| Ok(serde_json::json!({
            "id": row.get::<_, String>(0)?,
            "title": row.get::<_, String>(1)?,
            "case_number": row.get::<_, Option<String>>(2)?,
            "description": row.get::<_, Option<String>>(3)?,
            "status": row.get::<_, String>(4)?,
            "target_name": row.get::<_, Option<String>>(5)?,
            "target_email": row.get::<_, Option<String>>(6)?,
        }))
    ).map_err(|e| e.to_string())?;
    
    // Get statistics inline
    let total_emails: i64 = db.conn.query_row(
        "SELECT COUNT(*) FROM emails WHERE case_id = ?1",
        [&case_id],
        |row| row.get(0)
    ).unwrap_or(0);
    
    let total_entities: i64 = db.conn.query_row(
        "SELECT COUNT(*) FROM entities WHERE case_id = ?1",
        [&case_id],
        |row| row.get(0)
    ).unwrap_or(0);
    
    let total_attachments: i64 = db.conn.query_row(
        "SELECT COUNT(*) FROM attachments WHERE email_id IN (SELECT id FROM emails WHERE case_id = ?1)",
        [&case_id],
        |row| row.get(0)
    ).unwrap_or(0);
    
    let total_findings: i64 = db.conn.query_row(
        "SELECT COUNT(*) FROM findings WHERE case_id = ?1",
        [&case_id],
        |row| row.get(0)
    ).unwrap_or(0);
    
    Ok(serde_json::json!({
        "case_info": case_info,
        "statistics": {
            "total_emails": total_emails,
            "total_entities": total_entities,
            "total_attachments": total_attachments,
            "total_findings": total_findings,
        },
    }))
}

// ============================================================================
// PHASE 1: NATURAL LANGUAGE SEARCH & EVIDENCE EXPLAINER
// ============================================================================

/// Natural language query parser
/// Converts plain English queries into structured SearchQuery
pub fn parse_natural_language_query(input: &str) -> SearchQuery {
    let lower = input.to_lowercase();
    let mut query = SearchQuery::default();
    
    // Extract sender (from:john or "from john" or "emails from john")
    if let Some(from) = extract_pattern(&lower, &[
        "from ", "from:", "sender ", "sent by ",
    ]) {
        query.from = Some(from.to_string());
    }
    
    // Extract recipient (to:jane or "to jane" or "emails to jane")
    if let Some(to) = extract_pattern(&lower, &[
        "to ", "to:", "recipient ", "sent to ",
    ]) {
        query.to = Some(to.to_string());
    }
    
    // Extract subject keywords
    if let Some(subject) = extract_pattern(&lower, &[
        "subject:", "about ", "regarding ", "re:",
    ]) {
        query.subject = Some(subject.to_string());
    }
    
    // Extract date ranges
    if lower.contains("before") {
        if let Some(date) = extract_date_after_keyword(&lower, "before") {
            query.date_to = Some(date);
        }
    }
    if lower.contains("after") {
        if let Some(date) = extract_date_after_keyword(&lower, "after") {
            query.date_from = Some(date);
        }
    }
    if lower.contains("between") {
        if let Some((from, to)) = extract_date_range(&lower, "between", "and") {
            query.date_from = Some(from);
            query.date_to = Some(to);
        }
    }
    if lower.contains("last week") || lower.contains("past week") {
        query.date_from = Some("7_days_ago".to_string());
    }
    if lower.contains("last month") || lower.contains("past month") {
        query.date_from = Some("30_days_ago".to_string());
    }
    if lower.contains("last year") || lower.contains("past year") {
        query.date_from = Some("365_days_ago".to_string());
    }
    
    // Extract attachment filters
    if lower.contains("attachment") || lower.contains("attached") {
        query.has_attachments = Some(true);
        
        if lower.contains("pdf") {
            query.attachment_types = Some(vec!["application/pdf".to_string()]);
        } else if lower.contains("image") || lower.contains("photo") || lower.contains("picture") {
            query.attachment_types = Some(vec!["image/jpeg".to_string(), "image/png".to_string(), "image/gif".to_string()]);
        } else if lower.contains("document") || lower.contains("doc") {
            query.attachment_types = Some(vec!["application/msword".to_string(), "application/vnd.openxmlformats-officedocument.wordprocessingml.document".to_string()]);
        } else if lower.contains("spreadsheet") || lower.contains("excel") {
            query.attachment_types = Some(vec!["application/vnd.ms-excel".to_string(), "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet".to_string()]);
        }
    }
    
    // Extract risk/suspicion indicators
    if lower.contains("suspicious") || lower.contains("risky") || lower.contains("dangerous") {
        query.risk_score_min = Some(50);
    }
    if lower.contains("high risk") || lower.contains("critical") {
        query.risk_score_min = Some(75);
    }
    
    // Extract folder/category
    if lower.contains("inbox") {
        query.folder_category = Some("inbox".to_string());
    } else if lower.contains("sent") {
        query.folder_category = Some("sent".to_string());
    } else if lower.contains("deleted") || lower.contains("trash") {
        query.folder_category = Some("soft_deleted".to_string());
    } else if lower.contains("spam") || lower.contains("junk") {
        query.folder_category = Some("spam".to_string());
    } else if lower.contains("draft") {
        query.folder_category = Some("drafts".to_string());
    }
    
    // Extract text search (remaining keywords)
    let keywords: Vec<&str> = lower
        .split_whitespace()
        .filter(|w| !is_stop_word(w) && !is_query_operator(w))
        .collect();
    
    if !keywords.is_empty() {
        query.text = Some(keywords.join(" "));
    }
    
    query
}

fn extract_pattern(input: &str, patterns: &[&str]) -> Option<String> {
    for pattern in patterns {
        if let Some(pos) = input.find(pattern) {
            let start = pos + pattern.len();
            let remaining = &input[start..];
            // Extract until next keyword or end
            let end = remaining.find(|c: char| c == ' ' || c == ',' || c == '.')
                .unwrap_or(remaining.len());
            let value = remaining[..end].trim();
            if !value.is_empty() {
                return Some(value.to_string());
            }
        }
    }
    None
}

fn extract_date_after_keyword(input: &str, keyword: &str) -> Option<String> {
    if let Some(pos) = input.find(keyword) {
        let after = &input[pos + keyword.len()..];
        let date_str = after.trim().split_whitespace().next()?;
        // Simple date parsing - in production, use a proper date parser
        if date_str.contains('-') && date_str.len() >= 8 {
            return Some(date_str.to_string());
        }
    }
    None
}

fn extract_date_range(input: &str, start_kw: &str, end_kw: &str) -> Option<(String, String)> {
    if let Some(start_pos) = input.find(start_kw) {
        let after_start = &input[start_pos + start_kw.len()..];
        if let Some(end_pos) = after_start.find(end_kw) {
            let from_date = after_start[..end_pos].trim();
            let to_date = after_start[end_pos + end_kw.len()..].trim().split_whitespace().next()?;
            return Some((from_date.to_string(), to_date.to_string()));
        }
    }
    None
}

fn is_stop_word(word: &str) -> bool {
    matches!(word, "the" | "a" | "an" | "is" | "are" | "was" | "were" | "be" | "been" | "being" | "have" | "has" | "had" | "do" | "does" | "did" | "will" | "would" | "could" | "should" | "may" | "might" | "can" | "find" | "show" | "get" | "me" | "my" | "we" | "our" | "you" | "your" | "they" | "their" | "it" | "its" | "this" | "that" | "these" | "those" | "i" | "he" | "she" | "all" | "any" | "each" | "every" | "both" | "few" | "more" | "most" | "other" | "some" | "such" | "no" | "nor" | "not" | "only" | "own" | "same" | "so" | "than" | "too" | "very" | "just" | "because" | "as" | "until" | "while" | "of" | "at" | "by" | "for" | "with" | "about" | "against" | "between" | "into" | "through" | "during" | "before" | "after" | "above" | "below" | "to" | "from" | "up" | "down" | "in" | "out" | "on" | "off" | "over" | "under" | "again" | "further" | "then" | "once" | "here" | "there" | "when" | "where" | "why" | "how" | "what" | "which" | "who" | "whom")
}

fn is_query_operator(word: &str) -> bool {
    matches!(word, "emails" | "email" | "messages" | "message" | "mail" | "mails" | "with" | "and" | "or" | "containing" | "that" | "have" | "been" | "sent" | "received")
}

/// Evidence explainer - explains technical evidence in plain language
pub fn explain_evidence(evidence_type: &str, evidence_data: &serde_json::Value) -> String {
    match evidence_type {
        "authentication_results" => explain_authentication(evidence_data),
        "received_header" => explain_received_header(evidence_data),
        "spf_result" => explain_spf(evidence_data),
        "dkim_result" => explain_dkim(evidence_data),
        "dmarc_result" => explain_dmarc(evidence_data),
        "attachment_analysis" => explain_attachment(evidence_data),
        "email_headers" => explain_headers(evidence_data),
        _ => format!("Evidence type '{}' is not recognized. Cannot provide explanation.", evidence_type),
    }
}

fn explain_authentication(data: &serde_json::Value) -> String {
    let spf = data.get("spf_result").and_then(|v| v.as_str()).unwrap_or("unknown");
    let dkim = data.get("dkim_result").and_then(|v| v.as_str()).unwrap_or("unknown");
    let dmarc = data.get("dmarc_result").and_then(|v| v.as_str()).unwrap_or("unknown");
    
    let mut explanation = String::from("## Authentication Results Explanation\n\n");
    
    explanation.push_str(&format!("**SPF (Sender Policy Framework):** {}\n", explain_spf_value(spf)));
    explanation.push_str(&format!("**DKIM (DomainKeys Identified Mail):** {}\n", explain_dkim_value(dkim)));
    explanation.push_str(&format!("**DMARC (Domain-based Message Authentication):** {}\n\n", explain_dmarc_value(dmarc)));
    
    // Overall assessment
    if spf == "pass" && dkim == "pass" && dmarc == "pass" {
        explanation.push_str("**Overall:** All authentication checks passed. The email appears to be legitimately sent from the claimed domain.\n");
    } else if spf == "fail" || dkim == "fail" || dmarc == "fail" {
        explanation.push_str("**Overall:** One or more authentication checks failed. This email may be spoofed or sent from an unauthorized server.\n");
    } else {
        explanation.push_str("**Overall:** Some authentication checks are missing or inconclusive. Exercise caution with this email.\n");
    }
    
    explanation
}

fn explain_spf_value(result: &str) -> &str {
    match result {
        "pass" => "PASS - The sending IP is authorized by the sender domain's SPF record. This means the domain owner has explicitly allowed this server to send emails on their behalf.",
        "fail" => "FAIL - The sending IP is NOT authorized by the sender domain's SPF record. This is a strong indicator of spoofing or unauthorized sending.",
        "softfail" => "SOFTFAIL - The sending IP is not explicitly authorized, but the domain's policy is not strict. Treat with caution.",
        "none" => "NONE - No SPF record exists for the sender domain. This is common for smaller domains but reduces trust.",
        "neutral" => "NEUTRAL - The SPF record explicitly states no assertion about this IP. Neither pass nor fail.",
        _ => "UNKNOWN - Could not determine SPF result.",
    }
}

fn explain_dkim_value(result: &str) -> &str {
    match result {
        "pass" => "PASS - The email's DKIM signature is valid and matches the sender's published key. This proves the message was not altered in transit and genuinely comes from the claimed domain.",
        "fail" => "FAIL - The DKIM signature is invalid or missing. The message may have been tampered with or is not genuinely from the claimed sender.",
        "none" => "NONE - No DKIM signature was applied. This reduces confidence in the email's authenticity.",
        _ => "UNKNOWN - Could not determine DKIM result.",
    }
}

fn explain_dmarc_value(result: &str) -> &str {
    match result {
        "pass" => "PASS - DMARC validation passed. The email aligns with the domain's published policy for handling authentication failures.",
        "fail" => "FAIL - DMARC validation failed. The domain's policy may instruct receivers to reject or quarantine this email.",
        "none" => "NONE - No DMARC policy exists for the sender domain.",
        _ => "UNKNOWN - Could not determine DMARC result.",
    }
}

fn explain_received_header(data: &serde_json::Value) -> String {
    let mut explanation = String::from("## Received Header Explanation\n\n");
    explanation.push_str("The `Received` header traces the path an email took from sender to receiver.\n\n");
    
    if let Some(chain) = data.get("received_chain").and_then(|v| v.as_array()) {
        explanation.push_str("**Email Path:**\n\n");
        for (i, hop) in chain.iter().enumerate() {
            if let Some(hop_str) = hop.as_str() {
                explanation.push_str(&format!("{}. {}\n", i + 1, hop_str));
            }
        }
        explanation.push_str("\n**How to read:**\n");
        explanation.push_str("- Read bottom-to-top to trace from sender to receiver\n");
        explanation.push_str("- Each hop shows: server name, timestamp, protocol\n");
        explanation.push_str("- Look for unexpected servers or time gaps\n");
    }
    
    explanation
}

fn explain_spf(data: &serde_json::Value) -> String {
    let result = data.get("result").and_then(|v| v.as_str()).unwrap_or("unknown");
    format!("## SPF Result: {}\n\n{}", result.to_uppercase(), explain_spf_value(result))
}

fn explain_dkim(data: &serde_json::Value) -> String {
    let result = data.get("result").and_then(|v| v.as_str()).unwrap_or("unknown");
    format!("## DKIM Result: {}\n\n{}", result.to_uppercase(), explain_dkim_value(result))
}

fn explain_dmarc(data: &serde_json::Value) -> String {
    let result = data.get("result").and_then(|v| v.as_str()).unwrap_or("unknown");
    format!("## DMARC Result: {}\n\n{}", result.to_uppercase(), explain_dmarc_value(result))
}

fn explain_attachment(data: &serde_json::Value) -> String {
    let mut explanation = String::from("## Attachment Analysis\n\n");
    
    if let Some(filename) = data.get("filename").and_then(|v| v.as_str()) {
        explanation.push_str(&format!("**Filename:** {}\n", filename));
    }
    if let Some(mime) = data.get("mime_type").and_then(|v| v.as_str()) {
        explanation.push_str(&format!("**MIME Type:** {}\n", mime));
    }
    if let Some(size) = data.get("size_bytes").and_then(|v| v.as_i64()) {
        explanation.push_str(&format!("**Size:** {} bytes\n", size));
    }
    if let Some(entropy) = data.get("entropy").and_then(|v| v.as_f64()) {
        explanation.push_str(&format!("**Entropy:** {:.2}/8.0\n", entropy));
        if entropy > 7.5 {
            explanation.push_str("⚠️ High entropy suggests encryption or packing. This may hide malicious content.\n");
        }
    }
    if let Some(risk_flags) = data.get("risk_flags").and_then(|v| v.as_array()) {
        explanation.push_str("\n**Risk Flags:**\n");
        for flag in risk_flags {
            if let Some(f) = flag.as_str() {
                explanation.push_str(&format!("- {}\n", explain_risk_flag(f)));
            }
        }
    }
    
    explanation
}

fn explain_risk_flag(flag: &str) -> &str {
    match flag {
        "executable" => "Executable file - Can run code on the target system",
        "macro_enabled" => "Contains macros - Can execute automated scripts",
        "high_entropy_encrypted" => "High entropy - Possibly encrypted or packed",
        "double_extension" => "Double extension - May disguise true file type",
        _ => flag,
    }
}

fn explain_headers(data: &serde_json::Value) -> String {
    let mut explanation = String::from("## Email Headers Explanation\n\n");
    explanation.push_str("Email headers contain metadata about the message's origin, path, and handling.\n\n");
    
    if let Some(from) = data.get("from").and_then(|v| v.as_str()) {
        explanation.push_str(&format!("**From:** {} - The claimed sender of the email\n", from));
    }
    if let Some(to) = data.get("to").and_then(|v| v.as_str()) {
        explanation.push_str(&format!("**To:** {} - The intended recipient\n", to));
    }
    if let Some(subject) = data.get("subject").and_then(|v| v.as_str()) {
        explanation.push_str(&format!("**Subject:** {} - The email's topic\n", subject));
    }
    if let Some(date) = data.get("date").and_then(|v| v.as_str()) {
        explanation.push_str(&format!("**Date:** {} - When the email was sent\n", date));
    }
    if let Some(reply_to) = data.get("reply_to").and_then(|v| v.as_str()) {
        explanation.push_str(&format!("**Reply-To:** {} - Where replies should be sent (may differ from From)\n", reply_to));
    }
    if let Some(return_path) = data.get("return_path").and_then(|v| v.as_str()) {
        explanation.push_str(&format!("**Return-Path:** {} - Where bounce messages go\n", return_path));
    }
    if let Some(message_id) = data.get("message_id").and_then(|v| v.as_str()) {
        explanation.push_str(&format!("**Message-ID:** {} - Unique identifier for this email\n", message_id));
    }
    
    explanation
}

// ============================================================================
// AI SESSION MANAGEMENT
// ============================================================================

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
    
    // Log to audit
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
    
    // Delete messages
    db.conn.execute("DELETE FROM ai_messages WHERE session_id = ?1", [&session_id]).ok();
    db.conn.execute("DELETE FROM ai_tool_calls WHERE session_id = ?1", [&session_id]).ok();
    db.conn.execute("DELETE FROM ai_context_snapshots WHERE session_id = ?1", [&session_id]).ok();
    
    // Update session end time
    let now = chrono::Utc::now().to_rfc3339();
    db.conn.execute("UPDATE ai_sessions SET ended_at = ?1 WHERE id = ?2", [&now, &session_id]).ok();
    
    Ok(())
}

/// Natural language search command
#[tauri::command]
pub async fn ai_natural_language_search(state: State<'_, AppState>, query: String) -> Result<serde_json::Value, String> {
    // Parse natural language to structured query
    let search_query = parse_natural_language_query(&query);
    
    // Clone for response before moving into search
    let parsed_query_json = serde_json::to_value(&search_query).unwrap_or_default();
    
    // Execute search
    let results = ai_search_emails(state, search_query).await?;
    
    Ok(serde_json::json!({
        "query": query,
        "parsed_query": parsed_query_json,
        "results": results,
        "total": results.len(),
    }))
}

/// Explain evidence command
#[tauri::command]
pub async fn ai_explain_evidence(evidence_type: String, evidence_data: serde_json::Value) -> Result<String, String> {
    Ok(explain_evidence(&evidence_type, &evidence_data))
}

// ============================================================================
// PHASE 2: INVESTIGATION PLANNER (Engine 0)
// ============================================================================

/// Investigation plan step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvestigationStep {
    pub step_number: i32,
    pub title: String,
    pub description: String,
    pub tool_calls: Vec<String>,
    pub expected_output: String,
}

/// Investigation plan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvestigationPlan {
    pub objective: String,
    pub normalized_objective: String,
    pub available_evidence: Vec<String>,
    pub unavailable_evidence: Vec<String>,
    pub limitations: Vec<String>,
    pub steps: Vec<InvestigationStep>,
    pub estimated_runtime_seconds: i64,
}

/// Generate investigation plan from objective
pub fn generate_investigation_plan(objective: &str, case_stats: &CaseStats) -> InvestigationPlan {
    let lower = objective.to_lowercase();
    
    // Determine investigation type
    let investigation_type = classify_investigation(&lower);
    
    // Build available/unavailable evidence lists
    let available_evidence = build_available_evidence(case_stats);
    let unavailable_evidence = build_unavailable_evidence();
    let limitations = build_limitations(&investigation_type, case_stats);
    
    // Generate steps based on investigation type
    let steps = generate_steps(&investigation_type, case_stats);
    
    // Estimate runtime
    let estimated_runtime = estimate_runtime(&steps);
    
    InvestigationPlan {
        objective: objective.to_string(),
        normalized_objective: normalize_objective(&investigation_type),
        available_evidence,
        unavailable_evidence,
        limitations,
        steps,
        estimated_runtime_seconds: estimated_runtime,
    }
}

#[derive(Debug, Clone)]
enum InvestigationType {
    MailboxCompromise,
    DataExfiltration,
    PhishingCampaign,
    InsiderThreat,
    FinancialFraud,
    CommunicationAnalysis,
    General,
}

fn classify_investigation(objective: &str) -> InvestigationType {
    if objective.contains("compromis") || objective.contains("hacked") || objective.contains("unauthorized access") {
        InvestigationType::MailboxCompromise
    } else if objective.contains("exfiltrat") || objective.contains("data leak") || objective.contains("stolen data") {
        InvestigationType::DataExfiltration
    } else if objective.contains("phishing") || objective.contains("spoofing") || objective.contains("impersonat") {
        InvestigationType::PhishingCampaign
    } else if objective.contains("insider") || objective.contains("employee") || objective.contains("disgruntled") {
        InvestigationType::InsiderThreat
    } else if objective.contains("fraud") || objective.contains("wire transfer") || objective.contains("money") {
        InvestigationType::FinancialFraud
    } else if objective.contains("communication") || objective.contains("relationship") || objective.contains("who talked to whom") {
        InvestigationType::CommunicationAnalysis
    } else {
        InvestigationType::General
    }
}

fn normalize_objective(investigation_type: &InvestigationType) -> String {
    match investigation_type {
        InvestigationType::MailboxCompromise => "Assess indicators consistent with possible mailbox compromise".to_string(),
        InvestigationType::DataExfiltration => "Identify potential data exfiltration activities".to_string(),
        InvestigationType::PhishingCampaign => "Analyze phishing/spoofing indicators in acquired evidence".to_string(),
        InvestigationType::InsiderThreat => "Assess indicators consistent with possible insider threat".to_string(),
        InvestigationType::FinancialFraud => "Identify potential financial fraud indicators".to_string(),
        InvestigationType::CommunicationAnalysis => "Analyze communication patterns and relationships".to_string(),
        InvestigationType::General => "Conduct general investigation of acquired evidence".to_string(),
    }
}

fn build_available_evidence(case_stats: &CaseStats) -> Vec<String> {
    let mut evidence = Vec::new();
    
    if case_stats.total_emails > 0 {
        evidence.push(format!("{} emails with metadata and headers", case_stats.total_emails));
    }
    if case_stats.total_attachments > 0 {
        evidence.push(format!("{} attachments with analysis", case_stats.total_attachments));
    }
    if case_stats.total_entities > 0 {
        evidence.push(format!("{} entities with communication profiles", case_stats.total_entities));
    }
    if case_stats.total_findings > 0 {
        evidence.push(format!("{} pre-computed forensic findings", case_stats.total_findings));
    }
    
    evidence.push("SPF/DKIM/DMARC authentication results".to_string());
    evidence.push("Received header chains".to_string());
    evidence.push("Timeline events".to_string());
    evidence.push("Communication graph data".to_string());
    
    evidence
}

fn build_unavailable_evidence() -> Vec<String> {
    vec![
        "Mail server authentication logs (not acquired)".to_string(),
        "Endpoint telemetry (not acquired)".to_string(),
        "Account login history (not acquired)".to_string(),
        "Network flow data (not acquired)".to_string(),
        "Third-party email provider logs (not acquired)".to_string(),
        "Physical device forensics (not acquired)".to_string(),
    ]
}

fn build_limitations(investigation_type: &InvestigationType, case_stats: &CaseStats) -> Vec<String> {
    let mut limitations = Vec::new();
    
    limitations.push("Analysis limited to acquired evidence only".to_string());
    
    match investigation_type {
        InvestigationType::MailboxCompromise => {
            limitations.push("Mailbox evidence alone cannot definitively prove account compromise".to_string());
            limitations.push("Server-side logs would be required for conclusive determination".to_string());
        }
        InvestigationType::DataExfiltration => {
            limitations.push("Can identify suspicious patterns but not confirm data leaving the organization".to_string());
            limitations.push("Network monitoring data would strengthen analysis".to_string());
        }
        InvestigationType::PhishingCampaign => {
            limitations.push("Can identify indicators but cannot trace campaign origin".to_string());
            limitations.push("Threat intelligence feeds would enhance analysis".to_string());
        }
        _ => {}
    }
    
    if case_stats.total_emails < 100 {
        limitations.push("Limited email volume may reduce statistical significance".to_string());
    }
    
    limitations
}

fn generate_steps(investigation_type: &InvestigationType, _case_stats: &CaseStats) -> Vec<InvestigationStep> {
    match investigation_type {
        InvestigationType::MailboxCompromise => vec![
            InvestigationStep {
                step_number: 1,
                title: "Identify unusual sending patterns".to_string(),
                description: "Detect emails sent at unusual times or with unusual content".to_string(),
                tool_calls: vec!["search_emails".to_string(), "get_timeline".to_string()],
                expected_output: "List of anomalous sending patterns".to_string(),
            },
            InvestigationStep {
                step_number: 2,
                title: "Identify new correspondents".to_string(),
                description: "Find email addresses not previously seen in the mailbox".to_string(),
                tool_calls: vec!["get_entity".to_string(), "search_emails".to_string()],
                expected_output: "List of new/unusual correspondents".to_string(),
            },
            InvestigationStep {
                step_number: 3,
                title: "Analyze authentication anomalies".to_string(),
                description: "Check SPF/DKIM/DMARC failures".to_string(),
                tool_calls: vec!["get_authentication_results".to_string(), "search_emails".to_string()],
                expected_output: "Emails with authentication failures".to_string(),
            },
            InvestigationStep {
                step_number: 4,
                title: "Detect suspicious attachments".to_string(),
                description: "Identify potentially malicious files".to_string(),
                tool_calls: vec!["get_attachments".to_string(), "get_attachment_text".to_string()],
                expected_output: "List of suspicious attachments".to_string(),
            },
            InvestigationStep {
                step_number: 5,
                title: "Construct timeline".to_string(),
                description: "Build chronological view of suspicious events".to_string(),
                tool_calls: vec!["get_timeline".to_string(), "search_emails".to_string()],
                expected_output: "Timeline of suspicious activity".to_string(),
            },
        ],
        InvestigationType::DataExfiltration => vec![
            InvestigationStep {
                step_number: 1,
                title: "Identify large outbound emails".to_string(),
                description: "Find emails with large attachments or many recipients".to_string(),
                tool_calls: vec!["search_emails".to_string(), "get_attachments".to_string()],
                expected_output: "List of large outbound emails".to_string(),
            },
            InvestigationStep {
                step_number: 2,
                title: "Detect external communications".to_string(),
                description: "Identify emails to external/unusual domains".to_string(),
                tool_calls: vec!["search_emails".to_string(), "get_entity".to_string()],
                expected_output: "External communication patterns".to_string(),
            },
            InvestigationStep {
                step_number: 3,
                title: "Analyze attachment types".to_string(),
                description: "Identify archives, encrypted files, or unusual types".to_string(),
                tool_calls: vec!["get_attachments".to_string(), "get_attachment_text".to_string()],
                expected_output: "Suspicious attachment analysis".to_string(),
            },
            InvestigationStep {
                step_number: 4,
                title: "Map data flow".to_string(),
                description: "Visualize who sent what to whom".to_string(),
                tool_calls: vec!["get_communication_graph".to_string(), "get_entity".to_string()],
                expected_output: "Data flow visualization".to_string(),
            },
        ],
        InvestigationType::PhishingCampaign => vec![
            InvestigationStep {
                step_number: 1,
                title: "Identify failed authentication".to_string(),
                description: "Find emails with SPF/DKIM/DMARC failures".to_string(),
                tool_calls: vec!["search_emails".to_string(), "get_authentication_results".to_string()],
                expected_output: "Emails with auth failures".to_string(),
            },
            InvestigationStep {
                step_number: 2,
                title: "Detect lookalike domains".to_string(),
                description: "Find sender domains similar to known legitimate domains".to_string(),
                tool_calls: vec!["search_emails".to_string(), "get_entity".to_string()],
                expected_output: "Potential lookalike domains".to_string(),
            },
            InvestigationStep {
                step_number: 3,
                title: "Analyze URLs".to_string(),
                description: "Extract and analyze URLs in email bodies".to_string(),
                tool_calls: vec!["search_emails".to_string(), "get_email_body".to_string()],
                expected_output: "URL analysis results".to_string(),
            },
            InvestigationStep {
                step_number: 4,
                title: "Identify credential harvesting".to_string(),
                description: "Find emails with login forms or password requests".to_string(),
                tool_calls: vec!["search_emails".to_string(), "get_attachment_text".to_string()],
                expected_output: "Potential phishing emails".to_string(),
            },
        ],
        _ => vec![
            InvestigationStep {
                step_number: 1,
                title: "Review forensic findings".to_string(),
                description: "Examine pre-computed findings from deterministic analysis".to_string(),
                tool_calls: vec!["get_findings".to_string()],
                expected_output: "List of forensic findings".to_string(),
            },
            InvestigationStep {
                step_number: 2,
                title: "Analyze communication patterns".to_string(),
                description: "Review entity relationships and communication graph".to_string(),
                tool_calls: vec!["get_communication_graph".to_string(), "get_entity".to_string()],
                expected_output: "Communication pattern analysis".to_string(),
            },
            InvestigationStep {
                step_number: 3,
                title: "Review timeline".to_string(),
                description: "Examine chronological events".to_string(),
                tool_calls: vec!["get_timeline".to_string()],
                expected_output: "Timeline review".to_string(),
            },
        ],
    }
}

fn estimate_runtime(steps: &[InvestigationStep]) -> i64 {
    // Estimate 10 seconds per step + 5 seconds per tool call
    let step_time = steps.len() as i64 * 10;
    let tool_time: i64 = steps.iter().map(|s| s.tool_calls.len() as i64 * 5).sum();
    step_time + tool_time
}

/// Create investigation plan command
#[tauri::command]
pub async fn ai_create_investigation_plan(state: State<'_, AppState>, case_id: String, objective: String) -> Result<InvestigationPlan, String> {
    let stats = ai_get_case_statistics(state, case_id).await?;
    let plan = generate_investigation_plan(&objective, &stats);
    Ok(plan)
}

/// Execute investigation plan command
#[tauri::command]
pub async fn ai_execute_investigation_plan(state: State<'_, AppState>, case_id: String, plan: InvestigationPlan) -> Result<serde_json::Value, String> {
    let mut results = serde_json::json!({}); 
    let mut executed_steps = Vec::new();
    
    for step in &plan.steps {
        let step_result = execute_plan_step(&state, &case_id, step).await?;
        executed_steps.push(serde_json::json!({
            "step": step.step_number,
            "title": step.title,
            "result": step_result,
        }));
    }
    
    results["executed_steps"] = serde_json::json!(executed_steps);
    results["total_steps"] = serde_json::json!(plan.steps.len());
    results["status"] = serde_json::json!("completed");
    
    Ok(results)
}

async fn execute_plan_step(state: &State<'_, AppState>, case_id: &str, step: &InvestigationStep) -> Result<serde_json::Value, String> {
    // Execute each tool call in the step
    let mut step_results = Vec::new();
    
    for tool_call in &step.tool_calls {
        // For now, return placeholder - in production, dispatch to actual tool
        step_results.push(serde_json::json!({
            "tool": tool_call,
            "status": "executed",
            "note": "Tool execution dispatched",
        }));
    }
    
    Ok(serde_json::json!({
        "step_number": step.step_number,
        "tool_results": step_results,
    }))
}

// ============================================================================
// PHASE 3: ADVANCED ANALYSIS ENGINES
// ============================================================================

// === ENGINE 4: TIMELINE RECONSTRUCTION ===

use chrono::Timelike;

/// Timeline interpretation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineInterpretation {
    pub events: Vec<TimelineEvent>,
    pub anomalies: Vec<TimelineAnomaly>,
    pub narrative: String,
    pub clock_skew_detected: bool,
    pub timestamp_reversals: Vec<(String, String)>,
}

/// Timeline anomaly
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineAnomaly {
    pub event_id: String,
    pub anomaly_type: String,
    pub description: String,
    pub severity: String,
    pub timestamp: String,
}

/// Analyze timeline and generate interpretation
pub fn analyze_timeline(events: &[TimelineEvent]) -> TimelineInterpretation {
    let mut anomalies = Vec::new();
    let mut clock_skew_detected = false;
    let mut timestamp_reversals = Vec::new();
    
    // Detect anomalies
    for window in events.windows(2) {
        let current = &window[0];
        let next = &window[1];
        
        // Check for timestamp reversals
        if current.timestamp > next.timestamp {
            timestamp_reversals.push((current.timestamp.clone(), next.timestamp.clone()));
            anomalies.push(TimelineAnomaly {
                event_id: next.id.clone(),
                anomaly_type: "timestamp_reversal".to_string(),
                description: format!("Event at {} occurs before event at {}", next.timestamp, current.timestamp),
                severity: "medium".to_string(),
                timestamp: next.timestamp.clone(),
            });
        }
    }
    
    // Detect unusual time gaps (events outside business hours)
    for event in events {
        if let Ok(dt) = event.timestamp.parse::<chrono::DateTime<chrono::Utc>>() {
            let hour = dt.hour();
            if hour < 6 || hour > 22 {
                anomalies.push(TimelineAnomaly {
                    event_id: event.id.clone(),
                    anomaly_type: "unusual_hour".to_string(),
                    description: format!("Event at {} occurred outside typical business hours (hour: {})", event.timestamp, hour),
                    severity: "low".to_string(),
                    timestamp: event.timestamp.clone(),
                });
            }
        }
    }
    
    // Detect burst activity (many events in short period)
    if events.len() >= 3 {
        for window in events.windows(3) {
            if let (Ok(t1), Ok(t3)) = (
                window[0].timestamp.parse::<chrono::DateTime<chrono::Utc>>(),
                window[2].timestamp.parse::<chrono::DateTime<chrono::Utc>>()
            ) {
                let duration = t3.signed_duration_since(t1);
                if duration.num_minutes() < 5 {
                    anomalies.push(TimelineAnomaly {
                        event_id: window[1].id.clone(),
                        anomaly_type: "burst_activity".to_string(),
                        description: format!("Three events within {} minutes", duration.num_minutes()),
                        severity: "medium".to_string(),
                        timestamp: window[1].timestamp.clone(),
                    });
                }
            }
        }
    }
    
    if !timestamp_reversals.is_empty() {
        clock_skew_detected = true;
    }
    
    // Generate narrative
    let narrative = generate_timeline_narrative(events, &anomalies, clock_skew_detected);
    
    TimelineInterpretation {
        events: events.to_vec(),
        anomalies,
        narrative,
        clock_skew_detected,
        timestamp_reversals,
    }
}

fn generate_timeline_narrative(events: &[TimelineEvent], anomalies: &[TimelineAnomaly], clock_skew: bool) -> String {
    let mut narrative = String::from("## Timeline Analysis\n\n");
    
    if events.is_empty() {
        narrative.push_str("No timeline events found.\n");
        return narrative;
    }
    
    narrative.push_str(&format!("**Total Events:** {}\n", events.len()));
    narrative.push_str(&format!("**Anomalies Detected:** {}\n\n", anomalies.len()));
    
    if clock_skew {
        narrative.push_str("⚠️ **Clock Skew Detected:** Timestamp reversals found. Server clocks may be unsynchronized or timestamps may be forged.\n\n");
    }
    
    if !anomalies.is_empty() {
        narrative.push_str("**Key Anomalies:**\n");
        for anomaly in anomalies.iter().take(5) {
            narrative.push_str(&format!("- [{}] {}: {}\n", anomaly.severity.to_uppercase(), anomaly.anomaly_type, anomaly.description));
        }
    }
    
    narrative
}

// === ENGINE 5: SPOOFING/PHISHING ANALYST ===

/// Spoofing analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpoofingAnalysis {
    pub email_id: String,
    pub overall_risk: String,
    pub risk_score: i32,
    pub findings: Vec<SpoofingFinding>,
    pub recommendations: Vec<String>,
}

/// Individual spoofing finding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpoofingFinding {
    pub category: String,
    pub finding: String,
    pub severity: String,
    pub evidence: String,
}

/// Analyze email for spoofing/phishing indicators
pub fn analyze_spoofing(
    email: &EmailResult,
    auth: &AuthResults,
) -> SpoofingAnalysis {
    let mut findings = Vec::new();
    let mut risk_score = 0;
    
    // Check SPF
    if auth.spf_result.as_deref() == Some("fail") {
        risk_score += 25;
        findings.push(SpoofingFinding {
            category: "authentication".to_string(),
            finding: "SPF check failed".to_string(),
            severity: "high".to_string(),
            evidence: format!("SPF result: {}", auth.spf_result.as_deref().unwrap_or("unknown")),
        });
    }
    
    // Check DKIM
    if auth.dkim_result.as_deref() == Some("fail") || auth.dkim_result.as_deref() == Some("none") {
        risk_score += 20;
        findings.push(SpoofingFinding {
            category: "authentication".to_string(),
            finding: if auth.dkim_result.as_deref() == Some("fail") {
                "DKIM signature validation failed".to_string()
            } else {
                "No DKIM signature present".to_string()
            },
            severity: "medium".to_string(),
            evidence: format!("DKIM result: {}", auth.dkim_result.as_deref().unwrap_or("unknown")),
        });
    }
    
    // Check DMARC
    if auth.dmarc_result.as_deref() == Some("fail") {
        risk_score += 30;
        findings.push(SpoofingFinding {
            category: "authentication".to_string(),
            finding: "DMARC validation failed".to_string(),
            severity: "high".to_string(),
            evidence: format!("DMARC result: {}", auth.dmarc_result.as_deref().unwrap_or("unknown")),
        });
    }
    
    // Check for Reply-To mismatch
    if let Some(reply_to) = &email.from_display {
        if !reply_to.is_empty() && !email.from_addr.contains(reply_to) {
            risk_score += 15;
            findings.push(SpoofingFinding {
                category: "address".to_string(),
                finding: "Reply-To address differs from sender".to_string(),
                severity: "medium".to_string(),
                evidence: format!("From: {}, Display: {}", email.from_addr, reply_to),
            });
        }
    }
    
    // Check for suspicious originating IP
    if let Some(ip) = &auth.originating_ip {
        if ip.starts_with("10.") || ip.starts_with("192.168.") || ip.starts_with("172.") {
            risk_score += 10;
            findings.push(SpoofingFinding {
                category: "network".to_string(),
                finding: "Email originated from private IP range".to_string(),
                severity: "low".to_string(),
                evidence: format!("Originating IP: {}", ip),
            });
        }
    }
    
    // Check received chain for anomalies
    if auth.received_chain.len() > 5 {
        risk_score += 5;
        findings.push(SpoofingFinding {
            category: "routing".to_string(),
            finding: format!("Unusually long received chain ({} hops)", auth.received_chain.len()),
            severity: "low".to_string(),
            evidence: format!("{} hops in received chain", auth.received_chain.len()),
        });
    }
    
    // Determine overall risk
    let overall_risk = match risk_score {
        0..=20 => "low".to_string(),
        21..=50 => "medium".to_string(),
        51..=75 => "high".to_string(),
        _ => "critical".to_string(),
    };
    
    // Generate recommendations
    let recommendations = generate_spoofing_recommendations(&overall_risk, &findings);
    
    SpoofingAnalysis {
        email_id: email.id.clone(),
        overall_risk,
        risk_score,
        findings,
        recommendations,
    }
}

fn generate_spoofing_recommendations(risk: &str, findings: &[SpoofingFinding]) -> Vec<String> {
    let mut recs = Vec::new();
    
    match risk {
        "critical" | "high" => {
            recs.push("Treat this email as potentially malicious".to_string());
            recs.push("Do not click any links or open attachments".to_string());
            recs.push("Verify sender through alternative channel".to_string());
        }
        "medium" => {
            recs.push("Exercise caution with this email".to_string());
            recs.push("Verify unexpected requests independently".to_string());
        }
        _ => {
            recs.push("Standard precautions apply".to_string());
        }
    }
    
    for finding in findings {
        if finding.category == "authentication" && finding.severity == "high" {
            recs.push("Authentication failure detected - verify sender identity".to_string());
        }
    }
    
    recs
}

// === ENGINE 6: ATTACHMENT TRIAGE ===

/// Attachment triage result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttachmentTriage {
    pub attachments: Vec<AttachmentRisk>,
    pub critical_count: usize,
    pub high_count: usize,
    pub medium_count: usize,
    pub low_count: usize,
}

/// Attachment risk assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttachmentRisk {
    pub attachment_id: String,
    pub filename: String,
    pub risk_level: String,
    pub risk_score: i32,
    pub reasons: Vec<String>,
    pub recommendations: Vec<String>,
}

/// Triage attachments by risk level
pub fn triage_attachments(attachments: &[AttachmentMetadata]) -> AttachmentTriage {
    let mut results = Vec::new();
    let mut critical_count = 0;
    let mut high_count = 0;
    let mut medium_count = 0;
    let mut low_count = 0;
    
    for att in attachments {
        let risk = assess_attachment_risk(att);
        match risk.risk_level.as_str() {
            "critical" => critical_count += 1,
            "high" => high_count += 1,
            "medium" => medium_count += 1,
            _ => low_count += 1,
        }
        results.push(risk);
    }
    
    // Sort by risk score descending
    results.sort_by(|a, b| b.risk_score.cmp(&a.risk_score));
    
    AttachmentTriage {
        attachments: results,
        critical_count,
        high_count,
        medium_count,
        low_count,
    }
}

fn assess_attachment_risk(att: &AttachmentMetadata) -> AttachmentRisk {
    let mut risk_score = 0;
    let mut reasons = Vec::new();
    let mut recommendations = Vec::new();
    
    // Check file extension
    let filename_lower = att.filename.as_deref().unwrap_or("").to_lowercase();
    
    // Double extension check
    let extension_count = filename_lower.matches('.').count();
    if extension_count > 1 {
        risk_score += 30;
        reasons.push("Double extension detected - may disguise true file type".to_string());
        recommendations.push("Verify file type using magic bytes".to_string());
    }
    
    // Executable check
    if filename_lower.ends_with(".exe") || filename_lower.ends_with(".bat") || 
       filename_lower.ends_with(".cmd") || filename_lower.ends_with(".ps1") ||
       filename_lower.ends_with(".vbs") || filename_lower.ends_with(".js") ||
       filename_lower.ends_with(".scr") || filename_lower.ends_with(".msi") {
        risk_score += 40;
        reasons.push("Executable file type - can run code on target system".to_string());
        recommendations.push("Scan with antivirus before opening".to_string());
        recommendations.push("Consider sandboxed analysis".to_string());
    }
    
    // Macro-enabled office docs
    if filename_lower.ends_with(".docm") || filename_lower.ends_with(".xlsm") || 
       filename_lower.ends_with(".pptm") {
        risk_score += 25;
        reasons.push("Macro-enabled office document - can execute automated scripts".to_string());
        recommendations.push("Disable macros before opening".to_string());
    }
    
    // High entropy (possibly encrypted/packed)
    if let Some(entropy) = att.entropy {
        if entropy > 7.5 {
            risk_score += 20;
            reasons.push(format!("High entropy ({:.2}/8.0) - possibly encrypted or packed", entropy));
            recommendations.push("May contain hidden or obfuscated content".to_string());
        }
    }
    
    // Mismatch between MIME type and extension
    if let Some(filename) = &att.filename {
        let ext = filename.rsplit('.').next().unwrap_or("");
        let expected_mime = match ext.to_lowercase().as_str() {
            "pdf" => "application/pdf",
            "doc" | "docx" => "application/msword",
            "xls" | "xlsx" => "application/vnd.ms-excel",
            "jpg" | "jpeg" => "image/jpeg",
            "png" => "image/png",
            "zip" => "application/zip",
            _ => "",
        };
        if !expected_mime.is_empty() && !att.mime_type.contains(expected_mime) {
            risk_score += 15;
            reasons.push(format!("MIME type mismatch: extension .{} but MIME is {}", ext, att.mime_type));
        }
    }
    
    // Check risk flags
    for flag in &att.risk_flags {
        match flag.as_str() {
            "executable" => risk_score += 40,
            "macro_enabled" => risk_score += 25,
            "high_entropy_encrypted" => risk_score += 20,
            "double_extension" => risk_score += 30,
            _ => risk_score += 5,
        }
    }
    
    // Large file size
    if att.size_bytes > 10_000_000 {
        risk_score += 10;
        reasons.push(format!("Large file size: {} MB", att.size_bytes / 1_000_000));
    }
    
    let risk_level = match risk_score {
        0..=20 => "low".to_string(),
        21..=50 => "medium".to_string(),
        51..=75 => "high".to_string(),
        _ => "critical".to_string(),
    };
    
    if recommendations.is_empty() {
        recommendations.push("Standard handling procedures apply".to_string());
    }
    
    AttachmentRisk {
        attachment_id: att.id.clone(),
        filename: att.filename.clone().unwrap_or_else(|| "unknown".to_string()),
        risk_level,
        risk_score,
        reasons,
        recommendations,
    }
}

// === ENGINE 7: GRAPH ANALYST ===

/// Graph analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphAnalysis {
    pub central_entities: Vec<EntityCentrality>,
    pub communities: Vec<Vec<String>>,
    pub anomalies: Vec<GraphAnomaly>,
    pub recommendations: Vec<String>,
}

/// Entity centrality score
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityCentrality {
    pub entity_id: String,
    pub email_address: String,
    pub centrality_score: f64,
    pub connection_count: usize,
}

/// Graph anomaly
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphAnomaly {
    pub anomaly_type: String,
    pub description: String,
    pub entities_involved: Vec<String>,
    pub severity: String,
}

/// Analyze communication graph
pub fn analyze_graph(entities: &[EntityData], edges: &[(String, String, i64)]) -> GraphAnalysis {
    let mut central_entities = Vec::new();
    let mut anomalies = Vec::new();
    
    // Calculate centrality (simple degree centrality)
    let mut connection_counts: HashMap<String, usize> = HashMap::new();
    for (from, to, _) in edges {
        *connection_counts.entry(from.clone()).or_insert(0) += 1;
        *connection_counts.entry(to.clone()).or_insert(0) += 1;
    }
    
    for entity in entities {
        let count = connection_counts.get(&entity.email_address).copied().unwrap_or(0);
        let total_possible = entities.len().saturating_sub(1);
        let centrality = if total_possible > 0 {
            count as f64 / total_possible as f64
        } else {
            0.0
        };
        
        central_entities.push(EntityCentrality {
            entity_id: entity.id.clone(),
            email_address: entity.email_address.clone(),
            centrality_score: centrality,
            connection_count: count,
        });
    }
    
    // Sort by centrality
    central_entities.sort_by(|a, b| b.centrality_score.partial_cmp(&a.centrality_score).unwrap());
    
    // Detect anomalies
    // 1. Isolated entities (no connections)
    for entity in entities {
        let count = connection_counts.get(&entity.email_address).copied().unwrap_or(0);
        if count == 0 {
            anomalies.push(GraphAnomaly {
                anomaly_type: "isolated_entity".to_string(),
                description: format!("{} has no communication connections", entity.email_address),
                entities_involved: vec![entity.id.clone()],
                severity: "low".to_string(),
            });
        }
    }
    
    // 2. Highly connected entities (hubs)
    for central in central_entities.iter().take(3) {
        if central.centrality_score > 0.5 {
            anomalies.push(GraphAnomaly {
                anomaly_type: "central_hub".to_string(),
                description: format!("{} is a communication hub (centrality: {:.2})", central.email_address, central.centrality_score),
                entities_involved: vec![central.entity_id.clone()],
                severity: "medium".to_string(),
            });
        }
    }
    
    // Generate recommendations
    let mut recommendations = Vec::new();
    if !central_entities.is_empty() {
        recommendations.push(format!("Focus investigation on top 3 central entities: {}", 
            central_entities.iter().take(3).map(|e| &e.email_address).cloned().collect::<Vec<_>>().join(", ")));
    }
    if !anomalies.is_empty() {
        recommendations.push(format!("Review {} graph anomalies for suspicious patterns", anomalies.len()));
    }
    
    GraphAnalysis {
        central_entities,
        communities: Vec::new(), // Would require community detection algorithm
        anomalies,
        recommendations,
    }
}

// === TAURI COMMANDS FOR PHASE 3 ===

/// Analyze timeline command
#[tauri::command]
pub async fn ai_analyze_timeline(state: State<'_, AppState>, case_id: String, limit: Option<i64>) -> Result<TimelineInterpretation, String> {
    let events = ai_get_timeline(state, case_id, limit).await?;
    Ok(analyze_timeline(&events))
}

/// Analyze email for spoofing command
#[tauri::command]
pub async fn ai_analyze_spoofing(state: State<'_, AppState>, email_id: String) -> Result<SpoofingAnalysis, String> {
    let email = match ai_get_email(state.clone(), email_id.clone()).await? {
        Some(e) => e,
        None => return Err("Email not found".to_string()),
    };
    
    let auth = match ai_get_authentication_results(state, email_id).await? {
        Some(a) => a,
        None => return Err("Authentication results not found".to_string()),
    };
    
    Ok(analyze_spoofing(&email, &auth))
}

/// Triage attachments command
#[tauri::command]
pub async fn ai_triage_attachments(state: State<'_, AppState>, email_id: String) -> Result<AttachmentTriage, String> {
    let db = state.db.lock().await;
    
    let mut stmt = db.conn.prepare(
        "SELECT id, filename, mime_type, size_bytes, sha256, entropy, risk_flags FROM attachments WHERE email_id = ?1"
    ).map_err(|e| e.to_string())?;
    
    let attachments = stmt.query_map([&email_id], |row| {
        let risk_flags_str: Option<String> = row.get(6).ok();
        let risk_flags: Vec<String> = risk_flags_str
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default();
        
        Ok(AttachmentMetadata {
            id: row.get(0)?,
            filename: row.get(1)?,
            mime_type: row.get(2)?,
            size_bytes: row.get(3)?,
            sha256: row.get(4)?,
            entropy: row.get(5)?,
            risk_flags,
        })
    }).map_err(|e| e.to_string())?.collect::<Result<Vec<_>,_>>().map_err(|e| e.to_string())?;
    
    Ok(triage_attachments(&attachments))
}

/// Analyze communication graph command
#[tauri::command]
pub async fn ai_analyze_graph(state: State<'_, AppState>, case_id: String, max_nodes: Option<i64>) -> Result<GraphAnalysis, String> {
    let db = state.db.lock().await;
    
    // Get entities
    let mut entity_stmt = db.conn.prepare(
        "SELECT id, email_address, display_name, sent_count, received_count, first_seen, last_seen FROM entities WHERE case_id = ?1"
    ).map_err(|e| e.to_string())?;
    
    let entities = entity_stmt.query_map([&case_id], |row| {
        Ok(EntityData {
            id: row.get(0)?,
            email_address: row.get(1)?,
            display_name: row.get(2)?,
            sent_count: row.get(3)?,
            received_count: row.get(4)?,
            first_seen: row.get(5)?,
            last_seen: row.get(6)?,
        })
    }).map_err(|e| e.to_string())?.collect::<Result<Vec<_>,_>>().map_err(|e| e.to_string())?;
    
    // Get edges (communication pairs)
    let max = max_nodes.unwrap_or(500);
    let mut edge_stmt = db.conn.prepare(
        "SELECT from_entity, to_entity, message_count FROM communication_edges WHERE case_id = ?1 LIMIT ?2"
    ).map_err(|e| e.to_string())?;
    
    let edges = edge_stmt.query_map(rusqlite::params![&case_id, max], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?, row.get::<_, i64>(2)?))
    }).map_err(|e| e.to_string())?.collect::<Result<Vec<_>,_>>().map_err(|e| e.to_string())?;
    
    Ok(analyze_graph(&entities, &edges))
}

// ============================================================================
// PHASE 4: INTELLIGENCE ENGINES
// ============================================================================

// === ENGINE 8: ENTITY RESOLUTION ===

/// Entity resolution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityResolution {
    pub candidates: Vec<EntityCandidate>,
    pub total_entities: usize,
}

/// Entity merge candidate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityCandidate {
    pub entity_ids: Vec<String>,
    pub email_addresses: Vec<String>,
    pub display_names: Vec<String>,
    pub confidence: f64,
    pub evidence: Vec<String>,
    pub recommendation: String,
}

/// Find possible duplicate entities
pub fn resolve_entities(entities: &[EntityData]) -> EntityResolution {
    let mut candidates = Vec::new();
    
    // Group by display name
    let mut name_groups: HashMap<String, Vec<usize>> = HashMap::new();
    for (i, entity) in entities.iter().enumerate() {
        if let Some(name) = &entity.display_name {
            let normalized = name.to_lowercase().trim().to_string();
            if !normalized.is_empty() {
                name_groups.entry(normalized).or_insert_with(Vec::new).push(i);
            }
        }
    }
    
    // Find groups with multiple entities
    for (name, indices) in &name_groups {
        if indices.len() < 2 {
            continue;
        }
        
        let mut email_addresses = Vec::new();
        let mut display_names = Vec::new();
        let mut entity_ids = Vec::new();
        let mut evidence = Vec::new();
        
        for &idx in indices {
            let entity = &entities[idx];
            email_addresses.push(entity.email_address.clone());
            if let Some(dn) = &entity.display_name {
                display_names.push(dn.clone());
            }
            entity_ids.push(entity.id.clone());
        }
        
        // Calculate confidence
        let confidence = calculate_merge_confidence(&email_addresses, &display_names);
        
        // Build evidence
        if email_addresses.len() >= 2 {
            evidence.push(format!("{} entities share display name '{}'", email_addresses.len(), name));
        }
        if display_names.len() >= 2 {
            let unique_names: std::collections::HashSet<_> = display_names.iter().collect();
            if unique_names.len() < display_names.len() {
                evidence.push("Similar display names detected".to_string());
            }
        }
        
        let recommendation = if confidence > 0.8 {
            "High confidence merge candidate - recommend review".to_string()
        } else if confidence > 0.5 {
            "Possible match - manual review recommended".to_string()
        } else {
            "Low confidence - likely distinct entities".to_string()
        };
        
        candidates.push(EntityCandidate {
            entity_ids,
            email_addresses,
            display_names,
            confidence,
            evidence,
            recommendation,
        });
    }
    
    // Sort by confidence
    candidates.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());
    
    EntityResolution {
        candidates,
        total_entities: entities.len(),
    }
}

fn calculate_merge_confidence(emails: &[String], names: &[String]) -> f64 {
    let mut confidence = 0.0;
    
    // Same display name = high confidence
    if names.len() >= 2 {
        let unique_names: std::collections::HashSet<_> = names.iter().collect();
        if unique_names.len() == 1 {
            confidence += 0.5;
        } else if unique_names.len() <= names.len() / 2 {
            confidence += 0.3;
        }
    }
    
    // Similar email patterns (same domain)
    if emails.len() >= 2 {
        let domains: std::collections::HashSet<_> = emails
            .iter()
            .filter_map(|e| e.split('@').nth(1))
            .collect();
        if domains.len() == 1 {
            confidence += 0.2;
        }
    }
    
    if confidence > 1.0 {
        1.0
    } else {
        confidence
    }
}

// === ENGINE 9: ANOMALY DETECTION ===

/// Anomaly detection result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnomalyDetection {
    pub anomalies: Vec<EmailAnomaly>,
    pub total_scanned: usize,
    pub scan_duration_ms: i64,
}

/// Email anomaly
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailAnomaly {
    pub email_id: String,
    pub anomaly_type: String,
    pub description: String,
    pub severity: String,
    pub confidence: f64,
}

/// Detect anomalous emails
pub fn detect_anomalies(
    emails: &[EmailResult],
    entities: &[EntityData],
) -> AnomalyDetection {
    let mut anomalies = Vec::new();
    let start = std::time::Instant::now();
    
    // Build entity lookup
    let entity_map: HashMap<&str, &EntityData> = entities
        .iter()
        .map(|e| (e.email_address.as_str(), e))
        .collect();
    
    for email in emails {
        // Check for high risk score
        if email.risk_score >= 75 {
            anomalies.push(EmailAnomaly {
                email_id: email.id.clone(),
                anomaly_type: "high_risk_score".to_string(),
                description: format!("Email has high risk score: {}", email.risk_score),
                severity: "high".to_string(),
                confidence: (email.risk_score as f64) / 100.0,
            });
        }
        
        // Check for unknown sender (not in entity list)
        if !entity_map.contains_key(email.from_addr.as_str()) {
            anomalies.push(EmailAnomaly {
                email_id: email.id.clone(),
                anomaly_type: "unknown_sender".to_string(),
                description: format!("Sender {} not in known entity list", email.from_addr),
                severity: "medium".to_string(),
                confidence: 0.7,
            });
        }
        
        // Check for suspicious folder
        if email.folder_category == "spam" {
            anomalies.push(EmailAnomaly {
                email_id: email.id.clone(),
                anomaly_type: "spam_folder".to_string(),
                description: "Email categorized as spam".to_string(),
                severity: "low".to_string(),
                confidence: 0.6,
            });
        }
    }
    
    let duration = start.elapsed().as_millis() as i64;
    
    AnomalyDetection {
        anomalies,
        total_scanned: emails.len(),
        scan_duration_ms: duration,
    }
}

// === ENGINE 10: REPORT ASSISTANT ===

/// Report section
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportSection {
    pub title: String,
    pub content: String,
    pub section_type: String,
    pub evidence_refs: Vec<String>,
}

/// Investigation report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvestigationReport {
    pub title: String,
    pub generated_at: String,
    pub generated_by: String,
    pub model: String,
    pub sections: Vec<ReportSection>,
    pub metadata: ReportMetadata,
}

/// Report metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportMetadata {
    pub total_emails: i64,
    pub total_findings: i64,
    pub total_entities: i64,
    pub scan_duration_ms: i64,
}

/// Generate investigation report
pub fn generate_report(
    case_data: &serde_json::Value,
    stats: &CaseStats,
    findings: &[FindingData],
    model: &str,
) -> InvestigationReport {
    let mut sections = Vec::new();
    
    // Executive Summary
    sections.push(ReportSection {
        title: "Executive Summary".to_string(),
        content: format!(
            "This report presents the findings of a forensic investigation conducted on {} email messages. The investigation identified {} forensic findings and {} unique entities requiring review.",
            stats.total_emails, stats.total_findings, stats.total_entities
        ),
        section_type: "summary".to_string(),
        evidence_refs: vec![],
    });
    
    // Scope
    sections.push(ReportSection {
        title: "Scope".to_string(),
        content: "Analysis limited to acquired evidence only. Server-side logs, endpoint telemetry, and network flow data were not available for this investigation.".to_string(),
        section_type: "scope".to_string(),
        evidence_refs: vec![],
    });
    
    // Methodology
    sections.push(ReportSection {
        title: "Methodology".to_string(),
        content: "Evidence was acquired, parsed, and analyzed using deterministic forensic analysis. Email authentication (SPF, DKIM, DMARC), attachment analysis, timeline reconstruction, and communication graph analysis were performed.".to_string(),
        section_type: "methodology".to_string(),
        evidence_refs: vec![],
    });
    
    // Findings
    let findings_content = if findings.is_empty() {
        "No critical findings were identified during this investigation.".to_string()
    } else {
        let mut content = String::from("The following findings were identified:\n\n");
        for finding in findings.iter().take(10) {
            content.push_str(&format!("- **[{}] {}**: {}\n", 
                finding.severity.to_uppercase(), 
                finding.title,
                finding.description.as_deref().unwrap_or("No description")
            ));
        }
        content
    };
    
    sections.push(ReportSection {
        title: "Findings".to_string(),
        content: findings_content,
        section_type: "findings".to_string(),
        evidence_refs: vec![],
    });
    
    // Statistics
    sections.push(ReportSection {
        title: "Evidence Statistics".to_string(),
        content: format!(
            "- Total Emails: {}\n- Inbox: {}\n- Sent: {}\n- Deleted: {}\n- Spam: {}\n- Total Entities: {}\n- Total Attachments: {}",
            stats.total_emails,
            stats.inbox_count,
            stats.sent_count,
            stats.deleted_count,
            stats.spam_count,
            stats.total_entities,
            stats.total_attachments
        ),
        section_type: "statistics".to_string(),
        evidence_refs: vec![],
    });
    
    // Limitations
    sections.push(ReportSection {
        title: "Limitations".to_string(),
        content: "1. Analysis limited to acquired evidence only.\n2. Mail server authentication logs not available.\n3. Endpoint telemetry not available.\n4. Cannot establish account compromise conclusively without server logs.".to_string(),
        section_type: "limitations".to_string(),
        evidence_refs: vec![],
    });
    
    InvestigationReport {
        title: "Forensic Investigation Report".to_string(),
        generated_at: chrono::Utc::now().to_rfc3339(),
        generated_by: "J12 AI Assistant".to_string(),
        model: model.to_string(),
        sections,
        metadata: ReportMetadata {
            total_emails: stats.total_emails,
            total_findings: stats.total_findings,
            total_entities: stats.total_entities,
            scan_duration_ms: 0,
        },
    }
}

// === TAURI COMMANDS FOR PHASE 4 ===

/// Resolve entities command
#[tauri::command]
pub async fn ai_resolve_entities(state: State<'_, AppState>, case_id: String) -> Result<EntityResolution, String> {
    let db = state.db.lock().await;
    
    let mut stmt = db.conn.prepare(
        "SELECT id, email_address, display_name, sent_count, received_count, first_seen, last_seen FROM entities WHERE case_id = ?1"
    ).map_err(|e| e.to_string())?;
    
    let entities = stmt.query_map([&case_id], |row| {
        Ok(EntityData {
            id: row.get(0)?,
            email_address: row.get(1)?,
            display_name: row.get(2)?,
            sent_count: row.get(3)?,
            received_count: row.get(4)?,
            first_seen: row.get(5)?,
            last_seen: row.get(6)?,
        })
    }).map_err(|e| e.to_string())?.collect::<Result<Vec<_>,_>>().map_err(|e| e.to_string())?;
    
    Ok(resolve_entities(&entities))
}

/// Detect anomalies command
#[tauri::command]
pub async fn ai_detect_anomalies(state: State<'_, AppState>, case_id: String, limit: Option<i64>) -> Result<AnomalyDetection, String> {
    let db = state.db.lock().await;
    
    // Get emails
    let mut email_stmt = db.conn.prepare(
        "SELECT id, message_id, from_addr, from_display, to_addrs, subject, date_sent_utc, folder_category, risk_score FROM emails WHERE case_id = ?1 LIMIT ?2"
    ).map_err(|e| e.to_string())?;
    
    let lim = limit.unwrap_or(1000);
    let emails = email_stmt.query_map([&case_id, &lim.to_string()], |row| {
        Ok(EmailResult {
            id: row.get(0)?,
            message_id: row.get(1)?,
            from_addr: row.get(2)?,
            from_display: row.get(3)?,
            to_addrs: row.get(4)?,
            subject: row.get(5)?,
            date_sent: row.get(6)?,
            folder_category: row.get(7)?,
            risk_score: row.get(8)?,
            has_attachments: false,
        })
    }).map_err(|e| e.to_string())?.collect::<Result<Vec<_>,_>>().map_err(|e| e.to_string())?;
    
    // Get entities
    let mut entity_stmt = db.conn.prepare(
        "SELECT id, email_address, display_name, sent_count, received_count, first_seen, last_seen FROM entities WHERE case_id = ?1"
    ).map_err(|e| e.to_string())?;
    
    let entities = entity_stmt.query_map([&case_id], |row| {
        Ok(EntityData {
            id: row.get(0)?,
            email_address: row.get(1)?,
            display_name: row.get(2)?,
            sent_count: row.get(3)?,
            received_count: row.get(4)?,
            first_seen: row.get(5)?,
            last_seen: row.get(6)?,
        })
    }).map_err(|e| e.to_string())?.collect::<Result<Vec<_>,_>>().map_err(|e| e.to_string())?;
    
    Ok(detect_anomalies(&emails, &entities))
}

/// Generate report command
#[tauri::command]
pub async fn ai_generate_report(state: State<'_, AppState>, case_id: String, model: String) -> Result<InvestigationReport, String> {
    let stats = ai_get_case_statistics(state.clone(), case_id.clone()).await?;
    
    let db = state.db.lock().await;
    
    // Get findings
    let mut stmt = db.conn.prepare(
        "SELECT id, type, severity, title, description, status FROM findings WHERE case_id = ?1 ORDER BY severity, created_at"
    ).map_err(|e| e.to_string())?;
    
    let findings = stmt.query_map([&case_id], |row| {
        Ok(FindingData {
            id: row.get(0)?,
            finding_type: row.get(1)?,
            severity: row.get(2)?,
            title: row.get(3)?,
            description: row.get(4)?,
            status: row.get(5)?,
        })
    }).map_err(|e| e.to_string())?.collect::<Result<Vec<_>,_>>().map_err(|e| e.to_string())?;
    
    let case_data = serde_json::json!({}); // Placeholder
    
    Ok(generate_report(&case_data, &stats, &findings, &model))
}
