use std::collections::HashMap;
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
use super::super::helpers::*;

#[tauri::command]
pub async fn custody_chain(state: State<'_, AppState>, input: EmptyInput) -> Result<Vec<CustodyEvent>, String> {
    let db = state.db.lock().await;
    let mut stmt = db.conn.prepare(
        "SELECT id, evidence_id, action, performed_by, timestamp, notes, 'J12 Email Forensic Suite' as tool, '1.0.0' as tool_version, NULL as hash_before, NULL as hash_after
         FROM chain_of_custody WHERE case_id = ?1
         UNION ALL
         SELECT ce.id, ce.evidence_id, ce.action, ce.actor as performed_by, ce.timestamp, ce.detail as notes, ce.tool, ce.tool_version, ce.hash_before, ce.hash_after
         FROM custody_events ce
         JOIN evidence_items ei ON ce.evidence_id = ei.id
         WHERE ei.case_id = ?1
         ORDER BY timestamp ASC"
    ).map_err(|e| e.to_string())?;
    let events = stmt.query_map([&input.case_id], |row| {
        Ok(CustodyEvent { 
            id: row.get(0)?, 
            evidence_id: row.get::<_, Option<String>>(1)?.unwrap_or_default(), 
            action: row.get(2)?, 
            actor: row.get::<_, Option<String>>(3)?.unwrap_or_else(|| "Examiner".to_string()), 
            timestamp: parse_dt(row.get::<_,String>(4)?.as_str()), 
            tool: row.get::<_, Option<String>>(6)?.unwrap_or_else(|| "J12 Email Forensic Suite".to_string()),
            tool_version: row.get::<_, Option<String>>(7)?.unwrap_or_else(|| "1.0.0".to_string()),
            hash_before: row.get(8)?,
            hash_after: row.get(9)?,
            detail: row.get(5)?,
        })
    }).map_err(|e| e.to_string())?.collect::<Result<Vec<_>,_>>().map_err(|e| e.to_string())?;
    Ok(events)
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
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty() && s != "all");

    let db = state.db.lock().await;

    let (te, de, he, ta, tf, cf, hf, mf, lf, inbox_c, important_c, sent_c, drafts_c, spam_c, other_c, top_correspondents, entity_c, ev_count) = if let Some(ref ev_id) = evidence_id {
        let te: i64 = db.conn.query_row("SELECT COUNT(*) FROM emails WHERE case_id=?1 AND evidence_id=?2", rusqlite::params![&cid, ev_id], |r| r.get(0)).unwrap_or(0);
        let de: i64 = db.conn.query_row("SELECT COUNT(*) FROM emails WHERE case_id=?1 AND evidence_id=?2 AND (is_deleted=1 OR deleted_recovered=1)", rusqlite::params![&cid, ev_id], |r| r.get(0)).unwrap_or(0);
        let he: i64 = db.conn.query_row("SELECT COUNT(*) FROM emails WHERE case_id=?1 AND evidence_id=?2 AND risk_score>50", rusqlite::params![&cid, ev_id], |r| r.get(0)).unwrap_or(0);
        let ta: i64 = db.conn.query_row("SELECT COUNT(*) FROM attachments WHERE email_id IN (SELECT id FROM emails WHERE case_id=?1 AND evidence_id=?2)", rusqlite::params![&cid, ev_id], |r| r.get(0)).unwrap_or(0);
        
        let tf: i64 = db.conn.query_row(
            "SELECT COUNT(*) FROM findings WHERE case_id=?1 AND (evidence_refs LIKE '%' || ?2 || '%' OR EXISTS (SELECT 1 FROM emails e WHERE e.case_id=?1 AND e.evidence_id=?2 AND instr(findings.email_ids, e.id)>0))",
            rusqlite::params![&cid, ev_id],
            |r| r.get(0)
        ).unwrap_or(0);

        let cf: u32 = db.conn.query_row(
            "SELECT COUNT(*) FROM findings WHERE case_id=?1 AND severity='critical' AND (evidence_refs LIKE '%' || ?2 || '%' OR EXISTS (SELECT 1 FROM emails e WHERE e.case_id=?1 AND e.evidence_id=?2 AND instr(findings.email_ids, e.id)>0))",
            rusqlite::params![&cid, ev_id],
            |r| r.get(0)
        ).unwrap_or(0);

        let hf: u32 = db.conn.query_row(
            "SELECT COUNT(*) FROM findings WHERE case_id=?1 AND severity='high' AND (evidence_refs LIKE '%' || ?2 || '%' OR EXISTS (SELECT 1 FROM emails e WHERE e.case_id=?1 AND e.evidence_id=?2 AND instr(findings.email_ids, e.id)>0))",
            rusqlite::params![&cid, ev_id],
            |r| r.get(0)
        ).unwrap_or(0);

        let mf: u32 = db.conn.query_row(
            "SELECT COUNT(*) FROM findings WHERE case_id=?1 AND severity='medium' AND (evidence_refs LIKE '%' || ?2 || '%' OR EXISTS (SELECT 1 FROM emails e WHERE e.case_id=?1 AND e.evidence_id=?2 AND instr(findings.email_ids, e.id)>0))",
            rusqlite::params![&cid, ev_id],
            |r| r.get(0)
        ).unwrap_or(0);

        let lf: u32 = db.conn.query_row(
            "SELECT COUNT(*) FROM findings WHERE case_id=?1 AND severity='low' AND (evidence_refs LIKE '%' || ?2 || '%' OR EXISTS (SELECT 1 FROM emails e WHERE e.case_id=?1 AND e.evidence_id=?2 AND instr(findings.email_ids, e.id)>0))",
            rusqlite::params![&cid, ev_id],
            |r| r.get(0)
        ).unwrap_or(0);

        let mut stmt = db.conn.prepare("SELECT from_addr, COUNT(*) as cnt FROM emails WHERE case_id=?1 AND evidence_id=?2 AND from_addr != '' GROUP BY from_addr ORDER BY cnt DESC LIMIT 5").map_err(|e| e.to_string())?;
        let top_corr = stmt.query_map(rusqlite::params![&cid, ev_id], |row| {
            let count: i64 = row.get(1)?;
            Ok(TopCorrespondent { email: row.get(0)?, sent: count as u32, received: 0 })
        }).map_err(|e| e.to_string())?.collect::<Result<Vec<_>,_>>().map_err(|e| e.to_string())?;

        let inbox_c: i64 = db.conn.query_row("SELECT COUNT(*) FROM emails WHERE case_id=?1 AND evidence_id=?2 AND folder_category='inbox'", rusqlite::params![&cid, ev_id], |r| r.get(0)).unwrap_or(0);
        let important_c: i64 = db.conn.query_row(
            "SELECT COUNT(*) FROM emails WHERE case_id=?1 AND evidence_id=?2 AND (folder_category='important' OR folder_name LIKE '%important%' OR flags LIKE '%important%')",
            rusqlite::params![&cid, ev_id],
            |r| r.get(0)
        ).unwrap_or(0);
        let sent_c: i64 = db.conn.query_row("SELECT COUNT(*) FROM emails WHERE case_id=?1 AND evidence_id=?2 AND folder_category='sent'", rusqlite::params![&cid, ev_id], |r| r.get(0)).unwrap_or(0);
        let drafts_c: i64 = db.conn.query_row("SELECT COUNT(*) FROM emails WHERE case_id=?1 AND evidence_id=?2 AND folder_category='drafts'", rusqlite::params![&cid, ev_id], |r| r.get(0)).unwrap_or(0);
        let spam_c: i64 = db.conn.query_row("SELECT COUNT(*) FROM emails WHERE case_id=?1 AND evidence_id=?2 AND folder_category='spam'", rusqlite::params![&cid, ev_id], |r| r.get(0)).unwrap_or(0);
        let other_c: i64 = db.conn.query_row("SELECT COUNT(*) FROM emails WHERE case_id=?1 AND evidence_id=?2 AND folder_category NOT IN ('inbox', 'important', 'sent', 'drafts', 'spam', 'trash', 'soft_deleted')", rusqlite::params![&cid, ev_id], |r| r.get(0)).unwrap_or(0);
        let entity_c: i64 = db.conn.query_row("SELECT COUNT(DISTINCT from_addr) FROM emails WHERE case_id=?1 AND evidence_id=?2 AND from_addr != ''", rusqlite::params![&cid, ev_id], |r| r.get(0)).unwrap_or(0);
        let ev_count: u32 = 1;

        (te, de, he, ta, tf, cf, hf, mf, lf, inbox_c, important_c, sent_c, drafts_c, spam_c, other_c, top_corr, entity_c, ev_count)
    } else {
        let te: i64 = db.conn.query_row("SELECT COUNT(*) FROM emails WHERE case_id=?1", [&cid], |r| r.get(0)).unwrap_or(0);
        let de: i64 = db.conn.query_row("SELECT COUNT(*) FROM emails WHERE case_id=?1 AND (is_deleted=1 OR deleted_recovered=1)", [&cid], |r| r.get(0)).unwrap_or(0);
        let he: i64 = db.conn.query_row("SELECT COUNT(*) FROM emails WHERE case_id=?1 AND risk_score>50", [&cid], |r| r.get(0)).unwrap_or(0);
        let ta: i64 = db.conn.query_row("SELECT COUNT(*) FROM attachments WHERE email_id IN (SELECT id FROM emails WHERE case_id=?1)", [&cid], |r| r.get(0)).unwrap_or(0);
        let tf: i64 = db.conn.query_row("SELECT COUNT(*) FROM findings WHERE case_id=?1", [&cid], |r| r.get(0)).unwrap_or(0);

        let cf: u32 = db.conn.query_row("SELECT COUNT(*) FROM findings WHERE case_id=?1 AND severity='critical'", [&cid], |r| r.get(0)).unwrap_or(0);
        let hf: u32 = db.conn.query_row("SELECT COUNT(*) FROM findings WHERE case_id=?1 AND severity='high'", [&cid], |r| r.get(0)).unwrap_or(0);
        let mf: u32 = db.conn.query_row("SELECT COUNT(*) FROM findings WHERE case_id=?1 AND severity='medium'", [&cid], |r| r.get(0)).unwrap_or(0);
        let lf: u32 = db.conn.query_row("SELECT COUNT(*) FROM findings WHERE case_id=?1 AND severity='low'", [&cid], |r| r.get(0)).unwrap_or(0);

        let mut stmt = db.conn.prepare("SELECT from_addr, COUNT(*) as cnt FROM emails WHERE case_id=?1 AND from_addr != '' GROUP BY from_addr ORDER BY cnt DESC LIMIT 5").map_err(|e| e.to_string())?;
        let top_corr = stmt.query_map([&cid], |row| {
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
        let entity_c: i64 = db.conn.query_row("SELECT COUNT(DISTINCT from_addr) FROM emails WHERE case_id=?1 AND from_addr != ''", [&cid], |r| r.get(0)).unwrap_or(0);
        let ev_count: u32 = db.conn.query_row("SELECT COUNT(*) FROM evidence_items WHERE case_id=?1", [&cid], |r| r.get(0)).unwrap_or(0);

        (te, de, he, ta, tf, cf, hf, mf, lf, inbox_c, important_c, sent_c, drafts_c, spam_c, other_c, top_corr, entity_c, ev_count)
    };

    let mut severity_map = HashMap::new();
    severity_map.insert("critical".to_string(), cf);
    severity_map.insert("high".to_string(), hf);
    severity_map.insert("medium".to_string(), mf);
    severity_map.insert("low".to_string(), lf);

    Ok(DashboardData {
        evidence_count: ev_count,
        email_count: te as u32,
        deleted_recovered: de as u32,
        entity_count: entity_c as u32,
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

        tx.execute(
            "DELETE FROM findings WHERE case_id = ?1 AND status NOT IN ('confirmed', 'dismissed', 'reviewed', 'manual')",
            [&case_id],
        ).map_err(|e| e.to_string())?;

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

    crate::audit_logger::log_forensic_event(
        &case_id,
        "SECURITY_ANALYSIS",
        "THREAT_ANALYSIS_EXECUTED",
        "System Analyzer",
        None,
        None,
        &format!("Analyzed {} emails and generated {} security findings & risk scores", emails.len(), findings_count)
    );

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
