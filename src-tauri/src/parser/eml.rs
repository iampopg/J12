use std::fs;
use std::path::Path;

use crate::db::generate_id;
use super::types::RawEmail;
use super::headers::*;
use super::mime::{base64_decode, decode_mime_word, extract_boundary, parse_multipart, qp_decode};

/// Parse EML file (single RFC 5322 message)
pub fn parse_eml(path: &Path) -> Result<Vec<RawEmail>, String> {
    let data = fs::read(path).map_err(|e| format!("Read error: {}", e))?;
    let content = String::from_utf8_lossy(&data);
    let email = parse_rfc5322(&content, 0, data.len() as u64)?;
    Ok(vec![email])
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
    let mut content_transfer_encoding = String::new();
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
                "content-transfer-encoding" => content_transfer_encoding = current_header_value.trim().to_lowercase(),
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
    
    let headers_json = serde_json::to_string(&headers_map).unwrap_or_else(|_| "{}".to_string());
    
    let (body_text, body_html, attachments) = if content_type.starts_with("multipart/") {
        let boundary = extract_boundary(&content_type);
        if let Some(boundary) = boundary {
            parse_multipart(body, &boundary)
        } else {
            (Some(body.to_string()), None, vec![])
        }
    } else {
        let decoded_bytes = if content_transfer_encoding.contains("base64") {
            base64_decode(body.trim())
        } else if content_transfer_encoding.contains("quoted-printable") || body.contains("=3D") || body.contains("=21") || body.contains("=20\n") || body.contains("=\r\n") {
            qp_decode(body)
        } else {
            body.as_bytes().to_vec()
        };
        let decoded_str = String::from_utf8_lossy(&decoded_bytes).to_string();

        if content_type.contains("text/html") {
            (None, Some(decoded_str), vec![])
        } else if content_type.contains("text/plain") || content_type.is_empty() {
            let text = if decoded_str.trim().is_empty() { None } else { Some(decoded_str) };
            (text, None, vec![])
        } else {
            (Some(decoded_str), None, vec![])
        }
    };
    
    let (folder_name, folder_category, recovery_status) = match &folder_raw {
        Some(path) => {
            let lower = path.to_lowercase();
            let (category, recovery) = if lower.contains("sent") {
                ("sent", "normal")
            } else if lower.contains("deleted") || lower.contains("trash") || lower.contains("bin") {
                ("soft_deleted", "soft_deleted")
            } else if lower.contains("draft") {
                ("drafts", "normal")
            } else if lower.contains("important") {
                ("important", "normal")
            } else if lower.contains("starred") || lower.contains("flagged") {
                ("starred", "normal")
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
        headers_raw: header_section.to_string(),
        headers_json,
        body_text,
        body_html,
        raw_size: size,
        raw_offset: offset,
        attachments,
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
