use std::path::PathBuf;
use rusqlite::{Connection, Result};
use sha2::{Sha256, Sha512, Digest};
use std::fs;
use std::io::Read;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use dirs;

pub fn parse_dt(s: &str) -> DateTime<Utc> { s.parse().unwrap_or_else(|_| Utc::now()) }

pub struct Database {
    pub conn: Connection,
}

impl Database {
    pub fn new() -> Self {
        // Use file-based database in user data directory
        let data_dir = Self::get_data_dir();
        std::fs::create_dir_all(&data_dir).ok();
        let db_path = data_dir.join("forensic.db");
        let conn = Connection::open(&db_path).expect("Failed to open database");
        let mut db = Database { conn };
        db.init_schema();
        db
    }

    fn get_data_dir() -> PathBuf {
        if let Some(data_dir) = dirs::data_dir() {
            data_dir.join("email-forensic")
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
        self.conn.execute_batch("
            CREATE TABLE IF NOT EXISTS cases (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                case_number TEXT,
                description TEXT,
                status TEXT DEFAULT 'open',
                owner_id TEXT,
                target_email TEXT,
                target_name TEXT,
                target_organization TEXT,
                investigation_type TEXT DEFAULT 'general',
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS evidence_items (
                id TEXT PRIMARY KEY,
                case_id TEXT NOT NULL REFERENCES cases(id),
                filename TEXT NOT NULL,
                original_path TEXT NOT NULL,
                stored_path TEXT NOT NULL,
                format TEXT NOT NULL,
                sha256 TEXT NOT NULL,
                sha512 TEXT,
                size_bytes INTEGER NOT NULL,
                source_description TEXT,
                acquired_by TEXT,
                acquired_at TEXT NOT NULL,
                acquisition_method TEXT NOT NULL,
                integrity_level TEXT NOT NULL,
                parse_status TEXT DEFAULT 'pending',
                parse_error TEXT,
                message_count INTEGER DEFAULT 0,
                deleted_recovered INTEGER DEFAULT 0,
                created_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS emails (
                id TEXT PRIMARY KEY,
                evidence_id TEXT NOT NULL REFERENCES evidence_items(id),
                case_id TEXT NOT NULL REFERENCES cases(id),
                message_id TEXT,
                from_addr TEXT NOT NULL,
                from_display TEXT,
                to_addrs TEXT NOT NULL DEFAULT '[]',
                cc_addrs TEXT DEFAULT '[]',
                subject TEXT,
                date_sent TEXT,
                date_sent_utc TEXT,
                headers_raw TEXT,
                body_text TEXT,
                body_html TEXT,
                folder_name TEXT,
                folder_category TEXT DEFAULT 'other',
                recovery_status TEXT DEFAULT 'normal',
                is_deleted INTEGER DEFAULT 0,
                deleted_recovered INTEGER DEFAULT 0,
                risk_score INTEGER DEFAULT 0,
                flags TEXT DEFAULT '[]',
                created_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS attachments (
                id TEXT PRIMARY KEY,
                email_id TEXT NOT NULL REFERENCES emails(id),
                filename TEXT,
                sha256 TEXT NOT NULL,
                mime_type TEXT,
                size_bytes INTEGER NOT NULL,
                stored_path TEXT,
                entropy REAL,
                risk_flags TEXT DEFAULT '[]',
                created_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS custody_events (
                id TEXT PRIMARY KEY,
                evidence_id TEXT NOT NULL REFERENCES evidence_items(id),
                action TEXT NOT NULL,
                actor TEXT NOT NULL,
                timestamp TEXT NOT NULL,
                tool TEXT NOT NULL,
                tool_version TEXT NOT NULL,
                hash_before TEXT,
                hash_after TEXT,
                detail TEXT
            );

            CREATE TABLE IF NOT EXISTS audit_log (
                id TEXT PRIMARY KEY,
                actor TEXT NOT NULL,
                action TEXT NOT NULL,
                target_type TEXT,
                target_id TEXT,
                timestamp TEXT NOT NULL,
                detail TEXT
            );

            CREATE TABLE IF NOT EXISTS findings (
                id TEXT PRIMARY KEY,
                case_id TEXT NOT NULL REFERENCES cases(id),
                type TEXT NOT NULL,
                severity TEXT NOT NULL,
                confidence TEXT NOT NULL,
                title TEXT NOT NULL,
                description TEXT,
                evidence_refs TEXT DEFAULT '[]',
                email_ids TEXT DEFAULT '[]',
                status TEXT DEFAULT 'open',
                created_at TEXT NOT NULL,
                reviewed_by TEXT,
                reviewed_at TEXT,
                notes TEXT
            );

            CREATE TABLE IF NOT EXISTS entities (
                id TEXT PRIMARY KEY,
                case_id TEXT NOT NULL REFERENCES cases(id),
                email_address TEXT NOT NULL,
                display_name TEXT,
                first_seen TEXT,
                last_seen TEXT,
                sent_count INTEGER DEFAULT 0,
                received_count INTEGER DEFAULT 0,
                role TEXT DEFAULT 'unknown',
                UNIQUE(case_id, email_address)
            );

            CREATE TABLE IF NOT EXISTS communication_edges (
                id TEXT PRIMARY KEY,
                case_id TEXT NOT NULL REFERENCES cases(id),
                from_entity TEXT NOT NULL,
                to_entity TEXT NOT NULL,
                message_count INTEGER DEFAULT 0,
                first_seen TEXT,
                last_seen TEXT,
                UNIQUE(case_id, from_entity, to_entity)
            );

            CREATE TABLE IF NOT EXISTS timeline_events (
                id TEXT PRIMARY KEY,
                case_id TEXT NOT NULL REFERENCES cases(id),
                evidence_id TEXT NOT NULL,
                email_id TEXT,
                event_type TEXT NOT NULL,
                timestamp TEXT NOT NULL,
                actor TEXT,
                summary TEXT
            );
        "        ).expect("Failed to initialize schema");
        
        // Migration: add target_email column if missing
        self.conn.execute("ALTER TABLE cases ADD COLUMN target_email TEXT", []).ok();
        // Migration: add target_name column if missing
        self.conn.execute("ALTER TABLE cases ADD COLUMN target_name TEXT", []).ok();
        // Migration: add target_organization column if missing
        self.conn.execute("ALTER TABLE cases ADD COLUMN target_organization TEXT", []).ok();
        // Migration: add investigation_type column if missing
        self.conn.execute("ALTER TABLE cases ADD COLUMN investigation_type TEXT DEFAULT 'general'", []).ok();
        // Migration: add folder_name column if missing
        self.conn.execute("ALTER TABLE emails ADD COLUMN folder_name TEXT", []).ok();
        // Migration: add folder_category column if missing
        self.conn.execute("ALTER TABLE emails ADD COLUMN folder_category TEXT DEFAULT 'other'", []).ok();
        // Migration: add recovery_status column if missing
        self.conn.execute("ALTER TABLE emails ADD COLUMN recovery_status TEXT DEFAULT 'normal'", []).ok();
        // Migration: add reviewed_by column to findings if missing
        self.conn.execute("ALTER TABLE findings ADD COLUMN reviewed_by TEXT", []).ok();
        // Migration: add reviewed_at column to findings if missing
        self.conn.execute("ALTER TABLE findings ADD COLUMN reviewed_at TEXT", []).ok();
        // Migration: add notes column to findings if missing
        self.conn.execute("ALTER TABLE findings ADD COLUMN notes TEXT", []).ok();
        
        // Migration: update existing emails with folder_category from headers_raw X-Folder
        self.migrate_folder_categories();
    }
    
    fn migrate_folder_categories(&mut self) {
        // Get all emails with folder_category = 'other' and headers_raw containing X-Folder
        let mut stmt = match self.conn.prepare(
            "SELECT id, headers_raw FROM emails WHERE folder_category = 'other' AND headers_raw LIKE '%X-Folder:%'"
        ) {
            Ok(s) => s,
            Err(_) => return,
        };
        
        let rows: Vec<(String, String)> = match stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        }) {
            Ok(r) => r.filter_map(|r| r.ok()).collect(),
            Err(_) => return,
        };
        
        for (id, headers) in rows {
            // Extract X-Folder value
            let folder = headers.lines()
                .find(|l| l.starts_with("X-Folder:"))
                .and_then(|l| l.split_once(':'))
                .map(|(_, v)| v.trim().to_lowercase());
            
            let category = match folder.as_deref() {
                Some(f) if f.contains("sent") => "sent",
                Some(f) if f.contains("deleted") => "soft_deleted",
                Some(f) if f.contains("draft") => "drafts",
                Some(f) if f.contains("inbox") => "inbox",
                Some(f) if f.contains("junk") || f.contains("spam") => "spam",
                _ => "other",
            };
            
            self.conn.execute(
                "UPDATE emails SET folder_category = ?1, folder_name = ?2 WHERE id = ?3",
                [category, folder.as_deref().unwrap_or(""), &id],
            ).ok();
        }
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
