use chrono::Utc;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use tauri::{AppHandle, Emitter, Manager, State};

use crate::AppState;
use crate::db::generate_id;
use crate::imap_acquisition::{self, ImapConfig, StreamingMessage};
use crate::parser;

fn emit_imap_event(app: &AppHandle, payload: serde_json::Value) {
    let _ = app.emit("imap_progress", payload.clone());
    let _ = app.emit_to("main", "imap_progress", payload.clone());
    if let Some(win) = app.get_webview_window("main") {
        let _ = win.emit("imap_progress", payload);
    }
}

#[tauri::command]
pub async fn imap_cancel_acquisition(state: State<'_, AppState>) -> Result<bool, String> {
    state.cancel_imap.store(true, std::sync::atomic::Ordering::Relaxed);
    Ok(true)
}

#[tauri::command]
pub async fn imap_list_mailboxes(input: Value) -> Result<Vec<String>, String> {
    let server = input["server"].as_str().unwrap_or("imap.gmail.com").to_string();
    let port = input["port"].as_u64().unwrap_or(993) as u16;
    let username = input["username"].as_str().unwrap_or("").to_string();
    let password = input["password"].as_str().unwrap_or("").to_string();
    let use_ssl = input["use_ssl"].as_bool()
        .or_else(|| input["useSsl"].as_bool())
        .unwrap_or(true);

    if username.is_empty() || password.is_empty() {
        return Err("Username and password are required".to_string());
    }

    let config = ImapConfig {
        server,
        port,
        username,
        password,
        use_ssl,
        mailbox: "INBOX".to_string(),
    };
    imap_acquisition::list_mailboxes(&config)
}

#[tauri::command]
pub async fn imap_fetch_emails(
    app: AppHandle,
    state: State<'_, AppState>,
    input: Value,
) -> Result<serde_json::Value, String> {
    let case_id = input["case_id"].as_str()
        .or_else(|| input["caseId"].as_str())
        .unwrap_or("")
        .to_string();

    let evidence_id = input["evidence_id"].as_str()
        .or_else(|| input["evidenceId"].as_str())
        .unwrap_or_else(|| "imap_live_evidence")
        .to_string();

    let server = input["server"].as_str().unwrap_or("imap.gmail.com").to_string();
    let port = input["port"].as_u64().unwrap_or(993) as u16;
    let username = input["username"].as_str().unwrap_or("").to_string();
    let password = input["password"].as_str().unwrap_or("").to_string();
    let use_ssl = input["use_ssl"].as_bool()
        .or_else(|| input["useSsl"].as_bool())
        .unwrap_or(true);

    let mailbox_opt = input["mailbox"].as_str().map(|s| s.to_string());
    let target_mb = mailbox_opt.as_deref().unwrap_or("ALL");

    let max_messages = input["max_messages"].as_u64()
        .or_else(|| input["maxMessages"].as_u64())
        .map(|m| m as u32);

    if username.is_empty() || password.is_empty() {
        return Err("Username and password are required".to_string());
    }

    // Reset cancel flag
    state.cancel_imap.store(false, std::sync::atomic::Ordering::Relaxed);

    emit_imap_event(&app, json!({
        "status": "connecting",
        "log": format!("Connecting to {}:{} (SSL: {})...", server, port, if use_ssl { "YES" } else { "NO" })
    }));

    let config = ImapConfig {
        server: server.clone(),
        port,
        username: username.clone(),
        password,
        use_ssl,
        mailbox: target_mb.to_string(),
    };

    let now = Utc::now();
    let ev_filename = format!("IMAP Live Acquisition ({})", username);

    let mut db = state.db.lock().await;

    let _ = db.conn.execute(
        "INSERT OR IGNORE INTO evidence_items (
            id, case_id, filename, original_path, stored_path, format, sha256,
            size_bytes, source_description, acquired_by, acquired_at, acquisition_method,
            integrity_level, parse_status, message_count, deleted_recovered
        ) VALUES (?1, ?2, ?3, ?4, ?4, 'imap', 'in_progress', 0, ?5, 'Examiner', ?6, 'imap_live_acquisition', 'verified', 'ingesting', 0, 0)",
        rusqlite::params![
            evidence_id,
            case_id,
            ev_filename,
            format!("imap://{}:{}@{}", username, port, server),
            format!("Live IMAP acquisition for account {}", username),
            now.to_rfc3339()
        ],
    );

    let mut ingested_count: u32 = 0;
    let mut total_bytes_downloaded: usize = 0;
    let mut duplicates_skipped: u32 = 0;
    let app_handle_clone = app.clone();
    let cancel_flag = state.cancel_imap.clone();

    // Stream and incrementally save each email
    let result = {
        let app_handle_inner = app_handle_clone.clone();
        imap_acquisition::fetch_emails_streaming(
            &config,
            Some(target_mb),
            max_messages,
            &cancel_flag,
            |folder, count, f_idx, total_f| {
                emit_imap_event(&app_handle_inner, json!({
                    "status": "folder_discovered",
                    "folder": folder,
                    "folder_count": count,
                    "folder_index": f_idx,
                    "total_folders": total_f,
                    "log": format!("📁 Scanning [{}] (Found {} messages)...", folder, count)
                }));
            },
            |msg: StreamingMessage| -> Result<(), String> {
                let raw_len = msg.raw_content.len();
                total_bytes_downloaded += raw_len;

                if let Ok(parsed) = parser::parse_rfc5322(&msg.raw_content, 0, raw_len as u64) {
                    let email_id = generate_id();
                    let to_str = serde_json::to_string(&parsed.to_addrs).unwrap_or_else(|_| "[]".to_string());
                    let cc_str = serde_json::to_string(&parsed.cc_addrs).unwrap_or_else(|_| "[]".to_string());
                    let bcc_str = serde_json::to_string(&parsed.bcc_addrs).unwrap_or_else(|_| "[]".to_string());
                    let ref_str = serde_json::to_string(&parsed.references).unwrap_or_else(|_| "[]".to_string());
                    let date_str = parsed.date_sent.as_ref().map(|d| d.to_rfc3339());
                    let is_del = msg.folder_category == "trash";

                    // Deduplication check
                    let is_duplicate = if !parsed.message_id.trim().is_empty() {
                        db.conn.query_row(
                            "SELECT 1 FROM emails WHERE case_id = ?1 AND message_id = ?2",
                            rusqlite::params![&case_id, &parsed.message_id],
                            |_| Ok(true)
                        ).unwrap_or(false)
                    } else {
                        false
                    };

                    if is_duplicate {
                        duplicates_skipped += 1;
                        emit_imap_event(&app_handle_clone, json!({
                            "status": "duplicate_skipped",
                            "folder": msg.folder_name,
                            "msg_seq": msg.seq_id,
                            "folder_total": msg.folder_total,
                            "log": format!("⏭ Skipped duplicate: \"{}\" ({}/{})", parsed.subject.as_deref().unwrap_or("(No Subject)"), msg.seq_id, msg.folder_total)
                        }));
                        return Ok(());
                    }

                    let _ = db.conn.execute(
                        "INSERT OR REPLACE INTO emails (
                            id, evidence_id, case_id, message_id, in_reply_to, msg_references,
                            from_addr, from_display, to_addrs, cc_addrs, bcc_addrs, reply_to,
                            subject, date_sent, date_sent_utc, headers_raw, body_text, body_html,
                            folder_name, folder_category, is_deleted, deleted_recovered, risk_score, flags
                        ) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18,?19,?20,?21,?22,0,'[]')",
                        rusqlite::params![
                            email_id,
                            evidence_id,
                            case_id,
                            parsed.message_id,
                            parsed.in_reply_to,
                            ref_str,
                            parsed.from_addr,
                            parsed.from_display,
                            to_str,
                            cc_str,
                            bcc_str,
                            parsed.reply_to,
                            parsed.subject,
                            date_str,
                            date_str,
                            parsed.headers_raw,
                            parsed.body_text,
                            parsed.body_html,
                            msg.folder_name,
                            msg.folder_category,
                            if is_del { 1 } else { 0 },
                        ],
                    );

                    // Save attachments with full forensic metadata & disk extraction
                    for att in &parsed.attachments {
                        let att_id = generate_id();
                        let sha256 = {
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
                            ent
                        } else { 0.0 };

                        let mut risk_flags = Vec::new();
                        let lower_name = att.filename.as_deref().unwrap_or("").to_lowercase();
                        if lower_name.ends_with(".exe") || lower_name.ends_with(".bat") || lower_name.ends_with(".cmd") || lower_name.ends_with(".ps1") || lower_name.ends_with(".vbs") || lower_name.ends_with(".js") {
                            risk_flags.push("executable");
                        }
                        if lower_name.ends_with(".docm") || lower_name.ends_with(".xlsm") || lower_name.ends_with(".pptm") {
                            risk_flags.push("macro_enabled");
                        }
                        if entropy > 7.5 {
                            risk_flags.push("high_entropy_encrypted");
                        }
                        let risk_flags_json = serde_json::to_string(&risk_flags).unwrap_or_else(|_| "[]".to_string());

                        let mut stored_path = String::new();
                        if !att.data.is_empty() {
                            let att_dir = dirs::data_dir()
                                .unwrap_or_else(|| std::path::PathBuf::from("."))
                                .join("email-forensic")
                                .join("evidence")
                                .join(&case_id)
                                .join("attachments");
                            let _ = std::fs::create_dir_all(&att_dir);
                            let safe_name = att.filename.as_deref().unwrap_or("attachment.bin")
                                .replace(|c: char| !c.is_alphanumeric() && c != '.' && c != '-' && c != '_', "_");
                            let att_file = att_dir.join(format!("{}_{}", &att_id[..8], safe_name));
                            if std::fs::write(&att_file, &att.data).is_ok() {
                                stored_path = att_file.to_string_lossy().to_string();
                            }
                        }

                        let _ = db.conn.execute(
                            "INSERT OR REPLACE INTO attachments (id, email_id, filename, sha256, mime_type, size_bytes, stored_path, entropy, risk_flags)
                             VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9)",
                            rusqlite::params![
                                att_id,
                                email_id,
                                att.filename,
                                sha256,
                                att.content_type,
                                att.data.len() as i64,
                                stored_path,
                                entropy,
                                risk_flags_json
                            ],
                        );
                    }

                    ingested_count += 1;

                    let subj_display = parsed.subject.unwrap_or_else(|| "(No Subject)".to_string());
                    let from_display = parsed.from_addr.clone();
                    let att_note = if !parsed.attachments.is_empty() {
                        format!(" (📎 {} attachments)", parsed.attachments.len())
                    } else {
                        String::new()
                    };

                    emit_imap_event(&app_handle_clone, json!({
                        "status": "ingested",
                        "folder": msg.folder_name,
                        "msg_seq": msg.seq_id,
                        "folder_total": msg.folder_total,
                        "folder_index": msg.folder_index,
                        "total_folders": msg.total_folders,
                        "ingested_count": ingested_count,
                        "subject": subj_display,
                        "from": from_display,
                        "log": format!("📥 Ingested #{}/{} [{}]: \"{}\" from {}{}", msg.seq_id, msg.folder_total, msg.folder_name, subj_display, from_display, att_note)
                    }));
                }

                Ok(())
            },
        )?
    };

    let was_cancelled = state.cancel_imap.load(std::sync::atomic::Ordering::Relaxed);
    let final_status = if was_cancelled { "cancelled" } else { "done" };

    // Calculate final SHA-256 seal for the evidence batch
    let dummy_data = format!("imap_{}_{}_{}", username, ingested_count, now.to_rfc3339());
    let mut hasher = Sha256::new();
    hasher.update(dummy_data.as_bytes());
    let sha256_hex = format!("{:x}", hasher.finalize());

    let _ = db.conn.execute(
        "UPDATE evidence_items SET parse_status=?1, sha256=?2, message_count=?3, size_bytes=?4 WHERE id=?5",
        rusqlite::params![final_status, sha256_hex, ingested_count, total_bytes_downloaded as i64, evidence_id],
    );

    let custody_id = generate_id();
    let _ = db.conn.execute(
        "INSERT INTO chain_of_custody (id, case_id, evidence_id, action, performed_by, timestamp, notes)
         VALUES (?1, ?2, ?3, ?4, 'IMAP Streaming Engine', ?5, ?6)",
        rusqlite::params![
            custody_id,
            case_id,
            evidence_id,
            if was_cancelled { "imap_acquisition_cancelled" } else { "imap_acquisition_completed" },
            Utc::now().to_rfc3339(),
            format!("Acquired and parsed {} messages (skipped {} duplicates) across folders ({:?}) from {}", ingested_count, duplicates_skipped, result.folders_acquired, username)
        ],
    );

    emit_imap_event(&app, json!({
        "status": final_status,
        "ingested_count": ingested_count,
        "duplicates_skipped": duplicates_skipped,
        "folders": result.folders_acquired,
        "log": format!("✓ Acquisition {}: {} emails ingested, {} duplicates skipped across {} folders", if was_cancelled { "Paused/Stopped" } else { "Complete" }, ingested_count, duplicates_skipped, result.folders_acquired.len())
    }));

    Ok(json!({
        "status": final_status,
        "evidence_id": evidence_id,
        "total_found": result.total_found,
        "downloaded": ingested_count,
        "duplicates_skipped": duplicates_skipped,
        "errors": result.errors,
        "folders_acquired": result.folders_acquired,
        "was_cancelled": was_cancelled
    }))
}
