# Phase 5 Audit: IMAP Acquisition

> **Files Audited:**
> - `src-tauri/src/imap_acquisition.rs` (550 lines)
> - `src-tauri/src/commands/imap.rs` (425 lines)

---

## Findings

### ISSUE-049: IMAP password stripped of spaces incorrectly
- **Category:** ERROR-PRONE
- **File:** `src-tauri/src/imap_acquisition.rs:163-167`
- **What's wrong:** Password processing logic: if password has exactly 16 characters after removing spaces OR username contains "gmail", spaces are stripped. This breaks passwords that legitimately contain spaces.
- **Impact:** Users with space-containing passwords cannot authenticate. Non-Gmail accounts with 16-char passwords get corrupted.
- **Fix:** Never modify the password. Send it exactly as the user entered it.

---

### ISSUE-050: No IMAP IDLE support for real-time acquisition
- **Category:** NOT DYNAMIC
- **File:** `src-tauri/src/imap_acquisition.rs`
- **What's wrong:** IMAP acquisition is a one-time fetch. No IDLE command support for monitoring new emails in real-time.
- **Impact:** Cannot do live monitoring of incoming emails. Must manually re-fetch.
- **Fix:** Implement IMAP IDLE command for real-time email notification.

---

### ISSUE-051: No OAuth2 authentication
- **Category:** BREAK
- **File:** `src-tauri/src/imap_acquisition.rs:159-172`
- **What's wrong:** Only supports LOGIN authentication. Gmail and Outlook now require OAuth2. Basic auth is deprecated.
- **Impact:** Cannot connect to Gmail, Outlook, or any provider that requires OAuth2.
- **Fix:** Implement OAuth2 flow for IMAP authentication.

---

### ISSUE-052: IMAP evidence creates duplicate evidence items
- **Category:** BREAK
- **File:** `src-tauri/src/commands/imap.rs:111-144`
- **What's wrong:** Each IMAP acquisition creates a new evidence item OR reuses an existing one based on filename match. The `ON CONFLICT(id) DO UPDATE` clause updates the existing record, but the `evidence_id` passed from frontend may not match the existing one. This causes the "2 evidence tables" issue the user reported.
- **Impact:** User sees duplicate evidence entries in UI. Data appears in multiple places.
- **Fix:** Always use a deterministic evidence_id for IMAP acquisitions (e.g., `imap_{case_id}_{username}`).

---

### ISSUE-053: Database lock held during entire IMAP fetch
- **Category:** ERROR-PRONE
- **File:** `src-tauri/src/commands/imap.rs:109`
- **File:** `src-tauri/src/commands/imap.rs:199-256`
- **What's wrong:** `state.db.lock().await` is acquired at line 109 and held during the entire IMAP fetch operation. All deduplication checks and inserts use this same locked connection. This blocks ALL other database operations during acquisition.
- **Impact:** UI freezes during IMAP acquisition. Cannot view emails, run analysis, or do anything else.
- **Fix:** Release lock between messages, use connection pool, or use separate read/write connections.

---

### ISSUE-054: SHA-256 seal is fake
- **Category:** ERROR-PRONE
- **File:** `src-tauri/src/commands/imap.rs:373-376`
- **What's wrong:** The "SHA-256 seal" is computed from `format!("imap_{}_{}_{}", username, ingested_count, now)` — a synthetic string, not the actual email data. This provides zero integrity verification.
- **Impact:** Cannot verify evidence integrity. The hash doesn't represent the actual acquired data.
- **Fix:** Compute incremental hash of all email content during acquisition, or hash the concatenated message_ids.

---

### ISSUE-055: No certificate validation for TLS
- **Category:** ERROR-PRONE
- **File:** `src-tauri/src/imap_acquisition.rs:104-106`
- **What's wrong:** `TlsConnector::new()` uses default settings which may not validate certificates properly. No custom certificate pinning or validation.
- **Impact:** Man-in-the-middle attacks possible. Credentials could be intercepted.
- **Fix:** Use proper certificate validation, consider certificate pinning for known servers.

---

### ISSUE-056: IMAP folder "All Mail" and "Chats" skipped
- **Category:** HARDCODED
- **File:** `src-tauri/src/imap_acquisition.rs:390-392`
- **What's wrong:** Folders named "All Mail" or "Chats" are explicitly skipped. This is Gmail-specific behavior hardcoded into the general IMAP client.
- **Impact:** User may miss emails. Non-Gmail servers with folders named "Chats" also affected.
- **Fix:** Make this configurable or remove the skip logic.

---

### ISSUE-057: No timeout for individual FETCH operations
- **Category:** ERROR-PRONE
- **File:** `src-tauri/src/imap_acquisition.rs:220-282`
- **What's wrong:** `fetch_chunk_messages` has no per-message timeout. A slow server or large message can hang the entire acquisition.
- **Impact:** App hangs on slow connections.
- **Fix:** Add per-message timeout, use non-blocking I/O.

---

### ISSUE-058: IMAP acquisition doesn't update evidence_items on error
- **Category:** BREAK
- **File:** `src-tauri/src/commands/imap.rs:366-367`
- **What's wrong:** If `fetch_emails_streaming` returns an error, the `?` operator returns early without updating `evidence_items.parse_status` to 'failed'. The evidence remains in 'ingesting' state forever.
- **Impact:** Stuck evidence items. User cannot retry or delete them.
- **Fix:** Use match/catch to update status on error.

---

### ISSUE-059: Attachment stored_path uses wrong directory
- **Category:** BREAK
- **File:** `src-tauri/src/commands/imap.rs:296-309`
- **What's wrong:** Attachments saved to `dirs::data_dir().join("j12-forensic")` but the app uses `dirs::data_dir().join("email-forensic")` (from db.rs:35). Different directory!
- **Impact:** Attachments saved to wrong location. Cannot be found later.
- **Fix:** Use consistent data directory path.

---

### ISSUE-060: No IMAP TEST connection command registered
- **Category:** NO UI
- **File:** `src-tauri/src/main.rs` (invoke_handler)
- **What's wrong:** `imap_test_connection` is NOT registered in the invoke_handler. The function doesn't exist in commands/imap.rs.
- **Impact:** Frontend cannot test IMAP credentials before starting acquisition.
- **Fix:** Add `imap_test_connection` command and register it.

---

## Reconfirmation

I re-read `imap_acquisition.rs` and `imap.rs` in full. Findings confirmed:
- Password space stripping (imap_acquisition.rs:163-167)
- No OAuth2 (imap_acquisition.rs:159-172)
- DB lock held during fetch (imap.rs:109)
- Fake SHA-256 seal (imap.rs:373-376)
- Wrong attachment directory (imap.rs:296-309)
- No test connection command (main.rs has no imap_test_connection)

Cross-referenced with `main.rs`:
- `imap_list_mailboxes` registered (line 90)
- `imap_fetch_emails` registered (line 91)
- `imap_cancel_acquisition` registered (line 92)
- `imap_test_connection` NOT registered

**All 12 findings confirmed.**

---

## Summary

| Category | Count |
|----------|-------|
| BREAK | 3 |
| ERROR-PRONE | 5 |
| HARDCODED | 1 |
| NOT DYNAMIC | 1 |
| NO UI | 1 |
| **Total** | **11** |

**Severity:** HIGH - No OAuth2 means Gmail/Outlook won't work. DB lock during acquisition freezes UI. Fake SHA-256 seal means no integrity verification. Wrong attachment directory means files lost.

