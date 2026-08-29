use tauri::State;
use crate::AppState;
use super::types::{
    CaseStats, SearchQuery, EmailResult, AuthResults, EntityData, TimelineEvent,
    FindingData, ToolRiskLevel, ToolDefinition, ToolParameter, EvidenceGatewayPolicy,
    InvestigationBudget
};

pub fn get_tool_definitions() -> Vec<ToolDefinition> {
    vec![
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

#[allow(dead_code)]
pub fn validate_tool_call(
    tool_name: &str,
    policy: &EvidenceGatewayPolicy,
    budget: &InvestigationBudget,
    current_calls: i64,
) -> Result<(), String> {
    if current_calls >= budget.max_tool_calls {
        return Err(format!("Tool call budget exceeded ({}/{})", current_calls, budget.max_tool_calls));
    }
    
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
                ToolRiskLevel::Expensive => Ok(()),
                ToolRiskLevel::Dangerous => Err("Dangerous tools not allowed".to_string()),
            }
        }
        None => Err(format!("Unknown tool: {}", tool_name)),
    }
}

/// Get case statistics
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

/// Search emails
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

/// Get email by ID
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

/// Get authentication results
#[tauri::command]
pub async fn ai_get_authentication_results(state: State<'_, AppState>, email_id: String) -> Result<Option<AuthResults>, String> {
    let db = state.db.lock().await;
    
    let headers_raw: Option<String> = db.conn.query_row(
        "SELECT headers_raw FROM emails WHERE id = ?1",
        [&email_id],
        |row| row.get(0)
    ).ok();
    
    if let Some(headers) = headers_raw {
        let mut auth = AuthResults {
            email_id: email_id.clone(),
            spf_result: None,
            dkim_result: None,
            dmarc_result: None,
            arc_result: None,
            received_chain: Vec::new(),
            originating_ip: None,
        };
        
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

/// Get entity by ID or email
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

