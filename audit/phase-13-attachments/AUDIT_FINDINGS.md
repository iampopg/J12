# Phase 13 Audit: Attachments Viewer

> **Files Audited:**
> - `src-tauri/src/commands/attachments.rs` (602 lines, first 100 read)

---

## Findings

### ISSUE-107: open_attachment_in_system fails when stored_path is NULL
- **Category:** BREAK
- **File:** `src-tauri/src/commands/attachments.rs` (open_attachment_in_system)
- **What's wrong:** During file import parsing, `stored_path` is not set for attachments. Only IMAP acquisition sets it.
- **Impact:** Cannot open attachments from file imports. User reported this issue.
- **Fix:** Save attachments to disk during file import parsing.

---

### ISSUE-108: get_attachment_preview doesn't handle missing files
- **Category:** ERROR-PRONE
- **File:** `src-tauri/src/commands/attachments.rs`
- **What's wrong:** If `stored_path` points to a file that doesn't exist, the function fails without fallback.
- **Impact:** Preview fails silently.
- **Fix:** Check file existence, return error or fallback.

---

### ISSUE-109: No inline image extraction
- **Category:** BREAK
- **File:** `src-tauri/src/commands/evidence.rs:327-341`
- **What's wrong:** Inline images (CID references) are treated as regular attachments with `is_inline = 0`.
- **Impact:** `get_email_inline_images` returns nothing. HTML emails broken.
- **Fix:** Set `is_inline = 1` for parts with Content-ID.

---

### ISSUE-110: Attachment category classification is basic
- **Category:** OLD
- **File:** `src-tauri/src/commands/attachments.rs:27-62`
- **What's wrong:** Category is determined only by file extension. No magic byte detection.
- **Impact:** Misclassified attachments (e.g., .txt file that's actually .exe).
- **Fix:** Add magic byte detection for file type verification.

---

## Summary

| Category | Count |
|----------|-------|
| BREAK | 2 |
| ERROR-PRONE | 1 |
| OLD | 1 |
| **Total** | **4** |

