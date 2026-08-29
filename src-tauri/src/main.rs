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
pub mod ai;
pub mod audit_logger;
pub mod bip39_wordlist;

use db::Database;

pub struct AppState {
    pub db: Mutex<Database>,
    pub data_dir: PathBuf,
    pub cancel_imap: std::sync::Arc<std::sync::atomic::AtomicBool>,
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .manage(AppState {
            db: Mutex::new(Database::new()),
            data_dir: PathBuf::new(),
            cancel_imap: std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
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
            commands::evidence_delete,
            commands::write_temp_file,
            commands::open_file_dialog,
            commands::open_folder_dialog,
            commands::open_forensic_logs_folder,
            commands::get_case_audit_trail,
            commands::read_file,
            commands::parse_evidence,
            commands::email_list,
            commands::email_get,
            commands::email_headers,
            commands::search,
            commands::fts_search,
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
            commands::imap_cancel_acquisition,
            commands::imap_device_flow_start,
            commands::imap_device_flow_poll,
            commands::case_attachments_summary,
            commands::case_attachments_list,
            commands::export_attachment,
            commands::get_attachment_preview,
            commands::extract_attachment_text,
            commands::batch_extract_case_attachments,
            commands::open_attachment_in_system,
            commands::reveal_in_finder,
            commands::case_artifacts_summary,
            commands::case_artifacts_list,
            commands::rescan_case_artifacts,
            commands::open_external_url,
            commands::bookmark_add,
            commands::bookmark_remove,
            commands::bookmarks_list,
            commands::bookmark_check,
            commands::pop3_test_connection,
            commands::pop3_fetch_emails,
            // AI Subsystem Commands (ISSUE-114)
            ai::fetch_kiloai_models,
            ai::fetch_openrouter_models,
            ai::ai_chat,
            ai::ai_get_case_statistics,
            ai::ai_search_emails,
            ai::ai_get_email,
            ai::ai_get_authentication_results,
            ai::ai_get_entity,
            ai::ai_get_timeline,
            ai::ai_get_findings,
            ai::ai_get_case_context,
            ai::ai_create_session,
            ai::ai_get_session_history,
            ai::ai_clear_session,
            ai::ai_natural_language_search,
            ai::ai_explain_evidence,
            ai::ai_create_investigation_plan,
            ai::ai_execute_investigation_plan,
            ai::ai_analyze_timeline,
            ai::ai_analyze_spoofing,
            ai::ai_triage_attachments,
            ai::ai_analyze_graph,
            ai::ai_resolve_entities,
            ai::ai_detect_anomalies,
            ai::ai_generate_report,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
