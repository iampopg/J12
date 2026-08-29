use rusqlite::Connection;

pub fn run_migrations_and_triggers(conn: &Connection) {
    // Migration: add target_email column if missing
    conn.execute("ALTER TABLE cases ADD COLUMN target_email TEXT", []).ok();
    // Migration: add target_name column if missing
    conn.execute("ALTER TABLE cases ADD COLUMN target_name TEXT", []).ok();
    // Migration: add target_organization column if missing
    conn.execute("ALTER TABLE cases ADD COLUMN target_organization TEXT", []).ok();
    // Migration: add investigation_type column if missing
    conn.execute("ALTER TABLE cases ADD COLUMN investigation_type TEXT DEFAULT 'general'", []).ok();
    // Migration: add working_dir column if missing
    conn.execute("ALTER TABLE cases ADD COLUMN working_dir TEXT", []).ok();
    // Migration: add folder_name column if missing
    conn.execute("ALTER TABLE emails ADD COLUMN folder_name TEXT", []).ok();
    // Migration: add folder_category column if missing
    conn.execute("ALTER TABLE emails ADD COLUMN folder_category TEXT DEFAULT 'other'", []).ok();
    // Migration: add recovery_status column if missing
    conn.execute("ALTER TABLE emails ADD COLUMN recovery_status TEXT DEFAULT 'normal'", []).ok();
    // Migration: add case_id to audit_log if missing
    conn.execute("ALTER TABLE audit_log ADD COLUMN case_id TEXT", []).ok();
    // Migration: add reviewed_by column to findings if missing
    conn.execute("ALTER TABLE findings ADD COLUMN reviewed_by TEXT", []).ok();
    // Migration: add reviewed_at column to findings if missing
    conn.execute("ALTER TABLE findings ADD COLUMN reviewed_at TEXT", []).ok();
    // Migration: add notes column to findings if missing
    conn.execute("ALTER TABLE findings ADD COLUMN notes TEXT", []).ok();
    // Migration: add aliases column to entities if missing
    conn.execute("ALTER TABLE entities ADD COLUMN aliases TEXT", []).ok();
    // Migration: ensure email_notes has both note and content columns
    conn.execute("ALTER TABLE email_notes ADD COLUMN note TEXT", []).ok();
    conn.execute("ALTER TABLE email_notes ADD COLUMN content TEXT", []).ok();
    conn.execute("UPDATE email_notes SET note = content WHERE note IS NULL AND content IS NOT NULL", []).ok();

    // Migration: ensure attachments has extracted_text, ocr_status, created_at
    conn.execute("ALTER TABLE attachments ADD COLUMN extracted_text TEXT", []).ok();
    conn.execute("ALTER TABLE attachments ADD COLUMN ocr_status TEXT DEFAULT 'pending'", []).ok();
    conn.execute("ALTER TABLE attachments ADD COLUMN created_at TEXT DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))", []).ok();

    // Migration: ensure emails has created_at, msg_references, received_chain, flags
    conn.execute("ALTER TABLE emails ADD COLUMN created_at TEXT DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))", []).ok();
    conn.execute("ALTER TABLE emails ADD COLUMN msg_references TEXT DEFAULT '[]'", []).ok();
    conn.execute("ALTER TABLE emails ADD COLUMN received_chain TEXT DEFAULT '[]'", []).ok();
    conn.execute("ALTER TABLE emails ADD COLUMN flags TEXT DEFAULT '[]'", []).ok();

    // Migration: ensure evidence_items has created_at, deleted_recovered
    conn.execute("ALTER TABLE evidence_items ADD COLUMN created_at TEXT DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))", []).ok();
    conn.execute("ALTER TABLE evidence_items ADD COLUMN deleted_recovered INTEGER DEFAULT 0", []).ok();

    // Forensic Integrity: Immutable audit log triggers (ISSUE-122)
    conn.execute("
        CREATE TRIGGER IF NOT EXISTS trg_audit_log_no_delete
        BEFORE DELETE ON audit_log
        BEGIN
            SELECT RAISE(ABORT, 'Audit log records are immutable and cannot be deleted.');
        END;
    ", []).ok();

    conn.execute("
        CREATE TRIGGER IF NOT EXISTS trg_audit_log_no_update
        BEFORE UPDATE ON audit_log
        BEGIN
            SELECT RAISE(ABORT, 'Audit log records are immutable and cannot be modified.');
        END;
    ", []).ok();

    // Forensic Integrity Check on Startup (ISSUE-125)
    if let Ok(mut stmt) = conn.prepare("PRAGMA integrity_check(1)") {
        if let Ok(mut rows) = stmt.query([]) {
            if let Ok(Some(row)) = rows.next() {
                let status: String = row.get(0).unwrap_or_else(|_| "ok".to_string());
                if status != "ok" {
                    eprintln!("[FORENSIC INTEGRITY WARNING] SQLite PRAGMA integrity_check failed: {}", status);
                }
            }
        }
    }

    migrate_folder_categories(conn);
}

fn migrate_folder_categories(conn: &Connection) {
    let mut stmt = match conn.prepare(
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
        
        conn.execute(
            "UPDATE emails SET folder_category = ?1, folder_name = ?2 WHERE id = ?3",
            [category, folder.as_deref().unwrap_or(""), &id],
        ).ok();
    }
}
