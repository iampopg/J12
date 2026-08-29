# Phase 10 Audit: Timeline & Target Profile

> **Files Audited:**
> - `src-tauri/src/commands/analysis.rs` (timeline_data, target_profile sections)

---

## Findings

### ISSUE-096: timeline_data uses date_sent which may be NULL
- **Category:** ERROR-PRONE
- **File:** `src-tauri/src/commands/analysis.rs:993-1033`
- **What's wrong:** `strftime('%Y-%m-%d', date_sent)` — uses `date_sent` which may be NULL for some emails. Should use `date_sent_utc`.
- **Impact:** Emails with NULL date_sent excluded from timeline.
- **Fix:** Use `date_sent_utc` consistently.

---

### ISSUE-097: target_profile uses LIKE for email matching
- **Category:** ERROR-PRONE
- **File:** `src-tauri/src/commands/cases.rs:473-502`
- **What's wrong:** Uses `format!("%{}%", target_email)` with LIKE. Substring matching.
- **Impact:** Wrong target data returned.
- **Fix:** Use exact matching.

---

### ISSUE-098: timeline_events table never populated
- **Category:** NO BACKEND
- **File:** `src-tauri/src/db.rs` (table exists)
- **What's wrong:** `timeline_events` table exists but is never written to. Timeline uses on-the-fly queries.
- **Impact:** Pre-computed timeline table unused.
- **Fix:** Populate timeline_events during email ingestion.

---

## Summary

| Category | Count |
|----------|-------|
| ERROR-PRONE | 2 |
| NO BACKEND | 1 |
| **Total** | **3** |

