//! Email parsers for EML, MBOX, and MSG formats.
//! Each parser produces normalized RawEmail structs for the database.

use std::path::Path;
use std::fs;
use std::io::{Read, BufRead};
use sha2::{Sha256, Digest};
use chrono::{DateTime, Utc, TimeZone};
use crate::db::generate_id;

/// Normalized email output from any parser
#[derive(Debug, Clone)]
pub struct RawEmail {
    pub message_id: String,
    pub from_addr: String,
    pub from_display: Option<String>,
    pub to_addrs: Vec<String>,
    pub cc_addrs: Vec<String>,
    pub bcc_addrs: Vec<String>,
    pub subject: Option<String>,
    pub date_sent: Option<DateTime<Utc>>,
    pub headers_raw: String,
    pub body_text: Option<String>,
    pub body_html: Option<String>,
    pub raw_size: u64,
    pub raw_offset: u64,
    pub folder_name: Option<String>,
    pub folder_category: String,
    pub recovery_status: String, // inbox|sent|deleted|drafts|other
    pub attachments: Vec<RawAttachment>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct RawAttachment {
    pub filename: Option<String>,
    pub content_type: String,
    pub data: Vec<u8>,
}

/// Parse EML file (single RFC 5322 message)
pub fn parse_eml(path: &Path) -> Result<Vec<RawEmail>, String> {
    let data = fs::read(path).map_err(|e| format!("Read error: {}", e))?;
    let content = String::from_utf8_lossy(&data);
    let email = parse_rfc5322(&content, 0, data.len() as u64)?;
    Ok(vec![email])
}

/// Parse MBOX file (concatenated RFC 5322 messages)
/// Efficient: reads file once, splits on "From " lines
pub fn parse_mbox(path: &Path) -> Result<Vec<RawEmail>, String> {
    let data = fs::read(path).map_err(|e| format!("Read error: {}", e))?;
    let content = String::from_utf8_lossy(&data);
    let mut emails = Vec::new();
    
    // Split on "From " lines (mbox separator)
    let mut current_msg = String::new();
    let mut msg_offset: u64 = 0;
    let mut line_start: u64 = 0;
    
    for line in content.lines() {
        if line.starts_with("From ") && !current_msg.trim().is_empty() {
            // Parse the accumulated message
            let msg_len = line_start - msg_offset;
            match parse_rfc5322(&current_msg, msg_offset, msg_len) {
                Ok(email) => emails.push(email),
                Err(e) => {
                    // Skip malformed messages but continue
                    eprintln!("Warning: skipped message at offset {}: {}", msg_offset, e);
                }
            }
            current_msg.clear();
            msg_offset = line_start;
        }
        current_msg.push_str(line);
        current_msg.push('\n');
        line_start += line.len() as u64 + 1;
    }
    
    // Parse the last message
    if !current_msg.trim().is_empty() {
        let msg_len = content.len() as u64 - msg_offset;
        match parse_rfc5322(&current_msg, msg_offset, msg_len) {
            Ok(email) => emails.push(email),
            Err(e) => eprintln!("Warning: skipped last message: {}", e),
        }
    }
    
    Ok(emails)
}

/// Parse a single RFC 5322 message
pub fn parse_rfc5322(content: &str, offset: u64, size: u64) -> Result<RawEmail, String> {
    let mut warnings = Vec::new();
    
    // Split headers and body at first blank line
    let (header_section, body) = if let Some(idx) = content.find("\r\n\r\n") {
        (&content[..idx], &content[idx+4..])
    } else if let Some(idx) = content.find("\n\n") {
        (&content[..idx], &content[idx+2..])
    } else {
        (content, "")
    };
    
    let mut message_id = String::new();
    let mut from_addr = String::new();
    let mut from_display = None;
    let mut to_addrs = Vec::new();
    let mut cc_addrs = Vec::new();
    let mut subject = None;
    let mut date_sent = None;
    let mut content_type = String::new();
    let mut folder_raw: Option<String> = None;
    
    // Parse headers
    for line in header_section.lines() {
        if line.starts_with(' ') || line.starts_with('\t') {
            continue; // skip folded headers
        }
        if line.is_empty() {
            break;
        }
        if let Some((key, value)) = line.split_once(':') {
            let key = key.trim().to_lowercase();
            let value = value.trim();
            match key.as_str() {
                "message-id" => message_id = value.trim_matches(|c| c == '<' || c == '>').to_string(),
                "from" => {
                    if from_addr.is_empty() {
                        from_addr = extract_email(value);
                        from_display = extract_display_name(value);
                    }
                }
                "x-from" => {
                    if from_display.is_none() {
                        let name = value.trim().trim_matches('"').trim();
                        if !name.is_empty() && !name.contains('@') { from_display = Some(name.to_string()); }
                    }
                }
                "to" => {
                    for addr in extract_address_list(value) {
                        if !to_addrs.contains(&addr) { to_addrs.push(addr); }
                    }
                }
                "cc" => cc_addrs = extract_address_list(value),
                "subject" => subject = Some(value.to_string()),
                "date" => date_sent = parse_date(value),
                "content-type" => content_type = value.to_string(),
                "x-folder" => {
                    // Extract Outlook folder path for categorization
                    folder_raw = Some(value.to_string());
                }
                _ => {}
            }
        }
    }
    
    if message_id.is_empty() {
        message_id = format!("gen_{}", generate_id());
        warnings.push("Generated Message-ID".to_string());
    }
    
    if from_addr.is_empty() {
        from_addr = "unknown@unknown".to_string();
        warnings.push("Missing From address".to_string());
    }
    
    let (body_text, body_html, attachments) = if content_type.starts_with("multipart/") {
        // Parse multipart message
        let boundary = extract_boundary(&content_type);
        if let Some(boundary) = boundary {
            parse_multipart(body, &boundary)
        } else {
            (Some(body.to_string()), None, vec![])
        }
    } else if content_type.contains("text/html") {
        (None, Some(body.to_string()), vec![])
    } else if content_type.contains("text/plain") || content_type.is_empty() {
        {
            let text = if body.trim().is_empty() { None } else { Some(body.to_string()) };
            (text, None, vec![])
        }
    } else {
        (Some(body.to_string()), None, vec![])
    };
    
    // Categorize email based on X-Folder header with forensic distinction
    let (folder_name, folder_category, recovery_status) = match &folder_raw {
        Some(path) => {
            let lower = path.to_lowercase();
            let (category, recovery) = if lower.contains("sent") {
                ("sent", "normal")
            } else if lower.contains("deleted") {
                ("soft_deleted", "soft_deleted")  // In Deleted Items / Recycle Bin
            } else if lower.contains("draft") {
                ("drafts", "normal")
            } else if lower.contains("inbox") {
                ("inbox", "normal")
            } else if lower.contains("junk") || lower.contains("spam") {
                ("spam", "normal")
            } else {
                ("other", "normal")
            };
            (folder_raw.clone(), category.to_string(), recovery.to_string())
        }
        None => (None, "inbox".to_string(), "normal".to_string()),
    };
    
    Ok(RawEmail {
        message_id,
        from_addr,
        from_display,
        to_addrs,
        cc_addrs,
        bcc_addrs: vec![],
        folder_name,
        folder_category,
        recovery_status,
        subject,
        date_sent,
        headers_raw: header_section.chars().take(2000).collect(),
        body_text,
        body_html,
        raw_size: size,
        raw_offset: offset,
        attachments: vec![],
        warnings,
    })
}

fn extract_email(s: &str) -> String {
    if let Some(start) = s.find('<') {
        if let Some(end) = s.find('>') {
            if end > start { return s[start+1..end].to_string(); }
        }
    }
    // Try to find email-like pattern
    if let Some(start) = s.find(|c: char| c.is_alphanumeric() || c == '.' || c == '_' || c == '-' || c == '+') {
        let rest = &s[start..];
        if let Some(end) = rest.find(|c: char| c == ' ' || c == '>' || c == '\r' || c == '\n' || c == ',') {
            let candidate = &rest[..end];
            if candidate.contains('@') && candidate.contains('.') {
                return candidate.to_string();
            }
        } else if rest.contains('@') && rest.contains('.') {
            return rest.to_string();
        }
    }
    s.trim().to_string()
}

fn extract_display_name(s: &str) -> Option<String> {
    if let Some(start) = s.find('<') {
        let name = s[..start].trim().trim_matches('"').trim();
        if !name.is_empty() && !name.contains('@') { return Some(name.to_string()); }
    }
    None
}

fn extract_address_list(s: &str) -> Vec<String> {
    s.split(',').map(|a| extract_email(a.trim())).filter(|a| !a.is_empty() && a != "unknown@unknown").collect()
}

fn parse_date(s: &str) -> Option<DateTime<Utc>> {
    let s = s.trim();
    if let Ok(dt) = DateTime::parse_from_rfc2822(s) { return Some(dt.with_timezone(&Utc)); }
    for fmt in &["%a, %d %b %Y %H:%M:%S %z", "%d %b %Y %H:%M:%S %z", "%Y-%m-%dT%H:%M:%S%z", "%a, %d %b %Y %H:%M:%S"] {
        if let Ok(dt) = DateTime::parse_from_str(s, fmt) { return Some(dt.with_timezone(&Utc)); }
    }
    None
}

/// Compute SHA-256 hash of data
pub fn sha256_data(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    format!("{:x}", hasher.finalize())
}

/// Extract boundary string from Content-Type header
fn extract_boundary(content_type: &str) -> Option<String> {
    if let Some(idx) = content_type.find("boundary=") {
        let rest = &content_type[idx + 9..];
        let boundary = rest.trim_matches('"').trim_matches('\'').trim();
        if !boundary.is_empty() {
            return Some(boundary.to_string());
        }
    }
    None
}

/// Parse multipart MIME body into text, html, and attachments
fn parse_multipart(body: &str, boundary: &str) -> (Option<String>, Option<String>, Vec<RawAttachment>) {
    let mut text_parts = Vec::new();
    let mut html_parts = Vec::new();
    let mut attachments = Vec::new();
    
    let delimiter = format!("--{}", boundary);
    
    // Split by boundary
    let parts: Vec<&str> = body.split(&delimiter).collect();
    
    for part in &parts {
        let part = part.trim_start_matches("\r\n").trim_start_matches("\n");
        if part.is_empty() || part == "--" {
            continue;
        }
        
        // Split headers from body
        let (header_section, body_content) = if let Some(idx) = part.find("\r\n\r\n") {
            (&part[..idx], &part[idx+4..])
        } else if let Some(idx) = part.find("\n\n") {
            (&part[..idx], &part[idx+2..])
        } else {
            continue;
        };
        
        // Parse part headers
        let mut part_content_type = String::new();
        let mut part_encoding = String::new();
        let mut part_filename = None;
        let mut part_name = None;
        
        for line in header_section.lines() {
            if let Some((key, value)) = line.split_once(':') {
                let key = key.trim().to_lowercase();
                let value = value.trim();
                match key.as_str() {
                    "content-type" => {
                        part_content_type = value.split(';').next().unwrap_or(value).trim().to_lowercase();
                        // Extract filename from Content-Type name= parameter
                        if let Some(idx) = value.find("name=") {
                            let name_rest = &value[idx + 5..];
                            part_name = Some(name_rest.trim_matches('"').trim().to_string());
                        }
                    }
                    "content-transfer-encoding" => part_encoding = value.to_lowercase(),
                    "content-disposition" => {
                        if let Some(idx) = value.find("filename=") {
                            let fname_rest = &value[idx + 9..];
                            part_filename = Some(fname_rest.trim_matches('"').trim_matches('\'').trim().to_string());
                        }
                    }
                    _ => {}
                }
            }
        }
        
        // Decode body
        let decoded = if part_encoding.contains("base64") {
            base64_decode(body_content.trim())
        } else if part_encoding.contains("quoted-printable") {
            qp_decode(body_content)
        } else {
            body_content.as_bytes().to_vec()
        };
        
        // Categorize part
        if part_content_type.starts_with("multipart/") {
            // Nested multipart - recurse
            if let Some(inner_boundary) = extract_boundary(&part_content_type) {
                let (t, h, a) = parse_multipart(body_content, &inner_boundary);
                if let Some(t) = t { text_parts.push(t); }
                if let Some(h) = h { html_parts.push(h); }
                attachments.extend(a);
            }
        } else if part_content_type.starts_with("text/plain") {
            text_parts.push(String::from_utf8_lossy(&decoded).to_string());
        } else if part_content_type.starts_with("text/html") {
            html_parts.push(String::from_utf8_lossy(&decoded).to_string());
        } else if part_content_type.starts_with("image/") || part_content_type.starts_with("application/") {
            let filename = part_filename.or(part_name).or_else(|| {
                // Generate filename from content type
                let ext = match part_content_type.as_str() {
                    "image/jpeg" | "image/jpg" => "jpg",
                    "image/png" => "png",
                    "image/gif" => "gif",
                    "application/pdf" => "pdf",
                    "application/zip" => "zip",
                    "application/msword" => "doc",
                    _ => "bin",
                };
                Some(format!("attachment_{}.{}", attachments.len() + 1, ext))
            });
            
            attachments.push(RawAttachment {
                filename,
                content_type: part_content_type,
                data: decoded,
            });
        } else if !decoded.is_empty() {
            // Other types - treat as attachment
            let filename = part_filename.or(part_name);
            attachments.push(RawAttachment {
                filename,
                content_type: part_content_type,
                data: decoded,
            });
        }
    }
    
    (
        if text_parts.is_empty() { None } else { Some(text_parts.join("\n")) },
        if html_parts.is_empty() { None } else { Some(html_parts.join("\n")) },
        attachments,
    )
}

/// Simple base64 decode
fn base64_decode(input: &str) -> Vec<u8> {
    let cleaned: String = input.chars().filter(|c| !c.is_whitespace()).collect();
    match base64::Engine::decode(&base64::engine::general_purpose::STANDARD, &cleaned) {
        Ok(data) => data,
        Err(_) => cleaned.as_bytes().to_vec(),
    }
}

/// Simple quoted-printable decode
fn qp_decode(input: &str) -> Vec<u8> {
    let mut result = Vec::new();
    let mut chars = input.chars().peekable();
    
    while let Some(c) = chars.next() {
        if c == '=' {
            if let (Some(h), Some(l)) = (chars.next(), chars.next()) {
                if h == '\r' || h == '\n' {
                    // Soft line break, skip
                    continue;
                }
                let hex = format!("{}{}", h, l);
                if let Ok(byte) = u8::from_str_radix(&hex, 16) {
                    result.push(byte);
                }
            }
        } else {
            result.push(c as u8);
        }
    }
    
    result
}