use serde_json::Value;
use tauri::State;

use crate::AppState;
use crate::models::*;
use super::super::helpers::*;

#[tauri::command]
pub async fn email_list(state: State<'_, AppState>, input: EmailListInput) -> Result<Vec<EmailMessage>, String> {
    let db = state.db.lock().await;
    let limit = input.limit.unwrap_or(50) as i64;
    let offset = input.offset.unwrap_or(0) as i64;

    let mut conditions = vec!["case_id = ?".to_string()];
    let mut params: Vec<Box<dyn rusqlite::ToSql>> = vec![Box::new(input.case_id.clone())];

    if let Some(ref ev_id) = input.evidence_id.as_ref().filter(|s| !s.is_empty() && *s != "all") {
        conditions.push("evidence_id = ?".to_string());
        params.push(Box::new((*ev_id).clone()));
    }

    if let Some(ref from_f) = input.from_filter {
        if from_f == "soft_deleted" {
            conditions.push("deleted_recovered = 1".to_string());
        } else if from_f == "hard_deleted" {
            conditions.push("is_deleted = 1 AND deleted_recovered = 0".to_string());
        } else if from_f == "recoverable" {
            conditions.push("(deleted_recovered = 1 OR is_deleted = 1)".to_string());
        } else if from_f == "all" {
            // No filter
        } else if from_f.contains('@') {
            conditions.push("from_addr LIKE ?".to_string());
            params.push(Box::new(format!("%{}%", from_f)));
        } else {
            conditions.push("folder_category = ?".to_string());
            params.push(Box::new(from_f.clone()));
        }
    }

    let sql = format!(
        "SELECT id,evidence_id,case_id,message_id,from_addr,from_display,to_addrs,cc_addrs,subject,date_sent,date_sent_utc,folder_name,folder_category,is_deleted,deleted_recovered,risk_score,flags 
         FROM emails WHERE {} ORDER BY date_sent_utc DESC LIMIT {} OFFSET {}",
        conditions.join(" AND "), limit, offset
    );

    let mut stmt = db.conn.prepare(&sql).map_err(|e| e.to_string())?;
    let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();

    let emails = stmt.query_map(param_refs.as_slice(), |row| {
        Ok(EmailMessage {
            id: row.get(0)?, evidence_id: row.get(1)?, case_id: row.get(2)?, message_id: row.get(3)?,
            from_addr: row.get(4)?, from_display: row.get(5)?, to_addrs: row.get(6)?, cc_addrs: row.get(7)?,
            subject: row.get(8)?, date_sent: row.get(9)?, date_sent_utc: row.get(10)?,
            headers_raw: None,
            body_text: None,
            body_html: None,
            folder_name: row.get(11)?, folder_category: row.get(12)?,
            is_deleted: boolv(row,13), deleted_recovered: boolv(row,14), risk_score: u8v(row,15), flags: row.get(16)?
        })
    }).map_err(|e| e.to_string())?.collect::<Result<Vec<_>,_>>().map_err(|e| e.to_string())?;

    Ok(emails)
}

#[tauri::command]
pub async fn email_get(state: State<'_, AppState>, input: Value) -> Result<Option<EmailMessage>, String> {
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
        |row| {
            let mut body_text: Option<String> = row.get(12)?;
            let mut body_html: Option<String> = row.get(13)?;
            let mut subject: Option<String> = row.get(8)?;

            if let Some(ref s) = subject {
                if s.contains("=?") {
                    subject = Some(crate::parser::mime::decode_mime_word(s));
                }
            }

            if let Some(ref text) = body_text {
                if text.contains("=3D") || text.contains("=21") || text.contains("=\r\n") || text.contains("=\n") {
                    body_text = Some(crate::parser::mime::qp_decode_str(text));
                }
            }

            if let Some(ref html) = body_html {
                if html.contains("=3D") || html.contains("=21") || html.contains("=\r\n") || html.contains("=\n") {
                    body_html = Some(crate::parser::mime::qp_decode_str(html));
                }
            }

            Ok(EmailMessage { 
                id: row.get(0)?, 
                evidence_id: row.get(1)?, 
                case_id: row.get(2)?, 
                message_id: row.get(3)?, 
                from_addr: row.get(4)?, 
                from_display: row.get(5)?, 
                to_addrs: row.get(6)?, 
                cc_addrs: row.get(7)?, 
                subject, 
                date_sent: row.get(9)?, 
                date_sent_utc: row.get(10)?, 
                headers_raw: row.get(11)?, 
                body_text, 
                body_html, 
                folder_name: row.get(14)?, 
                folder_category: row.get(15)?, 
                is_deleted: boolv(row,16), 
                deleted_recovered: boolv(row,17), 
                risk_score: u8v(row,18), 
                flags: row.get(19)? 
            })
        }
    );
    match r { 
        Ok(e) => {
            crate::audit_logger::log_forensic_event(
                &e.case_id,
                "EMAIL_INSPECTION",
                "EMAIL_RECORD_READ",
                "Examiner",
                Some(&e.evidence_id),
                None,
                &format!("Inspected email record [{}] Subject: \"{}\" From: <{}>", e.id, e.subject.as_deref().unwrap_or("(No Subject)"), e.from_addr)
            );
            Ok(Some(e))
        }, 
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None), 
        Err(e) => Err(e.to_string()) 
    }
}

#[tauri::command]
pub async fn search(state: State<'_, AppState>, input: SearchInput) -> Result<Vec<EmailMessage>, String> {
    let db = state.db.lock().await;
    let limit = input.limit.unwrap_or(100) as i64;
    let q = format!("%{}%", input.query.trim());
    
    let mut conditions = vec!["case_id = ?".to_string()];
    let mut params: Vec<Box<dyn rusqlite::ToSql>> = vec![Box::new(input.case_id.clone())];

    if let Some(ref ev_id) = input.evidence_id.as_ref().filter(|s| !s.is_empty() && *s != "all") {
        conditions.push("evidence_id = ?".to_string());
        params.push(Box::new((*ev_id).clone()));
    }

    if !input.query.trim().is_empty() {
        conditions.push("(from_addr LIKE ? OR to_addrs LIKE ? OR subject LIKE ? OR body_text LIKE ? OR body_html LIKE ?)".to_string());
        params.push(Box::new(q.clone()));
        params.push(Box::new(q.clone()));
        params.push(Box::new(q.clone()));
        params.push(Box::new(q.clone()));
        params.push(Box::new(q.clone()));
    }

    let sql = format!(
        "SELECT id,evidence_id,case_id,message_id,from_addr,from_display,to_addrs,cc_addrs,subject,date_sent,date_sent_utc,headers_raw,body_text,body_html,folder_name,folder_category,is_deleted,deleted_recovered,risk_score,flags 
         FROM emails WHERE {} ORDER BY date_sent DESC LIMIT {}",
        conditions.join(" AND "), limit
    );

    let mut stmt = db.conn.prepare(&sql).map_err(|e| e.to_string())?;
    let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();

    let emails = stmt.query_map(param_refs.as_slice(), |row| {
        Ok(EmailMessage { id: row.get(0)?, evidence_id: row.get(1)?, case_id: row.get(2)?, message_id: row.get(3)?, from_addr: row.get(4)?, from_display: row.get(5)?, to_addrs: row.get(6)?, cc_addrs: row.get(7)?, subject: row.get(8)?, date_sent: row.get(9)?, date_sent_utc: row.get(10)?, headers_raw: row.get(11)?, body_text: row.get(12)?, body_html: row.get(13)?, folder_name: row.get(14)?, folder_category: row.get(15)?, is_deleted: boolv(row,16), deleted_recovered: boolv(row,17), risk_score: u8v(row,18), flags: row.get(19)? })
    }).map_err(|e| e.to_string())?.collect::<Result<Vec<_>,_>>().map_err(|e| e.to_string())?;

    crate::audit_logger::log_forensic_event(
        &input.case_id,
        "SEARCH",
        "QUERY_EXECUTED",
        "Examiner",
        input.evidence_id.as_deref(),
        None,
        &format!("Executed keyword search: \"{}\" (Returned {} results)", input.query, emails.len())
    );

    Ok(emails)
}

#[tauri::command]
pub async fn advanced_search(state: State<'_, AppState>, input: SearchInput) -> Result<Vec<EmailMessage>, String> {
    search(state, input).await
}

#[tauri::command]
pub async fn email_headers(state: State<'_, AppState>, email_id: String) -> Result<Value, String> {
    let db = state.db.lock().await;
    let (case_id, headers_raw): (Option<String>, Option<String>) = db.conn.query_row(
        "SELECT case_id, headers_raw FROM emails WHERE id=?1",
        [&email_id],
        |row| Ok((row.get(0)?, row.get(1)?)),
    ).unwrap_or((None, None));

    if let Some(ref cid) = case_id {
        crate::audit_logger::log_forensic_event(
            cid,
            "EMAIL_INSPECTION",
            "HEADERS_INSPECTED",
            "Examiner",
            None,
            None,
            &format!("Parsed raw RFC headers for email ID: {}", email_id)
        );
    }

    let raw = headers_raw.unwrap_or_default();
    let mut parsed_headers = std::collections::HashMap::new();

    for line in raw.lines() {
        if let Some(idx) = line.find(':') {
            let key = line[..idx].trim().to_lowercase();
            let val = line[idx+1..].trim().to_string();
            parsed_headers.entry(key).or_insert_with(Vec::new).push(val);
        }
    }

    Ok(serde_json::json!({
        "email_id": email_id,
        "raw": raw,
        "parsed": parsed_headers,
    }))
}

#[tauri::command]
pub async fn emails_by_date(state: State<'_, AppState>, input: Value) -> Result<Vec<EmailMessage>, String> {
    let case_id = input["case_id"].as_str()
        .or_else(|| input["caseId"].as_str())
        .or_else(|| input["input"]["case_id"].as_str())
        .or_else(|| input["input"]["caseId"].as_str())
        .unwrap_or("");
    let date = input["date"].as_str()
        .or_else(|| input["input"]["date"].as_str())
        .unwrap_or("");
    let evidence_id = input["evidence_id"].as_str()
        .or_else(|| input["evidenceId"].as_str())
        .or_else(|| input["input"]["evidence_id"].as_str())
        .or_else(|| input["input"]["evidenceId"].as_str())
        .map(|s| s.trim())
        .filter(|s| !s.is_empty() && *s != "all");

    let db = state.db.lock().await;
    let d_pattern = format!("{}%", date);

    let emails = if let Some(ev_id) = evidence_id {
        let mut stmt = db.conn.prepare(
            "SELECT id,evidence_id,case_id,message_id,from_addr,from_display,to_addrs,cc_addrs,subject,date_sent,date_sent_utc,folder_name,folder_category,is_deleted,deleted_recovered,risk_score,flags
             FROM emails WHERE case_id=?1 AND evidence_id=?2 AND (date_sent_utc LIKE ?3 OR date_sent LIKE ?3) ORDER BY date_sent_utc ASC LIMIT 1000"
        ).map_err(|e| e.to_string())?;

        let res: Vec<EmailMessage> = stmt.query_map(rusqlite::params![case_id, ev_id, d_pattern], |row| {
            Ok(EmailMessage {
                id: row.get(0)?, evidence_id: row.get(1)?, case_id: row.get(2)?, message_id: row.get(3)?,
                from_addr: row.get(4)?, from_display: row.get(5)?, to_addrs: row.get(6)?, cc_addrs: row.get(7)?,
                subject: row.get(8)?, date_sent: row.get(9)?, date_sent_utc: row.get(10)?,
                headers_raw: None,
                body_text: None,
                body_html: None,
                folder_name: row.get(11)?, folder_category: row.get(12)?,
                is_deleted: boolv(row, 13), deleted_recovered: boolv(row, 14), risk_score: u8v(row, 15), flags: row.get(16)?
            })
        }).map_err(|e| e.to_string())?.filter_map(|r| r.ok()).collect();
        res
    } else {
        let mut stmt = db.conn.prepare(
            "SELECT id,evidence_id,case_id,message_id,from_addr,from_display,to_addrs,cc_addrs,subject,date_sent,date_sent_utc,folder_name,folder_category,is_deleted,deleted_recovered,risk_score,flags
             FROM emails WHERE case_id=?1 AND (date_sent_utc LIKE ?2 OR date_sent LIKE ?2) ORDER BY date_sent_utc ASC LIMIT 1000"
        ).map_err(|e| e.to_string())?;

        let res: Vec<EmailMessage> = stmt.query_map(rusqlite::params![case_id, d_pattern], |row| {
            Ok(EmailMessage {
                id: row.get(0)?, evidence_id: row.get(1)?, case_id: row.get(2)?, message_id: row.get(3)?,
                from_addr: row.get(4)?, from_display: row.get(5)?, to_addrs: row.get(6)?, cc_addrs: row.get(7)?,
                subject: row.get(8)?, date_sent: row.get(9)?, date_sent_utc: row.get(10)?,
                headers_raw: None,
                body_text: None,
                body_html: None,
                folder_name: row.get(11)?, folder_category: row.get(12)?,
                is_deleted: boolv(row, 13), deleted_recovered: boolv(row, 14), risk_score: u8v(row, 15), flags: row.get(16)?
            })
        }).map_err(|e| e.to_string())?.filter_map(|r| r.ok()).collect();
        res
    };

    Ok(emails)
}

#[tauri::command]
pub async fn emails_between(state: State<'_, AppState>, input: Value) -> Result<Vec<EmailMessage>, String> {
    let case_id = input["case_id"].as_str().or_else(|| input["caseId"].as_str()).unwrap_or("");
    let entity1 = input["entity1"].as_str().unwrap_or("");
    let entity2 = input["entity2"].as_str().unwrap_or("");
    let db = state.db.lock().await;

    let e1_like = format!("%{}%", entity1);
    let e2_like = format!("%{}%", entity2);

    let mut stmt = db.conn.prepare(
        "SELECT id,evidence_id,case_id,message_id,from_addr,from_display,to_addrs,cc_addrs,subject,date_sent,date_sent_utc,folder_name,folder_category,is_deleted,deleted_recovered,risk_score,flags
         FROM emails 
         WHERE case_id=?1 AND (
           (from_addr LIKE ?2 AND (to_addrs LIKE ?3 OR cc_addrs LIKE ?3))
           OR
           (from_addr LIKE ?3 AND (to_addrs LIKE ?2 OR cc_addrs LIKE ?2))
         )
         ORDER BY date_sent_utc ASC LIMIT 1000"
    ).map_err(|e| e.to_string())?;

    let emails = stmt.query_map(rusqlite::params![case_id, e1_like, e2_like], |row| {
        Ok(EmailMessage {
            id: row.get(0)?, evidence_id: row.get(1)?, case_id: row.get(2)?, message_id: row.get(3)?,
            from_addr: row.get(4)?, from_display: row.get(5)?, to_addrs: row.get(6)?, cc_addrs: row.get(7)?,
            subject: row.get(8)?, date_sent: row.get(9)?, date_sent_utc: row.get(10)?,
            headers_raw: None,
            body_text: None,
            body_html: None,
            folder_name: row.get(11)?, folder_category: row.get(12)?,
            is_deleted: boolv(row, 13), deleted_recovered: boolv(row, 14), risk_score: u8v(row, 15), flags: row.get(16)?
        })
    }).map_err(|e| e.to_string())?.collect::<Result<Vec<_>,_>>().map_err(|e| e.to_string())?;

    Ok(emails)
}
