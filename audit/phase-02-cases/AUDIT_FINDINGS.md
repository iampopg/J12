# Phase 2 Audit: Case Management

> **Files Audited:**
> - `src-tauri/src/commands/cases.rs` (876 lines)
> - `src/pages/CaseListPage.tsx` (296 lines)
> - `src/pages/CaseWorkspace.tsx` (case-related sections)

---

## Findings

### ISSUE-011: SQL Injection in case_update
- **Category:** ERROR-PRONE
- **File:** `src-tauri/src/commands/cases.rs:138-143`
- **What's wrong:** `case_update` builds SQL by string concatenation with only single-quote escaping. The `format!("title='{}'", input.title.replace('\'',"''"))` pattern is vulnerable to SQL injection.
- **Impact:** An attacker could modify or delete any data in the database through the case title, description, or status fields.
- **Fix:** Use parameterized queries with `?1` placeholders instead of string formatting.

---

### ISSUE-012: owner_id hardcoded to "default"
- **Category:** HARDCODED
- **File:** `src-tauri/src/commands/cases.rs:69`, `src-tauri/src/commands/cases.rs:91`, `src-tauri/src/commands/cases.rs:116`
- **What's wrong:** All cases get `owner_id: "default".to_string()`. No actual user ownership is tracked.
- **Impact:** Cannot implement multi-user access control. All cases belong to "default" user.
- **Fix:** Pass authenticated user ID from frontend, store in cases table.

---

### ISSUE-013: No case update audit logging
- **Category:** NO BACKEND
- **File:** `src-tauri/src/commands/cases.rs:134-146`
- **What's wrong:** `case_update` does not call `audit_logger::log_forensic_event`. Only `case_create` logs.
- **Impact:** No record of who changed case details or when. Chain of custody gap.
- **Fix:** Add audit logging to `case_update` function.

---

### ISSUE-014: No case delete audit logging
- **Category:** NO BACKEND
- **File:** `src-tauri/src/commands/cases.rs:148-194`
- **What's wrong:** `case_delete` performs cascade delete of all case data but does NOT log to audit_log or chain_of_custody.
- **Impact:** Complete destruction of forensic evidence with no audit trail. This violates forensic integrity principles.
- **Fix:** Log case deletion to audit_log before performing the delete.

---

### ISSUE-015: Foreign keys disabled during delete
- **Category:** ERROR-PRONE
- **File:** `src-tauri/src/commands/cases.rs:165`, `src-tauri/src/commands/cases.rs:191`
- **What's wrong:** `PRAGMA foreign_keys = OFF` is set before cascade delete, then re-enabled. If the program crashes between these lines, foreign keys remain disabled.
- **Impact:** Database integrity compromised. Orphaned records possible.
- **Fix:** Use proper foreign key constraints with ON DELETE CASCADE in schema, or ensure the re-enable is in a finally block.

---

### ISSUE-016: Case list shows no email count or evidence count
- **Category:** NOT DYNAMIC
- **File:** `src/pages/CaseListPage.tsx:256-288`
- **What's wrong:** Case cards show only title, status, type, and creation date. No indication of how many emails, evidence items, or findings exist.
- **Impact:** User cannot assess case size or activity without opening it.
- **Fix:** Add `case_list` backend query that includes counts, display on cards.

---

### ISSUE-017: No case search or filter
- **Category:** NO UI
- **File:** `src/pages/CaseListPage.tsx`
- **What's wrong:** No search bar, no filter by status, no filter by investigation type, no sort options.
- **Impact:** With many cases, user cannot find a specific case.
- **Fix:** Add search and filter controls to case list page.

---

### ISSUE-018: No case close/archive functionality
- **Category:** NO UI
- **File:** `src/pages/CaseListPage.tsx`, `src/pages/CaseWorkspace.tsx`
- **What's wrong:** Cases can only be "open". No button to close, archive, or mark complete. The `case_update` allows status change but UI has no control for it.
- **Impact:** No case lifecycle management.
- **Fix:** Add status change buttons (Close, Archive, Reopen) in case management view.

---

### ISSUE-019: Case creation doesn't validate working directory
- **Category:** ERROR-PRONE
- **File:** `src-tauri/src/commands/cases.rs:30-37`
- **What's wrong:** `std::fs::create_dir_all` result is discarded with `let _ =`. If directory creation fails (permissions, disk full), the case is still created.
- **Impact:** Case exists but working directory doesn't. All file operations will fail silently.
- **Fix:** Check directory creation result, return error if it fails.

---

### ISSUE-020: Case number not unique
- **Category:** BREAK
- **File:** `src-tauri/src/commands/cases.rs:15`, `src-tauri/src/db.rs` (cases table)
- **What's wrong:** No UNIQUE constraint on `case_number` column. Two cases can have the same number.
- **Impact:** Case number collisions. Cannot use case number as identifier.
- **Fix:** Add UNIQUE constraint on case_number in schema.

---

### ISSUE-021: Case delete confirmation in UI is inadequate
- **Category:** ERROR-PRONE
- **File:** `src/pages/CaseWorkspace.tsx:1920`
- **File:** `src/pages/CaseListPage.tsx` (no delete button at all)
- **What's wrong:** CaseListPage has no delete button. CaseWorkspace has delete but confirmation is browser default.
- **Impact:** Accidental case deletion destroys all evidence permanently.
- **Fix:** Add explicit "TYPE CASE NAME TO CONFIRM" dialog before deletion.

---

### ISSUE-022: Case workspace hardcoded username display
- **Category:** HARDCODED
- **File:** `src/pages/CaseListPage.tsx:111`
- **What's wrong:** Topbar shows `admin · investigator` hardcoded. Not from auth context.
- **Impact:** Always shows "admin" even if logged in as different user.
- **Fix:** Use `useAuth()` to display actual logged-in user.

---

## Reconfirmation

I re-read `cases.rs` in full. Findings confirmed:
- SQL injection in `case_update` (line 138-143)
- `owner_id: "default"` hardcoded (lines 69, 91, 116)
- No audit logging in `case_update` or `case_delete`
- Foreign keys disabled during delete (line 165, 191)

Cross-referenced with `CaseListPage.tsx`:
- No search/filter UI
- No email count on cards
- Hardcoded username display (line 111)

Cross-referenced with `db.rs`:
- `case_number` column has no UNIQUE constraint

**All 12 findings confirmed.**

---

## Summary

| Category | Count |
|----------|-------|
| ERROR-PRONE | 4 |
| HARDCODED | 2 |
| NO BACKEND | 2 |
| NOT DYNAMIC | 1 |
| BREAK | 1 |
| NO UI | 2 |
| **Total** | **12** |

**Severity:** HIGH - SQL injection in case_update is critical. Missing audit logging on delete violates forensic integrity. Hardcoded owner_id prevents multi-user support.

