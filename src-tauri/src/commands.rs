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
            
            // Insert attachments
            for att in &email.attachments {
                let att_id = generate_id();
                let sha256 = crate::parser::sha256_data(&att.data);
                let size = att.data.len() as i64;
                tx.execute(
                    "INSERT INTO attachments (id, email_id, filename, sha256, mime_type, size_bytes, stored_path, entropy, risk_flags)
                     VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9)",
                    rusqlite::params![
                        &att_id, &email_id, &att.filename, &att.content_type, &att.content_type, &size, "", 0.0, "[]"
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

/// Get attachments for an email
#[tauri::command]
pub async fn email_attachments(state: State<'_, AppState>, email_id: String) -> Result<Vec<Attachment>, String> {
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

/// Auto-detect potential targets from email data
#[tauri::command]
pub async fn auto_detect_targets(state: State<'_, AppState>, case_id: String) -> Result<serde_json::Value, String> {
    let db = state.db.lock().await;
    
    // Find most frequent email addresses (sent + received)
    let mut stmt = db.conn.prepare("
        SELECT addr, SUM(cnt) as total FROM (
            SELECT from_addr as addr, COUNT(*) as cnt FROM emails WHERE case_id=?1 GROUP BY from_addr
            UNION ALL
            SELECT json_each.value as addr, COUNT(*) as cnt FROM emails, json_each(emails.to_addrs) WHERE emails.case_id=?1 GROUP BY json_each.value
        ) GROUP BY addr ORDER BY total DESC LIMIT 20
    ").map_err(|e| e.to_string())?;
    
    let rows: Vec<(String, i64)> = stmt.query_map([&case_id], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
    }).map_err(|e| e.to_string())?.collect::<Result<Vec<_>,_>>().map_err(|e| e.to_string())?;
    
    // For each address, get display name and stats
    let mut targets = Vec::new();
    for (addr, count) in &rows {
        // Get display name
        let display_name: Option<String> = db.conn.query_row(
            "SELECT DISTINCT from_display FROM emails WHERE from_addr=?1 AND from_display IS NOT NULL AND from_display != '' LIMIT 1",
            [addr.as_str()],
            |row| row.get(0),
        ).ok();
        
        // Get sent count
        let sent: i64 = db.conn.query_row(
            "SELECT COUNT(*) FROM emails WHERE case_id=?1 AND from_addr=?2",
            rusqlite::params![&case_id, addr.as_str()],
            |row| row.get(0),
        ).unwrap_or(0);
        
        // Get received count
        let received: i64 = db.conn.query_row(
            "SELECT COUNT(*) FROM emails WHERE case_id=?1 AND to_addrs LIKE ?2",
            rusqlite::params![&case_id, format!("%{}%", addr.as_str())],
            |row| row.get(0),
        ).unwrap_or(0);
        
        targets.push(serde_json::json!({
            "email": addr,
            "display_name": display_name,
            "total_emails": count,
            "sent": sent,
            "received": received,
        }));
    }
    
    Ok(serde_json::json!({
        "targets": targets,
        "case_id": case_id,
    }))
}
#[tauri::command]
pub async fn target_profile(state: State<'_, AppState>, case_id: String) -> Result<serde_json::Value, String> {
    let db = state.db.lock().await;
    
    // Get case info
    let case: (String, Option<String>, Option<String>, Option<String>) = db.conn.query_row(
        "SELECT title, target_email, target_name, target_organization FROM cases WHERE id=?1",
        [&case_id],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
    ).map_err(|e| e.to_string())?;
    
    let (case_title, target_email, target_name, target_organization) = case;
    
    // If no target email, return basic info
    let email = target_email.clone().unwrap_or_default();
    
    // Count emails where target is sender
    let sent_count: i64 = if email.is_empty() { 0 } else {
        db.conn.query_row(
            "SELECT COUNT(*) FROM emails WHERE case_id=?1 AND from_addr LIKE ?2",
            rusqlite::params![&case_id, format!("%{}%", email)],
            |r| r.get(0),
        ).unwrap_or(0)
    };
    
    // Count emails where target is recipient
    let received_count: i64 = if email.is_empty() { 0 } else {
        db.conn.query_row(
            "SELECT COUNT(*) FROM emails WHERE case_id=?1 AND (to_addrs LIKE ?2 OR cc_addrs LIKE ?2)",
            rusqlite::params![&case_id, format!("%{}%", email)],
            |r| r.get(0),
        ).unwrap_or(0)
    };
    
    // Get first and last seen dates
    let first_seen: Option<String> = if email.is_empty() { None } else {
        db.conn.query_row(
            "SELECT MIN(date_sent_utc) FROM emails WHERE case_id=?1 AND (from_addr LIKE ?2 OR to_addrs LIKE ?2 OR cc_addrs LIKE ?2)",
            rusqlite::params![&case_id, format!("%{}%", email)],
            |r| r.get::<_, Option<String>>(0),
        ).ok().flatten()
    };
    
    let last_seen: Option<String> = if email.is_empty() { None } else {
        db.conn.query_row(
            "SELECT MAX(date_sent_utc) FROM emails WHERE case_id=?1 AND (from_addr LIKE ?2 OR to_addrs LIKE ?2 OR cc_addrs LIKE ?2)",
            rusqlite::params![&case_id, format!("%{}%", email)],
            |r| r.get::<_, Option<String>>(0),
        ).ok().flatten()
    };
    
    // Get top correspondents (who target emails most)
    let top_correspondents: Vec<(String, i64)> = if email.is_empty() { vec![] } else {
        let mut stmt = db.conn.prepare(
            "SELECT from_addr, COUNT(*) as cnt FROM emails WHERE case_id=?1 AND (to_addrs LIKE ?2 OR cc_addrs LIKE ?2) GROUP BY from_addr ORDER BY cnt DESC LIMIT 10"
        ).ok();
        
        match stmt {
            Some(mut s) => {
                s.query_map(rusqlite::params![&case_id, format!("%{}%", email)], |row| {
                    Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
                }).ok().map(|r| r.filter_map(|x| x.ok()).collect()).unwrap_or_default()
            }
            None => vec![]
        }
    };
    
    // Get top subjects
    let top_subjects: Vec<(String, i64)> = if email.is_empty() { vec![] } else {
        let mut stmt = db.conn.prepare(
            "SELECT subject, COUNT(*) as cnt FROM emails WHERE case_id=?1 AND (from_addr LIKE ?2 OR to_addrs LIKE ?2 OR cc_addrs LIKE ?2) AND subject IS NOT NULL GROUP BY subject ORDER BY cnt DESC LIMIT 10"
        ).ok();
        
        match stmt {
            Some(mut s) => {
                s.query_map(rusqlite::params![&case_id, format!("%{}%", email)], |row| {
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
            rusqlite::params![&case_id, format!("%{}%", email)],
            |r| r.get::<_, f64>(0),
        ).unwrap_or(0.0)
    };
    
    // Get all display names used by target
    let display_names: Vec<String> = if email.is_empty() { vec![] } else {
        let mut stmt = db.conn.prepare(
            "SELECT DISTINCT from_display FROM emails WHERE case_id=?1 AND from_addr LIKE ?2 AND from_display IS NOT NULL AND from_display != ''"
        ).ok();
        
        match stmt {
            Some(mut s) => {
                s.query_map(rusqlite::params![&case_id, format!("%{}%", email)], |row| {
                    Ok(row.get::<_, String>(0)?)
                }).ok().map(|r| r.filter_map(|x| x.ok()).collect()).unwrap_or_default()
            }
            None => vec![]
        }
    };
    
    Ok(serde_json::json!({
        "case_id": case_id,
        "case_title": case_title,
        "target_email": target_email,
        "target_name": target_name,
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

/// Advanced search with operators
#[tauri::command]
pub async fn advanced_search(state: State<'_, AppState>, input: SearchInput) -> Result<Vec<EmailMessage>, String> {
    let db = state.db.lock().await;
    let limit = input.limit.unwrap_or(100) as i64;
    
    let query = input.query.trim();
    let mut sql = String::from("SELECT id,evidence_id,case_id,message_id,from_addr,from_display,to_addrs,cc_addrs,subject,date_sent,date_sent_utc,headers_raw,body_text,body_html,folder_name,folder_category,is_deleted,deleted_recovered,risk_score,flags FROM emails WHERE case_id=?1");
    let mut params: Vec<Box<dyn rusqlite::ToSql>> = vec![Box::new(input.case_id.clone())];
    
    if query.contains(':') {
        let parts: Vec<&str> = query.split_whitespace().collect();
        for part in &parts {
            if let Some((key, value)) = part.split_once(':') {
                let value = value.trim_matches('"');
                match key.to_lowercase().as_str() {
                    "from" => {
                        sql.push_str(" AND from_addr LIKE ?");
                        params.push(Box::new(format!("%{}%", value)));
                    }
                    "to" => {
                        sql.push_str(" AND (to_addrs LIKE ? OR cc_addrs LIKE ?)");
                        params.push(Box::new(format!("%{}%", value)));
                        params.push(Box::new(format!("%{}%", value)));
                    }
                    "subject" => {
                        sql.push_str(" AND subject LIKE ?");
                        params.push(Box::new(format!("%{}%", value)));
                    }
                    "body" => {
                        sql.push_str(" AND body_text LIKE ?");
                        params.push(Box::new(format!("%{}%", value)));
                    }
                    "domain" => {
                        sql.push_str(" AND (from_addr LIKE ? OR to_addrs LIKE ?)");
                        params.push(Box::new(format!("%{}%", value)));
                        params.push(Box::new(format!("%{}%", value)));
                    }
                    "after" => {
                        sql.push_str(" AND date_sent_utc >= ?");
                        params.push(Box::new(value.to_string()));
                    }
                    "before" => {
                        sql.push_str(" AND date_sent_utc <= ?");
                        params.push(Box::new(value.to_string()));
                    }
                    "risk" => {
                        if value.starts_with('>') {
                            let threshold: i64 = value[1..].parse().unwrap_or(50);
                            sql.push_str(" AND risk_score >= ?");
                            params.push(Box::new(threshold));
                        }
                    }
                    "has" => {
                        match value.to_lowercase().as_str() {
                            "attachment" | "attachments" => {
                                sql.push_str(" AND id IN (SELECT email_id FROM attachments)");
                            }
                            "url" | "urls" => {
                                sql.push_str(" AND (body_text LIKE '%http%' OR body_html LIKE '%http%')");
                            }
                            "ip" | "ips" => {
                                sql.push_str(" AND headers_raw LIKE '%[0-9]%'");
                            }
                            _ => {}
                        }
                    }
                    "ip" => {
                        sql.push_str(" AND headers_raw LIKE ?");
                        params.push(Box::new(format!("%{}%", value)));
                    }
                    "hash" => {
                        sql.push_str(" AND id IN (SELECT email_id FROM attachments WHERE sha256 LIKE ?)");
                        params.push(Box::new(format!("%{}%", value)));
                    }
                    "filename" => {
                        sql.push_str(" AND id IN (SELECT email_id FROM attachments WHERE filename LIKE ?)");
                        params.push(Box::new(format!("%{}%", value)));
                    }
                    "folder" => {
                        sql.push_str(" AND folder_category = ?");
                        params.push(Box::new(value.to_lowercase()));
                    }
                    _ => {}
                }
            }
        }
    } else {
        sql.push_str(" AND (from_addr LIKE ? OR to_addrs LIKE ? OR subject LIKE ? OR body_text LIKE ?)");
        let pattern = format!("%{}%", query);
        params.push(Box::new(pattern.clone()));
        params.push(Box::new(pattern.clone()));
        params.push(Box::new(pattern.clone()));
        params.push(Box::new(pattern));
    }
    
    sql.push_str(" ORDER BY date_sent_utc DESC LIMIT ?");
    params.push(Box::new(limit));
    
    let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();
    let mut stmt = db.conn.prepare(&sql).map_err(|e| e.to_string())?;
    let emails = stmt.query_map(param_refs.as_slice(), |row| {
        Ok(EmailMessage { id: row.get(0)?, evidence_id: row.get(1)?, case_id: row.get(2)?, message_id: row.get(3)?, from_addr: row.get(4)?, from_display: row.get(5)?, to_addrs: row.get(6)?, cc_addrs: row.get(7)?, subject: row.get(8)?, date_sent: row.get(9)?, date_sent_utc: row.get(10)?, headers_raw: row.get(11)?, body_text: row.get(12)?, body_html: row.get(13)?, folder_name: row.get(14)?, folder_category: row.get(15)?, is_deleted: boolv(row,16), deleted_recovered: boolv(row,17), risk_score: u8v(row,18), flags: row.get(19)? })
    }).map_err(|e| e.to_string())?.collect::<Result<Vec<_>,_>>().map_err(|e| e.to_string())?;
    Ok(emails)
}

/// Extract and store entities from emails
#[tauri::command]
pub async fn extract_entities(state: State<'_, AppState>, case_id: String) -> Result<u32, String> {
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
    
    let mut entity_counts: std::collections::HashMap<String, (Option<String>, i64, i64, Option<String>, Option<String>)> = std::collections::HashMap::new();
    
    for (from_addr, from_display, to_addrs, cc_addrs, date) in rows {
        // From address (sent)
        let entry = entity_counts.entry(from_addr.clone()).or_insert((from_display.clone(), 0, 0, date.clone(), date.clone()));
        entry.1 += 1;
        update_date_range(&mut entry.3, &mut entry.4, &date);
        
        // To addresses (received)
        let to_list: Vec<String> = serde_json::from_str(&to_addrs).unwrap_or_default();
        for to_addr in to_list {
            let entry = entity_counts.entry(to_addr).or_insert((None, 0, 0, date.clone(), date.clone()));
            entry.2 += 1;
            update_date_range(&mut entry.3, &mut entry.4, &date);
        }
        
        // CC addresses
        let cc_list: Vec<String> = serde_json::from_str(&cc_addrs).unwrap_or_default();
        for cc_addr in cc_list {
            let entry = entity_counts.entry(cc_addr).or_insert((None, 0, 0, date.clone(), date.clone()));
            entry.2 += 1;
            update_date_range(&mut entry.3, &mut entry.4, &date);
        }
    }
    
    db.conn.execute("DELETE FROM entities WHERE case_id=?1", [&case_id]).ok();
    
    let mut count = 0;
    for (email, (display_name, sent, received, first_seen, last_seen)) in entity_counts {
        let id = generate_id();
        db.conn.execute(
            "INSERT INTO entities (id, case_id, email_address, display_name, first_seen, last_seen, sent_count, received_count, role) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,'unknown')",
            rusqlite::params![&id, &case_id, &email, &display_name, &first_seen, &last_seen, sent as i64, received as i64],
        ).map_err(|e| format!("Insert entity: {}", e))?;
        count += 1;
    }
    
    Ok(count)
}

/// Get entities for a case
#[tauri::command]
pub async fn entity_list(state: State<'_, AppState>, input: EmptyInput) -> Result<Vec<Entity>, String> {
    let db = state.db.lock().await;
    let mut stmt = db.conn.prepare("SELECT id,case_id,email_address,display_name,first_seen,last_seen,sent_count,received_count,role FROM entities WHERE case_id=?1 ORDER BY (sent_count + received_count) DESC").map_err(|e| e.to_string())?;
    let entities = stmt.query_map([&input.case_id], |row| {
        Ok(Entity { id: row.get(0)?, case_id: row.get(1)?, email_address: row.get(2)?, display_name: row.get(3)?, first_seen: row.get(4)?, last_seen: row.get(5)?, sent_count: row.get(6)?, received_count: row.get(7)?, role: row.get(8)? })
    }).map_err(|e| e.to_string())?.collect::<Result<Vec<_>,_>>().map_err(|e| e.to_string())?;
    Ok(entities)
}

/// Get single entity deep-dive
#[tauri::command]
pub async fn entity_dive(state: State<'_, AppState>, input: EntityInput) -> Result<serde_json::Value, String> {
    let db = state.db.lock().await;
    
    let entity: (String, Option<String>, Option<String>, Option<String>, i64, i64) = db.conn.query_row(
        "SELECT email_address, display_name, first_seen, last_seen, sent_count, received_count FROM entities WHERE case_id=?1 AND email_address=?2",
        rusqlite::params![&input.case_id, &input.email_address],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?, row.get(5)?)),
    ).map_err(|e| e.to_string())?;
    
    let mut stmt = db.conn.prepare(
        "SELECT from_addr, COUNT(*) as cnt FROM emails WHERE case_id=?1 AND (to_addrs LIKE ?2 OR cc_addrs LIKE ?2) GROUP BY from_addr ORDER BY cnt DESC LIMIT 10"
    ).map_err(|e| e.to_string())?;
    let sent_to: Vec<(String, i64)> = stmt.query_map(rusqlite::params![&input.case_id, format!("%{}%", entity.0)], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
    }).map_err(|e| e.to_string())?.collect::<Result<Vec<_>,_>>().map_err(|e| e.to_string())?;
    
    let mut stmt2 = db.conn.prepare(
        "SELECT to_addrs FROM emails WHERE case_id=?1 AND from_addr=?2"
    ).map_err(|e| e.to_string())?;
    let received_from_rows: Vec<Vec<String>> = stmt2.query_map(rusqlite::params![&input.case_id, entity.0.clone()], |row| {
        let to_addrs: String = row.get(0)?;
        let addrs: Vec<String> = serde_json::from_str(&to_addrs).unwrap_or_default();
        Ok(addrs)
    }).map_err(|e| e.to_string())?.collect::<Result<Vec<_>,_>>().map_err(|e| e.to_string())?;
    
    let mut received_counts: std::collections::HashMap<String, i64> = std::collections::HashMap::new();
    for addrs in received_from_rows {
        for addr in addrs {
            *received_counts.entry(addr).or_insert(0) += 1;
        }
    }
    let mut received_from_vec: Vec<(String, i64)> = received_counts.into_iter().collect();
    received_from_vec.sort_by(|a, b| b.1.cmp(&a.1));
    received_from_vec.truncate(10);
    
    Ok(serde_json::json!({
        "email": entity.0,
        "display_name": entity.1,
        "first_seen": entity.2,
        "last_seen": entity.3,
        "sent_count": entity.4,
        "received_count": entity.5,
        "sent_to": sent_to,
        "received_from": received_from_vec,
    }))
}

/// Get emails for a specific entity (with date/attachment filters)
#[tauri::command]
pub async fn entity_emails(state: State<'_, AppState>, input: serde_json::Value) -> Result<Vec<EmailMessage>, String> {
    let db = state.db.lock().await;
    let case_id = input["case_id"].as_str().unwrap_or("");
    let email_addr = input["email"].as_str().unwrap_or("");
    let date_from = input["date_from"].as_str().unwrap_or("");
    let date_to = input["date_to"].as_str().unwrap_or("");
    let has_attachment = input["has_attachment"].as_bool().unwrap_or(false);
    
    let mut sql = String::from("
        SELECT id, evidence_id, case_id, message_id, from_addr, from_display, to_addrs, cc_addrs, 
               subject, date_sent, date_sent_utc, headers_raw, body_text, body_html, 
               folder_name, folder_category, is_deleted, deleted_recovered, risk_score, flags
        FROM emails 
        WHERE case_id=?1 AND (from_addr=?2 OR to_addrs LIKE ?3 OR cc_addrs LIKE ?3)
    ");
    let mut params: Vec<Box<dyn rusqlite::ToSql>> = vec![
        Box::new(case_id.to_string()),
        Box::new(email_addr.to_string()),
        Box::new(format!("%{}%", email_addr)),
    ];
    
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
    
    sql.push_str(" ORDER BY date_sent_utc DESC LIMIT 200");
    
    let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();
    let mut stmt = db.conn.prepare(&sql).map_err(|e| e.to_string())?;
    let emails = stmt.query_map(param_refs.as_slice(), |row| {
        Ok(EmailMessage { id: row.get(0)?, evidence_id: row.get(1)?, case_id: row.get(2)?, message_id: row.get(3)?, from_addr: row.get(4)?, from_display: row.get(5)?, to_addrs: row.get(6)?, cc_addrs: row.get(7)?, subject: row.get(8)?, date_sent: row.get(9)?, date_sent_utc: row.get(10)?, headers_raw: row.get(11)?, body_text: row.get(12)?, body_html: row.get(13)?, folder_name: row.get(14)?, folder_category: row.get(15)?, is_deleted: boolv(row,16), deleted_recovered: boolv(row,17), risk_score: u8v(row,18), flags: row.get(19)? })
    }).map_err(|e| e.to_string())?.collect::<Result<Vec<_>,_>>().map_err(|e| e.to_string())?;
    Ok(emails)
}
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
    
    let mut stmt = db.conn.prepare("
        SELECT id, evidence_id, case_id, message_id, from_addr, from_display, to_addrs, cc_addrs, 
               subject, date_sent, date_sent_utc, headers_raw, body_text, body_html, 
               folder_name, folder_category, is_deleted, deleted_recovered, risk_score, flags
        FROM emails 
        WHERE case_id=?1 AND (
            (from_addr=?2 AND (to_addrs LIKE ?3 OR cc_addrs LIKE ?3))
            OR (from_addr=?4 AND (to_addrs LIKE ?5 OR cc_addrs LIKE ?5))
        )
        ORDER BY date_sent_utc DESC
        LIMIT 200
    ").map_err(|e| e.to_string())?;
    
    let emails = stmt.query_map(rusqlite::params![
        case_id, from_addr, format!("%{}%", to_addr), to_addr, format!("%{}%", from_addr)
    ], |row| {
        Ok(EmailMessage { id: row.get(0)?, evidence_id: row.get(1)?, case_id: row.get(2)?, message_id: row.get(3)?, from_addr: row.get(4)?, from_display: row.get(5)?, to_addrs: row.get(6)?, cc_addrs: row.get(7)?, subject: row.get(8)?, date_sent: row.get(9)?, date_sent_utc: row.get(10)?, headers_raw: row.get(11)?, body_text: row.get(12)?, body_html: row.get(13)?, folder_name: row.get(14)?, folder_category: row.get(15)?, is_deleted: boolv(row,16), deleted_recovered: boolv(row,17), risk_score: u8v(row,18), flags: row.get(19)? })
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
    
    let mut stmt = db.conn.prepare("
        SELECT email_address, display_name, sent_count, received_count, (sent_count + received_count) as total
        FROM entities WHERE case_id=?1 ORDER BY total DESC LIMIT 100
    ").map_err(|e| e.to_string())?;
    
    let nodes: Vec<serde_json::Value> = stmt.query_map([&input.case_id], |row| {
        Ok(serde_json::json!({
            "id": row.get::<_, String>(0)?,
            "name": row.get::<_, Option<String>>(1)?,
            "sent": row.get::<_, i64>(2)?,
            "received": row.get::<_, i64>(3)?,
            "total": row.get::<_, i64>(4)?,
        }))
    }).map_err(|e| e.to_string())?.collect::<Result<Vec<_>,_>>().map_err(|e| e.to_string())?;
    
    let mut stmt2 = db.conn.prepare("
        SELECT e1.email_addr as src, e2.email_addr as tgt, COUNT(*) as w
        FROM emails em, entities e1, entities e2
        WHERE em.case_id=?1 AND e1.case_id=?1 AND e2.case_id=?1
          AND em.from_addr = e1.email_address
          AND em.to_addrs LIKE '%' || e2.email_address || '%'
        GROUP BY src, tgt ORDER BY w DESC LIMIT 200
    ").map_err(|e| e.to_string())?;
    
    let edges: Vec<serde_json::Value> = stmt2.query_map([&input.case_id], |row| {
        Ok(serde_json::json!({
            "source": row.get::<_, String>(0)?,
            "target": row.get::<_, String>(1)?,
            "weight": row.get::<_, i64>(2)?,
        }))
    }).map_err(|e| e.to_string())?.collect::<Result<Vec<_>,_>>().map_err(|e| e.to_string())?;
    
    Ok(serde_json::json!({ "nodes": nodes, "edges": edges }))
}
