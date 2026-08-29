# Codebase Audit Skill

> **Purpose:** Systematic forensic audit of a Rust+Tauri+React application to find broken, hardcoded, stale, error-prone, or disconnected code.
> **Usage:** "Audit this codebase" or "Redo the audit" or "Find all issues in [component/phase]"

---

## Audit Methodology

### Phase 0: Preparation

1. **Read existing documentation** (if any):
   - `docs/ARCHITECTURE.md` - System overview
   - `docs/SYSTEM_AUDIT.md` - Commands, types, regex patterns
   - `docs/DATABASE_REFERENCE.md` - Schema, tables, relationships
   - `docs/AUDIT_PLAN.md` - Previous audit plan (if exists)

2. **Identify the tech stack**:
   - Frontend: React + TypeScript (check `package.json`)
   - Backend: Rust + Tauri 2 (check `Cargo.toml`, `tauri.conf.json`)
   - Database: SQLite (check `db.rs`)

3. **Map the command registry**:
   - Read `src-tauri/src/main.rs` invoke_handler
   - List all registered Tauri commands
   - Cross-reference with frontend invocations

---

### Phase 1: Static Analysis (No Running App)

#### 1.1 Find all source files
```
Glob: src-tauri/src/**/*.rs
Glob: src/**/*.{ts,tsx}
Glob: src-tauri/src/commands/*.rs
```

#### 1.2 Identify dangerous patterns
Search for these patterns across all Rust files:

| Pattern | Risk | Category |
|---------|------|----------|
| `.unwrap()` | Panic on None/Err | ERROR-PRONE |
| `.expect("...")` | Panic with message | ERROR-PRONE |
| `todo!()` | Unimplemented code | NO BACKEND |
| `unimplemented!()` | Unimplemented code | NO BACKEND |
| `format!("SELECT ... {}", x)` | SQL injection | ERROR-PRONE |
| `&input["field"]["sub"]` | Missing null checks | ERROR-PRONE |
| `state.db.lock().await` held long | UI freeze | BREAK |
| `let _ =` ignoring errors | Silent failures | ERROR-PRONE |
| `hardcoded_value` not from config | Static values | HARDCODED |
| `DEFAULT` constants in code | Magic values | HARDCODED |

#### 1.3 Check command registration
```rust
// In main.rs, verify every command in commands/ is registered
.invoke_handler(tauri::generate_handler![
    // List all expected commands
])
```

Compare with actual files in `src-tauri/src/commands/`.

---

### Phase 2: UI-to-Backend Trace

For every button, form, and interactive element in the frontend:

1. **Find the click handler** in `.tsx` files
2. **Trace to `invoke('command_name', ...)`**
3. **Verify the command exists** in `main.rs` invoke_handler
4. **Verify the command is implemented** in `src-tauri/src/commands/*.rs`
5. **Check error handling** - does the command return `Result<T, String>`?

Mark as:
- **NO BACKEND** if UI exists but command is missing/unimplemented
- **BREAK** if command exists but will fail at runtime

---

### Phase 3: Backend-to-Data Trace

For every Tauri command:

1. **Find all SQL queries** (`SELECT`, `INSERT`, `UPDATE`, `DELETE`)
2. **Verify table names** exist in `db.rs` schema
3. **Verify column names** match struct fields
4. **Check for SQL injection** (string interpolation vs parameterized)
5. **Check error handling** (`.map_err(|e| e.to_string())?` vs `.ok()`)

Mark as:
- **BREAK** if table/column doesn't exist
- **ERROR-PRONE** if SQL injection possible
- **NOT DYNAMIC** if no real-time updates

---

### Phase 4: Cross-Reference Checks

#### 4.1 Database Schema vs Code
```bash
# Extract all CREATE TABLE statements
grep "CREATE TABLE" src-tauri/src/db.rs

# Extract all table names from SQL queries
grep -r "FROM \|INTO \|UPDATE " src-tauri/src/commands/
```

Verify every table in queries exists in schema.

#### 4.2 Structs vs Database
```bash
# Find all struct definitions
grep "pub struct " src-tauri/src/models.rs

# Find all row.get() calls
grep "row.get" src-tauri/src/commands/
```

Verify struct fields match column indices.

#### 4.3 Commands vs Frontend
```bash
# Find all invoke calls
grep -r "invoke(" src/

# Find all command registrations
grep "::command]" src-tauri/src/main.rs
```

Verify every invoke has a registered command.

---

### Phase 5: Security Audit

#### 5.1 Authentication
- [ ] Is auth frontend-only or backend-enforced?
- [ ] Are passwords hashed?
- [ ] Is there session expiration?
- [ ] Is there brute force protection?
- [ ] Are there role-based access controls?

#### 5.2 SQL Injection
- [ ] Any `format!("... {}", user_input)` in SQL?
- [ ] All queries use `?1, ?2` placeholders?

#### 5.3 Data Protection
- [ ] Secrets stored in plaintext?
- [ ] TLS certificate validation?
- [ ] Input validation on all commands?

---

### Phase 6: Forensic Integrity Audit

For forensic applications specifically:

- [ ] Chain of custody logging for all actions
- [ ] SHA-256 hashing of evidence
- [ ] Audit trail immutability
- [ ] Evidence integrity verification
- [ ] Timestamp accuracy (UTC)
- [ ] Read-only evidence access

---

## Issue Classification

### Categories

| Category | Definition | Example |
|----------|------------|---------|
| **BREAK** | Doesn't work at all | Missing table, unimplemented function |
| **ERROR-PRONE** | Will fail under certain conditions | SQL injection, missing error handling |
| **NO BACKEND** | Frontend exists but backend missing | Button with no command |
| **NOT DYNAMIC** | Static/stale data | No real-time updates, cached data |
| **OLD** | Outdated code | TODO comments, deprecated patterns |
| **HARDCODED** | Static values | Magic numbers, default credentials |
| **NO UI** | Backend exists but no frontend | Command with no button |

### Severity Levels

| Level | Action |
|-------|--------|
| **CRITICAL** | Fix immediately - security or data loss |
| **HIGH** | Fix before release - major functionality broken |
| **MEDIUM** | Should fix - partial functionality |
| **LOW** | Nice to have - minor issues |

---

## Output Format

### Per-Phase Report

Create `audit/phase-XX-name/AUDIT_FINDINGS.md`:

```markdown
# Phase X: [Name]

> **Files Audited:**
> - `path/to/file.rs` (lines)

---

## Findings

### ISSUE-XXX: [Short description]
- **Category:** BREAK | ERROR-PRONE | NO BACKEND | NOT DYNAMIC | OLD | HARDCODED | NO UI
- **File:** `path/to/file.rs:123`
- **What's wrong:** [description]
- **Impact:** [what breaks because of this]
- **Fix:** [suggested fix]

---

## Reconfirmation

[Re-read files and confirm findings]

---

## Summary

| Category | Count |
|----------|-------|
| BREAK | X |
| ERROR-PRONE | X |
| ... | ... |
| **Total** | **X** |
```

### Master Report

Create `audit/MASTER_AUDIT_REPORT.md`:

```markdown
# Master Audit Report

## Executive Summary
[Top 5 critical issues]

## Issues by Severity
[CRITICAL, HIGH, MEDIUM, LOW tables]

## Issues by Category
[Count per category]

## Issues by Phase
[Count per phase]

## Top 10 Fixes (Priority Order)
[Numbered list]

## Architecture Health Score
[Score per area out of 10]
```

---

## Segmentation Strategy

Audit in dependency order:

| Layer | Phases |
|-------|--------|
| **Foundation** | Authentication, Case Management, Database |
| **Data Ingestion** | File Import, IMAP, POP3 |
| **Core Features** | Email List, Analysis, Entities, Timeline, Artifacts |
| **User Tools** | Notes/Tags, Attachments, Reports |
| **AI** | AI Integration |
| **Integrity** | Chain of Custody, Audit Log |

---

## Verification Steps

After finding an issue:

1. **Read the actual code** at the reported location
2. **Trace the execution path** - will this code actually run?
3. **Check for guards** - is there error handling that prevents the issue?
4. **Verify the impact** - what actually breaks?
5. **Reconfirm** - re-read the file and confirm the finding

### False Positive Checks

Before marking an issue as TRUE POSITIVE, verify:

- [ ] Is this an intentional architectural decision?
- [ ] Is there error handling that prevents the issue?
- [ ] Is the code path actually reachable?
- [ ] Has this been fixed in recent updates?

---

## Special Patterns to Check

### Rust/Tauri Specific

| Pattern | Location | Risk |
|---------|----------|------|
| `state.db.lock().await` held during network I/O | IMAP/POP3 | UI freeze |
| `format!("SQL {}", input)` | All commands | SQL injection |
| `#[tauri::command]` not in invoke_handler | main.rs | Command unavailable |
| `pub mod ai;` not declared | main.rs | Module inaccessible |
| `CREATE TABLE IF NOT EXISTS` duplicated | db.rs | Confusing but harmless |
| `.ok()` on Result | All commands | Silent failures |

### Forensic Application Specific

| Pattern | Location | Risk |
|---------|----------|------|
| Custody events not logged | case_delete, evidence_delete | Chain of custody gap |
| SHA-256 computed from synthetic data | IMAP/POP3 | Fake integrity |
| Audit log writable/deletable | db.rs | Tamperable evidence |
| Foreign keys disabled | case_delete | Orphaned records |
| Evidence stored_path not set | File import | Cannot open attachments |

---

## Quick Start Commands

```bash
# Find all Tauri commands
grep -r "#\[tauri::command\]" src-tauri/src/ | wc -l

# Find all invoke calls
grep -r "invoke(" src/ | wc -l

# Find all unwrap calls
grep -r "\.unwrap()" src-tauri/src/ | wc -l

# Find all SQL injection risks
grep -r "format!.*SELECT\|format!.*INSERT\|format!.*UPDATE\|format!.*DELETE" src-tauri/src/

# Find all missing error handling
grep -r "\.ok()" src-tauri/src/commands/

# Find all hardcoded values
grep -r "\"admin\"\|default\|hardcoded" src-tauri/src/
```

---

## Audit Checklist

- [ ] Phase 0: Read existing documentation
- [ ] Phase 1: Static analysis (dangerous patterns)
- [ ] Phase 2: UI-to-Backend trace
- [ ] Phase 3: Backend-to-Data trace
- [ ] Phase 4: Cross-reference checks
- [ ] Phase 5: Security audit
- [ ] Phase 6: Forensic integrity audit
- [ ] Reconfirm all findings
- [ ] Write per-phase reports
- [ ] Write master report
- [ ] Prioritize fixes

---

*Skill version: 1.0*
*Created: 2026-08-27*
*For: J12 Forensic Email Investigation Platform*

