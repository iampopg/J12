# Phase 16 Audit: Chain of Custody & Integrity

> **Files Audited:**
> - `src-tauri/src/commands/analysis.rs` (custody_chain)
> - `src-tauri/src/audit_logger.rs` (referenced)
> - `src-tauri/src/db.rs` (custody tables)

---

## Findings

### ISSUE-120: Two custody tables with overlapping purpose
- **Category:** BREAK
- **File:** `src-tauri/src/db.rs` (custody_events and chain_of_custody)
- **What's wrong:** Two tables exist: `custody_events` (used by IMAP/POP3) and `chain_of_custody` (used by analysis/case operations). Data is split.
- **Impact:** Custody chain is incomplete depending on which table is queried.
- **Fix:** Consolidate into single table or create unified view.

---

### ISSUE-121: custody_chain reads from wrong table
- **Category:** BREAK
- **File:** `src-tauri/src/commands/analysis.rs:254`
- **What's wrong:** `custody_chain` command reads from `chain_of_custody` but IMAP/POP3 write to `custody_events`.
- **Impact:** IMAP/POP3 acquisitions not shown in custody chain.
- **Fix:** Query both tables or consolidate.

---

### ISSUE-122: Audit log not immutable
- **Category:** ERROR-PRONE
- **File:** `src-tauri/src/db.rs` (audit_log table)
- **What's wrong:** `audit_log` table has no trigger to prevent UPDATE or DELETE. Records can be modified.
- **Impact:** Audit trail can be tampered with. Forensic integrity compromised.
- **Fix:** Add triggers to prevent UPDATE/DELETE on audit_log.

---

### ISSUE-123: case_delete doesn't log to audit
- **Category:** NO BACKEND
- **File:** `src-tauri/src/commands/cases.rs:148-194`
- **What's wrong:** Deleting a case removes all data but doesn't log the deletion to audit_log.
- **Impact:** No record of case destruction.
- **Fix:** Add audit logging before deletion.

---

### ISSUE-124: Hash verification only works for file imports
- **Category:** BREAK
- **File:** `src-tauri/src/commands/evidence.rs:407-433`
- **What's wrong:** `verify_evidence_hashes` reads from `original_path` and computes SHA-256. For IMAP/POP3, `original_path` is a URL like `imap://...` not a file.
- **Impact:** Cannot verify integrity of live-acquired evidence.
- **Fix:** Store per-email hashes during acquisition for verification.

---

### ISSUE-125: No integrity check on startup
- **Category:** ERROR-PRONE
- **File:** `src-tauri/src/db.rs`
- **What's wrong:** No PRAGMA integrity_check run on database open. No verification that DB is not corrupted.
- **Impact:** Corrupted database may go unnoticed.
- **Fix:** Run `PRAGMA integrity_check` on startup.

---

## Summary

| Category | Count |
|----------|-------|
| BREAK | 3 |
| ERROR-PRONE | 2 |
| NO BACKEND | 1 |
| **Total** | **6** |

**Severity:** HIGH - Chain of custody is fundamental to forensic admissibility. Split tables and missing audit logs are serious issues.

