use chrono::{DateTime, Utc};
use sha2::{Sha256, Digest};

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
    pub is_inline: bool,
}

/// Compute SHA-256 hash of data
pub fn sha256_data(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    format!("{:x}", hasher.finalize())
}
