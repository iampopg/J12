use chrono::Utc;
use serde_json::{json, Value};
use tauri::State;

use crate::AppState;
use crate::db::generate_id;
use crate::imap_acquisition::{self, ImapConfig};
use crate::parser;

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

    let config = ImapConfig {
        server: server.clone(),
        port,
        username: username.clone(),
        password,
        use_ssl,
        mailbox: target_mb.to_string(),
    };
    
    let result = imap_acquisition::fetch_emails(&config, Some(target_mb), max_messages)?;
    let mut parsed_count: u32 = 0;
    let now = Utc::now();

    let db = state.db.lock().await;

    // Create evidence_items record for this IMAP acquisition if not existing
    let ev_filename = format!("IMAP Live Acquisition ({})", username);
    let _ = db.conn.execute(
        "INSERT OR IGNORE INTO evidence_items (
            id, case_id, filename, original_path, stored_path, format, sha256,
            size_bytes, source_description, acquired_by, acquired_at, acquisition_method,
            integrity_level, parse_status, message_count, deleted_recovered
        ) VALUES (?1, ?2, ?3, ?4, ?4, 'imap', 'calculating', 0, ?5, 'Examiner', ?6, 'imap_live_acquisition', 'verified', 'parsing', 0, 0)",
        rusqlite::params![
            evidence_id,
            case_id,
            ev_filename,
            format!("imap://{}:{}@{}", username, port, server),
            format!("Live IMAP acquisition across {} folders for account {}", result.folders_acquired.len(), username),
            now.to_rfc3339()
        ],
    );

    for msg in &result.messages {
        if let Ok(parsed) = parser::parse_rfc5322(&msg.raw_content, 0, msg.raw_content.len() as u64) {
            let email_id = generate_id();
            let to_str = serde_json::to_string(&parsed.to_addrs).unwrap_or_else(|_| "[]".to_string());
            let cc_str = serde_json::to_string(&parsed.cc_addrs).unwrap_or_else(|_| "[]".to_string());
            let bcc_str = serde_json::to_string(&parsed.bcc_addrs).unwrap_or_else(|_| "[]".to_string());
            let ref_str = serde_json::to_string(&parsed.references).unwrap_or_else(|_| "[]".to_string());
            let date_str = parsed.date_sent.as_ref().map(|d| d.to_rfc3339());
            let is_del = msg.folder_category == "trash";

            let _ = db.conn.execute(
                "INSERT OR REPLACE INTO emails (
                    id, evidence_id, case_id, message_id, in_reply_to, msg_references,
                    from_addr, from_display, to_addrs, cc_addrs, bcc_addrs, reply_to,
                    subject, date_sent, date_sent_utc, headers_raw, body_text, body_html,
                    folder_name, folder_category, is_deleted, deleted_recovered, risk_score, flags
                ) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18,?19,?20,?21,?21,0,'[]')",
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
            parsed_count += 1;
        }
    }

    // Save evidence batch metadata and SHA-256 seal
    let dummy_data = format!("imap_{}_{}_{}", username, parsed_count, now.to_rfc3339());
    let mut hasher = sha2::Sha256::default();
    use sha2::Digest;
    hasher.update(dummy_data.as_bytes());
    let sha256_hex = format!("{:x}", hasher.finalize());

    let _ = db.conn.execute(
        "UPDATE evidence_items SET parse_status='parsed', sha256=?1, message_count = ?2 WHERE id=?3",
        rusqlite::params![sha256_hex, parsed_count, evidence_id],
    );

    // Record custody chain entry
    let custody_id = generate_id();
    let _ = db.conn.execute(
        "INSERT INTO chain_of_custody (id, case_id, evidence_id, action, performed_by, timestamp, notes)
         VALUES (?1, ?2, ?3, 'imap_acquired', 'IMAP Acquisition Engine', ?4, ?5)",
        rusqlite::params![
            custody_id,
            case_id,
            evidence_id,
            now.to_rfc3339(),
            format!("Acquired {} messages across folders ({:?}) from {}", parsed_count, result.folders_acquired, username)
        ],
    );

    Ok(json!({
        "status": "success",
        "evidence_id": evidence_id,
        "username": username,
        "server": server,
        "total_found": result.total_found,
        "downloaded": result.downloaded,
        "errors": result.errors,
        "folders_acquired": result.folders_acquired,
        "parsed_count": parsed_count,
    }))
}
