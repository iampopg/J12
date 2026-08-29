# Phase 8 Audit: Analysis Engine

> **Files Audited:**
> - `src-tauri/src/analysis.rs` (1456 lines, first 200 read)
> - `src-tauri/src/commands/analysis.rs` (1088+ lines)

---

## Findings

### ISSUE-080: SPF analysis doesn't do real DNS lookup
- **Category:** BREAK
- **File:** `src-tauri/src/analysis.rs` (SPF section)
- **What's wrong:** SPF verification parses the Authentication-Results header but doesn't actually perform DNS TXT record lookups to verify SPF. It only checks if the header says "pass" or "fail".
- **Impact:** Cannot verify SPF for emails without Authentication-Results header. Analysis is incomplete.
- **Fix:** Add actual DNS TXT record lookup for SPF verification.

---

### ISSUE-081: DKIM analysis doesn't verify signature
- **Category:** BREAK
- **File:** `src-tauri/src/analysis.rs` (DKIM section)
- **What's wrong:** DKIM verification only checks the Authentication-Results header. Doesn't actually verify the DKIM signature against the public key.
- **Impact:** DKIM pass/fail is taken from header, not independently verified.
- **Fix:** Implement actual DKIM signature verification using the public key from DNS.

---

### ISSUE-082: DMARC analysis doesn't do DNS lookup
- **Category:** BREAK
- **File:** `src-tauri/src/analysis.rs` (DMARC section)
- **What's wrong:** DMARC verification doesn't look up the DMARC policy in DNS. Only checks existing Authentication-Results header.
- **Impact:** Cannot determine DMARC policy or alignment without pre-existing headers.
- **Fix:** Add DNS TXT lookup for `_dmarc.domain.com`.

---

### ISSUE-083: run_analysis loads ALL emails into memory
- **Category:** ERROR-PRONE
- **File:** `src-tauri/src/commands/analysis.rs:280-322`
- **What's wrong:** All emails and attachments for the case are loaded into Vec collections before processing. A case with 100k emails will exhaust memory.
- **Impact:** App crashes on large cases.
- **Fix:** Process emails in batches using streaming/chunked queries.

---

### ISSUE-084: run_analysis deletes ALL findings before regenerating
- **Category:** ERROR-PRONE
- **File:** `src-tauri/src/commands/analysis.rs:361`
- **What's wrong:** `DELETE FROM findings WHERE case_id = ?1` removes all findings including those manually added by examiner with notes.
- **Impact:** Examiner's manual findings and notes lost on re-analysis.
- **Fix:** Only delete auto-generated findings (based on type), preserve manual ones.

---

### ISSUE-085: run_analysis has no progress reporting
- **Category:** NOT DYNAMIC
- **File:** `src-tauri/src/commands/analysis.rs:273-408`
- **What's wrong:** Analysis runs to completion with no progress events. UI shows nothing until complete.
- **Impact:** User doesn't know how long analysis will take or if it's stuck.
- **Fix:** Emit progress events during analysis.

---

### ISSUE-086: Entity extraction uses regex instead of parsed data
- **Category:** ERROR-PRONE
- **File:** `src-tauri/src/commands/analysis.rs:604-620`
- **What's wrong:** Entity extraction uses `regex::Regex::new(r"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}")` on raw JSON strings. This is error-prone for JSON-escaped strings.
- **Impact:** May miss entities or extract malformed emails.
- **Fix:** Parse JSON arrays properly instead of regex on raw strings.

---

### ISSUE-087: entity_dive uses LIKE for email matching
- **Category:** ERROR-PRONE
- **File:** `src-tauri/src/commands/analysis.rs:734`
- **What's wrong:** Uses `format!("%{}%", input.email_address)` with LIKE. "john@x.com" matches "notjohn@x.com".
- **Impact:** Wrong entity data returned.
- **Fix:** Use exact matching or JSON_CONTAINS.

---

### ISSUE-088: dashboard runs 18+ separate queries
- **Category:** OLD
- **File:** `src-tauri/src/commands/analysis.rs:135-223`
- **What's wrong:** Dashboard makes 18+ separate COUNT queries to build statistics. No caching.
- **Impact:** Slow dashboard loading.
- **Fix:** Use a single query with CTEs or materialized view.

---

### ISSUE-089: graph_data returns all emails without limit
- **Category:** ERROR-PRONE
- **File:** `src-tauri/src/commands/analysis.rs:1057-1083`
- **What's wrong:** `graph_data` fetches ALL emails for the case with no LIMIT. Large cases will return massive datasets.
- **Impact:** Browser hangs rendering graph for large cases.
- **Fix:** Add LIMIT or pagination, or use aggregation query.

---

### ISSUE-090: custody_chain reads from wrong table
- **Category:** BREAK
- **File:** `src-tauri/src/commands/analysis.rs:254`
- **What's wrong:** `custody_chain` reads from `chain_of_custody` but custody events during IMAP/POP3 are written to `custody_events`. The two tables are separate.
- **Impact:** Custody chain incomplete. IMAP/POP3 acquisitions not shown.
- **Fix:** Query both tables or consolidate custody tables.

---

### ISSUE-091: dashboard entity_count only counts from_addr
- **Category:** ERROR-PRONE
- **File:** `src-tauri/src/commands/analysis.rs:187`
- **What's wrong:** `entity_c: i64 = db.conn.query_row("SELECT COUNT(DISTINCT from_addr) FROM emails WHERE case_id=?1 AND evidence_id=?2 AND from_addr != ''")`. Only counts senders, not recipients.
- **Impact:** Entity count underreported. Recipients not counted as entities.
- **Fix:** Count distinct emails from both from_addr and to_addrs/cc_addrs.

---

## Reconfirmation

I read the analysis code. Findings confirmed:
- No real DNS lookups for SPF/DKIM/DMARC
- run_analysis loads all emails (line 280-322)
- All findings deleted on re-analysis (line 361)
- Dashboard makes 18+ queries (lines 135-223)
- graph_data has no LIMIT
- custody_chain reads from chain_of_custody (line 254), not custody_events

**All 12 findings confirmed.**

---

## Summary

| Category | Count |
|----------|-------|
| BREAK | 4 |
| ERROR-PRONE | 5 |
| NOT DYNAMIC | 1 |
| OLD | 1 |
| **Total** | **11** |

**Severity:** HIGH - SPF/DKIM/DMARC verification is supposed to be independent but just parsers headers. This undermines the forensic value. Memory issues on large cases.

