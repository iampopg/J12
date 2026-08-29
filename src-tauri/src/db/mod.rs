use std::path::PathBuf;
use rusqlite::{Connection, Result};
use sha2::{Sha256, Sha512, Digest};
use std::fs;
use std::io::Read;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use dirs;

pub mod schema;
pub mod migrations;

pub fn parse_dt(s: &str) -> DateTime<Utc> { 
    s.parse().unwrap_or_else(|_| Utc::now()) 
}

pub struct Database {
    pub conn: Connection,
}

impl Database {
    pub fn new() -> Self {
        let data_dir = Self::get_data_dir();
        std::fs::create_dir_all(&data_dir).ok();
        let db_path = data_dir.join("forensic.db");
        let conn = Connection::open(&db_path).expect("Failed to open database");
        let _ = conn.execute_batch("
            PRAGMA journal_mode = WAL;
            PRAGMA synchronous = NORMAL;
            PRAGMA foreign_keys = ON;
            PRAGMA busy_timeout = 5000;
            PRAGMA cache_size = -64000;
        ");
        let _: Result<String> = conn.query_row("PRAGMA integrity_check;", [], |row| row.get(0));
        let mut db = Database { conn };
        db.init_schema();
        db
    }

    pub fn get_data_dir() -> PathBuf {
        if let Some(base_data) = dirs::data_dir() {
            let primary = base_data.join("j12-forensic");
            let legacy = base_data.join("email-forensic");
            if !primary.exists() {
                if legacy.exists() {
                    let _ = std::fs::rename(&legacy, &primary);
                } else {
                    let _ = std::fs::create_dir_all(&primary);
                }
            } else if legacy.exists() {
                let primary_db = primary.join("forensic.db");
                let legacy_db = legacy.join("forensic.db");
                if legacy_db.exists() && (!primary_db.exists() || primary_db.metadata().map(|m| m.len()).unwrap_or(0) < 100_000) {
                    let _ = std::fs::copy(&legacy_db, &primary_db);
                }
            }
            primary
        } else {
            PathBuf::from("./data")
        }
    }

    #[allow(dead_code)]
    pub fn with_path(path: &PathBuf) -> Self {
        let conn = Connection::open(path).expect("Failed to open database");
        let mut db = Database { conn };
        db.init_schema();
        db
    }

    fn init_schema(&mut self) {
        schema::init_tables_and_indexes(&self.conn);
        migrations::run_migrations_and_triggers(&self.conn);
    }
}

pub fn compute_sha256(path: &PathBuf) -> Result<String, std::io::Error> {
    let mut file = fs::File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 8192];
    loop {
        let count = file.read(&mut buffer)?;
        if count == 0 { break; }
        hasher.update(&buffer[..count]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

pub fn compute_sha512(path: &PathBuf) -> Result<String, std::io::Error> {
    let mut file = fs::File::open(path)?;
    let mut hasher = Sha512::new();
    let mut buffer = [0u8; 8192];
    loop {
        let count = file.read(&mut buffer)?;
        if count == 0 { break; }
        hasher.update(&buffer[..count]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

pub fn detect_format(filename: &str) -> String {
    let lower = filename.to_lowercase();
    let path_buf = PathBuf::from(&lower);
    let ext = path_buf
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");
    match ext {
        "eml" => return "eml".to_string(),
        "mbox" => return "mbox".to_string(),
        "msg" => return "msg".to_string(),
        "pst" => return "pst".to_string(),
        "ost" => return "ost".to_string(),
        "emlx" => return "emlx".to_string(),
        "dat" => return "tnef".to_string(),
        _ => {}
    }
    if lower.contains("mbox") || lower.contains("inbox") || lower.contains("sent") || lower.contains("mailbox") {
        return "mbox".to_string();
    }
    "unknown".to_string()
}

pub fn generate_id() -> String {
    Uuid::new_v4().to_string()
}
