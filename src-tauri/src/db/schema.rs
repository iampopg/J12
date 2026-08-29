use rusqlite::Connection;

pub fn init_tables_and_indexes(conn: &Connection) {
    conn.execute_batch("
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
            working_dir TEXT,
            created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
            updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
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
            acquired_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
            acquisition_method TEXT NOT NULL,
            integrity_level TEXT NOT NULL,
            parse_status TEXT DEFAULT 'pending',
            parse_error TEXT,
            message_count INTEGER DEFAULT 0,
            deleted_recovered INTEGER DEFAULT 0,
            created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
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
            bcc_addrs TEXT DEFAULT '[]',
            to_display_names TEXT DEFAULT '[]',
            cc_display_names TEXT DEFAULT '[]',
            subject TEXT,
            subject_raw TEXT,
            date_sent TEXT,
            date_sent_utc TEXT,
            headers_raw TEXT,
            headers_json TEXT,
            body_text TEXT,
            body_html TEXT,
            folder_name TEXT,
            folder_category TEXT DEFAULT 'other',
            recovery_status TEXT DEFAULT 'normal',
            is_deleted INTEGER DEFAULT 0,
            deleted_recovered INTEGER DEFAULT 0,
            risk_score INTEGER DEFAULT 0,
            flags TEXT DEFAULT '[]',
            received_chain TEXT DEFAULT '[]',
            return_path TEXT,
            reply_to TEXT,
            x_mailer TEXT,
            x_originating_ip TEXT,
            importance TEXT,
            in_reply_to TEXT,
            msg_references TEXT DEFAULT '[]',
            x_to_header TEXT,
            x_cc_header TEXT,
            created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
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
            extracted_text TEXT,
            ocr_status TEXT DEFAULT 'pending',
            created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
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
            created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
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
            aliases TEXT,
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

        CREATE TABLE IF NOT EXISTS case_notes (
            id TEXT PRIMARY KEY,
            case_id TEXT NOT NULL REFERENCES cases(id),
            author TEXT NOT NULL,
            title TEXT NOT NULL,
            content TEXT NOT NULL,
            category TEXT DEFAULT 'general',
            pinned INTEGER DEFAULT 0,
            created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
            updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
        );

        CREATE TABLE IF NOT EXISTS email_tags (
            id TEXT PRIMARY KEY,
            case_id TEXT NOT NULL REFERENCES cases(id),
            email_id TEXT NOT NULL REFERENCES emails(id),
            tag TEXT NOT NULL,
            color TEXT DEFAULT '#3b82f6',
            created_by TEXT NOT NULL,
            created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
            UNIQUE(case_id, email_id, tag)
        );

        CREATE TABLE IF NOT EXISTS item_bookmarks (
            id TEXT PRIMARY KEY,
            case_id TEXT NOT NULL,
            item_id TEXT NOT NULL,
            item_type TEXT NOT NULL,
            label TEXT,
            color TEXT,
            note TEXT,
            created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
            UNIQUE(case_id, item_id)
        );

        CREATE TABLE IF NOT EXISTS email_notes (
            id TEXT PRIMARY KEY,
            case_id TEXT NOT NULL REFERENCES cases(id),
            email_id TEXT NOT NULL REFERENCES emails(id),
            author TEXT NOT NULL,
            note TEXT,
            content TEXT,
            created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
            updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
        );

        CREATE TABLE IF NOT EXISTS chain_of_custody (
            id TEXT PRIMARY KEY,
            case_id TEXT NOT NULL REFERENCES cases(id),
            evidence_id TEXT,
            action TEXT NOT NULL,
            performed_by TEXT NOT NULL,
            timestamp TEXT NOT NULL,
            notes TEXT
        );

        CREATE TABLE IF NOT EXISTS artifacts_cache (
            id TEXT PRIMARY KEY,
            case_id TEXT NOT NULL REFERENCES cases(id),
            domain_id TEXT NOT NULL,
            subcategory_id TEXT NOT NULL,
            title TEXT NOT NULL,
            primary_value TEXT NOT NULL,
            secondary_value TEXT,
            details TEXT,
            severity TEXT DEFAULT 'info',
            artifact_type TEXT DEFAULT 'native',
            email_id TEXT,
            email_subject TEXT,
            email_from TEXT,
            date_sent_utc TEXT,
            created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
        );

        CREATE TABLE IF NOT EXISTS forensic_artifacts (
            id TEXT PRIMARY KEY,
            case_id TEXT NOT NULL REFERENCES cases(id),
            domain_id TEXT NOT NULL,
            subcategory_id TEXT NOT NULL,
            title TEXT NOT NULL,
            primary_value TEXT NOT NULL,
            secondary_value TEXT,
            details TEXT,
            severity TEXT NOT NULL,
            artifact_type TEXT NOT NULL,
            confidence TEXT,
            email_id TEXT NOT NULL,
            email_subject TEXT,
            email_from TEXT NOT NULL,
            date_sent_utc TEXT
        );

        CREATE TABLE IF NOT EXISTS ai_sessions (
            id TEXT PRIMARY KEY,
            case_id TEXT NOT NULL REFERENCES cases(id),
            provider TEXT NOT NULL,
            model TEXT NOT NULL,
            created_at TEXT NOT NULL,
            ended_at TEXT
        );

        CREATE TABLE IF NOT EXISTS ai_messages (
            id TEXT PRIMARY KEY,
            session_id TEXT NOT NULL REFERENCES ai_sessions(id),
            role TEXT NOT NULL,
            content TEXT NOT NULL,
            evidence_refs TEXT,
            timestamp TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS ai_tool_calls (
            id TEXT PRIMARY KEY,
            session_id TEXT NOT NULL REFERENCES ai_sessions(id),
            tool_name TEXT NOT NULL,
            arguments TEXT,
            result TEXT,
            timestamp TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS ai_audit_log (
            id TEXT PRIMARY KEY,
            case_id TEXT NOT NULL REFERENCES cases(id),
            action TEXT NOT NULL,
            provider TEXT,
            timestamp TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS ai_context_snapshots (
            id TEXT PRIMARY KEY,
            session_id TEXT NOT NULL REFERENCES ai_sessions(id),
            snapshot_data TEXT NOT NULL,
            timestamp TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS ai_search_index (
            id TEXT PRIMARY KEY,
            case_id TEXT NOT NULL REFERENCES cases(id),
            email_id TEXT NOT NULL REFERENCES emails(id),
            embedding TEXT,
            summary TEXT,
            created_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS ai_entity_resolutions (
            id TEXT PRIMARY KEY,
            case_id TEXT NOT NULL REFERENCES cases(id),
            entity_a TEXT NOT NULL,
            entity_b TEXT NOT NULL,
            confidence REAL NOT NULL,
            rationale TEXT,
            created_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS ai_investigation_plans (
            id TEXT PRIMARY KEY,
            case_id TEXT NOT NULL REFERENCES cases(id),
            objective TEXT NOT NULL,
            plan_json TEXT NOT NULL,
            status TEXT DEFAULT 'draft',
            created_at TEXT NOT NULL
        );
    ").expect("Failed to initialize schema");

    // Indexes
    let _ = conn.execute("CREATE INDEX IF NOT EXISTS idx_forensic_artifacts_case ON forensic_artifacts(case_id)", []);
    let _ = conn.execute("CREATE INDEX IF NOT EXISTS idx_forensic_artifacts_dom ON forensic_artifacts(case_id, domain_id)", []);
    let _ = conn.execute("CREATE INDEX IF NOT EXISTS idx_forensic_artifacts_sub ON forensic_artifacts(case_id, subcategory_id)", []);
    let _ = conn.execute("CREATE INDEX IF NOT EXISTS idx_emails_case_id ON emails(case_id)", []);
    let _ = conn.execute("CREATE INDEX IF NOT EXISTS idx_emails_case_evidence ON emails(case_id, evidence_id)", []);
    let _ = conn.execute("CREATE INDEX IF NOT EXISTS idx_emails_from_addr ON emails(from_addr)", []);
    let _ = conn.execute("CREATE INDEX IF NOT EXISTS idx_emails_date_sent ON emails(date_sent_utc)", []);
    let _ = conn.execute("CREATE INDEX IF NOT EXISTS idx_emails_folder ON emails(folder_category)", []);
    let _ = conn.execute("CREATE INDEX IF NOT EXISTS idx_emails_evidence_id ON emails(evidence_id)", []);
    let _ = conn.execute("CREATE INDEX IF NOT EXISTS idx_emails_subject ON emails(subject)", []);
    let _ = conn.execute("CREATE INDEX IF NOT EXISTS idx_emails_message_id ON emails(message_id)", []);
    let _ = conn.execute("CREATE INDEX IF NOT EXISTS idx_findings_case_id ON findings(case_id)", []);
    let _ = conn.execute("CREATE INDEX IF NOT EXISTS idx_findings_severity ON findings(severity)", []);
    let _ = conn.execute("CREATE INDEX IF NOT EXISTS idx_entities_case_id ON entities(case_id)", []);
    let _ = conn.execute("CREATE INDEX IF NOT EXISTS idx_entities_email ON entities(email_address)", []);
    let _ = conn.execute("CREATE INDEX IF NOT EXISTS idx_timeline_case_id ON timeline_events(case_id)", []);
    let _ = conn.execute("CREATE INDEX IF NOT EXISTS idx_timeline_timestamp ON timeline_events(timestamp)", []);
    let _ = conn.execute("CREATE INDEX IF NOT EXISTS idx_custody_evidence_id ON custody_events(evidence_id)", []);
    let _ = conn.execute("CREATE INDEX IF NOT EXISTS idx_evidence_case_id ON evidence_items(case_id)", []);
    let _ = conn.execute("CREATE INDEX IF NOT EXISTS idx_case_notes_case_id ON case_notes(case_id)", []);
    let _ = conn.execute("CREATE INDEX IF NOT EXISTS idx_email_tags_case_id ON email_tags(case_id)", []);
    let _ = conn.execute("CREATE INDEX IF NOT EXISTS idx_email_notes_case_id ON email_notes(case_id)", []);
    let _ = conn.execute("CREATE INDEX IF NOT EXISTS idx_communication_edges_case_id ON communication_edges(case_id)", []);
    let _ = conn.execute("CREATE INDEX IF NOT EXISTS idx_attachments_email_id ON attachments(email_id)", []);
    let _ = conn.execute("CREATE INDEX IF NOT EXISTS idx_attachments_sha256 ON attachments(sha256)", []);
    let _ = conn.execute("CREATE INDEX IF NOT EXISTS idx_audit_log_case_id ON audit_log(case_id)", []);
    let _ = conn.execute("CREATE INDEX IF NOT EXISTS idx_audit_log_target ON audit_log(target_type, target_id)", []);
    let _ = conn.execute("CREATE INDEX IF NOT EXISTS idx_ai_sessions_case ON ai_sessions(case_id)", []);
    let _ = conn.execute("CREATE INDEX IF NOT EXISTS idx_ai_messages_session ON ai_messages(session_id)", []);

    // Migrations for existing databases
    let _ = conn.execute("ALTER TABLE attachments ADD COLUMN extracted_text TEXT", []);
    let _ = conn.execute("ALTER TABLE attachments ADD COLUMN ocr_status TEXT DEFAULT 'pending'", []);

    // SQLite FTS5 Full-Text Search Engine Setup (Porter Stemmer + Unicode61)
    let _ = conn.execute("
        CREATE VIRTUAL TABLE IF NOT EXISTS emails_fts USING fts5(
            email_id UNINDEXED,
            case_id UNINDEXED,
            subject,
            from_addr,
            to_addrs,
            body_text,
            attachment_text,
            tokenize = 'porter unicode61'
        );
    ", []);

    // Automatic FTS5 Synchronization Triggers
    let _ = conn.execute("
        CREATE TRIGGER IF NOT EXISTS emails_fts_ai AFTER INSERT ON emails BEGIN
            INSERT INTO emails_fts(email_id, case_id, subject, from_addr, to_addrs, body_text, attachment_text)
            VALUES (new.id, new.case_id, coalesce(new.subject, ''), coalesce(new.from_addr, ''), coalesce(new.to_addrs, ''), coalesce(new.body_text, ''), '');
        END;
    ", []);

    let _ = conn.execute("
        CREATE TRIGGER IF NOT EXISTS emails_fts_ad AFTER DELETE ON emails BEGIN
            DELETE FROM emails_fts WHERE email_id = old.id;
        END;
    ", []);

    let _ = conn.execute("
        CREATE TRIGGER IF NOT EXISTS emails_fts_au AFTER UPDATE OF subject, from_addr, to_addrs, body_text ON emails BEGIN
            DELETE FROM emails_fts WHERE email_id = old.id;
            INSERT INTO emails_fts(email_id, case_id, subject, from_addr, to_addrs, body_text, attachment_text)
            VALUES (new.id, new.case_id, coalesce(new.subject, ''), coalesce(new.from_addr, ''), coalesce(new.to_addrs, ''), coalesce(new.body_text, ''), '');
        END;
    ", []);

    // Backfill any unindexed emails into FTS5
    let _ = conn.execute("
        INSERT INTO emails_fts(email_id, case_id, subject, from_addr, to_addrs, body_text, attachment_text)
        SELECT id, case_id, coalesce(subject, ''), coalesce(from_addr, ''), coalesce(to_addrs, ''), coalesce(body_text, ''), ''
        FROM emails WHERE id NOT IN (SELECT email_id FROM emails_fts);
    ", []);
}
