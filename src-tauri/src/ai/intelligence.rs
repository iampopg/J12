use std::collections::HashMap;
use tauri::State;
use crate::AppState;
use super::types::{
    EntityData, EntityResolution, EntityCandidate, AnomalyDetection, EmailAnomaly,
    EmailResult
};

// === ENGINE 8: ENTITY RESOLUTION ===

pub fn resolve_entities(entities: &[EntityData]) -> EntityResolution {
    let mut candidates = Vec::new();
    
    let mut name_groups: HashMap<String, Vec<usize>> = HashMap::new();
    for (i, entity) in entities.iter().enumerate() {
        if let Some(name) = &entity.display_name {
            let normalized = name.to_lowercase().trim().to_string();
            if !normalized.is_empty() {
                name_groups.entry(normalized).or_insert_with(Vec::new).push(i);
            }
        }
    }
    
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
        
        let confidence = calculate_merge_confidence(&email_addresses, &display_names);
        
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
    
    candidates.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());
    
    EntityResolution {
        candidates,
        total_entities: entities.len(),
    }
}

fn calculate_merge_confidence(emails: &[String], names: &[String]) -> f64 {
    let mut confidence = 0.0;
    
    if names.len() >= 2 {
        let unique_names: std::collections::HashSet<_> = names.iter().collect();
        if unique_names.len() == 1 {
            confidence += 0.5;
        } else if unique_names.len() <= names.len() / 2 {
            confidence += 0.3;
        }
    }
    
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

pub fn detect_anomalies(
    emails: &[EmailResult],
    entities: &[EntityData],
) -> AnomalyDetection {
    let mut anomalies = Vec::new();
    let start = std::time::Instant::now();
    
    let entity_map: HashMap<&str, &EntityData> = entities
        .iter()
        .map(|e| (e.email_address.as_str(), e))
        .collect();
    
    for email in emails {
        if email.risk_score >= 75 {
            anomalies.push(EmailAnomaly {
                email_id: email.id.clone(),
                anomaly_type: "high_risk_score".to_string(),
                description: format!("Email has high risk score: {}", email.risk_score),
                severity: "high".to_string(),
                confidence: (email.risk_score as f64) / 100.0,
            });
        }
        
        if !entity_map.contains_key(email.from_addr.as_str()) {
            anomalies.push(EmailAnomaly {
                email_id: email.id.clone(),
                anomaly_type: "unknown_sender".to_string(),
                description: format!("Sender {} not in known entity list", email.from_addr),
                severity: "medium".to_string(),
                confidence: 0.7,
            });
        }
        
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

#[tauri::command]
pub async fn ai_detect_anomalies(state: State<'_, AppState>, case_id: String, limit: Option<i64>) -> Result<AnomalyDetection, String> {
    let db = state.db.lock().await;
    
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
