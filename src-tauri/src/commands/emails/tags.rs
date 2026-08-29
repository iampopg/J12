use chrono::Utc;
use serde_json::Value;
use tauri::State;

use crate::AppState;
use crate::db::generate_id;
use crate::models::EmailTag;

#[tauri::command]
pub async fn email_tags_list(
    state: State<'_, AppState>,
    input: Option<Value>,
    case_id: Option<String>,
    email_id: Option<String>,
) -> Result<Vec<EmailTag>, String> {
    let case_id_val = case_id
        .or_else(|| input.as_ref().and_then(|v| v["case_id"].as_str().or_else(|| v["caseId"].as_str()).or_else(|| v["input"]["case_id"].as_str()).or_else(|| v["input"]["caseId"].as_str()).map(|s| s.to_string())))
        .unwrap_or_default();

    let email_id_val = email_id
        .or_else(|| input.as_ref().and_then(|v| v["email_id"].as_str().or_else(|| v["emailId"].as_str()).or_else(|| v["input"]["email_id"].as_str()).or_else(|| v["input"]["emailId"].as_str()).map(|s| s.to_string())))
        .unwrap_or_default();

    let db = state.db.lock().await;

    if !email_id_val.is_empty() {
        let mut stmt = db.conn.prepare(
            "SELECT id, case_id, email_id, tag, color, created_at
             FROM email_tags
             WHERE email_id = ?1
             ORDER BY created_at DESC"
        ).map_err(|e| e.to_string())?;

        let tags = stmt.query_map([&email_id_val], |row| {
            Ok(EmailTag {
                id: row.get(0)?,
                case_id: row.get(1)?,
                email_id: row.get(2)?,
                tag: row.get(3)?,
                color: row.get::<_, Option<String>>(4)?.unwrap_or_else(|| "#3b82f6".to_string()),
                created_by: "Examiner".to_string(),
                created_at: row.get::<_, String>(5)?,
            })
        }).map_err(|e| e.to_string())?.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())?;

        return Ok(tags);
    }

    if !case_id_val.is_empty() {
        let mut stmt = db.conn.prepare(
            "SELECT id, case_id, email_id, tag, color, created_at
             FROM email_tags
             WHERE case_id = ?1 OR email_id IN (SELECT id FROM emails WHERE case_id = ?1)
             ORDER BY created_at DESC"
        ).map_err(|e| e.to_string())?;

        let tags = stmt.query_map([&case_id_val], |row| {
            Ok(EmailTag {
                id: row.get(0)?,
                case_id: row.get(1)?,
                email_id: row.get(2)?,
                tag: row.get(3)?,
                color: row.get::<_, Option<String>>(4)?.unwrap_or_else(|| "#3b82f6".to_string()),
                created_by: "Examiner".to_string(),
                created_at: row.get::<_, String>(5)?,
            })
        }).map_err(|e| e.to_string())?.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())?;

        return Ok(tags);
    }

    let mut stmt = db.conn.prepare(
        "SELECT id, case_id, email_id, tag, color, created_at
         FROM email_tags
         ORDER BY created_at DESC"
    ).map_err(|e| e.to_string())?;

    let tags = stmt.query_map([], |row| {
        Ok(EmailTag {
            id: row.get(0)?,
            case_id: row.get(1)?,
            email_id: row.get(2)?,
            tag: row.get(3)?,
            color: row.get::<_, Option<String>>(4)?.unwrap_or_else(|| "#3b82f6".to_string()),
            created_by: "Examiner".to_string(),
            created_at: row.get::<_, String>(5)?,
        })
    }).map_err(|e| e.to_string())?.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())?;

    Ok(tags)
}

#[tauri::command]
pub async fn email_tag_add(state: State<'_, AppState>, input: Value) -> Result<EmailTag, String> {
    let case_id = input["case_id"].as_str()
        .or_else(|| input["caseId"].as_str())
        .or_else(|| input["input"]["case_id"].as_str())
        .or_else(|| input["input"]["caseId"].as_str())
        .unwrap_or("")
        .to_string();

    let email_id = input["email_id"].as_str()
        .or_else(|| input["emailId"].as_str())
        .or_else(|| input["input"]["email_id"].as_str())
        .or_else(|| input["input"]["emailId"].as_str())
        .unwrap_or("")
        .to_string();

    let tag = input["tag"].as_str()
        .or_else(|| input["input"]["tag"].as_str())
        .unwrap_or("")
        .to_string();

    let color = input["color"].as_str()
        .or_else(|| input["input"]["color"].as_str())
        .unwrap_or("#3b82f6")
        .to_string();

    let created_by = input["created_by"].as_str()
        .or_else(|| input["createdBy"].as_str())
        .or_else(|| input["input"]["created_by"].as_str())
        .or_else(|| input["input"]["createdBy"].as_str())
        .unwrap_or("Investigator")
        .to_string();

    if email_id.is_empty() || tag.is_empty() {
        return Err("email_id and tag are required".to_string());
    }

    let db = state.db.lock().await;
    let id = generate_id();
    let now = Utc::now().to_rfc3339();

    db.conn.execute(
        "INSERT OR REPLACE INTO email_tags (id, case_id, email_id, tag, color, created_by, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        rusqlite::params![id, case_id, email_id, tag, color, created_by, now],
    ).map_err(|e| e.to_string())?;

    crate::audit_logger::log_forensic_event(
        &case_id,
        "TAGGING",
        "TAG_ATTACHED",
        &created_by,
        None,
        None,
        &format!("Attached tag [{}] to email [{}]", tag, email_id)
    );

    Ok(EmailTag {
        id,
        case_id,
        email_id,
        tag,
        color,
        created_by,
        created_at: now,
    })
}

#[tauri::command]
pub async fn email_tag_remove(state: State<'_, AppState>, input: Value) -> Result<(), String> {
    let case_id = input["case_id"].as_str()
        .or_else(|| input["caseId"].as_str())
        .or_else(|| input["input"]["case_id"].as_str())
        .or_else(|| input["input"]["caseId"].as_str())
        .unwrap_or("")
        .to_string();

    let email_id = input["email_id"].as_str()
        .or_else(|| input["emailId"].as_str())
        .or_else(|| input["input"]["email_id"].as_str())
        .or_else(|| input["input"]["emailId"].as_str())
        .unwrap_or("")
        .to_string();

    let tag = input["tag"].as_str()
        .or_else(|| input["input"]["tag"].as_str())
        .unwrap_or("")
        .to_string();

    let db = state.db.lock().await;
    db.conn.execute(
        "DELETE FROM email_tags WHERE email_id = ?1 AND tag = ?2",
        rusqlite::params![email_id, tag],
    ).map_err(|e| e.to_string())?;

    if !case_id.is_empty() {
        crate::audit_logger::log_forensic_event(
            &case_id,
            "TAGGING",
            "TAG_REMOVED",
            "Examiner",
            None,
            None,
            &format!("Removed tag [{}] from email [{}]", tag, email_id)
        );
    }
    Ok(())
}
