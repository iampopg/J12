use chrono::Utc;
use serde_json::Value;
use tauri::State;

use crate::AppState;
use crate::db::{generate_id, parse_dt};
use crate::models::*;

#[tauri::command]
pub async fn case_create(state: State<'_, AppState>, input: CaseCreateInput) -> Result<Case, String> {
    let db = state.db.lock().await;
    let now = Utc::now();
    let id = generate_id();
    let cn = input.case_number.clone().unwrap_or_else(|| format!("CASE-{}", &id[..8]));
    let desc = input.description.clone().unwrap_or_default();
    let target_email = input.target_email.clone().unwrap_or_default();
    let target_name = input.target_name.clone().unwrap_or_default();
    let target_org = input.target_organization.clone().unwrap_or_default();
    let inv_type = input.investigation_type.clone().unwrap_or_else(|| "general".to_string());
    let owner_id = input.owner_id.clone().unwrap_or_else(|| "admin".to_string());
    let working_dir = input.working_dir.clone().unwrap_or_else(|| {
        if let Some(doc_dir) = dirs::document_dir() {
            doc_dir.join("J12_Cases").join(&cn).to_string_lossy().to_string()
        } else {
            format!("./cases/{}", cn)
        }
    });

    if !working_dir.is_empty() {
        let p = std::path::Path::new(&working_dir);
        let _ = std::fs::create_dir_all(p);
        let _ = std::fs::create_dir_all(p.join("evidence"));
        let _ = std::fs::create_dir_all(p.join("attachments"));
        let _ = std::fs::create_dir_all(p.join("exports"));
        let _ = std::fs::create_dir_all(p.join("reports"));
    }
    
    db.conn.execute(
        "INSERT INTO cases (id,title,case_number,investigation_type,description,status,target_email,target_name,target_organization,working_dir,created_at,updated_at,owner_id)
         VALUES (?1,?2,?3,?4,?5,'open',?6,?7,?8,?9,?10,?10,?11)",
        rusqlite::params![id, input.title, cn, inv_type, desc, target_email, target_name, target_org, working_dir, now.to_rfc3339(), owner_id],
    ).map_err(|e| e.to_string())?;

    let custody_id = generate_id();
    let _ = db.conn.execute(
        "INSERT INTO chain_of_custody (id, case_id, evidence_id, action, performed_by, timestamp, notes)
         VALUES (?1, ?2, NULL, 'case_created', 'Examiner', ?3, ?4)",
        rusqlite::params![custody_id, id, now.to_rfc3339(), format!("Case '{}' created with working directory '{}'", input.title, working_dir)],
    );

    crate::audit_logger::log_forensic_event(
        &id,
        "CASE_LIFECYCLE",
        "CASE_CREATED",
        "Examiner",
        None,
        None,
        &format!("Created forensic case \"{}\" [ID: {}] Case Number: {}", input.title, id, cn)
    );

    Ok(Case {
        id,
        title: input.title,
        case_number: cn,
        description: desc,
        status: "open".to_string(),
        owner_id,
        target_email: input.target_email,
        target_name: input.target_name,
        target_organization: input.target_organization,
        investigation_type: inv_type,
        working_dir: Some(working_dir),
        created_at: now,
        updated_at: now,
    })
}

#[tauri::command]
pub async fn case_list(state: State<'_, AppState>) -> Result<Vec<Case>, String> {
    let db = state.db.lock().await;
    let mut stmt = db.conn.prepare("SELECT id,title,case_number,description,status,target_email,target_name,target_organization,investigation_type,working_dir,created_at,updated_at,owner_id FROM cases ORDER BY created_at DESC").map_err(|e| e.to_string())?;
    let cases = stmt.query_map([], |row| {
        Ok(Case {
            id: row.get(0)?, 
            title: row.get(1)?, 
            case_number: row.get(2)?, 
            description: row.get(3)?, 
            status: row.get(4)?,
            target_email: row.get(5)?, 
            target_name: row.get(6)?, 
            target_organization: row.get(7)?,
            investigation_type: row.get(8)?,
            working_dir: row.get(9)?,
            created_at: parse_dt(row.get::<_,String>(10)?.as_str()),
            updated_at: parse_dt(row.get::<_,String>(11)?.as_str()),
            owner_id: row.get::<_, Option<String>>(12)?.unwrap_or_else(|| "admin".to_string()),
        })
    }).map_err(|e| e.to_string())?.collect::<Result<Vec<_>,_>>().map_err(|e| e.to_string())?;
    Ok(cases)
}

#[tauri::command]
pub async fn case_get(state: State<'_, AppState>, input: EmptyInput) -> Result<Option<Case>, String> {
    let db = state.db.lock().await;
    let r = db.conn.query_row(
        "SELECT id,title,case_number,description,status,target_email,target_name,target_organization,investigation_type,working_dir,created_at,updated_at,owner_id FROM cases WHERE id=?1",
        [&input.case_id],
        |row| Ok(Case { 
            id: row.get(0)?, 
            title: row.get(1)?, 
            case_number: row.get(2)?, 
            description: row.get(3)?, 
            status: row.get(4)?, 
            target_email: row.get(5)?, 
            target_name: row.get(6)?, 
            target_organization: row.get(7)?, 
            investigation_type: row.get(8)?, 
            working_dir: row.get(9)?,
            created_at: parse_dt(row.get::<_,String>(10)?.as_str()), 
            updated_at: parse_dt(row.get::<_,String>(11)?.as_str()),
            owner_id: row.get::<_, Option<String>>(12)?.unwrap_or_else(|| "admin".to_string()),
        })
    );
    match r {
        Ok(c) => Ok(Some(c)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
pub async fn case_update(state: State<'_, AppState>, input: CaseUpdateInput) -> Result<(), String> {
    let db = state.db.lock().await;
    let now = Utc::now().to_rfc3339();
    let title = if input.title.trim().is_empty() { None } else { Some(input.title) };

    db.conn.execute(
        "UPDATE cases SET 
            title = COALESCE(?1, title),
            description = COALESCE(?2, description),
            status = COALESCE(?3, status),
            target_email = COALESCE(?4, target_email),
            target_name = COALESCE(?5, target_name),
            target_organization = COALESCE(?6, target_organization),
            owner_id = COALESCE(?7, owner_id),
            updated_at = ?8
         WHERE id = ?9",
        rusqlite::params![
            title,
            input.description,
            input.status,
            input.target_email,
            input.target_name,
            input.target_organization,
            input.owner_id,
            now,
            input.case_id
        ],
    ).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn case_delete(state: State<'_, AppState>, input: Value) -> Result<bool, String> {
    let case_id = input["case_id"].as_str()
        .or_else(|| input["caseId"].as_str())
        .or_else(|| input["input"]["case_id"].as_str())
        .or_else(|| input["input"]["caseId"].as_str())
        .or_else(|| input.as_str())
        .unwrap_or("")
        .to_string();

    if case_id.is_empty() {
        return Err("Case ID is required for deletion".to_string());
    }

    crate::audit_logger::log_forensic_event(
        &case_id,
        "CASE_DELETION",
        "CASE_DESTROYED",
        "Examiner",
        None,
        None,
        &format!("Case {} and associated evidence, artifacts, findings, and AI context cascade deleted by examiner", case_id),
    );

    let mut db = state.db.lock().await;

    let _ = db.conn.execute(
        "INSERT INTO audit_log (id, actor, action, target_type, target_id, timestamp, detail)
         VALUES (?1, 'Examiner', 'case_deleted', 'case', ?2, ?3, ?4)",
        rusqlite::params![
            generate_id(),
            &case_id,
            Utc::now().to_rfc3339(),
            format!("Case {} deleted", case_id)
        ],
    );

    let tx = db.conn.transaction().map_err(|e| e.to_string())?;

    let _ = tx.execute("DELETE FROM ai_messages WHERE session_id IN (SELECT id FROM ai_sessions WHERE case_id = ?1)", [&case_id]);
    let _ = tx.execute("DELETE FROM ai_tool_calls WHERE session_id IN (SELECT id FROM ai_sessions WHERE case_id = ?1)", [&case_id]);
    let _ = tx.execute("DELETE FROM ai_audit_log WHERE session_id IN (SELECT id FROM ai_sessions WHERE case_id = ?1)", [&case_id]);
    let _ = tx.execute("DELETE FROM ai_context_snapshots WHERE case_id = ?1", [&case_id]);
    let _ = tx.execute("DELETE FROM ai_search_index WHERE case_id = ?1", [&case_id]);
    let _ = tx.execute("DELETE FROM ai_entity_resolutions WHERE case_id = ?1", [&case_id]);
    let _ = tx.execute("DELETE FROM ai_investigation_plans WHERE case_id = ?1", [&case_id]);
    let _ = tx.execute("DELETE FROM ai_sessions WHERE case_id = ?1", [&case_id]);

    let _ = tx.execute("DELETE FROM item_bookmarks WHERE case_id = ?1", [&case_id]);
    let _ = tx.execute("DELETE FROM email_tags WHERE case_id = ?1 OR email_id IN (SELECT id FROM emails WHERE case_id = ?1 OR evidence_id IN (SELECT id FROM evidence_items WHERE case_id = ?1))", [&case_id]);
    let _ = tx.execute("DELETE FROM email_notes WHERE case_id = ?1 OR email_id IN (SELECT id FROM emails WHERE case_id = ?1 OR evidence_id IN (SELECT id FROM evidence_items WHERE case_id = ?1))", [&case_id]);
    let _ = tx.execute("DELETE FROM attachments WHERE email_id IN (SELECT id FROM emails WHERE case_id = ?1 OR evidence_id IN (SELECT id FROM evidence_items WHERE case_id = ?1))", [&case_id]);
    let _ = tx.execute("DELETE FROM forensic_artifacts WHERE case_id = ?1 OR email_id IN (SELECT id FROM emails WHERE case_id = ?1 OR evidence_id IN (SELECT id FROM evidence_items WHERE case_id = ?1))", [&case_id]);
    let _ = tx.execute("DELETE FROM artifacts_cache WHERE case_id = ?1", [&case_id]);
    let _ = tx.execute("DELETE FROM communication_edges WHERE case_id = ?1", [&case_id]);
    let _ = tx.execute("DELETE FROM timeline_events WHERE case_id = ?1 OR evidence_id IN (SELECT id FROM evidence_items WHERE case_id = ?1)", [&case_id]);
    let _ = tx.execute("DELETE FROM entities WHERE case_id = ?1", [&case_id]);
    let _ = tx.execute("DELETE FROM findings WHERE case_id = ?1", [&case_id]);
    let _ = tx.execute("DELETE FROM custody_events WHERE evidence_id IN (SELECT id FROM evidence_items WHERE case_id = ?1)", [&case_id]);
    let _ = tx.execute("DELETE FROM chain_of_custody WHERE case_id = ?1 OR evidence_id IN (SELECT id FROM evidence_items WHERE case_id = ?1)", [&case_id]);
    let _ = tx.execute("DELETE FROM case_notes WHERE case_id = ?1", [&case_id]);
    let _ = tx.execute("DELETE FROM emails WHERE case_id = ?1 OR evidence_id IN (SELECT id FROM evidence_items WHERE case_id = ?1)", [&case_id]);
    let _ = tx.execute("DELETE FROM evidence_items WHERE case_id = ?1", [&case_id]);
    let r = tx.execute("DELETE FROM cases WHERE id = ?1", [&case_id]).map_err(|e| e.to_string())?;

    tx.commit().map_err(|e| e.to_string())?;

    Ok(r > 0)
}

#[tauri::command]
pub async fn open_external_url(url: String) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    let _ = std::process::Command::new("open").arg(&url).spawn();

    #[cfg(target_os = "windows")]
    let _ = std::process::Command::new("cmd").args(["/C", "start", &url]).spawn();

    #[cfg(target_os = "linux")]
    let _ = std::process::Command::new("xdg-open").arg(&url).spawn();

    Ok(())
}
