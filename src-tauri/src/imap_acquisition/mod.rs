pub mod client;
pub mod stream;
pub mod oauth;

use serde::{Deserialize, Serialize};

pub use stream::{fetch_emails_streaming, list_mailboxes};
pub use oauth::*;

#[derive(Debug, Clone)]
pub struct ImapConfig {
    pub server: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub auth_type: String,
    pub access_token: Option<String>,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamingMessage {
    pub folder_name: String,
    pub folder_category: String,
    pub seq_id: u32,
    pub folder_total: u32,
    pub folder_index: usize,
    pub total_folders: usize,
    pub overall_seq: u32,
    pub overall_total: u32,
    pub raw_content: String,
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
    } else if lower.contains("important") {
        "important".to_string()
    } else if lower.contains("starred") || lower.contains("flagged") {
        "starred".to_string()
    } else if lower.contains("inbox") {
        "inbox".to_string()
    } else {
        "other".to_string()
    }
}
