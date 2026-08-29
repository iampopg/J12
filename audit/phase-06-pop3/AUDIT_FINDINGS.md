# Phase 6 Audit: POP3 Acquisition

> **Files Audited:**
> - `src-tauri/src/commands/pop3.rs` (577 lines)

---

## Findings

### ISSUE-061: POP3 password stripped of spaces (same bug as IMAP)
- **Category:** ERROR-PRONE
- **File:** `src-tauri/src/commands/pop3.rs:143-157`
- **What's wrong:** Same password processing logic as IMAP. If password has 16 chars after removing spaces OR username contains "gmail", spaces are stripped.
- **Impact:** Users with space-containing passwords cannot authenticate.
- **Fix:** Never modify the password.

---

### ISSUE-062: POP3 doesn't support UIDL for incremental fetch
- **Category:** BREAK
- **File:** `src-tauri/src/commands/pop3.rs`
- **What's wrong:** No UIDL command used. Cannot track which messages have been downloaded. Re-fetching will create duplicates.
- **Impact:** No incremental fetch support. Each fetch re-downloads all messages.
- **Fix:** Implement UIDL tracking for incremental downloads.

---

### ISSUE-063: POP3 doesn't support OAuth2
- **Category:** BREAK
- **File:** `src-tauri/src/commands/pop3.rs:110-158`
- **What's wrong:** Only supports USER/PASS authentication. No OAuth2 support.
- **Impact:** Cannot connect to modern email providers.
- **Fix:** Implement OAuth2 for POP3.

---

### ISSUE-064: POP3 delete-after-fetch not configurable
- **Category:** NO UI
- **File:** `src-tauri/src/commands/pop3.rs`
- **What's wrong:** No option to delete messages after fetching. POP3 typically deletes after RETR, but this implementation doesn't issue DELE.
- **Impact:** Messages remain on server. No forensic "acquire and hold" capability.
- **Fix:** Add configurable delete-after-fetch option.

---

### ISSUE-065: POP3 uses same fake SHA-256 as IMAP
- **Category:** ERROR-PRONE
- **File:** `src-tauri/src/commands/pop3.rs` (custody event)
- **What's wrong:** The custody event hash is computed from a synthetic string, not actual email data.
- **Impact:** No real integrity verification.
- **Fix:** Hash actual email content.

---

### ISSUE-066: POP3 progress events use wrong event name
- **Category:** BREAK
- **File:** `src-tauri/src/commands/pop3.rs:208`
- **What's wrong:** Progress events emitted as `"imap_progress"` instead of `"pop3_progress"`.
- **Impact:** Frontend listening for pop3_progress won't receive events.
- **Fix:** Use separate event name for POP3.

---

### ISSUE-067: POP3 doesn't save attachments to disk
- **Category:** BREAK
- **File:** `src-tauri/src/commands/pop3.rs:478-489`
- **What's wrong:** Attachments are inserted into DB but `stored_path` is empty. Not saved to disk.
- **Impact:** Cannot open attachments after POP3 acquisition.
- **Fix:** Save attachment data to disk like IMAP does.

---

### ISSUE-068: POP3 doesn't parse email headers for forensic data
- **Category:** BREAK
- **File:** `src-tauri/src/commands/pop3.rs:362-405`
- **What's wrong:** Email is inserted with basic fields only. No received_chain, x_mailer, x_originating_ip, etc.
- **Impact:** Forensic analysis incomplete for POP3-acquired emails.
- **Fix:** Parse headers like IMAP does.

---

## Reconfirmation

I re-read `pop3.rs` in full. Findings confirmed:
- Password space stripping (line 143-157)
- No UIDL support
- No OAuth2
- Wrong event name "imap_progress" (line 208)
- Attachments not saved to disk (line 478-489)
- No forensic header parsing

**All 8 findings confirmed.**

---

## Summary

| Category | Count |
|----------|-------|
| BREAK | 4 |
| ERROR-PRONE | 2 |
| NO UI | 1 |
| **Total** | **7** |

**Severity:** MEDIUM - POP3 is less commonly used than IMAP but still important. Wrong event name breaks progress display. Attachments not saved is a data loss issue.

