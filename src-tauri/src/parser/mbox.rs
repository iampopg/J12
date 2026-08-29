use std::fs;
use std::path::Path;

use super::types::RawEmail;
use super::eml::parse_rfc5322;

/// Parse MBOX file (concatenated RFC 5322 messages)
/// Efficient: reads file once, splits on "From " lines
pub fn parse_mbox(path: &Path) -> Result<Vec<RawEmail>, String> {
    let data = fs::read(path).map_err(|e| format!("Read error: {}", e))?;
    let content = String::from_utf8_lossy(&data);
    let mut emails = Vec::new();
    
    let mut current_msg = String::new();
    let mut msg_offset: u64 = 0;
    let mut line_start: u64 = 0;
    
    for line in content.lines() {
        if line.starts_with("From ") && !current_msg.trim().is_empty() {
            let msg_len = line_start - msg_offset;
            match parse_rfc5322(&current_msg, msg_offset, msg_len) {
                Ok(email) => emails.push(email),
                Err(e) => {
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
    
    if !current_msg.trim().is_empty() {
        let msg_len = content.len() as u64 - msg_offset;
        match parse_rfc5322(&current_msg, msg_offset, msg_len) {
            Ok(email) => emails.push(email),
            Err(e) => eprintln!("Warning: skipped last message: {}", e),
        }
    }
    
    Ok(emails)
}
