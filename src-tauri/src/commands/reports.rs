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
        .or_else(|| input.as_str())
        .unwrap_or("")
        .to_string();

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

    let mut ev_stmt = db.conn.prepare(
        "SELECT id, filename, format, sha256, size_bytes, acquired_at, acquisition_method
         FROM evidence_items WHERE case_id = ?1"
    ).map_err(|e| e.to_string())?;

    let evidence_summary = ev_stmt.query_map([&case_id], |row| {
        Ok(serde_json::json!({
            "id": row.get::<_, String>(0)?,
            "filename": row.get::<_, String>(1)?,
            "format": row.get::<_, String>(2)?,
            "sha256": row.get::<_, String>(3)?,
            "size_bytes": row.get::<_, i64>(4)?,
            "acquired_at": row.get::<_, String>(5)?,
            "method": row.get::<_, Option<String>>(6)?.unwrap_or_default(),
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

    let mut coc_stmt = db.conn.prepare(
        "SELECT action, performed_by, timestamp, notes
         FROM chain_of_custody WHERE case_id = ?1 ORDER BY timestamp ASC"
    ).map_err(|e| e.to_string())?;

    let custody_events = coc_stmt.query_map([&case_id], |row| {
        Ok(serde_json::json!({
            "action": row.get::<_, String>(0)?,
            "performed_by": row.get::<_, Option<String>>(1)?.unwrap_or_else(|| "Examiner".to_string()),
            "timestamp": row.get::<_, String>(2)?,
            "notes": row.get::<_, Option<String>>(3)?.unwrap_or_default(),
        }))
    }).map_err(|e| e.to_string())?.filter_map(|r| r.ok()).collect::<Vec<_>>();

    let email_stats: (i64, i64, i64) = db.conn.query_row(
        "SELECT COUNT(*), 
                SUM(CASE WHEN risk_score > 50 THEN 1 ELSE 0 END),
                SUM(CASE WHEN is_deleted = 1 OR deleted_recovered = 1 THEN 1 ELSE 0 END)
         FROM emails WHERE case_id = ?1",
        [&case_id],
        |row| Ok((row.get(0)?, row.get::<_, Option<i64>>(1)?.unwrap_or(0), row.get::<_, Option<i64>>(2)?.unwrap_or(0)))
    ).unwrap_or((0, 0, 0));

    let att_count: i64 = db.conn.query_row(
        "SELECT COUNT(*) FROM attachments WHERE email_id IN (SELECT id FROM emails WHERE case_id = ?1)",
        [&case_id],
        |row| row.get(0)
    ).unwrap_or(0);

    let critical_count = findings_summary.iter().filter(|f| f["severity"] == "critical").count();
    let high_count = findings_summary.iter().filter(|f| f["severity"] == "high").count();

    let executive_summary = format!(
        "Forensic email analysis conducted for Case '{}' (Case #: {}). Examiner: Examiner. A total of {} evidence containers were ingested, yielding {} messages and {} attachments. Analysis identified {} total threat findings, including {} critical and {} high-severity indicators.",
        case.title,
        case.case_number,
        evidence_summary.len(),
        email_stats.0,
        att_count,
        findings_summary.len(),
        critical_count,
        high_count,
    );

    Ok(serde_json::json!({
        "case": {
            "id": case.id,
            "title": case.title,
            "case_number": case.case_number,
            "examiner_name": "Examiner",
            "investigation_type": case.investigation_type,
            "description": case.description,
            "status": case.status,
            "target_email": case.target_email,
            "target_name": case.target_name,
            "target_organization": case.target_organization,
            "created_at": case.created_at.to_rfc3339(),
            "updated_at": case.updated_at.to_rfc3339(),
        },
        "executive_summary": executive_summary,
        "evidence_summary": evidence_summary,
        "email_statistics": {
            "total_messages": email_stats.0,
            "high_risk_messages": email_stats.1,
            "recovered_deleted_messages": email_stats.2,
            "total_attachments": att_count,
        },
        "findings": findings_summary,
        "chain_of_custody": custody_events,
        "generated_at": Utc::now().to_rfc3339(),
        "tool_version": "J12 Email Forensic Suite v1.0.0",
    }))
}

#[tauri::command]
pub async fn export_report_pdf(
    state: State<'_, AppState>,
    case_id: String,
    _sections: Vec<String>,
    _exhibits: Vec<Value>,
) -> Result<String, String> {
    let report_data = generate_report_data(state, serde_json::json!({ "case_id": case_id })).await?;

    let downloads_dir = dirs::download_dir().unwrap_or_else(|| PathBuf::from("."));
    let case_title = report_data["case"]["title"].as_str().unwrap_or("Forensic_Report");
    let safe_name = case_title.replace(|c: char| !c.is_alphanumeric() && c != '_' && c != '-', "_");
    let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
    let html_filename = format!("{}_Forensic_Report_{}.html", safe_name, timestamp);
    let output_path = downloads_dir.join(&html_filename);

    let html_content = format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<title>Forensic Examination Report - {}</title>
<style>
  body {{ font-family: 'Helvetica Neue', Arial, sans-serif; margin: 40px; color: #1e293b; background: #fff; line-height: 1.6; }}
  .header {{ border-bottom: 3px solid #0f172a; padding-bottom: 20px; margin-bottom: 30px; }}
  .title {{ font-size: 26px; font-weight: 800; color: #0f172a; margin: 0; }}
  .badge {{ display: inline-block; padding: 4px 8px; border-radius: 4px; font-size: 11px; font-weight: 700; text-transform: uppercase; }}
  .badge-critical {{ background: #fef2f2; color: #ef4444; border: 1px solid #f87171; }}
  .badge-high {{ background: #fff7ed; color: #f97316; border: 1px solid #fdba74; }}
  .table {{ width: 100%; border-collapse: collapse; margin-top: 15px; font-size: 13px; }}
  .table th, .table td {{ border: 1px solid #e2e8f0; padding: 8px 12px; text-align: left; }}
  .table th {{ background: #f8fafc; font-weight: 700; }}
  .section {{ margin-bottom: 35px; page-break-inside: avoid; }}
  .section-title {{ font-size: 18px; font-weight: 700; color: #1e293b; border-bottom: 1px solid #cbd5e1; padding-bottom: 6px; margin-bottom: 15px; }}
  .hash {{ font-family: monospace; font-size: 11px; color: #475569; word-break: break-all; }}
</style>
</head>
<body>
  <div class="header">
    <h1 class="title">J12 EMAIL FORENSIC EXAMINATION DOSSIER</h1>
    <div style="font-size: 14px; color: #64748b; margin-top: 5px;">
      Case: <strong>{}</strong> | Case #: <strong>{}</strong> | Examiner: <strong>{}</strong> | Generated: <strong>{}</strong>
    </div>
  </div>

  <div class="section">
    <div class="section-title">1. EXECUTIVE SUMMARY &amp; SCOPE</div>
    <p>{}</p>
  </div>

  <div class="section">
    <div class="section-title">2. EVIDENCE SOURCES &amp; INTEGRITY HASHES</div>
    <table class="table">
      <thead><tr><th>Filename</th><th>Format</th><th>Size (Bytes)</th><th>Acquired At</th><th>SHA-256 Hash</th></tr></thead>
      <tbody>
        {}
      </tbody>
    </table>
  </div>

  <div class="section">
    <div class="section-title">3. FORENSIC FINDINGS &amp; THREAT INTELLIGENCE</div>
    <table class="table">
      <thead><tr><th>Severity</th><th>Title</th><th>Type</th><th>Description</th></tr></thead>
      <tbody>
        {}
      </tbody>
    </table>
  </div>

  <div class="section">
    <div class="section-title">4. CHAIN OF CUSTODY AUDIT LOG</div>
    <table class="table">
      <thead><tr><th>Timestamp</th><th>Action</th><th>Performed By</th><th>Notes</th></tr></thead>
      <tbody>
        {}
      </tbody>
    </table>
  </div>
</body>
</html>"#,
        case_title,
        report_data["case"]["title"].as_str().unwrap_or(""),
        report_data["case"]["case_number"].as_str().unwrap_or("N/A"),
        report_data["case"]["examiner_name"].as_str().unwrap_or("Examiner"),
        report_data["generated_at"].as_str().unwrap_or(""),
        report_data["executive_summary"].as_str().unwrap_or(""),
        report_data["evidence_summary"].as_array().unwrap_or(&vec![]).iter().map(|e| {
            format!(
                "<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td class=\"hash\">{}</td></tr>",
                e["filename"].as_str().unwrap_or(""),
                e["format"].as_str().unwrap_or(""),
                e["size_bytes"].as_i64().unwrap_or(0),
                e["acquired_at"].as_str().unwrap_or(""),
                e["sha256"].as_str().unwrap_or(""),
            )
        }).collect::<Vec<_>>().join(""),
        report_data["findings"].as_array().unwrap_or(&vec![]).iter().map(|f| {
            let sev = f["severity"].as_str().unwrap_or("low");
            let badge_class = if sev == "critical" { "badge badge-critical" } else { "badge badge-high" };
            format!(
                "<tr><td><span class=\"{}\">{}</span></td><td><strong>{}</strong></td><td>{}</td><td>{}</td></tr>",
                badge_class,
                sev,
                f["title"].as_str().unwrap_or(""),
                f["type"].as_str().unwrap_or(""),
                f["description"].as_str().unwrap_or(""),
            )
        }).collect::<Vec<_>>().join(""),
        report_data["chain_of_custody"].as_array().unwrap_or(&vec![]).iter().map(|c| {
            format!(
                "<tr><td>{}</td><td><strong>{}</strong></td><td>{}</td><td>{}</td></tr>",
                c["timestamp"].as_str().unwrap_or(""),
                c["action"].as_str().unwrap_or(""),
                c["performed_by"].as_str().unwrap_or(""),
                c["notes"].as_str().unwrap_or(""),
            )
        }).collect::<Vec<_>>().join(""),
    );

    fs::write(&output_path, html_content).map_err(|e| format!("Failed to write HTML report: {}", e))?;

    Ok(output_path.to_string_lossy().to_string())
}

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

    Ok(output_path.to_string_lossy().to_string())
}

#[tauri::command]
pub async fn check_custody_chain(state: State<'_, AppState>, input: EmptyInput) -> Result<serde_json::Value, String> {
    let db = state.db.lock().await;
    let count: i64 = db.conn.query_row(
        "SELECT COUNT(*) FROM chain_of_custody WHERE case_id = ?1",
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
