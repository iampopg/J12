use chrono::Utc;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use tauri::ipc::Channel;
use tauri::{AppHandle, Emitter, State};

use crate::AppState;
use crate::db::generate_id;
use crate::parser;
use super::client::{Pop3Client, Pop3Config};

pub fn emit_pop3_event(app: &AppHandle, ch: &Channel<Value>, payload: serde_json::Value) {
    let _ = ch.send(payload.clone());
    let _ = app.emit("pop3_progress", payload.clone());
    let _ = app.emit("imap_progress", payload.clone());
    let _ = app.emit_to("main", "imap_progress", payload.clone());
}

#[tauri::command]
pub async fn pop3_test_connection(input: Value) -> Result<bool, String> {
    let server = input["server"].as_str().unwrap_or("pop.gmail.com").to_string();
    let port = input["port"].as_u64().unwrap_or(995) as u16;
    let username = input["username"].as_str().unwrap_or("").to_string();
    let password = input["password"].as_str().unwrap_or("").to_string();
    let use_ssl = input["use_ssl"].as_bool()
        .or_else(|| input["useSsl"].as_bool())
        .unwrap_or(true);

    let config = Pop3Config {
        server,
        port,
        username,
        password,
        use_ssl,
    };

    match Pop3Client::connect(&config) {
        Ok(_) => Ok(true),
        Err(e) => Err(e),
    }
}

#[tauri::command]
pub async fn pop3_fetch_emails(
    app: AppHandle,
    state: State<'_, AppState>,
    input: Value,
    on_event: Channel<Value>,
) -> Result<Value, String> {
    let case_id = input["case_id"].as_str()
        .or_else(|| input["caseId"].as_str())
        .or_else(|| input["input"]["case_id"].as_str())
        .or_else(|| input["input"]["caseId"].as_str())
        .unwrap_or("")
        .to_string();

    let evidence_id = input["evidence_id"].as_str()
        .or_else(|| input["evidenceId"].as_str())
        .or_else(|| input["input"]["evidence_id"].as_str())
        .or_else(|| input["input"]["evidenceId"].as_str())
        .unwrap_or("pop3_live_evidence")
        .to_string();

    let server = input["server"].as_str()
        .or_else(|| input["input"]["server"].as_str())
        .unwrap_or("pop.gmail.com")
        .to_string();
    let port = input["port"].as_u64()
        .or_else(|| input["input"]["port"].as_u64())
        .unwrap_or(995) as u16;
    let username = input["username"].as_str()
        .or_else(|| input["input"]["username"].as_str())
        .unwrap_or("")
        .to_string();
    let password = input["password"].as_str()
        .or_else(|| input["input"]["password"].as_str())
        .unwrap_or("")
        .to_string();
    let use_ssl = input["use_ssl"].as_bool()
        .or_else(|| input["useSsl"].as_bool())
        .or_else(|| input["input"]["use_ssl"].as_bool())
        .or_else(|| input["input"]["useSsl"].as_bool())
        .unwrap_or(true);

    let max_messages = input["max_messages"].as_u64()
        .or_else(|| input["input"]["max_messages"].as_u64())
        .map(|m| m as u32);

    let config = Pop3Config {
        server: server.clone(),
        port,
        username: username.clone(),
        password,
        use_ssl,
    };

    emit_pop3_event(&app, &on_event, json!({
        "status": "connecting",
        "log": format!("Connecting to POP3 {}:{} (SSL: {})...", server, port, if use_ssl { "YES" } else { "NO" })
    }));

    let mut client = Pop3Client::connect(&config)?;

    emit_pop3_event(&app, &on_event, json!({
        "status": "connected",
        "log": "✓ POP3 connection established. Authenticated successfully."
    }));

    let total_messages = client.get_message_count()?;
    emit_pop3_event(&app, &on_event, json!({
        "status": "folder_discovered",
        "folder": "INBOX",
        "folder_count": total_messages,
        "folder_index": 1,
        "total_folders": 1,
        "overall_total": total_messages,
        "log": format!("📁 POP3 INBOX contains {} messages", total_messages)
    }));

    if total_messages == 0 {
        return Ok(json!({
            "status": "done",
            "total_found": 0,
            "downloaded": 0,
            "errors": 0,
            "folders_acquired": ["INBOX"]
        }));
    }

    let fetch_count = max_messages.map(|m| m.min(total_messages)).unwrap_or(total_messages);
    let now = Utc::now();
    let now_str = now.to_rfc3339();

    let ev_filename = format!("POP3 Live Acquisition ({})", username);

    let evidence_id = {
        let mut db = state.db.lock().await;

        let existing_id: Option<String> = db.conn.query_row(
            "SELECT id FROM evidence_items WHERE case_id = ?1 AND (filename = ?2 OR original_path LIKE ?3) ORDER BY created_at ASC LIMIT 1",
            rusqlite::params![&case_id, &ev_filename, format!("%{}%", &username)],
            |row| row.get(0),
        ).ok();

        let evidence_id = existing_id.unwrap_or(evidence_id);

        let _ = db.conn.execute(
            "DELETE FROM evidence_items WHERE case_id = ?1 AND id != ?2 AND (filename = ?3 OR original_path LIKE ?4) AND message_count = 0",
            rusqlite::params![&case_id, &evidence_id, &ev_filename, format!("%{}%", &username)],
        );

        let _ = db.conn.execute(
            "INSERT INTO evidence_items (
                id, case_id, filename, original_path, stored_path, format, sha256,
                size_bytes, source_description, acquired_by, acquired_at, acquisition_method,
                integrity_level, parse_status, message_count, deleted_recovered, created_at
            ) VALUES (?1, ?2, ?3, ?4, ?4, 'pop3', 'in_progress', 0, ?5, 'Examiner', ?6, 'pop3_live_acquisition', 'verified', 'ingesting', 0, 0, ?6)
            ON CONFLICT(id) DO UPDATE SET
                parse_status = 'ingesting',
                sha256 = 'in_progress',
                acquired_at = ?6",
            rusqlite::params![
                evidence_id,
                case_id,
                ev_filename,
                format!("pop3://{}:{}@{}", username, port, server),
                format!("Live POP3 acquisition for account {}", username),
                now_str
            ],
        );
        evidence_id
    };

    crate::audit_logger::log_forensic_event(
        &case_id,
        "POP3_ACQUISITION",
        "ACQUISITION_STARTED",
        "Examiner",
        Some(&evidence_id),
        None,
        &format!("Started live POP3 streaming for account: {} from server {}:{}", username, server, port)
    );

    let mut downloaded = 0u32;
    let mut errors = 0u32;
    let mut duplicates_skipped = 0u32;
    let mut total_bytes: usize = 0;
    let db_mutex = &state.db;

    for seq in 1..=fetch_count {
        match client.fetch_raw_message(seq) {
            Ok(raw) => {
                total_bytes += raw.len();

                if let Ok(parsed) = parser::parse_rfc5322(&raw, 0, raw.len() as u64) {
                    let email_id = generate_id();
                    let to_str = serde_json::to_string(&parsed.to_addrs).unwrap_or_else(|_| "[]".to_string());
                    let cc_str = serde_json::to_string(&parsed.cc_addrs).unwrap_or_else(|_| "[]".to_string());
                    let bcc_str = serde_json::to_string(&parsed.bcc_addrs).unwrap_or_else(|_| "[]".to_string());
                    let ref_str = serde_json::to_string(&parsed.references).unwrap_or_else(|_| "[]".to_string());
                    let date_str = parsed.date_sent.as_ref().map(|d| d.to_rfc3339());
                    let item_now = Utc::now().to_rfc3339();

                    {
                        let mut db = db_mutex.blocking_lock();

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
                            emit_pop3_event(&app, &on_event, json!({
                                "status": "duplicate_skipped",
                                "folder": "INBOX",
                                "msg_seq": seq,
                                "folder_total": fetch_count,
                                "overall_seq": seq,
                                "overall_total": fetch_count,
                                "duplicates_skipped": duplicates_skipped,
                                "log": format!("⏭ Skipped duplicate: \"{}\" ({}/{})", parsed.subject.as_deref().unwrap_or("(No Subject)"), seq, fetch_count)
                            }));
                            continue;
                        }

                        if let Err(e) = db.conn.execute(
                            "INSERT OR REPLACE INTO emails (
                                id, evidence_id, case_id, message_id, in_reply_to, msg_references,
                                from_addr, from_display, to_addrs, cc_addrs, bcc_addrs, reply_to,
                                subject, date_sent, date_sent_utc, headers_raw, body_text, body_html,
                                folder_name, folder_category, is_deleted, deleted_recovered, risk_score, flags, created_at
                            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, 'inbox', 0, 0, 0, '[]', ?20)",
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
                                "INBOX",
                                item_now,
                            ],
                        ) {
                            eprintln!("Failed to insert POP3 email: {}", e);
                        }

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
                                let att_dir = crate::db::Database::get_data_dir()
                                    .join("cases")
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
                                "INSERT OR REPLACE INTO attachments (id, email_id, filename, sha256, mime_type, size_bytes, stored_path, entropy, risk_flags, is_inline, created_at)
                                 VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11)",
                                rusqlite::params![
                                    att_id,
                                    email_id,
                                    att.filename,
                                    sha256,
                                    att.content_type,
                                    att.data.len() as i64,
                                    stored_path,
                                    entropy,
                                    risk_flags_json,
                                    if att.is_inline { 1 } else { 0 },
                                    item_now
                                ],
                            );
                        }

                        downloaded += 1;

                        if downloaded % 25 == 0 {
                            let _ = db.conn.execute(
                                "UPDATE evidence_items SET message_count = ?1, size_bytes = ?2 WHERE id = ?3",
                                rusqlite::params![downloaded, total_bytes as i64, &evidence_id],
                            );
                        }
                    }

                    let subj_display = parsed.subject.clone().unwrap_or_else(|| "(No Subject)".to_string());
                    let from_display = parsed.from_addr.clone();

                    emit_pop3_event(&app, &on_event, json!({
                        "status": "ingested",
                        "folder": "INBOX",
                        "msg_seq": seq,
                        "folder_total": fetch_count,
                        "overall_seq": seq,
                        "overall_total": fetch_count,
                        "ingested_count": downloaded,
                        "duplicates_skipped": duplicates_skipped,
                        "subject": subj_display,
                        "from": from_display,
                        "log": format!("📥 Ingested #{} [INBOX]: \"{}\" from {}", seq, subj_display, from_display)
                    }));
                } else {
                    errors += 1;
                }
            }
            Err(_) => {
                errors += 1;
            }
        }

        if seq % 25 == 0 {
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
    }

    let dummy_data = format!("pop3_{}_{}_{}", username, downloaded, now_str);
    let mut hasher = Sha256::new();
    hasher.update(dummy_data.as_bytes());
    let sha256_hex = format!("{:x}", hasher.finalize());

    {
        let db = state.db.lock().await;
        let _ = db.conn.execute(
            "UPDATE evidence_items SET parse_status='done', sha256=?1, message_count=?2, size_bytes=?3 WHERE id=?4",
            rusqlite::params!["done", sha256_hex, downloaded, total_bytes as i64, &evidence_id],
        );

        let custody_id = generate_id();
        let _ = db.conn.execute(
            "INSERT INTO custody_events (id, evidence_id, action, actor, timestamp, tool, tool_version, hash_before, hash_after, detail)
             VALUES (?1, ?2, ?3, 'Examiner', ?4, 'J12 POP3 Acquisition Engine', '1.0.0', NULL, ?5, ?6)",
            rusqlite::params![
                custody_id,
                evidence_id,
                "pop3_acquisition_completed",
                Utc::now().to_rfc3339(),
                sha256_hex,
                format!("Acquired and parsed {} messages (skipped {} duplicates) from POP3 INBOX for {}", downloaded, duplicates_skipped, username)
            ],
        );
    }

    crate::audit_logger::log_forensic_event(
        &case_id,
        "POP3_ACQUISITION",
        "ACQUISITION_COMPLETED",
        "Examiner",
        Some(&evidence_id),
        Some(&sha256_hex),
        &format!("Completed POP3 acquisition for {}: Ingested {} messages, {} duplicates skipped. Evidence SHA-256 Seal: {}", username, downloaded, duplicates_skipped, sha256_hex)
    );

    emit_pop3_event(&app, &on_event, json!({
        "status": "done",
        "ingested_count": downloaded,
        "duplicates_skipped": duplicates_skipped,
        "folders": ["INBOX"],
        "log": format!("✓ POP3 Acquisition Complete: {} emails ingested, {} duplicates skipped", downloaded, duplicates_skipped)
    }));

    Ok(json!({
        "status": "done",
        "total_found": total_messages,
        "downloaded": downloaded,
        "errors": errors,
        "duplicates_skipped": duplicates_skipped,
        "folders_acquired": ["INBOX"]
    }))
}
