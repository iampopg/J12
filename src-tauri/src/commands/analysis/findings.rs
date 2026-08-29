use chrono::Utc;
use serde_json::Value;
use tauri::State;

use crate::AppState;
use crate::db::parse_dt;
use crate::models::*;
use super::super::helpers::*;

#[tauri::command]
pub async fn findings_list(state: State<'_, AppState>, input: Value) -> Result<Vec<Finding>, String> {
    let case_id = input["case_id"].as_str()
        .or_else(|| input["caseId"].as_str())
        .or_else(|| input["input"]["case_id"].as_str())
        .or_else(|| input["input"]["caseId"].as_str())
        .or_else(|| input.as_str())
        .unwrap_or("")
        .to_string();

    let evidence_id = input["evidence_id"].as_str()
        .or_else(|| input["evidenceId"].as_str())
        .or_else(|| input["input"]["evidence_id"].as_str())
        .or_else(|| input["input"]["evidenceId"].as_str())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty() && s != "all");

    let db = state.db.lock().await;

    let findings = if let Some(ref ev_id) = evidence_id {
        let mut stmt = db.conn.prepare(
            "SELECT id,case_id,type,severity,confidence,title,description,evidence_refs,email_ids,status,created_at,reviewed_by,reviewed_at,notes 
             FROM findings 
             WHERE case_id=?1 
               AND (
                 evidence_refs LIKE '%' || ?2 || '%' 
                 OR EXISTS (
                   SELECT 1 FROM emails e 
                   WHERE e.case_id=?1 AND e.evidence_id=?2 
                     AND instr(findings.email_ids, e.id) > 0
                 )
               )
             ORDER BY 
               CASE severity 
                 WHEN 'critical' THEN 1 
                 WHEN 'high' THEN 2 
                 WHEN 'medium' THEN 3 
                 WHEN 'low' THEN 4 
                 ELSE 5 
               END, 
               created_at DESC"
        ).map_err(|e| e.to_string())?;

        let res: Vec<Finding> = stmt.query_map(rusqlite::params![&case_id, ev_id], |row| {
            Ok(Finding { 
                id: row.get(0)?, 
                case_id: row.get(1)?, 
                type_: row.get(2)?, 
                severity: row.get(3)?, 
                confidence: row.get::<_, Option<String>>(4)?.unwrap_or_else(|| "0.85".to_string()), 
                title: row.get(5)?, 
                description: row.get(6)?, 
                evidence_refs: row.get(7)?, 
                email_ids: row.get(8)?, 
                status: row.get(9)?, 
                created_at: parse_dt(row.get::<_, String>(10)?.as_str()), 
                reviewed_by: row.get(11)?, 
                reviewed_at: row.get::<_, Option<String>>(12)?.map(|s| parse_dt(&s)), 
                notes: row.get(13)? 
            })
        }).map_err(|e| e.to_string())?.filter_map(|r| r.ok()).collect();
        res
    } else {
        let mut stmt = db.conn.prepare(
            "SELECT id,case_id,type,severity,confidence,title,description,evidence_refs,email_ids,status,created_at,reviewed_by,reviewed_at,notes 
             FROM findings WHERE case_id=?1 
             ORDER BY 
               CASE severity 
                 WHEN 'critical' THEN 1 
                 WHEN 'high' THEN 2 
                 WHEN 'medium' THEN 3 
                 WHEN 'low' THEN 4 
                 ELSE 5 
               END, 
               created_at DESC"
        ).map_err(|e| e.to_string())?;

        let res: Vec<Finding> = stmt.query_map([&case_id], |row| {
            Ok(Finding { 
                id: row.get(0)?, 
                case_id: row.get(1)?, 
                type_: row.get(2)?, 
                severity: row.get(3)?, 
                confidence: row.get::<_, Option<String>>(4)?.unwrap_or_else(|| "0.85".to_string()), 
                title: row.get(5)?, 
                description: row.get(6)?, 
                evidence_refs: row.get(7)?, 
                email_ids: row.get(8)?, 
                status: row.get(9)?, 
                created_at: parse_dt(row.get::<_, String>(10)?.as_str()), 
                reviewed_by: row.get(11)?, 
                reviewed_at: row.get::<_, Option<String>>(12)?.map(|s| parse_dt(&s)), 
                notes: row.get(13)? 
            })
        }).map_err(|e| e.to_string())?.filter_map(|r| r.ok()).collect();
        res
    };

    Ok(findings)
}

#[tauri::command]
pub async fn update_finding_status(
    state: State<'_, AppState>,
    input: Value,
) -> Result<(), String> {
    let finding_id = input["finding_id"].as_str()
        .or_else(|| input["findingId"].as_str())
        .or_else(|| input["id"].as_str())
        .or_else(|| input["input"]["finding_id"].as_str())
        .or_else(|| input["input"]["findingId"].as_str())
        .or_else(|| input["input"]["id"].as_str())
        .unwrap_or("")
        .to_string();

    let status = input["new_status"].as_str()
        .or_else(|| input["newStatus"].as_str())
        .or_else(|| input["status"].as_str())
        .or_else(|| input["input"]["new_status"].as_str())
        .or_else(|| input["input"]["newStatus"].as_str())
        .or_else(|| input["input"]["status"].as_str())
        .unwrap_or("")
        .to_string();

    let reviewed_by = input["reviewed_by"].as_str()
        .or_else(|| input["reviewedBy"].as_str())
        .or_else(|| input["author"].as_str())
        .or_else(|| input["input"]["reviewed_by"].as_str())
        .or_else(|| input["input"]["reviewedBy"].as_str())
        .or_else(|| input["input"]["author"].as_str())
        .unwrap_or("Investigator")
        .to_string();

    if finding_id.is_empty() {
        return Err("finding_id is required".to_string());
    }

    let status_lower = status.to_lowercase();
    let normalized_status = match status_lower.as_str() {
        "open" | "new" => "new",
        "reviewed" => "reviewed",
        "confirmed" => "confirmed",
        "rejected" | "false_positive" => "false_positive",
        "dismissed" => "dismissed",
        _ => "reviewed",
    };

    let db = state.db.lock().await;
    let now = Utc::now().to_rfc3339();

    db.conn.execute(
        "UPDATE findings SET status = ?1, reviewed_by = ?2, reviewed_at = ?3 WHERE id = ?4",
        rusqlite::params![normalized_status, reviewed_by, now, finding_id],
    ).map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn add_finding_note(
    state: State<'_, AppState>,
    input: Value,
) -> Result<(), String> {
    let finding_id = input["finding_id"].as_str()
        .or_else(|| input["findingId"].as_str())
        .or_else(|| input["id"].as_str())
        .or_else(|| input["input"]["finding_id"].as_str())
        .or_else(|| input["input"]["findingId"].as_str())
        .or_else(|| input["input"]["id"].as_str())
        .unwrap_or("")
        .to_string();

    let note = input["note"].as_str()
        .or_else(|| input["content"].as_str())
        .or_else(|| input["input"]["note"].as_str())
        .or_else(|| input["input"]["content"].as_str())
        .unwrap_or("")
        .to_string();

    let author = input["author"].as_str()
        .or_else(|| input["created_by"].as_str())
        .or_else(|| input["createdBy"].as_str())
        .or_else(|| input["input"]["author"].as_str())
        .or_else(|| input["input"]["created_by"].as_str())
        .unwrap_or("Investigator")
        .to_string();

    if finding_id.is_empty() {
        return Err("finding_id is required".to_string());
    }
    if note.is_empty() {
        return Err("note content is required".to_string());
    }

    let db = state.db.lock().await;
    let existing: Option<String> = db.conn.query_row(
        "SELECT notes FROM findings WHERE id = ?1",
        [&finding_id],
        |row| row.get(0),
    ).map_err(|e| e.to_string())?;

    let timestamp = Utc::now().to_rfc3339();
    let new_note_entry = format!("[{} by {}]: {}", timestamp, author, note);
    let updated_notes = match existing {
        Some(prev) if !prev.is_empty() => format!("{}\n{}", prev, new_note_entry),
        _ => new_note_entry,
    };

    db.conn.execute(
        "UPDATE findings SET notes = ?1 WHERE id = ?2",
        rusqlite::params![updated_notes, finding_id],
    ).map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn finding_emails(state: State<'_, AppState>, input: Value) -> Result<Vec<EmailMessage>, String> {
    let finding_id = input["finding_id"].as_str()
        .or_else(|| input["findingId"].as_str())
        .or_else(|| input["id"].as_str())
        .or_else(|| input["input"]["finding_id"].as_str())
        .or_else(|| input["input"]["findingId"].as_str())
        .or_else(|| input["input"]["id"].as_str())
        .or_else(|| input.as_str())
        .unwrap_or("")
        .to_string();

    let db = state.db.lock().await;

    let email_ids_str: Option<String> = db.conn.query_row(
        "SELECT email_ids FROM findings WHERE id = ?1",
        [&finding_id],
        |row| row.get(0),
    ).map_err(|e| e.to_string())?;

    let email_ids: Vec<String> = match email_ids_str {
        Some(s) => serde_json::from_str(&s).unwrap_or_default(),
        None => Vec::new(),
    };

    if email_ids.is_empty() {
        return Ok(Vec::new());
    }

    let placeholders: Vec<String> = (1..=email_ids.len()).map(|i| format!("?{}", i)).collect();
    let sql = format!(
        "SELECT id, evidence_id, case_id, message_id, from_addr, from_display, to_addrs, cc_addrs, subject,
                date_sent, date_sent_utc, headers_raw, body_text, body_html, folder_name, folder_category,
                is_deleted, deleted_recovered, risk_score, flags
         FROM emails WHERE id IN ({}) ORDER BY date_sent_utc DESC",
        placeholders.join(",")
    );

    let mut stmt = db.conn.prepare(&sql).map_err(|e| e.to_string())?;
    let params: Vec<&dyn rusqlite::ToSql> = email_ids.iter().map(|id| id as &dyn rusqlite::ToSql).collect();

    let emails = stmt.query_map(params.as_slice(), |row| {
        Ok(EmailMessage {
            id: row.get(0)?, evidence_id: row.get(1)?, case_id: row.get(2)?, message_id: row.get(3)?,
            from_addr: row.get(4)?, from_display: row.get(5)?, to_addrs: row.get(6)?, cc_addrs: row.get(7)?,
            subject: row.get(8)?, date_sent: row.get(9)?, date_sent_utc: row.get(10)?, headers_raw: row.get(11)?,
            body_text: row.get(12)?, body_html: row.get(13)?, folder_name: row.get(14)?, folder_category: row.get(15)?,
            is_deleted: boolv(row, 16), deleted_recovered: boolv(row, 17), risk_score: u8v(row, 18), flags: row.get(19)?
        })
    }).map_err(|e| e.to_string())?.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())?;

    Ok(emails)
}
