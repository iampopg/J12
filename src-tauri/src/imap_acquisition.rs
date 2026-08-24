//! Live IMAP email acquisition module
//! Connects to real IMAP servers over TLS (port 993) or standard port (143),
//! authenticates credentials, enumerates ALL mailboxes (Inbox, Sent, Trash, Spam, Drafts, Archive, etc.),
//! and downloads raw RFC-822 EML emails across the entire account without arbitrary limits.

use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;
use std::path::PathBuf;
use native_tls::TlsConnector;

#[derive(Debug, Clone)]
pub struct ImapConfig {
    pub server: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub use_ssl: bool,
    pub mailbox: String,
}

#[derive(Debug, Clone)]
pub struct ImapFolderMessage {
    pub folder_name: String,
    pub folder_category: String,
    pub raw_content: String,
}

#[derive(Debug, Clone)]
pub struct ImapAcquisitionResult {
    pub total_found: u32,
    pub downloaded: u32,
    pub errors: u32,
    pub folders_acquired: Vec<String>,
    pub messages: Vec<ImapFolderMessage>,
}

enum ImapStream {
    Tls(native_tls::TlsStream<TcpStream>),
    Plain(TcpStream),
}

impl std::io::Read for ImapStream {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match self {
            ImapStream::Tls(s) => s.read(buf),
            ImapStream::Plain(s) => s.read(buf),
        }
    }
}

impl std::io::Write for ImapStream {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match self {
            ImapStream::Tls(s) => s.write(buf),
            ImapStream::Plain(s) => s.write(buf),
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        match self {
            ImapStream::Tls(s) => s.flush(),
            ImapStream::Plain(s) => s.flush(),
        }
    }
}

struct ImapClient {
    stream: BufReader<ImapStream>,
    tag_counter: u32,
}

pub fn categorize_imap_folder(folder_name: &str) -> String {
    let lower = folder_name.to_lowercase();
    if lower.contains("sent") {
        "sent".to_string()
    } else if lower.contains("draft") {
        "drafts".to_string()
    } else if lower.contains("trash") || lower.contains("deleted") || lower.contains("bin") {
        "trash".to_string()
    } else if lower.contains("spam") || lower.contains("junk") {
        "spam".to_string()
    } else if lower.contains("archive") || lower.contains("all mail") {
        "archive".to_string()
    } else if lower.contains("inbox") {
        "inbox".to_string()
    } else {
        "other".to_string()
    }
}

impl ImapClient {
    fn connect(config: &ImapConfig) -> Result<Self, String> {
        let addr = format!("{}:{}", config.server, config.port);
        let tcp_stream = TcpStream::connect(&addr).map_err(|e| format!("Failed to connect to {}: {}", addr, e))?;

        let stream = if config.use_ssl || config.port == 993 {
            let connector = TlsConnector::new().map_err(|e| format!("TLS init error: {}", e))?;
            let tls_stream = connector.connect(&config.server, tcp_stream).map_err(|e| format!("TLS handshake error: {}", e))?;
            ImapStream::Tls(tls_stream)
        } else {
            ImapStream::Plain(tcp_stream)
        };

        let mut client = ImapClient {
            stream: BufReader::new(stream),
            tag_counter: 1,
        };

        // Read server greeting banner
        let mut banner = String::new();
        client.stream.read_line(&mut banner).map_err(|e| format!("Failed reading greeting: {}", e))?;

        // Authenticate with LOGIN
        client.login(&config.username, &config.password)?;

        Ok(client)
    }

    fn next_tag(&mut self) -> String {
        let tag = format!("A{:04}", self.tag_counter);
        self.tag_counter += 1;
        tag
    }

    fn send_command(&mut self, cmd: &str) -> Result<(String, Vec<String>), String> {
        let tag = self.next_tag();
        let full_cmd = format!("{} {}\r\n", tag, cmd);
        self.stream.get_mut().write_all(full_cmd.as_bytes()).map_err(|e| format!("Write command error: {}", e))?;
        self.stream.get_mut().flush().map_err(|e| format!("Flush error: {}", e))?;

        let mut responses = Vec::new();
        loop {
            let mut line = String::new();
            let n = self.stream.read_line(&mut line).map_err(|e| format!("Read error: {}", e))?;
            if n == 0 {
                return Err("Connection closed unexpectedly by IMAP server".to_string());
            }

            let trimmed = line.trim_end_matches(&['\r', '\n'][..]).to_string();
            if trimmed.starts_with(&tag) {
                if trimmed.contains("OK") {
                    return Ok((trimmed, responses));
                } else {
                    return Err(format!("IMAP error: {}", trimmed));
                }
            } else {
                responses.push(trimmed);
            }
        }
    }

    fn login(&mut self, user: &str, pass: &str) -> Result<(), String> {
        let clean_user = user.trim().replace('"', "\\\"");
        let mut clean_pass = pass.trim().trim_matches(&['\\', '"', '\'', ' '][..]).trim().to_string();
        
        // If password has spaces and matches 16-char App Password pattern (or Gmail account), strip spaces
        let no_spaces = clean_pass.replace(' ', "");
        if no_spaces.len() == 16 || user.to_lowercase().contains("gmail") {
            clean_pass = no_spaces;
        }

        let cmd = format!("LOGIN \"{}\" \"{}\"", clean_user, clean_pass.replace('"', "\\\""));
        self.send_command(&cmd).map_err(|e| format!("IMAP Authentication failed. Check your password or use an App Password: {}", e))?;
        Ok(())
    }

    fn list_mailboxes(&mut self) -> Result<Vec<String>, String> {
        let (_, lines) = self.send_command("LIST \"\" \"*\"")?;
        let mut boxes = Vec::new();
        for line in lines {
            if line.to_uppercase().starts_with("* LIST") {
                if let Some(last_quote) = line.rfind('"') {
                    if let Some(first_quote) = line[..last_quote].rfind('"') {
                        let name = line[first_quote+1..last_quote].to_string();
                        if !name.is_empty() && !boxes.contains(&name) {
                            boxes.push(name);
                            continue;
                        }
                    }
                }
                if let Some(last_space) = line.rfind(' ') {
                    let folder = line[last_space+1..].trim().to_string();
                    if !folder.is_empty() && !boxes.contains(&folder) {
                        boxes.push(folder);
                    }
                }
            }
        }
        if boxes.is_empty() {
            boxes.push("INBOX".to_string());
        }
        Ok(boxes)
    }

    fn select_mailbox(&mut self, mailbox: &str) -> Result<u32, String> {
        let cmd = format!("SELECT \"{}\"", mailbox);
        let (_, lines) = self.send_command(&cmd)?;
        let mut exists_count = 0;
        for line in lines {
            let upper = line.to_uppercase();
            if upper.starts_with("* ") && upper.contains("EXISTS") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    if let Ok(num) = parts[1].parse::<u32>() {
                        exists_count = num;
                    }
                }
            }
        }
        Ok(exists_count)
    }

    fn fetch_raw_message(&mut self, seq_id: u32) -> Result<String, String> {
        let tag = self.next_tag();
        let cmd = format!("{} FETCH {} (BODY.PEEK[])\r\n", tag, seq_id);
        self.stream.get_mut().write_all(cmd.as_bytes()).map_err(|e| e.to_string())?;
        self.stream.get_mut().flush().map_err(|e| e.to_string())?;

        let mut first_line = String::new();
        self.stream.read_line(&mut first_line).map_err(|e| e.to_string())?;

        let byte_count = if let Some(open_brace) = first_line.rfind('{') {
            if let Some(close_brace) = first_line[open_brace..].find('}') {
                let num_str = &first_line[open_brace+1..open_brace+close_brace];
                num_str.parse::<usize>().unwrap_or(0)
            } else { 0 }
        } else { 0 };

        let mut body_bytes = vec![0u8; byte_count];
        if byte_count > 0 {
            use std::io::Read;
            self.stream.read_exact(&mut body_bytes).map_err(|e| e.to_string())?;
        }

        loop {
            let mut line = String::new();
            self.stream.read_line(&mut line).map_err(|e| e.to_string())?;
            if line.starts_with(&tag) {
                break;
            }
        }

        String::from_utf8(body_bytes.clone()).or_else(|_| Ok(String::from_utf8_lossy(&body_bytes).to_string()))
    }
}

/// List available mailboxes on remote IMAP server
pub fn list_mailboxes(config: &ImapConfig) -> Result<Vec<String>, String> {
    let mut client = ImapClient::connect(config)?;
    client.list_mailboxes()
}

/// Dynamically fetch emails across ALL mailboxes or a specific selected mailbox
pub fn fetch_emails(
    config: &ImapConfig,
    target_mailbox: Option<&str>,
    max_per_folder: Option<u32>,
) -> Result<ImapAcquisitionResult, String> {
    let mut client = ImapClient::connect(config)?;
    let all_folders = client.list_mailboxes()?;

    let folders_to_process = if let Some(single) = target_mailbox {
        if single.is_empty() || single.to_uppercase() == "ALL" {
            all_folders
        } else {
            vec![single.to_string()]
        }
    } else {
        all_folders
    };

    let mut total_found = 0;
    let mut downloaded = 0;
    let mut errors = 0;
    let mut folders_acquired = Vec::new();
    let mut messages = Vec::new();

    for folder in &folders_to_process {
        // Skip [Gmail]/All Mail if other individual folders are selected to avoid duplicate downloading
        if folders_to_process.len() > 1 && (folder.contains("All Mail") || folder.contains("Chats")) {
            continue;
        }

        match client.select_mailbox(folder) {
            Ok(count) => {
                total_found += count;
                if count > 0 {
                    folders_acquired.push(folder.clone());
                    let limit = if let Some(m) = max_per_folder { m.min(count) } else { count };
                    let category = categorize_imap_folder(folder);

                    for seq in 1..=limit {
                        match client.fetch_raw_message(seq) {
                            Ok(raw) => {
                                downloaded += 1;
                                messages.push(ImapFolderMessage {
                                    folder_name: folder.clone(),
                                    folder_category: category.clone(),
                                    raw_content: raw,
                                });
                            }
                            Err(_) => {
                                errors += 1;
                            }
                        }
                        
                        // Rate limiting: be nice to IMAP server (Gmail allows ~250 commands/min)
                        if seq % 25 == 0 {
                            std::thread::sleep(std::time::Duration::from_millis(100));
                        }
                    }
                }
            }
            Err(_) => {
                errors += 1;
            }
        }
    }

    Ok(ImapAcquisitionResult {
        total_found,
        downloaded,
        errors,
        folders_acquired,
        messages,
    })
}

/// Save raw email to evidence store
pub fn save_imap_email(
    evidence_id: &str,
    case_id: &str,
    raw_email: &str,
    message_id: &str,
) -> Result<String, String> {
    let storage_dir = dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("j12-forensic")
        .join("evidence")
        .join(case_id)
        .join("imap")
        .join(evidence_id);
    
    std::fs::create_dir_all(&storage_dir).map_err(|e| format!("Create dir: {}", e))?;
    
    let safe_id = message_id.replace(|c: char| !c.is_alphanumeric() && c != '-' && c != '_', "_");
    let filename = format!("{}.eml", if safe_id.is_empty() { "msg" } else { &safe_id });
    let filepath = storage_dir.join(&filename);
    
    std::fs::write(&filepath, raw_email).map_err(|e| format!("Write file: {}", e))?;
    
    Ok(filepath.to_string_lossy().to_string())
}