use chrono::Utc;
use serde_json::Value;
use tauri::State;

use crate::AppState;
use crate::db::{generate_id, parse_dt};
use crate::models::*;
use super::helpers::*;

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
    
    db.conn.execute(
        "INSERT INTO cases (id,name,case_number,examiner_name,investigation_type,description,status,target_email,target_name,target_organization,created_at,updated_at)
         VALUES (?1,?2,?3,'Examiner',?4,?5,'active',?6,?7,?8,?9,?9)",
        rusqlite::params![id, input.title, cn, inv_type, desc, target_email, target_name, target_org, now.to_rfc3339()],
    ).map_err(|e| e.to_string())?;

    // Create initial custody log event
    let custody_id = generate_id();
    let _ = db.conn.execute(
        "INSERT INTO chain_of_custody (id, case_id, evidence_id, action, performed_by, timestamp, notes)
         VALUES (?1, ?2, NULL, 'case_created', 'Examiner', ?3, ?4)",
        rusqlite::params![custody_id, id, now.to_rfc3339(), format!("Case {} created for target {}", input.title, target_email)],
    );

    Ok(Case {
        id,
        title: input.title,
        case_number: cn,
        description: desc,
        status: "active".to_string(),
        owner_id: "default".to_string(),
        target_email: input.target_email,
        target_name: input.target_name,
        target_organization: input.target_organization,
        investigation_type: inv_type,
        created_at: now,
        updated_at: now,
    })
}

#[tauri::command]
pub async fn case_list(state: State<'_, AppState>) -> Result<Vec<Case>, String> {
    let db = state.db.lock().await;
    let mut stmt = db.conn.prepare("SELECT id,name,case_number,description,status,target_email,target_name,target_organization,investigation_type,created_at,updated_at FROM cases ORDER BY created_at DESC").map_err(|e| e.to_string())?;
    let cases = stmt.query_map([], |row| {
        Ok(Case {
            id: row.get(0)?, 
            title: row.get(1)?, 
            case_number: row.get(2)?, 
            description: row.get(3)?, 
            status: row.get(4)?,
            owner_id: "default".to_string(),
            target_email: row.get(5)?, 
            target_name: row.get(6)?, 
            target_organization: row.get(7)?,
            investigation_type: row.get(8)?,
            created_at: parse_dt(row.get::<_,String>(9)?.as_str()),
            updated_at: parse_dt(row.get::<_,String>(10)?.as_str()),
        })
    }).map_err(|e| e.to_string())?.collect::<Result<Vec<_>,_>>().map_err(|e| e.to_string())?;
    Ok(cases)
}

#[tauri::command]
pub async fn case_get(state: State<'_, AppState>, input: EmptyInput) -> Result<Option<Case>, String> {
    let db = state.db.lock().await;
    let r = db.conn.query_row("SELECT id,name,case_number,description,status,target_email,target_name,target_organization,investigation_type,created_at,updated_at FROM cases WHERE id=?1", [&input.case_id],
        |row| Ok(Case { 
            id: row.get(0)?, 
            title: row.get(1)?, 
            case_number: row.get(2)?, 
            description: row.get(3)?, 
            status: row.get(4)?, 
            owner_id: "default".to_string(),
            target_email: row.get(5)?, 
            target_name: row.get(6)?, 
            target_organization: row.get(7)?, 
            investigation_type: row.get(8)?, 
            created_at: parse_dt(row.get::<_,String>(9)?.as_str()), 
            updated_at: parse_dt(row.get::<_,String>(10)?.as_str()) 
        }));
    match r { Ok(c) => Ok(Some(c)), Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None), Err(e) => Err(e.to_string()) }
}

#[tauri::command]
pub async fn case_update(state: State<'_, AppState>, input: CaseUpdateInput) -> Result<(), String> {
    let db = state.db.lock().await;
    let now = Utc::now().to_rfc3339();
    let mut sets = Vec::new();
    if !input.title.is_empty() { sets.push(format!("name='{}'", input.title.replace('\'',"''"))); }
    if let Some(ref v) = input.description { sets.push(format!("description='{}'", v.replace('\'',"''"))); }
    if let Some(ref v) = input.status { sets.push(format!("status='{}'", v.replace('\'',"''"))); }
    if sets.is_empty() { return Ok(()); }
    sets.push(format!("updated_at='{}'", now));
    let sql = format!("UPDATE cases SET {} WHERE id='{}'", sets.join(","), input.case_id.replace('\'',"''"));
    db.conn.execute(&sql, []).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn case_delete(state: State<'_, AppState>, input: EmptyInput) -> Result<bool, String> {
    let db = state.db.lock().await;
    let cid = &input.case_id;
    let _ = db.conn.execute("DELETE FROM findings WHERE case_id=?1", [cid]);
    let _ = db.conn.execute("DELETE FROM entities WHERE case_id=?1", [cid]);
    let _ = db.conn.execute("DELETE FROM attachments WHERE email_id IN (SELECT id FROM emails WHERE case_id=?1)", [cid]);
    let _ = db.conn.execute("DELETE FROM emails WHERE case_id=?1", [cid]);
    let _ = db.conn.execute("DELETE FROM evidence_items WHERE case_id=?1", [cid]);
    let _ = db.conn.execute("DELETE FROM chain_of_custody WHERE case_id=?1", [cid]);
    let _ = db.conn.execute("DELETE FROM case_notes WHERE case_id=?1", [cid]);
    let _ = db.conn.execute("DELETE FROM email_tags WHERE email_id IN (SELECT id FROM emails WHERE case_id=?1)", [cid]);
    let _ = db.conn.execute("DELETE FROM email_notes WHERE email_id IN (SELECT id FROM emails WHERE case_id=?1)", [cid]);
    let r = db.conn.execute("DELETE FROM cases WHERE id=?1", [cid]).map_err(|e| e.to_string())?;
    Ok(r > 0)
}

#[tauri::command]
pub async fn auto_detect_targets(state: State<'_, AppState>, input: Value) -> Result<Value, String> {
    let case_id = input["case_id"].as_str()
        .or_else(|| input["caseId"].as_str())
        .or_else(|| input.as_str())
        .unwrap_or("")
        .to_string();

    let db = state.db.lock().await;

    let mut stmt = db.conn.prepare(
        "SELECT from_addr, from_display, COUNT(*) as sent_count 
         FROM emails 
         WHERE case_id = ?1 AND from_addr != ''
         GROUP BY from_addr 
         ORDER BY sent_count DESC 
         LIMIT 10"
    ).map_err(|e| e.to_string())?;

    let candidates = stmt.query_map([&case_id], |row| {
        let email: String = row.get(0)?;
        let name: Option<String> = row.get(1)?;
        let count: i64 = row.get(2)?;
        
        let org = email.split('@').nth(1).unwrap_or("").to_string();
        
        Ok(serde_json::json!({
            "email": email,
            "name": name.unwrap_or_else(|| email.split('@').next().unwrap_or("").to_string()),
            "organization": org,
            "sent_count": count,
            "confidence": if count > 20 { "high" } else if count > 5 { "medium" } else { "low" }
        }))
    }).map_err(|e| e.to_string())?.filter_map(|r| r.ok()).collect::<Vec<_>>();

    Ok(serde_json::json!({
        "candidates": candidates
    }))
}

#[tauri::command]
pub async fn target_profile(state: State<'_, AppState>, input: Value) -> Result<Value, String> {
    let case_id = input["case_id"].as_str()
        .or_else(|| input["caseId"].as_str())
        .or_else(|| input.as_str())
        .unwrap_or("")
        .to_string();

    let db = state.db.lock().await;

    let case_row = db.conn.query_row(
        "SELECT target_email, target_name, target_organization FROM cases WHERE id = ?1",
        [&case_id],
        |row| Ok((row.get::<_, Option<String>>(0)?, row.get::<_, Option<String>>(1)?, row.get::<_, Option<String>>(2)?))
    );

    let (mut target_email, mut target_name, mut target_org) = match case_row {
        Ok((e, n, o)) => (e.unwrap_or_default(), n.unwrap_or_default(), o.unwrap_or_default()),
        Err(_) => (String::new(), String::new(), String::new()),
    };

    if target_email.is_empty() {
        let top_sender: Result<(String, Option<String>), _> = db.conn.query_row(
            "SELECT from_addr, from_display FROM emails WHERE case_id = ?1 GROUP BY from_addr ORDER BY COUNT(*) DESC LIMIT 1",
            [&case_id],
            |row| Ok((row.get(0)?, row.get(1)?))
        );

        if let Ok((email, name)) = top_sender {
            target_email = email.clone();
            target_name = name.unwrap_or_else(|| email.split('@').next().unwrap_or("").to_string());
            target_org = email.split('@').nth(1).unwrap_or("").to_string();

            let _ = db.conn.execute(
                "UPDATE cases SET target_email = ?1, target_name = ?2, target_organization = ?3 WHERE id = ?4",
                rusqlite::params![&target_email, &target_name, &target_org, &case_id]
            );
        }
    }

    if target_email.is_empty() {
        return Ok(serde_json::json!({
            "target": null,
            "stats": {},
            "top_contacts": [],
            "threat_breakdown": {}
        }));
    }

    let target_like = format!("%{}%", target_email);
    let sent_count: i64 = db.conn.query_row(
        "SELECT COUNT(*) FROM emails WHERE case_id = ?1 AND from_addr LIKE ?2",
        rusqlite::params![&case_id, &target_like],
        |r| r.get(0)
    ).unwrap_or(0);

    let received_count: i64 = db.conn.query_row(
        "SELECT COUNT(*) FROM emails WHERE case_id = ?1 AND (to_addrs LIKE ?2 OR cc_addrs LIKE ?2)",
        rusqlite::params![&case_id, &target_like],
        |r| r.get(0)
    ).unwrap_or(0);

    let flagged_count: i64 = db.conn.query_row(
        "SELECT COUNT(*) FROM emails WHERE case_id = ?1 AND (from_addr LIKE ?2 OR to_addrs LIKE ?2) AND risk_score > 30",
        rusqlite::params![&case_id, &target_like],
        |r| r.get(0)
    ).unwrap_or(0);

    let attachment_count: i64 = db.conn.query_row(
        "SELECT COUNT(*) FROM attachments a JOIN emails e ON a.email_id = e.id WHERE e.case_id = ?1 AND e.from_addr LIKE ?2",
        rusqlite::params![&case_id, &target_like],
        |r| r.get(0)
    ).unwrap_or(0);

    let mut stmt = db.conn.prepare(
        "SELECT to_addrs, COUNT(*) as contact_count 
         FROM emails 
         WHERE case_id = ?1 AND from_addr LIKE ?2 AND to_addrs != '' 
         GROUP BY to_addrs 
         ORDER BY contact_count DESC 
         LIMIT 6"
    ).map_err(|e| e.to_string())?;

    let top_contacts = stmt.query_map(rusqlite::params![&case_id, &target_like], |row| {
        let addr: String = row.get(0)?;
        let count: i64 = row.get(1)?;
        Ok(serde_json::json!({
            "email": addr,
            "message_count": count
        }))
    }).map_err(|e| e.to_string())?.filter_map(|r| r.ok()).collect::<Vec<_>>();

    let mut threat_stmt = db.conn.prepare(
        "SELECT type, severity, COUNT(*) as count 
         FROM findings 
         WHERE case_id = ?1 
         GROUP BY type, severity 
         ORDER BY count DESC"
    ).map_err(|e| e.to_string())?;

    let threat_summary = threat_stmt.query_map([&case_id], |row| {
        Ok(serde_json::json!({
            "type": row.get::<_, String>(0)?,
            "severity": row.get::<_, String>(1)?,
            "count": row.get::<_, i64>(2)?
        }))
    }).map_err(|e| e.to_string())?.filter_map(|r| r.ok()).collect::<Vec<_>>();

    Ok(serde_json::json!({
        "target": {
            "email": target_email,
            "name": target_name,
            "organization": target_org
        },
        "stats": {
            "sent_count": sent_count,
            "received_count": received_count,
            "flagged_count": flagged_count,
            "attachment_count": attachment_count,
            "total_interactions": sent_count + received_count
        },
        "top_contacts": top_contacts,
        "threat_summary": threat_summary
    }))
}

// Case Notes
#[tauri::command]
pub async fn case_notes_list(state: State<'_, AppState>, input: EmptyInput) -> Result<Vec<CaseNote>, String> {
    let db = state.db.lock().await;
    let mut stmt = db.conn.prepare(
        "SELECT id, case_id, author, title, content, is_pinned, created_at, updated_at
         FROM case_notes
         WHERE case_id = ?1
         ORDER BY is_pinned DESC, created_at DESC"
    ).map_err(|e| e.to_string())?;

    let notes = stmt.query_map([&input.case_id], |row| {
        Ok(CaseNote {
            id: row.get(0)?,
            case_id: row.get(1)?,
            author: row.get(2)?,
            title: row.get(3)?,
            content: row.get(4)?,
            category: "general".to_string(),
            pinned: boolv(row, 5),
            created_at: row.get::<_, String>(6)?,
            updated_at: row.get::<_, String>(7)?,
        })
    }).map_err(|e| e.to_string())?.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())?;

    Ok(notes)
}

#[tauri::command]
pub async fn case_note_create(state: State<'_, AppState>, input: CaseNoteCreateInput) -> Result<CaseNote, String> {
    let db = state.db.lock().await;
    let id = generate_id();
    let now = Utc::now().to_rfc3339();
    let author = input.author.unwrap_or_else(|| "Examiner".to_string());
    let is_pinned = input.pinned.unwrap_or(false);
    let category = input.category.unwrap_or_else(|| "general".to_string());

    db.conn.execute(
        "INSERT INTO case_notes (id, case_id, author, title, content, is_pinned, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?7)",
        rusqlite::params![
            id,
            input.case_id,
            author,
            input.title,
            input.content,
            is_pinned,
            now
        ],
    ).map_err(|e| e.to_string())?;

    Ok(CaseNote {
        id,
        case_id: input.case_id,
        author,
        title: input.title,
        content: input.content,
        category,
        pinned: is_pinned,
        created_at: now.clone(),
        updated_at: now,
    })
}

#[tauri::command]
pub async fn case_note_update(state: State<'_, AppState>, input: CaseNoteUpdateInput) -> Result<(), String> {
    let db = state.db.lock().await;
    let now = Utc::now().to_rfc3339();
    let mut sets = Vec::new();

    if let Some(ref title) = input.title {
        sets.push(format!("title='{}'", title.replace('\'', "''")));
    }
    if let Some(ref content) = input.content {
        sets.push(format!("content='{}'", content.replace('\'', "''")));
    }
    if let Some(pinned) = input.pinned {
        sets.push(format!("is_pinned={}", if pinned { 1 } else { 0 }));
    }

    if sets.is_empty() {
        return Ok(());
    }

    sets.push(format!("updated_at='{}'", now));
    let sql = format!("UPDATE case_notes SET {} WHERE id='{}'", sets.join(","), input.id.replace('\'', "''"));
    db.conn.execute(&sql, []).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn case_note_toggle_pin(state: State<'_, AppState>, note_id: String) -> Result<bool, String> {
    let db = state.db.lock().await;
    let current_pinned: bool = db.conn.query_row(
        "SELECT is_pinned FROM case_notes WHERE id = ?1",
        [&note_id],
        |row| Ok(boolv(row, 0)),
    ).map_err(|e| e.to_string())?;

    let new_pinned = !current_pinned;
    let now = Utc::now().to_rfc3339();

    db.conn.execute(
        "UPDATE case_notes SET is_pinned = ?1, updated_at = ?2 WHERE id = ?3",
        rusqlite::params![new_pinned, now, note_id],
    ).map_err(|e| e.to_string())?;

    Ok(new_pinned)
}

#[tauri::command]
pub async fn case_note_delete(state: State<'_, AppState>, note_id: String) -> Result<(), String> {
    let db = state.db.lock().await;
    db.conn.execute("DELETE FROM case_notes WHERE id = ?1", [&note_id]).map_err(|e| e.to_string())?;
    Ok(())
}
