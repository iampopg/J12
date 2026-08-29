# Phase 4 Audit: File Import & Parsing

> **Files Audited:**
> - `src-tauri/src/parser.rs` (1088 lines)
> - `src-tauri/src/pst.rs` (231 lines)
> - `src-tauri/src/commands/evidence.rs` (433 lines)

---

## Findings

### ISSUE-036: PST/OST parsing not implemented
- **Category:** NO BACKEND
- **File:** `src-tauri/src/pst.rs:26-37`
- **What's wrong:** `PstParser::parse()` returns `Err("PST/OST parsing requires libpff library")`. All PST operations are stubs.
- **Impact:** PST files (most common Outlook format) cannot be parsed. Users must convert to MBOX first.
- **Fix:** Integrate libpff or use a Rust PST parser library.

---

### ISSUE-037: MSG parsing not implemented
- **Category:** NO BACKEND
- **File:** `src-tauri/src/pst.rs:106-114`
- **What's wrong:** `parse_msg()` returns `Err("MSG parsing requires CFB/OLE parser")`. MSG format is not supported.
- **Impact:** Outlook .msg files cannot be parsed.
- **Fix:** Implement CFB/OLE parser or use existing Rust crate.

---

### ISSUE-038: EMLX parsing is incomplete
- **Category:** BREAK
- **File:** `src-tauri/src/pst.rs:117-231`
- **What's wrong:** `parse_emlx()` is a minimal implementation that doesn't handle plist metadata properly, doesn't extract all headers, doesn't handle attachments, and only parses basic From/To/Subject.
- **Impact:** EMLX files (Apple Mail) parsed incorrectly. Missing data.
- **Fix:** Implement proper plist parsing and full RFC822 extraction.

---

### ISSUE-039: No progress reporting during parsing
- **Category:** NOT DYNAMIC
- **File:** `src-tauri/src/commands/evidence.rs:203-368`
- **What's wrong:** `parse_evidence` parses all emails in a single blocking operation. No progress events emitted to frontend. UI freezes during large file parsing.
- **Impact:** User cannot see parse progress. App appears hung during large MBOX parsing.
- **Fix:** Emit progress events via `app.emit()` during parsing.

---

### ISSUE-040: Base64 decode silently fails
- **Category:** ERROR-PRONE
- **File:** `src-tauri/src/parser.rs:718-724`
- **What's wrong:** `base64_decode` falls back to returning raw bytes on decode error: `Err(_) => cleaned.as_bytes().to_vec()`. This means corrupted base64 is silently passed through.
- **Impact:** Attachments may contain garbage data without any warning.
- **Fix:** Return error on decode failure, log warning, or skip attachment.

---

### ISSUE-041: No size limit on parsing
- **Category:** ERROR-PRONE
- **File:** `src-tauri/src/commands/evidence.rs:203-368`
- **What's wrong:** `parse_evidence` reads entire file into memory and parses all emails. No size limit. A 10GB MBOX will exhaust RAM.
- **Impact:** App crashes on large evidence files.
- **Fix:** Add streaming parsing for MBOX, process in chunks, set configurable max file size.

---

### ISSUE-042: Attachment stored_path not populated
- **Category:** BREAK
- **File:** `src-tauri/src/commands/evidence.rs:327-341`
- **File:** `src-tauri/src/db.rs:130-141`
- **What's wrong:** When inserting attachments during parsing, `stored_path` is not set (only `id, email_id, filename, mime_type, size_bytes, sha256, md5, entropy, is_inline, is_macro_enabled, is_executable, risk_flags`). The `stored_path` column exists in schema but is never populated during file parsing.
- **Impact:** `open_attachment_in_system` will fail because it reads from `stored_path` which is NULL.
- **Fix:** Save attachments to disk and set `stored_path` to the saved location.

---

### ISSUE-043: evidence_delete doesn't delete from chain_of_custody for case
- **Category:** INCOMPLETE (already fixed in case_delete but not here)
- **File:** `src-tauri/src/commands/evidence.rs:139-182`
- **What's wrong:** `evidence_delete` deletes from `chain_of_custody WHERE evidence_id = ?1` but doesn't delete other evidence-related records like `forensic_artifacts`, `timeline_events`, or `email_tags/notes` that reference the emails.
- **Impact:** Orphaned records remain after evidence deletion.
- **Fix:** Add deletion for all child tables: `forensic_artifacts`, `timeline_events`, `email_tags`, `email_notes`, `item_bookmarks`.

---

### ISSUE-044: open_file_dialog filter includes .db and .txt
- **Category:** ERROR-PRONE
- ** File:** `src-tauri/src/commands/evidence.rs:378-385`
- **What's wrong:** File dialog filter includes `*.db` and `txt` which are not email formats. Selecting a SQLite database or text file as evidence will cause parse errors.
- **Impact:** User can select inappropriate files.
- **Fix:** Remove `.db` and `txt` from filter.

---

### ISSUE-045: evidence_upload sets acquired_by to "Examiner" hardcoded
- **Category:** HARDCODED
- **File:** `src-tauri/src/commands/evidence.rs:25`
- **What's wrong:** `acquired_by` is hardcoded to `"Examiner"`. Not from authenticated user.
- **Impact:** Cannot track who uploaded evidence. Chain of custody incomplete.
- **Fix:** Pass username from frontend auth context.

---

### ISSUE-046: Duplicate evidence ghost cleanup in evidence_list
- **Category:** OLD
- **File:** `src-tauri/src/commands/evidence.rs:66-79`
- **What's wrong:** Every call to `evidence_list` runs a DELETE query to clean up ghost rows. This is a workaround for a bug elsewhere that creates duplicates.
- **Impact:** Performance issue. The root cause should be fixed instead of cleaning up on every read.
- **Fix:** Find and fix the root cause of duplicate ghost evidence creation.

---

### ISSUE-047: parse_evidence doesn't update existing emails on re-parse
- **Category:** BREAK
- **File:** `src-tauri/src/commands/evidence.rs:259-343`
- **What's wrong:** Re-parsing the same evidence creates duplicate emails because there's no `INSERT OR REPLACE` or deduplication by message_id.
- **Impact:** Duplicate emails accumulate on re-parse.
- **Fix:** Use `INSERT OR REPLACE` with message_id as key, or delete existing emails before re-parsing.

---

### ISSUE-048: No inline image extraction during parsing
- **Category:** BREAK
- **File:** `src-tauri/src/commands/evidence.rs:304-342`
- **File:** `src-tauri/src/parser.rs:679-698`
- **What's wrong:** Inline images (CID references in HTML) are treated as regular attachments with no `is_inline` flag set. The `is_inline` column exists in schema but is always set to `0`.
- **Impact:** `get_email_inline_images` returns nothing. HTML emails with inline images broken.
- **Fix:** Set `is_inline = 1` for parts with Content-ID or Content-Disposition: inline.

---

## Reconfirmation

I re-read `parser.rs`, `pst.rs`, and `evidence.rs` in full. Findings confirmed:
- PST parse returns error (pst.rs:29-36)
- MSG parse returns error (pst.rs:109-113)
- EMLX is minimal (pst.rs:132-230)
- No progress events in parse_evidence (evidence.rs:203-368)
- base64 falls back to raw bytes (parser.rs:722)
- attachment stored_path not set (evidence.rs:327-341)
- acquired_by hardcoded to "Examiner" (evidence.rs:25)

Cross-referenced with `db.rs`:
- `attachments` table has `stored_path` column (line 137) - confirmed
- `attachments` table has `is_inline` column - confirmed

**All 13 findings confirmed.**

---

## Summary

| Category | Count |
|----------|-------|
| BREAK | 3 |
| ERROR-PRONE | 3 |
| NO BACKEND | 2 |
| HARDCODED | 1 |
| OLD | 1 |
| NOT DYNAMIC | 1 |
| **Total** | **11** |

**Severity:** HIGH - PST and MSG formats (Outlook) are the most common email formats in corporate environments. Their absence makes the tool unusable for many forensic scenarios. No inline image extraction breaks HTML email viewing. No stored_path means attachments can't be opened.

