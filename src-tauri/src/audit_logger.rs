use chrono::Utc;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;

/// Get standard case directory path
pub fn get_case_dir(case_id: &str) -> PathBuf {
    let base = crate::db::Database::get_data_dir()
        .join("cases")
        .join(case_id);
    let _ = fs::create_dir_all(&base);
    base
}

/// Get the path to the immutable forensic audit log for a case
pub fn get_case_audit_log_path(case_id: &str) -> PathBuf {
    get_case_dir(case_id).join("forensic_audit_trail.log")
}

/// Append a cryptographically verifiable forensic audit event to the case log on disk
pub fn log_forensic_event(
    case_id: &str,
    module: &str,
    action: &str,
    actor: &str,
    evidence_id: Option<&str>,
    hash: Option<&str>,
    details: &str,
) {
    if case_id.is_empty() { return; }
    let log_path = get_case_audit_log_path(case_id);
    let timestamp = Utc::now().to_rfc3339();
    let ev_str = evidence_id.unwrap_or("N/A");
    let hash_str = hash.unwrap_or("N/A");

    let entry = format!(
        "[{}] [MODULE: {}] [ACTION: {}] [ACTOR: {}] [EVIDENCE_ID: {}] [HASH: {}] {}\n",
        timestamp, module, action, actor, ev_str, hash_str, details
    );

    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(&log_path) {
        let _ = file.write_all(entry.as_bytes());
        let _ = file.flush();
    }
}
