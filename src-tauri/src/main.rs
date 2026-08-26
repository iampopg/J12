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
mod ai;

use db::Database;
use models::*;
use analysis::{AnalysisResult, analyze_headers, analyze_authentication, detect_spoofing, generate_findings, calculate_risk_score};
use ai::*;

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
            commands::get_case_email_count,
            commands::generate_report_data,
            commands::export_report_pdf,
            commands::verify_evidence_hashes,
            commands::export_audit_log,
            commands::check_custody_chain,
            commands::imap_list_mailboxes,
            commands::imap_fetch_emails,
            commands::imap_cancel_acquisition,
            commands::pop3_test_connection,
            commands::pop3_fetch_emails,
            commands::case_attachments_summary,
            commands::case_attachments_list,
            commands::export_attachment,
            commands::get_attachment_preview,
            commands::open_attachment_in_system,
            commands::reveal_in_finder,
            commands::get_email_inline_images,
            commands::bookmark_add,
            commands::bookmark_remove,
            commands::bookmarks_list,
            commands::bookmark_check,
            commands::case_artifacts_summary,
            commands::case_artifacts_list,
            commands::rescan_case_artifacts,
            commands::open_external_url,
            // AI Commands (Phase 0 + Phase 1)
            ai_get_case_statistics,
            ai_search_emails,
            ai_get_email,
            ai_get_authentication_results,
            ai_get_entity,
            ai_get_timeline,
            ai_get_findings,
            ai_get_case_context,
            ai_create_session,
            ai_get_session_history,
            ai_clear_session,
            ai_natural_language_search,
            ai_explain_evidence,
            ai_create_investigation_plan,
            ai_execute_investigation_plan,
            ai_analyze_timeline,
            ai_analyze_spoofing,
            ai_triage_attachments,
            ai_analyze_graph,
            fetch_kiloai_models,
            fetch_openrouter_models,
            ai_chat,
            ai_resolve_entities,
            ai_detect_anomalies,
            ai_generate_report,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
