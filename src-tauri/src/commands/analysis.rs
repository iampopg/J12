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
        .or_else(|| input["input"]["case_id"].as_str())
        .or_else(|| input["input"]["caseId"].as_str())
        .or_else(|| input.as_str())
        .unwrap_or("")
        .to_string();

    let evidence_id = input["evidence_id"].as_str()
        .or_else(|| input["evidenceId"].as_str())
        .or_else(|| input["input"]["evidence_id"].as_str())
        .or_else(|| input["input"]["evidenceId"].as_str())
        .filter(|s| !s.trim().is_empty() && *s != "all");

    let db = state.db.lock().await;

    let findings = if let Some(ev_id) = evidence_id {
        let mut stmt = db.conn.prepare(
            "SELECT id,case_id,type,severity,confidence,title,description,evidence_refs,email_ids,status,created_at,reviewed_by,reviewed_at,notes 
             FROM findings WHERE case_id=?1 AND (evidence_refs LIKE ?2 OR email_ids LIKE ?2)
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
        let ev_like = format!("%{}%", ev_id);
        let rows = stmt.query_map([&case_id, &ev_like], |row| {
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
        rows
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

        let rows = stmt.query_map([&case_id], |row| {
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
        rows
    };

    Ok(findings)
}

#[tauri::command]
pub async fn dashboard(state: State<'_, AppState>, input: Value) -> Result<DashboardData, String> {
    let cid = input["case_id"].as_str()
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
        .filter(|s| !s.trim().is_empty() && *s != "all");

    let db = state.db.lock().await;

    if let Some(ev_id) = evidence_id {
        let te: i64 = db.conn.query_row("SELECT COUNT(*) FROM emails WHERE case_id=?1 AND evidence_id=?2", [&cid, ev_id], |r| r.get(0)).unwrap_or(0);
        let de: i64 = db.conn.query_row("SELECT COUNT(*) FROM emails WHERE case_id=?1 AND evidence_id=?2 AND (is_deleted=1 OR deleted_recovered=1)", [&cid, ev_id], |r| r.get(0)).unwrap_or(0);
        let he: i64 = db.conn.query_row("SELECT COUNT(*) FROM emails WHERE case_id=?1 AND evidence_id=?2 AND risk_score>50", [&cid, ev_id], |r| r.get(0)).unwrap_or(0);
        let ta: i64 = db.conn.query_row("SELECT COUNT(*) FROM attachments a JOIN emails e ON a.email_id = e.id WHERE e.case_id=?1 AND e.evidence_id=?2", [&cid, ev_id], |r| r.get(0)).unwrap_or(0);
        let tf: i64 = db.conn.query_row("SELECT COUNT(*) FROM findings WHERE case_id=?1 AND (evidence_refs LIKE ?2 OR email_ids LIKE ?2)", [&cid, &format!("%{}%", ev_id)], |r| r.get(0)).unwrap_or(0);

        let mut stmt = db.conn.prepare("SELECT from_addr, COUNT(*) as cnt FROM emails WHERE case_id=?1 AND evidence_id=?2 GROUP BY from_addr ORDER BY cnt DESC LIMIT 5").map_err(|e| e.to_string())?;
        let top_correspondents = stmt.query_map([&cid, ev_id], |row| {
            let count: i64 = row.get(1)?;
            Ok(TopCorrespondent { email: row.get(0)?, sent: count as u32, received: 0 })
        }).map_err(|e| e.to_string())?.collect::<Result<Vec<_>,_>>().map_err(|e| e.to_string())?;

        let inbox_c: i64 = db.conn.query_row("SELECT COUNT(*) FROM emails WHERE case_id=?1 AND evidence_id=?2 AND folder_category='inbox'", [&cid, ev_id], |r| r.get(0)).unwrap_or(0);
        let important_c: i64 = db.conn.query_row(
            "SELECT COUNT(*) FROM emails WHERE case_id=?1 AND evidence_id=?2 AND (folder_category='important' OR folder_name LIKE '%important%' OR flags LIKE '%important%')",
            [&cid, ev_id],
            |r| r.get(0)
        ).unwrap_or(0);
        let sent_c: i64 = db.conn.query_row("SELECT COUNT(*) FROM emails WHERE case_id=?1 AND evidence_id=?2 AND folder_category='sent'", [&cid, ev_id], |r| r.get(0)).unwrap_or(0);
        let drafts_c: i64 = db.conn.query_row("SELECT COUNT(*) FROM emails WHERE case_id=?1 AND evidence_id=?2 AND folder_category='drafts'", [&cid, ev_id], |r| r.get(0)).unwrap_or(0);
        let spam_c: i64 = db.conn.query_row("SELECT COUNT(*) FROM emails WHERE case_id=?1 AND evidence_id=?2 AND folder_category='spam'", [&cid, ev_id], |r| r.get(0)).unwrap_or(0);
        let other_c: i64 = db.conn.query_row("SELECT COUNT(*) FROM emails WHERE case_id=?1 AND evidence_id=?2 AND folder_category NOT IN ('inbox', 'important', 'sent', 'drafts', 'spam', 'trash', 'soft_deleted')", [&cid, ev_id], |r| r.get(0)).unwrap_or(0);

        return Ok(DashboardData {
            evidence_count: ta as u32,
            email_count: te as u32,
            deleted_recovered: de as u32,
            entity_count: 0,
            finding_count: tf as u32,
            severity_breakdown: HashMap::new(),
            date_range: (None, None),
            top_correspondents,
            sent_count: sent_c as u32,
            inbox_count: inbox_c as u32,
            important_count: important_c as u32,
            soft_deleted_count: de as u32,
            drafts_count: drafts_c as u32,
            spam_count: spam_c as u32,
            other_count: other_c as u32,
            high_risk_emails: he as u32,
        });
    }

    let te: i64 = db.conn.query_row("SELECT COUNT(*) FROM emails WHERE case_id=?1", [&cid], |r| r.get(0)).unwrap_or(0);
    let de: i64 = db.conn.query_row("SELECT COUNT(*) FROM emails WHERE case_id=?1 AND (is_deleted=1 OR deleted_recovered=1)", [&cid], |r| r.get(0)).unwrap_or(0);
    let he: i64 = db.conn.query_row("SELECT COUNT(*) FROM emails WHERE case_id=?1 AND risk_score>50", [&cid], |r| r.get(0)).unwrap_or(0);
    let ta: i64 = db.conn.query_row("SELECT COUNT(*) FROM attachments WHERE email_id IN (SELECT id FROM emails WHERE case_id=?1)", [&cid], |r| r.get(0)).unwrap_or(0);
    let tf: i64 = db.conn.query_row("SELECT COUNT(*) FROM findings WHERE case_id=?1", [&cid], |r| r.get(0)).unwrap_or(0);

    let cf: u32 = db.conn.query_row("SELECT COUNT(*) FROM findings WHERE case_id=?1 AND severity='critical'", [&cid], |r| r.get(0)).unwrap_or(0);
    let hf: u32 = db.conn.query_row("SELECT COUNT(*) FROM findings WHERE case_id=?1 AND severity='high'", [&cid], |r| r.get(0)).unwrap_or(0);
    let mf: u32 = db.conn.query_row("SELECT COUNT(*) FROM findings WHERE case_id=?1 AND severity='medium'", [&cid], |r| r.get(0)).unwrap_or(0);
    let lf: u32 = db.conn.query_row("SELECT COUNT(*) FROM findings WHERE case_id=?1 AND severity='low'", [&cid], |r| r.get(0)).unwrap_or(0);

    let mut severity_map = HashMap::new();
    severity_map.insert("critical".to_string(), cf);
    severity_map.insert("high".to_string(), hf);
    severity_map.insert("medium".to_string(), mf);
    severity_map.insert("low".to_string(), lf);

    let mut stmt = db.conn.prepare("SELECT from_addr, COUNT(*) as cnt FROM emails WHERE case_id=?1 GROUP BY from_addr ORDER BY cnt DESC LIMIT 5").map_err(|e| e.to_string())?;
    let top_correspondents = stmt.query_map([&cid], |row| {
        let count: i64 = row.get(1)?;
        Ok(TopCorrespondent { email: row.get(0)?, sent: count as u32, received: 0 })
    }).map_err(|e| e.to_string())?.collect::<Result<Vec<_>,_>>().map_err(|e| e.to_string())?;

    let inbox_c: i64 = db.conn.query_row("SELECT COUNT(*) FROM emails WHERE case_id=?1 AND folder_category='inbox'", [&cid], |r| r.get(0)).unwrap_or(0);
    let important_c: i64 = db.conn.query_row(
        "SELECT COUNT(*) FROM emails WHERE case_id=?1 AND (folder_category='important' OR folder_name LIKE '%important%' OR flags LIKE '%important%')",
        [&cid],
        |r| r.get(0)
    ).unwrap_or(0);
    let sent_c: i64 = db.conn.query_row("SELECT COUNT(*) FROM emails WHERE case_id=?1 AND folder_category='sent'", [&cid], |r| r.get(0)).unwrap_or(0);
    let drafts_c: i64 = db.conn.query_row("SELECT COUNT(*) FROM emails WHERE case_id=?1 AND folder_category='drafts'", [&cid], |r| r.get(0)).unwrap_or(0);
    let spam_c: i64 = db.conn.query_row("SELECT COUNT(*) FROM emails WHERE case_id=?1 AND folder_category='spam'", [&cid], |r| r.get(0)).unwrap_or(0);
    let other_c: i64 = db.conn.query_row("SELECT COUNT(*) FROM emails WHERE case_id=?1 AND folder_category NOT IN ('inbox', 'important', 'sent', 'drafts', 'spam', 'trash', 'soft_deleted')", [&cid], |r| r.get(0)).unwrap_or(0);

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
        important_count: important_c as u32,
        soft_deleted_count: de as u32,
        drafts_count: drafts_c as u32,
        spam_count: spam_c as u32,
        other_count: other_c as u32,
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
                is_deleted: boolv(row, 16), deleted_recovered: boolv(row, 17), risk_score: u8v(row, 18), flags: row.get(19)?, attachment_count: 0, image_count: 0
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
            is_deleted: boolv(row, 16), deleted_recovered: boolv(row, 17), risk_score: u8v(row, 18), flags: row.get(19)?, attachment_count: 0, image_count: 0
        })
    }).map_err(|e| e.to_string())?.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())?;

    Ok(emails)
}

fn classify_entity_role(email: &str, display_name: Option<&str>) -> &'static str {
    let local = email.split('@').next().unwrap_or("").to_lowercase();
    let domain = email.split('@').nth(1).unwrap_or("").to_lowercase();
    let dname = display_name.unwrap_or("").to_lowercase();

    // Automated / service keywords in local part
    let org_prefixes = [
        "noreply", "no-reply", "no_reply", "donotreply", "do-not-reply",
        "support", "help", "helpdesk", "info", "contact", "marketing", "sales",
        "billing", "news", "newsletter", "newsdigest", "shop", "store", "orders",
        "updates", "team", "hello", "hi", "admin", "system", "mailer-daemon",
        "daemon", "postmaster", "bounce", "security", "notification", "notifications",
        "alert", "alerts", "alertsp", "pncalerts", "confirm", "confirmation",
        "receipt", "receipts", "promo", "promotions", "reply", "feedback",
        "accounts", "auth", "autonotify", "automated", "service", "services",
        "welcome", "engage", "member", "membership", "digest", "daily", "weekly",
        "investor", "press", "careers", "jobs", "privacy", "legal", "invoice",
        "customer", "reps", "xboxreps", "huntingtononline", "online", "premium",
        "informational", "extravaluechecks", "chase", "bounce-", "bounces", "aws-"
    ];

    for prefix in &org_prefixes {
        if local == *prefix
            || local.starts_with(&format!("{}-", prefix))
            || local.starts_with(&format!("{}.", prefix))
            || local.starts_with(&format!("{}_", prefix))
            || local.contains(prefix)
        {
            return "organization";
        }
    }

    // Subdomains dedicated to transactional bots or broadcast services
    if domain.contains(".mail.")
        || domain.contains(".emails.")
        || domain.contains(".e.")
        || domain.contains(".engage.")
        || domain.contains(".insideapple.")
        || domain.contains(".alertsp.")
        || domain.contains(".m.")
        || domain.ends_with("redditmail.com")
        || domain.ends_with("academia-mail.com")
        || domain.contains("e-mail.")
    {
        return "organization";
    }

    // Display name clues
    let org_dname_keywords = [
        "team", "support", "updates", "alerts", "notifications", "news", "customer", "security",
        "renewals", "marketing", "department", "llc", "inc", "corp", "bank", "service", "store",
        "digest", "accounts", "official", "mailer", "daemon"
    ];
    for kw in &org_dname_keywords {
        if dname.contains(kw) {
            return "organization";
        }
    }

    "person"
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
        "SELECT from_addr, from_display, to_addrs, cc_addrs, bcc_addrs, date_sent_utc 
         FROM emails WHERE case_id = ?1"
    ).map_err(|e| e.to_string())?;

    let rows = stmt.query_map([&case_id], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, Option<String>>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, String>(3)?,
            row.get::<_, String>(4)?,
            row.get::<_, Option<String>>(5)?,
        ))
    }).map_err(|e| e.to_string())?.filter_map(|r| r.ok()).collect::<Vec<_>>();

    // email_addr -> (sent_count, recv_count, Option<display_name>, Option<first_seen>, Option<last_seen>)
    let mut entity_map: HashMap<String, (i64, i64, Option<String>, Option<String>, Option<String>)> = HashMap::new();
    let re_email = regex::Regex::new(r"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}").unwrap();

    for (from, from_disp, to, cc, bcc, date_sent) in rows {
        // Extract sender (sent_count)
        for cap in re_email.captures_iter(&from) {
            let val = cap[0].to_lowercase();
            let entry = entity_map.entry(val).or_insert((0, 0, None, None, None));
            entry.0 += 1;

            if entry.2.is_none() && from_disp.is_some() {
                let d = from_disp.clone().unwrap_or_default().trim().to_string();
                if !d.is_empty() && d != cap[0] {
                    entry.2 = Some(d);
                }
            }

            if let Some(ref d) = date_sent {
                if entry.3.is_none() || entry.3.as_ref().map(|v| d < v).unwrap_or(false) {
                    entry.3 = Some(d.clone());
                }
                if entry.4.is_none() || entry.4.as_ref().map(|v| d > v).unwrap_or(false) {
                    entry.4 = Some(d.clone());
                }
            }
        }

        // Extract recipients (received_count)
        let recipients = format!("{} {} {}", to, cc, bcc);
        for cap in re_email.captures_iter(&recipients) {
            let val = cap[0].to_lowercase();
            let entry = entity_map.entry(val).or_insert((0, 0, None, None, None));
            entry.1 += 1;

            if let Some(ref d) = date_sent {
                if entry.3.is_none() || entry.3.as_ref().map(|v| d < v).unwrap_or(false) {
                    entry.3 = Some(d.clone());
                }
                if entry.4.is_none() || entry.4.as_ref().map(|v| d > v).unwrap_or(false) {
                    entry.4 = Some(d.clone());
                }
            }
        }
    }

    let mut count = 0;
    let now = Utc::now().to_rfc3339();

    for (email_addr, (sent, recv, display_name, first_seen, last_seen)) in entity_map {
        let id = generate_id();
        let f_seen = first_seen.unwrap_or_else(|| now.clone());
        let l_seen = last_seen.unwrap_or_else(|| now.clone());
        let disp_name = display_name.unwrap_or_else(|| email_addr.clone());
        let role = classify_entity_role(&email_addr, Some(&disp_name));

        let _ = db.conn.execute(
            "INSERT OR REPLACE INTO entities (id, case_id, email_address, display_name, first_seen, last_seen, sent_count, received_count, role, aliases)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, '[]')",
            rusqlite::params![id, case_id, email_addr, disp_name, f_seen, l_seen, sent, recv, role]
        );
        count += 1;
    }

    Ok(count)
}

#[tauri::command]
pub async fn entity_list(state: State<'_, AppState>, input: Value) -> Result<Vec<Entity>, String> {
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
        .filter(|s| !s.trim().is_empty() && *s != "all");

    let db = state.db.lock().await;

    let entities = if let Some(ev_id) = evidence_id {
        let mut stmt = db.conn.prepare(
            "SELECT from_addr, from_display, to_addrs, cc_addrs, date_sent_utc 
             FROM emails WHERE case_id = ?1 AND evidence_id = ?2"
        ).map_err(|e| e.to_string())?;

        let rows = stmt.query_map([&case_id, ev_id], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, Option<String>>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, Option<String>>(4)?,
            ))
        }).map_err(|e| e.to_string())?.filter_map(|r| r.ok()).collect::<Vec<_>>();

        let mut entity_map: HashMap<String, (i64, i64, Option<String>, Option<String>, Option<String>)> = HashMap::new();
        let re_email = regex::Regex::new(r"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}").unwrap();

        for (from, from_disp, to, cc, date_sent) in rows {
            for cap in re_email.captures_iter(&from) {
                let val = cap[0].to_lowercase();
                let entry = entity_map.entry(val).or_insert((0, 0, None, None, None));
                entry.0 += 1;
                if entry.2.is_none() && from_disp.is_some() {
                    let d = from_disp.clone().unwrap_or_default().trim().to_string();
                    if !d.is_empty() && d != cap[0] { entry.2 = Some(d); }
                }
                if let Some(ref d) = date_sent {
                    if entry.3.is_none() || entry.3.as_ref().map(|v| d < v).unwrap_or(false) { entry.3 = Some(d.clone()); }
                    if entry.4.is_none() || entry.4.as_ref().map(|v| d > v).unwrap_or(false) { entry.4 = Some(d.clone()); }
                }
            }
            let recipients = format!("{} {}", to, cc);
            for cap in re_email.captures_iter(&recipients) {
                let val = cap[0].to_lowercase();
                let entry = entity_map.entry(val).or_insert((0, 0, None, None, None));
                entry.1 += 1;
                if let Some(ref d) = date_sent {
                    if entry.3.is_none() || entry.3.as_ref().map(|v| d < v).unwrap_or(false) { entry.3 = Some(d.clone()); }
                    if entry.4.is_none() || entry.4.as_ref().map(|v| d > v).unwrap_or(false) { entry.4 = Some(d.clone()); }
                }
            }
        }

        let now = Utc::now().to_rfc3339();
        let mut list = Vec::new();
        for (addr, (sent, recv, disp, f_seen, l_seen)) in entity_map {
            let role = classify_entity_role(&addr, disp.as_deref());
            list.push(Entity {
                id: format!("ent-{}", addr),
                case_id: case_id.clone(),
                email_address: addr.clone(),
                display_name: Some(disp.unwrap_or_else(|| addr.clone())),
                first_seen: Some(f_seen.unwrap_or_else(|| now.clone())),
                last_seen: Some(l_seen.unwrap_or_else(|| now.clone())),
                sent_count: sent,
                received_count: recv,
                role: role.to_string(),
                aliases: Some(String::new()),
            });
        }
        list.sort_by(|a, b| (b.sent_count + b.received_count).cmp(&(a.sent_count + a.received_count)));
        list
    } else {
        let mut stmt = db.conn.prepare(
            "SELECT id, case_id, email_address, display_name, first_seen, last_seen, sent_count, received_count, role, aliases
             FROM entities WHERE case_id = ?1 ORDER BY (sent_count + received_count) DESC"
        ).map_err(|e| e.to_string())?;

        let ents = stmt.query_map([&case_id], |row| {
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
        ents
    };

    Ok(entities)
}

#[tauri::command]
pub async fn entity_dive(state: State<'_, AppState>, input: EntityInput) -> Result<Value, String> {
    let db = state.db.lock().await;
    let email = input.email_address;
    let email_like = format!("%{}%", email);

    // Look up display_name, first_seen, last_seen, aliases from entities table
    let entity_row: (Option<String>, Option<String>, Option<String>, Option<String>) = db.conn.query_row(
        "SELECT display_name, first_seen, last_seen, aliases FROM entities WHERE case_id=?1 AND email_address=?2",
        rusqlite::params![&input.case_id, &email],
        |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?))
    ).unwrap_or((None, None, None, None));

    let display_name = entity_row.0;
    let first_seen = entity_row.1;
    let last_seen = entity_row.2;
    let aliases_raw = entity_row.3.unwrap_or_default();
    let aliases: Vec<String> = if !aliases_raw.is_empty() {
        aliases_raw.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect()
    } else {
        vec![]
    };

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

    let deleted_count: i64 = db.conn.query_row(
        "SELECT COUNT(*) FROM emails WHERE case_id=?1 AND (from_addr LIKE ?2 OR to_addrs LIKE ?2 OR cc_addrs LIKE ?2) AND (is_deleted=1 OR deleted_recovered=1)",
        rusqlite::params![&input.case_id, &email_like],
        |r| r.get(0)
    ).unwrap_or(0);

    let flagged_count: i64 = db.conn.query_row(
        "SELECT COUNT(*) FROM emails WHERE case_id=?1 AND (from_addr LIKE ?2 OR to_addrs LIKE ?2 OR cc_addrs LIKE ?2) AND risk_score >= 25",
        rusqlite::params![&input.case_id, &email_like],
        |r| r.get(0)
    ).unwrap_or(0);

    let total_count = sent_count + recv_count;

    // Top communicating partners (sent to)
    let mut sent_stmt = db.conn.prepare(
        "SELECT to_addrs, COUNT(*) as c FROM emails WHERE case_id=?1 AND from_addr LIKE ?2 GROUP BY to_addrs ORDER BY c DESC LIMIT 8"
    ).map_err(|e| e.to_string())?;
    let sent_to: Vec<(String, i64)> = sent_stmt.query_map(rusqlite::params![&input.case_id, &email_like], |r| {
        Ok((r.get(0)?, r.get(1)?))
    }).map_err(|e| e.to_string())?.filter_map(|r| r.ok()).collect();

    // Top communicating partners (received from)
    let mut recv_stmt = db.conn.prepare(
        "SELECT from_addr, COUNT(*) as c FROM emails WHERE case_id=?1 AND (to_addrs LIKE ?2 OR cc_addrs LIKE ?2) GROUP BY from_addr ORDER BY c DESC LIMIT 8"
    ).map_err(|e| e.to_string())?;
    let received_from: Vec<(String, i64)> = recv_stmt.query_map(rusqlite::params![&input.case_id, &email_like], |r| {
        Ok((r.get(0)?, r.get(1)?))
    }).map_err(|e| e.to_string())?.filter_map(|r| r.ok()).collect();

    // Top subjects
    let mut subj_stmt = db.conn.prepare(
        "SELECT subject, COUNT(*) as c FROM emails WHERE case_id=?1 AND (from_addr LIKE ?2 OR to_addrs LIKE ?2) AND subject IS NOT NULL AND subject != '' GROUP BY subject ORDER BY c DESC LIMIT 8"
    ).map_err(|e| e.to_string())?;
    let top_subjects: Vec<(String, i64)> = subj_stmt.query_map(rusqlite::params![&input.case_id, &email_like], |r| {
        Ok((r.get::<_, Option<String>>(0)?.unwrap_or_default(), r.get(1)?))
    }).map_err(|e| e.to_string())?.filter_map(|r| r.ok()).collect();

    Ok(serde_json::json!({
        "email": email,
        "display_name": display_name,
        "first_seen": first_seen,
        "last_seen": last_seen,
        "sent_count": sent_count,
        "received_count": recv_count,
        "deleted_count": deleted_count,
        "flagged_count": flagged_count,
        "total_count": total_count,
        "aliases": aliases,
        "sent_to": sent_to,
        "received_from": received_from,
        "top_subjects": top_subjects,
    }))
}

#[tauri::command]
pub async fn entity_emails(state: State<'_, AppState>, input: Value) -> Result<Vec<EmailMessage>, String> {
    let case_id = input["case_id"].as_str().unwrap_or("");
    let email_addr = input["email_address"].as_str().or_else(|| input["email"].as_str()).unwrap_or("");
    let filter_type = input["filter_type"].as_str().unwrap_or("all");
    let partner = input["partner_email"].as_str().unwrap_or("");
    let query = input["q"].as_str().unwrap_or("");
    let date_from = input["date_from"].as_str().unwrap_or("");
    let date_to = input["date_to"].as_str().unwrap_or("");
    let has_att = input["has_attachment"].as_bool().unwrap_or(false);
    let human_only = input["human_only"].as_bool().unwrap_or(false);

    let db = state.db.lock().await;

    let e_like = format!("%{}%", email_addr);
    let mut conditions = vec!["case_id = ?1".to_string()];
    let mut params: Vec<Box<dyn rusqlite::ToSql>> = vec![Box::new(case_id.to_string())];

    // Filter by direction / tab
    match filter_type {
        "sent" => {
            conditions.push("from_addr LIKE ?".to_string());
            params.push(Box::new(e_like.clone()));
        }
        "received" => {
            conditions.push("(to_addrs LIKE ? OR cc_addrs LIKE ?)".to_string());
            params.push(Box::new(e_like.clone()));
            params.push(Box::new(e_like.clone()));
        }
        "deleted" => {
            conditions.push("(from_addr LIKE ? OR to_addrs LIKE ? OR cc_addrs LIKE ?)".to_string());
            params.push(Box::new(e_like.clone()));
            params.push(Box::new(e_like.clone()));
            params.push(Box::new(e_like.clone()));
            conditions.push("(is_deleted = 1 OR deleted_recovered = 1)".to_string());
        }
        "flagged" => {
            conditions.push("(from_addr LIKE ? OR to_addrs LIKE ? OR cc_addrs LIKE ?)".to_string());
            params.push(Box::new(e_like.clone()));
            params.push(Box::new(e_like.clone()));
            params.push(Box::new(e_like.clone()));
            conditions.push("risk_score >= 25".to_string());
        }
        _ => {
            conditions.push("(from_addr LIKE ? OR to_addrs LIKE ? OR cc_addrs LIKE ?)".to_string());
            params.push(Box::new(e_like.clone()));
            params.push(Box::new(e_like.clone()));
            params.push(Box::new(e_like.clone()));
        }
    }

    // Partner filter
    if !partner.is_empty() {
        let p_like = format!("%{}%", partner);
        conditions.push("(from_addr LIKE ? OR to_addrs LIKE ? OR cc_addrs LIKE ?)".to_string());
        params.push(Box::new(p_like.clone()));
        params.push(Box::new(p_like.clone()));
        params.push(Box::new(p_like));
    }

    // Search query
    if !query.is_empty() {
        let q_like = format!("%{}%", query);
        conditions.push("(subject LIKE ? OR from_addr LIKE ? OR to_addrs LIKE ? OR body_text LIKE ?)".to_string());
        params.push(Box::new(q_like.clone()));
        params.push(Box::new(q_like.clone()));
        params.push(Box::new(q_like.clone()));
        params.push(Box::new(q_like));
    }

    // Dates
    if !date_from.is_empty() {
        conditions.push("date_sent_utc >= ?".to_string());
        params.push(Box::new(date_from.to_string()));
    }
    if !date_to.is_empty() {
        conditions.push("date_sent_utc <= ?".to_string());
        params.push(Box::new(format!("{}T23:59:59Z", date_to)));
    }

    // Has attachment
    if has_att {
        conditions.push("(SELECT COUNT(*) FROM attachments WHERE email_id = e.id) > 0".to_string());
    }

    // Human Only Filter (Excludes automated newsletters, notifications, no-reply, OTPs, receipts)
    if human_only {
        conditions.push("from_addr NOT LIKE '%no-reply%' AND from_addr NOT LIKE '%noreply%' AND from_addr NOT LIKE '%donotreply%' AND from_addr NOT LIKE '%notifications%' AND from_addr NOT LIKE '%alerts%' AND from_addr NOT LIKE '%mailer-daemon%' AND from_addr NOT LIKE '%automated%' AND from_addr NOT LIKE '%newsletter%' AND from_addr NOT LIKE '%billing%' AND from_addr NOT LIKE '%bounce%'".to_string());
        conditions.push("(subject NOT LIKE '%verification code%' AND subject NOT LIKE '%one-time password%' AND subject NOT LIKE '%otp%' AND subject NOT LIKE '%password reset%' AND subject NOT LIKE '%order confirmation%' AND subject NOT LIKE '%your receipt%')".to_string());
    }

    let sql = format!(
        "SELECT id, evidence_id, case_id, message_id, from_addr, from_display, to_addrs, cc_addrs, subject,
                date_sent, date_sent_utc, folder_name, folder_category,
                is_deleted, deleted_recovered, risk_score, flags,
                body_text, body_html, headers_raw,
                (SELECT COUNT(*) FROM attachments WHERE email_id = e.id) as attachment_count,
                (SELECT COUNT(*) FROM attachments WHERE email_id = e.id AND (mime_type LIKE 'image/%' OR filename LIKE '%.jpg' OR filename LIKE '%.jpeg' OR filename LIKE '%.png' OR filename LIKE '%.gif' OR filename LIKE '%.webp')) as image_count
         FROM emails e WHERE {}
         ORDER BY date_sent_utc DESC LIMIT 1000",
        conditions.join(" AND ")
    );

    let mut stmt = db.conn.prepare(&sql).map_err(|e| e.to_string())?;
    let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();

    let emails = stmt.query_map(param_refs.as_slice(), |row| {
        Ok(EmailMessage {
            id: row.get(0)?, evidence_id: row.get(1)?, case_id: row.get(2)?, message_id: row.get(3)?,
            from_addr: row.get(4)?, from_display: row.get(5)?, to_addrs: row.get(6)?, cc_addrs: row.get(7)?,
            subject: row.get(8)?, date_sent: row.get(9)?, date_sent_utc: row.get(10)?,
            folder_name: row.get(11)?, folder_category: row.get(12)?,
            is_deleted: boolv(row, 13), deleted_recovered: boolv(row, 14), risk_score: u8v(row, 15), flags: row.get(16)?,
            body_text: row.get(17)?,
            body_html: row.get(18)?,
            headers_raw: row.get(19)?,
            attachment_count: row.get::<_, Option<i64>>(20)?.unwrap_or(0) as u32,
            image_count: row.get::<_, Option<i64>>(21)?.unwrap_or(0) as u32,
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
pub async fn timeline_data(state: State<'_, AppState>, input: Value) -> Result<Value, String> {
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
        .filter(|s| !s.trim().is_empty() && *s != "all");

    let db = state.db.lock().await;

    let points = if let Some(ev_id) = evidence_id {
        let mut stmt = db.conn.prepare(
            "SELECT strftime('%Y-%m-%d', date_sent) as day, 
                    COUNT(*) as total_count,
                    SUM(CASE WHEN risk_score > 50 THEN 1 ELSE 0 END) as high_risk_count,
                    SUM(CASE WHEN is_deleted = 1 OR deleted_recovered = 1 THEN 1 ELSE 0 END) as deleted_count
             FROM emails 
             WHERE case_id = ?1 AND evidence_id = ?2 AND day IS NOT NULL 
             GROUP BY day 
             ORDER BY day ASC"
        ).map_err(|e| e.to_string())?;

        let r = stmt.query_map([&case_id, ev_id], |row| {
            Ok(serde_json::json!({
                "date": row.get::<_, String>(0)?,
                "total_count": row.get::<_, i64>(1)?,
                "high_risk_count": row.get::<_, i64>(2)?,
                "deleted_count": row.get::<_, i64>(3)?
            }))
        }).map_err(|e| e.to_string())?;
        r.filter_map(|row| row.ok()).collect::<Vec<_>>()
    } else {
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

        let r = stmt.query_map([&case_id], |row| {
            Ok(serde_json::json!({
                "date": row.get::<_, String>(0)?,
                "total_count": row.get::<_, i64>(1)?,
                "high_risk_count": row.get::<_, i64>(2)?,
                "deleted_count": row.get::<_, i64>(3)?
            }))
        }).map_err(|e| e.to_string())?;
        r.filter_map(|row| row.ok()).collect::<Vec<_>>()
    };

    Ok(serde_json::json!(points))
}

#[tauri::command]
pub async fn graph_data(state: State<'_, AppState>, input: Value) -> Result<Value, String> {
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
        .filter(|s| !s.trim().is_empty() && *s != "all");

    let db = state.db.lock().await;

    let rows = if let Some(ev_id) = evidence_id {
        let mut stmt = db.conn.prepare(
            "SELECT from_addr, to_addrs, risk_score FROM emails WHERE case_id = ?1 AND evidence_id = ?2 AND from_addr != ''"
        ).map_err(|e| e.to_string())?;
        let r = stmt.query_map([&case_id, ev_id], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, Option<i64>>(2)?.unwrap_or(0) as u8
            ))
        }).map_err(|e| e.to_string())?;
        r.filter_map(|row| row.ok()).collect::<Vec<_>>()
    } else {
        let mut stmt = db.conn.prepare(
            "SELECT from_addr, to_addrs, risk_score FROM emails WHERE case_id = ?1 AND from_addr != ''"
        ).map_err(|e| e.to_string())?;
        let r = stmt.query_map([&case_id], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, Option<i64>>(2)?.unwrap_or(0) as u8
            ))
        }).map_err(|e| e.to_string())?;
        r.filter_map(|row| row.ok()).collect::<Vec<_>>()
    };

    let mut node_set: HashSet<String> = HashSet::new();
    let mut node_risks: HashMap<String, u8> = HashMap::new();
    let mut node_sent: HashMap<String, u32> = HashMap::new();
    let mut node_recv: HashMap<String, u32> = HashMap::new();
    let mut edge_map: HashMap<(String, String), u32> = HashMap::new();

    for (from, to_json, risk) in rows {
        node_set.insert(from.clone());
        *node_sent.entry(from.clone()).or_insert(0) += 1;
        let current_risk = node_risks.entry(from.clone()).or_insert(0);
        if risk > *current_risk {
            *current_risk = risk;
        }

        let recipients: Vec<String> = serde_json::from_str(&to_json).unwrap_or_default();
        for r in recipients {
            if !r.is_empty() {
                node_set.insert(r.clone());
                *node_recv.entry(r.clone()).or_insert(0) += 1;
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
        let s = node_sent.get(&email).cloned().unwrap_or(0);
        let r = node_recv.get(&email).cloned().unwrap_or(0);
        serde_json::json!({
            "id": email,
            "label": email,
            "name": email,
            "risk_score": risk,
            "sent": s,
            "received": r,
            "total": s + r,
            "is_target": false
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
