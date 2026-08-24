use chrono::Utc;
use serde_json::Value;
use tauri::State;

use crate::AppState;
use crate::db::generate_id;
use crate::models::*;
use super::helpers::*;

#[tauri::command]
pub async fn email_list(state: State<'_, AppState>, input: EmailListInput) -> Result<Vec<EmailMessage>, String> {
    let db = state.db.lock().await;
    let limit = input.limit.unwrap_or(50) as i64;
    let offset = input.offset.unwrap_or(0) as i64;

    let mut conditions = vec!["case_id = ?1".to_string()];
    let mut params: Vec<Box<dyn rusqlite::ToSql>> = vec![Box::new(input.case_id.clone())];

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
    let mut stmt = db.conn.prepare("SELECT id,evidence_id,case_id,message_id,from_addr,from_display,to_addrs,cc_addrs,subject,date_sent,date_sent_utc,headers_raw,body_text,body_html,folder_name,folder_category,is_deleted,deleted_recovered,risk_score,flags FROM emails WHERE case_id=?1 AND (from_addr LIKE ?2 OR to_addrs LIKE ?2 OR subject LIKE ?2 OR body_text LIKE ?2) ORDER BY date_sent DESC LIMIT ?3").map_err(|e| e.to_string())?;
    let emails = stmt.query_map(rusqlite::params![&input.case_id, &q, limit], |row| {
        Ok(EmailMessage { id: row.get(0)?, evidence_id: row.get(1)?, case_id: row.get(2)?, message_id: row.get(3)?, from_addr: row.get(4)?, from_display: row.get(5)?, to_addrs: row.get(6)?, cc_addrs: row.get(7)?, subject: row.get(8)?, date_sent: row.get(9)?, date_sent_utc: row.get(10)?, headers_raw: row.get(11)?, body_text: row.get(12)?, body_html: row.get(13)?, folder_name: row.get(14)?, folder_category: row.get(15)?, is_deleted: boolv(row,16), deleted_recovered: boolv(row,17), risk_score: u8v(row,18), flags: row.get(19)? })
    }).map_err(|e| e.to_string())?.collect::<Result<Vec<_>,_>>().map_err(|e| e.to_string())?;
    Ok(emails)
}

#[tauri::command]
pub async fn advanced_search(state: State<'_, AppState>, input: SearchInput) -> Result<Vec<EmailMessage>, String> {
    search(state, input).await
}

#[tauri::command]
pub async fn email_headers(state: State<'_, AppState>, email_id: String) -> Result<Value, String> {
    let db = state.db.lock().await;
    let headers_raw: Option<String> = db.conn.query_row(
        "SELECT headers_raw FROM emails WHERE id=?1",
        [&email_id],
        |row| row.get(0),
    ).map_err(|e| e.to_string())?;

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
    let case_id = input["case_id"].as_str().or_else(|| input["caseId"].as_str()).unwrap_or("");
    let date = input["date"].as_str().unwrap_or("");
    let db = state.db.lock().await;

    let d_pattern = format!("{}%", date);
    let mut stmt = db.conn.prepare(
        "SELECT id,evidence_id,case_id,message_id,from_addr,from_display,to_addrs,cc_addrs,subject,date_sent,date_sent_utc,headers_raw,body_text,body_html,folder_name,folder_category,is_deleted,deleted_recovered,risk_score,flags
         FROM emails WHERE case_id=?1 AND (date_sent_utc LIKE ?2 OR date_sent LIKE ?2) ORDER BY date_sent_utc ASC"
    ).map_err(|e| e.to_string())?;

    let emails = stmt.query_map(rusqlite::params![case_id, d_pattern], |row| {
        Ok(EmailMessage {
            id: row.get(0)?, evidence_id: row.get(1)?, case_id: row.get(2)?, message_id: row.get(3)?,
            from_addr: row.get(4)?, from_display: row.get(5)?, to_addrs: row.get(6)?, cc_addrs: row.get(7)?,
            subject: row.get(8)?, date_sent: row.get(9)?, date_sent_utc: row.get(10)?, headers_raw: row.get(11)?,
            body_text: row.get(12)?, body_html: row.get(13)?, folder_name: row.get(14)?, folder_category: row.get(15)?,
            is_deleted: boolv(row,16), deleted_recovered: boolv(row,17), risk_score: u8v(row,18), flags: row.get(19)?
        })
    }).map_err(|e| e.to_string())?.collect::<Result<Vec<_>,_>>().map_err(|e| e.to_string())?;

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
        "SELECT id,evidence_id,case_id,message_id,from_addr,from_display,to_addrs,cc_addrs,subject,date_sent,date_sent_utc,headers_raw,body_text,body_html,folder_name,folder_category,is_deleted,deleted_recovered,risk_score,flags
         FROM emails 
         WHERE case_id=?1 AND (
           (from_addr LIKE ?2 AND (to_addrs LIKE ?3 OR cc_addrs LIKE ?3))
           OR
           (from_addr LIKE ?3 AND (to_addrs LIKE ?2 OR cc_addrs LIKE ?2))
         )
         ORDER BY date_sent_utc ASC"
    ).map_err(|e| e.to_string())?;

    let emails = stmt.query_map(rusqlite::params![case_id, e1_like, e2_like], |row| {
        Ok(EmailMessage {
            id: row.get(0)?, evidence_id: row.get(1)?, case_id: row.get(2)?, message_id: row.get(3)?,
            from_addr: row.get(4)?, from_display: row.get(5)?, to_addrs: row.get(6)?, cc_addrs: row.get(7)?,
            subject: row.get(8)?, date_sent: row.get(9)?, date_sent_utc: row.get(10)?, headers_raw: row.get(11)?,
            body_text: row.get(12)?, body_html: row.get(13)?, folder_name: row.get(14)?, folder_category: row.get(15)?,
            is_deleted: boolv(row,16), deleted_recovered: boolv(row,17), risk_score: u8v(row,18), flags: row.get(19)?
        })
    }).map_err(|e| e.to_string())?.collect::<Result<Vec<_>,_>>().map_err(|e| e.to_string())?;

    Ok(emails)
}

// Tags & Notes
#[tauri::command]
pub async fn email_tags_list(state: State<'_, AppState>, case_id: String) -> Result<Vec<EmailTag>, String> {
    let db = state.db.lock().await;
    let mut stmt = db.conn.prepare(
        "SELECT id, case_id, email_id, tag, color, created_at
         FROM email_tags
         WHERE email_id IN (SELECT id FROM emails WHERE case_id = ?1)
         ORDER BY created_at DESC"
    ).map_err(|e| e.to_string())?;

    let tags = stmt.query_map([&case_id], |row| {
        Ok(EmailTag {
            id: row.get(0)?,
            case_id: row.get(1)?,
            email_id: row.get(2)?,
            tag: row.get(3)?,
            color: row.get::<_, Option<String>>(4)?.unwrap_or_else(|| "#3b82f6".to_string()),
            created_by: "Examiner".to_string(),
            created_at: row.get::<_, String>(5)?,
        })
    }).map_err(|e| e.to_string())?.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())?;

    Ok(tags)
}

#[tauri::command]
pub async fn email_tag_add(state: State<'_, AppState>, input: EmailTagAddInput) -> Result<EmailTag, String> {
    let db = state.db.lock().await;
    let id = generate_id();
    let now = Utc::now().to_rfc3339();
    let color = input.color.unwrap_or_else(|| "#3b82f6".to_string());
    let created_by = input.created_by.unwrap_or_else(|| "Examiner".to_string());

    db.conn.execute(
        "INSERT INTO email_tags (id, case_id, email_id, tag, color, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        rusqlite::params![id, input.case_id, input.email_id, input.tag, color, now],
    ).map_err(|e| e.to_string())?;

    Ok(EmailTag {
        id,
        case_id: input.case_id,
        email_id: input.email_id,
        tag: input.tag,
        color,
        created_by,
        created_at: now,
    })
}

#[tauri::command]
pub async fn email_tag_remove(state: State<'_, AppState>, input: EmailTagRemoveInput) -> Result<(), String> {
    let db = state.db.lock().await;
    db.conn.execute(
        "DELETE FROM email_tags WHERE email_id = ?1 AND tag = ?2",
        rusqlite::params![input.email_id, input.tag],
    ).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn email_notes_list(state: State<'_, AppState>, email_id: String) -> Result<Vec<EmailNote>, String> {
    let db = state.db.lock().await;
    let mut stmt = db.conn.prepare(
        "SELECT id, case_id, email_id, author, note, created_at, updated_at
         FROM email_notes
         WHERE email_id = ?1
         ORDER BY created_at DESC"
    ).map_err(|e| e.to_string())?;

    let notes = stmt.query_map([&email_id], |row| {
        Ok(EmailNote {
            id: row.get(0)?,
            case_id: row.get::<_, Option<String>>(1)?.unwrap_or_default(),
            email_id: row.get(2)?,
            author: row.get(3)?,
            content: row.get(4)?,
            created_at: row.get::<_, String>(5)?,
            updated_at: row.get::<_, Option<String>>(6)?.unwrap_or_default(),
        })
    }).map_err(|e| e.to_string())?.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())?;

    Ok(notes)
}

#[tauri::command]
pub async fn email_note_add(state: State<'_, AppState>, input: EmailNoteInput) -> Result<EmailNote, String> {
    let db = state.db.lock().await;
    let id = generate_id();
    let now = Utc::now().to_rfc3339();
    let author = input.author.unwrap_or_else(|| "Examiner".to_string());

    db.conn.execute(
        "INSERT INTO email_notes (id, case_id, email_id, author, note, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?6)",
        rusqlite::params![id, input.case_id, input.email_id, author, input.content, now],
    ).map_err(|e| e.to_string())?;

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
