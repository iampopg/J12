use std::collections::{HashMap, HashSet};
use chrono::Utc;
use serde_json::Value;
use tauri::State;

use crate::AppState;
use crate::analysis::{
    analyze_headers, analyze_authentication, detect_spoofing,
    analyze_attachment_metadata, generate_findings, calculate_risk_score, NewFinding
};
use crate::db::{generate_id, parse_dt};
use crate::models::*;
use super::helpers::*;

#[tauri::command]
pub async fn findings_list(state: State<'_, AppState>, input: Value) -> Result<Vec<Finding>, String> {
    let case_id = input["case_id"].as_str()
        .or_else(|| input["caseId"].as_str())
        .or_else(|| input.as_str())
        .unwrap_or("")
        .to_string();

    let db = state.db.lock().await;
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

    let findings = stmt.query_map([&case_id], |row| {
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
    }).map_err(|e| e.to_string())?.collect::<Result<Vec<_>,_>>().map_err(|e| e.to_string())?;

    Ok(findings)
}

#[tauri::command]
pub async fn dashboard(state: State<'_, AppState>, input: EmptyInput) -> Result<DashboardData, String> {
    let db = state.db.lock().await;
    let cid = &input.case_id;

    let te: i64 = db.conn.query_row("SELECT COUNT(*) FROM emails WHERE case_id=?1", [cid], |r| r.get(0)).unwrap_or(0);
    let de: i64 = db.conn.query_row("SELECT COUNT(*) FROM emails WHERE case_id=?1 AND (is_deleted=1 OR deleted_recovered=1)", [cid], |r| r.get(0)).unwrap_or(0);
    let he: i64 = db.conn.query_row("SELECT COUNT(*) FROM emails WHERE case_id=?1 AND risk_score>50", [cid], |r| r.get(0)).unwrap_or(0);
    let ta: i64 = db.conn.query_row("SELECT COUNT(*) FROM attachments WHERE email_id IN (SELECT id FROM emails WHERE case_id=?1)", [cid], |r| r.get(0)).unwrap_or(0);
    let tf: i64 = db.conn.query_row("SELECT COUNT(*) FROM findings WHERE case_id=?1", [cid], |r| r.get(0)).unwrap_or(0);

    let cf: u32 = db.conn.query_row("SELECT COUNT(*) FROM findings WHERE case_id=?1 AND severity='critical'", [cid], |r| r.get(0)).unwrap_or(0);
    let hf: u32 = db.conn.query_row("SELECT COUNT(*) FROM findings WHERE case_id=?1 AND severity='high'", [cid], |r| r.get(0)).unwrap_or(0);
    let mf: u32 = db.conn.query_row("SELECT COUNT(*) FROM findings WHERE case_id=?1 AND severity='medium'", [cid], |r| r.get(0)).unwrap_or(0);
    let lf: u32 = db.conn.query_row("SELECT COUNT(*) FROM findings WHERE case_id=?1 AND severity='low'", [cid], |r| r.get(0)).unwrap_or(0);

    let mut severity_map = HashMap::new();
    severity_map.insert("critical".to_string(), cf);
    severity_map.insert("high".to_string(), hf);
    severity_map.insert("medium".to_string(), mf);
    severity_map.insert("low".to_string(), lf);

    let mut stmt = db.conn.prepare("SELECT from_addr, COUNT(*) as cnt FROM emails WHERE case_id=?1 GROUP BY from_addr ORDER BY cnt DESC LIMIT 5").map_err(|e| e.to_string())?;
    let top_correspondents = stmt.query_map([cid], |row| {
        let count: i64 = row.get(1)?;
        Ok(TopCorrespondent { email: row.get(0)?, sent: count as u32, received: 0 })
    }).map_err(|e| e.to_string())?.collect::<Result<Vec<_>,_>>().map_err(|e| e.to_string())?;

    let inbox_c: i64 = db.conn.query_row("SELECT COUNT(*) FROM emails WHERE case_id=?1 AND folder_category='inbox'", [cid], |r| r.get(0)).unwrap_or(0);
    let sent_c: i64 = db.conn.query_row("SELECT COUNT(*) FROM emails WHERE case_id=?1 AND folder_category='sent'", [cid], |r| r.get(0)).unwrap_or(0);
    let drafts_c: i64 = db.conn.query_row("SELECT COUNT(*) FROM emails WHERE case_id=?1 AND folder_category='drafts'", [cid], |r| r.get(0)).unwrap_or(0);
    let spam_c: i64 = db.conn.query_row("SELECT COUNT(*) FROM emails WHERE case_id=?1 AND folder_category='spam'", [cid], |r| r.get(0)).unwrap_or(0);

    Ok(DashboardData {
        evidence_count: ta as u32,
        email_count: te as u32,
        deleted_recovered: de as u32,
        entity_count: 0,
        finding_count: tf as u32,
        severity_breakdown: severity_map,
        date_range: (None, None),
        top_correspondents,
        sent_count: sent_c as u32,
        inbox_count: inbox_c as u32,
        soft_deleted_count: de as u32,
        drafts_count: drafts_c as u32,
        spam_count: spam_c as u32,
        other_count: 0,
        high_risk_emails: he as u32,
    })
}

#[tauri::command]
pub async fn custody_chain(state: State<'_, AppState>, input: EmptyInput) -> Result<Vec<CustodyEvent>, String> {
    let db = state.db.lock().await;
    let mut stmt = db.conn.prepare("SELECT id,evidence_id,action,performed_by,timestamp,notes FROM chain_of_custody WHERE case_id=?1 ORDER BY timestamp ASC").map_err(|e| e.to_string())?;
    let events = stmt.query_map([&input.case_id], |row| {
        Ok(CustodyEvent { 
            id: row.get(0)?, 
            evidence_id: row.get::<_, Option<String>>(1)?.unwrap_or_default(), 
            action: row.get(2)?, 
            actor: row.get::<_, Option<String>>(3)?.unwrap_or_else(|| "Examiner".to_string()), 
            timestamp: parse_dt(row.get::<_,String>(4)?.as_str()), 
            tool: "J12 Email Forensic Suite".to_string(),
            tool_version: "1.0.0".to_string(),
            hash_before: None,
            hash_after: None,
            detail: row.get(5)?,
        })
    }).map_err(|e| e.to_string())?.collect::<Result<Vec<_>,_>>().map_err(|e| e.to_string())?;
    Ok(events)
}

#[tauri::command]
pub async fn run_analysis(state: State<'_, AppState>, input: Value) -> Result<u32, String> {
    let case_id = input["case_id"].as_str()
        .or_else(|| input["caseId"].as_str())
        .or_else(|| input.as_str())
        .unwrap_or("")
        .to_string();

    let (emails, attachments) = {
        let db = state.db.lock().await;

        let mut stmt = db.conn.prepare(
            "SELECT id, evidence_id, case_id, message_id, in_reply_to, msg_references,
                    from_addr, from_display, to_addrs, cc_addrs, bcc_addrs, reply_to,
                    subject, date_sent, date_sent_utc, headers_raw, body_text, body_html,
                    folder_name, folder_category, is_deleted, deleted_recovered, risk_score, flags
             FROM emails WHERE case_id = ?1"
        ).map_err(|e| e.to_string())?;

        let emails = stmt.query_map([&case_id], |row| {
            Ok(EmailMessage {
                id: row.get(0)?, evidence_id: row.get(1)?, case_id: row.get(2)?, message_id: row.get(3)?,
                from_addr: row.get(4)?, from_display: row.get(5)?, to_addrs: row.get(6)?, cc_addrs: row.get(7)?,
                subject: row.get(8)?, date_sent: row.get(9)?, date_sent_utc: row.get(10)?, headers_raw: row.get(11)?,
                body_text: row.get(12)?, body_html: row.get(13)?, folder_name: row.get(14)?, folder_category: row.get(15)?,
                is_deleted: boolv(row, 16), deleted_recovered: boolv(row, 17), risk_score: u8v(row, 18), flags: row.get(19)?
            })
        }).map_err(|e| e.to_string())?.filter_map(|r| r.ok()).collect::<Vec<_>>();

        let mut att_stmt = db.conn.prepare(
            "SELECT id, email_id, filename, sha256, mime_type, size_bytes, stored_path, entropy, risk_flags
             FROM attachments WHERE email_id IN (SELECT id FROM emails WHERE case_id = ?1)"
        ).map_err(|e| e.to_string())?;

        let attachments = att_stmt.query_map([&case_id], |row| {
            let risk_flags_str: String = row.get::<_, Option<String>>(8)?.unwrap_or_else(|| "[]".to_string());
            Ok(Attachment {
                id: row.get(0)?,
                email_id: row.get(1)?,
                filename: row.get(2)?,
                sha256: row.get(3)?,
                mime_type: row.get(4)?,
                size_bytes: row.get::<_, i64>(5)? as u64,
                stored_path: row.get::<_, Option<String>>(6)?.unwrap_or_default(),
                entropy: row.get(7)?,
                risk_flags: risk_flags_str,
            })
        }).map_err(|e| e.to_string())?.filter_map(|r| r.ok()).collect::<Vec<_>>();

        (emails, attachments)
    };

    let mut att_map: HashMap<String, Vec<Attachment>> = HashMap::new();
    for att in attachments {
        att_map.entry(att.email_id.clone()).or_insert_with(Vec::new).push(att);
    }

    let mut all_findings: Vec<NewFinding> = Vec::new();
    let mut email_risk_scores: Vec<(String, u8)> = Vec::new();

    for email in &emails {
        let headers = analyze_headers(email.headers_raw.as_deref().unwrap_or(""));
        let from_domain = email.from_addr.split('@').nth(1).unwrap_or("");
        let auth = analyze_authentication(email.headers_raw.as_deref().unwrap_or(""), from_domain, None);
        let spoof = detect_spoofing(&email.from_addr, email.from_display.as_deref(), email.headers_raw.as_deref().unwrap_or(""), &auth);

        let email_atts = att_map.get(&email.id).cloned().unwrap_or_default();
        let att_threats: Vec<_> = email_atts.iter().map(|a| {
            analyze_attachment_metadata(a.filename.as_deref(), a.mime_type.as_deref(), a.size_bytes, a.entropy, Some(&a.risk_flags))
        }).collect();

        let risk_score = calculate_risk_score(&headers, &auth, &spoof, &att_threats);
        email_risk_scores.push((email.id.clone(), risk_score));

        let findings = generate_findings(&email.id, &headers, &auth, &spoof, &att_threats);
        all_findings.extend(findings);
    }

    let findings_count = all_findings.len() as u32;

    {
        let mut db = state.db.lock().await;
        let tx = db.conn.transaction().map_err(|e| e.to_string())?;

        for (email_id, score) in email_risk_scores {
            tx.execute("UPDATE emails SET risk_score = ?1 WHERE id = ?2", rusqlite::params![score as i64, email_id])
                .map_err(|e| e.to_string())?;
        }

        tx.execute("DELETE FROM findings WHERE case_id = ?1", [&case_id]).map_err(|e| e.to_string())?;

        for f in all_findings {
            let fid = generate_id();
            let now = Utc::now().to_rfc3339();
            let email_ids_str = serde_json::to_string(&f.email_ids).unwrap_or_else(|_| "[]".to_string());

            tx.execute(
                "INSERT INTO findings (
                    id, case_id, type, severity, confidence, title, description,
                    evidence_refs, email_ids, status, created_at
                ) VALUES (?1,?2,?3,?4,?5,?6,?7,'[]',?8,'new',?9)",
                rusqlite::params![
                    fid, case_id, f.type_, f.severity, f.confidence, f.title, f.description,
                    email_ids_str, now,
                ],
            ).map_err(|e| e.to_string())?;
        }

        tx.commit().map_err(|e| e.to_string())?;
    }

    let custody_id = generate_id();
    let now = Utc::now();
    let db = state.db.lock().await;
    let _ = db.conn.execute(
        "INSERT INTO chain_of_custody (id, case_id, evidence_id, action, performed_by, timestamp, notes)
         VALUES (?1, ?2, NULL, 'analysis_run', 'System Analyzer', ?3, ?4)",
        rusqlite::params![
            custody_id,
            case_id,
            now.to_rfc3339(),
            format!("Automated analysis completed: {} findings generated across {} emails", findings_count, emails.len())
        ],
    );

    Ok(findings_count)
}

#[tauri::command]
pub async fn update_finding_status(
    state: State<'_, AppState>,
    finding_id: String,
    status: String,
    reviewed_by: String,
) -> Result<(), String> {
    let valid_statuses = ["new", "reviewed", "confirmed", "false_positive", "dismissed"];
    if !valid_statuses.contains(&status.as_str()) {
        return Err(format!("Invalid status '{}'. Must be one of: {:?}", status, valid_statuses));
    }

    let db = state.db.lock().await;
    let now = Utc::now().to_rfc3339();

    db.conn.execute(
        "UPDATE findings SET status = ?1, reviewed_by = ?2, reviewed_at = ?3 WHERE id = ?4",
        rusqlite::params![status, reviewed_by, now, finding_id],
    ).map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn add_finding_note(
    state: State<'_, AppState>,
    finding_id: String,
    note: String,
    author: String,
) -> Result<(), String> {
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

#[tauri::command]
pub async fn extract_entities(state: State<'_, AppState>, input: Value) -> Result<u32, String> {
    let case_id = input["case_id"].as_str()
        .or_else(|| input["caseId"].as_str())
        .or_else(|| input.as_str())
        .unwrap_or("")
        .to_string();

    let db = state.db.lock().await;

    let mut stmt = db.conn.prepare(
        "SELECT from_addr, to_addrs, cc_addrs, bcc_addrs 
         FROM emails WHERE case_id = ?1"
    ).map_err(|e| e.to_string())?;

    let rows = stmt.query_map([&case_id], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, String>(3)?,
        ))
    }).map_err(|e| e.to_string())?.filter_map(|r| r.ok()).collect::<Vec<_>>();

    let mut entity_map: HashMap<String, (i64, i64)> = HashMap::new();
    let re_email = regex::Regex::new(r"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}").unwrap();

    for (from, to, cc, bcc) in rows {
        // Extract sender (sent_count)
        for cap in re_email.captures_iter(&from) {
            let val = cap[0].to_lowercase();
            let entry = entity_map.entry(val).or_insert((0, 0));
            entry.0 += 1;
        }

        // Extract recipients (received_count)
        let recipients = format!("{} {} {}", to, cc, bcc);
        for cap in re_email.captures_iter(&recipients) {
            let val = cap[0].to_lowercase();
            let entry = entity_map.entry(val).or_insert((0, 0));
            entry.1 += 1;
        }
    }

    let mut count = 0;
    for (email_addr, (sent, recv)) in entity_map {
        let id = generate_id();
        let first_seen = Utc::now().to_rfc3339();

        let _ = db.conn.execute(
            "INSERT OR REPLACE INTO entities (id, case_id, email_address, display_name, first_seen, last_seen, sent_count, received_count, role, aliases)
             VALUES (?1, ?2, ?3, ?3, ?4, ?4, ?5, ?6, 'participant', '[]')",
            rusqlite::params![id, case_id, email_addr, first_seen, sent, recv]
        );
        count += 1;
    }

    Ok(count)
}

#[tauri::command]
pub async fn entity_list(state: State<'_, AppState>, input: EmptyInput) -> Result<Vec<Entity>, String> {
    let db = state.db.lock().await;
    let mut stmt = db.conn.prepare(
        "SELECT id, case_id, email_address, display_name, first_seen, last_seen, sent_count, received_count, role, aliases
         FROM entities WHERE case_id = ?1 ORDER BY (sent_count + received_count) DESC"
    ).map_err(|e| e.to_string())?;

    let entities = stmt.query_map([&input.case_id], |row| {
        Ok(Entity {
            id: row.get(0)?,
            case_id: row.get(1)?,
            email_address: row.get(2)?,
            display_name: row.get(3)?,
            first_seen: row.get(4)?,
            last_seen: row.get(5)?,
            sent_count: row.get(6)?,
            received_count: row.get(7)?,
            role: row.get(8)?,
            aliases: row.get(9)?,
        })
    }).map_err(|e| e.to_string())?.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())?;

    Ok(entities)
}

#[tauri::command]
pub async fn entity_dive(state: State<'_, AppState>, input: EntityInput) -> Result<Value, String> {
    let db = state.db.lock().await;

    let email_like = format!("%{}%", input.email_address);
    let sent_count: i64 = db.conn.query_row(
        "SELECT COUNT(*) FROM emails WHERE case_id=?1 AND from_addr LIKE ?2",
        rusqlite::params![&input.case_id, &email_like],
        |r| r.get(0)
    ).unwrap_or(0);

    let recv_count: i64 = db.conn.query_row(
        "SELECT COUNT(*) FROM emails WHERE case_id=?1 AND (to_addrs LIKE ?2 OR cc_addrs LIKE ?2)",
        rusqlite::params![&input.case_id, &email_like],
        |r| r.get(0)
    ).unwrap_or(0);

    Ok(serde_json::json!({
        "entity": {
            "email_address": input.email_address,
            "sent_count": sent_count,
            "received_count": recv_count,
        }
    }))
}

#[tauri::command]
pub async fn entity_emails(state: State<'_, AppState>, input: Value) -> Result<Vec<EmailMessage>, String> {
    let case_id = input["case_id"].as_str().unwrap_or("");
    let email_addr = input["email_address"].as_str().or_else(|| input["email"].as_str()).unwrap_or("");
    let db = state.db.lock().await;

    let e_like = format!("%{}%", email_addr);
    let mut stmt = db.conn.prepare(
        "SELECT id, evidence_id, case_id, message_id, from_addr, from_display, to_addrs, cc_addrs, subject,
                date_sent, date_sent_utc, headers_raw, body_text, body_html, folder_name, folder_category,
                is_deleted, deleted_recovered, risk_score, flags
         FROM emails WHERE case_id=?1 AND (from_addr LIKE ?2 OR to_addrs LIKE ?2 OR cc_addrs LIKE ?2)
         ORDER BY date_sent_utc DESC"
    ).map_err(|e| e.to_string())?;

    let emails = stmt.query_map(rusqlite::params![case_id, e_like], |row| {
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

#[tauri::command]
pub async fn entity_heatmap(state: State<'_, AppState>, input: EntityInput) -> Result<Value, String> {
    let db = state.db.lock().await;
    let email_like = format!("%{}%", input.email_address);

    let mut stmt = db.conn.prepare(
        "SELECT strftime('%Y-%m-%d', date_sent) as day, COUNT(*) as count 
         FROM emails WHERE case_id=?1 AND (from_addr LIKE ?2 OR to_addrs LIKE ?2) AND day IS NOT NULL 
         GROUP BY day ORDER BY day ASC"
    ).map_err(|e| e.to_string())?;

    let points = stmt.query_map(rusqlite::params![&input.case_id, email_like], |row| {
        Ok(serde_json::json!({
            "date": row.get::<_, String>(0)?,
            "count": row.get::<_, i64>(1)?
        }))
    }).map_err(|e| e.to_string())?.filter_map(|r| r.ok()).collect::<Vec<_>>();

    Ok(serde_json::json!(points))
}

#[tauri::command]
pub async fn timeline_data(state: State<'_, AppState>, input: EmptyInput) -> Result<Value, String> {
    let db = state.db.lock().await;

    let mut stmt = db.conn.prepare(
        "SELECT strftime('%Y-%m-%d', date_sent) as day, 
                COUNT(*) as total_count,
                SUM(CASE WHEN risk_score > 50 THEN 1 ELSE 0 END) as high_risk_count,
                SUM(CASE WHEN is_deleted = 1 OR deleted_recovered = 1 THEN 1 ELSE 0 END) as deleted_count
         FROM emails 
         WHERE case_id = ?1 AND day IS NOT NULL 
         GROUP BY day 
         ORDER BY day ASC"
    ).map_err(|e| e.to_string())?;

    let points = stmt.query_map([&input.case_id], |row| {
        Ok(serde_json::json!({
            "date": row.get::<_, String>(0)?,
            "total_count": row.get::<_, i64>(1)?,
            "high_risk_count": row.get::<_, i64>(2)?,
            "deleted_count": row.get::<_, i64>(3)?
        }))
    }).map_err(|e| e.to_string())?.filter_map(|r| r.ok()).collect::<Vec<_>>();

    Ok(serde_json::json!(points))
}

#[tauri::command]
pub async fn graph_data(state: State<'_, AppState>, input: EmptyInput) -> Result<Value, String> {
    let db = state.db.lock().await;

    let mut stmt = db.conn.prepare(
        "SELECT from_addr, to_addrs, risk_score FROM emails WHERE case_id = ?1 AND from_addr != ''"
    ).map_err(|e| e.to_string())?;

    let rows = stmt.query_map([&input.case_id], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, Option<i64>>(2)?.unwrap_or(0) as u8
        ))
    }).map_err(|e| e.to_string())?.filter_map(|r| r.ok()).collect::<Vec<_>>();

    let mut node_set: HashSet<String> = HashSet::new();
    let mut node_risks: HashMap<String, u8> = HashMap::new();
    let mut edge_map: HashMap<(String, String), u32> = HashMap::new();

    for (from, to_json, risk) in rows {
        node_set.insert(from.clone());
        let current_risk = node_risks.entry(from.clone()).or_insert(0);
        if risk > *current_risk {
            *current_risk = risk;
        }

        let recipients: Vec<String> = serde_json::from_str(&to_json).unwrap_or_default();
        for r in recipients {
            if !r.is_empty() {
                node_set.insert(r.clone());
                let r_risk = node_risks.entry(r.clone()).or_insert(0);
                if risk > *r_risk {
                    *r_risk = risk;
                }

                let edge_key = if from < r {
                    (from.clone(), r.clone())
                } else {
                    (r.clone(), from.clone())
                };

                *edge_map.entry(edge_key).or_insert(0) += 1;
            }
        }
    }

    let nodes: Vec<Value> = node_set.into_iter().map(|email| {
        let risk = node_risks.get(&email).cloned().unwrap_or(0);
        serde_json::json!({
            "id": email,
            "label": email,
            "risk_score": risk
        })
    }).collect();

    let edges: Vec<Value> = edge_map.into_iter().map(|((source, target), weight)| {
        serde_json::json!({
            "source": source,
            "target": target,
            "weight": weight
        })
    }).collect();

    Ok(serde_json::json!({
        "nodes": nodes,
        "edges": edges
    }))
}
