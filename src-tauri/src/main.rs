use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::sync::Mutex;

mod db;
mod models;
mod commands;
mod parser;
mod pst;
mod analysis;
mod imap_acquisition;

use db::Database;
use models::*;
use analysis::{AnalysisResult, analyze_headers, analyze_authentication, detect_spoofing, generate_findings, calculate_risk_score};

pub struct AppState {
    db: Mutex<Database>,
    data_dir: PathBuf,
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .manage(AppState {
            db: Mutex::new(Database::new()),
            data_dir: PathBuf::new(),
        })
        .invoke_handler(tauri::generate_handler![
            commands::case_create,
            commands::case_list,
            commands::case_get,
            commands::case_update,
            commands::case_delete,
            commands::evidence_upload,
            commands::evidence_list,
            commands::evidence_status,
            commands::write_temp_file,
            commands::open_file_dialog,
            commands::read_file,
            commands::parse_evidence,
            commands::email_list,
            commands::email_get,
            commands::email_headers,
            commands::search,
            commands::findings_list,
            commands::dashboard,
            commands::custody_chain,
            commands::run_analysis,
            commands::update_finding_status,
            commands::add_finding_note,
            commands::finding_emails,
            commands::target_profile,
            commands::auto_detect_targets,
            commands::email_attachments,
            commands::advanced_search,
            commands::extract_entities,
            commands::entity_list,
            commands::entity_dive,
            commands::entity_emails,
            commands::entity_heatmap,
            commands::extract_entities,
            commands::emails_by_date,
            commands::emails_between,
            commands::timeline_data,
            commands::graph_data,
            commands::case_notes_list,
            commands::case_note_create,
            commands::case_note_update,
            commands::case_note_toggle_pin,
            commands::case_note_delete,
            commands::email_tags_list,
            commands::email_tag_add,
            commands::email_tag_remove,
            commands::email_notes_list,
            commands::email_note_add,
            commands::email_note_delete,
            commands::generate_report_data,
            commands::export_report_pdf,
            commands::verify_evidence_hashes,
            commands::export_audit_log,
            commands::check_custody_chain,
            commands::imap_list_mailboxes,
            commands::imap_fetch_emails,
            commands::case_attachments_list,
            commands::export_attachment,
            commands::case_artifacts_summary,
            commands::case_artifacts_list,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
