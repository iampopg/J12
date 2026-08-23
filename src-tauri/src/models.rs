use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Case {
    pub id: String,
    pub title: String,
    pub case_number: String,
    pub description: String,
    pub status: String,
    pub owner_id: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EvidenceItem {
    pub id: String,
    pub case_id: String,
    pub filename: String,
    pub original_path: String,
    pub stored_path: String,
    pub format: String,
    pub sha256: String,
    pub sha512: Option<String>,
    pub size_bytes: u64,
    pub source_description: String,
    pub acquired_by: String,
    pub acquired_at: DateTime<Utc>,
    pub acquisition_method: String,
    pub integrity_level: String,
    pub parse_status: String,
    pub parse_error: Option<String>,
    pub message_count: u32,
    pub deleted_recovered: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EmailMessage {
    pub id: String,
    pub evidence_id: String,
    pub case_id: String,
    pub message_id: Option<String>,
    pub from_addr: String,
    pub from_display: Option<String>,
    pub to_addrs: String,
    pub cc_addrs: String,
    pub subject: Option<String>,
    pub date_sent: Option<String>,
    pub date_sent_utc: Option<String>,
    pub headers_raw: Option<String>,
    pub body_text: Option<String>,
    pub body_html: Option<String>,
    pub folder_name: Option<String>,
    pub folder_category: String,
    pub is_deleted: bool,
    pub deleted_recovered: bool,
    pub risk_score: u8,
    pub flags: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Attachment {
    pub id: String,
    pub email_id: String,
    pub filename: Option<String>,
    pub sha256: String,
    pub mime_type: Option<String>,
    pub size_bytes: u64,
    pub stored_path: String,
    pub entropy: Option<f64>,
    pub risk_flags: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CustodyEvent {
    pub id: String,
    pub evidence_id: String,
    pub action: String,
    pub actor: String,
    pub timestamp: DateTime<Utc>,
    pub tool: String,
    pub tool_version: String,
    pub hash_before: Option<String>,
    pub hash_after: Option<String>,
    pub detail: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Finding {
    pub id: String,
    pub case_id: String,
    pub type_: String,
    pub severity: String,
    pub confidence: String,
    pub title: String,
    pub description: Option<String>,
    pub evidence_refs: String,
    pub email_ids: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub reviewed_by: Option<String>,
    pub reviewed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DashboardData {
    pub evidence_count: u32,
    pub email_count: u32,
    pub deleted_recovered: u32,
    pub entity_count: u32,
    pub finding_count: u32,
    pub severity_breakdown: std::collections::HashMap<String, u32>,
    pub date_range: (Option<String>, Option<String>),
    pub top_correspondents: Vec<TopCorrespondent>,
    pub sent_count: u32,
    pub inbox_count: u32,
    pub soft_deleted_count: u32,
    pub drafts_count: u32,
    pub spam_count: u32,
    pub other_count: u32,
    pub high_risk_emails: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TopCorrespondent {
    pub email: String,
    pub sent: u32,
    pub received: u32,
}

#[derive(Debug, Deserialize)]
pub struct CaseCreateInput {
    pub title: String,
    pub case_number: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct EvidenceUploadInput {
    pub case_id: String,
    pub file_path: String,
    pub source_description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct EmailListInput {
    pub case_id: String,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
    pub from_filter: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SearchInput {
    pub case_id: String,
    pub query: String,
    pub limit: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct EmptyInput {
    pub case_id: String,
}
