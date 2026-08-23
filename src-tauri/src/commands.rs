use crate::models::*;
use crate::db::{compute_sha256, compute_sha512, detect_format, generate_id, parse_dt};
use crate::AppState;
use crate::analysis::{analyze_headers, analyze_authentication, detect_spoofing, generate_findings, calculate_risk_score, NewFinding};
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
pub async fn email_get(state: State<'_, AppState>, input: EmptyInput) -> Result<Option<EmailMessage>, String> {
    let db = state.db.lock().await;
    let r = db.conn.query_row("SELECT id,evidence_id,case_id,message_id,from_addr,from_display,to_addrs,cc_addrs,subject,date_sent,date_sent_utc,headers_raw,body_text,body_html,folder_name,folder_category,is_deleted,deleted_recovered,risk_score,flags FROM emails WHERE id=?1", [&input.case_id],
        |row| Ok(EmailMessage { id: row.get(0)?, evidence_id: row.get(1)?, case_id: row.get(2)?, message_id: row.get(3)?, from_addr: row.get(4)?, from_display: row.get(5)?, to_addrs: row.get(6)?, cc_addrs: row.get(7)?, subject: row.get(8)?, date_sent: row.get(9)?, date_sent_utc: row.get(10)?, headers_raw: row.get(11)?, body_text: row.get(12)?, body_html: row.get(13)?, folder_name: row.get(14)?, folder_category: row.get(15)?, is_deleted: boolv(row,16), deleted_recovered: boolv(row,17), risk_score: u8v(row,18), flags: row.get(19)? }));
    match r { Ok(e) => Ok(Some(e)), Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None), Err(e) => Err(e.to_string()) }
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
pub async fn findings_list(state: State<'_, AppState>, input: EmptyInput) -> Result<Vec<Finding>, String> {
    let db = state.db.lock().await;
    let mut stmt = db.conn.prepare("SELECT id,case_id,type,severity,confidence,title,description,evidence_refs,email_ids,status,created_at,reviewed_by,reviewed_at FROM findings WHERE case_id=?1 ORDER BY created_at DESC").map_err(|e| e.to_string())?;
    let findings = stmt.query_map([&input.case_id], |row| {
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
    let fc: i64 = db.conn.query_row("SELECT COUNT(*) FROM findings WHERE case_id=?1", [&input.case_id], |r| r.get(0)).unwrap_or(0);
    
    // Severity breakdown for findings
    let mut severity_breakdown = std::collections::HashMap::new();
    for severity in &["critical", "high", "medium", "low"] {
        let count: i64 = db.conn.query_row(
            "SELECT COUNT(*) FROM findings WHERE case_id=?1 AND severity=?2",
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
            let date_str = email.date_sent.map(|d| d.to_rfc3339());
            
            tx.execute(
                "INSERT INTO emails (id, evidence_id, case_id, message_id, from_addr, from_display, to_addrs, cc_addrs, subject, date_sent, date_sent_utc, headers_raw, body_text, body_html, folder_name, folder_category, recovery_status, is_deleted, deleted_recovered, risk_score, flags, created_at)
                 VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18,?19,?20,?21,?22)",
                rusqlite::params![
                    &email_id, &evidence_id, &case_id, &email.message_id, &email.from_addr, &email.from_display,
                    &to_json, &cc_json, &email.subject, &date_str, &date_str, &email.headers_raw,
                    &email.body_text, &email.body_html, &email.folder_name, &email.folder_category, &email.recovery_status, 0i64, 0i64, 0i64, "[]", &chrono::Utc::now().to_rfc3339()
                ],
            ).map_err(|e| format!("Insert email {}: {}", email.message_id, e))?;
        }
        
        // Update evidence status before commit
        tx.execute(
            "UPDATE evidence_items SET parse_status='done', message_count=?1 WHERE id=?2",
            rusqlite::params![count as i64, &evidence_id],
        ).map_err(|e| format!("Update status: {}", e))?;
        
        tx.commit().map_err(|e| format!("Commit error: {}", e))?;
    }
    
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
pub async fn run_analysis(state: State<'_, AppState>, case_id: String) -> Result<u32, String> {
    // Clear existing findings for this case to avoid duplicates
    {
        let db = state.db.lock().await;
        db.conn.execute("DELETE FROM findings WHERE case_id=?1", [&case_id]).map_err(|e| format!("Clear findings: {}", e))?;
    }

    // Fetch all emails for this case
    let emails = {
        let db = state.db.lock().await;
        let mut stmt = db.conn.prepare(
            "SELECT id, from_addr, from_display, headers_raw FROM emails WHERE case_id=?1"
        ).map_err(|e| e.to_string())?;
        
        let rows = stmt.query_map([&case_id], |row| {
            Ok((
                row.get::<_, String>(0)?,      // id
                row.get::<_, String>(1)?,      // from_addr
                row.get::<_, Option<String>>(2)?, // from_display
                row.get::<_, Option<String>>(3)?, // headers_raw
            ))
        }).map_err(|e| e.to_string())?
          .collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())?;
        
        rows
    };
    
    let mut total_findings: u32 = 0;
    
    // Analyze each email
    for (email_id, from_addr, from_display, headers_raw) in &emails {
        let headers = headers_raw.as_deref().unwrap_or("");
        let from_domain = from_addr.split('@').nth(1).unwrap_or("");
        
        // Run analyses
        let header_analysis = analyze_headers(headers);
        let source_ip = header_analysis.originating_ip.as_deref();
        let auth_results = analyze_authentication(headers, from_domain, source_ip);
        let spoof_findings = detect_spoofing(from_addr, from_display.as_deref(), headers, &auth_results);
        let risk_score = calculate_risk_score(&header_analysis, &auth_results, &spoof_findings, &[]);
        
        // Generate findings
        let new_findings = generate_findings(
            email_id,
            &header_analysis,
            &auth_results,
            &spoof_findings,
            &[], // No attachment data in memory
        );
        
        // Store findings in database
        {
            let db = state.db.lock().await;
            for finding in &new_findings {
                let id = generate_id();
                let email_ids_json = serde_json::to_string(&finding.email_ids).unwrap_or_default();
                let evidence_refs = "[]".to_string();
                
                db.conn.execute(
                    "INSERT INTO findings (id, case_id, type, severity, confidence, title, description, evidence_refs, email_ids, status, created_at)
                     VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,'open',?10)",
                    rusqlite::params![
                        &id, &case_id, &finding.type_, &finding.severity, &finding.confidence,
                        &finding.title, &finding.description, &evidence_refs, &email_ids_json, &Utc::now().to_rfc3339()
                    ],
                ).map_err(|e| format!("Insert finding: {}", e))?;
            }
            
            // Update risk score on email
            db.conn.execute(
                "UPDATE emails SET risk_score = ?1 WHERE id = ?2",
                rusqlite::params![risk_score as i64, email_id],
            ).ok();
        }
        
        total_findings += new_findings.len() as u32;
    }
    
    Ok(total_findings)
}

/// Update finding status (review workflow)
#[tauri::command]
pub async fn update_finding_status(
    state: State<'_, AppState>,
    finding_id: String,
    new_status: String,
    reviewed_by: String,
) -> Result<(), String> {
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
    finding_id: String,
    note: String,
    author: String,
) -> Result<(), String> {
    let db = state.db.lock().await;
    
    // Get existing description and append note
    let existing: String = db.conn.query_row(
        "SELECT description FROM findings WHERE id=?1",
        [&finding_id],
        |row| row.get(0),
    ).map_err(|e| e.to_string())?;
    
    let updated = format!("{}\n\n[{}] {}", existing, Utc::now().format("%Y-%m-%d %H:%M"), note);
    
    db.conn.execute(
        "UPDATE findings SET description = ?1 WHERE id = ?2",
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
