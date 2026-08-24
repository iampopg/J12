use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri::State;

use crate::AppState;
use crate::models::*;

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

pub fn classify_attachment_category(filename: &str, mime: &str, entropy: Option<f64>, risk_flags: Option<&str>) -> String {
    let lower_name = filename.to_lowercase();
    let lower_mime = mime.to_lowercase();
    let has_risk = risk_flags.map(|r| !r.is_empty() && r != "[]").unwrap_or(false);
    let high_entropy = entropy.map(|e| e > 7.2).unwrap_or(false);

    if lower_name.ends_with(".exe") || lower_name.ends_with(".scr") || lower_name.ends_with(".bat") 
       || lower_name.ends_with(".vbs") || lower_name.ends_with(".js") || lower_name.ends_with(".ps1")
       || lower_name.ends_with(".hta") || lower_name.ends_with(".iso") || lower_name.ends_with(".img")
       || has_risk {
        return "dangerous".to_string();
    }
    if lower_name.ends_with(".jpg") || lower_name.ends_with(".jpeg") || lower_name.ends_with(".png")
       || lower_name.ends_with(".gif") || lower_name.ends_with(".bmp") || lower_name.ends_with(".webp")
       || lower_mime.starts_with("image/") {
        return "images".to_string();
    }
    if lower_name.ends_with(".pdf") || lower_name.ends_with(".doc") || lower_name.ends_with(".docx")
       || lower_name.ends_with(".xls") || lower_name.ends_with(".xlsx") || lower_name.ends_with(".ppt")
       || lower_name.ends_with(".pptx") || lower_name.ends_with(".txt") || lower_name.ends_with(".csv")
       || lower_mime.contains("pdf") || lower_mime.contains("officedocument") || lower_mime.contains("msword") {
        return "documents".to_string();
    }
    if lower_name.ends_with(".zip") || lower_name.ends_with(".rar") || lower_name.ends_with(".7z")
       || lower_name.ends_with(".tar") || lower_name.ends_with(".gz") || high_entropy {
        return "archives".to_string();
    }
    if lower_name.ends_with(".mp3") || lower_name.ends_with(".wav") || lower_name.ends_with(".mp4")
       || lower_name.ends_with(".mov") || lower_name.ends_with(".m4a") || lower_mime.starts_with("audio/")
       || lower_mime.starts_with("video/") {
        return "media".to_string();
    }
    "other".to_string()
}

#[tauri::command]
pub async fn email_attachments(state: State<'_, AppState>, input: Value) -> Result<Vec<Attachment>, String> {
    let email_id = input["email_id"].as_str()
        .or_else(|| input["emailId"].as_str())
        .or_else(|| input.as_str())
        .unwrap_or("")
        .to_string();

    let db = state.db.lock().await;
    let mut stmt = db.conn.prepare(
        "SELECT id, email_id, filename, mime_type, size_bytes, sha256, md5, entropy, is_inline, is_macro_enabled, is_executable, risk_flags 
         FROM attachments WHERE email_id = ?1"
    ).map_err(|e| e.to_string())?;

    let attachments = stmt.query_map([&email_id], |row| {
        let risk_flags_str: String = row.get::<_, Option<String>>(11)?.unwrap_or_default();

        Ok(Attachment {
            id: row.get(0)?,
            email_id: row.get(1)?,
            filename: row.get(2)?,
            sha256: row.get(5)?,
            mime_type: row.get(3)?,
            size_bytes: row.get::<_, i64>(4)? as u64,
            stored_path: String::new(),
            entropy: row.get(7)?,
            risk_flags: risk_flags_str,
        })
    }).map_err(|e| e.to_string())?.filter_map(|r| r.ok()).collect::<Vec<_>>();

    Ok(attachments)
}

#[tauri::command]
pub async fn case_attachments_list(state: State<'_, AppState>, input: Value) -> Result<Vec<CaseAttachmentItem>, String> {
    let case_id = input["case_id"].as_str()
        .or_else(|| input["caseId"].as_str())
        .or_else(|| input.as_str())
        .unwrap_or("")
        .to_string();

    let db = state.db.lock().await;
    let mut stmt = db.conn.prepare(
        "SELECT a.id, a.email_id, a.filename, a.sha256, a.mime_type, a.size_bytes, 
                a.stored_path, a.entropy, a.risk_flags,
                e.subject, e.from_addr, e.date_sent_utc, e.risk_score
         FROM attachments a
         JOIN emails e ON a.email_id = e.id
         WHERE e.case_id = ?1
         ORDER BY a.size_bytes DESC"
    ).map_err(|e| e.to_string())?;

    let items = stmt.query_map([&case_id], |row| {
        let filename: String = row.get::<_, Option<String>>(2)?.unwrap_or_else(|| "attachment".to_string());
        let mime: String = row.get::<_, Option<String>>(4)?.unwrap_or_else(|| "application/octet-stream".to_string());
        let entropy: Option<f64> = row.get(7)?;
        let risk_flags: Option<String> = row.get(8)?;

        let category = classify_attachment_category(&filename, &mime, entropy, risk_flags.as_deref());

        Ok(CaseAttachmentItem {
            id: row.get(0)?,
            email_id: row.get(1)?,
            filename,
            sha256: row.get(3)?,
            mime_type: mime,
            size_bytes: row.get::<_, i64>(5)? as u64,
            stored_path: row.get(6)?,
            entropy,
            risk_flags,
            email_subject: row.get(9)?,
            email_from: row.get(10)?,
            email_date: row.get(11)?,
            email_risk_score: row.get::<_, Option<i64>>(12)?.unwrap_or(0) as u8,
            category,
        })
    }).map_err(|e| e.to_string())?.filter_map(|r| r.ok()).collect::<Vec<_>>();

    Ok(items)
}

#[tauri::command]
pub async fn export_attachment(
    state: State<'_, AppState>,
    input: Value,
) -> Result<String, String> {
    let attachment_id = input["attachment_id"].as_str()
        .or_else(|| input["attachmentId"].as_str())
        .or_else(|| input.as_str())
        .unwrap_or("");

    let dest_dir = input["destination_dir"].as_str()
        .or_else(|| input["dest"].as_str())
        .or_else(|| input["destination"].as_str())
        .unwrap_or("");

    let db = state.db.lock().await;

    let (filename, stored_path): (String, Option<String>) = db.conn.query_row(
        "SELECT filename, stored_path FROM attachments WHERE id=?1",
        [attachment_id],
        |r| Ok((r.get::<_, Option<String>>(0)?.unwrap_or_else(|| "exported_file".to_string()), r.get(1)?)),
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
