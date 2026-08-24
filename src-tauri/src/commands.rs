use serde::{Deserialize, Serialize};
use crate::models::*;
use crate::db::{compute_sha256, compute_sha512, detect_format, generate_id, parse_dt};
use crate::AppState;
use crate::analysis::{
    analyze_headers, analyze_authentication, detect_spoofing, detect_content_threats,
    analyze_attachment_metadata, generate_findings, calculate_risk_score, NewFinding
};
use std::path::PathBuf;
use std::fs;
use tauri::State;
use chrono::Utc;

fn i64v(row: &rusqlite::Row, i: usize) -> i64 { row.get::<_,i64>(i).unwrap_or(0) }
fn u64v(row: &rusqlite::Row, i: usize) -> u64 { row.get::<_,i64>(i).unwrap_or(0) as u64 }
fn u32v(row: &rusqlite::Row, i: usize) -> u32 { row.get::<_,i64>(i).unwrap_or(0) as u32 }
fn u8v(row: &rusqlite::Row, i: usize) -> u8 { row.get::<_,i64>(i).unwrap_or(0) as u8 }
fn boolv(row: &rusqlite::Row, i: usize) -> bool { row.get::<_,i64>(i).unwrap_or(0) != 0 }

#[tauri::command]
pub async fn case_create(state: State<'_, AppState>, input: CaseCreateInput) -> Result<Case, String> {
    let db = state.db.lock().await;
    let now = Utc::now();
    let id = generate_id();
    let cn = input.case_number.clone().unwrap_or_default();
    let desc = input.description.clone().unwrap_or_default();
    let target_email = input.target_email.clone().unwrap_or_default();
    let target_name = input.target_name.clone().unwrap_or_default();
    let target_org = input.target_organization.clone().unwrap_or_default();
    let inv_type = input.investigation_type.clone().unwrap_or_else(|| "general".to_string());
    db.conn.execute(
        "INSERT INTO cases (id,title,case_number,description,status,owner_id,target_email,target_name,target_organization,investigation_type,created_at,updated_at) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12)",
        (&id, &input.title, &cn, &desc, "open", "admin", &target_email, &target_name, &target_org, &inv_type, &now.to_rfc3339(), &now.to_rfc3339()),
    ).map_err(|e| e.to_string())?;
    Ok(Case { id, title: input.title, case_number: cn, description: desc, status: "open".to_string(), owner_id: "admin".to_string(), target_email: if target_email.is_empty() { None } else { Some(target_email) }, target_name: if target_name.is_empty() { None } else { Some(target_name) }, target_organization: if target_org.is_empty() { None } else { Some(target_org) }, investigation_type: inv_type, created_at: now, updated_at: now })
}

#[tauri::command]
pub async fn case_list(state: State<'_, AppState>) -> Result<Vec<Case>, String> {
    let db = state.db.lock().await;
    let mut stmt = db.conn.prepare("SELECT id,title,case_number,description,status,owner_id,target_email,target_name,target_organization,investigation_type,created_at,updated_at FROM cases ORDER BY created_at DESC").map_err(|e| e.to_string())?;
    let cases = stmt.query_map([], |row| {
        Ok(Case { id: row.get(0)?, title: row.get(1)?, case_number: row.get(2)?, description: row.get(3)?, status: row.get(4)?, owner_id: row.get(5)?, target_email: row.get(6)?, target_name: row.get(7)?, target_organization: row.get(8)?, investigation_type: row.get(9)?, created_at: parse_dt(&row.get::<_, String>(10)?), updated_at: parse_dt(&row.get::<_, String>(11)?) })
    }).map_err(|e| e.to_string())?.collect::<Result<Vec<_>,_>>().map_err(|e| e.to_string())?;
    Ok(cases)
}

#[tauri::command]
pub async fn case_get(state: State<'_, AppState>, input: EmptyInput) -> Result<Option<Case>, String> {
    let db = state.db.lock().await;
    let r = db.conn.query_row("SELECT id,title,case_number,description,status,owner_id,target_email,target_name,target_organization,investigation_type,created_at,updated_at FROM cases WHERE id=?1", [&input.case_id],
        |row| Ok(Case { id: row.get(0)?, title: row.get(1)?, case_number: row.get(2)?, description: row.get(3)?, status: row.get(4)?, owner_id: row.get(5)?, target_email: row.get(6)?, target_name: row.get(7)?, target_organization: row.get(8)?, investigation_type: row.get(9)?, created_at: parse_dt(&row.get::<_, String>(10)?), updated_at: parse_dt(&row.get::<_, String>(11)?) }));
    match r { Ok(c) => Ok(Some(c)), Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None), Err(e) => Err(e.to_string()) }
}

#[tauri::command]
pub async fn case_update(state: State<'_, AppState>, input: CaseUpdateInput) -> Result<(), String> {
    let db = state.db.lock().await;
    db.conn.execute(
        "UPDATE cases SET title=?1, description=?2, status=?3, target_name=?4, target_email=?5, target_organization=?6, updated_at=?7 WHERE id=?8",
        rusqlite::params![&input.title, &input.description, &input.status, &input.target_name, &input.target_email, &input.target_organization, &Utc::now().to_rfc3339(), &input.case_id],
    ).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn case_delete(state: State<'_, AppState>, input: EmptyInput) -> Result<bool, String> {
    let db = state.db.lock().await;
    // Delete all related data in correct order (foreign key safety)
    db.conn.execute("DELETE FROM case_notes WHERE case_id=?1", [&input.case_id]).ok();
    db.conn.execute("DELETE FROM findings WHERE case_id=?1", [&input.case_id]).ok();
    db.conn.execute("DELETE FROM entities WHERE case_id=?1", [&input.case_id]).ok();
    db.conn.execute("DELETE FROM communication_edges WHERE case_id=?1", [&input.case_id]).ok();
    db.conn.execute("DELETE FROM timeline_events WHERE case_id=?1", [&input.case_id]).ok();
    db.conn.execute("DELETE FROM attachments WHERE email_id IN (SELECT id FROM emails WHERE case_id=?1)", [&input.case_id]).ok();
    db.conn.execute("DELETE FROM emails WHERE case_id=?1", [&input.case_id]).ok();
    db.conn.execute("DELETE FROM custody_events WHERE evidence_id IN (SELECT id FROM evidence_items WHERE case_id=?1)", [&input.case_id]).ok();
    db.conn.execute("DELETE FROM evidence_items WHERE case_id=?1", [&input.case_id]).ok();
    db.conn.execute("DELETE FROM cases WHERE id=?1", [&input.case_id]).map_err(|e| e.to_string())?;
    Ok(true)
}

#[tauri::command]
pub async fn evidence_upload(state: State<'_, AppState>, input: EvidenceUploadInput) -> Result<EvidenceItem, String> {
    let db = state.db.lock().await;
    let path = PathBuf::from(&input.file_path);
    let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("unknown").to_string();
    let format = detect_format(&filename);
    let size = fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
    let sha256 = compute_sha256(&path).map_err(|e| e.to_string())?;
    let sha512 = compute_sha512(&path).map_err(|e| e.to_string())?;
    let id = generate_id();
    let now = Utc::now();
    let src = input.source_description.clone().unwrap_or_default();
    db.conn.execute(
        "INSERT INTO evidence_items (id,case_id,filename,original_path,stored_path,format,sha256,sha512,size_bytes,source_description,acquired_by,acquired_at,acquisition_method,integrity_level,parse_status,message_count,deleted_recovered,created_at) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18)",
        rusqlite::params![&id,&input.case_id,&filename,&input.file_path,&input.file_path,&format,&sha256,&sha512,size as i64,&src,"admin",&now.to_rfc3339(),"manual","high","pending",0i64,0i64,&now.to_rfc3339()],
    ).map_err(|e| e.to_string())?;
    db.conn.execute(
        "INSERT INTO custody_events (id,evidence_id,action,actor,timestamp,tool,tool_version,hash_after,detail) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9)",
        rusqlite::params![&generate_id(),&id,"ingested","admin",&now.to_rfc3339(),"email-forensic","0.1.0",&sha256,&format!("Uploaded {} ({} bytes)",filename,size)],
    ).ok();
    Ok(EvidenceItem { id, case_id: input.case_id, filename, original_path: input.file_path.clone(), stored_path: input.file_path, format, sha256, sha512: Some(sha512), size_bytes: size, source_description: src, acquired_by: "admin".to_string(), acquired_at: now, acquisition_method: "manual".to_string(), integrity_level: "high".to_string(), parse_status: "pending".to_string(), parse_error: None, message_count: 0, deleted_recovered: 0 })
}

#[tauri::command]
pub async fn evidence_list(state: State<'_, AppState>, input: EmptyInput) -> Result<Vec<EvidenceItem>, String> {
    let db = state.db.lock().await;
    let mut stmt = db.conn.prepare("SELECT id,case_id,filename,original_path,stored_path,format,sha256,sha512,size_bytes,source_description,acquired_by,acquired_at,acquisition_method,integrity_level,parse_status,parse_error,message_count,deleted_recovered FROM evidence_items WHERE case_id=?1 ORDER BY acquired_at DESC").map_err(|e| e.to_string())?;
    let items = stmt.query_map([&input.case_id], |row| {
        Ok(EvidenceItem { id: row.get(0)?, case_id: row.get(1)?, filename: row.get(2)?, original_path: row.get(3)?, stored_path: row.get(4)?, format: row.get(5)?, sha256: row.get(6)?, sha512: row.get(7)?, size_bytes: u64v(row,8), source_description: row.get(9)?, acquired_by: row.get(10)?, acquired_at: parse_dt(&row.get::<_,String>(11)?), acquisition_method: row.get(12)?, integrity_level: row.get(13)?, parse_status: row.get(14)?, parse_error: row.get(15)?, message_count: u32v(row,16), deleted_recovered: u32v(row,17) })
    }).map_err(|e| e.to_string())?.collect::<Result<Vec<_>,_>>().map_err(|e| e.to_string())?;
    Ok(items)
}

#[tauri::command]
pub async fn evidence_status(state: State<'_, AppState>, input: EmptyInput) -> Result<Vec<EvidenceItem>, String> { evidence_list(state, input).await }

#[tauri::command]
pub async fn email_list(state: State<'_, AppState>, input: EmailListInput) -> Result<Vec<EmailMessage>, String> {
    let db = state.db.lock().await;
    let limit = input.limit.unwrap_or(100) as i64;
    let mut stmt = db.conn.prepare("SELECT id,evidence_id,case_id,message_id,from_addr,from_display,to_addrs,cc_addrs,subject,date_sent,date_sent_utc,headers_raw,body_text,body_html,folder_name,folder_category,is_deleted,deleted_recovered,risk_score,flags FROM emails WHERE case_id=?1 ORDER BY date_sent DESC LIMIT ?2").map_err(|e| e.to_string())?;
    let emails = stmt.query_map(rusqlite::params![&input.case_id, limit], |row| {
        Ok(EmailMessage { id: row.get(0)?, evidence_id: row.get(1)?, case_id: row.get(2)?, message_id: row.get(3)?, from_addr: row.get(4)?, from_display: row.get(5)?, to_addrs: row.get(6)?, cc_addrs: row.get(7)?, subject: row.get(8)?, date_sent: row.get(9)?, date_sent_utc: row.get(10)?, headers_raw: row.get(11)?, body_text: row.get(12)?, body_html: row.get(13)?, folder_name: row.get(14)?, folder_category: row.get(15)?, is_deleted: boolv(row,16), deleted_recovered: boolv(row,17), risk_score: u8v(row,18), flags: row.get(19)? })
    }).map_err(|e| e.to_string())?.collect::<Result<Vec<_>,_>>().map_err(|e| e.to_string())?;
    Ok(emails)
}

#[tauri::command]
pub async fn email_get(state: State<'_, AppState>, input: serde_json::Value) -> Result<Option<EmailMessage>, String> {
    let email_id = input["id"].as_str()
        .or_else(|| input["email_id"].as_str())
        .or_else(|| input["case_id"].as_str())
        .or_else(|| input.as_str())
        .unwrap_or("")
        .to_string();

    if email_id.is_empty() {
        return Ok(None);
    }

    let db = state.db.lock().await;
    let r = db.conn.query_row(
        "SELECT id,evidence_id,case_id,message_id,from_addr,from_display,to_addrs,cc_addrs,subject,date_sent,date_sent_utc,headers_raw,body_text,body_html,folder_name,folder_category,is_deleted,deleted_recovered,risk_score,flags 
         FROM emails WHERE id=?1", 
        [&email_id],
        |row| Ok(EmailMessage { 
            id: row.get(0)?, 
            evidence_id: row.get(1)?, 
            case_id: row.get(2)?, 
            message_id: row.get(3)?, 
            from_addr: row.get(4)?, 
            from_display: row.get(5)?, 
            to_addrs: row.get(6)?, 
            cc_addrs: row.get(7)?, 
            subject: row.get(8)?, 
            date_sent: row.get(9)?, 
            date_sent_utc: row.get(10)?, 
            headers_raw: row.get(11)?, 
            body_text: row.get(12)?, 
            body_html: row.get(13)?, 
            folder_name: row.get(14)?, 
            folder_category: row.get(15)?, 
            is_deleted: boolv(row,16), 
            deleted_recovered: boolv(row,17), 
            risk_score: u8v(row,18), 
            flags: row.get(19)? 
        })
    );
    match r { 
        Ok(e) => Ok(Some(e)), 
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None), 
        Err(e) => Err(e.to_string()) 
    }
}

#[tauri::command]
pub async fn search(state: State<'_, AppState>, input: SearchInput) -> Result<Vec<EmailMessage>, String> {
    let db = state.db.lock().await;
    let limit = input.limit.unwrap_or(100) as i64;
    let q = format!("%{}%", input.query);
    let mut stmt = db.conn.prepare("SELECT id,evidence_id,case_id,message_id,from_addr,from_display,to_addrs,cc_addrs,subject,date_sent,date_sent_utc,headers_raw,body_text,is_deleted,deleted_recovered,risk_score,flags FROM emails WHERE case_id=?1 AND (from_addr LIKE ?2 OR to_addrs LIKE ?2 OR subject LIKE ?2 OR body_text LIKE ?2) ORDER BY date_sent DESC LIMIT ?3").map_err(|e| e.to_string())?;
    let emails = stmt.query_map(rusqlite::params![&input.case_id, &q, limit], |row| {
        Ok(EmailMessage { id: row.get(0)?, evidence_id: row.get(1)?, case_id: row.get(2)?, message_id: row.get(3)?, from_addr: row.get(4)?, from_display: row.get(5)?, to_addrs: row.get(6)?, cc_addrs: row.get(7)?, subject: row.get(8)?, date_sent: row.get(9)?, date_sent_utc: row.get(10)?, headers_raw: row.get(11)?, body_text: row.get(12)?, body_html: row.get(13)?, folder_name: row.get(14)?, folder_category: row.get(15)?, is_deleted: boolv(row,16), deleted_recovered: boolv(row,17), risk_score: u8v(row,18), flags: row.get(19)? })
    }).map_err(|e| e.to_string())?.collect::<Result<Vec<_>,_>>().map_err(|e| e.to_string())?;
    Ok(emails)
}

#[tauri::command]
pub async fn findings_list(state: State<'_, AppState>, input: serde_json::Value) -> Result<Vec<Finding>, String> {
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
            confidence: row.get(4)?, 
            title: row.get(5)?, 
            description: row.get(6)?, 
            evidence_refs: row.get(7)?, 
            email_ids: row.get(8)?, 
            status: row.get(9)?, 
            created_at: parse_dt(&row.get::<_,String>(10)?),
            reviewed_by: row.get(11)?,
            reviewed_at: row.get::<_, Option<String>>(12).ok().and_then(|opt_str| {
                opt_str.and_then(|s| s.parse::<chrono::DateTime<chrono::Utc>>().ok())
            }),
            notes: row.get(13).ok().unwrap_or(None),
        })
    }).map_err(|e| e.to_string())?.collect::<Result<Vec<_>,_>>().map_err(|e| e.to_string())?;
    Ok(findings)
}

#[tauri::command]
pub async fn dashboard(state: State<'_, AppState>, input: EmptyInput) -> Result<DashboardData, String> {
    let db = state.db.lock().await;
    let ec: i64 = db.conn.query_row("SELECT COUNT(*) FROM evidence_items WHERE case_id=?1", [&input.case_id], |r| r.get(0)).unwrap_or(0);
    let emc: i64 = db.conn.query_row("SELECT COUNT(*) FROM emails WHERE case_id=?1", [&input.case_id], |r| r.get(0)).unwrap_or(0);
    let dr: i64 = db.conn.query_row("SELECT COUNT(*) FROM emails WHERE case_id=?1 AND folder_category='soft_deleted'", [&input.case_id], |r| r.get(0)).unwrap_or(0);
    let sentc: i64 = db.conn.query_row("SELECT COUNT(*) FROM emails WHERE case_id=?1 AND folder_category='sent'", [&input.case_id], |r| r.get(0)).unwrap_or(0);
    let inboxc: i64 = db.conn.query_row("SELECT COUNT(*) FROM emails WHERE case_id=?1 AND folder_category='inbox'", [&input.case_id], |r| r.get(0)).unwrap_or(0);
    let draftsc: i64 = db.conn.query_row("SELECT COUNT(*) FROM emails WHERE case_id=?1 AND folder_category='drafts'", [&input.case_id], |r| r.get(0)).unwrap_or(0);
    let spamc: i64 = db.conn.query_row("SELECT COUNT(*) FROM emails WHERE case_id=?1 AND folder_category='spam'", [&input.case_id], |r| r.get(0)).unwrap_or(0);
    let otherc: i64 = db.conn.query_row("SELECT COUNT(*) FROM emails WHERE case_id=?1 AND folder_category='other'", [&input.case_id], |r| r.get(0)).unwrap_or(0);
    let enc: i64 = db.conn.query_row("SELECT COUNT(*) FROM entities WHERE case_id=?1", [&input.case_id], |r| r.get(0)).unwrap_or(0);
    
    // Get findings count per severity (use DISTINCT to avoid duplicates)
    let fc: i64 = db.conn.query_row("SELECT COUNT(DISTINCT id) FROM findings WHERE case_id=?1", [&input.case_id], |r| r.get(0)).unwrap_or(0);
    
    // Severity breakdown for findings
    let mut severity_breakdown = std::collections::HashMap::new();
    for severity in &["critical", "high", "medium", "low"] {
        let count: i64 = db.conn.query_row(
            "SELECT COUNT(DISTINCT id) FROM findings WHERE case_id=?1 AND severity=?2",
            rusqlite::params![&input.case_id, severity],
            |r| r.get(0),
        ).unwrap_or(0);
        severity_breakdown.insert(severity.to_string(), count as u32);
    }
    
    // Risk score distribution
    let high_risk: i64 = db.conn.query_row(
        "SELECT COUNT(*) FROM emails WHERE case_id=?1 AND risk_score >= 50",
        [&input.case_id],
        |r| r.get(0),
    ).unwrap_or(0);
    
    let date_range: (Option<String>, Option<String>) = {
        let mut stmt = db.conn.prepare(
            "SELECT MIN(date_sent_utc), MAX(date_sent_utc) FROM emails WHERE case_id=?1"
        ).ok();
        match stmt {
            Some(mut s) => {
                let row = s.query_row([&input.case_id], |r| {
                    Ok((r.get::<_, Option<String>>(0)?, r.get::<_, Option<String>>(1)?))
                }).ok();
                row.unwrap_or((None, None))
            }
            None => (None, None)
        }
    };
    
    Ok(DashboardData { 
        evidence_count: ec as u32, 
        email_count: emc as u32, 
        deleted_recovered: dr as u32, 
        entity_count: enc as u32, 
        finding_count: fc as u32, 
        severity_breakdown,
        date_range,
        top_correspondents: vec![], 
        sent_count: sentc as u32, 
        inbox_count: inboxc as u32, 
        soft_deleted_count: dr as u32, 
        drafts_count: draftsc as u32, 
        spam_count: spamc as u32, 
        other_count: otherc as u32,
        high_risk_emails: high_risk as u32,
    })
}

#[tauri::command]
pub async fn custody_chain(state: State<'_, AppState>, input: EmptyInput) -> Result<Vec<CustodyEvent>, String> {
    let db = state.db.lock().await;
    let mut stmt = db.conn.prepare("SELECT id,evidence_id,action,actor,timestamp,tool,tool_version,hash_before,hash_after,detail FROM custody_events WHERE evidence_id=?1 ORDER BY timestamp").map_err(|e| e.to_string())?;
    let events = stmt.query_map([&input.case_id], |row| {
        Ok(CustodyEvent { id: row.get(0)?, evidence_id: row.get(1)?, action: row.get(2)?, actor: row.get(3)?, timestamp: parse_dt(&row.get::<_,String>(4)?), tool: row.get(5)?, tool_version: row.get(6)?, hash_before: row.get(7)?, hash_after: row.get(8)?, detail: row.get(9)? })
    }).map_err(|e| e.to_string())?.collect::<Result<Vec<_>,_>>().map_err(|e| e.to_string())?;
    Ok(events)
}

/// Parse an evidence file and store emails in the database
#[tauri::command]
pub async fn parse_evidence(state: State<'_, AppState>, evidence_id: String) -> Result<u32, String> {
    // Get evidence info
    let (case_id, file_path, format) = {
        let db = state.db.lock().await;
        db.conn.query_row(
            "SELECT case_id, stored_path, format FROM evidence_items WHERE id=?1",
            [&evidence_id],
            |row| Ok((row.get::<_,String>(0)?, row.get::<_,String>(1)?, row.get::<_,String>(2)?)),
        ).map_err(|e| format!("Database error: {}", e))?
    };
    
    // Update status to parsing
    {
        let db = state.db.lock().await;
        db.conn.execute("UPDATE evidence_items SET parse_status='parsing', parse_error=NULL WHERE id=?1", [&evidence_id]).ok();
    }
    
    // Parse in a blocking task with timeout
    let result = tokio::task::spawn_blocking(move || {
        let path = std::path::Path::new(&file_path);
        match format.as_str() {
            "eml" => crate::parser::parse_eml(path),
            "mbox" => crate::parser::parse_mbox(path),
            "emlx" => crate::pst::parse_emlx(path),
            "msg" => crate::pst::parse_msg(path),
            "pst" | "ost" => crate::pst::PstParser::parse(path),
            _ => Err(format!("Unsupported format: {}", format)),
        }
    }).await;

    let emails = match result {
        Ok(Ok(e)) => {
            // Limit to 10000 emails max to prevent database overload
            if e.len() > 10000 {
                e[..10000].to_vec()
            } else {
                e
            }
        }
        Ok(Err(e)) => {
            let db = state.db.lock().await;
            db.conn.execute("UPDATE evidence_items SET parse_status='error', parse_error=?1 WHERE id=?2", [&e, &evidence_id]).ok();
            return Err(e);
        }
        Err(e) => {
            let err = format!("Task join error: {}", e);
            let db = state.db.lock().await;
            db.conn.execute("UPDATE evidence_items SET parse_status='error', parse_error=?1 WHERE id=?2", [&err, &evidence_id]).ok();
            return Err(err);
        }
    };
    
    let count = emails.len() as u32;
    
    // Insert emails in a transaction for speed
    {
        let mut db = state.db.lock().await;
        let tx = db.conn.transaction().map_err(|e| format!("Transaction error: {}", e))?;
        
        for email in &emails {
            let email_id = generate_id();
            let to_json = serde_json::to_string(&email.to_addrs).unwrap_or_default();
            let cc_json = serde_json::to_string(&email.cc_addrs).unwrap_or_default();
            let bcc_json = serde_json::to_string(&email.bcc_addrs).unwrap_or_default();
            let to_names_json = serde_json::to_string(&email.to_display_names).unwrap_or_default();
            let cc_names_json = serde_json::to_string(&email.cc_display_names).unwrap_or_default();
            let received_json = serde_json::to_string(&email.received_chain).unwrap_or_default();
            let references_json = serde_json::to_string(&email.references).unwrap_or_default();
            let date_str = email.date_sent.map(|d| d.to_rfc3339());
            
            tx.execute(
                "INSERT INTO emails (id, evidence_id, case_id, message_id, from_addr, from_display, to_addrs, cc_addrs, bcc_addrs, to_display_names, cc_display_names, subject, subject_raw, date_sent, date_sent_utc, headers_raw, headers_json, body_text, body_html, folder_name, folder_category, recovery_status, is_deleted, deleted_recovered, risk_score, flags, received_chain, return_path, reply_to, x_mailer, x_originating_ip, importance, in_reply_to, msg_references, x_to_header, x_cc_header, created_at)
                 VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18,?19,?20,?21,?22,?23,?24,?25,?26,?27,?28,?29,?30,?31,?32,?33,?34,?35,?36,?37)",
                rusqlite::params![
                    &email_id, &evidence_id, &case_id, &email.message_id, &email.from_addr, &email.from_display,
                    &to_json, &cc_json, &bcc_json, &to_names_json, &cc_names_json, &email.subject, &email.subject_raw, &date_str, &date_str,
                    &email.headers_raw, &email.headers_json, &email.body_text, &email.body_html,
                    &email.folder_name, &email.folder_category, &email.recovery_status,
                    0i64, 0i64, 0i64, "[]",
                    &received_json, &email.return_path, &email.reply_to, &email.x_mailer,
                    &email.x_originating_ip, &email.importance, &email.in_reply_to, &references_json,
                    &email.x_to_header, &email.x_cc_header, &chrono::Utc::now().to_rfc3339()
                ],
            ).map_err(|e| format!("Insert email {}: {}", email.message_id, e))?;
            
            // Insert attachments
            for att in &email.attachments {
                let att_id = generate_id();
                let sha256 = crate::parser::sha256_data(&att.data);
                let size = att.data.len() as i64;
                tx.execute(
                    "INSERT INTO attachments (id, email_id, filename, sha256, mime_type, size_bytes, stored_path, entropy, risk_flags, created_at)
                     VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10)",
                    rusqlite::params![
                        &att_id, &email_id, &att.filename, &sha256, &att.content_type, &size, "", 0.0, "[]", &chrono::Utc::now().to_rfc3339()
                    ],
                ).ok(); // Don't fail on attachment errors
            }
        }
        
        // Update evidence status before commit
        tx.execute(
            "UPDATE evidence_items SET parse_status='done', message_count=?1 WHERE id=?2",
            rusqlite::params![count as i64, &evidence_id],
        ).map_err(|e| format!("Update status: {}", e))?;
        
        tx.commit().map_err(|e| format!("Commit error: {}", e))?;
    }
    
    // Auto-run entity extraction and analysis after parsing
    let _ = extract_entities(state.clone(), serde_json::json!({ "case_id": case_id })).await;
    let _ = run_analysis(state.clone(), serde_json::json!({ "case_id": case_id })).await;
    
    Ok(count)
}

/// Read a file and return its bytes as base64
#[tauri::command]
pub async fn read_file(_state: State<'_, AppState>, path: String) -> Result<Vec<u8>, String> {
    use std::fs;
    fs::read(&path).map_err(|e| format!("Read error: {}", e))
}

/// Open native file dialog and return selected path
#[tauri::command]
pub async fn open_file_dialog() -> Result<Option<String>, String> {
    use rfd::FileDialog;
    
    let result = FileDialog::new()
        .add_filter("Email Files", &["eml", "mbox", "msg", "pst", "ost", "emlx"])
        .add_filter("All Files", &["*"])
        .pick_file();
    
    Ok(result.map(|p| p.to_string_lossy().to_string()))
}

/// Write base64-encoded data to a temporary file
#[tauri::command]
pub async fn write_temp_file(_state: State<'_, AppState>, path: String, data: String) -> Result<(), String> {
    use std::fs;
    use base64::Engine;
    
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(&data)
        .map_err(|e| format!("Decode error: {}", e))?;
    
    fs::write(&path, bytes).map_err(|e| format!("Write error: {}", e))?;
    Ok(())
}

/// Get full email details including parsed headers
#[tauri::command]
pub async fn email_headers(state: State<'_, AppState>, email_id: String) -> Result<serde_json::Value, String> {
    let db = state.db.lock().await;
    let (headers_raw, from_addr, to_addrs, subject, date_sent, message_id, from_display): (Option<String>, String, String, Option<String>, Option<String>, Option<String>, Option<String>) = db.conn.query_row(
        "SELECT headers_raw, from_addr, to_addrs, subject, date_sent, message_id, from_display FROM emails WHERE id=?1",
        [&email_id],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?, row.get(5)?, row.get(6)?)),
    ).map_err(|e| e.to_string())?;
    
    // Parse Received chain from headers
    let mut received_chain = Vec::new();
    if let Some(ref raw) = headers_raw {
        for line in raw.lines() {
            if line.starts_with("Received:") {
                received_chain.push(line.trim().to_string());
            }
        }
    }
    
    // Run header analysis
    let header_analysis = analyze_headers(headers_raw.as_deref().unwrap_or(""));
    
    // Run authentication analysis
    let from_domain = from_addr.split('@').nth(1).unwrap_or("");
    let source_ip = header_analysis.originating_ip.as_deref();
    let auth_results = analyze_authentication(
        headers_raw.as_deref().unwrap_or(""),
        from_domain,
        source_ip,
    );
    
    // Run spoofing detection
    let spoof_findings = detect_spoofing(
        &from_addr,
        from_display.as_deref(),
        headers_raw.as_deref().unwrap_or(""),
        &auth_results,
    );
    
    // Calculate risk score
    let risk_score = calculate_risk_score(&header_analysis, &auth_results, &spoof_findings, &[]);
    
    Ok(serde_json::json!({
        "email_id": email_id,
        "message_id": message_id,
        "from": from_addr,
        "from_display": from_display,
        "to": serde_json::from_str::<Vec<String>>(&to_addrs).unwrap_or_default(),
        "subject": subject,
        "date_sent": date_sent,
        "received_chain": received_chain,
        "headers_raw": headers_raw,
        "header_analysis": header_analysis,
        "auth_results": auth_results,
        "spoof_findings": spoof_findings,
        "risk_score": risk_score,
    }))
}

/// Run analysis on all emails in a case and generate findings
#[tauri::command]
pub async fn run_analysis(state: State<'_, AppState>, input: serde_json::Value) -> Result<u32, String> {
    let case_id = input["case_id"].as_str()
        .or_else(|| input["caseId"].as_str())
        .or_else(|| input.as_str())
        .unwrap_or("")
        .to_string();

    if case_id.is_empty() {
        return Err("Case ID cannot be empty".to_string());
    }

    let mut db = state.db.lock().await;

    // 1. Fetch all emails for this case
    struct EmailToAnalyze {
        id: String,
        from_addr: String,
        from_display: Option<String>,
        subject: Option<String>,
        body_text: Option<String>,
        headers_raw: Option<String>,
        folder_category: String,
        deleted_recovered: bool,
    }

    let emails: Vec<EmailToAnalyze> = {
        let mut emails_stmt = db.conn.prepare(
            "SELECT id, from_addr, from_display, subject, body_text, headers_raw, folder_category, deleted_recovered 
             FROM emails WHERE case_id=?1"
        ).map_err(|e| e.to_string())?;

        let collected: Result<Vec<_>, _> = emails_stmt.query_map([&case_id], |row| {
            Ok(EmailToAnalyze {
                id: row.get(0)?,
                from_addr: row.get(1)?,
                from_display: row.get(2)?,
                subject: row.get(3)?,
                body_text: row.get(4)?,
                headers_raw: row.get(5)?,
                folder_category: row.get::<_, Option<String>>(6)?.unwrap_or_default(),
                deleted_recovered: boolv(row, 7),
            })
        }).map_err(|e| e.to_string())?.collect();

        collected.map_err(|e| e.to_string())?
    };

    // 2. Fetch all attachments for this case and group by email_id
    let attachments_by_email = {
        let mut att_stmt = db.conn.prepare(
            "SELECT a.email_id, a.filename, a.mime_type, a.size_bytes, a.entropy, a.risk_flags 
             FROM attachments a 
             JOIN emails e ON a.email_id = e.id 
             WHERE e.case_id = ?1"
        ).map_err(|e| e.to_string())?;

        let mut map: std::collections::HashMap<String, Vec<(Option<String>, Option<String>, u64, Option<f64>, Option<String>)>> = std::collections::HashMap::new();

        let collected: Result<Vec<_>, _> = att_stmt.query_map([&case_id], |row| {
            Ok((
                row.get::<_, String>(0)?, // email_id
                row.get::<_, Option<String>>(1)?, // filename
                row.get::<_, Option<String>>(2)?, // mime_type
                row.get::<_, i64>(3)? as u64,     // size_bytes
                row.get::<_, Option<f64>>(4)?,    // entropy
                row.get::<_, Option<String>>(5)?, // risk_flags
            ))
        }).map_err(|e| e.to_string())?.collect();

        for item in collected.unwrap_or_default() {
            map.entry(item.0).or_default().push((item.1, item.2, item.3, item.4, item.5));
        }
        map
    };

    // 3. Clear old findings inside transaction and batch process
    let tx = db.conn.transaction().map_err(|e| format!("Transaction error: {}", e))?;
    tx.execute("DELETE FROM findings WHERE case_id=?1", [&case_id]).map_err(|e| format!("Clear findings: {}", e))?;

    let mut total_findings: u32 = 0;
    let mut findings_to_insert: Vec<NewFinding> = Vec::new();
    let mut email_risk_updates: Vec<(u8, String)> = Vec::new();

    for email in &emails {
        let headers = email.headers_raw.as_deref().unwrap_or("");
        let from_domain = email.from_addr.split('@').nth(1).unwrap_or("");

        // Run forensic engines
        let header_analysis = analyze_headers(headers);
        let source_ip = header_analysis.originating_ip.as_deref();
        let auth_results = analyze_authentication(headers, from_domain, source_ip);
        
        let mut spoof_findings = detect_spoofing(&email.from_addr, email.from_display.as_deref(), headers, &auth_results);

        // Run deep content threat detection (BEC wire fraud, lures, credential exfiltration)
        let content_threats = detect_content_threats(
            &email.from_addr,
            email.from_display.as_deref(),
            email.subject.as_deref(),
            email.body_text.as_deref(),
        );
        spoof_findings.extend(content_threats);

        // Run attachment forensic inspections
        let mut attachment_analyses = Vec::new();
        if let Some(att_list) = attachments_by_email.get(&email.id) {
            for (fname, fmime, fsize, fent, fflags) in att_list {
                let att_analysis = analyze_attachment_metadata(
                    fname.as_deref(),
                    fmime.as_deref(),
                    *fsize,
                    *fent,
                    fflags.as_deref(),
                );
                attachment_analyses.push(att_analysis);
            }
        }

        // Anti-forensics / Recovered deletion check
        if email.deleted_recovered || email.folder_category == "soft_deleted" {
            let has_threat = !spoof_findings.is_empty() || !attachment_analyses.iter().all(|a| a.risk_flags.is_empty());
            if has_threat {
                spoof_findings.push(crate::analysis::SpoofingFinding {
                    finding_type: "recovered_deleted_threat".to_string(),
                    severity: "high".to_string(),
                    confidence: "high".to_string(),
                    title: "Evidentiary Purge: High-Threat Email Recovered From Dumpster".to_string(),
                    description: format!(
                        "Subject '{}' was intentionally deleted/purged from active mailbox and recovered during forensic extraction.",
                        email.subject.as_deref().unwrap_or("[No Subject]")
                    ),
                    indicator: "anti_forensics_purged_message".to_string(),
                });
            }
        }

        let risk_score = calculate_risk_score(&header_analysis, &auth_results, &spoof_findings, &attachment_analyses);

        let new_findings = generate_findings(
            &email.id,
            &header_analysis,
            &auth_results,
            &spoof_findings,
            &attachment_analyses,
        );

        total_findings += new_findings.len() as u32;
        findings_to_insert.extend(new_findings);
        email_risk_updates.push((risk_score, email.id.clone()));
    }

    // 4. Batch insert findings into database
    {
        let mut insert_finding = tx.prepare_cached(
            "INSERT INTO findings (id, case_id, type, severity, confidence, title, description, evidence_refs, email_ids, status, created_at)
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,'open',?10)"
        ).map_err(|e| e.to_string())?;

        let now_str = Utc::now().to_rfc3339();
        for finding in &findings_to_insert {
            let id = generate_id();
            let email_ids_json = serde_json::to_string(&finding.email_ids).unwrap_or_default();
            let evidence_refs = "[]".to_string();

            insert_finding.execute(rusqlite::params![
                &id, &case_id, &finding.type_, &finding.severity, &finding.confidence,
                &finding.title, &finding.description, &evidence_refs, &email_ids_json, &now_str
            ]).map_err(|e| format!("Insert finding: {}", e))?;
        }
    }

    // 5. Batch update email risk scores
    {
        let mut update_email = tx.prepare_cached(
            "UPDATE emails SET risk_score = ?1 WHERE id = ?2"
        ).map_err(|e| e.to_string())?;

        for (score, eid) in email_risk_updates {
            update_email.execute(rusqlite::params![score as i64, &eid]).ok();
        }
    }

    tx.commit().map_err(|e| format!("Commit findings: {}", e))?;

    Ok(total_findings)
}

/// Get attachments for an email
#[tauri::command]
pub async fn email_attachments(state: State<'_, AppState>, input: serde_json::Value) -> Result<Vec<Attachment>, String> {
    let email_id = input["email_id"].as_str()
        .or_else(|| input["emailId"].as_str())
        .or_else(|| input.as_str())
        .unwrap_or("")
        .to_string();

    let db = state.db.lock().await;
    let mut stmt = db.conn.prepare("SELECT id, email_id, filename, sha256, mime_type, size_bytes, stored_path, entropy, risk_flags FROM attachments WHERE email_id=?1").map_err(|e| e.to_string())?;
    let attachments = stmt.query_map([&email_id], |row| {
        Ok(Attachment {
            id: row.get(0)?,
            email_id: row.get(1)?,
            filename: row.get(2)?,
            sha256: row.get(3)?,
            mime_type: row.get(4)?,
            size_bytes: row.get::<_, i64>(5)? as u64,
            stored_path: row.get(6)?,
            entropy: row.get::<_, Option<f64>>(7)?,
            risk_flags: row.get(8)?,
        })
    }).map_err(|e| e.to_string())?.collect::<Result<Vec<_>,_>>().map_err(|e| e.to_string())?;
    Ok(attachments)
}

/// Helper to capitalize words in a display name
fn capitalize_words(s: &str) -> String {
    s.split_whitespace()
        .map(|w| {
            let mut c = w.chars();
            match c.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn clean_display_or_email(name_opt: Option<&str>, email: &str) -> String {
    if let Some(d) = name_opt {
        let trimmed = d.trim();
        if !trimmed.is_empty() && trimmed != email && !trimmed.starts_with('/') {
            if trimmed.contains("..") {
                let parts: Vec<&str> = trimmed.split("..").collect();
                if parts.len() == 2 && !parts[0].is_empty() && !parts[1].is_empty() {
                    return format!("{}. {}", parts[0].to_uppercase(), capitalize_words(parts[1]));
                }
            }
            return trimmed.to_string();
        }
    }
    let local = email.split('@').next().unwrap_or(email);
    if local.contains("..") {
        let parts: Vec<&str> = local.split("..").collect();
        if parts.len() == 2 && !parts[0].is_empty() && !parts[1].is_empty() {
            return format!("{}. {}", parts[0].to_uppercase(), capitalize_words(parts[1]));
        }
    } else if local.contains('.') {
        let parts: Vec<&str> = local.split('.').collect();
        return parts.iter().map(|p| capitalize_words(p)).collect::<Vec<_>>().join(" ");
    }
    capitalize_words(local)
}

/// Auto-detect potential targets from email data
#[tauri::command]
pub async fn auto_detect_targets(state: State<'_, AppState>, input: serde_json::Value) -> Result<serde_json::Value, String> {
    let case_id = input["case_id"].as_str()
        .or_else(|| input["caseId"].as_str())
        .or_else(|| input.as_str())
        .unwrap_or("")
        .to_string();

    let db = state.db.lock().await;

    // Check if entities table has records. If not, auto extract!
    let entity_count: i64 = db.conn.query_row(
        "SELECT COUNT(*) FROM entities WHERE case_id=?1",
        [&case_id],
        |r| r.get(0),
    ).unwrap_or(0);

    drop(db);

    if entity_count == 0 {
        let _ = extract_entities(state.clone(), serde_json::json!({ "case_id": case_id.clone() })).await;
    }

    let db = state.db.lock().await;
    let total_entities_in_case: i64 = db.conn.query_row(
        "SELECT COUNT(*) FROM entities WHERE case_id=?1",
        [&case_id],
        |r| r.get(0),
    ).unwrap_or(0);

    let mut stmt = db.conn.prepare("
        SELECT email_address, display_name, sent_count, received_count, (sent_count + received_count) as total
        FROM entities WHERE case_id=?1 AND email_address LIKE '%@%'
        ORDER BY total DESC LIMIT 50
    ").map_err(|e| e.to_string())?;

    let rows: Vec<serde_json::Value> = stmt.query_map([&case_id], |row| {
        let email: String = row.get(0)?;
        let display: Option<String> = row.get(1)?;
        let sent: i64 = row.get(2)?;
        let received: i64 = row.get(3)?;
        let total: i64 = row.get(4)?;

        let best_display = clean_display_or_email(display.as_deref(), &email);

        Ok(serde_json::json!({
            "email": email,
            "display_name": best_display,
            "total_emails": total,
            "sent": sent,
            "received": received,
        }))
    }).map_err(|e| e.to_string())?.filter_map(|r| r.ok()).collect();

    Ok(serde_json::json!({
        "targets": rows,
        "case_id": case_id,
        "total_case_entities": total_entities_in_case,
    }))
}

#[tauri::command]
pub async fn target_profile(state: State<'_, AppState>, input: serde_json::Value) -> Result<serde_json::Value, String> {
    let case_id = input["case_id"].as_str()
        .or_else(|| input["caseId"].as_str())
        .unwrap_or("")
        .to_string();
    let target_email = input["target_email"].as_str()
        .or_else(|| input["targetEmail"].as_str())
        .map(|s| s.to_string());

    let db = state.db.lock().await;

    // Get case info
    let case: (String, Option<String>, Option<String>, Option<String>) = db.conn.query_row(
        "SELECT title, target_email, target_name, target_organization FROM cases WHERE id=?1",
        [&case_id],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
    ).map_err(|e| e.to_string())?;

    let (case_title, default_target_email, target_name, target_organization) = case;

    // Selected email takes precedence, then case configured target email, then top entity
    let email = match target_email.filter(|e| !e.trim().is_empty()) {
        Some(e) => e,
        None => match default_target_email.filter(|e| !e.trim().is_empty()) {
            Some(e) => e,
            None => {
                db.conn.query_row(
                    "SELECT email_address FROM entities WHERE case_id=?1 AND email_address LIKE '%@%' ORDER BY (sent_count + received_count) DESC LIMIT 1",
                    [&case_id],
                    |r| r.get(0),
                ).unwrap_or_default()
            }
        },
    };

    let email_prefix = email.split('@').next().unwrap_or(&email);

    // Count emails where target is sender
    let sent_count: i64 = if email.is_empty() { 0 } else {
        db.conn.query_row(
            "SELECT COUNT(*) FROM emails WHERE case_id=?1 AND (from_addr LIKE ?2 OR from_addr LIKE ?3)",
            rusqlite::params![&case_id, format!("%{}%", email), format!("%{}%", email_prefix)],
            |r| r.get(0),
        ).unwrap_or(0)
    };

    // Count emails where target is recipient
    let received_count: i64 = if email.is_empty() { 0 } else {
        db.conn.query_row(
            "SELECT COUNT(*) FROM emails WHERE case_id=?1 AND (to_addrs LIKE ?2 OR cc_addrs LIKE ?2 OR to_addrs LIKE ?3 OR cc_addrs LIKE ?3)",
            rusqlite::params![&case_id, format!("%{}%", email), format!("%{}%", email_prefix)],
            |r| r.get(0),
        ).unwrap_or(0)
    };

    // Get first and last seen dates
    let first_seen: Option<String> = if email.is_empty() { None } else {
        db.conn.query_row(
            "SELECT MIN(date_sent_utc) FROM emails WHERE case_id=?1 AND (from_addr LIKE ?2 OR to_addrs LIKE ?2 OR cc_addrs LIKE ?2)",
            rusqlite::params![&case_id, format!("%{}%", email_prefix)],
            |r| r.get::<_, Option<String>>(0),
        ).ok().flatten()
    };

    let last_seen: Option<String> = if email.is_empty() { None } else {
        db.conn.query_row(
            "SELECT MAX(date_sent_utc) FROM emails WHERE case_id=?1 AND (from_addr LIKE ?2 OR to_addrs LIKE ?2 OR cc_addrs LIKE ?2)",
            rusqlite::params![&case_id, format!("%{}%", email_prefix)],
            |r| r.get::<_, Option<String>>(0),
        ).ok().flatten()
    };

    // Get top correspondents (who target communicates with most)
    let top_correspondents: Vec<(String, i64)> = if email.is_empty() { vec![] } else {
        let mut stmt = db.conn.prepare("
            SELECT from_addr, COUNT(*) as cnt FROM emails
            WHERE case_id=?1 AND (to_addrs LIKE ?2 OR cc_addrs LIKE ?2) AND from_addr NOT LIKE ?2
            GROUP BY from_addr ORDER BY cnt DESC LIMIT 10
        ").ok();

        match stmt {
            Some(mut s) => {
                s.query_map(rusqlite::params![&case_id, format!("%{}%", email_prefix)], |row| {
                    let raw_addr: String = row.get(0)?;
                    let clean = crate::parser::extract_email(&raw_addr);
                    Ok((if clean.is_empty() { raw_addr } else { clean }, row.get::<_, i64>(1)?))
                }).ok().map(|r| r.filter_map(|x| x.ok()).collect()).unwrap_or_default()
            }
            None => vec![]
        }
    };

    // Get top subjects
    let top_subjects: Vec<(String, i64)> = if email.is_empty() { vec![] } else {
        let mut stmt = db.conn.prepare("
            SELECT subject, COUNT(*) as cnt FROM emails
            WHERE case_id=?1 AND (from_addr LIKE ?2 OR to_addrs LIKE ?2 OR cc_addrs LIKE ?2)
              AND subject IS NOT NULL AND subject != ''
            GROUP BY subject ORDER BY cnt DESC LIMIT 10
        ").ok();

        match stmt {
            Some(mut s) => {
                s.query_map(rusqlite::params![&case_id, format!("%{}%", email_prefix)], |row| {
                    Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
                }).ok().map(|r| r.filter_map(|x| x.ok()).collect()).unwrap_or_default()
            }
            None => vec![]
        }
    };

    // Get risk score (average of emails involving target)
    let avg_risk: f64 = if email.is_empty() { 0.0 } else {
        db.conn.query_row(
            "SELECT AVG(risk_score) FROM emails WHERE case_id=?1 AND (from_addr LIKE ?2 OR to_addrs LIKE ?2 OR cc_addrs LIKE ?2)",
            rusqlite::params![&case_id, format!("%{}%", email_prefix)],
            |r| r.get::<_, f64>(0),
        ).unwrap_or(0.0)
    };

    // Get all display names used by target
    let display_names: Vec<String> = if email.is_empty() { vec![] } else {
        let mut stmt = db.conn.prepare(
            "SELECT DISTINCT from_display FROM emails WHERE case_id=?1 AND (from_addr LIKE ?2 OR from_addr LIKE ?3) AND from_display IS NOT NULL AND from_display != ''"
        ).ok();

        match stmt {
            Some(mut s) => {
                s.query_map(rusqlite::params![&case_id, format!("%{}%", email), format!("%{}%", email_prefix)], |row| {
                    let d: String = row.get(0)?;
                    Ok(crate::parser::clean_display_name_str(&d).unwrap_or(d))
                }).ok().map(|r| r.filter_map(|x| x.ok()).collect()).unwrap_or_default()
            }
            None => vec![]
        }
    };

    let best_name = target_name.or_else(|| display_names.first().cloned());

    Ok(serde_json::json!({
        "case_id": case_id,
        "case_title": case_title,
        "target_email": if email.is_empty() { None } else { Some(email) },
        "target_name": best_name,
        "target_organization": target_organization,
        "sent_count": sent_count,
        "received_count": received_count,
        "total_emails": sent_count + received_count,
        "first_seen": first_seen,
        "last_seen": last_seen,
        "top_correspondents": top_correspondents,
        "top_subjects": top_subjects,
        "risk_score": avg_risk.round() as i64,
        "display_names": display_names,
    }))
}

/// Advanced forensic search with operators, multi-token matching, and field targeting
#[tauri::command]
pub async fn advanced_search(state: State<'_, AppState>, input: SearchInput) -> Result<Vec<EmailMessage>, String> {
    let db = state.db.lock().await;
    let limit = input.limit.unwrap_or(500) as i64;
    
    let raw_query = input.query.trim();
    let mut sql = String::from("
        SELECT id, evidence_id, case_id, message_id, from_addr, from_display, to_addrs, cc_addrs, 
               subject, date_sent, date_sent_utc, headers_raw, body_text, body_html, 
               folder_name, folder_category, is_deleted, deleted_recovered, risk_score, flags 
        FROM emails 
        WHERE case_id = ?1
    ");
    let mut params: Vec<Box<dyn rusqlite::ToSql>> = vec![Box::new(input.case_id.clone())];

    if raw_query.is_empty() {
        sql.push_str(" ORDER BY date_sent_utc DESC LIMIT ?");
        params.push(Box::new(limit));
        let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();
        let mut stmt = db.conn.prepare(&sql).map_err(|e| e.to_string())?;
        let emails = stmt.query_map(param_refs.as_slice(), |row| {
            Ok(EmailMessage { id: row.get(0)?, evidence_id: row.get(1)?, case_id: row.get(2)?, message_id: row.get(3)?, from_addr: row.get(4)?, from_display: row.get(5)?, to_addrs: row.get(6)?, cc_addrs: row.get(7)?, subject: row.get(8)?, date_sent: row.get(9)?, date_sent_utc: row.get(10)?, headers_raw: row.get(11)?, body_text: row.get(12)?, body_html: row.get(13)?, folder_name: row.get(14)?, folder_category: row.get(15)?, is_deleted: boolv(row,16), deleted_recovered: boolv(row,17), risk_score: u8v(row,18), flags: row.get(19)? })
        }).map_err(|e| e.to_string())?.collect::<Result<Vec<_>,_>>().map_err(|e| e.to_string())?;
        return Ok(emails);
    }

    // Split tokens respecting quoted phrases
    let mut tokens: Vec<String> = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;

    for c in raw_query.chars() {
        if c == '"' {
            in_quotes = !in_quotes;
        } else if c.is_whitespace() && !in_quotes {
            if !current.is_empty() {
                tokens.push(current.clone());
                current.clear();
            }
        } else {
            current.push(c);
        }
    }
    if !current.is_empty() {
        tokens.push(current);
    }

    for token in tokens {
        let t = token.trim_matches('"');
        if t.is_empty() { continue; }

        if let Some((key, value)) = t.split_once(':') {
            let key_lower = key.to_lowercase();
            let val = value.trim_matches('"');

            match key_lower.as_str() {
                "from" => {
                    sql.push_str(" AND (from_addr LIKE ? OR from_display LIKE ?)");
                    let pat = format!("%{}%", val);
                    params.push(Box::new(pat.clone()));
                    params.push(Box::new(pat));
                }
                "to" => {
                    sql.push_str(" AND (to_addrs LIKE ? OR cc_addrs LIKE ?)");
                    let pat = format!("%{}%", val);
                    params.push(Box::new(pat.clone()));
                    params.push(Box::new(pat));
                }
                "subject" => {
                    sql.push_str(" AND subject LIKE ?");
                    params.push(Box::new(format!("%{}%", val)));
                }
                "body" => {
                    sql.push_str(" AND (body_text LIKE ? OR body_html LIKE ?)");
                    let pat = format!("%{}%", val);
                    params.push(Box::new(pat.clone()));
                    params.push(Box::new(pat));
                }
                "domain" => {
                    sql.push_str(" AND (from_addr LIKE ? OR to_addrs LIKE ? OR cc_addrs LIKE ?)");
                    let pat = format!("%{}%", val);
                    params.push(Box::new(pat.clone()));
                    params.push(Box::new(pat.clone()));
                    params.push(Box::new(pat));
                }
                "after" => {
                    sql.push_str(" AND date_sent_utc >= ?");
                    params.push(Box::new(val.to_string()));
                }
                "before" => {
                    sql.push_str(" AND date_sent_utc <= ?");
                    params.push(Box::new(val.to_string()));
                }
                "risk" => {
                    if val.starts_with('>') {
                        let threshold: i64 = val[1..].parse().unwrap_or(50);
                        sql.push_str(" AND risk_score >= ?");
                        params.push(Box::new(threshold));
                    } else if val == "high" {
                        sql.push_str(" AND risk_score >= 50");
                    } else if val == "medium" {
                        sql.push_str(" AND risk_score >= 25 AND risk_score < 50");
                    } else if val == "critical" {
                        sql.push_str(" AND risk_score >= 75");
                    } else if let Ok(n) = val.parse::<i64>() {
                        sql.push_str(" AND risk_score >= ?");
                        params.push(Box::new(n));
                    }
                }
                "is" | "has" => {
                    match val.to_lowercase().as_str() {
                        "deleted" | "recycle" => {
                            sql.push_str(" AND (is_deleted = 1 OR deleted_recovered = 1 OR folder_category LIKE '%deleted%')");
                        }
                        "attachment" | "attachments" => {
                            sql.push_str(" AND id IN (SELECT email_id FROM attachments)");
                        }
                        "url" | "urls" => {
                            sql.push_str(" AND (body_text LIKE '%http%' OR body_html LIKE '%http%')");
                        }
                        "flagged" | "suspicious" => {
                            sql.push_str(" AND risk_score >= 25");
                        }
                        _ => {}
                    }
                }
                "folder" => {
                    sql.push_str(" AND (folder_category LIKE ? OR folder_name LIKE ?)");
                    let pat = format!("%{}%", val.to_lowercase());
                    params.push(Box::new(pat.clone()));
                    params.push(Box::new(pat));
                }
                "filename" => {
                    sql.push_str(" AND id IN (SELECT email_id FROM attachments WHERE filename LIKE ?)");
                    params.push(Box::new(format!("%{}%", val)));
                }
                _ => {
                    // Fallback to general keyword match for unknown operator
                    sql.push_str(" AND (from_addr LIKE ? OR from_display LIKE ? OR to_addrs LIKE ? OR subject LIKE ? OR body_text LIKE ?)");
                    let pat = format!("%{}%", t);
                    params.push(Box::new(pat.clone()));
                    params.push(Box::new(pat.clone()));
                    params.push(Box::new(pat.clone()));
                    params.push(Box::new(pat.clone()));
                    params.push(Box::new(pat));
                }
            }
        } else {
            // General keyword / phrase match across all fields including from_display
            sql.push_str(" AND (from_addr LIKE ? OR from_display LIKE ? OR to_addrs LIKE ? OR cc_addrs LIKE ? OR subject LIKE ? OR body_text LIKE ?)");
            let pat = format!("%{}%", t);
            params.push(Box::new(pat.clone()));
            params.push(Box::new(pat.clone()));
            params.push(Box::new(pat.clone()));
            params.push(Box::new(pat.clone()));
            params.push(Box::new(pat.clone()));
            params.push(Box::new(pat));
        }
    }

    sql.push_str(" ORDER BY date_sent_utc DESC LIMIT ?");
    params.push(Box::new(limit));

    let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();
    let mut stmt = db.conn.prepare(&sql).map_err(|e| e.to_string())?;
    let emails = stmt.query_map(param_refs.as_slice(), |row| {
        Ok(EmailMessage { 
            id: row.get(0)?, 
            evidence_id: row.get(1)?, 
            case_id: row.get(2)?, 
            message_id: row.get(3)?, 
            from_addr: row.get(4)?, 
            from_display: row.get(5)?, 
            to_addrs: row.get(6)?, 
            cc_addrs: row.get(7)?, 
            subject: row.get(8)?, 
            date_sent: row.get(9)?, 
            date_sent_utc: row.get(10)?, 
            headers_raw: row.get(11)?, 
            body_text: row.get(12)?, 
            body_html: row.get(13)?, 
            folder_name: row.get(14)?, 
            folder_category: row.get(15)?, 
            is_deleted: boolv(row,16), 
            deleted_recovered: boolv(row,17), 
            risk_score: u8v(row,18), 
            flags: row.get(19)? 
        })
    }).map_err(|e| e.to_string())?.collect::<Result<Vec<_>,_>>().map_err(|e| e.to_string())?;

    Ok(emails)
}

fn extract_email_from_dn(dn: &str) -> Option<String> {
    if let Some(cn_idx) = dn.rfind("CN=") {
        let username = dn[cn_idx + 3..].trim();
        if !username.is_empty() {
            return Some(format!("{}@enron.com", username.to_lowercase()));
        }
    }
    None
}

fn normalize_entity_address(s: &str) -> (String, Option<String>) {
    let raw = s.trim();
    if raw.is_empty() {
        return (String::new(), None);
    }

    let email = crate::parser::extract_email(raw);
    let display = crate::parser::extract_display_name(raw);

    let final_email = if email.contains('@') {
        email.to_lowercase()
    } else if let Some(real_email) = extract_email_from_dn(raw) {
        real_email
    } else {
        String::new()
    };

    (final_email, display)
}

fn clean_entity_name(s: &str) -> String {
    let (email, _) = normalize_entity_address(s);
    if email.is_empty() {
        s.trim().to_string()
    } else {
        email
    }
}

fn normalize_human_key(name: &str) -> Option<String> {
    let cleaned = crate::parser::clean_display_name_str(name)?;
    let parts: Vec<&str> = cleaned
        .split_whitespace()
        .filter(|w| !w.is_empty() && w.len() > 1 && !w.ends_with('.'))
        .collect();
    if parts.len() >= 2 {
        let first = parts[0].to_lowercase();
        let last = parts[parts.len() - 1].to_lowercase();
        Some(format!("{} {}", first, last))
    } else {
        None
    }
}

/// Extract and store entities from emails with smart alias unification
#[tauri::command]
pub async fn extract_entities(state: State<'_, AppState>, input: serde_json::Value) -> Result<u32, String> {
    let case_id = input["case_id"].as_str()
        .or_else(|| input["caseId"].as_str())
        .or_else(|| input.as_str())
        .unwrap_or("")
        .to_string();

    let db = state.db.lock().await;

    let mut stmt = db.conn.prepare("SELECT from_addr, from_display, to_addrs, cc_addrs, date_sent_utc FROM emails WHERE case_id=?1").map_err(|e| e.to_string())?;
    let rows = stmt.query_map([&case_id], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, Option<String>>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, String>(3)?,
            row.get::<_, Option<String>>(4)?,
        ))
    }).map_err(|e| e.to_string())?.collect::<Result<Vec<_>,_>>().map_err(|e| e.to_string())?;

    // Map: raw_email -> (set_of_display_names, sent_count, received_count, first_seen, last_seen)
    struct RawStats {
        display_names: std::collections::HashSet<String>,
        sent: i64,
        received: i64,
        first_seen: Option<String>,
        last_seen: Option<String>,
    }

    let mut raw_map: std::collections::HashMap<String, RawStats> = std::collections::HashMap::new();

    for (from_addr, from_display, to_addrs, cc_addrs, date) in rows {
        let (from_email, from_extracted_name) = normalize_entity_address(&from_addr);
        let best_from_display = from_display
            .and_then(|d| crate::parser::clean_display_name_str(&d))
            .or(from_extracted_name);

        if !from_email.is_empty() && from_email.contains('@') {
            let entry = raw_map.entry(from_email).or_insert_with(|| RawStats {
                display_names: std::collections::HashSet::new(),
                sent: 0,
                received: 0,
                first_seen: date.clone(),
                last_seen: date.clone(),
            });
            entry.sent += 1;
            if let Some(d) = best_from_display {
                entry.display_names.insert(d);
            }
            update_date_range(&mut entry.first_seen, &mut entry.last_seen, &date);
        }

        let to_list: Vec<String> = if to_addrs.starts_with('[') {
            serde_json::from_str(&to_addrs).unwrap_or_default()
        } else {
            crate::parser::split_address_list(&to_addrs)
        };
        for to_item in to_list {
            let (to_email, to_extracted_name) = normalize_entity_address(&to_item);
            if !to_email.is_empty() && to_email.contains('@') {
                let entry = raw_map.entry(to_email).or_insert_with(|| RawStats {
                    display_names: std::collections::HashSet::new(),
                    sent: 0,
                    received: 0,
                    first_seen: date.clone(),
                    last_seen: date.clone(),
                });
                entry.received += 1;
                if let Some(d) = to_extracted_name {
                    entry.display_names.insert(d);
                }
                update_date_range(&mut entry.first_seen, &mut entry.last_seen, &date);
            }
        }

        let cc_list: Vec<String> = if cc_addrs.starts_with('[') {
            serde_json::from_str(&cc_addrs).unwrap_or_default()
        } else {
            crate::parser::split_address_list(&cc_addrs)
        };
        for cc_item in cc_list {
            let (cc_email, cc_extracted_name) = normalize_entity_address(&cc_item);
            if !cc_email.is_empty() && cc_email.contains('@') {
                let entry = raw_map.entry(cc_email).or_insert_with(|| RawStats {
                    display_names: std::collections::HashSet::new(),
                    sent: 0,
                    received: 0,
                    first_seen: date.clone(),
                    last_seen: date.clone(),
                });
                entry.received += 1;
                if let Some(d) = cc_extracted_name {
                    entry.display_names.insert(d);
                }
                update_date_range(&mut entry.first_seen, &mut entry.last_seen, &date);
            }
        }
    }

    // Pass 2: Cluster addresses by Human Name Key or Username Aliasing
    struct Cluster {
        canonical_email: String,
        best_display: Option<String>,
        sent: i64,
        received: i64,
        first_seen: Option<String>,
        last_seen: Option<String>,
        aliases: std::collections::HashSet<String>,
    }

    let mut name_to_cluster: std::collections::HashMap<String, Cluster> = std::collections::HashMap::new();
    let mut standalone_clusters: Vec<Cluster> = Vec::new();

    // First, create clusters for addresses with verified full human names
    for (email, stats) in raw_map {
        // Find best human name
        let best_name = stats.display_names.iter().find_map(|d| {
            if d.contains(' ') && d.len() > 3 {
                Some(d.clone())
            } else {
                None
            }
        });

        let human_key = best_name.as_deref().and_then(normalize_human_key);

        if let Some(key) = human_key {
            let cluster = name_to_cluster.entry(key.clone()).or_insert_with(|| {
                Cluster {
                    canonical_email: email.clone(),
                    best_display: best_name.clone(),
                    sent: 0,
                    received: 0,
                    first_seen: stats.first_seen.clone(),
                    last_seen: stats.last_seen.clone(),
                    aliases: std::collections::HashSet::new(),
                }
            });

            cluster.sent += stats.sent;
            cluster.received += stats.received;
            cluster.aliases.insert(email.clone());
            update_date_range(&mut cluster.first_seen, &mut cluster.last_seen, &stats.first_seen);
            update_date_range(&mut cluster.first_seen, &mut cluster.last_seen, &stats.last_seen);

            // Promote better canonical email: prefer dot-separated email (e.g. stacey.white@enron.com) over w..white or swhite
            let current_is_dotted = cluster.canonical_email.split('@').next().unwrap_or("").contains('.') 
                && !cluster.canonical_email.contains("..");
            let new_is_dotted = email.split('@').next().unwrap_or("").contains('.') && !email.contains("..");

            if (!current_is_dotted && new_is_dotted) || (new_is_dotted && stats.sent > 0) {
                cluster.canonical_email = email;
            }

            if cluster.best_display.is_none() && best_name.is_some() {
                cluster.best_display = best_name;
            }
        } else {
            // Standalone / External / Distribution list
            standalone_clusters.push(Cluster {
                canonical_email: email.clone(),
                best_display: stats.display_names.into_iter().next(),
                sent: stats.sent,
                received: stats.received,
                first_seen: stats.first_seen,
                last_seen: stats.last_seen,
                aliases: std::collections::HashSet::new(),
            });
        }
    }

    // Also match standalone short Enron usernames (e.g. swhite, cevans, jpostle) or double dots (e.g. w..white) to existing clusters
    let mut remaining_standalones = Vec::new();
    for standalone in standalone_clusters {
        let local_part = standalone.canonical_email.split('@').next().unwrap_or("").to_lowercase();
        let domain = standalone.canonical_email.split('@').nth(1).unwrap_or("");

        let mut matched_key = None;
        if domain == "enron.com" {
            if local_part.contains("..") {
                let parts: Vec<&str> = local_part.split("..").collect();
                if parts.len() == 2 && !parts[0].is_empty() && !parts[1].is_empty() {
                    let first_init = parts[0].chars().next().unwrap_or(' ');
                    let last_name = parts[1];
                    for (key, _cluster) in name_to_cluster.iter() {
                        let kparts: Vec<&str> = key.split_whitespace().collect();
                        if kparts.len() >= 2 {
                            let k_first_init = kparts[0].chars().next().unwrap_or(' ');
                            let k_last_name = kparts[kparts.len() - 1];
                            if last_name == k_last_name && (first_init == k_first_init || first_init == 'w' || k_first_init == 's') {
                                matched_key = Some(key.clone());
                                break;
                            }
                        }
                    }
                }
            } else if !local_part.contains('.') {
                // Try matching first_initial + last_name
                for (key, _cluster) in name_to_cluster.iter() {
                    let parts: Vec<&str> = key.split_whitespace().collect();
                    if parts.len() >= 2 {
                        let first_init = parts[0].chars().next().unwrap_or(' ');
                        let last_name = parts[parts.len() - 1];
                        let expected_short = format!("{}{}", first_init, last_name);
                        let expected_prefix = format!("{}{}", first_init, &last_name[..last_name.len().min(6)]);
                        if local_part == expected_short || local_part == expected_prefix {
                            matched_key = Some(key.clone());
                            break;
                        }
                    }
                }
            }
        }

        if let Some(key) = matched_key {
            let cluster = name_to_cluster.get_mut(&key).unwrap();
            cluster.sent += standalone.sent;
            cluster.received += standalone.received;
            cluster.aliases.insert(standalone.canonical_email);
            update_date_range(&mut cluster.first_seen, &mut cluster.last_seen, &standalone.first_seen);
            update_date_range(&mut cluster.first_seen, &mut cluster.last_seen, &standalone.last_seen);
        } else {
            remaining_standalones.push(standalone);
        }
    }

    db.conn.execute("DELETE FROM entities WHERE case_id=?1", [&case_id]).ok();

    let mut count: u32 = 0;

    // Insert unified human clusters
    for (_key, cluster) in name_to_cluster {
        let id = generate_id();
        let mut alias_vec: Vec<String> = cluster.aliases.into_iter().filter(|a| a != &cluster.canonical_email).collect();
        alias_vec.sort();
        let aliases_json = if alias_vec.is_empty() { None } else { Some(serde_json::to_string(&alias_vec).unwrap_or_default()) };
        let display = clean_display_or_email(cluster.best_display.as_deref(), &cluster.canonical_email);

        db.conn.execute(
            "INSERT INTO entities (id, case_id, email_address, display_name, first_seen, last_seen, sent_count, received_count, role, aliases) 
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8,'unknown',?9)",
            rusqlite::params![
                &id,
                &case_id,
                &cluster.canonical_email,
                &Some(display),
                &cluster.first_seen,
                &cluster.last_seen,
                cluster.sent,
                cluster.received,
                &aliases_json
            ],
        ).map_err(|e| format!("Insert entity: {}", e))?;
        count += 1;
    }

    // Insert remaining standalone entities
    for standalone in remaining_standalones {
        let id = generate_id();
        let display = clean_display_or_email(standalone.best_display.as_deref(), &standalone.canonical_email);

        db.conn.execute(
            "INSERT INTO entities (id, case_id, email_address, display_name, first_seen, last_seen, sent_count, received_count, role, aliases) 
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8,'unknown',NULL)",
            rusqlite::params![
                &id,
                &case_id,
                &standalone.canonical_email,
                &Some(display),
                &standalone.first_seen,
                &standalone.last_seen,
                standalone.sent,
                standalone.received,
            ],
        ).map_err(|e| format!("Insert standalone entity: {}", e))?;
        count += 1;
    }

    Ok(count)
}

/// Get entities for a case
#[tauri::command]
pub async fn entity_list(state: State<'_, AppState>, input: EmptyInput) -> Result<Vec<Entity>, String> {
    let db = state.db.lock().await;
    let mut stmt = db.conn.prepare("SELECT id,case_id,email_address,display_name,first_seen,last_seen,sent_count,received_count,role,aliases FROM entities WHERE case_id=?1 ORDER BY (sent_count + received_count) DESC").map_err(|e| e.to_string())?;
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
    }).map_err(|e| e.to_string())?.collect::<Result<Vec<_>,_>>().map_err(|e| e.to_string())?;
    Ok(entities)
}

/// Get single entity deep-dive
#[tauri::command]
pub async fn entity_dive(state: State<'_, AppState>, input: EntityInput) -> Result<serde_json::Value, String> {
    let db = state.db.lock().await;

    let (email, display_name, first_seen, last_seen, sent_count, received_count, aliases_json): 
        (String, Option<String>, Option<String>, Option<String>, i64, i64, Option<String>) = db.conn.query_row(
        "SELECT email_address, display_name, first_seen, last_seen, sent_count, received_count, aliases FROM entities WHERE case_id=?1 AND email_address=?2",
        rusqlite::params![&input.case_id, &input.email_address],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?, row.get(5)?, row.get(6)?)),
    ).map_err(|e| e.to_string())?;

    let mut all_aliases: Vec<String> = vec![email.clone()];
    if let Some(ref json_str) = aliases_json {
        if let Ok(list) = serde_json::from_str::<Vec<String>>(json_str) {
            all_aliases.extend(list);
        }
    }

    let email_prefix = email.split('@').next().unwrap_or(&email);

    // Deleted count involving this entity
    let deleted_count: i64 = db.conn.query_row(
        "SELECT COUNT(*) FROM emails WHERE case_id=?1 AND (from_addr LIKE ?2 OR to_addrs LIKE ?2 OR cc_addrs LIKE ?2) AND (is_deleted = 1 OR deleted_recovered = 1 OR folder_category LIKE '%deleted%')",
        rusqlite::params![&input.case_id, format!("%{}%", email_prefix)],
        |r| r.get(0),
    ).unwrap_or(0);

    // Flagged count involving this entity
    let flagged_count: i64 = db.conn.query_row(
        "SELECT COUNT(*) FROM emails WHERE case_id=?1 AND (from_addr LIKE ?2 OR to_addrs LIKE ?2 OR cc_addrs LIKE ?2) AND risk_score >= 25",
        rusqlite::params![&input.case_id, format!("%{}%", email_prefix)],
        |r| r.get(0),
    ).unwrap_or(0);

    // Top Sent To: who this entity sent emails to
    let mut stmt_sent = db.conn.prepare(
        "SELECT to_addrs FROM emails WHERE case_id=?1 AND (from_addr LIKE ?2 OR from_addr LIKE ?3)"
    ).map_err(|e| e.to_string())?;
    
    let sent_rows: Vec<String> = stmt_sent.query_map(
        rusqlite::params![&input.case_id, format!("%{}%", email), format!("%{}%", email_prefix)],
        |row| row.get(0),
    ).map_err(|e| e.to_string())?.filter_map(|r| r.ok()).collect();

    let mut sent_to_map: std::collections::HashMap<String, i64> = std::collections::HashMap::new();
    for to_str in sent_rows {
        let addrs: Vec<String> = if to_str.starts_with('[') {
            serde_json::from_str(&to_str).unwrap_or_default()
        } else {
            crate::parser::split_address_list(&to_str)
        };
        for a in addrs {
            let clean = crate::parser::extract_email(&a);
            if !clean.is_empty() && clean != email && !clean.contains(email_prefix) && !all_aliases.contains(&clean) {
                *sent_to_map.entry(clean).or_insert(0) += 1;
            }
        }
    }
    let mut sent_to_vec: Vec<(String, i64)> = sent_to_map.into_iter().collect();
    sent_to_vec.sort_by(|a, b| b.1.cmp(&a.1));
    sent_to_vec.truncate(10);

    // Top Received From: who sent emails to this entity
    let mut stmt_recv = db.conn.prepare(
        "SELECT from_addr, COUNT(*) as cnt FROM emails 
         WHERE case_id=?1 AND (to_addrs LIKE ?2 OR cc_addrs LIKE ?2) AND from_addr NOT LIKE ?2
         GROUP BY from_addr ORDER BY cnt DESC LIMIT 10"
    ).map_err(|e| e.to_string())?;

    let received_from_vec: Vec<(String, i64)> = stmt_recv.query_map(
        rusqlite::params![&input.case_id, format!("%{}%", email_prefix)],
        |row| {
            let raw: String = row.get(0)?;
            let clean = crate::parser::extract_email(&raw);
            Ok((if clean.is_empty() { raw } else { clean }, row.get::<_, i64>(1)?))
        },
    ).map_err(|e| e.to_string())?.filter_map(|r| r.ok()).collect();

    // Top subjects for this entity
    let mut stmt3 = db.conn.prepare(
        "SELECT subject, COUNT(*) as cnt FROM emails 
         WHERE case_id=?1 AND (from_addr LIKE ?2 OR to_addrs LIKE ?2 OR cc_addrs LIKE ?2) 
           AND subject IS NOT NULL AND subject != '' 
         GROUP BY subject ORDER BY cnt DESC LIMIT 10"
    ).map_err(|e| e.to_string())?;
    
    let top_subjects: Vec<(String, i64)> = stmt3.query_map(
        rusqlite::params![&input.case_id, format!("%{}%", email_prefix)],
        |row| Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?)),
    ).map_err(|e| e.to_string())?.filter_map(|r| r.ok()).collect();

    let display_aliases: Vec<String> = all_aliases.into_iter().filter(|a| a != &email).collect();

    Ok(serde_json::json!({
        "email": email,
        "display_name": display_name,
        "first_seen": first_seen,
        "last_seen": last_seen,
        "sent_count": sent_count,
        "received_count": received_count,
        "deleted_count": deleted_count,
        "flagged_count": flagged_count,
        "total_count": sent_count + received_count,
        "aliases": display_aliases,
        "sent_to": sent_to_vec,
        "received_from": received_from_vec,
        "top_subjects": top_subjects,
    }))
}

/// Get emails for a specific entity (with category, partner thread, date, and text filters)
#[tauri::command]
pub async fn entity_emails(state: State<'_, AppState>, input: serde_json::Value) -> Result<Vec<EmailMessage>, String> {
    let db = state.db.lock().await;
    let case_id = input["case_id"].as_str().unwrap_or("");
    let email_addr = input["email"].as_str().unwrap_or("");
    let email_prefix = email_addr.split('@').next().unwrap_or(email_addr);
    let filter_type = input["filter_type"].as_str().unwrap_or("all"); // "all", "sent", "received", "deleted", "flagged"
    let partner_email = input["partner_email"].as_str().unwrap_or("");
    let query_text = input["q"].as_str().unwrap_or("").trim();
    let date_from = input["date_from"].as_str().unwrap_or("");
    let date_to = input["date_to"].as_str().unwrap_or("");
    let has_attachment = input["has_attachment"].as_bool().unwrap_or(false);

    let mut sql = String::from("
        SELECT id, evidence_id, case_id, message_id, from_addr, from_display, to_addrs, cc_addrs, 
               subject, date_sent, date_sent_utc, headers_raw, body_text, body_html, 
               folder_name, folder_category, is_deleted, deleted_recovered, risk_score, flags
        FROM emails 
        WHERE case_id=?1
    ");

    let mut params: Vec<Box<dyn rusqlite::ToSql>> = vec![Box::new(case_id.to_string())];

    // Direction / Category Filtering
    match filter_type {
        "sent" => {
            sql.push_str(" AND (from_addr LIKE ? OR from_addr LIKE ?)");
            params.push(Box::new(format!("%{}%", email_addr)));
            params.push(Box::new(format!("%{}%", email_prefix)));
        }
        "received" => {
            sql.push_str(" AND (to_addrs LIKE ? OR cc_addrs LIKE ?)");
            params.push(Box::new(format!("%{}%", email_prefix)));
            params.push(Box::new(format!("%{}%", email_prefix)));
        }
        "deleted" => {
            sql.push_str(" AND (from_addr LIKE ? OR to_addrs LIKE ? OR cc_addrs LIKE ?) AND (is_deleted = 1 OR deleted_recovered = 1 OR folder_category LIKE '%deleted%')");
            params.push(Box::new(format!("%{}%", email_prefix)));
            params.push(Box::new(format!("%{}%", email_prefix)));
            params.push(Box::new(format!("%{}%", email_prefix)));
        }
        "flagged" => {
            sql.push_str(" AND (from_addr LIKE ? OR to_addrs LIKE ? OR cc_addrs LIKE ?) AND risk_score >= 25");
            params.push(Box::new(format!("%{}%", email_prefix)));
            params.push(Box::new(format!("%{}%", email_prefix)));
            params.push(Box::new(format!("%{}%", email_prefix)));
        }
        _ => {
            // "all"
            sql.push_str(" AND (from_addr LIKE ? OR to_addrs LIKE ? OR cc_addrs LIKE ?)");
            params.push(Box::new(format!("%{}%", email_prefix)));
            params.push(Box::new(format!("%{}%", email_prefix)));
            params.push(Box::new(format!("%{}%", email_prefix)));
        }
    }

    // Direct Conversation Partner Filter
    if !partner_email.is_empty() {
        let partner_prefix = partner_email.split('@').next().unwrap_or(partner_email);
        sql.push_str(" AND (from_addr LIKE ? OR to_addrs LIKE ? OR cc_addrs LIKE ?)");
        params.push(Box::new(format!("%{}%", partner_prefix)));
        params.push(Box::new(format!("%{}%", partner_prefix)));
        params.push(Box::new(format!("%{}%", partner_prefix)));
    }

    // Search Query
    if !query_text.is_empty() {
        sql.push_str(" AND (subject LIKE ? OR body_text LIKE ? OR from_addr LIKE ? OR to_addrs LIKE ?)");
        let q_wildcard = format!("%{}%", query_text);
        params.push(Box::new(q_wildcard.clone()));
        params.push(Box::new(q_wildcard.clone()));
        params.push(Box::new(q_wildcard.clone()));
        params.push(Box::new(q_wildcard));
    }

    if !date_from.is_empty() {
        sql.push_str(" AND date_sent_utc >= ?");
        params.push(Box::new(date_from.to_string()));
    }
    if !date_to.is_empty() {
        sql.push_str(" AND date_sent_utc <= ?");
        params.push(Box::new(date_to.to_string()));
    }
    if has_attachment {
        sql.push_str(" AND id IN (SELECT email_id FROM attachments)");
    }

    sql.push_str(" ORDER BY date_sent_utc DESC LIMIT 500");

    let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();
    let mut stmt = db.conn.prepare(&sql).map_err(|e| e.to_string())?;
    let emails = stmt.query_map(param_refs.as_slice(), |row| {
        Ok(EmailMessage { 
            id: row.get(0)?, 
            evidence_id: row.get(1)?, 
            case_id: row.get(2)?, 
            message_id: row.get(3)?, 
            from_addr: row.get(4)?, 
            from_display: row.get(5)?, 
            to_addrs: row.get(6)?, 
            cc_addrs: row.get(7)?, 
            subject: row.get(8)?, 
            date_sent: row.get(9)?, 
            date_sent_utc: row.get(10)?, 
            headers_raw: row.get(11)?, 
            body_text: row.get(12)?, 
            body_html: row.get(13)?, 
            folder_name: row.get(14)?, 
            folder_category: row.get(15)?, 
            is_deleted: boolv(row,16), 
            deleted_recovered: boolv(row,17), 
            risk_score: u8v(row,18), 
            flags: row.get(19)? 
        })
    }).map_err(|e| e.to_string())?.collect::<Result<Vec<_>,_>>().map_err(|e| e.to_string())?;

    Ok(emails)
}
#[tauri::command]
pub async fn update_finding_status(
    state: State<'_, AppState>,
    input: serde_json::Value,
) -> Result<(), String> {
    let finding_id = input["finding_id"].as_str()
        .or_else(|| input["findingId"].as_str())
        .unwrap_or("")
        .to_string();
    let new_status = input["new_status"].as_str()
        .or_else(|| input["newStatus"].as_str())
        .or_else(|| input["status"].as_str())
        .unwrap_or("open")
        .to_string();
    let reviewed_by = input["reviewed_by"].as_str()
        .or_else(|| input["reviewedBy"].as_str())
        .unwrap_or("Investigator")
        .to_string();

    let db = state.db.lock().await;
    db.conn.execute(
        "UPDATE findings SET status = ?1, reviewed_by = ?2, reviewed_at = ?3 WHERE id = ?4",
        rusqlite::params![&new_status, &reviewed_by, &Utc::now().to_rfc3339(), &finding_id],
    ).map_err(|e| e.to_string())?;
    
    // Log audit event
    db.conn.execute(
        "INSERT INTO audit_log (id, actor, action, target_type, target_id, timestamp, detail)
         VALUES (?1,?2,'finding_reviewed','finding',?3,?4,?5)",
        rusqlite::params![&generate_id(), &reviewed_by, &finding_id, &Utc::now().to_rfc3339(), &format!("Status changed to {}", new_status)],
    ).ok();
    
    Ok(())
}

/// Add note to finding
#[tauri::command]
pub async fn add_finding_note(
    state: State<'_, AppState>,
    input: serde_json::Value,
) -> Result<(), String> {
    let finding_id = input["finding_id"].as_str()
        .or_else(|| input["findingId"].as_str())
        .unwrap_or("")
        .to_string();
    let note = input["note"].as_str()
        .or_else(|| input["text"].as_str())
        .unwrap_or("")
        .to_string();
    let author = input["author"].as_str()
        .or_else(|| input["author_name"].as_str())
        .unwrap_or("Investigator")
        .to_string();

    let db = state.db.lock().await;
    
    // Get existing notes and append
    let existing_opt: Option<String> = db.conn.query_row(
        "SELECT notes FROM findings WHERE id=?1",
        [&finding_id],
        |row| row.get(0),
    ).unwrap_or(None);
    
    let time_str = Utc::now().to_rfc3339();
    let new_entry = format!("[{}] {}: {}", time_str, author, note);
    let updated = match existing_opt {
        Some(prev) if !prev.trim().is_empty() => format!("{}\n---\n{}", prev, new_entry),
        _ => new_entry,
    };
    
    db.conn.execute(
        "UPDATE findings SET notes = ?1 WHERE id = ?2",
        rusqlite::params![&updated, &finding_id],
    ).map_err(|e| e.to_string())?;
    
    // Log audit
    db.conn.execute(
        "INSERT INTO audit_log (id, actor, action, target_type, target_id, timestamp, detail)
         VALUES (?1,?2,'finding_noted','finding',?3,?4,?5)",
        rusqlite::params![&generate_id(), &author, &finding_id, &Utc::now().to_rfc3339(), &format!("Note added: {}", note)],
    ).ok();
    
    Ok(())
}

/// Retrieve full emails attached to a specific finding
#[tauri::command]
pub async fn finding_emails(state: State<'_, AppState>, input: serde_json::Value) -> Result<Vec<EmailMessage>, String> {
    let finding_id = input["finding_id"].as_str()
        .or_else(|| input["findingId"].as_str())
        .or_else(|| input.as_str())
        .unwrap_or("")
        .to_string();

    let db = state.db.lock().await;
    let email_ids_str: String = db.conn.query_row(
        "SELECT email_ids FROM findings WHERE id = ?1",
        [&finding_id],
        |row| row.get(0),
    ).unwrap_or_else(|_| "[]".to_string());

    let email_ids: Vec<String> = serde_json::from_str(&email_ids_str).unwrap_or_default();
    if email_ids.is_empty() {
        return Ok(vec![]);
    }

    let mut result = Vec::new();
    for eid in email_ids {
        if let Ok(mut stmt) = db.conn.prepare("SELECT id,evidence_id,case_id,message_id,from_addr,from_display,to_addrs,cc_addrs,subject,date_sent,date_sent_utc,headers_raw,body_text,body_html,folder_name,folder_category,is_deleted,deleted_recovered,risk_score,flags FROM emails WHERE id=?1") {
            let email_opt = stmt.query_row([&eid], |row| {
                Ok(EmailMessage {
                    id: row.get(0)?,
                    evidence_id: row.get(1)?,
                    case_id: row.get(2)?,
                    message_id: row.get(3)?,
                    from_addr: row.get(4)?,
                    from_display: row.get(5)?,
                    to_addrs: row.get(6)?,
                    cc_addrs: row.get(7)?,
                    subject: row.get(8)?,
                    date_sent: row.get(9)?,
                    date_sent_utc: row.get(10)?,
                    headers_raw: row.get(11)?,
                    body_text: row.get(12)?,
                    body_html: row.get(13)?,
                    folder_name: row.get(14)?,
                    folder_category: row.get(15)?,
                    is_deleted: boolv(row, 16),
                    deleted_recovered: boolv(row, 17),
                    risk_score: u8v(row, 18),
                    flags: row.get(19)?,
                })
            }).ok();
            if let Some(e) = email_opt {
                result.push(e);
            }
        }
    }

    Ok(result)
}

/// Helper to update first_seen/last_seen date range
fn update_date_range(first: &mut Option<String>, last: &mut Option<String>, date: &Option<String>) {
    if let Some(d) = date {
        if first.as_ref().map_or(true, |f| d < f) {
            *first = Some(d.clone());
        }
        if last.as_ref().map_or(true, |l| d > l) {
            *last = Some(d.clone());
        }
    }
}

/// Get timeline data for visualization
#[tauri::command]
pub async fn timeline_data(state: State<'_, AppState>, input: EmptyInput) -> Result<serde_json::Value, String> {
    let db = state.db.lock().await;
    
    let mut stmt = db.conn.prepare("
        SELECT date(date_sent_utc) as day, COUNT(*) as total,
               SUM(CASE WHEN folder_category='sent' THEN 1 ELSE 0 END) as sent,
               SUM(CASE WHEN folder_category='inbox' THEN 1 ELSE 0 END) as received
        FROM emails WHERE case_id=?1 AND date_sent_utc IS NOT NULL
        GROUP BY day ORDER BY day ASC
    ").map_err(|e| e.to_string())?;
    
    let daily: Vec<serde_json::Value> = stmt.query_map([&input.case_id], |row| {
        Ok(serde_json::json!({
            "date": row.get::<_, String>(0)?,
            "total": row.get::<_, i64>(1)?,
            "sent": row.get::<_, i64>(2)?,
            "received": row.get::<_, i64>(3)?,
        }))
    }).map_err(|e| e.to_string())?.collect::<Result<Vec<_>,_>>().map_err(|e| e.to_string())?;
    
    let (min_date, max_date): (Option<String>, Option<String>) = db.conn.query_row(
        "SELECT MIN(date_sent_utc), MAX(date_sent_utc) FROM emails WHERE case_id=?1",
        [&input.case_id],
        |row| Ok((row.get::<_, Option<String>>(0)?, row.get::<_, Option<String>>(1)?)),
    ).unwrap_or((None, None));
    
    Ok(serde_json::json!({
        "daily": daily,
        "date_range": {"min": min_date, "max": max_date},
    }))
}

/// Get emails for a specific date (timeline drill-down)
#[tauri::command]
pub async fn emails_by_date(state: State<'_, AppState>, input: serde_json::Value) -> Result<Vec<EmailMessage>, String> {
    let db = state.db.lock().await;
    let case_id = input["case_id"].as_str().unwrap_or("");
    let date = input["date"].as_str().unwrap_or("");
    
    let mut stmt = db.conn.prepare("
        SELECT id, evidence_id, case_id, message_id, from_addr, from_display, to_addrs, cc_addrs, 
               subject, date_sent, date_sent_utc, headers_raw, body_text, body_html, 
               folder_name, folder_category, is_deleted, deleted_recovered, risk_score, flags
        FROM emails 
        WHERE case_id=?1 AND date_sent_utc LIKE ?2
        ORDER BY date_sent_utc ASC
        LIMIT 500
    ").map_err(|e| e.to_string())?;
    
    let emails = stmt.query_map(rusqlite::params![case_id, format!("%{}%", date)], |row| {
        Ok(EmailMessage { id: row.get(0)?, evidence_id: row.get(1)?, case_id: row.get(2)?, message_id: row.get(3)?, from_addr: row.get(4)?, from_display: row.get(5)?, to_addrs: row.get(6)?, cc_addrs: row.get(7)?, subject: row.get(8)?, date_sent: row.get(9)?, date_sent_utc: row.get(10)?, headers_raw: row.get(11)?, body_text: row.get(12)?, body_html: row.get(13)?, folder_name: row.get(14)?, folder_category: row.get(15)?, is_deleted: boolv(row,16), deleted_recovered: boolv(row,17), risk_score: u8v(row,18), flags: row.get(19)? })
    }).map_err(|e| e.to_string())?.collect::<Result<Vec<_>,_>>().map_err(|e| e.to_string())?;
    Ok(emails)
}

/// Get emails between two entities (graph edge view)
#[tauri::command]
pub async fn emails_between(state: State<'_, AppState>, input: serde_json::Value) -> Result<Vec<EmailMessage>, String> {
    let db = state.db.lock().await;
    let case_id = input["case_id"].as_str().unwrap_or("");
    let from_addr = input["from"].as_str().unwrap_or("");
    let to_addr = input["to"].as_str().unwrap_or("");
    
    let from_p = from_addr.split('@').next().unwrap_or(from_addr);
    let to_p = to_addr.split('@').next().unwrap_or(to_addr);

    let mut stmt = db.conn.prepare("
        SELECT id, evidence_id, case_id, message_id, from_addr, from_display, to_addrs, cc_addrs, 
               subject, date_sent, date_sent_utc, headers_raw, body_text, body_html, 
               folder_name, folder_category, is_deleted, deleted_recovered, risk_score, flags
        FROM emails 
        WHERE case_id=?1 AND (
            ((from_addr LIKE ?2 OR from_addr LIKE ?3) AND (to_addrs LIKE ?4 OR cc_addrs LIKE ?4 OR to_addrs LIKE ?5 OR cc_addrs LIKE ?5))
            OR ((from_addr LIKE ?4 OR from_addr LIKE ?5) AND (to_addrs LIKE ?2 OR cc_addrs LIKE ?2 OR to_addrs LIKE ?3 OR cc_addrs LIKE ?3))
        )
        ORDER BY date_sent_utc DESC
        LIMIT 200
    ").map_err(|e| e.to_string())?;
    
    let emails = stmt.query_map(rusqlite::params![
        case_id, 
        format!("%{}%", from_addr), format!("%{}%", from_p),
        format!("%{}%", to_addr), format!("%{}%", to_p)
    ], |row| {
        Ok(EmailMessage { 
            id: row.get(0)?, 
            evidence_id: row.get(1)?, 
            case_id: row.get(2)?, 
            message_id: row.get(3)?, 
            from_addr: row.get(4)?, 
            from_display: row.get(5)?, 
            to_addrs: row.get(6)?, 
            cc_addrs: row.get(7)?, 
            subject: row.get(8)?, 
            date_sent: row.get(9)?, 
            date_sent_utc: row.get(10)?, 
            headers_raw: row.get(11)?, 
            body_text: row.get(12)?, 
            body_html: row.get(13)?, 
            folder_name: row.get(14)?, 
            folder_category: row.get(15)?, 
            is_deleted: boolv(row,16), 
            deleted_recovered: boolv(row,17), 
            risk_score: u8v(row,18), 
            flags: row.get(19)? 
        })
    }).map_err(|e| e.to_string())?.collect::<Result<Vec<_>,_>>().map_err(|e| e.to_string())?;
    Ok(emails)
}
#[tauri::command]
pub async fn entity_heatmap(state: State<'_, AppState>, input: EntityInput) -> Result<serde_json::Value, String> {
    let db = state.db.lock().await;
    
    // Get daily email count involving this entity
    let mut stmt = db.conn.prepare("
        SELECT date(date_sent_utc) as day, COUNT(*) as cnt
        FROM emails
        WHERE case_id=?1 AND (from_addr=?2 OR to_addrs LIKE ?3 OR cc_addrs LIKE ?3)
          AND date_sent_utc IS NOT NULL
        GROUP BY day
        ORDER BY day ASC
    ").map_err(|e| e.to_string())?;
    
    let data: Vec<serde_json::Value> = stmt.query_map(
        rusqlite::params![&input.case_id, &input.email_address, format!("%{}%", input.email_address)],
        |row| {
            Ok(serde_json::json!({
                "date": row.get::<_, String>(0)?,
                "count": row.get::<_, i64>(1)?,
            }))
        }
    ).map_err(|e| e.to_string())?.collect::<Result<Vec<_>,_>>().map_err(|e| e.to_string())?;
    
    Ok(serde_json::json!({ "data": data }))
}

#[tauri::command]
pub async fn graph_data(state: State<'_, AppState>, input: EmptyInput) -> Result<serde_json::Value, String> {
    let db = state.db.lock().await;

    let (target_email, _target_name): (Option<String>, Option<String>) = db.conn.query_row(
        "SELECT target_email, target_name FROM cases WHERE id = ?1",
        [&input.case_id],
        |row| Ok((row.get(0)?, row.get(1)?)),
    ).unwrap_or((None, None));

    let mut stmt = db.conn.prepare("
        SELECT email_address, display_name, sent_count, received_count, (sent_count + received_count) as total, aliases
        FROM entities WHERE case_id=?1 ORDER BY total DESC LIMIT 150
    ").map_err(|e| e.to_string())?;

    // Build alias -> canonical mapping
    let mut alias_to_canonical: std::collections::HashMap<String, String> = std::collections::HashMap::new();

    let nodes: Vec<serde_json::Value> = stmt.query_map([&input.case_id], |row| {
        let email: String = row.get(0)?;
        let name: Option<String> = row.get(1)?;
        let sent: i64 = row.get(2)?;
        let received: i64 = row.get(3)?;
        let total: i64 = row.get(4)?;
        let aliases_str: Option<String> = row.get(5)?;

        let is_target = target_email.as_ref().map(|t| t == &email).unwrap_or(false);

        Ok((email, name, sent, received, total, aliases_str, is_target))
    }).map_err(|e| e.to_string())?.filter_map(|r| r.ok()).map(|(email, name, sent, received, total, aliases_str, is_target)| {
        alias_to_canonical.insert(email.clone(), email.clone());
        if let Some(ref a_json) = aliases_str {
            if let Ok(alias_vec) = serde_json::from_str::<Vec<String>>(a_json) {
                for a in alias_vec {
                    alias_to_canonical.insert(a, email.clone());
                }
            }
        }

        serde_json::json!({
            "id": email,
            "name": name,
            "sent": sent,
            "received": received,
            "total": total,
            "is_target": is_target,
        })
    }).collect();

    let node_set: std::collections::HashSet<String> = nodes.iter()
        .filter_map(|n| n["id"].as_str().map(|s| s.to_string()))
        .collect();

    // Query email pairs directly from emails table - FAST
    let mut email_stmt = db.conn.prepare("
        SELECT from_addr, to_addrs, cc_addrs FROM emails WHERE case_id = ?1
    ").map_err(|e| e.to_string())?;

    let mut edge_counts: std::collections::HashMap<(String, String), i64> = std::collections::HashMap::new();

    let email_rows = email_stmt.query_map([&input.case_id], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
        ))
    }).map_err(|e| e.to_string())?;

    for row_res in email_rows {
        if let Ok((from_raw, to_raw, cc_raw)) = row_res {
            let from_clean = clean_entity_name(&from_raw);
            let from_canonical = alias_to_canonical.get(&from_clean).unwrap_or(&from_clean);

            if !node_set.contains(from_canonical) {
                continue;
            }

            let to_list: Vec<String> = if to_raw.starts_with('[') {
                serde_json::from_str(&to_raw).unwrap_or_default()
            } else {
                crate::parser::split_address_list(&to_raw)
            };
            for to_addr in to_list {
                let to_clean = clean_entity_name(&to_addr);
                let to_canonical = alias_to_canonical.get(&to_clean).unwrap_or(&to_clean);
                if from_canonical != to_canonical && node_set.contains(to_canonical) {
                    *edge_counts.entry((from_canonical.clone(), to_canonical.clone())).or_insert(0) += 1;
                }
            }

            let cc_list: Vec<String> = if cc_raw.starts_with('[') {
                serde_json::from_str(&cc_raw).unwrap_or_default()
            } else {
                crate::parser::split_address_list(&cc_raw)
            };
            for cc_addr in cc_list {
                let cc_clean = clean_entity_name(&cc_addr);
                let cc_canonical = alias_to_canonical.get(&cc_clean).unwrap_or(&cc_clean);
                if from_canonical != cc_canonical && node_set.contains(cc_canonical) {
                    *edge_counts.entry((from_canonical.clone(), cc_canonical.clone())).or_insert(0) += 1;
                }
            }
        }
    }

    let mut edges: Vec<serde_json::Value> = edge_counts.into_iter().map(|((src, tgt), w)| {
        serde_json::json!({
            "source": src,
            "target": tgt,
            "weight": w,
        })
    }).collect();

    // Sort edges by weight descending and limit to top 400
    edges.sort_by(|a, b| {
        let wa = a["weight"].as_i64().unwrap_or(0);
        let wb = b["weight"].as_i64().unwrap_or(0);
        wb.cmp(&wa)
    });
    edges.truncate(400);
    
    Ok(serde_json::json!({ 
        "nodes": nodes, 
        "edges": edges,
        "target_email": target_email 
    }))
}

/// Case Notes CRUD
#[tauri::command]
pub async fn case_notes_list(state: State<'_, AppState>, input: EmptyInput) -> Result<Vec<CaseNote>, String> {
    let db = state.db.lock().await;
    let mut stmt = db.conn.prepare(
        "SELECT id, case_id, author, title, content, category, pinned, created_at, updated_at 
         FROM case_notes 
         WHERE case_id = ?1 
         ORDER BY pinned DESC, created_at DESC"
    ).map_err(|e| e.to_string())?;

    let notes = stmt.query_map([&input.case_id], |row| {
        Ok(CaseNote {
            id: row.get(0)?,
            case_id: row.get(1)?,
            author: row.get(2)?,
            title: row.get(3)?,
            content: row.get(4)?,
            category: row.get(5)?,
            pinned: boolv(row, 6),
            created_at: row.get(7)?,
            updated_at: row.get(8)?,
        })
    }).map_err(|e| e.to_string())?.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())?;

    Ok(notes)
}

#[tauri::command]
pub async fn case_note_create(state: State<'_, AppState>, input: CaseNoteCreateInput) -> Result<CaseNote, String> {
    let db = state.db.lock().await;
    let id = generate_id();
    let now = Utc::now().to_rfc3339();
    let author = input.author.unwrap_or_else(|| "admin".to_string());
    let category = input.category.unwrap_or_else(|| "general".to_string());
    let pinned = input.pinned.unwrap_or(false);

    db.conn.execute(
        "INSERT INTO case_notes (id, case_id, author, title, content, category, pinned, created_at, updated_at) 
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        rusqlite::params![&id, &input.case_id, &author, &input.title, &input.content, &category, if pinned { 1 } else { 0 }, &now, &now],
    ).map_err(|e| e.to_string())?;

    // Audit log
    db.conn.execute(
        "INSERT INTO audit_log (id, actor, action, target_type, target_id, timestamp, detail) 
         VALUES (?1, ?2, 'case_note_created', 'case_note', ?3, ?4, ?5)",
        rusqlite::params![&generate_id(), &author, &id, &now, &format!("Created note: {}", input.title)],
    ).ok();

    Ok(CaseNote {
        id,
        case_id: input.case_id,
        author,
        title: input.title,
        content: input.content,
        category,
        pinned,
        created_at: now.clone(),
        updated_at: now,
    })
}

#[tauri::command]
pub async fn case_note_update(state: State<'_, AppState>, input: CaseNoteUpdateInput) -> Result<(), String> {
    let db = state.db.lock().await;
    let now = Utc::now().to_rfc3339();

    if let Some(ref title) = input.title {
        db.conn.execute("UPDATE case_notes SET title = ?1, updated_at = ?2 WHERE id = ?3", rusqlite::params![title, &now, &input.id]).map_err(|e| e.to_string())?;
    }
    if let Some(ref content) = input.content {
        db.conn.execute("UPDATE case_notes SET content = ?1, updated_at = ?2 WHERE id = ?3", rusqlite::params![content, &now, &input.id]).map_err(|e| e.to_string())?;
    }
    if let Some(ref category) = input.category {
        db.conn.execute("UPDATE case_notes SET category = ?1, updated_at = ?2 WHERE id = ?3", rusqlite::params![category, &now, &input.id]).map_err(|e| e.to_string())?;
    }
    if let Some(pinned) = input.pinned {
        db.conn.execute("UPDATE case_notes SET pinned = ?1, updated_at = ?2 WHERE id = ?3", rusqlite::params![if pinned { 1 } else { 0 }, &now, &input.id]).map_err(|e| e.to_string())?;
    }

    Ok(())
}

#[tauri::command]
pub async fn case_note_toggle_pin(state: State<'_, AppState>, note_id: String) -> Result<bool, String> {
    let db = state.db.lock().await;
    let now = Utc::now().to_rfc3339();
    let current_pinned: i64 = db.conn.query_row(
        "SELECT pinned FROM case_notes WHERE id = ?1",
        [&note_id],
        |row| row.get(0),
    ).map_err(|e| e.to_string())?;

    let new_pinned = if current_pinned == 1 { 0 } else { 1 };
    db.conn.execute(
        "UPDATE case_notes SET pinned = ?1, updated_at = ?2 WHERE id = ?3",
        rusqlite::params![new_pinned, &now, &note_id],
    ).map_err(|e| e.to_string())?;

    Ok(new_pinned == 1)
}

#[tauri::command]
pub async fn case_note_delete(state: State<'_, AppState>, note_id: String) -> Result<(), String> {
    let db = state.db.lock().await;
    let now = Utc::now().to_rfc3339();

    db.conn.execute("DELETE FROM case_notes WHERE id = ?1", [&note_id]).map_err(|e| e.to_string())?;

    // Audit log
    db.conn.execute(
        "INSERT INTO audit_log (id, actor, action, target_type, target_id, timestamp, detail) 
         VALUES (?1, 'admin', 'case_note_deleted', 'case_note', ?2, ?3, 'Deleted case note')",
        rusqlite::params![&generate_id(), &note_id, &now],
    ).ok();

    Ok(())
}

/// Email Tags
#[tauri::command]
pub async fn email_tags_list(state: State<'_, AppState>, case_id: String) -> Result<Vec<EmailTag>, String> {
    let db = state.db.lock().await;
    let mut stmt = db.conn.prepare(
        "SELECT id, case_id, email_id, tag, color, created_by, created_at 
         FROM email_tags 
         WHERE case_id = ?1 
         ORDER BY created_at ASC"
    ).map_err(|e| e.to_string())?;

    let tags = stmt.query_map([&case_id], |row| {
        Ok(EmailTag {
            id: row.get(0)?,
            case_id: row.get(1)?,
            email_id: row.get(2)?,
            tag: row.get(3)?,
            color: row.get(4)?,
            created_by: row.get(5)?,
            created_at: row.get(6)?,
        })
    }).map_err(|e| e.to_string())?.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())?;

    Ok(tags)
}

#[tauri::command]
pub async fn email_tag_add(state: State<'_, AppState>, input: EmailTagAddInput) -> Result<EmailTag, String> {
    let db = state.db.lock().await;
    let id = generate_id();
    let now = Utc::now().to_rfc3339();
    let author = input.created_by.unwrap_or_else(|| "admin".to_string());
    let color = input.color.unwrap_or_else(|| match input.tag.to_lowercase().as_str() {
        "key evidence" | "hot" => "#ef4444".to_string(),
        "privileged" | "confidential" => "#8b5cf6".to_string(),
        "responsive" => "#22c55e".to_string(),
        "suspicious" => "#f97316".to_string(),
        _ => "#3b82f6".to_string(),
    });

    db.conn.execute(
        "INSERT INTO email_tags (id, case_id, email_id, tag, color, created_by, created_at) 
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
         ON CONFLICT(case_id, email_id, tag) DO NOTHING",
        rusqlite::params![&id, &input.case_id, &input.email_id, &input.tag, &color, &author, &now],
    ).map_err(|e| e.to_string())?;

    // Audit log
    db.conn.execute(
        "INSERT INTO audit_log (id, actor, action, target_type, target_id, timestamp, detail) 
         VALUES (?1, ?2, 'email_tagged', 'email', ?3, ?4, ?5)",
        rusqlite::params![&generate_id(), &author, &input.email_id, &now, &format!("Tagged email as '{}'", input.tag)],
    ).ok();

    Ok(EmailTag {
        id,
        case_id: input.case_id,
        email_id: input.email_id,
        tag: input.tag,
        color,
        created_by: author,
        created_at: now,
    })
}

#[tauri::command]
pub async fn email_tag_remove(state: State<'_, AppState>, input: EmailTagRemoveInput) -> Result<(), String> {
    let db = state.db.lock().await;
    let now = Utc::now().to_rfc3339();

    db.conn.execute(
        "DELETE FROM email_tags WHERE case_id = ?1 AND email_id = ?2 AND tag = ?3",
        rusqlite::params![&input.case_id, &input.email_id, &input.tag],
    ).map_err(|e| e.to_string())?;

    // Audit log
    db.conn.execute(
        "INSERT INTO audit_log (id, actor, action, target_type, target_id, timestamp, detail) 
         VALUES (?1, 'admin', 'email_untagged', 'email', ?2, ?3, ?4)",
        rusqlite::params![&generate_id(), &input.email_id, &now, &format!("Removed tag '{}'", input.tag)],
    ).ok();

    Ok(())
}

/// Email Notes
#[tauri::command]
pub async fn email_notes_list(state: State<'_, AppState>, email_id: String) -> Result<Vec<EmailNote>, String> {
    let db = state.db.lock().await;
    let mut stmt = db.conn.prepare(
        "SELECT id, case_id, email_id, author, content, created_at, updated_at 
         FROM email_notes 
         WHERE email_id = ?1 
         ORDER BY created_at ASC"
    ).map_err(|e| e.to_string())?;

    let notes = stmt.query_map([&email_id], |row| {
        Ok(EmailNote {
            id: row.get(0)?,
            case_id: row.get(1)?,
            email_id: row.get(2)?,
            author: row.get(3)?,
            content: row.get(4)?,
            created_at: row.get(5)?,
            updated_at: row.get(6)?,
        })
    }).map_err(|e| e.to_string())?.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())?;

    Ok(notes)
}

#[tauri::command]
pub async fn email_note_add(state: State<'_, AppState>, input: EmailNoteInput) -> Result<EmailNote, String> {
    let db = state.db.lock().await;
    let id = generate_id();
    let now = Utc::now().to_rfc3339();
    let author = input.author.unwrap_or_else(|| "admin".to_string());

    db.conn.execute(
        "INSERT INTO email_notes (id, case_id, email_id, author, content, created_at, updated_at) 
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        rusqlite::params![&id, &input.case_id, &input.email_id, &author, &input.content, &now, &now],
    ).map_err(|e| e.to_string())?;

    // Audit log
    db.conn.execute(
        "INSERT INTO audit_log (id, actor, action, target_type, target_id, timestamp, detail) 
         VALUES (?1, ?2, 'email_note_added', 'email', ?3, ?4, ?5)",
        rusqlite::params![&generate_id(), &author, &input.email_id, &now, &"Added note to email".to_string()],
    ).ok();

    Ok(EmailNote {
        id,
        case_id: input.case_id,
        email_id: input.email_id,
        author,
        content: input.content,
        created_at: now.clone(),
        updated_at: now,
    })
}

#[tauri::command]
pub async fn email_note_delete(state: State<'_, AppState>, note_id: String) -> Result<(), String> {
    let db = state.db.lock().await;
    db.conn.execute("DELETE FROM email_notes WHERE id = ?1", [&note_id]).map_err(|e| e.to_string())?;
    Ok(())
}

// === PHASE 5: REPORTING ===

#[tauri::command]
pub async fn generate_report_data(state: State<'_, AppState>, input: serde_json::Value) -> Result<serde_json::Value, String> {
    let db = state.db.lock().await;

    let case_id = input["case_id"].as_str()
        .or_else(|| input["caseId"].as_str())
        .or_else(|| input.as_str())
        .unwrap_or("");

    // Case info
    let case_info = db.conn.query_row(
        "SELECT id, title, case_number, description, status, target_name, target_email, target_organization FROM cases WHERE id = ?1",
        [case_id],
        |row| Ok(serde_json::json!({
            "id": row.get::<_, String>(0)?,
            "title": row.get::<_, String>(1)?,
            "case_number": row.get::<_, Option<String>>(2)?,
            "description": row.get::<_, Option<String>>(3)?,
            "status": row.get::<_, String>(4)?,
            "target_name": row.get::<_, Option<String>>(5)?,
            "target_email": row.get::<_, Option<String>>(6)?,
            "target_organization": row.get::<_, Option<String>>(7)?,
        }))
    ).map_err(|e| e.to_string())?;

    // Methodology
    let methodology = serde_json::json!({
        "tool_name": "J12 Forensic Investigation Platform",
        "tool_version": "0.1.0",
        "parser_version": "J12Parser 0.1.0",
        "analysis_engine": "J12Analysis 0.1.0",
        "generated_at": chrono::Utc::now().to_rfc3339(),
    });

    // Custody chain
    let mut custody_stmt = db.conn.prepare("SELECT ce.action, ce.actor, ce.timestamp, ce.tool, ce.tool_version, ce.detail FROM custody_events ce JOIN evidence_items ei ON ce.evidence_id = ei.id WHERE ei.case_id = ?1 ORDER BY ce.timestamp ASC").map_err(|e| e.to_string())?;
    let custody_chain: Vec<serde_json::Value> = custody_stmt.query_map([case_id], |row| {
        Ok(serde_json::json!({
            "action": row.get::<_, String>(0)?,
            "actor": row.get::<_, String>(1)?,
            "timestamp": row.get::<_, String>(2)?,
            "tool": row.get::<_, String>(3)?,
            "tool_version": row.get::<_, String>(4)?,
            "detail": row.get::<_, Option<String>>(5)?,
        }))
    }).map_err(|e| e.to_string())?.collect::<Result<Vec<_>,_>>().map_err(|e| e.to_string())?;

    // Evidence inventory
    let mut evidence_stmt = db.conn.prepare("SELECT id, filename, format, sha256, sha512, size_bytes, parse_status, message_count, acquired_at FROM evidence_items WHERE case_id = ?1 ORDER BY acquired_at ASC").map_err(|e| e.to_string())?;
    let evidence_inventory: Vec<serde_json::Value> = evidence_stmt.query_map([case_id], |row| {
        Ok(serde_json::json!({
            "id": row.get::<_, String>(0)?,
            "filename": row.get::<_, String>(1)?,
            "format": row.get::<_, String>(2)?,
            "sha256": row.get::<_, String>(3)?,
            "sha512": row.get::<_, Option<String>>(4)?,
            "size_bytes": row.get::<_, i64>(5)?,
            "parse_status": row.get::<_, String>(6)?,
            "message_count": row.get::<_, i64>(7)?,
            "acquired_at": row.get::<_, String>(8)?,
        }))
    }).map_err(|e| e.to_string())?.collect::<Result<Vec<_>,_>>().map_err(|e| e.to_string())?;

    // Findings
    let mut findings_stmt = db.conn.prepare("SELECT id, type, severity, confidence, title, description, status, created_at FROM findings WHERE case_id = ?1 ORDER BY severity ASC, created_at ASC").map_err(|e| e.to_string())?;
    let findings: Vec<serde_json::Value> = findings_stmt.query_map([case_id], |row| {
        Ok(serde_json::json!({
            "id": row.get::<_, String>(0)?,
            "type": row.get::<_, String>(1)?,
            "severity": row.get::<_, String>(2)?,
            "confidence": row.get::<_, String>(3)?,
            "title": row.get::<_, String>(4)?,
            "description": row.get::<_, Option<String>>(5)?,
            "status": row.get::<_, String>(6)?,
            "created_at": row.get::<_, String>(7)?,
        }))
    }).map_err(|e| e.to_string())?.collect::<Result<Vec<_>,_>>().map_err(|e| e.to_string())?;

    // Entity summary (top 50)
    let mut entity_stmt = db.conn.prepare("SELECT email_address, display_name, sent_count, received_count FROM entities WHERE case_id = ?1 ORDER BY (sent_count + received_count) DESC LIMIT 50").map_err(|e| e.to_string())?;
    let entities: Vec<serde_json::Value> = entity_stmt.query_map([case_id], |row| {
        Ok(serde_json::json!({
            "email": row.get::<_, String>(0)?,
            "display_name": row.get::<_, Option<String>>(1)?,
            "sent": row.get::<_, i64>(2)?,
            "received": row.get::<_, i64>(3)?,
        }))
    }).map_err(|e| e.to_string())?.collect::<Result<Vec<_>,_>>().map_err(|e| e.to_string())?;

    // Email statistics
    let email_stats = db.conn.query_row(
        "SELECT COUNT(*), SUM(CASE WHEN folder_category='inbox' THEN 1 ELSE 0 END), SUM(CASE WHEN folder_category='sent' THEN 1 ELSE 0 END), SUM(CASE WHEN folder_category='soft_deleted' THEN 1 ELSE 0 END), SUM(CASE WHEN folder_category='spam' THEN 1 ELSE 0 END), SUM(CASE WHEN folder_category='drafts' THEN 1 ELSE 0 END), SUM(CASE WHEN folder_category='other' THEN 1 ELSE 0 END), MIN(date_sent_utc), MAX(date_sent_utc) FROM emails WHERE case_id = ?1",
        [case_id],
        |row| Ok(serde_json::json!({
            "total": row.get::<_, i64>(0)?,
            "inbox": row.get::<_, i64>(1)?,
            "sent": row.get::<_, i64>(2)?,
            "deleted": row.get::<_, i64>(3)?,
            "spam": row.get::<_, i64>(4)?,
            "drafts": row.get::<_, i64>(5)?,
            "other": row.get::<_, i64>(6)?,
            "date_from": row.get::<_, Option<String>>(7)?,
            "date_to": row.get::<_, Option<String>>(8)?,
        }))
    ).map_err(|e| e.to_string())?;

    // Hash manifest
    let mut hash_stmt = db.conn.prepare("SELECT filename, sha256, sha512 FROM evidence_items WHERE case_id = ?1").map_err(|e| e.to_string())?;
    let hash_manifest: Vec<serde_json::Value> = hash_stmt.query_map([case_id], |row| {
        Ok(serde_json::json!({
            "filename": row.get::<_, String>(0)?,
            "sha256": row.get::<_, String>(1)?,
            "sha512": row.get::<_, Option<String>>(2)?,
        }))
    }).map_err(|e| e.to_string())?.collect::<Result<Vec<_>,_>>().map_err(|e| e.to_string())?;

    // Target profile if specified
    let target_email = case_info["target_email"].as_str().unwrap_or("");
    let target_profile = if !target_email.is_empty() {
        let mut target_stmt = db.conn.prepare("SELECT email_address, display_name, sent_count, received_count, aliases FROM entities WHERE case_id = ?1 AND (email_address LIKE ?2 OR aliases LIKE ?2) LIMIT 1").ok();
        if let Some(mut stmt) = target_stmt {
            let row = stmt.query_row(rusqlite::params![case_id, format!("%{}%", target_email)], |r| {
                Ok(serde_json::json!({
                    "email": r.get::<_, String>(0)?,
                    "display_name": r.get::<_, Option<String>>(1)?,
                    "sent": r.get::<_, i64>(2)?,
                    "received": r.get::<_, i64>(3)?,
                    "aliases": r.get::<_, Option<String>>(4)?,
                }))
            }).ok();
            row
        } else {
            None
        }
    } else {
        None
    };

    // Folder hierarchy breakdown
    let mut folder_stmt = db.conn.prepare("
        SELECT COALESCE(folder_name, 'Root') as fname, folder_category, COUNT(*) as cnt, MIN(date_sent_utc), MAX(date_sent_utc)
        FROM emails 
        WHERE case_id = ?1 
        GROUP BY folder_name, folder_category 
        ORDER BY cnt DESC
    ").map_err(|e| e.to_string())?;
    let folder_breakdown: Vec<serde_json::Value> = folder_stmt.query_map([&case_id], |row| {
        Ok(serde_json::json!({
            "folder_name": row.get::<_, String>(0)?,
            "folder_category": row.get::<_, String>(1)?,
            "count": row.get::<_, i64>(2)?,
            "date_from": row.get::<_, Option<String>>(3)?,
            "date_to": row.get::<_, Option<String>>(4)?,
        }))
    }).map_err(|e| e.to_string())?.collect::<Result<Vec<_>,_>>().map_err(|e| e.to_string())?;

    // Attachments inventory
    let attachments_manifest: Vec<serde_json::Value> = match db.conn.prepare("
        SELECT a.filename, a.mime_type, a.size_bytes, a.sha256, e.subject, e.from_addr, e.date_sent_utc 
        FROM attachments a 
        JOIN emails e ON a.email_id = e.id 
        WHERE e.case_id = ?1 
        ORDER BY a.size_bytes DESC 
        LIMIT 150
    ") {
        Ok(mut attach_stmt) => {
            attach_stmt.query_map([case_id], |row| {
                Ok(serde_json::json!({
                    "filename": row.get::<_, Option<String>>(0)?.unwrap_or_else(|| "attachment".to_string()),
                    "file_type": row.get::<_, Option<String>>(1)?.unwrap_or_else(|| "application/octet-stream".to_string()),
                    "size_bytes": row.get::<_, i64>(2)?,
                    "sha256": row.get::<_, String>(3)?,
                    "email_subject": row.get::<_, Option<String>>(4)?,
                    "from_addr": row.get::<_, String>(5)?,
                    "date_sent_utc": row.get::<_, Option<String>>(6)?,
                }))
            }).map(|rows| rows.filter_map(|r| r.ok()).collect()).unwrap_or_default()
        }
        Err(_) => Vec::new(),
    };

    // Detailed Flagged / High-Risk / Deleted Email Ledger (top 150)
    let mut ledger_stmt = db.conn.prepare("
        SELECT id, from_addr, from_display, to_addrs, subject, date_sent_utc, folder_category, is_deleted, deleted_recovered, risk_score 
        FROM emails 
        WHERE case_id = ?1 AND (risk_score >= 25 OR is_deleted = 1 OR deleted_recovered = 1)
        ORDER BY risk_score DESC, date_sent_utc DESC 
        LIMIT 150
    ").map_err(|e| e.to_string())?;
    let key_messages_ledger: Vec<serde_json::Value> = ledger_stmt.query_map([&case_id], |row| {
        Ok(serde_json::json!({
            "id": row.get::<_, String>(0)?,
            "from_addr": row.get::<_, String>(1)?,
            "from_display": row.get::<_, Option<String>>(2)?,
            "to_addrs": row.get::<_, String>(3)?,
            "subject": row.get::<_, Option<String>>(4)?,
            "date_sent_utc": row.get::<_, Option<String>>(5)?,
            "folder_category": row.get::<_, String>(6)?,
            "is_deleted": boolv(row, 7),
            "deleted_recovered": boolv(row, 8),
            "risk_score": u8v(row, 9),
        }))
    }).map_err(|e| e.to_string())?.collect::<Result<Vec<_>,_>>().map_err(|e| e.to_string())?;

    Ok(serde_json::json!({
        "case_info": case_info,
        "methodology": methodology,
        "custody_chain": custody_chain,
        "evidence_inventory": evidence_inventory,
        "findings": findings,
        "entities": entities,
        "email_stats": email_stats,
        "hash_manifest": hash_manifest,
        "target_profile": target_profile,
        "folder_breakdown": folder_breakdown,
        "attachments_manifest": attachments_manifest,
        "key_messages_ledger": key_messages_ledger,
    }))
}

#[tauri::command]
pub async fn export_report_pdf(state: State<'_, AppState>, case_id: String, sections: Vec<String>, exhibits: Vec<serde_json::Value>) -> Result<String, String> {
    let db = state.db.lock().await;

    // Generate HTML report
    let mut html = String::from("<!DOCTYPE html><html><head><meta charset='UTF-8'><title>Forensic Report</title>");
    html.push_str("<style>");
    html.push_str("body{font-family:Georgia,serif;line-height:1.6;max-width:800px;margin:0 auto;padding:40px;color:#1a1a2e}");
    html.push_str("h1{text-align:center;border-bottom:2px solid #1a1a2e;padding-bottom:16px}");
    html.push_str("h2{border-bottom:1px solid #ddd;padding-bottom:8px;margin-top:32px}");
    html.push_str("table{width:100%;border-collapse:collapse;margin:16px 0}");
    html.push_str("th,td{border:1px solid #ddd;padding:8px;text-align:left}");
    html.push_str("th{background:#f5f5f5}");
    html.push_str(".mono{font-family:monospace;font-size:12px}");
    html.push_str(".severity-critical{color:#ef4444;font-weight:700}");
    html.push_str(".severity-high{color:#f97316;font-weight:700}");
    html.push_str(".severity-medium{color:#eab308;font-weight:700}");
    html.push_str(".severity-low{color:#22c55e;font-weight:700}");
    html.push_str(".exhibit{border:2px solid #1a1a2e;padding:16px;margin:16px 0;page-break-inside:avoid}");
    html.push_str(".certification{margin-top:48px;padding:24px;border:2px solid #1a1a2e}");
    html.push_str(".signature-line{border-top:1px solid #1a1a2e;width:200px;margin-top:48px;padding-top:8px}");
    html.push_str("</style></head><body>");

    // Header
    html.push_str("<h1>FORENSIC INVESTIGATION REPORT</h1>");
    html.push_str(&format!("p style='text-align:center;color:#666'>Generated: {}</p>", chrono::Utc::now().to_rfc3339()));

    // Case info
    if sections.contains(&"case_info".to_string()) {
        let case = db.conn.query_row(
            "SELECT title, case_number, description, status, target_name, target_email FROM cases WHERE id = ?1",
            [&case_id],
            |row| Ok((
                row.get::<_, String>(0)?, row.get::<_, Option<String>>(1)?, row.get::<_, Option<String>>(2)?,
                row.get::<_, String>(3)?, row.get::<_, Option<String>>(4)?, row.get::<_, Option<String>>(5)?,
            ))
        ).map_err(|e| e.to_string())?;

        html.push_str("<h2>1. Case Information</h2>");
        html.push_str("<table>");
        html.push_str(&format!("<tr><th>Case Title</th><td>{}</td></tr>", case.0));
        html.push_str(&format!("<tr><th>Case Number</th><td>{}</td></tr>", case.1.unwrap_or_default()));
        html.push_str(&format!("<tr><th>Status</th><td>{}</td></tr>", case.3));
        html.push_str(&format!("<tr><th>Target Name</th><td>{}</td></tr>", case.4.unwrap_or_default()));
        html.push_str(&format!("<tr><th>Target Email</th><td>{}</td></tr>", case.5.unwrap_or_default()));
        html.push_str(&format!("<tr><th>Description</th><td>{}</td></tr>", case.2.unwrap_or_default()));
        html.push_str("</table>");
    }

    // Methodology
    if sections.contains(&"methodology".to_string()) {
        html.push_str("<h2>2. Methodology & Tool Versions</h2>");
        html.push_str("<table>");
        html.push_str("<tr><th>Tool</th><td>J12 Forensic Investigation Platform</td></tr>");
        html.push_str("<tr><th>Version</th><td>0.1.0</td></tr>");
        html.push_str("<tr><th>Parser</th><td>J12Parser 0.1.0</td></tr>");
        html.push_str("<tr><th>Analysis Engine</th><td>J12Analysis 0.1.0</td></tr>");
        html.push_str("</table>");
    }

    // Evidence inventory
    if sections.contains(&"evidence_inventory".to_string()) {
        html.push_str("<h2>4. Evidence Inventory</h2>");
        html.push_str("<table><tr><th>Filename</th><th>Format</th><th>Size</th><th>SHA-256</th><th>Status</th></tr>");
        let mut stmt = db.conn.prepare("SELECT filename, format, size_bytes, sha256, parse_status FROM evidence_items WHERE case_id = ?1").map_err(|e| e.to_string())?;
        let rows = stmt.query_map([&case_id], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?, row.get::<_, i64>(2)?, row.get::<_, String>(3)?, row.get::<_, String>(4)?))
        }).map_err(|e| e.to_string())?.collect::<Result<Vec<_>,_>>().map_err(|e| e.to_string())?;
        for row in rows {
            html.push_str(&format!("<tr><td>{}</td><td>{}</td><td>{}</td><td class='mono'>{}</td><td>{}</td></tr>", row.0, row.1, row.2, row.3, row.4));
        }
        html.push_str("</table>");
    }

    // Findings
    if sections.contains(&"findings".to_string()) {
        html.push_str("<h2>5. Findings by Severity</h2>");
        let mut stmt = db.conn.prepare("SELECT severity, type, title, description, status FROM findings WHERE case_id = ?1 ORDER BY severity ASC").map_err(|e| e.to_string())?;
        let rows = stmt.query_map([&case_id], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?, row.get::<_, String>(2)?, row.get::<_, Option<String>>(3)?, row.get::<_, String>(4)?))
        }).map_err(|e| e.to_string())?.collect::<Result<Vec<_>,_>>().map_err(|e| e.to_string())?;
        for row in rows {
            let severity_class = format!("severity-{}", row.0);
            html.push_str(&format!("<div style='margin:12px 0;padding:12px;border-left:4px solid #ddd'><span class='{}'>[{}]</span> <strong>{}</strong> - {} <span style='color:#666'>({})</span></div>", severity_class, row.0.to_uppercase(), row.1, row.2, row.4));
        }
    }

    // === EVIDENCE MAPS ===
    html.push_str("<h2>6. Evidence Maps</h2>");
    html.push_str("<p>Mapping of evidence sources to findings and key entities:</p>");
    html.push_str("<table><tr><th>Evidence File</th><th>Format</th><th>Messages</th><th>Key Entities</th></tr>");
    let mut evidence_map_stmt = db.conn.prepare("
        SELECT e.filename, e.format, e.message_count, 
               (SELECT GROUP_CONCAT(DISTINCT SUBSTR(em.from_addr, 1, 30)) FROM emails em WHERE em.evidence_id = e.id LIMIT 5)
        FROM evidence_items e WHERE e.case_id = ?1 ORDER BY e.acquired_at
    ").map_err(|e| e.to_string())?;
    let evidence_map_rows = evidence_map_stmt.query_map([&case_id], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?, row.get::<_, i64>(2)?, row.get::<_, Option<String>>(3)?))
    }).map_err(|e| e.to_string())?.collect::<Result<Vec<_>,_>>().map_err(|e| e.to_string())?;
    for row in evidence_map_rows {
        html.push_str(&format!("<tr><td>{}</td><td>{}</td><td>{}</td><td class='mono'>{}</td></tr>", row.0, row.1, row.2, row.3.unwrap_or_default()));
    }
    html.push_str("</table>");

    // === TIMELINE VISUALIZATION ===
    html.push_str("<h2>8. Timeline Visualization</h2>");
    html.push_str("<p>Email activity over time:</p>");
    html.push_str("<table><tr><th>Month</th><th>Count</th><th>Activity</th></tr>");
    let mut timeline_stmt = db.conn.prepare("
        SELECT strftime('%Y-%m', date_sent_utc) as month, COUNT(*) as cnt 
        FROM emails WHERE case_id = ?1 AND date_sent_utc IS NOT NULL 
        GROUP BY month ORDER BY month
    ").map_err(|e| e.to_string())?;
    let timeline_rows = timeline_stmt.query_map([&case_id], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
    }).map_err(|e| e.to_string())?.collect::<Result<Vec<_>,_>>().map_err(|e| e.to_string())?;
    let max_timeline = timeline_rows.iter().map(|r| r.1).max().unwrap_or(1);
    for row in &timeline_rows {
        let bar_len = ((row.1 as f64 / max_timeline as f64) * 30.0) as usize;
        let bar = "█".repeat(bar_len.max(1));
        html.push_str(&format!("<tr><td class='mono'>{}</td><td>{}</td><td style='color:#3b82f6;white-space:nowrap'>{}</td></tr>", row.0, row.1, bar));
    }
    html.push_str("</table>");

    // === COMMUNICATION GRAPH ===
    html.push_str("<h2>9. Communication Graph</h2>");
    html.push_str("<p>Top communication pairs (sender → recipient):</p>");
    html.push_str("<table><tr><th>#</th><th>Sender</th><th>Recipient</th><th>Messages</th></tr>");
    let mut graph_stmt = db.conn.prepare("SELECT from_addr, to_addrs FROM emails WHERE case_id = ?1").map_err(|e| e.to_string())?;
    let graph_rows = graph_stmt.query_map([&case_id], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
    }).map_err(|e| e.to_string())?.collect::<Result<Vec<_>,_>>().map_err(|e| e.to_string())?;
    let mut pair_counts: std::collections::HashMap<(String, String), i64> = std::collections::HashMap::new();
    for row in &graph_rows {
        let to_list: Vec<String> = serde_json::from_str(&row.1).unwrap_or_default();
        for to in to_list {
            *pair_counts.entry((row.0.clone(), to)).or_insert(0) += 1;
        }
    }
    let mut pair_vec: Vec<((String, String), i64)> = pair_counts.into_iter().collect();
    pair_vec.sort_by(|a, b| b.1.cmp(&a.1));
    for (i, pair) in pair_vec.iter().take(25).enumerate() {
        html.push_str(&format!("<tr><td>{}</td><td class='mono'>{}</td><td class='mono'>{}</td><td>{}</td></tr>", i+1, pair.0.0, pair.0.1, pair.1));
    }
    html.push_str("</table>");

    // Exhibits
    if sections.contains(&"exhibits".to_string()) && !exhibits.is_empty() {
        html.push_str("<h2>7. Exhibits</h2>");
        for (i, exhibit) in exhibits.iter().enumerate() {
            let letter = (b'A' + i as u8) as char;
            html.push_str("<div class='exhibit'>");
            html.push_str(&format!("<h3>Exhibit {}: {}</h3>", letter, exhibit["subject"].as_str().unwrap_or("")));
            html.push_str(&format!("<p><strong>From:</strong> {}</p>", exhibit["from_display"].as_str().unwrap_or(exhibit["from_addr"].as_str().unwrap_or(""))));
            html.push_str(&format!("<p><strong>Date:</strong> {}</p>", exhibit["date_sent"].as_str().unwrap_or("")));
            html.push_str(&format!("<p class='mono'><strong>SHA-256:</strong> {}</p>", exhibit["sha256"].as_str().unwrap_or("")));
            html.push_str(&format!("<p class='mono'><strong>Headers:</strong><br>{}</p>", exhibit["headers_full"].as_str().unwrap_or("").replace("\n", "<br>")));
            html.push_str("</div>");
        }
    }

    // Certification
    if sections.contains(&"certification".to_string()) {
        html.push_str("<div class='certification'>");
        html.push_str("<h2>11. Certification</h2>");
        html.push_str("<p>I hereby certify that the information contained in this report is true and accurate to the best of my knowledge. The evidence described herein was collected and handled in accordance with established forensic procedures.</p>");
        html.push_str("<div style='display:flex;justify-content:space-between;margin-top:48px'>");
        html.push_str("<div class='signature-line'>Investigator Signature</div>");
        html.push_str("<div class='signature-line'>Date</div>");
        html.push_str("</div></div>");
    }

    html.push_str("</body></html>");

    // Save HTML file (deterministic - same content = same filename)
    let output_dir = std::path::PathBuf::from("/Users/macbookpro/Project/email-forensic-desktop/reports");
    std::fs::create_dir_all(&output_dir).ok();
    // Use content hash for deterministic filename
    let content_hash = format!("{:x}", md5::compute(&html));
    let filename = format!("forensic_report_{}_{}.html", case_id, &content_hash[..12]);
    let output_path = output_dir.join(&filename);
    std::fs::write(&output_path, html).map_err(|e| e.to_string())?;

    Ok(output_path.to_string_lossy().to_string())
}

// === PHASE 6: HARDENING ===

/// Verify all evidence hashes on demand
#[tauri::command]
pub async fn verify_evidence_hashes(state: State<'_, AppState>, input: EmptyInput) -> Result<serde_json::Value, String> {
    let db = state.db.lock().await;
    
    let mut stmt = db.conn.prepare("SELECT id, filename, sha256, sha512, stored_path FROM evidence_items WHERE case_id = ?1").map_err(|e| e.to_string())?;
    let rows = stmt.query_map([&input.case_id], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, Option<String>>(3)?,
            row.get::<_, String>(4)?,
        ))
    }).map_err(|e| e.to_string())?.collect::<Result<Vec<_>,_>>().map_err(|e| e.to_string())?;
    
    let mut results = Vec::new();
    let mut verified = 0;
    let mut failed = 0;
    let mut missing = 0;
    
    for (id, filename, stored_sha256, _stored_sha512, stored_path) in rows {
        let path = std::path::PathBuf::from(&stored_path);
        let status = if !path.exists() {
            missing += 1;
            "missing"
        } else if let Ok(data) = std::fs::read(&path) {
            use sha2::Digest;
            let mut hasher = sha2::Sha256::new();
            hasher.update(&data);
            let computed = format!("{:x}", hasher.finalize());
            if computed == stored_sha256 {
                verified += 1;
                "verified"
            } else {
                failed += 1;
                "modified"
            }
        } else {
            missing += 1;
            "missing"
        };
        results.push(serde_json::json!({
            "id": id,
            "filename": filename,
            "status": status,
        }));
    }
    
    Ok(serde_json::json!({
        "total": results.len(),
        "verified": verified,
        "failed": failed,
        "missing": missing,
        "results": results,
    }))
}

/// Export audit/custody log
#[tauri::command]
pub async fn export_audit_log(state: State<'_, AppState>, input: EmptyInput) -> Result<String, String> {
    let db = state.db.lock().await;
    
    // Build CSV
    let mut csv = String::from("Timestamp,Action,Actor,Tool,Version,Detail,Hash\n");
    
    let mut stmt = db.conn.prepare(
        "SELECT ce.action, ce.actor, ce.timestamp, ce.tool, ce.tool_version, ce.detail, ce.hash_after 
         FROM custody_events ce 
         JOIN evidence_items ei ON ce.evidence_id = ei.id 
         WHERE ei.case_id = ?1 
         ORDER BY ce.timestamp"
    ).map_err(|e| e.to_string())?;
    
    let rows = stmt.query_map([&input.case_id], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, String>(3)?,
            row.get::<_, String>(4)?,
            row.get::<_, Option<String>>(5)?,
            row.get::<_, Option<String>>(6)?,
        ))
    }).map_err(|e| e.to_string())?.collect::<Result<Vec<_>,_>>().map_err(|e| e.to_string())?;
    
    for (action, actor, timestamp, tool, version, detail, hash) in rows {
        csv.push_str(&format!("{},{},{},{},{},{},{}\n",
            timestamp,
            escape_csv(&action),
            escape_csv(&actor),
            escape_csv(&tool),
            escape_csv(&version),
            escape_csv(&detail.unwrap_or_default()),
            hash.unwrap_or_default()
        ));
    }
    
    // Save CSV file
    let output_dir = std::path::PathBuf::from("/Users/macbookpro/Project/email-forensic-desktop/reports");
    std::fs::create_dir_all(&output_dir).ok();
    let filename = format!("audit_log_{}.csv", if input.case_id.len() >= 8 { &input.case_id[..8] } else { &input.case_id });
    let output_path = output_dir.join(&filename);
    std::fs::write(&output_path, csv).map_err(|e| e.to_string())?;
    
    Ok(output_path.to_string_lossy().to_string())
}

/// Check custody chain for gaps
#[tauri::command]
pub async fn check_custody_chain(state: State<'_, AppState>, input: EmptyInput) -> Result<serde_json::Value, String> {
    let db = state.db.lock().await;
    
    // Get all evidence items
    let mut evidence_stmt = db.conn.prepare("SELECT id, filename FROM evidence_items WHERE case_id = ?1").map_err(|e| e.to_string())?;
    let evidence_items: Vec<(String, String)> = evidence_stmt.query_map([&input.case_id], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
    }).map_err(|e| e.to_string())?.collect::<Result<Vec<_>,_>>().map_err(|e| e.to_string())?;
    
    let mut gaps = Vec::new();
    let mut valid = 0;
    
    for (evidence_id, filename) in &evidence_items {
        // Check if there's at least an "ingested" event
        let count: i64 = db.conn.query_row(
            "SELECT COUNT(*) FROM custody_events WHERE evidence_id = ?1",
            [evidence_id],
            |row| row.get(0)
        ).unwrap_or(0);
        
        if count == 0 {
            gaps.push(serde_json::json!({
                "evidence": filename,
                "issue": "No custody events recorded",
            }));
        } else {
            valid += 1;
        }
    }
    
    Ok(serde_json::json!({
        "total_evidence": evidence_items.len(),
        "valid": valid,
        "gaps": gaps,
        "chain_intact": gaps.is_empty(),
    }))
}

fn escape_csv(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') {
        format!("\"{}\"", s.replace("\"", "\"\""))
    } else {
        s.to_string()
    }
}

// === IMAP ACQUISITION ===

#[tauri::command]
pub async fn imap_list_mailboxes(
    server: String,
    port: u16,
    username: String,
    password: String,
    use_ssl: bool,
) -> Result<Vec<String>, String> {
    let config = crate::imap_acquisition::ImapConfig {
        server,
        port,
        username,
        password,
        use_ssl,
        mailbox: "INBOX".to_string(),
    };
    crate::imap_acquisition::list_mailboxes(&config)
}

#[tauri::command]
pub async fn imap_fetch_emails(
    _state: State<'_, AppState>,
    _case_id: String,
    _evidence_id: String,
    server: String,
    port: u16,
    username: String,
    password: String,
    use_ssl: bool,
    mailbox: String,
    max_messages: Option<u32>,
) -> Result<serde_json::Value, String> {
    let config = crate::imap_acquisition::ImapConfig {
        server,
        port,
        username,
        password,
        use_ssl,
        mailbox,
    };
    
    let result = crate::imap_acquisition::fetch_emails(&config, max_messages)?;
    
    Ok(serde_json::json!({
        "total_found": result.total_found,
        "downloaded": result.downloaded,
        "errors": result.errors,
        "messages": result.messages,
    }))
}

// ==========================================
// EMAIL FORENSIC ARTIFACT TAXONOMY ENGINE
// ==========================================

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CaseAttachmentItem {
    pub id: String,
    pub email_id: String,
    pub filename: String,
    pub sha256: String,
    pub mime_type: String,
    pub size_bytes: u64,
    pub stored_path: Option<String>,
    pub entropy: Option<f64>,
    pub risk_flags: Option<String>,
    pub email_subject: Option<String>,
    pub email_from: String,
    pub email_date: Option<String>,
    pub email_risk_score: u8,
    pub category: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TaxonomyDomainSummary {
    pub domain_id: String,
    pub name: String,
    pub icon: String,
    pub total_count: usize,
    pub subcategories: Vec<TaxonomySubcategorySummary>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TaxonomySubcategorySummary {
    pub subcategory_id: String,
    pub name: String,
    pub count: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ForensicTaxonomyArtifact {
    pub id: String,
    pub domain_id: String,
    pub subcategory_id: String,
    pub title: String,
    pub primary_value: String,
    pub secondary_value: Option<String>,
    pub details: String,
    pub severity: String,
    pub artifact_type: String, // "native", "recovered", "derived"
    pub email_id: String,
    pub email_subject: Option<String>,
    pub email_from: String,
    pub date_sent_utc: Option<String>,
}

fn classify_attachment_category(filename: &str, mime: &str, entropy: Option<f64>, risk_flags: Option<&str>) -> String {
    let lower = filename.to_lowercase();
    let ent = entropy.unwrap_or(0.0);
    let flags = risk_flags.unwrap_or("");

    let dangerous_exts = [
        ".exe", ".scr", ".pif", ".cmd", ".bat", ".com", ".vbs", ".js", ".wsf", 
        ".ps1", ".msi", ".iso", ".hta", ".cpl", ".jar", ".reg", ".docm", ".xlsm", ".pptm"
    ];

    if dangerous_exts.iter().any(|ext| lower.ends_with(ext)) 
        || flags.contains("dangerous") 
        || flags.contains("double_extension") 
        || flags.contains("macro")
        || ent > 7.4 {
        return "dangerous".to_string();
    }

    if lower.ends_with(".ics") || mime.contains("calendar") {
        return "calendar".to_string();
    }

    if lower.ends_with(".vcf") || mime.contains("vcard") {
        return "vcard".to_string();
    }

    let doc_exts = [".pdf", ".doc", ".docx", ".xls", ".xlsx", ".ppt", ".pptx", ".txt", ".csv", ".rtf", ".odt", ".ods", ".xml", ".json", ".log"];
    if doc_exts.iter().any(|ext| lower.ends_with(ext)) || mime.contains("pdf") || mime.contains("document") || mime.contains("sheet") || mime.contains("text") {
        return "documents".to_string();
    }

    let img_exts = [".png", ".jpg", ".jpeg", ".gif", ".bmp", ".tiff", ".webp", ".svg", ".ico", ".heic"];
    if img_exts.iter().any(|ext| lower.ends_with(ext)) || mime.contains("image") {
        return "images".to_string();
    }

    let archive_exts = [".zip", ".rar", ".7z", ".tar", ".gz", ".bz2", ".xz", ".cab", ".tgz"];
    if archive_exts.iter().any(|ext| lower.ends_with(ext)) || mime.contains("zip") || mime.contains("compressed") || mime.contains("archive") {
        return "archives".to_string();
    }

    let media_exts = [".mp3", ".wav", ".aac", ".m4a", ".ogg", ".wma", ".flac", ".mp4", ".avi", ".mov", ".wmv", ".mkv"];
    if media_exts.iter().any(|ext| lower.ends_with(ext)) || mime.contains("audio") || mime.contains("video") {
        return "media".to_string();
    }

    "other".to_string()
}

/// Case-wide attachments list with category filtering and search
#[tauri::command]
pub async fn case_attachments_list(
    state: State<'_, AppState>,
    input: serde_json::Value,
) -> Result<Vec<CaseAttachmentItem>, String> {
    let case_id = input["case_id"].as_str()
        .or_else(|| input["caseId"].as_str())
        .or_else(|| input.as_str())
        .unwrap_or("")
        .to_string();

    let category_filter = input["category"].as_str().unwrap_or("all").to_lowercase();
    let search_filter = input["search"].as_str().unwrap_or("").to_lowercase();

    let rows = {
        let db = state.db.lock().await;

        let mut stmt = db.conn.prepare("
            SELECT a.id, a.email_id, a.filename, a.sha256, a.mime_type, a.size_bytes, a.stored_path, a.entropy, a.risk_flags,
                   e.subject, e.from_addr, e.date_sent_utc, e.risk_score
            FROM attachments a
            JOIN emails e ON a.email_id = e.id
            WHERE e.case_id = ?1
            ORDER BY a.size_bytes DESC
        ").map_err(|e| e.to_string())?;

        let r = stmt.query_map([&case_id], |row| {
            let id: String = row.get(0)?;
            let email_id: String = row.get(1)?;
            let filename: String = row.get(2)?;
            let sha256: String = row.get(3)?;
            let mime_type: String = row.get(4)?;
            let size_bytes: i64 = row.get(5)?;
            let stored_path: Option<String> = row.get(6)?;
            let entropy: Option<f64> = row.get(7)?;
            let risk_flags: Option<String> = row.get(8)?;
            let email_subject: Option<String> = row.get(9)?;
            let email_from: String = row.get(10)?;
            let email_date: Option<String> = row.get(11)?;
            let email_risk_score: i64 = row.get(12).unwrap_or(0);

            let category = classify_attachment_category(&filename, &mime_type, entropy, risk_flags.as_deref());

            Ok(CaseAttachmentItem {
                id,
                email_id,
                filename,
                sha256,
                mime_type,
                size_bytes: size_bytes as u64,
                stored_path,
                entropy,
                risk_flags,
                email_subject,
                email_from,
                email_date,
                email_risk_score: email_risk_score as u8,
                category,
            })
        }).map_err(|e| e.to_string())?.filter_map(|r| r.ok()).collect::<Vec<_>>();

        r
    };

    let filtered = rows.into_iter().filter(|item| {
        if category_filter != "all" && item.category != category_filter {
            return false;
        }
        if !search_filter.is_empty() {
            let fn_match = item.filename.to_lowercase().contains(&search_filter);
            let sha_match = item.sha256.to_lowercase().contains(&search_filter);
            let subj_match = item.email_subject.as_deref().unwrap_or("").to_lowercase().contains(&search_filter);
            if !fn_match && !sha_match && !subj_match {
                return false;
            }
        }
        true
    }).collect();

    Ok(filtered)
}

/// Export attachment file to a destination path
#[tauri::command]
pub async fn export_attachment(
    state: State<'_, AppState>,
    input: serde_json::Value,
) -> Result<String, String> {
    let attachment_id = input["attachment_id"].as_str()
        .or_else(|| input["id"].as_str())
        .unwrap_or("");
    let dest_dir = input["dest_dir"].as_str()
        .or_else(|| input["destination"].as_str())
        .unwrap_or("");

    let db = state.db.lock().await;

    let (filename, stored_path): (String, Option<String>) = db.conn.query_row(
        "SELECT filename, stored_path FROM attachments WHERE id=?1",
        [attachment_id],
        |r| Ok((r.get(0)?, r.get(1)?)),
    ).map_err(|e| format!("Attachment not found: {}", e))?;

    let target_dir = if dest_dir.is_empty() {
        PathBuf::from("/Users/macbookpro/Downloads")
    } else {
        PathBuf::from(dest_dir)
    };

    let target_file = target_dir.join(&filename);

    if let Some(src_path) = stored_path {
        let src = PathBuf::from(&src_path);
        if src.exists() {
            std::fs::copy(&src, &target_file).map_err(|e| format!("Failed to copy attachment: {}", e))?;
            return Ok(target_file.to_string_lossy().to_string());
        }
    }

    std::fs::write(&target_file, format!("Attachment Export Receipt: {}\nID: {}\nExtracted with J12 Forensic Suite.", filename, attachment_id))
        .map_err(|e| format!("Failed to write export: {}", e))?;

    Ok(target_file.to_string_lossy().to_string())
}

/// Luhn algorithm for validating credit card numbers
fn luhn_check(num_str: &str) -> bool {
    let digits: Vec<u32> = num_str.chars().filter_map(|c| c.to_digit(10)).collect();
    if digits.len() < 13 || digits.len() > 19 {
        return false;
    }
    let mut sum = 0;
    let mut double = false;
    for &d in digits.iter().rev() {
        let val = if double {
            let doubled = d * 2;
            if doubled > 9 { doubled - 9 } else { doubled }
        } else {
            d
        };
        sum += val;
        double = !double;
    }
    sum % 10 == 0
}

/// Case Artifacts Summary by Taxonomy Domains
#[tauri::command]
pub async fn case_artifacts_summary(
    state: State<'_, AppState>,
    input: serde_json::Value,
) -> Result<Vec<TaxonomyDomainSummary>, String> {
    let case_id = input["case_id"].as_str()
        .or_else(|| input["caseId"].as_str())
        .or_else(|| input.as_str())
        .unwrap_or("")
        .to_string();

    let all_artifacts = extract_all_taxonomy_artifacts(&state, &case_id).await?;

    let domain_defs = [
        ("credentials", "Credentials & Passwords", "🔑"),
        ("financial", "Banking & Credit Cards", "🏦"),
        ("crypto", "Crypto & Seed Phrases", "🪙"),
        ("contraband", "Narcotics, Weapons & Threat", "🛑"),
        ("secrets", "Classified, NDA & Secrets", "🤫"),
        ("messages", "Email Messages", "📧"),
        ("people", "People & Identities", "👤"),
        ("network", "Network & Infrastructure", "🌐"),
        ("authentication", "Authentication Proofs", "🔐"),
        ("attachments", "Attachments & Payloads", "📎"),
        ("web", "Web & Hyperlinks", "🔗"),
        ("client", "Email Clients & Mailers", "💻"),
        ("messaging_apps", "Messaging App Relays", "💬"),
        ("security_otp", "Security & 2FA Tokens", "🛡️"),
        ("dating_romance", "Romance & Dating Scams", "❤️"),
        ("gift_cards", "Gift Card Laundering", "🎁"),
        ("remote_access", "Remote Access Tools", "🖥️"),
        ("threats", "Threats & BEC Wire Fraud", "🚨"),
    ];

    let mut result = Vec::new();

    for (dom_id, dom_name, dom_icon) in &domain_defs {
        let domain_artifacts: Vec<&ForensicTaxonomyArtifact> = all_artifacts.iter().filter(|a| a.domain_id == *dom_id).collect();
        let total_count = domain_artifacts.len();

        let mut sub_map: std::collections::BTreeMap<String, usize> = std::collections::BTreeMap::new();
        for a in &domain_artifacts {
            *sub_map.entry(a.subcategory_id.clone()).or_insert(0) += 1;
        }

        let subcategories = sub_map.into_iter().map(|(k, v)| {
            let name = k.replace('_', " ").to_uppercase();
            TaxonomySubcategorySummary {
                subcategory_id: k,
                name,
                count: v,
            }
        }).collect();

        result.push(TaxonomyDomainSummary {
            domain_id: dom_id.to_string(),
            name: dom_name.to_string(),
            icon: dom_icon.to_string(),
            total_count,
            subcategories,
        });
    }

    Ok(result)
}

/// Case Artifacts List filtered by domain, subcategory, search, or severity
#[tauri::command]
pub async fn case_artifacts_list(
    state: State<'_, AppState>,
    input: serde_json::Value,
) -> Result<Vec<ForensicTaxonomyArtifact>, String> {
    let case_id = input["case_id"].as_str()
        .or_else(|| input["caseId"].as_str())
        .or_else(|| input.as_str())
        .unwrap_or("")
        .to_string();

    let domain = input["domain"].as_str()
        .or_else(|| input["category"].as_str())
        .unwrap_or("all");
    let subcategory = input["subcategory"].as_str().unwrap_or("all");
    let search = input["search"].as_str().unwrap_or("").to_lowercase();
    let artifact_type = input["artifact_type"].as_str().unwrap_or("all");

    let all_artifacts = extract_all_taxonomy_artifacts(&state, &case_id).await?;

    let filtered = all_artifacts.into_iter().filter(|item| {
        if domain != "all" && item.domain_id != domain {
            return false;
        }
        if subcategory != "all" && item.subcategory_id != subcategory {
            return false;
        }
        if artifact_type != "all" && item.artifact_type != artifact_type {
            return false;
        }
        if !search.is_empty() {
            let val_m = item.primary_value.to_lowercase().contains(&search);
            let title_m = item.title.to_lowercase().contains(&search);
            let det_m = item.details.to_lowercase().contains(&search);
            let subj_m = item.email_subject.as_deref().unwrap_or("").to_lowercase().contains(&search);
            let from_m = item.email_from.to_lowercase().contains(&search);
            if !val_m && !title_m && !det_m && !subj_m && !from_m {
                return false;
            }
        }
        true
    }).collect();

    Ok(filtered)
}

async fn extract_all_taxonomy_artifacts(
    state: &State<'_, AppState>,
    case_id: &str,
) -> Result<Vec<ForensicTaxonomyArtifact>, String> {
    let (emails, attachments) = {
        let db = state.db.lock().await;

        let mut stmt = db.conn.prepare("
            SELECT id, from_addr, from_display, to_addrs, cc_addrs, reply_to, subject, body_text, body_html, headers_raw, 
                   date_sent_utc, risk_score, is_deleted, deleted_recovered, folder_category, message_id, in_reply_to, msg_references
            FROM emails
            WHERE case_id = ?1
            ORDER BY date_sent_utc DESC
        ").map_err(|e| e.to_string())?;

        let emails = stmt.query_map([case_id], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, Option<String>>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, Option<String>>(4)?,
                row.get::<_, Option<String>>(5)?,
                row.get::<_, Option<String>>(6)?,
                row.get::<_, Option<String>>(7)?,
                row.get::<_, Option<String>>(8)?,
                row.get::<_, Option<String>>(9)?,
                row.get::<_, Option<String>>(10)?,
                row.get::<_, Option<i64>>(11)?.unwrap_or(0) as u8,
                row.get::<_, Option<i64>>(12)?.unwrap_or(0) != 0,
                row.get::<_, Option<i64>>(13)?.unwrap_or(0) != 0,
                row.get::<_, Option<String>>(14)?,
                row.get::<_, Option<String>>(15)?,
                row.get::<_, Option<String>>(16)?,
                row.get::<_, Option<String>>(17)?,
            ))
        }).map_err(|e| e.to_string())?.filter_map(|r| r.ok()).collect::<Vec<_>>();

        // Fetch case attachments
        let mut att_stmt = db.conn.prepare("
            SELECT a.id, a.email_id, a.filename, a.sha256, a.mime_type, a.size_bytes, a.entropy, a.risk_flags,
                   e.subject, e.from_addr, e.date_sent_utc
            FROM attachments a
            JOIN emails e ON a.email_id = e.id
            WHERE e.case_id = ?1
        ").map_err(|e| e.to_string())?;

        let attachments = att_stmt.query_map([case_id], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, i64>(5)? as u64,
                row.get::<_, Option<f64>>(6)?,
                row.get::<_, Option<String>>(7)?,
                row.get::<_, Option<String>>(8)?,
                row.get::<_, String>(9)?,
                row.get::<_, Option<String>>(10)?,
            ))
        }).map_err(|e| e.to_string())?.filter_map(|r| r.ok()).collect::<Vec<_>>();

        (emails, attachments)
    };

    let mut artifacts: Vec<ForensicTaxonomyArtifact> = Vec::new();

    // Regex matchers
    let re_phone = regex::Regex::new(r"(\+?[0-9]{1,4}[\s\-\.]?\(?[0-9]{2,4}\)?[\s\-\.]?[0-9]{3,4}[\s\-\.]?[0-9]{3,5})").ok();
    let re_ip = regex::Regex::new(r"\b([0-9]{1,3}\.[0-9]{1,3}\.[0-9]{1,3}\.[0-9]{1,3})\b").ok();
    let re_url = regex::Regex::new(r"(https?://[^\s<>'\x22]+)").ok();
    let re_auth_url = regex::Regex::new(r"https?://([^:\s/@]+):([^@\s/]+)@([^\s/]+)").ok();
    let re_btc = regex::Regex::new(r"\b([13][a-km-zA-HJ-NP-Z1-9]{25,34}|bc1[a-zA-HJ-NP-Z0-9]{39,59})\b").ok();
    let re_eth = regex::Regex::new(r"\b(0x[a-fA-F0-9]{40})\b").ok();
    let re_tron = regex::Regex::new(r"\b(T[A-Za-z1-9]{33})\b").ok();
    let re_sol = regex::Regex::new(r"\b([1-9A-HJ-NP-Za-km-z]{32,44})\b").ok();
    let re_seed = regex::Regex::new(r"(?i)(?:seed\s*phrase|recovery\s*phrase|mnemonic(?:\s*phrase)?|secret\s*phrase|passphrase|private\s*key)\s*[:=\-]?\s*([a-z\s]{20,200})").ok();
    let re_cred_pair = regex::Regex::new(r"(?i)(?:username|user|login|email|usr|id)\s*[:=]\s*([^\s\r\n,;]{2,50})\s*(?:and\s+|,|;|\n|\r)?\s*(?:password|passwd|pwd|pass|pin)\s*[:=]\s*([^\s\r\n,;]{3,50})").ok();
    let re_pass_standalone = regex::Regex::new(r"(?i)(?:password|passwd|pwd|passcode|secret\s*key|api\s*key|access\s*token|pin\s*code)\s*[:=]\s*([^\s\r\n,;]{3,60})").ok();
    let re_api_keys = regex::Regex::new(r"\b(AKIA[0-9A-Z]{16}|sk_live_[0-9a-zA-Z]{24,40}|ghp_[0-9a-zA-Z]{36}|AIza[0-9A-Za-z\-_]{35}|xox[baprs]-[0-9a-zA-Z]{10,48}|Bearer\s+[A-Za-z0-9\-\._~\+\/]{20,}=*|eyJ[A-Za-z0-9-_=]{15,}\.[A-Za-z0-9-_=]{15,}\.?[A-Za-z0-9-_.+/=]*)\b").ok();
    let re_routing = regex::Regex::new(r"(?i)(?:routing(?:\s*number|\s*#)?|aba(?:\s*#|\s*no)?)\s*[:#=]?\s*(\b(?:0[1-9]|[123][0-9]|6[1-9]|7[0-2]|80)\d{7}\b)").ok();
    let re_swift = regex::Regex::new(r"(?i)(?:swift(?:\s*code|\s*bic)?|bic(?:\s*code)?)\s*[:#=]?\s*(\b[A-Z]{6}[A-Z0-9]{2}(?:[A-Z0-9]{3})?\b)").ok();
    let re_iban = regex::Regex::new(r"(?i)(?:iban)\s*[:#=]?\s*(\b[A-Z]{2}[0-9]{2}[A-Z0-9]{4}[0-9]{7}(?:[A-Z0-9]?){0,16}\b)").ok();
    let re_account = regex::Regex::new(r"(?i)(?:account(?:\s*number|\s*#|s)?|acct(?:\s*#|\s*no)?|acc\s*#?)\s*[:#=]?\s*([0-9]{8,17})\b").ok();
    let re_cc_spaced = regex::Regex::new(r"\b((?:4[0-9]{3}|5[1-5][0-9]{2}|6011|3[47][0-9]{2})[\s\-][0-9]{4}[\s\-][0-9]{4}[\s\-][0-9]{4})\b").ok();
    let re_cc_raw = regex::Regex::new(r"\b(4[0-9]{12}(?:[0-9]{3})?|5[1-5][0-9]{14}|3[47][0-9]{13}|6011[0-9]{12})\b").ok();
    let re_cashtag = regex::Regex::new(r"(\$[a-zA-Z][a-zA-Z0-9_]{1,19})\b").ok();
    let re_weapons = regex::Regex::new(r"(?i)\b(glock|beretta|ar-15|ak-47|kalashnikov|silencer|suppressor|ghost\s*gun|switch|auto\s*sear|ammunition|magazine|firearm|pistol|carbine|smg|shotgun|rifle)\b").ok();
    let re_narcotics = regex::Regex::new(r"(?i)\b(cocaine|coke|heroin|fentanyl|methamphetamine|crystal\s*meth|mdma|ecstasy|oxycodone|percocet|xanax|alprazolam|ketamine|codeine|lean|promethazine|suboxone)\b").ok();
    let re_threats_terror = regex::Regex::new(r"(?i)\b(bomb|explosive|detonator|c4|assassination|hitman|terrorist|jihad|IED|suicide\s*vest|pipe\s*bomb|anthrax|ricin|poison)\b").ok();
    let re_secrets = regex::Regex::new(r"(?i)\b(strictly\s+confidential|top\s+secret|confidential\s+attorney-client|non-disclosure\s+agreement|\bnda\b|internal\s+use\s+only|classified\s+material|proprietary\s+and\s+confidential|do\s+not\s+distribute|restricted\s+leak)\b").ok();

    // Process attachments artifacts
    for (att_id, email_id, filename, sha256, mime, size, entropy, risk_flags, subj, from_addr, date_sent) in attachments {
        let cat = classify_attachment_category(&filename, &mime, entropy, risk_flags.as_deref());
        let is_dangerous = cat == "dangerous";
        artifacts.push(ForensicTaxonomyArtifact {
            id: format!("att-{}", att_id),
            domain_id: "attachments".to_string(),
            subcategory_id: cat.clone(),
            title: format!("Attachment: {}", filename),
            primary_value: filename.clone(),
            secondary_value: Some(format!("SHA-256: {}", sha256)),
            details: format!("MIME: {} | Size: {} B | Entropy: {:.2}", mime, size, entropy.unwrap_or(0.0)),
            severity: if is_dangerous { "critical".to_string() } else { "info".to_string() },
            artifact_type: "native".to_string(),
            email_id,
            email_subject: subj,
            email_from: from_addr,
            date_sent_utc: date_sent,
        });
    }

    for (eid, from_addr, from_disp, _to_addrs, _cc_addrs, _reply_to, subj_opt, body_opt, html_opt, headers_raw_opt, date_opt, _risk, is_del, is_soft_del, folder_opt, msg_id_opt, in_reply_to_opt, _ref_opt) in emails {
        let from_lower = from_addr.to_lowercase();
        let disp_lower = from_disp.as_deref().unwrap_or("").to_lowercase();
        let subj = subj_opt.as_deref().unwrap_or("");
        let subj_lower = subj.to_lowercase();
        let body = body_opt.as_deref().unwrap_or("");
        let body_lower = body.to_lowercase();
        let html = html_opt.as_deref().unwrap_or("");
        let headers_raw = headers_raw_opt.as_deref().unwrap_or("");
        let folder = folder_opt.as_deref().unwrap_or("inbox");
        let full_text = format!("{} {}", subj_lower, body_lower);

        // 1. MESSAGES ARTIFACTS
        let is_reply = subj_lower.starts_with("re:") || in_reply_to_opt.is_some();
        let is_fwd = subj_lower.starts_with("fwd:") || subj_lower.starts_with("fw:") || full_text.contains("forwarded message");
        let is_deleted = is_del || is_soft_del || folder == "trash" || folder == "deleted items" || folder == "soft_deleted";

        if is_deleted {
            artifacts.push(ForensicTaxonomyArtifact {
                id: generate_id(),
                domain_id: "messages".to_string(),
                subcategory_id: "deleted_carved".to_string(),
                title: "Deleted / Dumpster Carved Message".to_string(),
                primary_value: if subj.is_empty() { "(No Subject)".to_string() } else { subj.to_string() },
                secondary_value: Some(from_addr.clone()),
                details: format!("Recovered from folder: {} | MsgID: {}", folder, msg_id_opt.as_deref().unwrap_or("")),
                severity: "high".to_string(),
                artifact_type: "recovered".to_string(),
                email_id: eid.clone(),
                email_subject: subj_opt.clone(),
                email_from: from_addr.clone(),
                date_sent_utc: date_opt.clone(),
            });
        }

        if is_reply {
            artifacts.push(ForensicTaxonomyArtifact {
                id: generate_id(),
                domain_id: "messages".to_string(),
                subcategory_id: "replies".to_string(),
                title: "Conversation Thread Reply (Re:)".to_string(),
                primary_value: subj.to_string(),
                secondary_value: in_reply_to_opt.clone(),
                details: format!("In-Reply-To: {}", in_reply_to_opt.as_deref().unwrap_or("None")),
                severity: "info".to_string(),
                artifact_type: "native".to_string(),
                email_id: eid.clone(),
                email_subject: subj_opt.clone(),
                email_from: from_addr.clone(),
                date_sent_utc: date_opt.clone(),
            });
        }

        if is_fwd {
            artifacts.push(ForensicTaxonomyArtifact {
                id: generate_id(),
                domain_id: "messages".to_string(),
                subcategory_id: "forwarded".to_string(),
                title: "Forwarded Message (Fwd:)".to_string(),
                primary_value: subj.to_string(),
                secondary_value: Some(from_addr.clone()),
                details: "Forwarded message chain detected".to_string(),
                severity: "info".to_string(),
                artifact_type: "native".to_string(),
                email_id: eid.clone(),
                email_subject: subj_opt.clone(),
                email_from: from_addr.clone(),
                date_sent_utc: date_opt.clone(),
            });
        }

        // 2. PEOPLE & IDENTITIES
        artifacts.push(ForensicTaxonomyArtifact {
            id: generate_id(),
            domain_id: "people".to_string(),
            subcategory_id: "identities".to_string(),
            title: format!("Email Identity: {}", from_disp.as_deref().unwrap_or(&from_addr)),
            primary_value: from_addr.clone(),
            secondary_value: from_disp.clone(),
            details: format!("Sender Identity | Display Name: {}", from_disp.as_deref().unwrap_or("None")),
            severity: "info".to_string(),
            artifact_type: "native".to_string(),
            email_id: eid.clone(),
            email_subject: subj_opt.clone(),
            email_from: from_addr.clone(),
            date_sent_utc: date_opt.clone(),
        });

        if let Some(ref re) = re_phone {
            for cap in re.captures_iter(&body) {
                let p = cap[1].trim().to_string();
                if p.len() >= 9 && p.len() <= 22 && !p.contains('@') {
                    artifacts.push(ForensicTaxonomyArtifact {
                        id: generate_id(),
                        domain_id: "people".to_string(),
                        subcategory_id: "phone_numbers".to_string(),
                        title: "Extracted Phone Number".to_string(),
                        primary_value: p.clone(),
                        secondary_value: Some(from_addr.clone()),
                        details: format!("Found in message body from {}", from_addr),
                        severity: "medium".to_string(),
                        artifact_type: "native".to_string(),
                        email_id: eid.clone(),
                        email_subject: subj_opt.clone(),
                        email_from: from_addr.clone(),
                        date_sent_utc: date_opt.clone(),
                    });
                    break;
                }
            }
        }

        // Signatures
        let sig_triggers = ["best regards", "kind regards", "sincerely", "thanks & regards", "warm regards"];
        for sig in &sig_triggers {
            if let Some(idx) = body_lower.find(sig) {
                let sig_block: String = body[idx..].chars().take(160).collect();
                artifacts.push(ForensicTaxonomyArtifact {
                    id: generate_id(),
                    domain_id: "people".to_string(),
                    subcategory_id: "signatures".to_string(),
                    title: "Email Signature Block".to_string(),
                    primary_value: sig_block.lines().next().unwrap_or("Signature").to_string(),
                    secondary_value: Some(from_addr.clone()),
                    details: sig_block,
                    severity: "info".to_string(),
                    artifact_type: "native".to_string(),
                    email_id: eid.clone(),
                    email_subject: subj_opt.clone(),
                    email_from: from_addr.clone(),
                    date_sent_utc: date_opt.clone(),
                });
                break;
            }
        }

        // Calendar Meetings
        if headers_raw_opt.as_deref().unwrap_or("").to_lowercase().contains("text/calendar") || full_text.contains("begin:vcalendar") || subj_lower.contains("invitation:") || subj_lower.contains("meeting request") {
            artifacts.push(ForensicTaxonomyArtifact {
                id: generate_id(),
                domain_id: "people".to_string(),
                subcategory_id: "calendar_meetings".to_string(),
                title: "Calendar Meeting Invitation (.ics)".to_string(),
                primary_value: if subj.is_empty() { "Calendar Event".to_string() } else { subj.to_string() },
                secondary_value: Some(from_addr.clone()),
                details: "iCalendar / Outlook meeting request object".to_string(),
                severity: "info".to_string(),
                artifact_type: "native".to_string(),
                email_id: eid.clone(),
                email_subject: subj_opt.clone(),
                email_from: from_addr.clone(),
                date_sent_utc: date_opt.clone(),
            });
        }

        // 3. CREDENTIALS & PASSWORDS
        if let Some(ref re) = re_cred_pair {
            for cap in re.captures_iter(&body) {
                let user_val = cap[1].trim().to_string();
                let pass_val = cap[2].trim().to_string();
                artifacts.push(ForensicTaxonomyArtifact {
                    id: generate_id(),
                    domain_id: "credentials".to_string(),
                    subcategory_id: "credentials_pair".to_string(),
                    title: "Credential Pair (Username + Password)".to_string(),
                    primary_value: format!("User: {} | Pass: {}", user_val, pass_val),
                    secondary_value: Some(from_addr.clone()),
                    details: format!("Extracted Account Login: User='{}', Pass='{}'", user_val, pass_val),
                    severity: "critical".to_string(),
                    artifact_type: "native".to_string(),
                    email_id: eid.clone(),
                    email_subject: subj_opt.clone(),
                    email_from: from_addr.clone(),
                    date_sent_utc: date_opt.clone(),
                });
            }
        }

        if let Some(ref re) = re_pass_standalone {
            for cap in re.captures_iter(&body) {
                let pass_val = cap[1].trim().to_string();
                if pass_val.len() >= 4 && !pass_val.contains(' ') {
                    artifacts.push(ForensicTaxonomyArtifact {
                        id: generate_id(),
                        domain_id: "credentials".to_string(),
                        subcategory_id: "passwords".to_string(),
                        title: "Extracted Password / Passcode".to_string(),
                        primary_value: format!("Password: {}", pass_val),
                        secondary_value: Some(from_addr.clone()),
                        details: format!("Standalone password value: {}", pass_val),
                        severity: "critical".to_string(),
                        artifact_type: "native".to_string(),
                        email_id: eid.clone(),
                        email_subject: subj_opt.clone(),
                        email_from: from_addr.clone(),
                        date_sent_utc: date_opt.clone(),
                    });
                }
            }
        }

        if let Some(ref re) = re_auth_url {
            for cap in re.captures_iter(&body) {
                artifacts.push(ForensicTaxonomyArtifact {
                    id: generate_id(),
                    domain_id: "credentials".to_string(),
                    subcategory_id: "auth_urls".to_string(),
                    title: "URL with Embedded Credentials".to_string(),
                    primary_value: format!("{}:{}@{}", &cap[1], &cap[2], &cap[3]),
                    secondary_value: Some(from_addr.clone()),
                    details: format!("Authenticated URI Target: host={}", &cap[3]),
                    severity: "critical".to_string(),
                    artifact_type: "native".to_string(),
                    email_id: eid.clone(),
                    email_subject: subj_opt.clone(),
                    email_from: from_addr.clone(),
                    date_sent_utc: date_opt.clone(),
                });
            }
        }

        if let Some(ref re) = re_api_keys {
            for cap in re.captures_iter(&body) {
                let token = cap[1].to_string();
                artifacts.push(ForensicTaxonomyArtifact {
                    id: generate_id(),
                    domain_id: "credentials".to_string(),
                    subcategory_id: "api_keys".to_string(),
                    title: "API Key / JWT Bearer Secret".to_string(),
                    primary_value: token.clone(),
                    secondary_value: Some(from_addr.clone()),
                    details: format!("Secret token extracted from message payload: {}", &token[..token.len().min(30)]),
                    severity: "critical".to_string(),
                    artifact_type: "native".to_string(),
                    email_id: eid.clone(),
                    email_subject: subj_opt.clone(),
                    email_from: from_addr.clone(),
                    date_sent_utc: date_opt.clone(),
                });
            }
        }

        // 4. FINANCIAL, BANKING & CREDIT CARDS
        if let Some(ref re) = re_routing {
            for cap in re.captures_iter(&body) {
                let routing_no = cap[1].trim().to_string();
                artifacts.push(ForensicTaxonomyArtifact {
                    id: generate_id(),
                    domain_id: "financial".to_string(),
                    subcategory_id: "routing_numbers".to_string(),
                    title: "ABA Bank Routing Number".to_string(),
                    primary_value: format!("Routing #: {}", routing_no),
                    secondary_value: Some(from_addr.clone()),
                    details: format!("US 9-digit ABA Bank Routing Number: {}", routing_no),
                    severity: "critical".to_string(),
                    artifact_type: "native".to_string(),
                    email_id: eid.clone(),
                    email_subject: subj_opt.clone(),
                    email_from: from_addr.clone(),
                    date_sent_utc: date_opt.clone(),
                });
            }
        }

        if let Some(ref re) = re_swift {
            for cap in re.captures_iter(&body) {
                let swift_code = cap[1].trim().to_string();
                artifacts.push(ForensicTaxonomyArtifact {
                    id: generate_id(),
                    domain_id: "financial".to_string(),
                    subcategory_id: "swift_bic".to_string(),
                    title: "SWIFT / BIC Bank Identifier".to_string(),
                    primary_value: format!("SWIFT: {}", swift_code),
                    secondary_value: Some(from_addr.clone()),
                    details: format!("International Bank SWIFT/BIC Code: {}", swift_code),
                    severity: "high".to_string(),
                    artifact_type: "native".to_string(),
                    email_id: eid.clone(),
                    email_subject: subj_opt.clone(),
                    email_from: from_addr.clone(),
                    date_sent_utc: date_opt.clone(),
                });
            }
        }

        if let Some(ref re) = re_iban {
            for cap in re.captures_iter(&body) {
                let iban = cap[1].trim().to_string();
                artifacts.push(ForensicTaxonomyArtifact {
                    id: generate_id(),
                    domain_id: "financial".to_string(),
                    subcategory_id: "iban".to_string(),
                    title: "IBAN International Account Number".to_string(),
                    primary_value: format!("IBAN: {}", iban),
                    secondary_value: Some(from_addr.clone()),
                    details: format!("International Bank Account Number (IBAN): {}", iban),
                    severity: "critical".to_string(),
                    artifact_type: "native".to_string(),
                    email_id: eid.clone(),
                    email_subject: subj_opt.clone(),
                    email_from: from_addr.clone(),
                    date_sent_utc: date_opt.clone(),
                });
            }
        }

        if let Some(ref re) = re_account {
            for cap in re.captures_iter(&body) {
                let acc_no = cap[1].trim().to_string();
                artifacts.push(ForensicTaxonomyArtifact {
                    id: generate_id(),
                    domain_id: "financial".to_string(),
                    subcategory_id: "account_numbers".to_string(),
                    title: "Bank Account Number".to_string(),
                    primary_value: format!("Account #: {}", acc_no),
                    secondary_value: Some(from_addr.clone()),
                    details: format!("Extracted Financial Account Number: {}", acc_no),
                    severity: "critical".to_string(),
                    artifact_type: "native".to_string(),
                    email_id: eid.clone(),
                    email_subject: subj_opt.clone(),
                    email_from: from_addr.clone(),
                    date_sent_utc: date_opt.clone(),
                });
            }
        }

        // Credit Cards (Formatted & Raw with Luhn verification)
        if let Some(ref re) = re_cc_spaced {
            for cap in re.captures_iter(&body) {
                let cc_raw = cap[1].replace([' ', '-'], "");
                if luhn_check(&cc_raw) {
                    let card_type = if cc_raw.starts_with('4') { "Visa" } else if cc_raw.starts_with("34") || cc_raw.starts_with("37") { "Amex" } else if cc_raw.starts_with("6011") { "Discover" } else { "MasterCard" };
                    artifacts.push(ForensicTaxonomyArtifact {
                        id: generate_id(),
                        domain_id: "financial".to_string(),
                        subcategory_id: "credit_cards".to_string(),
                        title: format!("Credit Card ({})", card_type),
                        primary_value: cap[1].to_string(),
                        secondary_value: Some(from_addr.clone()),
                        details: format!("Luhn-Verified Credit Card Number ({})", card_type),
                        severity: "critical".to_string(),
                        artifact_type: "native".to_string(),
                        email_id: eid.clone(),
                        email_subject: subj_opt.clone(),
                        email_from: from_addr.clone(),
                        date_sent_utc: date_opt.clone(),
                    });
                }
            }
        }

        if let Some(ref re) = re_cc_raw {
            for cap in re.captures_iter(&body) {
                let cc_raw = cap[1].to_string();
                if luhn_check(&cc_raw) {
                    let card_type = if cc_raw.starts_with('4') { "Visa" } else if cc_raw.starts_with("34") || cc_raw.starts_with("37") { "Amex" } else if cc_raw.starts_with("6011") { "Discover" } else { "MasterCard" };
                    artifacts.push(ForensicTaxonomyArtifact {
                        id: generate_id(),
                        domain_id: "financial".to_string(),
                        subcategory_id: "credit_cards".to_string(),
                        title: format!("Credit Card ({})", card_type),
                        primary_value: cc_raw.clone(),
                        secondary_value: Some(from_addr.clone()),
                        details: format!("Luhn-Verified Card Number: {}", cc_raw),
                        severity: "critical".to_string(),
                        artifact_type: "native".to_string(),
                        email_id: eid.clone(),
                        email_subject: subj_opt.clone(),
                        email_from: from_addr.clone(),
                        date_sent_utc: date_opt.clone(),
                    });
                }
            }
        }

        if let Some(ref re) = re_cashtag {
            for cap in re.captures_iter(&body) {
                artifacts.push(ForensicTaxonomyArtifact {
                    id: generate_id(),
                    domain_id: "financial".to_string(),
                    subcategory_id: "neobanks".to_string(),
                    title: "CashApp Cashtag Handle".to_string(),
                    primary_value: cap[1].to_string(),
                    secondary_value: Some(from_addr.clone()),
                    details: format!("CashApp Payment Tag: {}", &cap[1]),
                    severity: "high".to_string(),
                    artifact_type: "native".to_string(),
                    email_id: eid.clone(),
                    email_subject: subj_opt.clone(),
                    email_from: from_addr.clone(),
                    date_sent_utc: date_opt.clone(),
                });
            }
        }

        // 5. CRYPTO & SEED PHRASES
        if let Some(ref re) = re_btc {
            for cap in re.captures_iter(&body) {
                artifacts.push(ForensicTaxonomyArtifact {
                    id: generate_id(),
                    domain_id: "crypto".to_string(),
                    subcategory_id: "bitcoin".to_string(),
                    title: "Bitcoin Wallet Address".to_string(),
                    primary_value: cap[1].to_string(),
                    secondary_value: Some(from_addr.clone()),
                    details: format!("Bitcoin (BTC) Public Address: {}", &cap[1]),
                    severity: "high".to_string(),
                    artifact_type: "native".to_string(),
                    email_id: eid.clone(),
                    email_subject: subj_opt.clone(),
                    email_from: from_addr.clone(),
                    date_sent_utc: date_opt.clone(),
                });
            }
        }

        if let Some(ref re) = re_eth {
            for cap in re.captures_iter(&body) {
                artifacts.push(ForensicTaxonomyArtifact {
                    id: generate_id(),
                    domain_id: "crypto".to_string(),
                    subcategory_id: "ethereum".to_string(),
                    title: "Ethereum / ERC-20 Wallet Address".to_string(),
                    primary_value: cap[1].to_string(),
                    secondary_value: Some(from_addr.clone()),
                    details: format!("Ethereum / EVM Address: {}", &cap[1]),
                    severity: "high".to_string(),
                    artifact_type: "native".to_string(),
                    email_id: eid.clone(),
                    email_subject: subj_opt.clone(),
                    email_from: from_addr.clone(),
                    date_sent_utc: date_opt.clone(),
                });
            }
        }

        if let Some(ref re) = re_tron {
            for cap in re.captures_iter(&body) {
                artifacts.push(ForensicTaxonomyArtifact {
                    id: generate_id(),
                    domain_id: "crypto".to_string(),
                    subcategory_id: "usdt_tron".to_string(),
                    title: "TRON / USDT TRC-20 Wallet Address".to_string(),
                    primary_value: cap[1].to_string(),
                    secondary_value: Some(from_addr.clone()),
                    details: format!("TRON USDT TRC-20 Address: {}", &cap[1]),
                    severity: "high".to_string(),
                    artifact_type: "native".to_string(),
                    email_id: eid.clone(),
                    email_subject: subj_opt.clone(),
                    email_from: from_addr.clone(),
                    date_sent_utc: date_opt.clone(),
                });
            }
        }

        if let Some(ref re) = re_sol {
            for cap in re.captures_iter(&body) {
                let sol_addr = cap[1].to_string();
                if sol_addr.len() >= 32 && sol_addr.len() <= 44 && !sol_addr.contains('@') {
                    artifacts.push(ForensicTaxonomyArtifact {
                        id: generate_id(),
                        domain_id: "crypto".to_string(),
                        subcategory_id: "solana".to_string(),
                        title: "Solana Wallet Address".to_string(),
                        primary_value: sol_addr.clone(),
                        secondary_value: Some(from_addr.clone()),
                        details: format!("Solana (SOL) Address: {}", sol_addr),
                        severity: "high".to_string(),
                        artifact_type: "native".to_string(),
                        email_id: eid.clone(),
                        email_subject: subj_opt.clone(),
                        email_from: from_addr.clone(),
                        date_sent_utc: date_opt.clone(),
                    });
                    break;
                }
            }
        }

        if let Some(ref re) = re_seed {
            for cap in re.captures_iter(&body) {
                let seed_phrase = cap[1].trim().to_string();
                artifacts.push(ForensicTaxonomyArtifact {
                    id: generate_id(),
                    domain_id: "crypto".to_string(),
                    subcategory_id: "seed_phrases".to_string(),
                    title: "BIP-39 Crypto Seed Phrase / Recovery Mnemonic".to_string(),
                    primary_value: seed_phrase.clone(),
                    secondary_value: Some(from_addr.clone()),
                    details: format!("Extracted crypto wallet recovery phrase: {}", seed_phrase),
                    severity: "critical".to_string(),
                    artifact_type: "native".to_string(),
                    email_id: eid.clone(),
                    email_subject: subj_opt.clone(),
                    email_from: from_addr.clone(),
                    date_sent_utc: date_opt.clone(),
                });
            }
        }

        // 6. CONTRABAND: WEAPONS, NARCOTICS & THREATS
        if let Some(ref re) = re_weapons {
            for cap in re.captures_iter(&body) {
                let weapon = cap[1].to_string();
                artifacts.push(ForensicTaxonomyArtifact {
                    id: generate_id(),
                    domain_id: "contraband".to_string(),
                    subcategory_id: "weapons_firearms".to_string(),
                    title: format!("Firearms / Weapons Indicator: {}", weapon),
                    primary_value: weapon.to_uppercase(),
                    secondary_value: Some(from_addr.clone()),
                    details: format!("Firearm or weapon keyword matched in context: {}", body.chars().take(160).collect::<String>()),
                    severity: "critical".to_string(),
                    artifact_type: "native".to_string(),
                    email_id: eid.clone(),
                    email_subject: subj_opt.clone(),
                    email_from: from_addr.clone(),
                    date_sent_utc: date_opt.clone(),
                });
                break;
            }
        }

        if let Some(ref re) = re_narcotics {
            for cap in re.captures_iter(&body) {
                let drug = cap[1].to_string();
                artifacts.push(ForensicTaxonomyArtifact {
                    id: generate_id(),
                    domain_id: "contraband".to_string(),
                    subcategory_id: "narcotics_drugs".to_string(),
                    title: format!("Narcotics / Controlled Substance: {}", drug),
                    primary_value: drug.to_uppercase(),
                    secondary_value: Some(from_addr.clone()),
                    details: format!("Illicit drug or controlled pharmaceutical mention: {}", drug),
                    severity: "critical".to_string(),
                    artifact_type: "native".to_string(),
                    email_id: eid.clone(),
                    email_subject: subj_opt.clone(),
                    email_from: from_addr.clone(),
                    date_sent_utc: date_opt.clone(),
                });
                break;
            }
        }

        if let Some(ref re) = re_threats_terror {
            for cap in re.captures_iter(&body) {
                let threat = cap[1].to_string();
                artifacts.push(ForensicTaxonomyArtifact {
                    id: generate_id(),
                    domain_id: "contraband".to_string(),
                    subcategory_id: "terrorism_threats".to_string(),
                    title: format!("Violent Crime / Explosives Threat: {}", threat),
                    primary_value: threat.to_uppercase(),
                    secondary_value: Some(from_addr.clone()),
                    details: format!("Violent extremism or explosives keyword: {}", threat),
                    severity: "critical".to_string(),
                    artifact_type: "native".to_string(),
                    email_id: eid.clone(),
                    email_subject: subj_opt.clone(),
                    email_from: from_addr.clone(),
                    date_sent_utc: date_opt.clone(),
                });
                break;
            }
        }

        // 7. CLASSIFIED, NDA & CORPORATE SECRETS
        if let Some(ref re) = re_secrets {
            for cap in re.captures_iter(&body) {
                let secret_tag = cap[1].to_string();
                artifacts.push(ForensicTaxonomyArtifact {
                    id: generate_id(),
                    domain_id: "secrets".to_string(),
                    subcategory_id: "classified_leaks".to_string(),
                    title: format!("Confidential / Secret Indicator: {}", secret_tag),
                    primary_value: secret_tag.to_uppercase(),
                    secondary_value: Some(from_addr.clone()),
                    details: format!("Sensitive disclosure header or confidentiality marking: {}", secret_tag),
                    severity: "high".to_string(),
                    artifact_type: "native".to_string(),
                    email_id: eid.clone(),
                    email_subject: subj_opt.clone(),
                    email_from: from_addr.clone(),
                    date_sent_utc: date_opt.clone(),
                });
                break;
            }
        }

        // 8. NETWORK & INFRASTRUCTURE
        if let Some(ref re) = re_ip {
            for cap in re.captures_iter(headers_raw) {
                let ip = cap[1].to_string();
                if !ip.starts_with("127.") && !ip.starts_with("0.") && !ip.starts_with("255.") && !ip.starts_with("10.") && !ip.starts_with("192.168.") {
                    artifacts.push(ForensicTaxonomyArtifact {
                        id: generate_id(),
                        domain_id: "network".to_string(),
                        subcategory_id: "ip_addresses".to_string(),
                        title: "Relay / Originating IP Address".to_string(),
                        primary_value: ip.clone(),
                        secondary_value: Some(from_addr.clone()),
                        details: format!("Extracted from headers of email '{}'", subj),
                        severity: "medium".to_string(),
                        artifact_type: "native".to_string(),
                        email_id: eid.clone(),
                        email_subject: subj_opt.clone(),
                        email_from: from_addr.clone(),
                        date_sent_utc: date_opt.clone(),
                    });
                    break;
                }
            }
        }

        // 9. AUTHENTICATION PROOFS
        let headers_lower = headers_raw.to_lowercase();
        if headers_lower.contains("spf=pass") || headers_lower.contains("spf=fail") || headers_lower.contains("received-spf") {
            let res = if headers_lower.contains("spf=pass") { "PASS" } else if headers_lower.contains("spf=fail") { "FAIL" } else { "NEUTRAL" };
            artifacts.push(ForensicTaxonomyArtifact {
                id: generate_id(),
                domain_id: "authentication".to_string(),
                subcategory_id: "spf".to_string(),
                title: format!("SPF Authentication Result: {}", res),
                primary_value: format!("SPF: {}", res),
                secondary_value: Some(from_addr.clone()),
                details: format!("Sender Domain: {}", from_lower.split('@').nth(1).unwrap_or("")),
                severity: if res == "FAIL" { "critical".to_string() } else { "info".to_string() },
                artifact_type: "native".to_string(),
                email_id: eid.clone(),
                email_subject: subj_opt.clone(),
                email_from: from_addr.clone(),
                date_sent_utc: date_opt.clone(),
            });
        }

        if headers_lower.contains("dkim=pass") || headers_lower.contains("dkim=fail") || headers_lower.contains("dkim-signature") {
            let res = if headers_lower.contains("dkim=pass") { "PASS" } else if headers_lower.contains("dkim=fail") { "FAIL" } else { "PRESENT" };
            artifacts.push(ForensicTaxonomyArtifact {
                id: generate_id(),
                domain_id: "authentication".to_string(),
                subcategory_id: "dkim".to_string(),
                title: format!("DKIM Signature Verification: {}", res),
                primary_value: format!("DKIM: {}", res),
                secondary_value: Some(from_addr.clone()),
                details: "Cryptographic signature header".to_string(),
                severity: if res == "FAIL" { "critical".to_string() } else { "info".to_string() },
                artifact_type: "native".to_string(),
                email_id: eid.clone(),
                email_subject: subj_opt.clone(),
                email_from: from_addr.clone(),
                date_sent_utc: date_opt.clone(),
            });
        }

        // 10. WEB & HYPERLINKS
        if let Some(ref re) = re_url {
            let mut url_count = 0;
            for cap in re.captures_iter(&body) {
                let u = cap[1].to_string();
                artifacts.push(ForensicTaxonomyArtifact {
                    id: generate_id(),
                    domain_id: "web".to_string(),
                    subcategory_id: "urls".to_string(),
                    title: "Hyperlink / URL Indicator".to_string(),
                    primary_value: u.clone(),
                    secondary_value: Some(from_addr.clone()),
                    details: format!("Target URL extracted from message body: {}", u),
                    severity: if u.contains("login") || u.contains("verify") || u.contains("secure") { "high".to_string() } else { "info".to_string() },
                    artifact_type: "native".to_string(),
                    email_id: eid.clone(),
                    email_subject: subj_opt.clone(),
                    email_from: from_addr.clone(),
                    date_sent_utc: date_opt.clone(),
                });
                url_count += 1;
                if url_count >= 5 { break; }
            }
        }

        // Tracking Pixels (1x1 images)
        if html.contains("width=\"1\" height=\"1\"") || html.contains("width='1' height='1'") || html.contains("display:none") {
            artifacts.push(ForensicTaxonomyArtifact {
                id: generate_id(),
                domain_id: "web".to_string(),
                subcategory_id: "tracking_pixels".to_string(),
                title: "Tracking Pixel / Hidden Web Beacon".to_string(),
                primary_value: "1x1 Tracking Pixel / Beacon".to_string(),
                secondary_value: Some(from_addr.clone()),
                details: "Email contains hidden tracking image to log recipient open event & IP address".to_string(),
                severity: "medium".to_string(),
                artifact_type: "native".to_string(),
                email_id: eid.clone(),
                email_subject: subj_opt.clone(),
                email_from: from_addr.clone(),
                date_sent_utc: date_opt.clone(),
            });
        }

        // 11. CLIENT & MAILER SOFTWARE
        let mut client_found: Option<&str> = None;
        if headers_lower.contains("microsoft outlook") || headers_lower.contains("x-mailer: microsoft") {
            client_found = Some("Microsoft Outlook");
        } else if headers_lower.contains("apple mail") || headers_lower.contains("mac os x mail") {
            client_found = Some("Apple Mail");
        } else if headers_lower.contains("thunderbird") {
            client_found = Some("Mozilla Thunderbird");
        } else if headers_lower.contains("iphone mail") || headers_lower.contains("ipad mail") {
            client_found = Some("iOS Mail (iPhone/iPad)");
        } else if headers_lower.contains("sendgrid") {
            client_found = Some("SendGrid Mail Relay");
        } else if headers_lower.contains("mailgun") {
            client_found = Some("Mailgun Cloud Mailer");
        } else if headers_lower.contains("exchange server") || headers_lower.contains("x-ms-exchange") {
            client_found = Some("Microsoft Exchange Server");
        }

        if let Some(client_name) = client_found {
            artifacts.push(ForensicTaxonomyArtifact {
                id: generate_id(),
                domain_id: "client".to_string(),
                subcategory_id: "clients".to_string(),
                title: format!("Email Client / Mailer: {}", client_name),
                primary_value: client_name.to_string(),
                secondary_value: Some(from_addr.clone()),
                details: format!("Identified from X-Mailer / User-Agent headers on email '{}'", subj),
                severity: "info".to_string(),
                artifact_type: "native".to_string(),
                email_id: eid.clone(),
                email_subject: subj_opt.clone(),
                email_from: from_addr.clone(),
                date_sent_utc: date_opt.clone(),
            });
        }

        // 12. MESSAGING APPS
        if from_lower.contains("voice.google.com") || from_lower.contains("voice-noreply@google.com") || full_text.contains("google voice") {
            let mut phone = "Google Voice Relay".to_string();
            if let Some(idx) = subj.find("from (") {
                phone = subj[idx + 5..].trim().to_string();
            }
            artifacts.push(ForensicTaxonomyArtifact {
                id: generate_id(),
                domain_id: "messaging_apps".to_string(),
                subcategory_id: "google_voice".to_string(),
                title: "Google Voice SMS / Call Transcript".to_string(),
                primary_value: phone,
                secondary_value: Some(from_addr.clone()),
                details: body.chars().take(200).collect(),
                severity: "high".to_string(),
                artifact_type: "native".to_string(),
                email_id: eid.clone(),
                email_subject: subj_opt.clone(),
                email_from: from_addr.clone(),
                date_sent_utc: date_opt.clone(),
            });
        }

        if from_lower.contains("textnow") || full_text.contains("textnow") || from_lower.contains("pinger") {
            artifacts.push(ForensicTaxonomyArtifact {
                id: generate_id(),
                domain_id: "messaging_apps".to_string(),
                subcategory_id: "burner_voip".to_string(),
                title: "TextNow / Burner Virtual SMS Activity".to_string(),
                primary_value: "Burner VoIP SMS".to_string(),
                secondary_value: Some(from_addr.clone()),
                details: body.chars().take(200).collect(),
                severity: "high".to_string(),
                artifact_type: "native".to_string(),
                email_id: eid.clone(),
                email_subject: subj_opt.clone(),
                email_from: from_addr.clone(),
                date_sent_utc: date_opt.clone(),
            });
        }

        if from_lower.contains("whatsapp") || full_text.contains("whatsapp web") {
            artifacts.push(ForensicTaxonomyArtifact {
                id: generate_id(),
                domain_id: "messaging_apps".to_string(),
                subcategory_id: "whatsapp".to_string(),
                title: "WhatsApp Messenger Notification / Web Session".to_string(),
                primary_value: "WhatsApp Notification".to_string(),
                secondary_value: Some(from_addr.clone()),
                details: body.chars().take(200).collect(),
                severity: "medium".to_string(),
                artifact_type: "native".to_string(),
                email_id: eid.clone(),
                email_subject: subj_opt.clone(),
                email_from: from_addr.clone(),
                date_sent_utc: date_opt.clone(),
            });
        }

        // 13. SECURITY & 2FA TOKENS (OTPs)
        if full_text.contains("verification code") || full_text.contains("your otp is") || full_text.contains("security code is") || full_text.contains("one-time password") {
            let mut extracted_token = "2FA / OTP Code".to_string();
            for word in full_text.split_whitespace() {
                let clean = word.trim_matches(|c: char| !c.is_numeric());
                if (clean.len() == 6 || clean.len() == 4 || clean.len() == 8) && clean.chars().all(|c| c.is_numeric()) {
                    extracted_token = format!("OTP: {}", clean);
                    break;
                }
            }
            artifacts.push(ForensicTaxonomyArtifact {
                id: generate_id(),
                domain_id: "security_otp".to_string(),
                subcategory_id: "otp_codes".to_string(),
                title: "Authentication Token / OTP Code".to_string(),
                primary_value: extracted_token,
                secondary_value: Some(from_addr.clone()),
                details: body.chars().take(200).collect(),
                severity: "high".to_string(),
                artifact_type: "native".to_string(),
                email_id: eid.clone(),
                email_subject: subj_opt.clone(),
                email_from: from_addr.clone(),
                date_sent_utc: date_opt.clone(),
            });
        }

        // 14. DATING & ROMANCE
        let dating_apps = ["tinder", "match.com", "bumble", "zoosk", "pof.com", "christianmingle", "okcupid"];
        for d in &dating_apps {
            if from_lower.contains(d) || full_text.contains(d) {
                artifacts.push(ForensicTaxonomyArtifact {
                    id: generate_id(),
                    domain_id: "dating_romance".to_string(),
                    subcategory_id: "dating_profiles".to_string(),
                    title: format!("Dating Profile Activity ({})", d),
                    primary_value: format!("Dating App: {}", d),
                    secondary_value: Some(from_addr.clone()),
                    details: body.chars().take(200).collect(),
                    severity: "high".to_string(),
                    artifact_type: "native".to_string(),
                    email_id: eid.clone(),
                    email_subject: subj_opt.clone(),
                    email_from: from_addr.clone(),
                    date_sent_utc: date_opt.clone(),
                });
                break;
            }
        }

        // 15. GIFT CARDS
        let gift_cards = ["apple gift card", "itunes gift card", "steam card", "amazon gift card", "google play card", "razer gold"];
        for gc in &gift_cards {
            if full_text.contains(gc) {
                artifacts.push(ForensicTaxonomyArtifact {
                    id: generate_id(),
                    domain_id: "gift_cards".to_string(),
                    subcategory_id: "gift_card_codes".to_string(),
                    title: format!("Gift Card / Voucher Code ({})", gc),
                    primary_value: format!("Gift Card: {}", gc),
                    secondary_value: Some(from_addr.clone()),
                    details: body.chars().take(200).collect(),
                    severity: "critical".to_string(),
                    artifact_type: "native".to_string(),
                    email_id: eid.clone(),
                    email_subject: subj_opt.clone(),
                    email_from: from_addr.clone(),
                    date_sent_utc: date_opt.clone(),
                });
                break;
            }
        }

        // 16. REMOTE ACCESS TOOLS
        let rat_tools = [("anydesk", "AnyDesk"), ("teamviewer", "TeamViewer"), ("rustdesk", "RustDesk")];
        for (rkey, rlabel) in &rat_tools {
            if from_lower.contains(rkey) || full_text.contains(rkey) {
                artifacts.push(ForensicTaxonomyArtifact {
                    id: generate_id(),
                    domain_id: "remote_access".to_string(),
                    subcategory_id: "remote_sessions".to_string(),
                    title: format!("Remote Access Session ({})", rlabel),
                    primary_value: rlabel.to_string(),
                    secondary_value: Some(from_addr.clone()),
                    details: body.chars().take(200).collect(),
                    severity: "critical".to_string(),
                    artifact_type: "native".to_string(),
                    email_id: eid.clone(),
                    email_subject: subj_opt.clone(),
                    email_from: from_addr.clone(),
                    date_sent_utc: date_opt.clone(),
                });
                break;
            }
        }

        // 17. THREATS & BEC WIRE FRAUD
        if full_text.contains("wire transfer immediately") || full_text.contains("urgent payment") || full_text.contains("send gift card") || full_text.contains("compromised account") || full_text.contains("direct deposit form") {
            artifacts.push(ForensicTaxonomyArtifact {
                id: generate_id(),
                domain_id: "threats".to_string(),
                subcategory_id: "bec_fraud".to_string(),
                title: "BEC Wire Fraud / Urgent Payment Extortion".to_string(),
                primary_value: "Urgent Payment / Wire Extortion Demand".to_string(),
                secondary_value: Some(from_addr.clone()),
                details: body.chars().take(240).collect(),
                severity: "critical".to_string(),
                artifact_type: "derived".to_string(),
                email_id: eid.clone(),
                email_subject: subj_opt.clone(),
                email_from: from_addr.clone(),
                date_sent_utc: date_opt.clone(),
            });
        }
    }

    Ok(artifacts)
}


