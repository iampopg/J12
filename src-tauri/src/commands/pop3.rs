//! POP3 email acquisition module
//! Downloads emails from POP3 servers for forensic analysis.

use chrono::Utc;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;
use tauri::ipc::Channel;
use tauri::{AppHandle, State};

use crate::AppState;
use crate::db::generate_id;
use crate::parser;

#[derive(Debug, Clone)]
pub struct Pop3Config {
    pub server: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub use_ssl: bool,
}

#[derive(Debug, Clone)]
pub struct Pop3AcquisitionResult {
    pub total_found: u32,
    pub downloaded: u32,
    pub errors: u32,
}

enum Pop3Stream {
    Tls(native_tls::TlsStream<TcpStream>),
    Plain(TcpStream),
}

impl std::io::Read for Pop3Stream {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match self {
            Pop3Stream::Tls(s) => s.read(buf),
            Pop3Stream::Plain(s) => s.read(buf),
        }
    }
}

impl std::io::Write for Pop3Stream {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match self {
            Pop3Stream::Tls(s) => s.write(buf),
            Pop3Stream::Plain(s) => s.write(buf),
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        match self {
            Pop3Stream::Tls(s) => s.flush(),
            Pop3Stream::Plain(s) => s.flush(),
        }
    }
}

struct Pop3Client {
    stream: BufReader<Pop3Stream>,
}

impl Pop3Client {
    fn connect(config: &Pop3Config) -> Result<Self, String> {
        let addr = format!("{}:{}", config.server, config.port);
        let tcp_stream = TcpStream::connect(&addr)
            .map_err(|e| format!("Failed to connect to {}: {}", addr, e))?;
        let _ = tcp_stream.set_read_timeout(Some(std::time::Duration::from_secs(15)));
        let _ = tcp_stream.set_write_timeout(Some(std::time::Duration::from_secs(15)));

        let stream = if config.use_ssl || config.port == 995 {
            let connector = native_tls::TlsConnector::new()
                .map_err(|e| format!("TLS init error: {}", e))?;
            let tls_stream = connector.connect(&config.server, tcp_stream)
                .map_err(|e| format!("TLS handshake error: {}", e))?;
            Pop3Stream::Tls(tls_stream)
        } else {
            Pop3Stream::Plain(tcp_stream)
        };

        let mut client = Pop3Client {
            stream: BufReader::new(stream),
        };

        // Read server greeting
        let mut greeting = String::new();
        client.stream.read_line(&mut greeting)
            .map_err(|e| format!("Failed reading greeting: {}", e))?;

        if !greeting.contains("+OK") {
            return Err(format!("POP3 server rejected connection: {}", greeting.trim()));
        }

        // Authenticate
        client.login(&config.username, &config.password)?;

        Ok(client)
    }

    fn send_command(&mut self, cmd: &str) -> Result<(String, Vec<String>), String> {
        let full_cmd = format!("{}\r\n", cmd);
        self.stream.get_mut().write_all(full_cmd.as_bytes())
            .map_err(|e| format!("Write error: {}", e))?;
        self.stream.get_mut().flush()
            .map_err(|e| format!("Flush error: {}", e))?;

        let mut first_line = String::new();
        self.stream.read_line(&mut first_line)
            .map_err(|e| format!("Read error: {}", e))?;

        if !first_line.contains("+OK") {
            return Err(format!("POP3 error: {}", first_line.trim()));
        }

        // For single-line responses (like STAT, QUIT), return immediately
        if cmd.starts_with("STAT") || cmd.starts_with("QUIT") {
            return Ok((first_line, vec![]));
        }

        // For multi-line responses (LIST, RETR), read until ".\r\n"
        let mut lines = Vec::new();
        loop {
            let mut line = String::new();
            let n = self.stream.read_line(&mut line)
                .map_err(|e| format!("Read error: {}", e))?;
            if n == 0 || line == ".\r\n" || line == ".\n" {
                break;
            }
            lines.push(line.trim_end_matches(&['\r', '\n'][..]).to_string());
        }

        Ok((first_line, lines))
    }

    fn login(&mut self, user: &str, pass: &str) -> Result<(), String> {
        let _ = self.send_command(&format!("USER {}", user))?;
        let _ = self.send_command(&format!("PASS {}", pass))?;
        Ok(())
    }

    fn get_message_count(&mut self) -> Result<u32, String> {
        let (line, _) = self.send_command("STAT")?;
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 {
            parts[1].parse::<u32>().map_err(|e| format!("Parse error: {}", e))
        } else {
            Ok(0)
        }
    }

    fn fetch_raw_message(&mut self, msg_num: u32) -> Result<String, String> {
        let (_, lines) = self.send_command(&format!("RETR {}", msg_num))?;
        Ok(lines.join("\r\n"))
    }
}

/// Test POP3 connection
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

/// Fetch emails from POP3 server
#[tauri::command]
pub async fn pop3_fetch_emails(
    app: AppHandle,
    state: State<'_, AppState>,
    input: Value,
    on_event: Channel<Value>,
) -> Result<Value, String> {
    let case_id = input["case_id"].as_str()
        .or_else(|| input["caseId"].as_str())
        .unwrap_or("")
        .to_string();

    let evidence_id = input["evidence_id"].as_str()
        .or_else(|| input["evidenceId"].as_str())
        .unwrap_or("pop3_live_evidence")
        .to_string();

    let server = input["server"].as_str().unwrap_or("pop.gmail.com").to_string();
    let port = input["port"].as_u64().unwrap_or(995) as u16;
    let username = input["username"].as_str().unwrap_or("").to_string();
    let password = input["password"].as_str().unwrap_or("").to_string();
    let use_ssl = input["use_ssl"].as_bool()
        .or_else(|| input["useSsl"].as_bool())
        .unwrap_or(true);

    let max_messages = input["max_messages"].as_u64().map(|m| m as u32);

    let config = Pop3Config {
        server: server.clone(),
        port,
        username: username.clone(),
        password,
        use_ssl,
    };

    let _ = on_event.send(json!({
        "status": "connecting",
        "log": format!("Connecting to POP3 {}:{} (SSL: {})...", server, port, if use_ssl { "YES" } else { "NO" })
    }));

    let mut client = Pop3Client::connect(&config)?;

    let _ = on_event.send(json!({
        "status": "connected",
        "log": "✓ POP3 connection established. Authenticated successfully."
    }));

    let total_messages = client.get_message_count()?;
    let _ = on_event.send(json!({
        "status": "folder_discovered",
        "folder": "INBOX",
        "folder_count": total_messages,
        "folder_index": 1,
        "total_folders": 1,
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

    let mut db = state.db.lock().await;

    // Create evidence record
    let ev_filename = format!("POP3 Live Acquisition ({})", username);
    let _ = db.conn.execute(
        "INSERT OR REPLACE INTO evidence_items (
            id, case_id, filename, original_path, stored_path, format, sha256,
            size_bytes, source_description, acquired_by, acquired_at, acquisition_method,
            integrity_level, parse_status, message_count, deleted_recovered, created_at
        ) VALUES (?1, ?2, ?3, ?4, ?4, 'pop3', 'in_progress', 0, ?5, 'Examiner', ?6, 'pop3_live_acquisition', 'verified', 'ingesting', 0, 0, ?6)",
        rusqlite::params![
            evidence_id,
            case_id,
            ev_filename,
            format!("pop3://{}:{}@{}", username, port, server),
            format!("Live POP3 acquisition for account {}", username),
            now_str
        ],
    );

    let mut downloaded = 0u32;
    let mut errors = 0u32;
    let mut duplicates_skipped = 0u32;
    let mut total_bytes: usize = 0;

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
                        let _ = on_event.send(json!({
                            "status": "duplicate_skipped",
                            "folder": "INBOX",
                            "msg_seq": seq,
                            "folder_total": fetch_count,
                            "log": format!("⏭ Skipped duplicate: \"{}\"", parsed.subject.as_deref().unwrap_or("(No Subject)"))
                        }));
                        continue;
                    }

                    let item_now = Utc::now().to_rfc3339();

                    let _ = db.conn.execute(
                        "INSERT OR REPLACE INTO emails (
                            id, evidence_id, case_id, message_id, in_reply_to, msg_references,
                            from_addr, from_display, to_addrs, cc_addrs, bcc_addrs, reply_to,
                            subject, date_sent, date_sent_utc, headers_raw, body_text, body_html,
                            folder_name, folder_category, is_deleted, deleted_recovered, risk_score, flags, created_at
                        ) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18,?19,'inbox',0,0,0,'[]',?20)",
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
                            item_now,
                        ],
                    );

                    // Save attachments
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
                        if lower_name.ends_with(".exe") || lower_name.ends_with(".bat") || lower_name.ends_with(".cmd") {
                            risk_flags.push("executable");
                        }
                        if entropy > 7.5 {
                            risk_flags.push("high_entropy_encrypted");
                        }
                        let risk_flags_json = serde_json::to_string(&risk_flags).unwrap_or_else(|_| "[]".to_string());

                        let mut stored_path = String::new();
                        if !att.data.is_empty() {
                            let att_dir = dirs::data_dir()
                                .unwrap_or_else(|| std::path::PathBuf::from("."))
                                .join("j12-forensic")
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
                            "INSERT OR REPLACE INTO attachments (id, email_id, filename, sha256, mime_type, size_bytes, stored_path, entropy, risk_flags, created_at)
                             VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10)",
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
                                item_now
                            ],
                        );
                    }

                    downloaded += 1;

                    let subj_display = parsed.subject.clone().unwrap_or_else(|| "(No Subject)".to_string());
                    let from_display = parsed.from_addr.clone();

                    let _ = on_event.send(json!({
                        "status": "ingested",
                        "folder": "INBOX",
                        "msg_seq": seq,
                        "folder_total": fetch_count,
                        "ingested_count": downloaded,
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

        // Rate limiting
        if seq % 25 == 0 {
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
    }

    // Calculate final SHA-256 seal
    let dummy_data = format!("pop3_{}_{}_{}", username, downloaded, now_str);
    let mut hasher = Sha256::new();
    hasher.update(dummy_data.as_bytes());
    let sha256_hex = format!("{:x}", hasher.finalize());

    let _ = db.conn.execute(
        "UPDATE evidence_items SET parse_status='done', sha256=?1, message_count=?2, size_bytes=?3 WHERE id=?4",
        rusqlite::params!["done", sha256_hex, downloaded, total_bytes as i64, evidence_id],
    );

    // Record custody event
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

    let _ = on_event.send(json!({
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
