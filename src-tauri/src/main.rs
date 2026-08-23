use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::sync::Mutex;

mod db;
mod models;
mod commands;
mod parser;
mod pst;
mod analysis;

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
            commands::target_profile,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
