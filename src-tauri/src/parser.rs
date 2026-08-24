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
    pub to_display_names: Vec<String>,
    pub cc_addrs: Vec<String>,
    pub cc_display_names: Vec<String>,
    pub bcc_addrs: Vec<String>,
    pub subject: Option<String>,
    pub subject_raw: Option<String>,
    pub date_sent: Option<DateTime<Utc>>,
    pub headers_raw: String,          // COMPLETE raw headers - NO truncation
    pub headers_json: String,         // All headers as JSON
    pub body_text: Option<String>,
    pub body_html: Option<String>,
    pub raw_size: u64,
    pub raw_offset: u64,
    pub folder_name: Option<String>,
    pub folder_category: String,
    pub recovery_status: String,
    pub attachments: Vec<RawAttachment>,
    pub warnings: Vec<String>,
    // Forensic headers
    pub received_chain: Vec<String>,
    pub return_path: Option<String>,
    pub reply_to: Option<String>,
    pub x_mailer: Option<String>,
    pub x_originating_ip: Option<String>,
    pub importance: Option<String>,
    pub in_reply_to: Option<String>,
    pub references: Vec<String>,
    pub x_to_header: Option<String>,
    pub x_cc_header: Option<String>,
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
    let mut to_display_names = Vec::new();
    let mut cc_addrs = Vec::new();
    let mut cc_display_names = Vec::new();
    let mut bcc_addrs = Vec::new();
    let mut subject = None;
    let mut subject_raw = None;
    let mut date_sent = None;
    let mut content_type = String::new();
    let mut folder_raw: Option<String> = None;
    
    // Forensic headers
    let mut received_chain = Vec::new();
    let mut return_path = None;
    let mut reply_to = None;
    let mut x_mailer = None;
    let mut x_originating_ip = None;
    let mut importance = None;
    let mut in_reply_to = None;
    let mut references = Vec::new();
    let mut x_to_header = None;
    let mut x_cc_header = None;
    
    // Build JSON headers
    let mut headers_map: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();
    
    // Parse headers - handle folded headers properly
    let mut current_header_key = String::new();
    let mut current_header_value = String::new();
    
    for line in header_section.lines() {
        if line.starts_with(' ') || line.starts_with('\t') {
            // Folded header - append to current
            current_header_value.push(' ');
            current_header_value.push_str(line.trim());
            continue;
        }
        
        // Save previous header
        if !current_header_key.is_empty() {
            let key_lower = current_header_key.to_lowercase();
            headers_map.entry(key_lower.clone()).or_insert_with(Vec::new).push(current_header_value.clone());
            
            // Process known headers
            match key_lower.as_str() {
                "message-id" => message_id = current_header_value.trim().trim_matches(|c| c == '<' || c == '>').to_string(),
                "from" => {
                    if from_addr.is_empty() {
                        from_addr = extract_email(&current_header_value);
                        from_display = extract_display_name(&current_header_value).map(|n| decode_mime_word(&n));
                    }
                }
                "x-from" => {
                    if from_display.is_none() {
                        let name = clean_exchange_name(&current_header_value);
                        if !name.is_empty() { from_display = Some(decode_mime_word(&name)); }
                    }
                }
                "to" => {
                    for (email, name) in extract_address_list_with_names(&current_header_value) {
                        if !to_addrs.contains(&email) {
                            to_addrs.push(email);
                            if let Some(n) = name {
                                to_display_names.push(decode_mime_word(&n));
                            }
                        }
                    }
                }
                "x-to" => {
                    x_to_header = Some(current_header_value.clone());
                    for (email, name) in extract_address_list_with_names(&current_header_value) {
                        if !to_addrs.contains(&email) { to_addrs.push(email); }
                        if let Some(n) = name {
                            let decoded = decode_mime_word(&n);
                            if !to_display_names.contains(&decoded) { to_display_names.push(decoded); }
                        }
                    }
                }
                "cc" => {
                    for (email, name) in extract_address_list_with_names(&current_header_value) {
                        if !cc_addrs.contains(&email) {
                            cc_addrs.push(email);
                            if let Some(n) = name {
                                cc_display_names.push(decode_mime_word(&n));
                            }
                        }
                    }
                }
                "xcc" => {
                    x_cc_header = Some(current_header_value.clone());
                    for (email, name) in extract_address_list_with_names(&current_header_value) {
                        if !cc_addrs.contains(&email) { cc_addrs.push(email); }
                        if let Some(n) = name {
                            let decoded = decode_mime_word(&n);
                            if !cc_display_names.contains(&decoded) { cc_display_names.push(decoded); }
                        }
                    }
                }
                "bcc" => bcc_addrs = extract_address_list(&current_header_value),
                "subject" => {
                    subject_raw = Some(current_header_value.clone());
                    subject = Some(decode_mime_word(&current_header_value));
                }
                "date" => date_sent = parse_date(&current_header_value),
                "content-type" => content_type = current_header_value.clone(),
                "x-folder" => folder_raw = Some(current_header_value.clone()),
                "received" => received_chain.push(current_header_value.clone()),
                "return-path" => return_path = Some(current_header_value.clone()),
                "reply-to" => reply_to = Some(extract_email(&current_header_value)),
                "x-mailer" => x_mailer = Some(current_header_value.clone()),
                "x-originating-ip" => {
                    let ip = extract_ip(&current_header_value);
                    if !ip.is_empty() { x_originating_ip = Some(ip); }
                }
                "importance" => importance = Some(current_header_value.clone()),
                "x-priority" => importance = Some(current_header_value.clone()),
                "in-reply-to" => in_reply_to = Some(current_header_value.trim().trim_matches(|c| c == '<' || c == '>').to_string()),
                "references" => {
                    for word in current_header_value.split_whitespace() {
                        let clean = word.trim_matches(|c| c == '<' || c == '>');
                        if !clean.is_empty() && !references.contains(&clean.to_string()) {
                            references.push(clean.to_string());
                        }
                    }
                }
                _ => {}
            }
        }
        
        // Start new header
        current_header_value.clear();
        if let Some((key, value)) = line.split_once(':') {
            current_header_key = key.trim().to_string();
            current_header_value = value.trim().to_string();
        } else {
            current_header_key.clear();
        }
    }
    
    // Don't forget the last header
    if !current_header_key.is_empty() {
        let key_lower = current_header_key.to_lowercase();
        headers_map.entry(key_lower.clone()).or_insert_with(Vec::new).push(current_header_value.clone());
        match key_lower.as_str() {
            "received" => received_chain.push(current_header_value.clone()),
            "return-path" => return_path = Some(current_header_value.clone()),
            _ => {}
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
    
    // Build headers JSON
    let headers_json = serde_json::to_string(&headers_map).unwrap_or_else(|_| "{}".to_string());
    
    // Parse body and attachments
    let (body_text, body_html, attachments) = if content_type.starts_with("multipart/") {
        let boundary = extract_boundary(&content_type);
        if let Some(boundary) = boundary {
            parse_multipart(body, &boundary)
        } else {
            (Some(body.to_string()), None, vec![])
        }
    } else if content_type.contains("text/html") {
        (None, Some(body.to_string()), vec![])
    } else if content_type.contains("text/plain") || content_type.is_empty() {
        let text = if body.trim().is_empty() { None } else { Some(body.to_string()) };
        (text, None, vec![])
    } else {
        (Some(body.to_string()), None, vec![])
    };
    
    // Categorize email based on X-Folder header
    let (folder_name, folder_category, recovery_status) = match &folder_raw {
        Some(path) => {
            let lower = path.to_lowercase();
            let (category, recovery) = if lower.contains("sent") {
                ("sent", "normal")
            } else if lower.contains("deleted") {
                ("soft_deleted", "soft_deleted")
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
        to_display_names,
        cc_addrs,
        cc_display_names,
        bcc_addrs,
        folder_name,
        folder_category,
        recovery_status,
        subject,
        subject_raw,
        date_sent,
        headers_raw: header_section.to_string(),  // COMPLETE headers - no truncation!
        headers_json,
        body_text,
        body_html,
        raw_size: size,
        raw_offset: offset,
        attachments,  // NOW properly populated!
        warnings,
        received_chain,
        return_path,
        reply_to,
        x_mailer,
        x_originating_ip,
        importance,
        in_reply_to,
        references,
        x_to_header,
        x_cc_header,
    })
}

fn clean_raw_email(s: &str) -> String {
    s.trim_matches(|c: char| c == '<' || c == '>' || c == '"' || c == '\'' || c == ',' || c == ';' || c == ' ')
     .trim()
     .to_string()
}

pub fn extract_email(s: &str) -> String {
    let s = s.trim();
    if s.is_empty() { return String::new(); }

    // Check for standard <email@domain.com>
    if let Some(start) = s.find('<') {
        if let Some(end) = s.rfind('>') {
            if end > start {
                let inside = s[start + 1..end].trim();
                if !inside.is_empty() {
                    let cleaned = clean_raw_email(inside);
                    if cleaned.contains('@') {
                        return cleaned;
                    }
                }
            }
        }
    }

    // Check if it's an Exchange DN: /O=ENRON/OU=NA/CN=RECIPIENTS/CN=SWHITE
    if s.contains("/O=") || s.contains("CN=") || s.contains("/o=") || s.contains("cn=") {
        if let Some(cn_idx) = s.to_uppercase().rfind("CN=") {
            let name = &s[cn_idx + 3..];
            let name = name.split(';').next().unwrap_or(name).split('/').next().unwrap_or(name).trim();
            let name_clean = name.trim_matches(|c| c == '<' || c == '>' || c == '"' || c == '\'');
            if !name_clean.is_empty() {
                return format!("{}@enron.com", name_clean.to_lowercase());
            }
        }
    }

    // Try to find email pattern user@domain
    if let Some(at_idx) = s.find('@') {
        let before = &s[..at_idx];
        let after = &s[at_idx + 1..];

        let user_start = before
            .rfind(|c: char| c == ' ' || c == '<' || c == '"' || c == ':' || c == ',')
            .map(|i| i + 1)
            .unwrap_or(0);
        let user = before[user_start..].trim();

        let domain_end = after
            .find(|c: char| c == ' ' || c == '>' || c == '"' || c == ';' || c == ',' || c == '\r' || c == '\n')
            .unwrap_or(after.len());
        let domain = after[..domain_end].trim();

        if !user.is_empty() && !domain.is_empty() {
            let candidate = format!("{}@{}", user, domain);
            return clean_raw_email(&candidate);
        }
    }

    clean_raw_email(s)
}

/// Extract display name from Exchange DN or standard name string
pub fn extract_display_name(s: &str) -> Option<String> {
    let s = s.trim();
    if s.is_empty() { return None; }

    // Standard "Name <email>"
    if let Some(start) = s.find('<') {
        let name_part = s[..start].trim();
        if !name_part.is_empty() {
            if let Some(cleaned) = clean_display_name_str(name_part) {
                return Some(cleaned);
            }
        }
    }

    // Exchange DN: extract human name from CN
    if s.contains("/O=") || s.contains("CN=") || s.contains("OU=") {
        if let Some(cn_idx) = s.to_uppercase().rfind("CN=") {
            let name = &s[cn_idx + 3..];
            let name = name.split(';').next().unwrap_or(name).split('/').next().unwrap_or(name).trim();
            if let Some(cleaned) = clean_display_name_str(name) {
                return Some(cleaned);
            }
        }
    }

    clean_display_name_str(s)
}

/// Clean a display name string (remove IMCEANOTES, @ENRON, convert "Last, First" to "First Last")
pub fn clean_display_name_str(s: &str) -> Option<String> {
    let mut name = s
        .trim_matches(|c| c == '<' || c == '>' || c == '"' || c == '\'' || c == ';' || c == ',')
        .trim()
        .to_string();

    if name.is_empty() { return None; }

    // Remove IMCEANOTES- and @ENRON suffix
    if let Some(idx) = name.find("IMCEANOTES-") {
        name = name[..idx].trim().to_string();
    }
    if let Some(idx) = name.find("@ENRON") {
        name = name[..idx].trim().to_string();
    }
    if let Some(idx) = name.find("@enron") {
        name = name[..idx].trim().to_string();
    }

    // If it has "Last, First", convert to "First Last"
    if name.contains(',') && !name.contains('@') {
        let parts: Vec<&str> = name.split(',').map(|p| p.trim()).collect();
        if parts.len() == 2 && !parts[0].is_empty() && !parts[1].is_empty() {
            name = format!("{} {}", parts[1], parts[0]);
        }
    }

    let trimmed = name
        .trim_matches(|c| c == '<' || c == '>' || c == '"' || c == '\'' || c == ',' || c == ';')
        .trim();

    if trimmed.is_empty() || (trimmed.contains('@') && !trimmed.contains(' ')) {
        return None;
    }

    Some(trimmed.to_string())
}

/// Split address list (e.g. "To: Last, First <email1>, Second <email2>") without breaking on commas inside names
pub fn split_address_list(s: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    let mut bracket_depth = 0;

    let chars: Vec<char> = s.chars().collect();
    let len = chars.len();
    let mut i = 0;

    while i < len {
        let c = chars[i];
        if c == '"' {
            in_quotes = !in_quotes;
            current.push(c);
        } else if c == '<' {
            bracket_depth += 1;
            current.push(c);
        } else if c == '>' {
            if bracket_depth > 0 { bracket_depth -= 1; }
            current.push(c);
        } else if c == ';' {
            if !in_quotes && bracket_depth == 0 {
                let trimmed = current.trim().to_string();
                if !trimmed.is_empty() { parts.push(trimmed); }
                current.clear();
            } else {
                current.push(c);
            }
        } else if c == ',' {
            if in_quotes || bracket_depth > 0 {
                current.push(c);
            } else {
                // If current part does NOT have '@' or '<', and the next chunk before comma contains '<' or '@',
                // this comma is between "Last, First" in a name!
                let has_email_in_current = current.contains('@') || current.contains('<');
                let remaining: String = chars[i+1..].iter().collect();
                let next_delim = remaining.find(|ch| ch == ';' || ch == ',');
                let next_chunk = match next_delim {
                    Some(idx) => &remaining[..idx],
                    None => &remaining,
                };
                let next_has_bracket = next_chunk.contains('<') && next_chunk.contains('>');
                let next_has_at = next_chunk.contains('@');

                if !has_email_in_current && (next_has_bracket || next_has_at) {
                    current.push(c);
                } else {
                    let trimmed = current.trim().to_string();
                    if !trimmed.is_empty() { parts.push(trimmed); }
                    current.clear();
                }
            }
        } else {
            current.push(c);
        }
        i += 1;
    }

    let trimmed = current.trim().to_string();
    if !trimmed.is_empty() { parts.push(trimmed); }
    parts
}

fn clean_exchange_name(s: &str) -> String {
    clean_display_name_str(s).unwrap_or_else(|| s.trim().to_string())
}

fn extract_address_list(s: &str) -> Vec<String> {
    split_address_list(s)
        .into_iter()
        .map(|a| extract_email(&a))
        .filter(|a| !a.is_empty() && a != "unknown@unknown")
        .collect()
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

/// Decode MIME encoded-word syntax: =?charset?encoding?text?=
fn decode_mime_word(s: &str) -> String {
    let mut result = String::new();
    let mut remaining = s;
    
    while let Some(start) = remaining.find("=?") {
        // Add text before encoded word
        result.push_str(&remaining[..start]);
        
        // Find end of encoded word (look for ?=)
        if let Some(end) = remaining[start..].find("?=") {
            let encoded = &remaining[start..start + end + 2];
            let decoded = decode_single_mime_word(encoded);
            result.push_str(&decoded);
            remaining = &remaining[start + end + 2..];
        } else {
            break;
        }
    }
    result.push_str(remaining);
    result.trim().to_string()
}

fn decode_single_mime_word(s: &str) -> String {
    // Format: =?charset?encoding?text?=
    let s = s.trim_start_matches("=?").trim_end_matches("?=");
    let parts: Vec<&str> = s.split('?').collect();
    if parts.len() != 3 { return s.to_string(); }
    
    let _charset = parts[0];
    let encoding = parts[1].to_uppercase();
    let text = parts[2];
    
    if encoding == "B" {
        // Base64 encoded
        let cleaned: String = text.chars().filter(|c| !c.is_whitespace()).collect();
        match base64::Engine::decode(&base64::engine::general_purpose::STANDARD, &cleaned) {
            Ok(bytes) => String::from_utf8_lossy(&bytes).to_string(),
            Err(_) => text.to_string(),
        }
    } else if encoding == "Q" {
        // Quoted-printable encoded
        let bytes = qp_decode(text);
        String::from_utf8_lossy(&bytes).to_string()
    } else {
        text.to_string()
    }
}

/// Extract list of (email, display_name) pairs
fn extract_address_list_with_names(s: &str) -> Vec<(String, Option<String>)> {
    split_address_list(s).into_iter().filter_map(|part| {
        let part = part.trim();
        if part.is_empty() { return None; }
        let email = extract_email(part);
        if email.is_empty() { return None; }
        let name = extract_display_name(part);
        Some((email, name))
    }).collect()
}

/// Extract IP address from header value
fn extract_ip(s: &str) -> String {
    // Look for pattern like [x.x.x.x] or just x.x.x.x
    if let Some(start) = s.find('[') {
        if let Some(end) = s[start..].find(']') {
            return s[start+1..start+end].to_string();
        }
    }
    // Try to find IP pattern
    for word in s.split(|c: char| c == ' ' || c == '\t' || c == '(' || c == ')' || c == ',') {
        let word = word.trim();
        if word.split('.').count() == 4 && word.chars().all(|c| c.is_digit(10) || c == '.') {
            return word.to_string();
        }
    }
    String::new()
}