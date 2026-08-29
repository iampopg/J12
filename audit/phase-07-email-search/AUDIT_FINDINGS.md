# Phase 7 Audit: Email List, Search & Filter

> **Files Audited:**
> - `src-tauri/src/commands/emails.rs` (586 lines)

---

## Findings

### ISSUE-069: Email list doesn't filter by evidence_id properly
- **Category:** BREAK
- **File:** `src-tauri/src/commands/emails.rs:19-22`
- **What's wrong:** The evidence_id filter uses `as_ref().filter(|s| !s.is_empty() && *s != "all")` but the condition checks `*s != "all"` on a `&&` chain with `as_ref()` which may not work as expected with `Option<String>`.
- **Impact:** May not filter by evidence correctly in some cases.
- **Fix:** Simplify the filter logic.

---

### ISSUE-070: Search is only LIKE query, not full-text
- **Category:** NOT DYNAMIC
- **File:** `src-tauri/src/commands/emails.rs:127-172`
- **What's wrong:** Search uses `LIKE '%query%'` pattern. No FTS5 full-text index. Cannot search for multiple words, phrases, or use boolean operators.
- **Impact:** Slow searches on large datasets. No relevance ranking. Cannot find "john AND jane".
- **Fix:** Implement FTS5 virtual table for full-text search.

---

### ISSUE-071: advanced_search is identical to search
- **Category:** OLD
- **File:** `src-tauri/src/commands/emails.rs:175-177`
- **What's wrong:** `advanced_search` just calls `search(state, input).await`. No advanced functionality at all.
- **Impact:** UI may advertise advanced search but it doesn't exist.
- **Fix:** Implement actual advanced search with operators (from:, to:, subject:, date range, etc.) or remove the command.

---

### ISSUE-072: email_list doesn't fetch headers_raw or body
- **Category:** BREAK
- **File:** `src-tauri/src/commands/emails.rs:56-59`
- **What's wrong:** `email_list` sets `headers_raw: None, body_text: None, body_html: None`. Must call `email_get` for each email to get full data.
- **Impact:** N+1 query problem. Loading 50 emails requires 51 database queries.
- **Fix:** Add a `with_body` parameter to optionally include body in list results.

---

### ISSUE-073: No pagination total count
- **Category:** NOT DYNAMIC
- **File:** `src-tauri/src/commands/emails.rs:11-65`
- **What's wrong:** `email_list` returns emails with LIMIT/OFFSET but never returns total count. Frontend cannot show "Page 1 of 50".
- **Impact:** No pagination UI possible. User doesn't know how many pages exist.
- **Fix:** Return total count alongside results.

---

### ISSUE-074: emails_by_date hardcoded LIMIT 1000
- **Category:** ERROR-PRONE
- **File:** `src-tauri/src/commands/emails.rs:241`, `src-tauri/src/commands/emails.rs:260`
- **What's wrong:** Both queries have `LIMIT 1000` hardcoded. No pagination.
- **Impact:** If more than 1000 emails on a day, results truncated silently.
- **Fix:** Add configurable limit/offset parameters.

---

### ISSUE-075: emails_between uses LIKE for entity matching
- **Category:** ERROR-PRONE
- **File:** `src-tauri/src/commands/emails.rs:282-316`
- **What's wrong:** Uses `LIKE '%entity1%'` for email matching. This matches substrings, not exact emails. "john@x.com" would match "notjohn@x.com".
- **Impact:** False positive matches. Wrong emails returned.
- **Fix:** Use exact matching with JSON extraction or normalize email storage.

---

### ISSUE-076: SQL injection risk in email_list
- **Category:** ERROR-PRONE
- **File:** `src-tauri/src/commands/emails.rs:16-46`
- **What's wrong:** While most parameters use `?` placeholders, the condition `deleted_recovered = 1` is built by string push, not parameterized. If `from_filter` contains SQL, it could be injected (though current code has fixed values).
- **Impact:** Potential SQL injection if from_filter values come from user input in future.
- **Fix:** Use parameterized queries for all conditions.

---

### ISSUE-077: Search doesn't search body_html
- **Category:** BREAK
- **File:** `src-tauri/src/commands/emails.rs:141`
- **What's wrong:** Search only looks at `from_addr, to_addrs, subject, body_text`. Doesn't search `body_html`.
- **Impact:** HTML-only emails are not searchable.
- **Fix:** Add `body_html` to search conditions.

---

### ISSUE-078: email_tags_list has dead code path
- **Category:** OLD
- **File:** `src-tauri/src/commands/emails.rs:382-400`
- **What's wrong:** If both `case_id_val` and `email_id_val` are empty, it returns ALL tags from ALL cases with no filter.
- **Impact:** Security issue - tags from other cases visible.
- **Fix:** Require at least one filter parameter.

---

### ISSUE-079: email_tag_add doesn't use created_by in audit
- **Category:** ERROR-PRONE
- **File:** `src-tauri/src/commands/emails.rs:450-458`
- **What's wrong:** Audit log uses `&created_by` but the actual tag creation uses hardcoded values. The `created_by` field is not stored in the tag record.
- **Impact:** Tag creator not tracked in database.
- **Fix:** Store `created_by` in email_tags table.

---

## Reconfirmation

I re-read `emails.rs` in full. Findings confirmed:
- advanced_search = search (line 176)
- email_list doesn't fetch body (lines 56-59)
- No total count returned
- Search doesn't include body_html (line 141)
- emails_by_date LIMIT 1000 (lines 241, 260)

**All 11 findings confirmed.**

---

## Summary

| Category | Count |
|----------|-------|
| BREAK | 2 |
| ERROR-PRONE | 3 |
| NOT DYNAMIC | 2 |
| OLD | 2 |
| **Total** | **10** |

**Severity:** MEDIUM - Search is basic LIKE query (slow, no FTS). N+1 query problem for email lists. No pagination support. HTML emails not searchable.

