use chrono::Utc;
use serde_json::json;
use tauri::State;

use crate::AppState;
use crate::db::generate_id;
use crate::imap_acquisition::{self, ImapConfig};
use crate::parser;

#[tauri::command]
pub async fn imap_list_mailboxes(
    server: String,
    port: u16,
    username: String,
    password: String,
    use_ssl: bool,
) -> Result<Vec<String>, String> {
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
    case_id: String,
    evidence_id: String,
    server: String,
    port: u16,
    username: String,
    password: String,
    use_ssl: bool,
    mailbox: String,
    max_messages: Option<u32>,
) -> Result<serde_json::Value, String> {
    let config = ImapConfig {
        server: server.clone(),
        port,
        username: username.clone(),
        password,
        use_ssl,
        mailbox: mailbox.clone(),
    };
    
    let result = imap_acquisition::fetch_emails(&config, max_messages)?;
    let mut parsed_count: u32 = 0;

    let db = state.db.lock().await;
    for raw in &result.messages {
        if let Ok(parsed) = parser::parse_rfc5322(raw, 0, raw.len() as u64) {
            let email_id = generate_id();
            let to_str = serde_json::to_string(&parsed.to_addrs).unwrap_or_else(|_| "[]".to_string());
            let cc_str = serde_json::to_string(&parsed.cc_addrs).unwrap_or_else(|_| "[]".to_string());
            let bcc_str = serde_json::to_string(&parsed.bcc_addrs).unwrap_or_else(|_| "[]".to_string());
            let ref_str = serde_json::to_string(&parsed.references).unwrap_or_else(|_| "[]".to_string());
            let date_str = parsed.date_sent.as_ref().map(|d| d.to_rfc3339());

            let _ = db.conn.execute(
                "INSERT OR REPLACE INTO emails (
                    id, evidence_id, case_id, message_id, in_reply_to, msg_references,
                    from_addr, from_display, to_addrs, cc_addrs, bcc_addrs, reply_to,
                    subject, date_sent, date_sent_utc, headers_raw, body_text, body_html,
                    folder_name, folder_category, is_deleted, deleted_recovered, risk_score, flags
                ) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18,?19,?20,0,0,0,'[]')",
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
                    mailbox,
                    "inbox",
                ],
            );
            parsed_count += 1;
        }
    }

    // Update evidence item stats
    let _ = db.conn.execute(
        "UPDATE evidence_items SET parse_status='parsed', message_count = message_count + ?1 WHERE id=?2",
        rusqlite::params![parsed_count, &evidence_id],
    );

    // Record custody chain
    let custody_id = generate_id();
    let _ = db.conn.execute(
        "INSERT INTO chain_of_custody (id, case_id, evidence_id, action, performed_by, timestamp, notes)
         VALUES (?1, ?2, ?3, 'imap_acquired', ?4, ?5, ?6)",
        rusqlite::params![
            custody_id,
            case_id,
            evidence_id,
            username,
            Utc::now().to_rfc3339(),
            format!("Acquired {} messages from IMAP mailbox '{}' on {}", parsed_count, mailbox, server)
        ],
    );

    Ok(json!({
        "total_found": result.total_found,
        "downloaded": result.downloaded,
        "parsed": parsed_count,
        "errors": result.errors,
        "mailbox": mailbox
    }))
}
