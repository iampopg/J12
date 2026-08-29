# Phase 15 Audit: AI Integration

> **Files Audited:**
> - `src-tauri/src/ai.rs` (2926+ lines, referenced functions)
> - `src-tauri/src/commands/mod.rs` (no ai module)

---

## Findings

### ISSUE-114: AI module not registered in main.rs
- **Category:** BREAK
- **File:** `src-tauri/src/main.rs` (invoke_handler)
- **File:** `src-tauri/src/commands/mod.rs`
- **What's wrong:** `ai.rs` exists with 25+ functions but `mod ai;` is NOT declared in main.rs and no AI commands are registered in invoke_handler.
- **Impact:** ALL AI commands are inaccessible from frontend. AI UI is completely disconnected.
- **Fix:** Add `mod ai;` to main.rs and register all AI commands in invoke_handler.

---

### ISSUE-115: AI tables not created in schema
- **Category:** BREAK
- **File:** `src-tauri/src/db.rs` (init_schema)
- **What's wrong:** 8 AI tables (`ai_sessions`, `ai_messages`, `ai_tool_calls`, `ai_audit_log`, `ai_context_snapshots`, `ai_search_index`, `ai_entity_resolutions`, `ai_investigation_plans`) are never created.
- **Impact:** AI commands fail with "no such table" error.
- **Fix:** Add CREATE TABLE statements for all AI tables.

---

### ISSUE-116: ai_create_session will fail
- **Category:** BREAK
- **File:** `src-tauri/src/ai.rs:1461-1477`
- **What's wrong:** `ai_create_session` inserts into `ai_sessions` which doesn't exist.
- **Impact:** Cannot create AI sessions.
- **Fix:** Create the tables first.

---

### ISSUE-117: ai_chat may not work without provider
- **Category:** ERROR-PRONE
- **File:** `src-tauri/src/ai.rs:185`
- **What's wrong:** `ai_chat` function exists but may not have proper error handling for missing API keys or unreachable providers.
- **Impact:** AI chat fails silently.
- **Fix:** Add proper error handling and user feedback.

---

### ISSUE-118: AI investigation plan execution not implemented
- **Category:** NO BACKEND
- **File:** `src-tauri/src/ai.rs` (investigation plan section)
- **What's wrong:** `ai_execute_investigation_plan` may be a stub or not fully implemented.
- **Impact:** Investigation plans cannot be executed.
- **Fix:** Implement plan execution engine.

---

### ISSUE-119: No AI model caching
- **Category:** ERROR-PRONE
- **File:** `src-tauri/src/ai.rs` (model fetching)
- **What's wrong:** `fetch_kiloai_models` and `fetch_openrouter_models` fetch models on every call. No caching.
- **Impact:** Slow AI setup. Rate limiting possible.
- **Fix:** Cache models for 24 hours.

---

## Summary

| Category | Count |
|----------|-------|
| BREAK | 3 |
| ERROR-PRONE | 2 |
| NO BACKEND | 1 |
| **Total** | **6** |

**Severity:** CRITICAL - The entire AI system is disconnected. The module exists, the functions exist, the UI exists, but nothing is wired together.

