use tauri::State;
use crate::AppState;
use super::types::{TimelineEvent, FindingData};

/// Get timeline events
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

/// Get findings
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

/// Get case context for AI
#[tauri::command]
pub async fn ai_get_case_context(state: State<'_, AppState>, case_id: String) -> Result<serde_json::Value, String> {
    let db = state.db.lock().await;
    
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
