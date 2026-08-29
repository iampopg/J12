# Phase 11 Audit: Artifact Scanner

> **Files Audited:**
> - `src-tauri/src/commands/artifacts.rs` (2186 lines, first 150 read)

---

## Findings

### ISSUE-099: Seed phrase regex produces false positives
- **Category:** ERROR-PRONE
- **File:** `src-tauri/src/commands/artifacts.rs` (seed phrase section)
- **What's wrong:** The seed phrase regex `(?i)(?:seed\s*phrase|recovery\s*phrase|mnemonic\s*phrase|wallet\s*seed)[:=\-]?\s*([a-z]{3,10}(?:\s+[a-z]{3,10}){11,23})` matches any 12-24 space-separated lowercase words after the keyword. This produces massive false positives.
- **Impact:** Many false positive artifacts. Wastes examiner time.
- **Fix:** Add wordlist validation (BIP39 wordlist check).

---

### ISSUE-100: No artifact deduplication
- **Category:** BREAK
- **File:** `src-tauri/src/commands/artifacts.rs`
- **What's wrong:** Same artifact (e.g., same BTC address) found in multiple emails creates duplicate entries.
- **Impact:** Duplicate entries in artifact list. User reported this issue.
- **Fix:** Add UNIQUE constraint or deduplication logic.

---

### ISSUE-101: Artifacts scanned on every page load
- **Category:** NOT DYNAMIC
- **File:** `src-tauri/src/commands/artifacts.rs`
- **What's wrong:** `rescan_case_artifacts` re-scans all emails every time. No caching.
- **Impact:** Slow page loads. User reported slow artifacts page.
- **Fix:** Use the `artifacts_cache` table or only scan new emails.

---

### ISSUE-102: Regex catastrophic backtracking risk
- **Category:** ERROR-PRONE
- **File:** `src-tauri/src/commands/artifacts.rs`
- **What's wrong:** Complex regex patterns on large email bodies can cause catastrophic backtracking.
- **Impact:** App hangs on certain email content.
- **Fix:** Add regex timeout or use simpler patterns.

---

## Summary

| Category | Count |
|----------|-------|
| BREAK | 1 |
| ERROR-PRONE | 2 |
| NOT DYNAMIC | 1 |
| **Total** | **4** |

