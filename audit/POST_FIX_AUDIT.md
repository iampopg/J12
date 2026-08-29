# Post-Fix Audit Report

> **Date:** 2026-08-27
> **Purpose:** Verify what was fixed, what remains, and any new issues introduced

---

## Summary

| Category | Count |
|----------|-------|
| **Fixed** | 10 |
| **Not Fixed** | 6 |
| **New Issues Introduced** | 0 |
| **Remaining from Original 125** | ~100 (minor issues still present) |

---

## 1. FIXED ISSUES (Verified by Code Review)

### ISSUE-114: AI module not registered in main.rs ✅ FIXED
**File:** `src-tauri/src/main.rs`
```rust
// Line 12 - Module now declared
pub mod ai;

// Lines 110-134 - All 25 AI commands registered
.invoke_handler(tauri::generate_handler![
    // ... existing commands ...
    ai::fetch_kiloai_models,
    ai::fetch_openrouter_models,
    ai::ai_chat,
    // ... 22 more AI commands ...
    ai::ai_generate_report,
])
```

### ISSUE-023 / ISSUE-115: AI tables not created in schema ✅ FIXED
**File:** `src-tauri/src/db.rs`
```rust
// Lines 319-359 - AI tables now created
CREATE TABLE IF NOT EXISTS ai_sessions (...);
CREATE TABLE IF NOT EXISTS ai_messages (...);
CREATE TABLE IF NOT EXISTS ai_tool_calls (...);
CREATE TABLE IF NOT EXISTS ai_audit_log (...);
CREATE TABLE IF NOT EXISTS ai_context_snapshots (...);
// ... ai_search_index, ai_entity_resolutions, ai_investigation_plans
```

### ISSUE-011: SQL injection in case_update ✅ FIXED
**File:** `src-tauri/src/commands/cases.rs`
```rust
// Lines 139-153 - Now uses parameterized queries
db.conn.execute(
    "UPDATE cases SET
        title = COALESCE(?1, title),
        description = COALESCE(?2, description),
        status = COALESCE(?3, status),
        updated_at = ?4
     WHERE id = ?5",
    rusqlite::params![title, input.description, input.status, now, input.case_id],
)?;
```

### ISSUE-014 / ISSUE-123: No case delete audit logging ✅ FIXED
**File:** `src-tauri/src/commands/cases.rs`
```rust
// Lines 171-179 - Disk audit log
crate::audit_logger::log_forensic_event(
    &case_id,
    "CASE_DELETION",
    "CASE_DESTROYED",
    ...
);

// Lines 184-193 - Database audit log
db.conn.execute(
    "INSERT INTO audit_log (id, actor, action, target_type, target_id, timestamp, detail)
     VALUES (?1, 'Examiner', 'case_deleted', 'case', ?2, ?3, ?4)",
    ...
);
```

### ISSUE-053: Database lock held during entire IMAP fetch ✅ FIXED
**File:** `src-tauri/src/commands/imap.rs`
```rust
// Line 165 - Mutex reference passed to closure
let db_mutex = &state.db;

// Lines 202-203 - Scoped lock (released after each insert)
{
    let mut db = db_mutex.blocking_lock();
    // ... insert operations ...
} // Lock released here
```

### ISSUE-042 / ISSUE-107: Attachment stored_path not populated ✅ FIXED
**File:** `src-tauri/src/commands/evidence.rs`
```rust
// Lines 314-353 - Attachments saved to disk during file import
let att_dir = Database::get_data_dir().join("cases").join(&case_id).join("attachments");
let _ = std::fs::create_dir_all(&att_dir);
let stored_path_str = if !att.data.is_empty() {
    if std::fs::write(&stored_file_path, &att.data).is_ok() {
        Some(stored_file_path.to_string_lossy().to_string())
    } else { None }
} else { None };

// Line 368 - stored_path included in INSERT
rusqlite::params![..., stored_path_str, ...]
```

### ISSUE-048 / ISSUE-109: Inline image extraction ✅ FIXED
**File:** `src-tauri/src/parser.rs` + `src-tauri/src/commands/evidence.rs`
```rust
// parser.rs line 54 - Field added to struct
pub struct RawAttachment {
    ...
    pub is_inline: bool,
}

// parser.rs lines 648-651 - Detection logic
"content-disposition" => {
    let disp_lower = value.to_lowercase();
    if disp_lower.contains("inline") {
        is_inline = true;
    }
}

// evidence.rs line 369 - Value set in INSERT
if att.is_inline { 1 } else { 0 },
```

### ISSUE-052: IMAP duplicate evidence entries ✅ FIXED
**File:** `src-tauri/src/commands/imap.rs`
```rust
// Lines 109-147 - Evidence ID reused via scope
let evidence_id = {
    let mut db = state.db.lock().await;
    let existing_id: Option<String> = db.conn.query_row(
        "SELECT id FROM evidence_items WHERE case_id = ?1 AND ...",
        ...
    ).ok();
    let evidence_id = existing_id.unwrap_or(evidence_id);
    // ... insert with ON CONFLICT(id) DO UPDATE ...
    evidence_id
};
```

---

## 2. NOT FIXED ISSUES (Still Present)

### ISSUE-001: Passwords stored in plaintext ❌ NOT FIXED
**File:** `src/auth.tsx`
```rust
// Line 32 - Still plaintext
passwordHash: "admin123",
```

### ISSUE-003: No backend authentication ❌ NOT FIXED
**File:** `src-tauri/src/main.rs`
- No auth middleware on commands
- No session verification

### ISSUE-029: Foreign keys disabled during delete ❌ NOT FIXED
**File:** `src-tauri/src/commands/cases.rs`
```rust
// Line 196 - Still present
let _ = db.conn.execute("PRAGMA foreign_keys = OFF;", []);
```

### ISSUE-059: Data directory path inconsistency ❌ NOT FIXED
**File:** `src-tauri/src/commands/imap.rs`
```rust
// Line 306 - Uses "j12-forensic"
.join("j12-forensic")
// But db.rs uses "email-forensic"
```

### ISSUE-043: evidence_delete incomplete ❌ NOT FIXED
**File:** `src-tauri/src/commands/evidence.rs`
- Missing: forensic_artifacts, artifacts_cache, communication_edges, timeline_events, email_tags, email_notes, item_bookmarks deletion

### ISSUE-106: No audit logging for evidence_delete ❌ NOT FIXED
**File:** `src-tauri/src/commands/evidence.rs`
- evidence_delete doesn't call audit_logger

---

## 3. NEW ISSUES INTRODUCED

### NEW-001: IMAP attachment INSERT missing is_inline column ✅ NOT AN ISSUE
**File:** `src-tauri/src/commands/imap.rs`
```rust
// Line 318-332 - INSERT statement NOW includes is_inline
"INSERT OR REPLACE INTO attachments (id, email_id, filename, sha256, mime_type, size_bytes, stored_path, entropy, risk_flags, is_inline, created_at)
 VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,11)",
// is_inline is set at line 330:
if att.is_inline { 1 } else { 0 },
```
**Verdict:** This was incorrectly flagged. The IMAP code DOES include is_inline.

---

## 4. UPDATED ARCHITECTURE HEALTH SCORE

| Area | Before | After | Notes |
|------|--------|-------|-------|
| Authentication | 2/10 | 2/10 | Still frontend-only |
| Database | 4/10 | 6/10 | AI tables added, indexes still missing |
| Security | 3/10 | 4/10 | SQL injection fixed, auth still weak |
| Email Parsing | 5/10 | 7/10 | Attachments saved, inline detected |
| IMAP/POP3 | 5/10 | 6/10 | DB lock fixed, path still inconsistent |
| Analysis | 6/10 | 6/10 | No change |
| AI | 1/10 | 5/10 | Module registered, tables created |
| UI/UX | 6/10 | 6/10 | No change |
| **Overall** | **4/10** | **5/10** | **Improved but not production-ready** |

---

## 5. REMAINING CRITICAL FIXES (Priority Order)

1. **ISSUE-003**: Add backend authentication middleware
2. **ISSUE-001**: Hash passwords properly
3. **ISSUE-059**: Standardize data directory paths
4. **ISSUE-029**: Remove PRAGMA foreign_keys=OFF
5. **ISSUE-043**: Complete evidence_delete cascade
6. **ISSUE-106**: Add audit logging to evidence_delete
7. **NEW-001**: Add is_inline to IMAP attachment INSERT

---

*Report generated: 2026-08-27*
*Auditor: Kilo*

