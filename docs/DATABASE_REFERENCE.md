# J12 Forensic - Complete Data Structure Reference

> **Database Location:** `~/Library/Application Support/email-forensic/forensic.db`
> **Engine:** SQLite 3 (WAL mode)
> **File Size:** 518 MB (main) + 15 MB (WAL) + 32 KB (SHM)
> **Page Size:** 4,096 bytes
> **Total Pages:** 132,545
> **Encoding:** UTF-8
> **Total Tables:** 25
> **Total Indexes:** 38 (25 custom + 13 autoindex)
> **Total Rust Structs:** 75+
> **Total Rust Enums:** 3
> **Total Command Inputs:** 15+

---

## Table of Contents

1. [Database Configuration](#database-configuration)
2. [Database Files](#database-files)
3. [Entity Relationship Diagram](#entity-relationship-diagram)
4. [Core Database Tables](#core-database-tables)
5. [Chain of Custody & Audit Tables](#chain-of-custody--audit-tables)
6. [Analysis & Findings Tables](#analysis--findings-tables)
7. [Artifacts Tables](#artifacts-tables)
8. [Notes, Tags & Bookmarks Tables](#notes-tags--bookmarks-tables)
9. [AI Database Tables](#ai-database-tables)
10. [Index Reference](#index-reference)
11. [Foreign Key Relationships](#foreign-key-relationships)
12. [Unique Constraints](#unique-constraints)
13. [Row Counts](#row-counts)
14. [Database Migrations](#database-migrations)
15. [Rust Data Structures (In-Memory)](#rust-data-structures-in-memory)
16. [Command Inputs & Outputs](#command-inputs--outputs)
17. [AI Data Structures](#ai-data-structures)
18. [Analysis Data Structures](#analysis-data-structures)
19. [Artifact Taxonomy](#artifact-taxonomy)
20. [IMAP/POP3 Acquisition](#imapimap-pop3-acquisition)
21. [Parser Data Structures](#parser-data-structures)
22. [Bookmark Data Structures](#bookmark-data-structures)

---

## Database Configuration

| Setting | Value | Description |
|---------|-------|-------------|
| `journal_mode` | WAL | Write-Ahead Logging for concurrent reads |
| `synchronous` | 2 (NORMAL) | Balance between safety and performance |
| `foreign_keys` | 0 (OFF) | Enforce referential integrity (disabled at file level) |
| `encoding` | UTF-8 | Character encoding |
| `page_size` | 4096 | Database page size in bytes |
| `cache_size` | -2000 | Cache size in KB (2000 KB) |
| `auto_vacuum` | 0 (OFF) | Auto vacuum mode |
| `schema_version` | 72 | Internal schema version |
| `user_version` | 0 | User-defined version |

---

## Database Files

| File | Size | Description |
|------|------|-------------|
| `forensic.db` | 518 MB | Main database file |
| `forensic.db-wal` | 15 MB | Write-Ahead Log (pending writes) |
| `forensic.db-shm` | 32 KB | Shared memory file (WAL index) |

---

## Entity Relationship Diagram

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│                              J12 FORENSIC DATABASE                              │
├─────────────────────────────────────────────────────────────────────────────────┤
│                                                                                 │
│  ┌──────────┐     ┌──────────────────┐     ┌─────────────────┐                 │
│  │  cases   │────▶│  evidence_items  │────▶│     emails      │                 │
│  └────┬─────┘     └────────┬─────────┘     └────────┬────────┘                 │
│       │                    │                        │                          │
│       │    ┌───────────────┼────────────────────────┼──────────────────┐       │
│       │    │               │                        │                  │       │
│       ▼    ▼               ▼                        ▼                  ▼       │
│  ┌─────────────┐  ┌──────────────┐  ┌─────────────────┐  ┌───────────────┐    │
│  │ case_notes  │  │attachments   │  │ forensic_artifacts│ │ email_notes   │    │
│  └─────────────┘  └──────────────┘  └─────────────────┘  └───────────────┘    │
│                                                                                 │
│  ┌─────────────┐  ┌──────────────────┐  ┌─────────────────┐                   │
│  │  findings   │  │    entities      │  │ email_tags      │                   │
│  └─────────────┘  └──────────────────┘  └─────────────────┘                   │
│                                                                                 │
│  ┌─────────────────────┐  ┌─────────────────────┐  ┌─────────────────────┐     │
│  │ communication_edges │  │  timeline_events    │  │   item_bookmarks    │     │
│  └─────────────────────┘  └─────────────────────┘  └─────────────────────┘     │
│                                                                                 │
│  ┌─────────────────────────────────────────────────────────────────────────┐   │
│  │                         AI TABLES                                        │   │
│  │  ┌────────────┐  ┌────────────┐  ┌────────────┐  ┌────────────────┐    │   │
│  │  │ai_sessions │─▶│ai_messages │─▶│ai_tool_calls│  │ai_audit_log    │    │   │
│  │  └────────────┘  └────────────┘  └────────────┘  └────────────────┘    │   │
│  │       │                │                                                  │   │
│  │       ▼                ▼                                                  │   │
│  │  ┌────────────────────┐  ┌─────────────────────┐  ┌─────────────────┐   │   │
│  │  │ai_context_snapshots│  │ai_evidence_citations│  │ai_model_runs    │   │   │
│  │  └────────────────────┘  └─────────────────────┘  └─────────────────┘   │   │
│  │                                                                          │   │
│  │  ┌────────────────────┐                                                  │   │
│  │  │ai_generated_findings│                                                 │   │
│  │  └────────────────────┘                                                  │   │
│  └─────────────────────────────────────────────────────────────────────────┘   │
│                                                                                 │
│  ┌─────────────────────┐  ┌─────────────────────┐                             │
│  │   custody_events    │  │   artifacts_cache   │                             │
│  └─────────────────────┘  └─────────────────────┘                             │
│                                                                                 │
│  ┌─────────────────────┐  ┌─────────────────────┐                             │
│  │   chain_of_custody  │  │     audit_log       │                             │
│  └─────────────────────┘  └─────────────────────┘                             │
│                                                                                 │
└─────────────────────────────────────────────────────────────────────────────────┘
```

---

## Core Database Tables

### cases

**Description:** Top-level container for forensic investigations. Each case represents a single investigation with its own evidence, emails, and findings.

| Column | Type | Constraints | Default | Description |
|--------|------|-------------|---------|-------------|
| `id` | TEXT | PRIMARY KEY | - | Unique case identifier (UUID) |
| `title` | TEXT | NOT NULL | - | Case title/name |
| `case_number` | TEXT | - | - | External case reference number |
| `description` | TEXT | - | - | Case description and scope |
| `status` | TEXT | - | `'open'` | Case status: `open`, `closed`, `archived` |
| `owner_id` | TEXT | - | - | Case owner/investigator ID |
| `target_email` | TEXT | - | - | Target person's email address |
| `target_name` | TEXT | - | - | Target person's name |
| `target_organization` | TEXT | - | - | Target organization |
| `investigation_type` | TEXT | - | `'general'` | Type: `general`, `fraud`, `phishing`, `insider_threat` |
| `working_dir` | TEXT | - | - | Working directory path for case files |
| `created_at` | TEXT | NOT NULL | - | Creation timestamp (ISO 8601) |
| `updated_at` | TEXT | NOT NULL | - | Last update timestamp (ISO 8601) |

**Row Count:** 2

---

### evidence_items

**Description:** Records of evidence files uploaded to the case. Tracks file integrity, acquisition method, and parsing status.

| Column | Type | Constraints | Default | Description |
|--------|------|-------------|---------|-------------|
| `id` | TEXT | PRIMARY KEY | - | Unique evidence identifier (UUID) |
| `case_id` | TEXT | NOT NULL, FK → cases(id) | - | Parent case |
| `filename` | TEXT | NOT NULL | - | Original filename |
| `original_path` | TEXT | NOT NULL | - | Original file location |
| `stored_path` | TEXT | NOT NULL | - | Internal storage path |
| `format` | TEXT | NOT NULL | - | File format: `eml`, `mbox`, `msg`, `pst`, `ost`, `emlx`, `tnef` |
| `sha256` | TEXT | NOT NULL | - | SHA-256 hash of file |
| `sha512` | TEXT | - | - | SHA-512 hash of file |
| `size_bytes` | INTEGER | NOT NULL | - | File size in bytes |
| `source_description` | TEXT | - | - | Description of evidence source |
| `acquired_by` | TEXT | - | - | Person who acquired the evidence |
| `acquired_at` | TEXT | NOT NULL | - | Acquisition timestamp |
| `acquisition_method` | TEXT | NOT NULL | - | Method: `file_import`, `imap`, `pop3`, `forensic_image` |
| `integrity_level` | TEXT | NOT NULL | - | Integrity: `verified`, `tampered`, `unknown` |
| `parse_status` | TEXT | - | `'pending'` | Status: `pending`, `parsing`, `completed`, `error` |
| `parse_error` | TEXT | - | - | Error message if parsing failed |
| `message_count` | INTEGER | - | 0 | Number of emails extracted |
| `deleted_recovered` | INTEGER | - | 0 | Count of recovered deleted emails |
| `created_at` | TEXT | NOT NULL | - | Creation timestamp |

**Row Count:** 3

---

### emails

**Description:** Core table storing all parsed email messages. Contains full headers, body content, and forensic metadata.

| Column | Type | Constraints | Default | Description |
|--------|------|-------------|---------|-------------|
| `id` | TEXT | PRIMARY KEY | - | Unique email identifier (UUID) |
| `evidence_id` | TEXT | NOT NULL, FK → evidence_items(id) | - | Source evidence file |
| `case_id` | TEXT | NOT NULL, FK → cases(id) | - | Parent case |
| `message_id` | TEXT | - | - | RFC 2822 Message-ID header |
| `from_addr` | TEXT | NOT NULL | - | Sender email address |
| `from_display` | TEXT | - | - | Sender display name |
| `to_addrs` | TEXT | NOT NULL | `'[]'` | JSON array of recipient emails |
| `cc_addrs` | TEXT | - | `'[]'` | JSON array of CC emails |
| `bcc_addrs` | TEXT | - | `'[]'` | JSON array of BCC emails |
| `to_display_names` | TEXT | - | `'[]'` | JSON array of recipient display names |
| `cc_display_names` | TEXT | - | `'[]'` | JSON array of CC display names |
| `subject` | TEXT | - | - | Email subject (decoded) |
| `subject_raw` | TEXT | - | - | Raw subject (encoded) |
| `date_sent` | TEXT | - | - | Original date header value |
| `date_sent_utc` | TEXT | - | - | Normalized UTC timestamp |
| `headers_raw` | TEXT | - | - | Complete raw headers |
| `headers_json` | TEXT | - | - | Parsed headers as JSON |
| `body_text` | TEXT | - | - | Plain text body |
| `body_html` | TEXT | - | - | HTML body content |
| `folder_name` | TEXT | - | - | Original folder name (from X-Folder) |
| `folder_category` | TEXT | - | `'other'` | Normalized: `inbox`, `sent`, `drafts`, `spam`, `soft_deleted`, `other` |
| `recovery_status` | TEXT | - | `'normal'` | `normal`, `recovered`, `orphaned` |
| `is_deleted` | INTEGER | - | 0 | Soft delete flag |
| `deleted_recovered` | INTEGER | - | 0 | Recovered from deletion |
| `risk_score` | INTEGER | - | 0 | Calculated risk score (0-100) |
| `flags` | TEXT | - | `'[]'` | JSON array of flags |
| `received_chain` | TEXT | - | `'[]'` | JSON array of Received headers |
| `return_path` | TEXT | - | - | Return-Path header |
| `reply_to` | TEXT | - | - | Reply-To header |
| `x_mailer` | TEXT | - | - | X-Mailer header |
| `x_originating_ip` | TEXT | - | - | X-Originating-IP header |
| `importance` | TEXT | - | - | Importance header |
| `in_reply_to` | TEXT | - | - | In-Reply-To header |
| `msg_references` | TEXT | - | `'[]'` | JSON array of References |
| `x_to_header` | TEXT | - | - | X-To header (Enron specific) |
| `x_cc_header` | TEXT | - | - | X-Cc header (Enron specific) |
| `created_at` | TEXT | NOT NULL | - | Creation timestamp |

**Row Count:** 14,227

---

### attachments

**Description:** Files attached to emails. Includes hash verification and risk analysis.

| Column | Type | Constraints | Default | Description |
|--------|------|-------------|---------|-------------|
| `id` | TEXT | PRIMARY KEY | - | Unique attachment identifier (UUID) |
| `email_id` | TEXT | NOT NULL, FK → emails(id) | - | Parent email |
| `filename` | TEXT | - | - | Attachment filename |
| `sha256` | TEXT | NOT NULL | - | SHA-256 hash of file |
| `mime_type` | TEXT | - | - | MIME type (e.g., `application/pdf`) |
| `size_bytes` | INTEGER | NOT NULL | - | File size in bytes |
| `stored_path` | TEXT | - | - | Internal storage path |
| `entropy` | REAL | - | - | Shannon entropy (randomness measure) |
| `risk_flags` | TEXT | - | `'[]'` | JSON array of risk indicators |
| `extracted_text` | TEXT | - | - | Extracted text from document (PDF, DOCX, etc.) |
| `ocr_status` | TEXT | - | `'pending'` | OCR status: `pending`, `completed`, `failed` |
| `created_at` | TEXT | NOT NULL | - | Creation timestamp |

**Row Count:** 154

---

### emails_fts (FTS5 Virtual Table)

**Description:** Full-text search index for emails. Uses SQLite FTS5 with Porter Stemmer for fast text search. Automatically synchronized with emails table via triggers.

| Column | Type | Constraints | Default | Description |
|--------|------|-------------|---------|-------------|
| `email_id` | TEXT | UNINDEXED | - | Reference to emails.id (not indexed, used for JOIN) |
| `case_id` | TEXT | UNINDEXED | - | Reference to cases.id (for filtering) |
| `subject` | TEXT | - | - | Email subject (indexed, searchable) |
| `from_addr` | TEXT | - | - | Sender address (indexed, searchable) |
| `to_addrs` | TEXT | - | - | Recipients (indexed, searchable) |
| `body_text` | TEXT | - | - | Email body text (indexed, searchable) |
| `attachment_text` | TEXT | - | - | Extracted text from attachments (indexed, searchable) |

**Tokenizer:** `porter unicode61`

**Features:**
- **Porter Stemmer**: Search "transfer" matches "transfers", "transferring", "transferred"
- **Boolean Queries**: `fraud AND (wire OR offshore) NOT payroll`
- **Proximity Search**: `NEAR("wire" "transfer", 5)`
- **Wildcard Search**: `crypt*` matches "crypto", "cryptocurrency"
- **Exact Phrase**: `"strictly confidential"`

**Auto-Sync Triggers:**
| Trigger | Event | Action |
|---------|-------|--------|
| `emails_fts_ai` | AFTER INSERT ON emails | Insert into FTS index |
| `emails_fts_ad` | AFTER DELETE ON emails | Delete from FTS index |
| `emails_fts_au` | AFTER UPDATE ON emails | Update FTS index |

**Performance:** < 5ms on 100,000+ emails (vs 30+ seconds with LIKE)

**Row Count:** 14,227 (synced with emails table)

---

## Chain of Custody & Audit Tables

### custody_events

**Description:** Detailed custody events for evidence items. Tracks every action performed on evidence with hash verification.

| Column | Type | Constraints | Default | Description |
|--------|------|-------------|---------|-------------|
| `id` | TEXT | PRIMARY KEY | - | Unique event identifier (UUID) |
| `evidence_id` | TEXT | NOT NULL, FK → evidence_items(id) | - | Related evidence |
| `action` | TEXT | NOT NULL | - | Action performed |
| `actor` | TEXT | NOT NULL | - | Person/system performing action |
| `timestamp` | TEXT | NOT NULL | - | Event timestamp |
| `tool` | TEXT | NOT NULL | - | Tool used |
| `tool_version` | TEXT | NOT NULL | - | Tool version |
| `hash_before` | TEXT | - | - | Hash before action |
| `hash_after` | TEXT | - | - | Hash after action |
| `detail` | TEXT | - | - | Additional details |

**Row Count:** 2

---

### chain_of_custody

**Description:** Simplified chain of custody log. Tracks evidence handling for legal proceedings.

| Column | Type | Constraints | Default | Description |
|--------|------|-------------|---------|-------------|
| `id` | TEXT | PRIMARY KEY | - | Unique entry identifier (UUID) |
| `case_id` | TEXT | NOT NULL, FK → cases(id) | - | Parent case |
| `evidence_id` | TEXT | - | - | Related evidence (optional) |
| `action` | TEXT | NOT NULL | - | Action performed |
| `performed_by` | TEXT | NOT NULL | - | Person performing action |
| `timestamp` | TEXT | NOT NULL | - | Event timestamp |
| `notes` | TEXT | - | - | Additional notes |

**Row Count:** 13

---

### audit_log

**Description:** General audit trail for all system actions.

| Column | Type | Constraints | Default | Description |
|--------|------|-------------|---------|-------------|
| `id` | TEXT | PRIMARY KEY | - | Unique log entry identifier (UUID) |
| `actor` | TEXT | NOT NULL | - | User/system performing action |
| `action` | TEXT | NOT NULL | - | Action performed |
| `target_type` | TEXT | - | - | Type of target entity |
| `target_id` | TEXT | - | - | ID of target entity |
| `timestamp` | TEXT | NOT NULL | - | Event timestamp |
| `detail` | TEXT | - | - | Additional details |

**Row Count:** 4

---

## Analysis & Findings Tables

### findings

**Description:** Security findings and alerts generated by analysis engines.

| Column | Type | Constraints | Default | Description |
|--------|------|-------------|---------|-------------|
| `id` | TEXT | PRIMARY KEY | - | Unique finding identifier (UUID) |
| `case_id` | TEXT | NOT NULL, FK → cases(id) | - | Parent case |
| `type` | TEXT | NOT NULL | - | Finding type: `FRAUD`, `SPOOF`, `ATTACH`, `ANOMALY` |
| `severity` | TEXT | NOT NULL | - | `critical`, `high`, `medium`, `low` |
| `confidence` | TEXT | NOT NULL | - | Confidence level |
| `title` | TEXT | NOT NULL | - | Finding title |
| `description` | TEXT | - | - | Detailed description |
| `evidence_refs` | TEXT | - | `'[]'` | JSON array of evidence references |
| `email_ids` | TEXT | - | `'[]'` | JSON array of related email IDs |
| `status` | TEXT | - | `'open'` | `open`, `reviewed`, `confirmed`, `rejected` |
| `created_at` | TEXT | NOT NULL | - | Creation timestamp |
| `reviewed_by` | TEXT | - | - | Reviewer name |
| `reviewed_at` | TEXT | - | - | Review timestamp |
| `notes` | TEXT | - | - | Review notes |

**Row Count:** 5

---

### entities

**Description:** People and organizations identified from email addresses.

| Column | Type | Constraints | Default | Description |
|--------|------|-------------|---------|-------------|
| `id` | TEXT | PRIMARY KEY | - | Unique entity identifier (UUID) |
| `case_id` | TEXT | NOT NULL, FK → cases(id) | - | Parent case |
| `email_address` | TEXT | NOT NULL | - | Email address |
| `display_name` | TEXT | - | - | Display name |
| `first_seen` | TEXT | - | - | First appearance timestamp |
| `last_seen` | TEXT | - | - | Last appearance timestamp |
| `sent_count` | INTEGER | - | 0 | Emails sent |
| `received_count` | INTEGER | - | 0 | Emails received |
| `role` | TEXT | - | `'unknown'` | `unknown`, `suspect`, `victim`, `witness` |
| `aliases` | TEXT | - | - | JSON array of alias email addresses |

**Constraints:** UNIQUE(case_id, email_address)

**Row Count:** 4,256

---

### communication_edges

**Description:** Communication relationships between entities. Used for graph visualization.

| Column | Type | Constraints | Default | Description |
|--------|------|-------------|---------|-------------|
| `id` | TEXT | PRIMARY KEY | - | Unique edge identifier (UUID) |
| `case_id` | TEXT | NOT NULL, FK → cases(id) | - | Parent case |
| `from_entity` | TEXT | NOT NULL | - | Sender entity |
| `to_entity` | TEXT | NOT NULL | - | Recipient entity |
| `message_count` | INTEGER | - | 0 | Number of messages |
| `first_seen` | TEXT | - | - | First communication timestamp |
| `last_seen` | TEXT | - | - | Last communication timestamp |

**Constraints:** UNIQUE(case_id, from_entity, to_entity)

**Row Count:** 0

---

### timeline_events

**Description:** Chronological events for timeline visualization.

| Column | Type | Constraints | Default | Description |
|--------|------|-------------|---------|-------------|
| `id` | TEXT | PRIMARY KEY | - | Unique event identifier (UUID) |
| `case_id` | TEXT | NOT NULL, FK → cases(id) | - | Parent case |
| `evidence_id` | TEXT | NOT NULL | - | Source evidence |
| `email_id` | TEXT | - | - | Related email |
| `event_type` | TEXT | NOT NULL | - | Type of event |
| `timestamp` | TEXT | NOT NULL | - | Event timestamp |
| `actor` | TEXT | - | - | Person involved |
| `summary` | TEXT | - | - | Event summary |

**Row Count:** 0

---

## Artifacts Tables

### forensic_artifacts

**Description:** Extracted forensic artifacts from email analysis (URLs, credentials, PII, crypto addresses, etc.).

| Column | Type | Constraints | Default | Description |
|--------|------|-------------|---------|-------------|
| `id` | TEXT | PRIMARY KEY | - | Unique artifact identifier (UUID) |
| `case_id` | TEXT | NOT NULL, FK → cases(id) | - | Parent case |
| `domain_id` | TEXT | NOT NULL | - | Artifact domain (e.g., `credentials`, `urls`, `pii`) |
| `subcategory_id` | TEXT | NOT NULL | - | Artifact subcategory |
| `title` | TEXT | NOT NULL | - | Artifact title |
| `primary_value` | TEXT | NOT NULL | - | Primary extracted value |
| `secondary_value` | TEXT | - | - | Secondary value |
| `details` | TEXT | - | - | Additional details |
| `severity` | TEXT | NOT NULL | - | `critical`, `high`, `medium`, `low`, `info` |
| `artifact_type` | TEXT | NOT NULL | - | `native`, `extracted`, `derived` |
| `confidence` | TEXT | - | - | Confidence level |
| `email_id` | TEXT | NOT NULL | - | Source email |
| `email_subject` | TEXT | - | - | Email subject |
| `email_from` | TEXT | NOT NULL | - | Email sender |
| `date_sent_utc` | TEXT | - | - | Email date |

**Row Count:** 5,418

---

### artifacts_cache

**Description:** Cached artifact results for performance. Stores pre-computed artifact views.

| Column | Type | Constraints | Default | Description |
|--------|------|-------------|---------|-------------|
| `id` | TEXT | PRIMARY KEY | - | Unique cache entry identifier (UUID) |
| `case_id` | TEXT | NOT NULL, FK → cases(id) | - | Parent case |
| `domain_id` | TEXT | NOT NULL | - | Artifact domain |
| `subcategory_id` | TEXT | NOT NULL | - | Artifact subcategory |
| `title` | TEXT | NOT NULL | - | Artifact title |
| `primary_value` | TEXT | NOT NULL | - | Primary value |
| `secondary_value` | TEXT | - | - | Secondary value |
| `details` | TEXT | - | - | Additional details |
| `severity` | TEXT | - | `'info'` | Severity level |
| `artifact_type` | TEXT | - | `'native'` | Artifact type |
| `email_id` | TEXT | - | - | Source email |
| `email_subject` | TEXT | - | - | Email subject |
| `email_from` | TEXT | - | - | Email sender |
| `date_sent_utc` | TEXT | - | - | Email date |
| `created_at` | TEXT | NOT NULL | - | Cache timestamp |

**Row Count:** 0

---

## Notes, Tags & Bookmarks Tables

### case_notes

**Description:** Notes attached to a case.

| Column | Type | Constraints | Default | Description |
|--------|------|-------------|---------|-------------|
| `id` | TEXT | PRIMARY KEY | - | Unique note identifier (UUID) |
| `case_id` | TEXT | NOT NULL, FK → cases(id) | - | Parent case |
| `author` | TEXT | NOT NULL | - | Note author |
| `title` | TEXT | NOT NULL | - | Note title |
| `content` | TEXT | NOT NULL | - | Note content |
| `category` | TEXT | - | `'general'` | Note category |
| `pinned` | INTEGER | - | 0 | Legacy pinned flag |
| `is_pinned` | INTEGER | - | 0 | Pinned flag |
| `created_at` | TEXT | NOT NULL | - | Creation timestamp |
| `updated_at` | TEXT | NOT NULL | - | Last update timestamp |

**Row Count:** 0

---

### email_notes

**Description:** Notes attached to specific emails.

| Column | Type | Constraints | Default | Description |
|--------|------|-------------|---------|-------------|
| `id` | TEXT | PRIMARY KEY | - | Unique note identifier (UUID) |
| `case_id` | TEXT | NOT NULL, FK → cases(id) | - | Parent case |
| `email_id` | TEXT | NOT NULL, FK → emails(id) | - | Related email |
| `author` | TEXT | NOT NULL | - | Note author |
| `content` | TEXT | NOT NULL | - | Note content |
| `created_at` | TEXT | NOT NULL | - | Creation timestamp |
| `updated_at` | TEXT | NOT NULL | - | Last update timestamp |

**Row Count:** 0

---

### email_tags

**Description:** User-defined tags for emails.

| Column | Type | Constraints | Default | Description |
|--------|------|-------------|---------|-------------|
| `id` | TEXT | PRIMARY KEY | - | Unique tag identifier (UUID) |
| `case_id` | TEXT | NOT NULL, FK → cases(id) | - | Parent case |
| `email_id` | TEXT | NOT NULL, FK → emails(id) | - | Related email |
| `tag` | TEXT | NOT NULL | - | Tag name |
| `color` | TEXT | - | `'#3b82f6'` | Tag color (hex) |
| `created_by` | TEXT | NOT NULL | - | Creator |
| `created_at` | TEXT | NOT NULL | - | Creation timestamp |

**Constraints:** UNIQUE(case_id, email_id, tag)

**Row Count:** 0

---

### item_bookmarks

**Description:** Universal bookmarks for any item type (emails, artifacts, entities, etc.).

| Column | Type | Constraints | Default | Description |
|--------|------|-------------|---------|-------------|
| `id` | TEXT | PRIMARY KEY | - | Unique bookmark identifier (UUID) |
| `case_id` | TEXT | NOT NULL, FK → cases(id) | - | Parent case |
| `item_id` | TEXT | NOT NULL | - | Bookmarked item ID |
| `item_type` | TEXT | NOT NULL | - | Item type: `email`, `artifact`, `entity`, etc. |
| `label` | TEXT | NOT NULL | `'Bookmarked'` | Bookmark label |
| `color` | TEXT | NOT NULL | `'#3b82f6'` | Bookmark color |
| `note` | TEXT | - | `''` | Bookmark note |
| `created_at` | TEXT | NOT NULL | - | Creation timestamp |

**Constraints:** UNIQUE(case_id, item_id)

**Row Count:** 2

---

## AI Database Tables

### ai_sessions

**Description:** AI chat sessions for a case.

| Column | Type | Constraints | Default | Description |
|--------|------|-------------|---------|-------------|
| `id` | TEXT | PRIMARY KEY | - | Unique session identifier (UUID) |
| `case_id` | TEXT | NOT NULL, FK → cases(id) | - | Parent case |
| `provider` | TEXT | NOT NULL | - | AI provider: `ollama`, `openai`, `anthropic`, `kiloai` |
| `model` | TEXT | NOT NULL | - | Model name |
| `model_version` | TEXT | - | - | Model version |
| `system_prompt_version` | TEXT | - | - | System prompt version |
| `created_at` | TEXT | NOT NULL | - | Session start timestamp |
| `ended_at` | TEXT | - | - | Session end timestamp |

**Row Count:** 0

---

### ai_messages

**Description:** Individual messages within an AI session.

| Column | Type | Constraints | Default | Description |
|--------|------|-------------|---------|-------------|
| `id` | TEXT | PRIMARY KEY | - | Unique message identifier (UUID) |
| `session_id` | TEXT | NOT NULL, FK → ai_sessions(id) | - | Parent session |
| `role` | TEXT | NOT NULL | - | `user`, `assistant`, `system` |
| `content` | TEXT | NOT NULL | - | Message content |
| `evidence_refs` | TEXT | - | `'[]'` | JSON array of evidence references |
| `timestamp` | TEXT | NOT NULL | - | Message timestamp |

**Row Count:** 0

---

### ai_tool_calls

**Description:** AI tool/function calls made during sessions.

| Column | Type | Constraints | Default | Description |
|--------|------|-------------|---------|-------------|
| `id` | TEXT | PRIMARY KEY | - | Unique call identifier (UUID) |
| `session_id` | TEXT | NOT NULL, FK → ai_sessions(id) | - | Parent session |
| `tool_name` | TEXT | NOT NULL | - | Tool name called |
| `arguments` | TEXT | NOT NULL | - | JSON arguments |
| `result_hash` | TEXT | - | - | Hash of result |
| `result_size` | INTEGER | - | 0 | Result size in bytes |
| `duration_ms` | INTEGER | - | 0 | Execution duration |
| `timestamp` | TEXT | NOT NULL | - | Call timestamp |

**Row Count:** 0

---

### ai_evidence_citations

**Description:** Evidence citations made by AI in responses.

| Column | Type | Constraints | Default | Description |
|--------|------|-------------|---------|-------------|
| `id` | TEXT | PRIMARY KEY | - | Unique citation identifier (UUID) |
| `message_id` | TEXT | NOT NULL, FK → ai_messages(id) | - | Parent message |
| `evidence_id` | TEXT | NOT NULL | - | Cited evidence ID |
| `artifact_id` | TEXT | - | - | Cited artifact ID |
| `citation_type` | TEXT | NOT NULL | - | Type: `email`, `artifact`, `finding` |
| `display_text` | TEXT | NOT NULL | - | Display text for citation |
| `is_validated` | INTEGER | - | 0 | Whether citation was validated |
| `representation_hash` | TEXT | - | - | Hash of cited content |

**Row Count:** 0

---

### ai_generated_findings

**Description:** Findings proposed by AI analysis.

| Column | Type | Constraints | Default | Description |
|--------|------|-------------|---------|-------------|
| `id` | TEXT | PRIMARY KEY | - | Unique finding identifier (UUID) |
| `case_id` | TEXT | NOT NULL, FK → cases(id) | - | Parent case |
| `session_id` | TEXT | FK → ai_sessions(id) | - | Source session |
| `title` | TEXT | NOT NULL | - | Finding title |
| `description` | TEXT | NOT NULL | - | Finding description |
| `severity` | TEXT | - | `'medium'` | `critical`, `high`, `medium`, `low` |
| `status` | TEXT | - | `'proposed'` | `proposed`, `accepted`, `rejected` |
| `evidence_refs` | TEXT | - | `'[]'` | JSON array of evidence references |
| `created_at` | TEXT | NOT NULL | - | Creation timestamp |
| `reviewed_at` | TEXT | - | - | Review timestamp |
| `reviewed_by` | TEXT | - | - | Reviewer |

**Row Count:** 0

---

### ai_model_runs

**Description:** AI model invocation tracking for cost/performance monitoring.

| Column | Type | Constraints | Default | Description |
|--------|------|-------------|---------|-------------|
| `id` | TEXT | PRIMARY KEY | - | Unique run identifier (UUID) |
| `session_id` | TEXT | NOT NULL, FK → ai_sessions(id) | - | Parent session |
| `provider` | TEXT | NOT NULL | - | AI provider |
| `model` | TEXT | NOT NULL | - | Model name |
| `model_version` | TEXT | - | - | Model version |
| `temperature` | REAL | - | - | Temperature setting |
| `tokens_input` | INTEGER | - | 0 | Input token count |
| `tokens_output` | INTEGER | - | 0 | Output token count |
| `timestamp` | TEXT | NOT NULL | - | Run timestamp |

**Row Count:** 0

---

### ai_context_snapshots

**Description:** Snapshots of AI context window state.

| Column | Type | Constraints | Default | Description |
|--------|------|-------------|---------|-------------|
| `id` | TEXT | PRIMARY KEY | - | Unique snapshot identifier (UUID) |
| `session_id` | TEXT | NOT NULL, FK → ai_sessions(id) | - | Parent session |
| `snapshot_type` | TEXT | NOT NULL | - | Type: `initial`, `compressed`, `overflow` |
| `emails_referenced` | TEXT | - | `'[]'` | JSON array of email IDs |
| `entities_investigated` | TEXT | - | `'[]'` | JSON array of entity IDs |
| `tools_called` | TEXT | - | `'[]'` | JSON array of tool names |
| `token_count` | INTEGER | - | 0 | Token count at snapshot |
| `timestamp` | TEXT | NOT NULL | - | Snapshot timestamp |

**Row Count:** 0

---

### ai_audit_log

**Description:** Audit trail for AI interactions and data sharing.

| Column | Type | Constraints | Default | Description |
|--------|------|-------------|---------|-------------|
| `id` | TEXT | PRIMARY KEY | - | Unique log entry identifier (UUID) |
| `case_id` | TEXT | NOT NULL, FK → cases(id) | - | Parent case |
| `action` | TEXT | NOT NULL | - | Action performed |
| `provider` | TEXT | - | - | AI provider used |
| `pages_shared` | INTEGER | - | 0 | Number of pages shared |
| `details` | TEXT | - | - | Additional details |
| `timestamp` | TEXT | NOT NULL | - | Event timestamp |

**Row Count:** 0

---

## Index Reference

### Custom Indexes (35)

| Index Name | Table | Columns | Unique | Origin |
|------------|-------|---------|--------|--------|
| `idx_emails_case_id` | emails | case_id | No | c |
| `idx_emails_from_addr` | emails | from_addr | No | c |
| `idx_emails_date_sent` | emails | date_sent_utc | No | c |
| `idx_emails_folder` | emails | folder_category | No | c |
| `idx_emails_subject` | emails | subject | No | c |
| `idx_emails_evidence_id` | emails | evidence_id | No | c |
| `idx_emails_message_id` | emails | message_id | No | c |
| `idx_attachments_email_id` | attachments | email_id | No | c |
| `idx_attachments_sha256` | attachments | sha256 | No | c |
| `idx_evidence_case_id` | evidence_items | case_id | No | c |
| `idx_custody_evidence_id` | custody_events | evidence_id | No | c |
| `idx_findings_case_id` | findings | case_id | No | c |
| `idx_findings_severity` | findings | severity | No | c |
| `idx_entities_case_id` | entities | case_id | No | c |
| `idx_entities_email` | entities | email_address | No | c |
| `idx_timeline_case_id` | timeline_events | case_id | No | c |
| `idx_timeline_timestamp` | timeline_events | timestamp | No | c |
| `idx_case_notes_case_id` | case_notes | case_id | No | c |
| `idx_email_tags_case_id` | email_tags | case_id | No | c |
| `idx_email_tags_email_id` | email_tags | email_id | No | c |
| `idx_email_notes_case_id` | email_notes | case_id | No | c |
| `idx_email_notes_email_id` | email_notes | email_id | No | c |
| `idx_bookmarks_case_id` | item_bookmarks | case_id | No | c |
| `idx_bookmarks_item_id` | item_bookmarks | item_id | No | c |
| `idx_chain_of_custody_case` | chain_of_custody | case_id | No | c |
| `idx_chain_of_custody_evidence` | chain_of_custody | evidence_id | No | c |
| `idx_edges_case_id` | communication_edges | case_id | No | c |
| `idx_audit_log_timestamp` | audit_log | timestamp | No | c |
| `idx_forensic_artifacts_case` | forensic_artifacts | case_id | No | c |
| `idx_forensic_artifacts_dom` | forensic_artifacts | case_id, domain_id | No | c |
| `idx_forensic_artifacts_sub` | forensic_artifacts | case_id, subcategory_id | No | c |
| `idx_artifacts_cache_case_id` | artifacts_cache | case_id | No | c |
| `idx_artifacts_cache_domain` | artifacts_cache | case_id, domain_id | No | c |
| `idx_ai_sessions_case` | ai_sessions | case_id | No | c |
| `idx_ai_messages_session` | ai_messages | session_id | No | c |
| `idx_ai_tool_calls_session` | ai_tool_calls | session_id | No | c |
| `idx_ai_citations_message` | ai_evidence_citations | message_id | No | c |
| `idx_ai_findings_case` | ai_generated_findings | case_id | No | c |
| `idx_ai_findings_status` | ai_generated_findings | status | No | c |
| `idx_ai_model_runs_session` | ai_model_runs | session_id | No | c |
| `idx_ai_context_session` | ai_context_snapshots | session_id | No | c |
| `idx_ai_audit_case` | ai_audit_log | case_id | No | c |
| `idx_ai_audit_timestamp` | ai_audit_log | timestamp | No | c |

### Auto-Index (Primary Key) Indexes (25)

Each table has a `sqlite_autoindex_<table>_1` index on its PRIMARY KEY column.

### Auto-Index (Unique Constraint) Indexes (4)

| Index Name | Table | Columns |
|------------|-------|---------|
| `sqlite_autoindex_entities_2` | entities | case_id, email_address |
| `sqlite_autoindex_communication_edges_2` | communication_edges | case_id, from_entity, to_entity |
| `sqlite_autoindex_email_tags_2` | email_tags | case_id, email_id, tag |
| `sqlite_autoindex_item_bookmarks_2` | item_bookmarks | case_id, item_id |

---

## Foreign Key Relationships

| Child Table | Column | Parent Table | Parent Column | ON DELETE | ON UPDATE |
|-------------|--------|--------------|---------------|-----------|-----------|
| evidence_items | case_id | cases | id | NO ACTION | NO ACTION |
| emails | evidence_id | evidence_items | id | NO ACTION | NO ACTION |
| emails | case_id | cases | id | NO ACTION | NO ACTION |
| attachments | email_id | emails | id | NO ACTION | NO ACTION |
| custody_events | evidence_id | evidence_items | id | NO ACTION | NO ACTION |
| findings | case_id | cases | id | NO ACTION | NO ACTION |
| entities | case_id | cases | id | NO ACTION | NO ACTION |
| communication_edges | case_id | cases | id | NO ACTION | NO ACTION |
| timeline_events | case_id | cases | id | NO ACTION | NO ACTION |
| case_notes | case_id | cases | id | NO ACTION | NO ACTION |
| email_tags | case_id | cases | id | NO ACTION | NO ACTION |
| email_tags | email_id | emails | id | NO ACTION | NO ACTION |
| email_notes | case_id | cases | id | NO ACTION | NO ACTION |
| email_notes | email_id | emails | id | NO ACTION | NO ACTION |
| chain_of_custody | case_id | cases | id | NO ACTION | NO ACTION |
| chain_of_custody | evidence_id | evidence_items | id | NO ACTION | NO ACTION |
| forensic_artifacts | case_id | cases | id | NO ACTION | NO ACTION |
| artifacts_cache | case_id | cases | id | NO ACTION | NO ACTION |
| item_bookmarks | case_id | cases | id | NO ACTION | NO ACTION |
| ai_sessions | case_id | cases | id | NO ACTION | NO ACTION |
| ai_messages | session_id | ai_sessions | id | NO ACTION | NO ACTION |
| ai_tool_calls | session_id | ai_sessions | id | NO ACTION | NO ACTION |
| ai_evidence_citations | message_id | ai_messages | id | NO ACTION | NO ACTION |
| ai_generated_findings | case_id | cases | id | NO ACTION | NO ACTION |
| ai_generated_findings | session_id | ai_sessions | id | NO ACTION | NO ACTION |
| ai_model_runs | session_id | ai_sessions | id | NO ACTION | NO ACTION |
| ai_context_snapshots | session_id | ai_sessions | id | NO ACTION | NO ACTION |
| ai_audit_log | case_id | cases | id | NO ACTION | NO ACTION |

---

## Unique Constraints

| Table | Constraint Name | Columns |
|-------|-----------------|---------|
| entities | sqlite_autoindex_entities_2 | case_id, email_address |
| communication_edges | sqlite_autoindex_communication_edges_2 | case_id, from_entity, to_entity |
| email_tags | sqlite_autoindex_email_tags_2 | case_id, email_id, tag |
| item_bookmarks | sqlite_autoindex_item_bookmarks_2 | case_id, item_id |

---

## Row Counts

| Table | Rows | Category |
|-------|------|----------|
| emails | 14,227 | Core |
| entities | 4,256 | Analysis |
| forensic_artifacts | 5,418 | Artifacts |
| attachments | 154 | Core |
| cases | 2 | Core |
| evidence_items | 3 | Core |
| findings | 5 | Analysis |
| chain_of_custody | 13 | Custody |
| custody_events | 2 | Custody |
| audit_log | 4 | Custody |
| item_bookmarks | 2 | Bookmarks |
| ai_sessions | 0 | AI |
| ai_messages | 0 | AI |
| ai_tool_calls | 0 | AI |
| ai_evidence_citations | 0 | AI |
| ai_generated_findings | 0 | AI |
| ai_model_runs | 0 | AI |
| ai_context_snapshots | 0 | AI |
| ai_audit_log | 0 | AI |
| case_notes | 0 | Notes |
| email_notes | 0 | Notes |
| email_tags | 0 | Tags |
| timeline_events | 0 | Analysis |
| communication_edges | 0 | Analysis |
| artifacts_cache | 0 | Artifacts |

---

## Database Migrations

| Migration | Table | Change |
|-----------|-------|--------|
| Add `target_email` | cases | New column |
| Add `target_name` | cases | New column |
| Add `target_organization` | cases | New column |
| Add `investigation_type` | cases | New column with default `'general'` |
| Add `working_dir` | cases | New column |
| Add `folder_name` | emails | New column |
| Add `folder_category` | emails | New column with default `'other'` |
| Add `recovery_status` | emails | New column with default `'normal'` |
| Add `reviewed_by` | findings | New column |
| Add `reviewed_at` | findings | New column |
| Add `notes` | findings | New column |
| Add `aliases` | entities | New column |
| Add `is_pinned` | case_notes | New column with default 0 |
| Sync `is_pinned` from `pinned` | case_notes | Data migration |

---

## Rust Data Structures (In-Memory)

### Core Models (models.rs)

#### Case
```rust
pub struct Case {
    pub id: String,
    pub title: String,
    pub case_number: String,
    pub description: String,
    pub status: String,
    pub owner_id: String,
    pub target_email: Option<String>,
    pub target_name: Option<String>,
    pub target_organization: Option<String>,
    pub investigation_type: String,
    pub working_dir: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

#### EvidenceItem
```rust
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
```

#### EmailMessage
```rust
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
    pub attachment_count: u32,
    pub image_count: u32,
}
```

#### Attachment
```rust
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
```

#### CustodyEvent
```rust
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
```

#### Finding
```rust
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
    pub notes: Option<String>,
}
```

#### DashboardData
```rust
pub struct DashboardData {
    pub evidence_count: u32,
    pub email_count: u32,
    pub deleted_recovered: u32,
    pub entity_count: u32,
    pub finding_count: u32,
    pub severity_breakdown: HashMap<String, u32>,
    pub date_range: (Option<String>, Option<String>),
    pub top_correspondents: Vec<TopCorrespondent>,
    pub sent_count: u32,
    pub inbox_count: u32,
    pub important_count: u32,
    pub soft_deleted_count: u32,
    pub drafts_count: u32,
    pub spam_count: u32,
    pub other_count: u32,
    pub high_risk_emails: u32,
}
```

#### TopCorrespondent
```rust
pub struct TopCorrespondent {
    pub email: String,
    pub sent: u32,
    pub received: u32,
}
```

#### Entity
```rust
pub struct Entity {
    pub id: String,
    pub case_id: String,
    pub email_address: String,
    pub display_name: Option<String>,
    pub first_seen: Option<String>,
    pub last_seen: Option<String>,
    pub sent_count: i64,
    pub received_count: i64,
    pub role: String,
    pub aliases: Option<String>,
}
```

#### CaseNote
```rust
pub struct CaseNote {
    pub id: String,
    pub case_id: String,
    pub author: String,
    pub title: String,
    pub content: String,
    pub category: String,
    pub pinned: bool,
    pub created_at: String,
    pub updated_at: String,
}
```

#### EmailTag
```rust
pub struct EmailTag {
    pub id: String,
    pub case_id: String,
    pub email_id: String,
    pub tag: String,
    pub color: String,
    pub created_by: String,
    pub created_at: String,
}
```

#### EmailNote
```rust
pub struct EmailNote {
    pub id: String,
    pub case_id: String,
    pub email_id: String,
    pub author: String,
    pub content: String,
    pub created_at: String,
    pub updated_at: String,
}
```

---

### Command Inputs (models.rs)

#### CaseCreateInput
```rust
pub struct CaseCreateInput {
    pub title: String,
    pub case_number: Option<String>,
    pub description: Option<String>,
    pub target_email: Option<String>,
    pub target_name: Option<String>,
    pub target_organization: Option<String>,
    pub investigation_type: Option<String>,
    pub working_dir: Option<String>,
}
```

#### CaseUpdateInput
```rust
pub struct CaseUpdateInput {
    pub case_id: String,
    pub title: String,
    pub description: Option<String>,
    pub status: Option<String>,
    pub target_email: Option<String>,
    pub target_name: Option<String>,
    pub target_organization: Option<String>,
}
```

#### EvidenceUploadInput
```rust
pub struct EvidenceUploadInput {
    pub case_id: String,
    pub file_path: String,
    pub source_description: Option<String>,
}
```

#### EmailListInput
```rust
pub struct EmailListInput {
    pub case_id: String,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
    pub from_filter: Option<String>,
}
```

#### SearchInput
```rust
pub struct SearchInput {
    pub case_id: String,
    pub query: String,
    pub limit: Option<u32>,
    pub evidence_id: Option<String>,
}
```

#### EntityInput
```rust
pub struct EntityInput {
    pub case_id: String,
    pub email_address: String,
}
```

#### EmptyInput
```rust
pub struct EmptyInput {
    pub case_id: String,
}
```

#### CaseNoteCreateInput
```rust
pub struct CaseNoteCreateInput {
    pub case_id: String,
    pub author: Option<String>,
    pub title: String,
    pub content: String,
    pub category: Option<String>,
    pub pinned: Option<bool>,
}
```

#### CaseNoteUpdateInput
```rust
pub struct CaseNoteUpdateInput {
    pub id: String,
    pub title: Option<String>,
    pub content: Option<String>,
    pub category: Option<String>,
    pub pinned: Option<bool>,
}
```

#### EmailTagAddInput
```rust
pub struct EmailTagAddInput {
    pub case_id: String,
    pub email_id: String,
    pub tag: String,
    pub color: Option<String>,
    pub created_by: Option<String>,
}
```

#### EmailTagRemoveInput
```rust
pub struct EmailTagRemoveInput {
    pub case_id: String,
    pub email_id: String,
    pub tag: String,
}
```

#### EmailNoteInput
```rust
pub struct EmailNoteInput {
    pub case_id: String,
    pub email_id: String,
    pub author: Option<String>,
    pub content: String,
}
```

---

## AI Data Structures (ai.rs)

#### KiloAIModel
```rust
pub struct KiloAIModel {
    pub id: String,
    pub name: String,
    pub provider: String,
    pub context_length: Option<u32>,
    pub input_types: Option<Vec<String>>,
    pub capabilities: Option<Vec<String>>,
    pub is_free: bool,
}
```

#### SearchQuery
```rust
pub struct SearchQuery {
    pub query: String,
    pub case_id: String,
    pub limit: Option<u32>,
    pub filters: Option<SearchFilters>,
}
```

#### EmailResult
```rust
pub struct EmailResult {
    pub id: String,
    pub subject: Option<String>,
    pub from_addr: String,
    pub date_sent: Option<String>,
    pub body_preview: Option<String>,
    pub relevance_score: f64,
    pub evidence_refs: Vec<String>,
}
```

#### AttachmentMetadata
```rust
pub struct AttachmentMetadata {
    pub id: String,
    pub filename: Option<String>,
    pub mime_type: Option<String>,
    pub size_bytes: u64,
    pub entropy: Option<f64>,
    pub risk_flags: String,
    pub sha256: String,
}
```

#### AuthResults
```rust
pub struct AuthResults {
    pub spf: Option<String>,
    pub dkim: Option<String>,
    pub dmarc: Option<String>,
    pub arc: Option<String>,
    pub auth_details: Option<serde_json::Value>,
}
```

#### EntityData
```rust
pub struct EntityData {
    pub email_address: String,
    pub display_name: Option<String>,
    pub sent_count: i64,
    pub received_count: i64,
    pub first_seen: Option<String>,
    pub last_seen: Option<String>,
    pub role: String,
}
```

#### TimelineEvent
```rust
pub struct TimelineEvent {
    pub timestamp: String,
    pub event_type: String,
    pub actor: Option<String>,
    pub summary: Option<String>,
    pub email_id: Option<String>,
}
```

#### FindingData
```rust
pub struct FindingData {
    pub id: String,
    pub type_: String,
    pub severity: String,
    pub title: String,
    pub description: Option<String>,
    pub email_ids: Vec<String>,
}
```

#### CaseStats
```rust
pub struct CaseStats {
    pub email_count: u32,
    pub entity_count: u32,
    pub finding_count: u32,
    pub attachment_count: u32,
    pub date_range: (Option<String>, Option<String>),
    pub folder_breakdown: HashMap<String, u32>,
    pub high_risk_count: u32,
}
```

#### ToolRiskLevel (Enum)
```rust
pub enum ToolRiskLevel {
    Safe,
    ReadOnly,
    RequiresApproval,
    Restricted,
}
```

#### InvestigationBudget
```rust
pub struct InvestigationBudget {
    pub max_tool_calls: u32,
    pub max_emails_scanned: u32,
    pub max_tokens: u32,
    pub current_tool_calls: u32,
    pub current_emails_scanned: u32,
    pub current_tokens: u32,
}
```

#### EvidenceGatewayPolicy
```rust
pub struct EvidenceGatewayPolicy {
    pub allow_ai_access: bool,
    pub max_pages_per_request: u32,
    pub allowed_tools: Vec<String>,
    pub blocked_tools: Vec<String>,
    pub require_human_approval: bool,
}
```

#### AIProviderType (Enum)
```rust
pub enum AIProviderType {
    Ollama,
    OpenAI,
    Anthropic,
    KiloAI,
    OpenRouter,
    Custom,
}
```

#### ToolDefinition
```rust
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub parameters: Vec<ToolParameter>,
    pub risk_level: ToolRiskLevel,
}
```

#### ToolParameter
```rust
pub struct ToolParameter {
    pub name: String,
    pub type_: String,
    pub description: String,
    pub required: bool,
}
```

#### InvestigationStep
```rust
pub struct InvestigationStep {
    pub step_number: u32,
    pub title: String,
    pub description: String,
    pub tool_name: String,
    pub parameters: serde_json::Value,
    pub expected_output: String,
}
```

#### InvestigationPlan
```rust
pub struct InvestigationPlan {
    pub title: String,
    pub description: String,
    pub steps: Vec<InvestigationStep>,
    pub estimated_duration_minutes: u32,
}
```

#### TimelineInterpretation
```rust
pub struct TimelineInterpretation {
    pub summary: String,
    pub key_events: Vec<TimelineEvent>,
    pub patterns: Vec<String>,
    pub anomalies: Vec<TimelineAnomaly>,
}
```

#### TimelineAnomaly
```rust
pub struct TimelineAnomaly {
    pub timestamp: String,
    pub description: String,
    pub severity: String,
    pub related_emails: Vec<String>,
}
```

#### SpoofingAnalysis
```rust
pub struct SpoofingAnalysis {
    pub is_spoofed: bool,
    pub confidence: f64,
    pub findings: Vec<SpoofingFinding>,
    pub recommendations: Vec<String>,
}
```

#### SpoofingFinding
```rust
pub struct SpoofingFinding {
    pub type_: String,
    pub description: String,
    pub evidence: String,
    pub severity: String,
}
```

#### AttachmentTriage
```rust
pub struct AttachmentTriage {
    pub attachment_id: String,
    pub risk_level: String,
    pub reasons: Vec<String>,
    pub recommended_action: String,
}
```

#### AttachmentRisk
```rust
pub struct AttachmentRisk {
    pub filename: String,
    pub risk_score: u8,
    pub risk_factors: Vec<String>,
    pub mime_type: Option<String>,
    pub entropy: Option<f64>,
}
```

#### GraphAnalysis
```rust
pub struct GraphAnalysis {
    pub central_entities: Vec<EntityCentrality>,
    pub communities: Vec<Vec<String>>,
    pub anomalies: Vec<GraphAnomaly>,
    pub summary: String,
}
```

#### EntityCentrality
```rust
pub struct EntityCentrality {
    pub email: String,
    pub centrality_score: f64,
    pub connections: u32,
    pub role: String,
}
```

#### GraphAnomaly
```rust
pub struct GraphAnomaly {
    pub type_: String,
    pub description: String,
    pub entities_involved: Vec<String>,
    pub severity: String,
}
```

#### EntityResolution
```rust
pub struct EntityResolution {
    pub canonical_entity: String,
    pub aliases: Vec<String>,
    pub confidence: f64,
    pub evidence: Vec<String>,
}
```

#### EntityCandidate
```rust
pub struct EntityCandidate {
    pub email: String,
    pub display_name: Option<String>,
    pub match_reason: String,
    pub confidence: f64,
}
```

#### AnomalyDetection
```rust
pub struct AnomalyDetection {
    pub anomalies: Vec<EmailAnomaly>,
    pub summary: String,
    pub risk_assessment: String,
}
```

#### EmailAnomaly
```rust
pub struct EmailAnomaly {
    pub email_id: String,
    pub type_: String,
    pub description: String,
    pub severity: String,
    pub evidence: String,
}
```

#### ReportSection
```rust
pub struct ReportSection {
    pub title: String,
    pub content: String,
    pub evidence_refs: Vec<String>,
    pub section_type: String,
}
```

#### InvestigationReport
```rust
pub struct InvestigationReport {
    pub title: String,
    pub generated_at: DateTime<Utc>,
    pub sections: Vec<ReportSection>,
    pub metadata: ReportMetadata,
}
```

#### ReportMetadata
```rust
pub struct ReportMetadata {
    pub case_id: String,
    pub generated_by: String,
    pub ai_provider: Option<String>,
    pub ai_model: Option<String>,
    pub disclaimers: Vec<String>,
}
```

---

## Analysis Data Structures (analysis.rs)

#### AnalysisResult
```rust
pub struct AnalysisResult {
    pub email_id: String,
    pub risk_score: u8,
    pub findings: Vec<NewFinding>,
    pub header_analysis: HeaderAnalysis,
    pub auth_results: AuthResults,
    pub spoofing_findings: Vec<SpoofingFinding>,
}
```

#### HeaderAnalysis
```rust
pub struct HeaderAnalysis {
    pub received_chain: Vec<Hop>,
    pub authentication_results: AuthResults,
    pub skew_events: Vec<SkewEvent>,
    pub anomalies: Vec<Anomaly>,
}
```

#### Hop
```rust
pub struct Hop {
    pub from: Option<String>,
    pub by: Option<String>,
    pub with: Option<String>,
    pub timestamp: Option<String>,
    pub delay_seconds: Option<i64>,
}
```

#### SkewEvent
```rust
pub struct SkewEvent {
    pub timestamp: String,
    pub expected_timestamp: String,
    pub skew_seconds: i64,
    pub description: String,
}
```

#### Anomaly
```rust
pub struct Anomaly {
    pub type_: String,
    pub description: String,
    pub severity: String,
    pub evidence: String,
}
```

#### AuthResults
```rust
pub struct AuthResults {
    pub spf: Option<AuthCheck>,
    pub dkim: Option<AuthCheck>,
    pub dmarc: Option<AuthCheck>,
    pub arc: Option<ArcSeal>,
}
```

#### AuthCheck
```rust
pub struct AuthCheck {
    pub result: String,
    pub domain: Option<String>,
    pub details: Option<String>,
}
```

#### ArcSeal
```rust
pub struct ArcSeal {
    pub seal_id: String,
    pub cv: String,
    pub algorithm: String,
}
```

#### SpoofingFinding
```rust
pub struct SpoofingFinding {
    pub type_: String,
    pub description: String,
    pub evidence: String,
    pub severity: String,
}
```

#### AttachmentAnalysis
```rust
pub struct AttachmentAnalysis {
    pub filename: Option<String>,
    pub mime_type: Option<String>,
    pub size_bytes: u64,
    pub entropy: Option<f64>,
    pub risk_flags: Vec<String>,
    pub is_suspicious: bool,
    pub risk_score: u8,
}
```

#### NewFinding
```rust
pub struct NewFinding {
    pub type_: String,
    pub severity: String,
    pub confidence: String,
    pub title: String,
    pub description: String,
    pub email_ids: Vec<String>,
}
```

---

## Artifact Taxonomy (commands/artifacts.rs)

#### TaxonomySubcategorySummary
```rust
pub struct TaxonomySubcategorySummary {
    pub subcategory_id: String,
    pub name: String,
    pub count: usize,
}
```

#### TaxonomyDomainSummary
```rust
pub struct TaxonomyDomainSummary {
    pub domain_id: String,
    pub name: String,
    pub icon: String,
    pub total_count: usize,
    pub subcategories: Vec<TaxonomySubcategorySummary>,
}
```

#### ForensicTaxonomyArtifact
```rust
pub struct ForensicTaxonomyArtifact {
    pub id: String,
    pub domain_id: String,
    pub subcategory_id: String,
    pub title: String,
    pub primary_value: String,
    pub secondary_value: Option<String>,
    pub details: String,
    pub severity: String,
    pub artifact_type: String,
    pub confidence: Option<String>,
    pub email_id: String,
    pub email_subject: Option<String>,
    pub email_from: String,
    pub date_sent_utc: Option<String>,
}
```

#### AppSignature
```rust
struct AppSignature {
    name: &'static str,
    domain_id: &'static str,
    subcategory: &'static str,
    keywords: &'static [&'static str],
    category_title: &'static str,
}
```

---

## IMAP/POP3 Acquisition

#### ImapConfig (imap_acquisition.rs)
```rust
pub struct ImapConfig {
    pub server: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub use_tls: bool,
    pub folder: String,
}
```

#### ImapFolderMessage
```rust
pub struct ImapFolderMessage {
    pub uid: u32,
    pub subject: Option<String>,
    pub from: Option<String>,
    pub date: Option<String>,
    pub size: Option<u32>,
}
```

#### ImapAcquisitionResult
```rust
pub struct ImapAcquisitionResult {
    pub total_messages: u32,
    pub fetched_messages: u32,
    pub new_messages: u32,
    pub errors: Vec<String>,
}
```

#### StreamingMessage
```rust
pub struct StreamingMessage {
    pub uid: u32,
    pub subject: Option<String>,
    pub from: Option<String>,
    pub date: Option<String>,
    pub body_preview: Option<String>,
}
```

#### Pop3Config (commands/pop3.rs)
```rust
pub struct Pop3Config {
    pub server: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub use_tls: bool,
}
```

#### Pop3AcquisitionResult
```rust
pub struct Pop3AcquisitionResult {
    pub total_messages: u32,
    pub fetched_messages: u32,
    pub new_messages: u32,
    pub errors: Vec<String>,
}
```

---

## Parser Data Structures (parser.rs)

#### RawEmail
```rust
pub struct RawEmail {
    pub message_id: Option<String>,
    pub from: Option<String>,
    pub to: Option<String>,
    pub cc: Option<String>,
    pub bcc: Option<String>,
    pub subject: Option<String>,
    pub date: Option<String>,
    pub headers: HashMap<String, String>,
    pub body_text: Option<String>,
    pub body_html: Option<String>,
    pub attachments: Vec<RawAttachment>,
}
```

#### RawAttachment
```rust
pub struct RawAttachment {
    pub filename: Option<String>,
    pub content_type: Option<String>,
    pub data: Vec<u8>,
    pub content_id: Option<String>,
}
```

#### PstParser (pst.rs)
```rust
pub struct PstParser;
```

#### PstFolder
```rust
pub struct PstFolder {
    pub name: String,
    pub message_count: u32,
    pub subfolders: Vec<PstFolder>,
}
```

---

## Bookmark Data Structures (commands/bookmarks.rs)

#### ItemBookmark
```rust
pub struct ItemBookmark {
    pub id: String,
    pub case_id: String,
    pub item_id: String,
    pub item_type: String,
    pub label: String,
    pub color: String,
    pub note: String,
    pub created_at: String,
}
```

---

## Attachment Data Structures (commands/attachments.rs)

#### CaseAttachmentItem
```rust
pub struct CaseAttachmentItem {
    pub id: String,
    pub email_id: String,
    pub filename: Option<String>,
    pub sha256: String,
    pub mime_type: Option<String>,
    pub size_bytes: u64,
    pub stored_path: String,
    pub entropy: Option<f64>,
    pub risk_flags: String,
    pub email_subject: Option<String>,
    pub email_from: Option<String>,
    pub date_sent: Option<String>,
}
```

#### AttachmentCategoryCounts
```rust
pub struct AttachmentCategoryCounts {
    pub documents: u32,
    pub images: u32,
    pub archives: u32,
    pub executables: u32,
    pub other: u32,
}
```

#### InlineImageData
```rust
pub struct InlineImageData {
    pub content_id: String,
    pub mime_type: String,
    pub data: String, // base64 encoded
    pub filename: Option<String>,
}
```

---

## Application State (main.rs)

#### AppState
```rust
pub struct AppState {
    pub db: Arc<Mutex<Database>>,
}
```

---

## Database Connection (db.rs)

#### Database
```rust
pub struct Database {
    pub conn: Connection,
}
```

---

## Notes

- All timestamps are stored as ISO 8601 strings (e.g., `2024-01-15T10:30:00Z`)
- All IDs are UUID v4 strings
- JSON arrays are stored as TEXT (e.g., `'["a", "b"]'`)
- The `folder_category` is derived from `X-Folder` header during migration
- AI tables are currently empty (Phase 0 - infrastructure ready)
- `communication_edges` and `timeline_events` are populated by analysis engines
- `artifacts_cache` is populated on-demand for performance
- Foreign keys are defined in schema but enforcement is disabled at file level (`foreign_keys = 0`)
- WAL mode is active with 15 MB of pending writes in WAL file

---

*Generated: 2026-08-26*
*Database: J12 Forensic Email Investigation Platform*
*Version: 1.0.0*
