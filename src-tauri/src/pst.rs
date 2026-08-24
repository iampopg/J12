//! PST/OST format parser module.
//! 
//! This module provides parsing for Outlook PST and OST files.
//! Uses libpff via FFI for low-level access.
//! 
//! For now, provides the interface structure. Full libpff integration
//! requires building the C library and generating FFI bindings.

use std::path::Path;
use crate::parser::RawEmail;

/// PST parser - handles Outlook Personal Folders (.pst) and Offline Folders (.ost)
pub struct PstParser;

impl PstParser {
    /// Check if this parser can handle the given file
    pub fn can_parse(path: &Path) -> bool {
        if let Some(ext) = path.extension() {
            let ext = ext.to_string_lossy().to_lowercase();
            return ext == "pst" || ext == "ost";
        }
        false
    }

    /// Parse a PST/OST file and return all emails
    pub fn parse(path: &Path) -> Result<Vec<RawEmail>, String> {
        // TODO: Integrate with libpff for actual PST/OST parsing
        // For now, return an error indicating PST support requires libpff
        Err(
            "PST/OST parsing requires libpff library.\n\
             To enable PST support:\n\
             1. Install libpff: brew install libpff (macOS) or build from source\n\
             2. Rebuild with PST support enabled\n\
             \n\
             For now, export your PST to MBOX format using Outlook or libpff tools.".to_string()
        )
    }

    /// Get folder hierarchy from PST/OST
    pub fn get_folder_hierarchy(path: &Path) -> Result<Vec<PstFolder>, String> {
        Err("PST folder browsing requires libpff library".to_string())
    }

    /// Recover deleted items from PST/OST
    pub fn recover_deleted(path: &Path) -> Result<Vec<RawEmail>, String> {
        Err("PST deleted recovery requires libpff library".to_string())
    }
}

/// Represents a folder in a PST/OST file
#[derive(Debug, Clone)]
pub struct PstFolder {
    pub id: String,
    pub name: String,
    pub parent_id: Option<String>,
    pub message_count: u32,
    pub subfolders: Vec<PstFolder>,
}

/// MAPI property tags for MSG and PST files
pub mod mapi {
    // Standard MAPI property tags (PidTag*)
    pub const PID_TAG_SUBJECT: u32 = 0x0037;
    pub const PID_TAG_SENDER_EMAIL_ADDRESS: u32 = 0x0C1F;
    pub const PID_TAG_SENDER_NAME: u32 = 0x0C1A;
    pub const PID_TAG_SENT_REPRESENTING_EMAIL_ADDRESS: u32 = 0x0042;
    pub const PID_TAG_SENT_REPRESENTING_NAME: u32 = 0x0041;
    pub const PID_TAG_RECEIVED_BY_EMAIL_ADDRESS: u32 = 0x0076;
    pub const PID_TAG_RECEIVED_BY_NAME: u32 = 0x0075;
    pub const PID_TAG_CLIENT_SUBMIT_TIME: u32 = 0x0039;
    pub const PID_TAG_MESSAGE_DELIVERY_TIME: u32 = 0x0E06;
    pub const PID_TAG_LAST_MODIFICATION_TIME: u32 = 0x3008;
    pub const PID_TAG_CREATION_TIME: u32 = 0x3007;
    pub const PID_TAG_MESSAGE_FLAGS: u32 = 0x0E07;
    pub const PID_TAG_INTERNET_MESSAGE_ID: u32 = 0x1035;
    pub const PID_TAG_INTERNET_REFERENCES: u32 = 0x1039;
    pub const PID_TAG_TRANSPORT_MESSAGE_HEADERS: u32 = 0x007D;
    pub const PID_TAG_BODY: u32 = 0x1000;
    pub const PID_TAG_BODY_HTML: u32 = 0x1013;
    pub const PID_TAG_RTF_COMPRESSED: u32 = 0x1009;
    pub const PID_TAG_HAS_ATTACHMENTS: u32 = 0x0E1B;
    pub const PID_TAG_MESSAGE_SIZE: u32 = 0x0E08;
    pub const PID_TAG_IMPORTANCE: u32 = 0x0017;
    pub const PID_TAG_PRIORITY: u32 = 0x0026;
    pub const PID_TAG_SENSITIVITY: u32 = 0x0036;
}

/// Parse a file based on its format and return emails
pub fn parse_file(path: &Path) -> Result<Vec<RawEmail>, String> {
    if let Some(ext) = path.extension() {
        let ext = ext.to_string_lossy().to_lowercase();
        match ext.as_str() {
            "eml" => crate::parser::parse_eml(path),
            "mbox" => crate::parser::parse_mbox(path),
            "msg" => parse_msg(path),
            "emlx" => parse_emlx(path),
            "pst" | "ost" => PstParser::parse(path),
            _ => Err(format!("Unsupported format: {}", ext)),
        }
    } else {
        Err("Cannot determine file format".to_string())
    }
}

/// Parse MSG file (Outlook item format)
pub fn parse_msg(path: &Path) -> Result<Vec<RawEmail>, String> {
    // MSG files are OLE Compound Document Format
    // For now, provide basic structure - full MSG parsing requires CFB/OLE parser
    Err(
        "MSG parsing requires CFB/OLE parser.\n\
         MSG files use Microsoft's Compound Document Format.\n\
         Full MSG support coming in Phase 2.".to_string()
    )
}

/// Parse EMLX file (Apple Mail format)
pub fn parse_emlx(path: &Path) -> Result<Vec<RawEmail>, String> {
    // EMLX files contain plist metadata + RFC822 message
    use std::fs;
    let content = fs::read_to_string(path).map_err(|e| format!("Read error: {}", e))?;
    
    // Find the RFC822 part (after the plist)
    let mut warnings = Vec::new();
    let mut message_id = String::new();
    let mut from_addr = String::new();
    let mut from_display = None;
    let mut to_addrs = Vec::new();
    let mut subject = None;
    let mut date_sent = None;
    let mut body_text = None;
    
    // Simple EMLX parsing: extract the email portion
    let email_part = if content.contains("\r\n\r\n") {
        content.split("\r\n\r\n").nth(1).unwrap_or("").to_string()
    } else if content.contains("\n\n") {
        content.split("\n\n").nth(1).unwrap_or("").to_string()
    } else {
        content.clone()
    };
    
    // Parse headers from the email portion
    for line in email_part.lines() {
        if line.starts_with(' ') || line.starts_with('\t') {
            continue;
        }
        if line.is_empty() {
            break; // End of headers
        }
        if let Some((key, value)) = line.split_once(':') {
            let key = key.trim().to_lowercase();
            let value = value.trim();
            match key.as_str() {
                "message-id" => message_id = value.trim_matches(|c| c == '<' || c == '>').to_string(),
                "from" => {
                    from_addr = value.to_string();
                    if from_addr.contains('<') && from_addr.contains('>') {
                        if let Some(start) = from_addr.find('<') {
                            if let Some(end) = from_addr.find('>') {
                                from_display = Some(from_addr[..start].trim().trim_matches('"').to_string());
                                from_addr = from_addr[start+1..end].to_string();
                            }
                        }
                    }
                }
                "to" => to_addrs.push(value.to_string()),
                "subject" => subject = Some(value.to_string()),
                "date" => {
                    if let Ok(dt) = chrono::DateTime::parse_from_rfc2822(value) {
                        date_sent = Some(dt.with_timezone(&chrono::Utc));
                    }
                }
                _ => {}
            }
        }
    }
    
    if message_id.is_empty() {
        message_id = format!("emlx_{}", uuid::Uuid::new_v4());
        warnings.push("Missing Message-ID".to_string());
    }
    
    // Extract body (after first blank line)
    let body = if let Some(idx) = email_part.find("\r\n\r\n") {
        email_part[idx+4..].to_string()
    } else if let Some(idx) = email_part.find("\n\n") {
        email_part[idx+2..].to_string()
    } else {
        String::new()
    };
    
    if !body.is_empty() {
        body_text = Some(body);
    }
    
    let email = RawEmail {
        message_id,
        from_addr,
        from_display,
        to_addrs,
        cc_addrs: Vec::new(),
        bcc_addrs: Vec::new(),
        to_display_names: Vec::new(),
        cc_display_names: Vec::new(),
        folder_name: None,
        folder_category: "other".to_string(),
        recovery_status: "normal".to_string(),
        subject,
        subject_raw: None,
        date_sent,
        headers_raw: email_part.lines().take(50).collect::<Vec<_>>().join("\n"),
        headers_json: "{}".to_string(),
        body_text,
        body_html: None,
        raw_size: std::fs::metadata(path).map(|m| m.len()).unwrap_or(0),
        raw_offset: 0,
        attachments: Vec::new(),
        warnings,
        received_chain: Vec::new(),
        return_path: None,
        reply_to: None,
        x_mailer: None,
        x_originating_ip: None,
        importance: None,
        in_reply_to: None,
        references: Vec::new(),
        x_to_header: None,
        x_cc_header: None,
    };
    
    Ok(vec![email])
}