# Phase 9 Audit: Entity Extraction & Graph

> **Files Audited:**
> - `src-tauri/src/commands/analysis.rs` (entity sections, lines 580-1088)

---

## Findings

### ISSUE-092: Entity extraction doesn't extract from to_addrs JSON properly
- **Category:** ERROR-PRONE
- **File:** `src-tauri/src/commands/analysis.rs:604-620`
- **What's wrong:** Uses regex on raw JSON string `"[\"email1\",\"email2\"]"` which may not handle all JSON escaping.
- **Impact:** May miss entities or extract malformed emails.
- **Fix:** Parse JSON array properly with serde_json.

---

### ISSUE-093: entity_list creates new IDs for evidence-scoped queries
- **Category:** BREAK
- **File:** `src-tauri/src/commands/analysis.rs:687-703`
- **What's wrong:** When filtering by evidence_id, entity_list creates `id: format!("ent_{}", email_addr)` dynamically instead of using the stored entity ID.
- **Impact:** Entity IDs change between views. Bookmarks may break.
- **Fix:** Always use stored entity IDs from database.

---

### ISSUE-094: graph_data doesn't limit nodes
- **Category:** ERROR-PRONE
- **File:** `src-tauri/src/commands/analysis.rs:1057-1083`
- **What's wrong:** Returns all emails without LIMIT. Large cases will crash the graph renderer.
- **Impact:** Browser hangs on large cases.
- **Fix:** Add LIMIT or aggregate in SQL.

---

### ISSUE-095: communication_edges never populated
- **Category:** NO BACKEND
- **File:** `src-tauri/src/db.rs` (table exists)
- **File:** `src-tauri/src/commands/analysis.rs` (no INSERT into communication_edges)
- **What's wrong:** The `communication_edges` table exists but is never written to. The graph uses on-the-fly queries instead.
- **Impact:** Pre-computed edge table unused. Graph slower than necessary.
- **Fix:** Populate communication_edges during entity extraction.

---

## Summary

| Category | Count |
|----------|-------|
| BREAK | 1 |
| ERROR-PRONE | 2 |
| NO BACKEND | 1 |
| **Total** | **4** |

