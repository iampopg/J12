# Phase 12 Audit: Notes, Tags & Bookmarks

> **Files Audited:**
> - `src-tauri/src/commands/emails.rs` (tags/notes sections)
> - `src-tauri/src/commands/bookmarks.rs` (342 lines)

---

## Findings

### ISSUE-103: email_tag_add doesn't store created_by
- **Category:** BREAK
- **File:** `src-tauri/src/commands/emails.rs:444-448`
- **What's wrong:** The `created_by` field is accepted as input but not stored in the INSERT. The column exists in schema but is always NULL.
- **Impact:** Cannot track who created a tag.
- **Fix:** Add `created_by` to INSERT statement.

---

### ISSUE-104: bookmark_check returns Option but frontend expects bool
- **Category:** BREAK
- **File:** `src-tauri/src/commands/bookmarks.rs:298-341`
- **What's wrong:** `bookmark_check` returns `Result<Option<ItemBookmark>, String>` but SYSTEM_AUDIT.md says it returns `bool`.
- **Impact:** Frontend may not handle the response correctly.
- **Fix:** Change return type to `bool` or update frontend.

---

### ISSUE-105: bookmarks_list duplicates tagged emails
- **Category:** BREAK
- **File:** `src-tauri/src/commands/bookmarks.rs:225-294`
- **What's wrong:** `bookmarks_list` combines actual bookmarks with email_tags, creating duplicate entries if an email is both bookmarked and tagged.
- **Impact:** Duplicate entries in Evidence Locker.
- **Fix:** Deduplicate by item_id, not just item_id+label.

---

### ISSUE-106: No audit logging for bookmark operations
- **Category:** NO BACKEND
- **File:** `src-tauri/src/commands/bookmarks.rs`
- **What's wrong:** `bookmark_add`, `bookmark_remove`, `bookmark_check` don't call `audit_logger`.
- **Impact:** No chain of custody for bookmark operations.
- **Fix:** Add audit logging.

---

## Summary

| Category | Count |
|----------|-------|
| BREAK | 2 |
| NO BACKEND | 1 |
| **Total** | **3** |

