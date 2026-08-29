use std::fs;
use std::path::PathBuf;
use chrono::Utc;
use tauri::State;

use crate::AppState;
use crate::db::parse_dt;
use crate::models::*;

#[tauri::command]
pub async fn export_audit_log(state: State<'_, AppState>, input: EmptyInput) -> Result<String, String> {
    let db = state.db.lock().await;
    let mut stmt = db.conn.prepare(
        "SELECT id, evidence_id, action, performed_by, timestamp, notes 
         FROM chain_of_custody WHERE case_id = ?1 ORDER BY timestamp ASC"
    ).map_err(|e| e.to_string())?;

    let events = stmt.query_map([&input.case_id], |row| {
        Ok(CustodyEvent {
            id: row.get(0)?,
            evidence_id: row.get::<_, Option<String>>(1)?.unwrap_or_default(),
            action: row.get(2)?,
            actor: row.get::<_, Option<String>>(3)?.unwrap_or_else(|| "Examiner".to_string()),
            timestamp: parse_dt(&row.get::<_, String>(4)?),
            tool: "J12 Email Forensic Suite".to_string(),
            tool_version: "1.0.0".to_string(),
            hash_before: None,
            hash_after: None,
            detail: row.get(5)?,
        })
    }).map_err(|e| e.to_string())?.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())?;

    let mut csv = String::from("id,evidence_id,action,actor,timestamp,detail\n");
    for ev in events {
        csv.push_str(&format!(
            "\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\"\n",
            ev.id,
            ev.evidence_id,
            ev.action,
            ev.actor,
            ev.timestamp.to_rfc3339(),
            ev.detail.unwrap_or_default().replace('"', "\"\""),
        ));
    }

    let downloads_dir = dirs::download_dir().unwrap_or_else(|| PathBuf::from("."));
    let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
    let safe_len = input.case_id.len().min(8);
    let filename = format!("audit_log_{}_{}.csv", &input.case_id[..safe_len], timestamp);
    let output_path = downloads_dir.join(&filename);

    fs::write(&output_path, csv).map_err(|e| format!("Failed to write audit CSV: {}", e))?;

    crate::audit_logger::log_forensic_event(
        &input.case_id,
        "REPORTING",
        "AUDIT_LOG_EXPORTED",
        "Examiner",
        None,
        None,
        &format!("Exported CSV custody chain log to \"{}\"", output_path.display())
    );

    Ok(output_path.to_string_lossy().to_string())
}

#[tauri::command]
pub async fn check_custody_chain(state: State<'_, AppState>, input: EmptyInput) -> Result<serde_json::Value, String> {
    let db = state.db.lock().await;
    let count: i64 = db.conn.query_row(
        "SELECT (
            (SELECT COUNT(*) FROM chain_of_custody WHERE case_id = ?1) +
            (SELECT COUNT(*) FROM custody_events ce JOIN evidence_items ei ON ce.evidence_id = ei.id WHERE ei.case_id = ?1)
         )",
        [&input.case_id],
        |r| r.get(0)
    ).unwrap_or(0);

    Ok(serde_json::json!({
        "case_id": input.case_id,
        "events_count": count,
        "is_valid": count > 0,
        "verified_at": Utc::now().to_rfc3339()
    }))
}
