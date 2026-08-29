use std::fs;
use std::path::PathBuf;
use chrono::Utc;
use serde_json::Value;
use tauri::State;

use crate::AppState;
use crate::db::parse_dt;
use crate::models::*;

#[tauri::command]
pub async fn generate_report_data(state: State<'_, AppState>, input: Value) -> Result<Value, String> {
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

    let case: Case = db.conn.query_row(
        "SELECT id, title, case_number, description, status,
                target_email, target_name, target_organization, investigation_type, working_dir, created_at, updated_at
         FROM cases WHERE id = ?1",
        [&case_id],
        |row| {
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
                created_at: parse_dt(&row.get::<_, String>(10)?),
                updated_at: parse_dt(&row.get::<_, String>(11)?),
            })
        },
    ).map_err(|e| format!("Case not found: {}", e))?;

    let (evidence_summary, findings_summary, folder_breakdown, key_messages_ledger, attachments_manifest, email_stats, att_count, executive_summary) = if let Some(ref ev_id) = evidence_id {
        let mut ev_stmt = db.conn.prepare(
            "SELECT id, filename, format, sha256, size_bytes, acquired_at, acquisition_method, message_count
             FROM evidence_items WHERE case_id = ?1 AND id = ?2"
        ).map_err(|e| e.to_string())?;

        let evidence_summary = ev_stmt.query_map(rusqlite::params![&case_id, ev_id], |row| {
            let msg_count: i64 = row.get::<_, Option<i64>>(7)?.unwrap_or(0);
            Ok(serde_json::json!({
                "id": row.get::<_, String>(0)?,
                "filename": row.get::<_, String>(1)?,
                "format": row.get::<_, String>(2)?,
                "sha256": row.get::<_, String>(3)?,
                "size_bytes": row.get::<_, i64>(4)?,
                "acquired_at": row.get::<_, String>(5)?,
                "method": row.get::<_, Option<String>>(6)?.unwrap_or_default(),
                "message_count": msg_count,
            }))
        }).map_err(|e| e.to_string())?.filter_map(|r| r.ok()).collect::<Vec<_>>();

        let mut f_stmt = db.conn.prepare(
            "SELECT id, type, severity, confidence, title, description, created_at, status, notes
             FROM findings 
             WHERE case_id = ?1 
               AND (
                 evidence_refs LIKE '%' || ?2 || '%' 
                 OR EXISTS (
                   SELECT 1 FROM emails e 
                   WHERE e.case_id = ?1 AND e.evidence_id = ?2 
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
               END"
        ).map_err(|e| e.to_string())?;

        let findings_summary = f_stmt.query_map(rusqlite::params![&case_id, ev_id], |row| {
            Ok(serde_json::json!({
                "id": row.get::<_, String>(0)?,
                "type": row.get::<_, String>(1)?,
                "severity": row.get::<_, String>(2)?,
                "confidence": row.get::<_, Option<String>>(3)?.unwrap_or_else(|| "0.85".to_string()),
                "title": row.get::<_, String>(4)?,
                "description": row.get::<_, String>(5)?,
                "created_at": row.get::<_, String>(6)?,
                "status": row.get::<_, String>(7)?,
                "notes": row.get::<_, Option<String>>(8)?.unwrap_or_default(),
            }))
        }).map_err(|e| e.to_string())?.filter_map(|r| r.ok()).collect::<Vec<_>>();

        let mut folder_stmt = db.conn.prepare(
            "SELECT folder_name, folder_category, COUNT(*), MIN(date_sent_utc), MAX(date_sent_utc)
             FROM emails WHERE case_id = ?1 AND evidence_id = ?2
             GROUP BY folder_name, folder_category
             ORDER BY COUNT(*) DESC"
        ).map_err(|e| e.to_string())?;

        let folder_breakdown = folder_stmt.query_map(rusqlite::params![&case_id, ev_id], |row| {
            Ok(serde_json::json!({
                "folder_name": row.get::<_, Option<String>>(0)?.unwrap_or_else(|| "Inbox".to_string()),
                "folder_category": row.get::<_, Option<String>>(1)?.unwrap_or_else(|| "inbox".to_string()),
                "count": row.get::<_, i64>(2)?,
                "date_from": row.get::<_, Option<String>>(3)?,
                "date_to": row.get::<_, Option<String>>(4)?,
            }))
        }).map_err(|e| e.to_string())?.filter_map(|r| r.ok()).collect::<Vec<_>>();

        let mut ledger_stmt = db.conn.prepare(
            "SELECT id, from_addr, from_display, to_addrs, subject, date_sent_utc, risk_score, folder_category, is_deleted, deleted_recovered
             FROM emails WHERE case_id = ?1 AND evidence_id = ?2 AND (risk_score > 30 OR is_deleted = 1 OR deleted_recovered = 1)
             ORDER BY risk_score DESC, date_sent_utc DESC
             LIMIT 100"
        ).map_err(|e| e.to_string())?;

        let key_messages_ledger = ledger_stmt.query_map(rusqlite::params![&case_id, ev_id], |row| {
            Ok(serde_json::json!({
                "id": row.get::<_, String>(0)?,
                "from_addr": row.get::<_, String>(1)?,
                "from_display": row.get::<_, Option<String>>(2)?,
                "to_addrs": row.get::<_, String>(3)?,
                "subject": row.get::<_, Option<String>>(4)?.unwrap_or_else(|| "(No Subject)".to_string()),
                "date_sent_utc": row.get::<_, Option<String>>(5)?,
                "risk_score": row.get::<_, i64>(6)?,
                "folder_category": row.get::<_, Option<String>>(7)?.unwrap_or_else(|| "inbox".to_string()),
                "is_deleted": row.get::<_, Option<bool>>(8)?.unwrap_or(false),
                "deleted_recovered": row.get::<_, Option<bool>>(9)?.unwrap_or(false),
            }))
        }).map_err(|e| e.to_string())?.filter_map(|r| r.ok()).collect::<Vec<_>>();

        let mut att_stmt = db.conn.prepare(
            "SELECT a.id, a.filename, a.sha256, a.mime_type, a.size_bytes, a.entropy, a.risk_flags, e.subject, e.from_addr
             FROM attachments a
             JOIN emails e ON a.email_id = e.id
             WHERE e.case_id = ?1 AND e.evidence_id = ?2
             ORDER BY a.size_bytes DESC
             LIMIT 100"
        ).map_err(|e| e.to_string())?;

        let attachments_manifest = att_stmt.query_map(rusqlite::params![&case_id, ev_id], |row| {
            Ok(serde_json::json!({
                "id": row.get::<_, String>(0)?,
                "filename": row.get::<_, Option<String>>(1)?.unwrap_or_else(|| "attachment.bin".to_string()),
                "sha256": row.get::<_, String>(2)?,
                "mime_type": row.get::<_, Option<String>>(3)?.unwrap_or_default(),
                "size_bytes": row.get::<_, i64>(4)?,
                "entropy": row.get::<_, Option<f64>>(5)?,
                "risk_flags": row.get::<_, Option<String>>(6)?,
                "email_subject": row.get::<_, Option<String>>(7)?,
                "email_from": row.get::<_, Option<String>>(8)?,
            }))
        }).map_err(|e| e.to_string())?.filter_map(|r| r.ok()).collect::<Vec<_>>();

        let email_stats: (i64, i64, i64, i64, i64, i64) = db.conn.query_row(
            "SELECT COUNT(*), 
                    SUM(CASE WHEN folder_category = 'inbox' THEN 1 ELSE 0 END),
                    SUM(CASE WHEN folder_category = 'sent' THEN 1 ELSE 0 END),
                    SUM(CASE WHEN folder_category = 'deleted' OR is_deleted = 1 OR deleted_recovered = 1 THEN 1 ELSE 0 END),
                    SUM(CASE WHEN folder_category = 'spam' THEN 1 ELSE 0 END),
                    SUM(CASE WHEN risk_score > 40 THEN 1 ELSE 0 END)
             FROM emails WHERE case_id = ?1 AND evidence_id = ?2",
            rusqlite::params![&case_id, ev_id],
            |row| Ok((
                row.get(0)?,
                row.get::<_, Option<i64>>(1)?.unwrap_or(0),
                row.get::<_, Option<i64>>(2)?.unwrap_or(0),
                row.get::<_, Option<i64>>(3)?.unwrap_or(0),
                row.get::<_, Option<i64>>(4)?.unwrap_or(0),
                row.get::<_, Option<i64>>(5)?.unwrap_or(0),
            ))
        ).unwrap_or((0, 0, 0, 0, 0, 0));

        let att_count: i64 = db.conn.query_row(
            "SELECT COUNT(*) FROM attachments WHERE email_id IN (SELECT id FROM emails WHERE case_id = ?1 AND evidence_id = ?2)",
            rusqlite::params![&case_id, ev_id],
            |row| row.get(0)
        ).unwrap_or(0);

        let critical_count = findings_summary.iter().filter(|f| f["severity"] == "critical").count();
        let high_count = findings_summary.iter().filter(|f| f["severity"] == "high").count();

        let target_filename = evidence_summary.first()
            .and_then(|e| e["filename"].as_str())
            .unwrap_or("Selected Source");

        let executive_summary = format!(
            "Forensic email analysis conducted for Case '{}' (Case Reference: {}) scoped specifically to Evidence Container '{}'. Processing of this evidence container yielded {} messages, {} attachments, and {} folder structures. Automated inspection identified {} security threat findings ({} critical, {} high-priority) and {} key evidentiary items entered into the forensic ledger.",
            case.title,
            case.case_number,
            target_filename,
            email_stats.0,
            att_count,
            folder_breakdown.len(),
            findings_summary.len(),
            critical_count,
            high_count,
            key_messages_ledger.len(),
        );

        (evidence_summary, findings_summary, folder_breakdown, key_messages_ledger, attachments_manifest, email_stats, att_count, executive_summary)
    } else {
        let mut ev_stmt = db.conn.prepare(
            "SELECT id, filename, format, sha256, size_bytes, acquired_at, acquisition_method, message_count
             FROM evidence_items WHERE case_id = ?1"
        ).map_err(|e| e.to_string())?;

        let evidence_summary = ev_stmt.query_map([&case_id], |row| {
            let msg_count: i64 = row.get::<_, Option<i64>>(7)?.unwrap_or(0);
            Ok(serde_json::json!({
                "id": row.get::<_, String>(0)?,
                "filename": row.get::<_, String>(1)?,
                "format": row.get::<_, String>(2)?,
                "sha256": row.get::<_, String>(3)?,
                "size_bytes": row.get::<_, i64>(4)?,
                "acquired_at": row.get::<_, String>(5)?,
                "method": row.get::<_, Option<String>>(6)?.unwrap_or_default(),
                "message_count": msg_count,
            }))
        }).map_err(|e| e.to_string())?.filter_map(|r| r.ok()).collect::<Vec<_>>();

        let mut f_stmt = db.conn.prepare(
            "SELECT id, type, severity, confidence, title, description, created_at, status, notes
             FROM findings WHERE case_id = ?1 ORDER BY 
               CASE severity 
                 WHEN 'critical' THEN 1 
                 WHEN 'high' THEN 2 
                 WHEN 'medium' THEN 3 
                 WHEN 'low' THEN 4 
                 ELSE 5 
               END"
        ).map_err(|e| e.to_string())?;

        let findings_summary = f_stmt.query_map([&case_id], |row| {
            Ok(serde_json::json!({
                "id": row.get::<_, String>(0)?,
                "type": row.get::<_, String>(1)?,
                "severity": row.get::<_, String>(2)?,
                "confidence": row.get::<_, Option<String>>(3)?.unwrap_or_else(|| "0.85".to_string()),
                "title": row.get::<_, String>(4)?,
                "description": row.get::<_, String>(5)?,
                "created_at": row.get::<_, String>(6)?,
                "status": row.get::<_, String>(7)?,
                "notes": row.get::<_, Option<String>>(8)?.unwrap_or_default(),
            }))
        }).map_err(|e| e.to_string())?.filter_map(|r| r.ok()).collect::<Vec<_>>();

        let mut folder_stmt = db.conn.prepare(
            "SELECT folder_name, folder_category, COUNT(*), MIN(date_sent_utc), MAX(date_sent_utc)
             FROM emails WHERE case_id = ?1
             GROUP BY folder_name, folder_category
             ORDER BY COUNT(*) DESC"
        ).map_err(|e| e.to_string())?;

        let folder_breakdown = folder_stmt.query_map([&case_id], |row| {
            Ok(serde_json::json!({
                "folder_name": row.get::<_, Option<String>>(0)?.unwrap_or_else(|| "Inbox".to_string()),
                "folder_category": row.get::<_, Option<String>>(1)?.unwrap_or_else(|| "inbox".to_string()),
                "count": row.get::<_, i64>(2)?,
                "date_from": row.get::<_, Option<String>>(3)?,
                "date_to": row.get::<_, Option<String>>(4)?,
            }))
        }).map_err(|e| e.to_string())?.filter_map(|r| r.ok()).collect::<Vec<_>>();

        let mut ledger_stmt = db.conn.prepare(
            "SELECT id, from_addr, from_display, to_addrs, subject, date_sent_utc, risk_score, folder_category, is_deleted, deleted_recovered
             FROM emails WHERE case_id = ?1 AND (risk_score > 30 OR is_deleted = 1 OR deleted_recovered = 1)
             ORDER BY risk_score DESC, date_sent_utc DESC
             LIMIT 100"
        ).map_err(|e| e.to_string())?;

        let key_messages_ledger = ledger_stmt.query_map([&case_id], |row| {
            Ok(serde_json::json!({
                "id": row.get::<_, String>(0)?,
                "from_addr": row.get::<_, String>(1)?,
                "from_display": row.get::<_, Option<String>>(2)?,
                "to_addrs": row.get::<_, String>(3)?,
                "subject": row.get::<_, Option<String>>(4)?.unwrap_or_else(|| "(No Subject)".to_string()),
                "date_sent_utc": row.get::<_, Option<String>>(5)?,
                "risk_score": row.get::<_, i64>(6)?,
                "folder_category": row.get::<_, Option<String>>(7)?.unwrap_or_else(|| "inbox".to_string()),
                "is_deleted": row.get::<_, Option<bool>>(8)?.unwrap_or(false),
                "deleted_recovered": row.get::<_, Option<bool>>(9)?.unwrap_or(false),
            }))
        }).map_err(|e| e.to_string())?.filter_map(|r| r.ok()).collect::<Vec<_>>();

        let mut att_stmt = db.conn.prepare(
            "SELECT a.id, a.filename, a.sha256, a.mime_type, a.size_bytes, a.entropy, a.risk_flags, e.subject, e.from_addr
             FROM attachments a
             JOIN emails e ON a.email_id = e.id
             WHERE e.case_id = ?1
             ORDER BY a.size_bytes DESC
             LIMIT 100"
        ).map_err(|e| e.to_string())?;

        let attachments_manifest = att_stmt.query_map([&case_id], |row| {
            Ok(serde_json::json!({
                "id": row.get::<_, String>(0)?,
                "filename": row.get::<_, Option<String>>(1)?.unwrap_or_else(|| "attachment.bin".to_string()),
                "sha256": row.get::<_, String>(2)?,
                "mime_type": row.get::<_, Option<String>>(3)?.unwrap_or_default(),
                "size_bytes": row.get::<_, i64>(4)?,
                "entropy": row.get::<_, Option<f64>>(5)?,
                "risk_flags": row.get::<_, Option<String>>(6)?,
                "email_subject": row.get::<_, Option<String>>(7)?,
                "email_from": row.get::<_, Option<String>>(8)?,
            }))
        }).map_err(|e| e.to_string())?.filter_map(|r| r.ok()).collect::<Vec<_>>();

        let email_stats: (i64, i64, i64, i64, i64, i64) = db.conn.query_row(
            "SELECT COUNT(*), 
                    SUM(CASE WHEN folder_category = 'inbox' THEN 1 ELSE 0 END),
                    SUM(CASE WHEN folder_category = 'sent' THEN 1 ELSE 0 END),
                    SUM(CASE WHEN folder_category = 'deleted' OR is_deleted = 1 OR deleted_recovered = 1 THEN 1 ELSE 0 END),
                    SUM(CASE WHEN folder_category = 'spam' THEN 1 ELSE 0 END),
                    SUM(CASE WHEN risk_score > 40 THEN 1 ELSE 0 END)
             FROM emails WHERE case_id = ?1",
            [&case_id],
            |row| Ok((
                row.get(0)?,
                row.get::<_, Option<i64>>(1)?.unwrap_or(0),
                row.get::<_, Option<i64>>(2)?.unwrap_or(0),
                row.get::<_, Option<i64>>(3)?.unwrap_or(0),
                row.get::<_, Option<i64>>(4)?.unwrap_or(0),
                row.get::<_, Option<i64>>(5)?.unwrap_or(0),
            ))
        ).unwrap_or((0, 0, 0, 0, 0, 0));

        let att_count: i64 = db.conn.query_row(
            "SELECT COUNT(*) FROM attachments WHERE email_id IN (SELECT id FROM emails WHERE case_id = ?1)",
            [&case_id],
            |row| row.get(0)
        ).unwrap_or(0);

        let critical_count = findings_summary.iter().filter(|f| f["severity"] == "critical").count();
        let high_count = findings_summary.iter().filter(|f| f["severity"] == "high").count();

        let executive_summary = format!(
            "Forensic email analysis conducted for Case '{}' (Case Reference: {}). A total of {} forensic evidence source(s) were processed, yielding {} messages, {} attachments, and {} folder structures. Automated inspection identified {} security threat findings ({} critical, {} high-priority) and {} key evidentiary items entered into the forensic ledger.",
            case.title,
            case.case_number,
            evidence_summary.len(),
            email_stats.0,
            att_count,
            folder_breakdown.len(),
            findings_summary.len(),
            critical_count,
            high_count,
            key_messages_ledger.len(),
        );

        (evidence_summary, findings_summary, folder_breakdown, key_messages_ledger, attachments_manifest, email_stats, att_count, executive_summary)
    };

    let mut coc_stmt = db.conn.prepare(
        "SELECT action, performed_by, timestamp, notes
         FROM chain_of_custody WHERE case_id = ?1
         UNION ALL
         SELECT ce.action, ce.actor as performed_by, ce.timestamp, ce.detail as notes
         FROM custody_events ce
         JOIN evidence_items ei ON ce.evidence_id = ei.id
         WHERE ei.case_id = ?1
         ORDER BY timestamp ASC"
    ).map_err(|e| e.to_string())?;

    let custody_events = coc_stmt.query_map([&case_id], |row| {
        Ok(serde_json::json!({
            "action": row.get::<_, String>(0)?,
            "performed_by": row.get::<_, Option<String>>(1)?.unwrap_or_else(|| "Examiner".to_string()),
            "timestamp": row.get::<_, String>(2)?,
            "notes": row.get::<_, Option<String>>(3)?.unwrap_or_default(),
        }))
    }).map_err(|e| e.to_string())?.filter_map(|r| r.ok()).collect::<Vec<_>>();

    let case_obj = serde_json::json!({
        "id": case.id,
        "title": case.title,
        "case_number": case.case_number,
        "examiner_name": "Senior Forensic Examiner",
        "investigation_type": case.investigation_type,
        "description": case.description,
        "status": case.status,
        "target_email": case.target_email,
        "target_name": case.target_name,
        "target_organization": case.target_organization,
        "working_dir": case.working_dir,
        "created_at": case.created_at.to_rfc3339(),
        "updated_at": case.updated_at.to_rfc3339(),
    });

    let email_stats_obj = serde_json::json!({
        "total": email_stats.0,
        "inbox": email_stats.1,
        "sent": email_stats.2,
        "deleted": email_stats.3,
        "spam": email_stats.4,
        "high_risk": email_stats.5,
        "total_attachments": att_count,
    });

    Ok(serde_json::json!({
        "case_info": case_obj,
        "case": case_obj,
        "executive_summary": executive_summary,
        "evidence_inventory": evidence_summary,
        "evidence_summary": evidence_summary,
        "email_stats": email_stats_obj,
        "email_statistics": email_stats_obj,
        "folder_breakdown": folder_breakdown,
        "key_messages_ledger": key_messages_ledger,
        "attachments_manifest": attachments_manifest,
        "findings": findings_summary,
        "chain_of_custody": custody_events,
        "custody_chain": custody_events,
        "generated_at": Utc::now().to_rfc3339(),
        "tool_version": "J12 Email Forensic Suite v1.0.0 (Standards: NIST SP 800-86 / ISO/IEC 27037)",
    }))
}
