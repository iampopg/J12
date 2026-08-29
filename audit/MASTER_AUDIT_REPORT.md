# J12 Forensic - Master Audit Report (Reviewed)

> **Audit Date:** 2026-08-27
> **Review Date:** 2026-08-27
> **Auditor:** Kilo
> **Reviewer:** User
> **Total Issues Found:** 125
> **Phases Audited:** 16

---

## Issue Classification Summary

| Category | Count | Action |
|----------|-------|--------|
| **False Positives (FP)** | 7 | No fix needed - architectural decisions |
| **Already Resolved** | 6 | Fixed in recent updates |
| **Confirmed True Positives (TP)** | 112 | Must be remediated |

---

## 1. False Positives (FP) - No Fix Needed

| ID | Issue | Reason |
|----|-------|--------|
| ISSUE-033 | emails.case_id redundant with evidence_id | **Intentional denormalization** - Eliminates expensive JOINs on 100k+ email filters |
| ISSUE-053 | Database lock held during entire IMAP fetch | **Async streaming** - Lock only acquired briefly for inserts, not over TCP sockets |
| ISSUE-066 | POP3 emits "imap_progress" instead of "pop3_progress" | **Intentional unified HUD** - Single global listener for all protocols |
| ISSUE-072 | email_list doesn't fetch body | **Intentional performance** - Loading 10k HTML bodies would consume GBs of RAM |
| ISSUE-080 | SPF analysis doesn't do DNS lookup | **Forensic standard** - Live DNS lookups violate containment, leak investigator IP |
| ISSUE-081 | DKIM analysis doesn't verify signature | **Forensic standard** - Must evaluate recorded headers at time of transmission |
| ISSUE-082 | DMARC analysis doesn't do DNS lookup | **Forensic standard** - Same as SPF, records may have changed post-incident |

---

## 2. Already Resolved - No Fix Needed

| ID | Issue | Resolution |
|----|-------|------------|
| ISSUE-052 | IMAP duplicate evidence entries | Fixed container reuse by case/account, auto-purged 0-message ghosts |
| ISSUE-099 | Seed phrase regex false positives | Implemented BIP-39 wordlist checksum validation |
| ISSUE-100 | Artifact duplicate entries | Implemented deterministic artifact hashing and deduplication |
| ISSUE-101 | Artifacts re-scanned on every page load | Implemented persistent SQLite caching in forensic_artifacts table |
| ISSUE-108 | Attachment preview fails for uncarved files | Implemented automatic base64 fallback carving from MIME body |
| ISSUE-106/123 | Missing audit logs for case actions | Implemented cryptographic disk audit logging in audit_logger.rs |

---

## 3. Confirmed True Positives (TP) - Must Fix

### Group A: Disconnected AI Subsystem (CRITICAL)

| ID | Issue | File | Fix |
|----|-------|------|-----|
| ISSUE-114 | AI module not registered in main.rs | `src-tauri/src/main.rs` | Add `pub mod ai;` and register all 25 AI commands |
| ISSUE-115 | AI tables not created in schema | `src-tauri/src/db.rs` | Add 8 AI tables: ai_sessions, ai_messages, ai_tool_calls, ai_audit_log, ai_context_snapshots, ai_search_index, ai_entity_resolutions, ai_investigation_plans |
| ISSUE-023 | AI tables never created in schema | `src-tauri/src/db.rs` | Same as ISSUE-115 |

### Group B: Security & SQL Sanitization (CRITICAL)

| ID | Issue | File | Fix |
|----|-------|------|-----|
| ISSUE-011 | SQL injection in case_update | `src-tauri/src/commands/cases.rs:138-143` | Replace string interpolation with parameterized queries |
| ISSUE-001 | Passwords stored in plaintext | `src/auth.tsx:32,121` | Use bcrypt/argon2 hashing |
| ISSUE-002 | No session expiration | `src/auth.tsx:45-51` | Add session timeout (30 min idle) |
| ISSUE-003 | No backend authentication | `src-tauri/src/main.rs` | Add auth middleware, verify session on every command |
| ISSUE-004 | Default credentials hardcoded | `src/pages/LoginPage.tsx:11-12,164` | Remove pre-filled credentials, force password change |
| ISSUE-005 | No brute force protection | `src/auth.tsx:66-90` | Add rate limiting, account lockout |

### Group C: File Import & Attachment Storage (HIGH)

| ID | Issue | File | Fix |
|----|-------|------|-----|
| ISSUE-042 | Attachment stored_path not populated | `src-tauri/src/commands/evidence.rs:327-341` | Save attachments to `<case_dir>/attachments/` and set stored_path |
| ISSUE-107 | open_attachment_in_system fails | `src-tauri/src/commands/attachments.rs` | Same as ISSUE-042 |
| ISSUE-059 | Data directory path inconsistency | `src-tauri/src/commands/imap.rs:296-309` | Standardize to `j12-forensic` everywhere |
| ISSUE-048 | No inline image extraction | `src-tauri/src/commands/evidence.rs:304-342` | Set `is_inline = 1` for CID parts |
| ISSUE-109 | Inline images not extracted | `src-tauri/src/parser.rs:679-698` | Same as ISSUE-048 |

### Group D: Search & Analysis Integrity (HIGH)

| ID | Issue | File | Fix |
|----|-------|------|-----|
| ISSUE-077 | Search doesn't include body_html | `src-tauri/src/commands/emails.rs:141` | Add `OR body_html LIKE ?` to search |
| ISSUE-084 | run_analysis deletes ALL findings | `src-tauri/src/commands/analysis.rs:361` | Only delete auto-generated, preserve manual findings |
| ISSUE-120 | Two custody tables | `src-tauri/src/db.rs` | Consolidate or create unified view |
| ISSUE-121 | custody_chain reads wrong table | `src-tauri/src/commands/analysis.rs:254` | Query both chain_of_custody and custody_events |
| ISSUE-125 | No integrity check on startup | `src-tauri/src/db.rs` | Run `PRAGMA integrity_check` on open |

### Group E: Database Schema (MEDIUM)

| ID | Issue | File | Fix |
|----|-------|------|-----|
| ISSUE-024 | Duplicate CREATE TABLE forensic_artifacts | `src-tauri/src/db.rs:293-309,318-334` | Remove duplicate at line 318 |
| ISSUE-025 | Duplicate CREATE TABLE chain_of_custody | `src-tauri/src/db.rs:265-273,358-366` | Remove duplicate at line 358 |
| ISSUE-026 | Migrations silently fail | `src-tauri/src/db.rs:369-396` | Check column existence before ALTER |
| ISSUE-027 | No versioned migration system | `src-tauri/src/db.rs` | Implement user_version tracking |
| ISSUE-028 | Duplicate index creation | `src-tauri/src/db.rs:313-366,399-422` | Remove duplicate index blocks |
| ISSUE-029 | Foreign keys disabled during delete | `src-tauri/src/commands/cases.rs:165,191` | Remove PRAGMA foreign_keys=OFF |
| ISSUE-030 | No composite index emails(case_id, evidence_id) | `src-tauri/src/db.rs` | Add composite index |
| ISSUE-031 | Missing audit_log indexes | `src-tauri/src/db.rs` | Add indexes on target_id, timestamp |
| ISSUE-032 | No users table | `src-tauri/src/db.rs` | Add users table for backend auth |
| ISSUE-034 | No CHECK constraints | `src-tauri/src/db.rs` | Add CHECK on status, severity, etc. |
| ISSUE-035 | artifacts_cache overlaps forensic_artifacts | `src-tauri/src/db.rs:275-291,293-309` | Consolidate to single table |

### Group F: IMAP/POP3 Issues (MEDIUM)

| ID | Issue | File | Fix |
|----|-------|------|-----|
| ISSUE-049 | Password space stripping | `src-tauri/src/imap_acquisition.rs:163-167` | Never modify user password |
| ISSUE-051 | No OAuth2 authentication | `src-tauri/src/imap_acquisition.rs:159-172` | Implement OAuth2 flow |
| ISSUE-054 | SHA-256 seal is fake | `src-tauri/src/commands/imap.rs:373-376` | Hash actual email content |
| ISSUE-055 | No certificate validation | `src-tauri/src/imap_acquisition.rs:104-106` | Add proper TLS cert validation |
| ISSUE-056 | IMAP folders hardcoded skip | `src-tauri/src/imap_acquisition.rs:390-392` | Make configurable or remove |
| ISSUE-058 | IMAP error doesn't update evidence | `src-tauri/src/commands/imap.rs:366-367` | Update parse_status on error |
| ISSUE-060 | No IMAP test connection | `src-tauri/src/main.rs` | Add imap_test_connection command |
| ISSUE-061 | POP3 password space stripping | `src-tauri/src/commands/pop3.rs:143-157` | Never modify password |
| ISSUE-062 | POP3 no UIDL support | `src-tauri/src/commands/pop3.rs` | Implement UIDL for incremental fetch |
| ISSUE-063 | POP3 no OAuth2 | `src-tauri/src/commands/pop3.rs:110-158` | Implement OAuth2 |
| ISSUE-067 | POP3 attachments not saved to disk | `src-tauri/src/commands/pop3.rs:478-489` | Save attachments like IMAP does |
| ISSUE-068 | POP3 no forensic header parsing | `src-tauri/src/commands/pop3.rs:362-405` | Parse headers like IMAP does |

### Group G: Parsing Limitations (MEDIUM)

| ID | Issue | File | Fix |
|----|-------|------|-----|
| ISSUE-036 | PST/OST parsing not implemented | `src-tauri/src/pst.rs:26-37` | Integrate libpff or document as unsupported |
| ISSUE-037 | MSG parsing not implemented | `src-tauri/src/pst.rs:106-114` | Implement CFB/OLE parser |
| ISSUE-038 | EMLX parsing incomplete | `src-tauri/src/pst.rs:117-231` | Implement proper plist parsing |
| ISSUE-040 | Base64 decode silently fails | `src-tauri/src/parser.rs:718-724` | Return error on decode failure |
| ISSUE-041 | No size limit on parsing | `src-tauri/src/commands/evidence.rs:203-368` | Add streaming/chunked parsing |

### Group H: UI/Feature Gaps (LOW-MEDIUM)

| ID | Issue | File | Fix |
|----|-------|------|-----|
| ISSUE-012 | owner_id hardcoded to "default" | `src-tauri/src/commands/cases.rs:69` | Pass authenticated user ID |
| ISSUE-013 | No case update audit logging | `src-tauri/src/commands/cases.rs:134-146` | Add audit logging |
| ISSUE-014 | No case delete audit logging | `src-tauri/src/commands/cases.rs:148-194` | Add audit logging before delete |
| ISSUE-015 | Foreign keys disabled | `src-tauri/src/commands/cases.rs:165,191` | Remove PRAGMA foreign_keys=OFF |
| ISSUE-016 | Case list no email count | `src/pages/CaseListPage.tsx:256-288` | Add counts to case_list query |
| ISSUE-017 | No case search/filter | `src/pages/CaseListPage.tsx` | Add search/filter controls |
| ISSUE-018 | No case close/archive | `src/pages/CaseListPage.tsx` | Add status change buttons |
| ISSUE-019 | Case creation doesn't validate directory | `src-tauri/src/commands/cases.rs:30-37` | Check create_dir_all result |
| ISSUE-020 | Case number not unique | `src-tauri/src/db.rs` | Add UNIQUE constraint |
| ISSUE-021 | Case delete confirmation inadequate | `src/pages/CaseWorkspace.tsx:1920` | Add explicit confirmation dialog |
| ISSUE-022 | Hardcoded username display | `src/pages/CaseListPage.tsx:111` | Use useAuth() |
| ISSUE-039 | No progress during parsing | `src-tauri/src/commands/evidence.rs:203-368` | Emit progress events |
| ISSUE-043 | evidence_delete incomplete | `src-tauri/src/commands/evidence.rs:139-182` | Delete from all child tables |
| ISSUE-044 | File dialog includes .db/.txt | `src-tauri/src/commands/evidence.rs:378-385` | Remove from filter |
| ISSUE-045 | acquired_by hardcoded | `src-tauri/src/commands/evidence.rs:25` | Pass from auth context |
| ISSUE-046 | Ghost cleanup in evidence_list | `src-tauri/src/commands/evidence.rs:66-79` | Fix root cause |
| ISSUE-047 | Re-parse creates duplicates | `src-tauri/src/commands/evidence.rs:259-343` | Use INSERT OR REPLACE or delete first |
| ISSUE-069 | evidence_id filter may not work | `src-tauri/src/commands/emails.rs:19-22` | Simplify filter logic |
| ISSUE-070 | Search is LIKE not FTS | `src-tauri/src/commands/emails.rs:127-172` | Implement FTS5 |
| ISSUE-071 | advanced_search = search | `src-tauri/src/commands/emails.rs:175-177` | Implement or remove |
| ISSUE-073 | No pagination total count | `src-tauri/src/commands/emails.rs:11-65` | Return total count |
| ISSUE-074 | emails_by_date LIMIT 1000 | `src-tauri/src/commands/emails.rs:241,260` | Add configurable limit |
| ISSUE-075 | emails_between LIKE matching | `src-tauri/src/commands/emails.rs:282-396` | Use exact matching |
| ISSUE-076 | SQL injection risk in email_list | `src-tauri/src/commands/emails.rs:16-46` | Parameterize all conditions |
| ISSUE-078 | email_tags_list dead code | `src-tauri/src/commands/emails.rs:382-400` | Require filter parameter |
| ISSUE-079 | email_tag_add doesn't store created_by | `src-tauri/src/commands/emails.rs:444-448` | Add to INSERT |
| ISSUE-083 | run_analysis loads all emails | `src-tauri/src/commands/analysis.rs:280-322` | Process in batches |
| ISSUE-085 | run_analysis no progress | `src-tauri/src/commands/analysis.rs:273-408` | Emit progress events |
| ISSUE-086 | Entity extraction uses regex | `src-tauri/src/commands/analysis.rs:604-620` | Parse JSON properly |
| ISSUE-087 | entity_dive LIKE matching | `src-tauri/src/commands/analysis.rs:734` | Use exact matching |
| ISSUE-088 | dashboard 18+ queries | `src-tauri/src/commands/analysis.rs:135-223` | Use single CTE query |
| ISSUE-089 | graph_data no LIMIT | `src-tauri/src/commands/analysis.rs:1057-1083` | Add LIMIT or aggregate |
| ISSUE-090 | custody_chain wrong table | `src-tauri/src/commands/analysis.rs:254` | Query both tables |
| ISSUE-091 | dashboard entity_count incomplete | `src-tauri/src/commands/analysis.rs:187` | Count recipients too |
| ISSUE-092 | Entity regex on JSON | `src-tauri/src/commands/analysis.rs:604-620` | Parse JSON properly |
| ISSUE-093 | entity_list dynamic IDs | `src-tauri/src/commands/analysis.rs:687-703` | Use stored IDs |
| ISSUE-094 | graph_data no limit | `src-tauri/src/commands/analysis.rs:1057-1083` | Add LIMIT |
| ISSUE-095 | communication_edges never populated | `src-tauri/src/commands/analysis.rs` | Populate during extraction |
| ISSUE-096 | timeline uses date_sent | `src-tauri/src/commands/analysis.rs:993-1033` | Use date_sent_utc |
| ISSUE-097 | target_profile LIKE matching | `src-tauri/src/commands/cases.rs:473-502` | Use exact matching |
| ISSUE-098 | timeline_events never populated | `src-tauri/src/db.rs` | Populate during ingestion |
| ISSUE-102 | Regex catastrophic backtracking | `src-tauri/src/commands/artifacts.rs` | Add timeout |
| ISSUE-103 | email_tag_add no created_by | `src-tauri/src/commands/emails.rs:444-448` | Store created_by |
| ISSUE-104 | bookmark_check return type | `src-tauri/src/commands/bookmarks.rs:298-341` | Fix return type |
| ISSUE-105 | bookmarks_list duplicates | `src-tauri/src/commands/bookmarks.rs:225-294` | Deduplicate by item_id |
| ISSUE-110 | Attachment category basic | `src-tauri/src/commands/attachments.rs:27-62` | Add magic byte detection |
| ISSUE-111 | PDF export may not work | `src-tauri/src/commands/reports.rs` | Add PDF library |
| ISSUE-112 | Report missing attachments | `src-tauri/src/commands/reports.rs` | Add attachment manifest |
| ISSUE-113 | No report template UI | `src/pages/CaseWorkspace.tsx` | Add template editor |
| ISSUE-116 | ai_create_session fails | `src-tauri/src/ai.rs:1461-1477` | Create tables first |
| ISSUE-117 | ai_chat error handling | `src-tauri/src/ai.rs:185` | Add proper error handling |
| ISSUE-118 | AI plan execution | `src-tauri/src/ai.rs` | Implement execution |
| ISSUE-119 | No AI model caching | `src-tauri/src/ai.rs` | Cache for 24h |
| ISSUE-122 | Audit log not immutable | `src-tauri/src/db.rs` | Add triggers |
| ISSUE-124 | Hash verification limited | `src-tauri/src/commands/evidence.rs:407-433` | Store per-email hashes |

---

## 4. Implementation Plan

### Step 1: Database Schema & AI Tables
- Update `src-tauri/src/db.rs` to create 8 AI tables
- Run `PRAGMA integrity_check` on startup
- Clean up duplicate table definitions
- Add missing composite indexes

### Step 2: Register AI Subsystem
- Add `pub mod ai;` to `src-tauri/src/main.rs`
- Register all 25 AI Tauri commands

### Step 3: Security & Query Parameterization
- Rewrite `case_update` to use parameterized SQL
- Add password hashing (bcrypt/argon2)
- Add session expiration

### Step 4: Attachment Extraction to Disk
- In `parse_evidence`, write attachments to `<case_dir>/attachments/<sha256>_<filename>`
- Set `stored_path` column
- Detect inline CID attachments, set `is_inline = 1`

### Step 5: Custody & Search Enhancement
- Unify custody_chain to query both tables
- Add `body_html` to search
- Preserve reviewed findings during re-analysis

---

## 5. Verification Plan

1. **Compilation**: `cargo check` in src-tauri, `npm run build` in root
2. **AI Verification**: Launch app, open AI Chat, verify model fetching and chat
3. **Attachment Opener**: Import EML/MBOX, click attachment, verify system open
4. **Security Check**: Verify case_update with special characters

---

*Report generated: 2026-08-27*
*Auditor: Kilo*
*Status: Reviewed and classified*

