# Phase 3 Audit: Database & Migrations

> **Files Audited:**
> - `src-tauri/src/db.rs` (517 lines)

---

## Findings

### ISSUE-023: AI tables never created in schema
- **Category:** BREAK
- **File:** `src-tauri/src/db.rs` (entire init_schema function)
- **What's wrong:** `ai.rs` code references `ai_sessions`, `ai_messages`, `ai_tool_calls`, `ai_audit_log`, `ai_context_snapshots`, `ai_search_index`, `ai_entity_resolutions`, `ai_investigation_plans` tables. **NONE of these tables are created** in `init_schema()`.
- **Impact:** AI commands will fail with "no such table" error. All 25 AI commands are broken at the database level.
- **Fix:** Add CREATE TABLE statements for all 8 AI tables in init_schema().

---

### ISSUE-024: Duplicate CREATE TABLE for forensic_artifacts
- **Category:** ERROR-PRONE
- **File:** `src-tauri/src/db.rs:293-309` and `src-tauri/src/db.rs:318-334`
- **What's wrong:** `forensic_artifacts` table is created twice — once in the main batch (line 293) and once as a "migration" (line 318). The second version is missing `REFERENCES cases(id)` foreign key.
- **Impact:** The second CREATE TABLE IF NOT EXISTS will be a no-op since table already exists, but the code is confusing and the FK constraint is missing from the second definition.
- **Fix:** Remove the duplicate at line 318-334.

---

### ISSUE-025: Duplicate CREATE TABLE for chain_of_custody
- **Category:** ERROR-PRONE
- **File:** `src-tauri/src/db.rs:265-273` and `src-tauri/src/db.rs:358-366`
- **What's wrong:** `chain_of_custody` table is created twice — once in main batch (line 265) with FK, and once as "migration" (line 358) without FK.
- **Impact:** Same as above — no-op but confusing.
- **Fix:** Remove the duplicate at line 358-366.

---

### ISSUE-026: Migrations silently fail
- **Category:** ERROR-PRONE
- **File:** `src-tauri/src/db.rs:369-396` (all ALTER TABLE statements)
- **What's wrong:** All migration statements use `.ok()` which silently ignores errors. If a column already exists, the ALTER TABLE fails but the error is swallowed.
- **Impact:** Cannot distinguish between "column already exists" (ok) and "disk full" (not ok). Migrations are not reliable.
- **Fix:** Check if column exists before ALTER TABLE, or use `IF NOT EXISTS` syntax (SQLite 3.56+).

---

### ISSUE-027: No versioned migration system
- **Category:** OLD
- **File:** `src-tauri/src/db.rs`
- **What's wrong:** No `user_version` tracking, no migration numbering, no way to know which migrations have run. All migrations run on every app startup.
- **Impact:** Cannot safely evolve schema. Old databases may miss new columns.
- **Fix:** Implement proper migration versioning with `PRAGMA user_version`.

---

### ISSUE-028: Duplicate index creation
- **Category:** OLD
- **File:** `src-tauri/src/db.rs:313-366` and `src-tauri/src/db.rs:399-422`
- **What's wrong:** Many indexes are created twice — once in the "PERFORMANCE INDEXES (Phase 6)" block (line 313+) and again in the second block (line 399+). Example: `idx_emails_case_id` appears at line 335 AND line 399.
- **Impact:** Wasted startup time. CREATE INDEX IF NOT EXISTS is idempotent but still unnecessary.
- **Fix:** Remove duplicate index creation blocks.

---

### ISSUE-029: Foreign keys enabled but not enforced on delete
- **Category:** BREAK
- **File:** `src-tauri/src/db.rs:26` vs `src-tauri/src/commands/cases.rs:165`
- **What's wrong:** Schema enables `PRAGMA foreign_keys = ON` but `case_delete` disables it with `PRAGMA foreign_keys = OFF`. The REFERENCES clauses in CREATE TABLE are therefore meaningless.
- **Impact:** Orphaned records possible. No referential integrity.
- **Fix:** Remove `PRAGMA foreign_keys = OFF` from case_delete. Use proper ON DELETE CASCADE in schema.

---

### ISSUE-030: No composite index on emails(case_id, evidence_id)
- **Category:** ERROR-PRONE
- **File:** `src-tauri/src/db.rs` (index section)
- **What's wrong:** Individual indexes on `case_id` and `evidence_id` exist, but no composite index. Many queries filter by both.
- **Impact:** Slow queries when filtering emails by case AND evidence.
- **Fix:** Add `CREATE INDEX idx_emails_case_evidence ON emails(case_id, evidence_id)`.

---

### ISSUE-031: Missing index on audit_log.target_id and timestamp
- **Category:** ERROR-PRONE
- **File:** `src-tauri/src/db.rs` (index section)
- **What's wrong:** `audit_log` table has no index on `target_id` or `timestamp`. Queries filtering by target or time range will be slow.
- **Impact:** Audit log queries slow on large datasets.
- **Fix:** Add indexes on `target_id`, `timestamp`, and `(target_type, target_id)`.

---

### ISSUE-032: No users table for authentication
- **Category:** NO BACKEND
- **File:** `src-tauri/src/db.rs` (schema)
- **What's wrong:** Authentication is frontend-only in localStorage. No `users` table in database. No way to persist users across devices or have proper auth.
- **Impact:** Cannot implement proper authentication. Users lost if localStorage cleared.
- **Fix:** Add `users` table with id, username, password_hash, role, created_at.

---

### ISSUE-033: emails.case_id redundant with evidence_id
- **Category:** OLD
- **File:** `src-tauri/src/db.rs:89-128`
- **What's wrong:** `emails` table has both `evidence_id` and `case_id`. Since `evidence_items` already has `case_id`, this is denormalized. Risk of inconsistency.
- **Impact:** An email could have `case_id` that doesn't match its `evidence_id`'s `case_id`.
- **Fix:** Remove `case_id` from emails, join through evidence_items. Or add a CHECK constraint.

---

### ISSUE-034: No CHECK constraints on enum-like fields
- **Category:** ERROR-PRONE
- **File:** `src-tauri/src/db.rs` (multiple tables)
- **What's wrong:** Fields like `status`, `severity`, `folder_category`, `investigation_type` have no CHECK constraints. Any string can be inserted.
- **Impact:** Data integrity issues. Typos like "opne" instead of "open" go undetected.
- **Fix:** Add CHECK constraints: `CHECK(status IN ('open','closed','archived'))`.

---

### ISSUE-035: artifacts_cache and forensic_artifacts tables overlap
- **Category:** OLD
- **File:** `src-tauri/src/db.rs:275-291` and `src-tauri/src/db.rs:293-309`
- **What's wrong:** Both tables have nearly identical schemas. `artifacts_cache` appears to be an older version that's no longer needed.
- **Impact:** Confusion about which table to query. Data may be split between both.
- **Fix:** Consolidate into single `forensic_artifacts` table, remove `artifacts_cache`.

---

## Reconfirmation

I re-read `db.rs` in full (517 lines). Findings confirmed:
- No AI tables in schema (confirmed by grep for "ai_" returning no results in db.rs)
- Duplicate forensic_artifacts at lines 293 and 318
- Duplicate chain_of_custody at lines 265 and 358
- All ALTER TABLE use `.ok()` (lines 369-396)
- Duplicate indexes (idx_emails_case_id at lines 335 and 399)
- No users table
- No CHECK constraints

Cross-referenced with `ai.rs`:
- `ai_create_session` inserts into `ai_sessions` (line 1467) — table doesn't exist
- `ai_get_session_history` selects from `ai_messages` (line 1486) — table doesn't exist
- `ai_clear_session` deletes from `ai_tool_calls`, `ai_context_snapshots` — tables don't exist

**All 13 findings confirmed.**

---

## Summary

| Category | Count |
|----------|-------|
| BREAK | 2 |
| ERROR-PRONE | 5 |
| OLD | 3 |
| NO BACKEND | 1 |
| **Total** | **11** |

**Severity:** CRITICAL - AI tables completely missing from schema means all AI functionality is broken. Migrations silently failing means database could be in inconsistent state. Foreign keys disabled means no referential integrity.

