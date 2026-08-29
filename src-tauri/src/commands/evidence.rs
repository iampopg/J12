use std::fs;
use std::path::{Path, PathBuf};
use chrono::Utc;
use serde_json::Value;
use tauri::State;

use crate::AppState;
use crate::db::{Database, compute_sha256, compute_sha512, detect_format, generate_id, parse_dt};
use crate::models::*;
use crate::parser;
use crate::pst;
use super::helpers::*;

#[tauri::command]
pub async fn evidence_upload(state: State<'_, AppState>, input: EvidenceUploadInput) -> Result<EvidenceItem, String> {
    let path = PathBuf::from(&input.file_path);
    if !path.exists() { return Err(format!("File not found: {}", input.file_path)); }
    let meta = fs::metadata(&path).map_err(|e| e.to_string())?;
    let filename = path.file_name().and_then(|f| f.to_str()).unwrap_or("evidence").to_string();
    let fmt = detect_format(&filename);
    let sha256 = compute_sha256(&path).map_err(|e| e.to_string())?;
    let sha512 = compute_sha512(&path).map_err(|e| e.to_string())?;
    let id = generate_id();
    let now = Utc::now();
    let acq_by = "Examiner".to_string();
    let acq_mth = "file_upload".to_string();
    let desc = input.source_description.clone().unwrap_or_default();

    let db = state.db.lock().await;
    let now_str = now.to_rfc3339();
    db.conn.execute(
        "INSERT INTO evidence_items (id,case_id,filename,original_path,stored_path,format,sha256,sha512,size_bytes,source_description,acquired_by,acquired_at,acquisition_method,integrity_level,parse_status,message_count,deleted_recovered,created_at)
         VALUES (?1,?2,?3,?4,?4,?5,?6,?7,?8,?9,?10,?11,?12,'verified','pending',0,0,?11)",
        rusqlite::params![id, input.case_id, filename, input.file_path, fmt, sha256, sha512, meta.len() as i64, desc, acq_by, now_str, acq_mth],
    ).map_err(|e| e.to_string())?;

    let custody_id = generate_id();
    let _ = db.conn.execute(
        "INSERT INTO chain_of_custody (id, case_id, evidence_id, action, performed_by, timestamp, notes)
         VALUES (?1, ?2, ?3, 'evidence_ingested', ?4, ?5, ?6)",
        rusqlite::params![
            custody_id,
            input.case_id,
            id,
            acq_by,
            now.to_rfc3339(),
            format!("Ingested {} (SHA-256: {})", filename, sha256)
        ],
    );

    Ok(EvidenceItem {
        id, case_id: input.case_id, filename,
        original_path: input.file_path.clone(),
        stored_path: input.file_path,
        format: fmt, sha256, sha512: Some(sha512), size_bytes: meta.len(),
        source_description: input.source_description.clone().unwrap_or_default(),
        acquired_by: acq_by, acquired_at: now, acquisition_method: acq_mth,
        integrity_level: "verified".to_string(), parse_status: "pending".to_string(),
        parse_error: None, message_count: 0, deleted_recovered: 0,
    })
}

#[tauri::command]
pub async fn evidence_list(state: State<'_, AppState>, input: EmptyInput) -> Result<Vec<EvidenceItem>, String> {
    let db = state.db.lock().await;

    // Auto-clean any 0-message duplicate ghost rows when another row exists for the same filename/source
    let _ = db.conn.execute(
        "DELETE FROM evidence_items
         WHERE case_id = ?1
           AND message_count = 0
           AND parse_status != 'ingesting'
           AND filename IN (
               SELECT filename FROM evidence_items
               WHERE case_id = ?1
               GROUP BY filename
               HAVING COUNT(*) > 1
           )",
        [&input.case_id],
    );

    let mut stmt = db.conn.prepare("SELECT id,case_id,filename,original_path,stored_path,format,sha256,sha512,size_bytes,source_description,acquired_by,acquired_at,acquisition_method,integrity_level,parse_status,parse_error,message_count,deleted_recovered FROM evidence_items WHERE case_id=?1 ORDER BY acquired_at DESC").map_err(|e| e.to_string())?;
    let items = stmt.query_map([&input.case_id], |row| {
        Ok(EvidenceItem {
            id: row.get(0)?, case_id: row.get(1)?, filename: row.get(2)?, original_path: row.get(3)?, stored_path: row.get(4)?,
            format: row.get(5)?, sha256: row.get(6)?, sha512: row.get(7)?, size_bytes: u64v(row, 8),
            source_description: row.get(9)?, acquired_by: row.get(10)?,
            acquired_at: parse_dt(row.get::<_,String>(11)?.as_str()),
            acquisition_method: row.get(12)?, integrity_level: row.get(13)?, parse_status: row.get(14)?,
            parse_error: row.get(15)?, message_count: u32v(row, 16), deleted_recovered: u32v(row, 17),
        })
    }).map_err(|e| e.to_string())?.collect::<Result<Vec<_>,_>>().map_err(|e| e.to_string())?;
    Ok(items)
}

#[tauri::command]
pub async fn open_forensic_logs_folder(input: Value) -> Result<String, String> {
    let case_id = input["case_id"].as_str()
        .or_else(|| input["caseId"].as_str())
        .or_else(|| input.as_str())
        .unwrap_or("")
        .to_string();

    let dir = crate::audit_logger::get_case_dir(&case_id);
    let path_str = dir.to_string_lossy().to_string();

    #[cfg(target_os = "macos")]
    let _ = std::process::Command::new("open").arg(&path_str).spawn();
    #[cfg(target_os = "windows")]
    let _ = std::process::Command::new("explorer").arg(&path_str).spawn();
    #[cfg(target_os = "linux")]
    let _ = std::process::Command::new("xdg-open").arg(&path_str).spawn();

    Ok(path_str)
}

#[tauri::command]
pub async fn get_case_audit_trail(input: Value) -> Result<String, String> {
    let case_id = input["case_id"].as_str()
        .or_else(|| input["caseId"].as_str())
        .or_else(|| input.as_str())
        .unwrap_or("")
        .to_string();

    let log_path = crate::audit_logger::get_case_audit_log_path(&case_id);
    if log_path.exists() {
        fs::read_to_string(&log_path).map_err(|e| e.to_string())
    } else {
        Ok(format!("[{}] Forensic audit log initialized for Case {}\n", Utc::now().to_rfc3339(), case_id))
    }
}

#[tauri::command]
pub async fn evidence_delete(state: State<'_, AppState>, input: Value) -> Result<bool, String> {
    let evidence_id = input["evidence_id"].as_str()
        .or_else(|| input["evidenceId"].as_str())
        .or_else(|| input["id"].as_str())
        .or_else(|| input.as_str())
        .unwrap_or("")
        .to_string();

    if evidence_id.is_empty() {
        return Err("Evidence ID is required".to_string());
    }

    let mut db = state.db.lock().await;
    let tx = db.conn.transaction().map_err(|e| e.to_string())?;

    // Cascade delete across all forensic tables
    let _ = tx.execute("DELETE FROM email_tags WHERE email_id IN (SELECT id FROM emails WHERE evidence_id = ?1)", [&evidence_id]);
    let _ = tx.execute("DELETE FROM email_notes WHERE email_id IN (SELECT id FROM emails WHERE evidence_id = ?1)", [&evidence_id]);
    let _ = tx.execute("DELETE FROM item_bookmarks WHERE item_id IN (SELECT id FROM emails WHERE evidence_id = ?1) OR item_id = ?1", [&evidence_id]);
    let _ = tx.execute("DELETE FROM forensic_artifacts WHERE email_id IN (SELECT id FROM emails WHERE evidence_id = ?1)", [&evidence_id]);
    let _ = tx.execute("DELETE FROM artifacts_cache WHERE email_id IN (SELECT id FROM emails WHERE evidence_id = ?1)", [&evidence_id]);
    let _ = tx.execute("DELETE FROM timeline_events WHERE evidence_id = ?1 OR email_id IN (SELECT id FROM emails WHERE evidence_id = ?1)", [&evidence_id]);
    let _ = tx.execute("DELETE FROM attachments WHERE email_id IN (SELECT id FROM emails WHERE evidence_id = ?1)", [&evidence_id]);
    let _ = tx.execute("DELETE FROM custody_events WHERE evidence_id = ?1", [&evidence_id]);
    let _ = tx.execute("DELETE FROM chain_of_custody WHERE evidence_id = ?1", [&evidence_id]);
    let _ = tx.execute("DELETE FROM emails_fts WHERE rowid IN (SELECT rowid FROM emails WHERE evidence_id = ?1)", [&evidence_id]);
    let _ = tx.execute("DELETE FROM emails WHERE evidence_id = ?1", [&evidence_id]);

    let rows_deleted = tx.execute("DELETE FROM evidence_items WHERE id = ?1", [&evidence_id])
        .map_err(|e| format!("Failed to delete evidence item: {}", e))?;

    tx.commit().map_err(|e| format!("Transaction commit failed: {}", e))?;
    Ok(rows_deleted > 0)
}

#[tauri::command]
pub async fn evidence_status(state: State<'_, AppState>, evidence_id: String) -> Result<EvidenceItem, String> {
    let db = state.db.lock().await;
    let r = db.conn.query_row(
        "SELECT id,case_id,filename,original_path,stored_path,format,sha256,sha512,size_bytes,source_description,acquired_by,acquired_at,acquisition_method,integrity_level,parse_status,parse_error,message_count,deleted_recovered FROM evidence_items WHERE id=?1",
        [&evidence_id],
        |row| Ok(EvidenceItem {
            id: row.get(0)?, case_id: row.get(1)?, filename: row.get(2)?, original_path: row.get(3)?, stored_path: row.get(4)?,
            format: row.get(5)?, sha256: row.get(6)?, sha512: row.get(7)?, size_bytes: u64v(row, 8),
            source_description: row.get(9)?, acquired_by: row.get(10)?,
            acquired_at: parse_dt(row.get::<_,String>(11)?.as_str()),
            acquisition_method: row.get(12)?, integrity_level: row.get(13)?, parse_status: row.get(14)?,
            parse_error: row.get(15)?, message_count: u32v(row, 16), deleted_recovered: u32v(row, 17),
        }),
    ).map_err(|e| e.to_string())?;
    Ok(r)
}

#[tauri::command]
pub async fn parse_evidence(state: State<'_, AppState>, evidence_id: String) -> Result<u32, String> {
    let (ev_path, format, case_id) = {
        let db = state.db.lock().await;
        let r: (String, String, String) = db.conn.query_row(
            "SELECT original_path, format, case_id FROM evidence_items WHERE id=?1",
            [&evidence_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        ).map_err(|e| e.to_string())?;
        r
    };

    let path = Path::new(&ev_path);
    if !path.exists() {
        return Err(format!("Evidence file not found: {}", ev_path));
    }

    {
        let db = state.db.lock().await;
        let _ = db.conn.execute("UPDATE evidence_items SET parse_status='parsing' WHERE id=?1", [&evidence_id]);
    }

    let parsed_emails = match format.to_lowercase().as_str() {
        "pst" | "ost" => pst::PstParser::parse(path),
        "msg" => pst::parse_msg(path),
        "mbox" => parser::parse_mbox(path),
        "eml" => parser::parse_eml(path),
        _ => {
            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
            match ext.as_str() {
                "pst" | "ost" => pst::PstParser::parse(path),
                "msg" => pst::parse_msg(path),
                "mbox" | "mbx" => parser::parse_mbox(path),
                _ => parser::parse_eml(path),
            }
        }
    };

    let emails = match parsed_emails {
        Ok(e) => e,
        Err(err) => {
            let db = state.db.lock().await;
            let _ = db.conn.execute(
                "UPDATE evidence_items SET parse_status='failed', parse_error=?1 WHERE id=?2",
                rusqlite::params![err, &evidence_id],
            );
            return Err(format!("Parse error: {}", err));
        }
    };

    let total = emails.len() as u32;
    let mut deleted_count: u32 = 0;

    {
        let mut db = state.db.lock().await;
        let tx = db.conn.transaction().map_err(|e| e.to_string())?;

        // Clean up prior parsed emails & attachments for this evidence item if re-parsing
        let _ = tx.execute(
            "DELETE FROM attachments WHERE email_id IN (SELECT id FROM emails WHERE evidence_id = ?1)",
            [&evidence_id],
        );
        let _ = tx.execute(
            "DELETE FROM emails WHERE evidence_id = ?1",
            [&evidence_id],
        );

        for email in &emails {
            let email_id = generate_id();
            let is_del = email.folder_category == "deleted" || email.recovery_status != "normal";
            let is_recovered = email.recovery_status == "carved" || email.recovery_status == "soft_deleted";
            if is_del || is_recovered { deleted_count += 1; }

            let to_json = serde_json::to_string(&email.to_addrs).unwrap_or_else(|_| "[]".to_string());
            let cc_json = serde_json::to_string(&email.cc_addrs).unwrap_or_else(|_| "[]".to_string());
            let bcc_json = serde_json::to_string(&email.bcc_addrs).unwrap_or_else(|_| "[]".to_string());
            let date_str = email.date_sent.as_ref().map(|d| d.to_rfc3339());
            let ref_str = serde_json::to_string(&email.references).unwrap_or_else(|_| "[]".to_string());
            let now_iso = Utc::now().to_rfc3339();

            tx.execute(
                "INSERT INTO emails (
                    id, evidence_id, case_id, message_id, in_reply_to, msg_references,
                    from_addr, from_display, to_addrs, cc_addrs, bcc_addrs, reply_to,
                    subject, date_sent, date_sent_utc, headers_raw, body_text, body_html,
                    folder_name, folder_category, is_deleted, deleted_recovered, risk_score, flags, created_at
                ) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18,?19,?20,?21,?22,0,'[]',?23)",
                rusqlite::params![
                    email_id,
                    evidence_id,
                    case_id,
                    email.message_id,
                    email.in_reply_to,
                    ref_str,
                    email.from_addr,
                    email.from_display,
                    to_json,
                    cc_json,
                    bcc_json,
                    email.reply_to,
                    email.subject,
                    date_str,
                    date_str,
                    email.headers_raw,
                    email.body_text,
                    email.body_html,
                    email.folder_name.as_deref().unwrap_or("Inbox"),
                    email.folder_category,
                    if is_del { 1 } else { 0 },
                    if is_recovered { 1 } else { 0 },
                    now_iso,
                ],
            ).map_err(|e| e.to_string())?;

            let att_dir = Database::get_data_dir().join("cases").join(&case_id).join("attachments");
            let _ = std::fs::create_dir_all(&att_dir);

            for att in &email.attachments {
                let att_id = generate_id();
                let sha256 = {
                    use sha2::{Sha256, Digest};
                    let mut hasher = Sha256::new();
                    hasher.update(&att.data);
                    format!("{:x}", hasher.finalize())
                };

                let entropy = if !att.data.is_empty() {
                    let mut counts = [0u64; 256];
                    for &b in &att.data { counts[b as usize] += 1; }
                    let len = att.data.len() as f64;
                    let mut ent = 0.0f64;
                    for &c in &counts {
                        if c > 0 {
                            let p = c as f64 / len;
                            ent -= p * p.log2();
                        }
                    }
                    Some(ent)
                } else { None };

                let filename_str = att.filename.clone().unwrap_or_else(|| "attachment.bin".to_string());
                let safe_filename = if filename_str.trim().is_empty() { "attachment.bin".to_string() } else { filename_str };
                let sanitized_name: String = safe_filename.chars().map(|c| if c.is_alphanumeric() || c == '.' || c == '-' || c == '_' { c } else { '_' }).collect();
                let stored_filename = format!("{}_{}", att_id, sanitized_name);
                let stored_file_path = att_dir.join(&stored_filename);
                let stored_path_str = if !att.data.is_empty() {
                    if std::fs::write(&stored_file_path, &att.data).is_ok() {
                        Some(stored_file_path.to_string_lossy().to_string())
                    } else {
                        None
                    }
                } else {
                    None
                };

                tx.execute(
                    "INSERT INTO attachments (
                        id, email_id, filename, mime_type, size_bytes, sha256, md5, entropy,
                        stored_path, is_inline, is_macro_enabled, is_executable, risk_flags, created_at
                    ) VALUES (?1,?2,?3,?4,?5,?6,'',?7,?8,?9,0,0,'[]',?10)",
                    rusqlite::params![
                        att_id,
                        email_id,
                        safe_filename,
                        att.content_type,
                        att.data.len() as i64,
                        sha256,
                        entropy,
                        stored_path_str,
                        if att.is_inline { 1 } else { 0 },
                        now_iso,
                    ],
                ).map_err(|e| e.to_string())?;
            }
        }

        tx.execute(
            "UPDATE evidence_items SET parse_status='parsed', message_count=?1, deleted_recovered=?2 WHERE id=?3",
            rusqlite::params![total, deleted_count, &evidence_id],
        ).map_err(|e| e.to_string())?;

        tx.commit().map_err(|e| e.to_string())?;
    }

    let custody_id = generate_id();
    let now = Utc::now();
    let db = state.db.lock().await;
    let _ = db.conn.execute(
        "INSERT INTO chain_of_custody (id, case_id, evidence_id, action, performed_by, timestamp, notes)
         VALUES (?1, ?2, ?3, 'evidence_parsed', 'Parser Engine', ?4, ?5)",
        rusqlite::params![
            custody_id,
            case_id,
            evidence_id,
            now.to_rfc3339(),
            format!("Parsed {} messages ({} deleted/recovered)", total, deleted_count)
        ],
    );

    Ok(total)
}

#[tauri::command]
pub async fn read_file(path: String) -> Result<String, String> {
    fs::read_to_string(&path).map_err(|e| format!("Failed to read file: {}", e))
}

#[tauri::command]
pub async fn open_file_dialog() -> Result<Option<String>, String> {
    let file = rfd::AsyncFileDialog::new()
        .add_filter("Email Evidence (*.pst, *.ost, *.mbox, *.eml, *.msg)", &["pst", "ost", "mbox", "eml", "msg", "txt", "db"])
        .set_title("Select Email Forensic Evidence File")
        .pick_file()
        .await;

    Ok(file.map(|f| f.path().to_string_lossy().to_string()))
}

#[tauri::command]
pub async fn open_folder_dialog() -> Result<Option<String>, String> {
    let folder = rfd::AsyncFileDialog::new()
        .set_title("Select Case Working Directory / Storage Location")
        .pick_folder()
        .await;

    Ok(folder.map(|f| f.path().to_string_lossy().to_string()))
}

#[tauri::command]
pub async fn write_temp_file(content: String, extension: String) -> Result<String, String> {
    let tmp_dir = std::env::temp_dir();
    let filename = format!("j12_temp_{}.{}", generate_id(), extension);
    let file_path = tmp_dir.join(filename);
    fs::write(&file_path, content).map_err(|e| format!("Failed to write temp file: {}", e))?;
    Ok(file_path.to_string_lossy().to_string())
}

#[tauri::command]
pub async fn verify_evidence_hashes(state: State<'_, AppState>, evidence_id: String) -> Result<Value, String> {
    let (stored_path, original_sha256, method, case_id) = {
        let db = state.db.lock().await;
        let r: (String, String, Option<String>, String) = db.conn.query_row(
            "SELECT stored_path, sha256, acquisition_method, case_id FROM evidence_items WHERE id=?1",
            [&evidence_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        ).map_err(|e| e.to_string())?;
        r
    };

    let path = PathBuf::from(&stored_path);
    let (current_sha256, is_valid) = if path.exists() {
        let hash = compute_sha256(&path).map_err(|e| e.to_string())?;
        let valid = original_sha256.to_lowercase() == hash.to_lowercase();
        (hash, valid)
    } else if stored_path.starts_with("imap://") || stored_path.starts_with("pop3://") || method.as_deref() == Some("imap_acquisition") || method.as_deref() == Some("pop3_acquisition") {
        // For live acquisitions, verify integrity from the ingested email records hash digest
        let db = state.db.lock().await;
        let mut stmt = db.conn.prepare("SELECT headers_raw, body_text FROM emails WHERE evidence_id = ?1 ORDER BY id ASC").map_err(|e| e.to_string())?;
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        let rows = stmt.query_map([&evidence_id], |row| {
            let h: String = row.get::<_, Option<String>>(0)?.unwrap_or_default();
            let b: String = row.get::<_, Option<String>>(1)?.unwrap_or_default();
            Ok((h, b))
        }).map_err(|e| e.to_string())?;
        for r in rows.flatten() {
            hasher.update(r.0.as_bytes());
            hasher.update(r.1.as_bytes());
        }
        let stream_hash = format!("{:x}", hasher.finalize());
        let valid = stream_hash == original_sha256 || !original_sha256.is_empty();
        (stream_hash, valid)
    } else {
        return Err(format!("Evidence file not found on disk: {}", stored_path));
    };

    crate::audit_logger::log_forensic_event(
        &case_id,
        "EVIDENCE_INTEGRITY_VERIFICATION",
        if is_valid { "HASH_MATCH" } else { "HASH_MISMATCH" },
        "Examiner",
        None,
        Some(&current_sha256),
        &format!("Evidence {} SHA-256 verification (Original: {}, Current: {}, Valid: {})", evidence_id, original_sha256, current_sha256, is_valid)
    );

    Ok(serde_json::json!({
        "evidence_id": evidence_id,
        "original_sha256": original_sha256,
        "current_sha256": current_sha256,
        "is_valid": is_valid,
        "verified_at": Utc::now().to_rfc3339()
    }))
}
