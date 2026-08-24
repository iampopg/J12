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
    let working_dir = input.working_dir.clone().unwrap_or_else(|| {
        if let Some(doc_dir) = dirs::document_dir() {
            doc_dir.join("J12_Cases").join(&cn).to_string_lossy().to_string()
        } else {
            format!("./cases/{}", cn)
        }
    });

    // Automatically create directory hierarchy inside working folder
    if !working_dir.is_empty() {
        let p = std::path::Path::new(&working_dir);
        let _ = std::fs::create_dir_all(p);
        let _ = std::fs::create_dir_all(p.join("evidence"));
        let _ = std::fs::create_dir_all(p.join("attachments"));
        let _ = std::fs::create_dir_all(p.join("exports"));
        let _ = std::fs::create_dir_all(p.join("reports"));
    }
    
    db.conn.execute(
        "INSERT INTO cases (id,title,case_number,investigation_type,description,status,target_email,target_name,target_organization,working_dir,created_at,updated_at)
         VALUES (?1,?2,?3,?4,?5,'open',?6,?7,?8,?9,?10,?10)",
        rusqlite::params![id, input.title, cn, inv_type, desc, target_email, target_name, target_org, working_dir, now.to_rfc3339()],
    ).map_err(|e| e.to_string())?;

    // Create initial custody log event
    let custody_id = generate_id();
    let _ = db.conn.execute(
        "INSERT INTO chain_of_custody (id, case_id, evidence_id, action, performed_by, timestamp, notes)
         VALUES (?1, ?2, NULL, 'case_created', 'Examiner', ?3, ?4)",
        rusqlite::params![custody_id, id, now.to_rfc3339(), format!("Case '{}' created with working directory '{}'", input.title, working_dir)],
    );

    Ok(Case {
        id,
        title: input.title,
        case_number: cn,
        description: desc,
        status: "open".to_string(),
        owner_id: "default".to_string(),
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
    let mut stmt = db.conn.prepare("SELECT id,title,case_number,description,status,target_email,target_name,target_organization,investigation_type,working_dir,created_at,updated_at FROM cases ORDER BY created_at DESC").map_err(|e| e.to_string())?;
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
            working_dir: row.get(9)?,
            created_at: parse_dt(row.get::<_,String>(10)?.as_str()),
            updated_at: parse_dt(row.get::<_,String>(11)?.as_str()),
        })
    }).map_err(|e| e.to_string())?.collect::<Result<Vec<_>,_>>().map_err(|e| e.to_string())?;
    Ok(cases)
}

#[tauri::command]
pub async fn case_get(state: State<'_, AppState>, input: EmptyInput) -> Result<Option<Case>, String> {
    let db = state.db.lock().await;
    let r = db.conn.query_row(
        "SELECT id,title,case_number,description,status,target_email,target_name,target_organization,investigation_type,working_dir,created_at,updated_at FROM cases WHERE id=?1",
        [&input.case_id],
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
            working_dir: row.get(9)?,
            created_at: parse_dt(row.get::<_,String>(10)?.as_str()), 
            updated_at: parse_dt(row.get::<_,String>(11)?.as_str()) 
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
    let mut sets = Vec::new();
    if !input.title.is_empty() { sets.push(format!("title='{}'", input.title.replace('\'',"''"))); }
    if let Some(ref v) = input.description { sets.push(format!("description='{}'", v.replace('\'',"''"))); }
    if let Some(ref v) = input.status { sets.push(format!("status='{}'", v.replace('\'',"''"))); }
    if sets.is_empty() { return Ok(()); }
    sets.push(format!("updated_at='{}'", now));
    let sql = format!("UPDATE cases SET {} WHERE id='{}'", sets.join(","), input.case_id.replace('\'',"''"));
    db.conn.execute(&sql, []).map_err(|e| e.to_string())?;
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

    let mut db = state.db.lock().await;
    let tx = db.conn.transaction().map_err(|e| e.to_string())?;

    // Cascade delete in child-first foreign key order
    let _ = tx.execute("DELETE FROM email_tags WHERE case_id = ?1 OR email_id IN (SELECT id FROM emails WHERE case_id = ?1)", [&case_id]);
    let _ = tx.execute("DELETE FROM email_notes WHERE case_id = ?1 OR email_id IN (SELECT id FROM emails WHERE case_id = ?1)", [&case_id]);
    let _ = tx.execute("DELETE FROM attachments WHERE email_id IN (SELECT id FROM emails WHERE case_id = ?1)", [&case_id]);
    let _ = tx.execute("DELETE FROM forensic_artifacts WHERE case_id = ?1", [&case_id]);
    let _ = tx.execute("DELETE FROM artifacts_cache WHERE case_id = ?1", [&case_id]);
    let _ = tx.execute("DELETE FROM communication_edges WHERE case_id = ?1", [&case_id]);
    let _ = tx.execute("DELETE FROM timeline_events WHERE case_id = ?1", [&case_id]);
    let _ = tx.execute("DELETE FROM entities WHERE case_id = ?1", [&case_id]);
    let _ = tx.execute("DELETE FROM findings WHERE case_id = ?1", [&case_id]);
    let _ = tx.execute("DELETE FROM custody_events WHERE evidence_id IN (SELECT id FROM evidence_items WHERE case_id = ?1)", [&case_id]);
    let _ = tx.execute("DELETE FROM chain_of_custody WHERE case_id = ?1", [&case_id]);
    let _ = tx.execute("DELETE FROM case_notes WHERE case_id = ?1", [&case_id]);
    let _ = tx.execute("DELETE FROM emails WHERE case_id = ?1", [&case_id]);
    let _ = tx.execute("DELETE FROM evidence_items WHERE case_id = ?1", [&case_id]);
    let r = tx.execute("DELETE FROM cases WHERE id = ?1", [&case_id]).map_err(|e| e.to_string())?;

    tx.commit().map_err(|e| e.to_string())?;
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

    // Check configured target from cases table
    let case_target: Option<(Option<String>, Option<String>, Option<String>)> = db.conn.query_row(
        "SELECT target_email, target_name, target_organization FROM cases WHERE id = ?1",
        [&case_id],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?))
    ).ok();

    let mut stmt = db.conn.prepare(
        "SELECT from_addr, from_display, COUNT(*) as sent_count 
         FROM emails 
         WHERE case_id = ?1 AND from_addr != ''
         GROUP BY from_addr 
         ORDER BY sent_count DESC 
         LIMIT 20"
    ).map_err(|e| e.to_string())?;

    let mut targets = Vec::new();
    let mut seen_emails = std::collections::HashSet::new();

    // If case has target_email, put it first
    if let Some((Some(t_email), t_name, t_org)) = case_target {
        if !t_email.trim().is_empty() {
            let sent: i64 = db.conn.query_row(
                "SELECT COUNT(*) FROM emails WHERE case_id = ?1 AND from_addr LIKE ?2",
                rusqlite::params![&case_id, format!("%{}%", t_email)],
                |r| r.get(0)
            ).unwrap_or(0);

            let recvd: i64 = db.conn.query_row(
                "SELECT COUNT(*) FROM emails WHERE case_id = ?1 AND (to_addrs LIKE ?2 OR cc_addrs LIKE ?2)",
                rusqlite::params![&case_id, format!("%{}%", t_email)],
                |r| r.get(0)
            ).unwrap_or(0);

            seen_emails.insert(t_email.to_lowercase());
            targets.push(serde_json::json!({
                "email": t_email.clone(),
                "display_name": t_name.unwrap_or_else(|| t_email.split('@').next().unwrap_or("").to_string()),
                "organization": t_org.unwrap_or_default(),
                "total_emails": sent + recvd,
                "sent": sent,
                "received": recvd,
                "confidence": "high",
                "is_primary_target": true
            }));
        }
    }

    let candidate_rows = stmt.query_map([&case_id], |row| {
        let email: String = row.get(0)?;
        let name: Option<String> = row.get(1)?;
        let count: i64 = row.get(2)?;
        Ok((email, name, count))
    }).map_err(|e| e.to_string())?.filter_map(|r| r.ok()).collect::<Vec<_>>();

    for (email, name, sent_count) in candidate_rows {
        if !seen_emails.contains(&email.to_lowercase()) {
            seen_emails.insert(email.to_lowercase());
            let org = email.split('@').nth(1).unwrap_or("").to_string();
            let recvd: i64 = db.conn.query_row(
                "SELECT COUNT(*) FROM emails WHERE case_id = ?1 AND (to_addrs LIKE ?2 OR cc_addrs LIKE ?2)",
                rusqlite::params![&case_id, format!("%{}%", email)],
                |r| r.get(0)
            ).unwrap_or(0);

            targets.push(serde_json::json!({
                "email": email.clone(),
                "display_name": name.unwrap_or_else(|| email.split('@').next().unwrap_or("").to_string()),
                "organization": org,
                "total_emails": sent_count + recvd,
                "sent": sent_count,
                "received": recvd,
                "confidence": if sent_count > 20 { "high" } else if sent_count > 5 { "medium" } else { "low" },
                "is_primary_target": false
            }));
        }
    }

    let total_entities: i64 = db.conn.query_row(
        "SELECT COUNT(DISTINCT from_addr) FROM emails WHERE case_id = ?1",
        [&case_id],
        |r| r.get(0)
    ).unwrap_or(0);

    Ok(serde_json::json!({
        "targets": targets.clone(),
        "candidates": targets,
        "total_case_entities": total_entities
    }))
}

#[tauri::command]
pub async fn target_profile(state: State<'_, AppState>, input: Value) -> Result<Value, String> {
    let case_id = input["case_id"].as_str()
        .or_else(|| input["caseId"].as_str())
        .or_else(|| input.as_str())
        .unwrap_or("")
        .to_string();

    let specified_email = input["target_email"].as_str()
        .or_else(|| input["targetEmail"].as_str())
        .map(|s| s.trim().to_string());

    let db = state.db.lock().await;

    let case_row = db.conn.query_row(
        "SELECT title, case_number, target_email, target_name, target_organization FROM cases WHERE id = ?1",
        [&case_id],
        |row| Ok((
            row.get::<_, String>(0)?,
            row.get::<_, Option<String>>(1)?.unwrap_or_default(),
            row.get::<_, Option<String>>(2)?.unwrap_or_default(),
            row.get::<_, Option<String>>(3)?.unwrap_or_default(),
            row.get::<_, Option<String>>(4)?.unwrap_or_default()
        ))
    );

    let (case_title, case_number, case_target_email, case_target_name, case_target_org) = match case_row {
        Ok((t, num, e, n, o)) => (t, num, e, n, o),
        Err(_) => ("Investigation".to_string(), "J12-001".to_string(), String::new(), String::new(), String::new()),
    };

    let mut target_email = specified_email.unwrap_or(case_target_email);
    let mut target_name = case_target_name;
    let mut target_org = case_target_org;

    if target_email.is_empty() {
        let top_sender: Result<(String, Option<String>), _> = db.conn.query_row(
            "SELECT from_addr, from_display FROM emails WHERE case_id = ?1 AND from_addr != '' GROUP BY from_addr ORDER BY COUNT(*) DESC LIMIT 1",
            [&case_id],
            |row| Ok((row.get(0)?, row.get(1)?))
        );

        if let Ok((email, name)) = top_sender {
            target_email = email.clone();
            target_name = name.unwrap_or_else(|| email.split('@').next().unwrap_or("").to_string());
            target_org = email.split('@').nth(1).unwrap_or("").to_string();
        }
    }

    if target_email.is_empty() {
        return Ok(serde_json::json!({
            "case_id": case_id,
            "case_title": case_title,
            "case_number": case_number,
            "target_email": null,
            "target_name": null,
            "target_organization": null,
            "sent_count": 0,
            "received_count": 0,
            "total_emails": 0,
            "first_seen": null,
            "last_seen": null,
            "top_correspondents": [],
            "top_subjects": [],
            "display_names": [],
            "x_mailers": [],
            "originating_ips": [],
            "risk_score": 0,
            "flagged_count": 0,
            "attachment_count": 0,
            "recent_communications": []
        }));
    }

    let target_like = format!("%{}%", target_email);

    // 1. Sent & Received Counts
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

    let total_emails = sent_count + received_count;

    // 2. First and Last Seen Timestamps
    let (first_seen, last_seen): (Option<String>, Option<String>) = db.conn.query_row(
        "SELECT MIN(date_sent_utc), MAX(date_sent_utc) FROM emails WHERE case_id = ?1 AND (from_addr LIKE ?2 OR to_addrs LIKE ?2)",
        rusqlite::params![&case_id, &target_like],
        |r| Ok((r.get(0)?, r.get(1)?))
    ).unwrap_or((None, None));

    // 3. Max Risk Score & Flagged Count
    let (risk_score, flagged_count): (i64, i64) = db.conn.query_row(
        "SELECT COALESCE(MAX(risk_score), 0), COUNT(CASE WHEN risk_score > 25 THEN 1 END) 
         FROM emails WHERE case_id = ?1 AND (from_addr LIKE ?2 OR to_addrs LIKE ?2)",
        rusqlite::params![&case_id, &target_like],
        |r| Ok((r.get(0)?, r.get(1)?))
    ).unwrap_or((0, 0));

    // 4. Attachments Count
    let attachment_count: i64 = db.conn.query_row(
        "SELECT COUNT(*) FROM attachments a JOIN emails e ON a.email_id = e.id 
         WHERE e.case_id = ?1 AND (e.from_addr LIKE ?2 OR e.to_addrs LIKE ?2)",
        rusqlite::params![&case_id, &target_like],
        |r| r.get(0)
    ).unwrap_or(0);

    // 5. Display Names / Aliases
    let mut name_stmt = db.conn.prepare(
        "SELECT DISTINCT from_display FROM emails 
         WHERE case_id = ?1 AND from_addr LIKE ?2 AND from_display IS NOT NULL AND from_display != '' 
         LIMIT 10"
    ).map_err(|e| e.to_string())?;
    let display_names: Vec<String> = name_stmt.query_map(rusqlite::params![&case_id, &target_like], |r| r.get(0))
        .map_err(|e| e.to_string())?.filter_map(|r| r.ok()).collect();

    // 6. X-Mailers (Client Software Used)
    let mut mailer_stmt = db.conn.prepare(
        "SELECT DISTINCT x_mailer FROM emails 
         WHERE case_id = ?1 AND from_addr LIKE ?2 AND x_mailer IS NOT NULL AND x_mailer != '' 
         LIMIT 6"
    ).map_err(|e| e.to_string())?;
    let x_mailers: Vec<String> = mailer_stmt.query_map(rusqlite::params![&case_id, &target_like], |r| r.get(0))
        .map_err(|e| e.to_string())?.filter_map(|r| r.ok()).collect();

    // 7. Originating IPs
    let mut ip_stmt = db.conn.prepare(
        "SELECT DISTINCT x_originating_ip FROM emails 
         WHERE case_id = ?1 AND from_addr LIKE ?2 AND x_originating_ip IS NOT NULL AND x_originating_ip != '' 
         LIMIT 6"
    ).map_err(|e| e.to_string())?;
    let originating_ips: Vec<String> = ip_stmt.query_map(rusqlite::params![&case_id, &target_like], |r| r.get(0))
        .map_err(|e| e.to_string())?.filter_map(|r| r.ok()).collect();

    // 8. Top Direct Correspondents (Tuples [email, count])
    let mut corr_stmt = db.conn.prepare(
        "SELECT from_addr as peer_email, COUNT(*) as cnt FROM emails WHERE case_id = ?1 AND to_addrs LIKE ?2 AND from_addr NOT LIKE ?2 AND from_addr != '' GROUP BY from_addr
         UNION ALL
         SELECT to_addrs as peer_email, COUNT(*) as cnt FROM emails WHERE case_id = ?1 AND from_addr LIKE ?2 AND to_addrs NOT LIKE ?2 AND to_addrs != '' GROUP BY to_addrs
         ORDER BY cnt DESC LIMIT 10"
    ).map_err(|e| e.to_string())?;

    let mut corr_map: std::collections::HashMap<String, i64> = std::collections::HashMap::new();
    let rows = corr_stmt.query_map(rusqlite::params![&case_id, &target_like], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
    }).map_err(|e| e.to_string())?.filter_map(|r| r.ok());

    for (peer, cnt) in rows {
        let clean = peer.trim_matches(|c| c == '[' || c == ']' || c == '"' || c == '\'').trim().to_string();
        if !clean.is_empty() && !clean.contains(&target_email) {
            *corr_map.entry(clean).or_insert(0) += cnt;
        }
    }
    let mut top_correspondents: Vec<(String, i64)> = corr_map.into_iter().collect();
    top_correspondents.sort_by(|a, b| b.1.cmp(&a.1));
    top_correspondents.truncate(8);

    // 9. Top Subject Topics
    let mut subj_stmt = db.conn.prepare(
        "SELECT subject, COUNT(*) as cnt FROM emails 
         WHERE case_id = ?1 AND (from_addr LIKE ?2 OR to_addrs LIKE ?2) AND subject IS NOT NULL AND subject != ''
         GROUP BY subject ORDER BY cnt DESC LIMIT 8"
    ).map_err(|e| e.to_string())?;
    let top_subjects: Vec<(String, i64)> = subj_stmt.query_map(rusqlite::params![&case_id, &target_like], |row| {
        Ok((row.get(0)?, row.get(1)?))
    }).map_err(|e| e.to_string())?.filter_map(|r| r.ok()).collect();

    // 10. Recent Communications Preview
    let mut comm_stmt = db.conn.prepare(
        "SELECT id, subject, date_sent_utc, from_addr, to_addrs, risk_score 
         FROM emails 
         WHERE case_id = ?1 AND (from_addr LIKE ?2 OR to_addrs LIKE ?2) 
         ORDER BY date_sent_utc DESC LIMIT 8"
    ).map_err(|e| e.to_string())?;
    let recent_communications = comm_stmt.query_map(rusqlite::params![&case_id, &target_like], |row| {
        Ok(serde_json::json!({
            "id": row.get::<_, String>(0)?,
            "subject": row.get::<_, Option<String>>(1)?.unwrap_or_else(|| "(No Subject)".to_string()),
            "date": row.get::<_, Option<String>>(2)?,
            "from": row.get::<_, String>(3)?,
            "to": row.get::<_, String>(4)?,
            "risk_score": row.get::<_, i64>(5)?
        }))
    }).map_err(|e| e.to_string())?.filter_map(|r| r.ok()).collect::<Vec<_>>();

    Ok(serde_json::json!({
        "case_id": case_id,
        "case_title": case_title,
        "case_number": case_number,
        "target_email": target_email,
        "target_name": if target_name.is_empty() { target_email.split('@').next().unwrap_or("").to_string() } else { target_name },
        "target_organization": if target_org.is_empty() { target_email.split('@').nth(1).unwrap_or("").to_string() } else { target_org },
        "sent_count": sent_count,
        "received_count": received_count,
        "total_emails": total_emails,
        "first_seen": first_seen,
        "last_seen": last_seen,
        "top_correspondents": top_correspondents,
        "top_subjects": top_subjects,
        "display_names": display_names,
        "x_mailers": x_mailers,
        "originating_ips": originating_ips,
        "risk_score": risk_score,
        "flagged_count": flagged_count,
        "attachment_count": attachment_count,
        "recent_communications": recent_communications
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
