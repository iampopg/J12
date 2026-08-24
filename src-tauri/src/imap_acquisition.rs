//! IMAP email acquisition module
//! Connects to IMAP servers and downloads emails for forensic analysis.
//!
//! NOTE: This module requires the `imap` crate which has API differences between versions.
//! The implementation below provides the structure but may need API adjustments based on
//! the exact version of the `imap` crate being used.

use std::path::Path;

use crate::db::generate_id;
use crate::parser::parse_rfc5322;

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
pub struct ImapAcquisitionResult {
    pub total_found: u32,
    pub downloaded: u32,
    pub errors: u32,
    pub messages: Vec<String>,
}

/// List available mailboxes
pub fn list_mailboxes(_config: &ImapConfig) -> Result<Vec<String>, String> {
    // TODO: Implement with correct imap crate API
    // 1. Connect to server
    // 2. Login with credentials
    // 3. List all mailboxes
    Err("IMAP not yet implemented - requires imap crate API adjustments".to_string())
}

/// Fetch emails from IMAP server
pub fn fetch_emails(_config: &ImapConfig, _max_messages: Option<u32>) -> Result<ImapAcquisitionResult, String> {
    // TODO: Implement with correct imap crate API
    // 1. Connect to server
    // 2. Select mailbox
    // 3. Fetch messages by sequence
    // 4. Parse each message with parse_rfc5322
    Err("IMAP not yet implemented - requires imap crate API adjustments".to_string())
}

/// Save raw email to evidence store
pub fn save_imap_email(
    evidence_id: &str,
    case_id: &str,
    raw_email: &str,
    message_id: &str,
) -> Result<String, String> {
    // Create storage directory
    let storage_dir = dirs::data_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("j12-forensic")
        .join("evidence")
        .join(case_id)
        .join("imap")
        .join(evidence_id);
    
    std::fs::create_dir_all(&storage_dir).map_err(|e| format!("Create dir: {}", e))?;
    
    // Save with message_id as filename
    let safe_id = message_id.replace(|c: char| !c.is_alphanumeric() && c != '-' && c != '_', "_");
    let filename = format!("{}.eml", safe_id);
    let filepath = storage_dir.join(&filename);
    
    std::fs::write(&filepath, raw_email).map_err(|e| format!("Write file: {}", e))?;
    
    Ok(filepath.to_string_lossy().to_string())
}