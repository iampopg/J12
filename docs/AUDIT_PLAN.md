# J12 Forensic - Comprehensive Audit Plan

> **Goal:** Find everything that's broken, hardcoded, stale, error-prone, or disconnected.
> **Method:** Segment-by-segment, check every UI element against its backend, every backend against its data source.

---

## Audit Checklist Categories

For **every** segment, we check:

| Check | What We Look For |
|-------|------------------|
| **BREAK** | Not linked to all data sources, missing foreign keys, orphaned records |
| **HARDCODED** | Responds only to one specific input, static values, magic numbers |
| **NOT DYNAMIC** | Static UI that doesn't refresh, no real-time updates, cached stale data |
| **OLD/STALE** | Unupdated code, deprecated patterns, TODO/FIXME comments, unused functions |
| **ERROR-PRONE** | Missing error handling, unwrap() without expect, no validation, panic paths |
| **NO UI** | Backend exists but no frontend to access it |
| **NO BACKEND** | UI exists but command is stubbed, unimplemented, or missing |

---

## Segmentation Strategy

We audit in **dependency order** — foundation first, features second, AI last.

```
Layer 1: Foundation
  ├── Phase 1: Authentication & Session
  ├── Phase 2: Case Management
  └── Phase 3: Database & Migrations

Layer 2: Data Ingestion
  ├── Phase 4: File Import & Parsing (EML, MBOX, PST, MSG)
  ├── Phase 5: IMAP Acquisition
  └── Phase 6: POP3 Acquisition

Layer 3: Core Features
  ├── Phase 7: Email List, Search & Filter
  ├── Phase 8: Analysis Engine (headers, spoofing, risk)
  ├── Phase 9: Entity Extraction & Communication Graph
  ├── Phase 10: Timeline & Target Profile
  └── Phase 11: Artifact Scanner & Taxonomy

Layer 4: User Tools
  ├── Phase 12: Notes, Tags & Bookmarks
  ├── Phase 13: Attachments Viewer
  └── Phase 14: Report Generation

Layer 5: AI
  └── Phase 15: AI Integration (all 25 commands)

Layer 6: Integrity
  └── Phase 16: Chain of Custody, Audit Log, Hash Verification
```

---

## Phase 1: Authentication & Session

### What to Check
| Component | File | Check |
|-----------|------|-------|
| Login UI | `LoginPage.tsx` | Does it handle all error states? |
| Auth context | `auth.tsx` | Session persistence? Logout? |
| Backend command | `commands/auth.rs` | Password hashing? Brute force protection? |
| Database | `users` table | Schema matches Rust struct? |

### Questions
- [ ] Is password hashed or plaintext?
- [ ] Does session expire?
- [ ] Can you logout?
- [ ] What happens if DB is locked?
- [ ] Is there a registration flow or only hardcoded users?
- [ ] Are there role checks on the backend or only UI?

---

## Phase 2: Case Management

### What to Check
| Component | File | Check |
|-----------|------|-------|
| Case list UI | `CaseList.tsx` | Empty state? Loading state? Error state? |
| Case workspace | `CaseWorkspace.tsx` | Does it load all case data? |
| Backend | `commands/cases.rs` | All 10 commands implemented? |
| Database | `cases`, `case_notes` tables | FK constraints? Cascading delete? |

### Questions
- [ ] What happens when you delete a case? Are all child records deleted?
- [ ] Can you have two cases with the same name?
- [ ] Is the case list paginated or loads everything?
- [ ] What if a case has 100k emails — does the UI handle it?
- [ ] Are case notes rich text or plain text?

---

## Phase 3: Database & Migrations

### What to Check
| Component | File | Check |
|-----------|------|-------|
| Schema | `db.rs` | All 25 tables created? |
| Migrations | `db.rs` | Versioned? Reversible? |
| Indexes | `db.rs` | 38 indexes match queries? |
| WAL mode | `db.rs` | Checkpointing? |

### Questions
- [ ] Are there any tables in code that don't exist in DB?
- [ ] Are there any tables in DB that aren't in code?
- [ ] Do all foreign keys have indexes?
- [ ] Are there N+1 query patterns?
- [ ] Is there any migration that could fail mid-way?

---

## Phase 4: File Import & Parsing

### What to Check
| Component | File | Check |
|-----------|------|-------|
| Parser | `parser.rs` | EML, MBOX, PST, MSG all work? |
| PST parser | `pst.rs` | Folder hierarchy? Deleted recovery? |
| MSG parser | `parser.rs` | Outlook format? |
| Backend | `commands/emails.rs` | Progress reporting? |
| UI | `CaseWorkspace.tsx` | File picker? Drag-drop? Progress bar? |

### Questions
- [ ] What happens with a corrupted EML file?
- [ ] What happens with a 500MB MBOX?
- [ ] Are all attachments extracted or only some?
- [ ] Is inline image extraction working?
- [ ] What if two emails have the same Message-ID?
- [ ] Is there duplicate detection?

---

## Phase 5: IMAP Acquisition

### What to Check
| Component | File | Check |
|-----------|------|-------|
| IMAP client | `imap_acquisition.rs` | SSL? StartTLS? Cert validation? |
| Backend | `commands/imap.rs` | Cancel works? Progress? |
| UI | `CaseWorkspace.tsx` | Server settings? Mailbox list? |
| State | `AppState.cancel_imap` | Atomic cancel? |

### Questions
- [ ] What if connection drops mid-fetch?
- [ ] Does it resume or restart from 0?
- [ ] Are all folders fetched or only INBOX?
- [ ] Is OAuth2 supported or only password?
- [ ] What if mailbox has 1M emails?
- [ ] Why are there 2 evidence tables showing? (user reported)

---

## Phase 6: POP3 Acquisition

### What to Check
| Component | File | Check |
|-----------|------|-------|
| Backend | `commands/pop3.rs` | Test connection? Fetch? |
| UI | `CaseWorkspace.tsx` | POP3 tab exists? |
| Parsing | Same as IMAP? | Reuses email parser? |

### Questions
- [ ] Is POP3 delete-after-fetch an option?
- [ ] Does it handle POP3 UIDL for incremental fetch?
- [ ] Is there progress reporting?
- [ ] Error handling for wrong credentials?

---

## Phase 7: Email List, Search & Filter

### What to Check
| Component | File | Check |
|-----------|------|-------|
| Email list | `EmailListView.tsx` | Pagination? Sort? Filter? |
| Search | `SearchView.tsx` | All operators work? |
| Backend | `commands/emails.rs` | SQL injection safe? |
| Advanced search | Same | Date range? Risk score? |

### Questions
- [ ] Is search full-text or LIKE query?
- [ ] Does `from:` search display name or address?
- [ ] What if search returns 0 results?
- [ ] Is there a loading skeleton or blank screen?
- [ ] Can you select multiple emails for batch operations?
- [ ] Does the list virtualize for 10k+ rows?

---

## Phase 8: Analysis Engine

### What to Check
| Component | File | Check |
|-----------|------|-------|
| Header analysis | `analysis.rs` | All headers parsed? |
| Auth verification | `analysis.rs` | SPF, DKIM, DMARC? |
| Spoofing detection | `analysis.rs` | Display name vs From? |
| Risk scoring | `analysis.rs` | Formula correct? |
| Backend | `commands/analysis.rs` | Async? Progress? |

### Questions
- [ ] What if headers are malformed?
- [ ] Is DNS lookup for SPF/DMARC async with timeout?
- [ ] Can risk score exceed 100?
- [ ] Are findings deduplicated?
- [ ] What triggers re-analysis?

---

## Phase 9: Entity Extraction & Graph

### What to Check
| Component | File | Check |
|-----------|------|-------|
| Entity list | `EntityDiveView.tsx` | All entity types? |
| Entity detail | Same | Aliases? Communication partners? |
| Graph | `GraphView.tsx` | Renders? Interactive? |
| Backend | `commands/entities.rs` | Graph data correct? |

### Questions
- [ ] How are entities merged (john@example.com vs John Doe <john@example.com>)?
- [ ] Is the graph force-directed or static?
- [ ] Can you click a node to see emails?
- [ ] What if there are 10k entities?

---

## Phase 10: Timeline & Target Profile

### What to Check
| Component | File | Check |
|-----------|------|-------|
| Timeline | `TimelineView.tsx` | Daily/monthly view? |
| Target profile | `TargetProfileView.tsx` | Auto-detect works? |
| Backend | `commands/analysis.rs` | Timeline data correct? |

### Questions
- [ ] Does timeline handle timezone correctly?
- [ ] Can you filter by date range?
- [ ] What if all emails have null date?
- [ ] Is target auto-detect based on frequency or ML?

---

## Phase 11: Artifact Scanner

### What to Check
| Component | File | Check |
|-----------|------|-------|
| Artifact UI | `ArtifactsView.tsx` | All 12 domains show? |
| Scanner | `commands/artifacts.rs` | All regex patterns work? |
| Taxonomy | Same | Subcategories correct? |
| Cache | `artifacts_cache` table | Invalidated on re-scan? |

### Questions
- [ ] Are regex patterns tested against false positives?
- [ ] What if body is 1MB of text — does regex catastrophic backtrack?
- [ ] Is seed phrase regex too broad? (known issue)
- [ ] Are artifacts linked to emails correctly?
- [ ] Can you export artifacts?
- [ ] Why are there duplicate entries? (user reported)

---

## Phase 12: Notes, Tags & Bookmarks

### What to Check
| Component | File | Check |
|-----------|------|-------|
| Email tags | `EmailListView.tsx` | Add/remove works? |
| Case notes | `CaseWorkspace.tsx` | CRUD all work? |
| Bookmarks | `EvidenceLockerView.tsx` | All item types? |
| Backend | `commands/tags.rs`, `commands/bookmarks.rs` | All 16 commands? |

### Questions
- [ ] Are tags per-case or global?
- [ ] Can two users have different tags for same email?
- [ ] What happens to bookmarks when email is deleted?
- [ ] Is there a tag color picker or only presets?

---

## Phase 13: Attachments Viewer

### What to Check
| Component | File | Check |
|-----------|------|-------|
| Attachment list | `AttachmentsView.tsx` | Shows all? |
| Preview | Same | Image preview? |
| Export | Same | Save to disk? |
| Backend | `commands/attachments.rs` | All 7 commands? |

### Questions
- [ ] Can you click to open attachment? (known issue: not working)
- [ ] Is there a hex viewer for binary files?
- [ ] Are attachments stored on disk or in DB?
- [ ] What if attachment path is missing but DB record exists?
- [ ] Can you search by attachment hash?

---

## Phase 14: Report Generation

### What to Check
| Component | File | Check |
|-----------|------|-------|
| Report UI | `ReportView.tsx` | Section selection? |
| PDF export | `commands/reports.rs` | Actually generates PDF? |
| Audit log | Same | Exports correctly? |
| Exhibits | Same | Email selection works? |

### Questions
- [ ] Is PDF generation Rust-side or frontend?
- [ ] What if report is 500 pages?
- [ ] Are exhibits numbered automatically?
- [ ] Can you customize report template?

---

## Phase 15: AI Integration

### What to Check
| Component | File | Check |
|-----------|------|-------|
| AI setup | `AISetupPage.tsx` | Provider selection? |
| AI chat | `AIChatWidget.tsx` | Streaming? Context? |
| Backend | `commands/ai.rs` | All 25 commands? |
| AI tables | 8 tables | Schema correct? |

### Questions
- [ ] Does AI work fully offline with Ollama?
- [ ] Is context window managed (token limit)?
- [ ] Are AI outputs cited to evidence?
- [ ] What happens when AI provider is unreachable?
- [ ] Is there rate limiting?
- [ ] Are AI sessions persisted across app restarts?
- [ ] Does investigation plan actually execute steps?
- [ ] Is entity resolution automatic or manual?

---

## Phase 16: Chain of Custody & Integrity

### What to Check
| Component | File | Check |
|-----------|------|-------|
| Custody UI | `IntegrityView.tsx` | Shows chain? |
| Hash verification | `commands/integrity.rs` | SHA-256 check? |
| Audit log | `audit_log` table | All actions logged? |

### Questions
- [ ] Is custody chain immutable (append-only)?
- [ ] What if hash mismatch — is it flagged?
- [ ] Can you export custody chain as evidence?
- [ ] Are all user actions logged or only some?

---

## Execution Order

We'll do this in **4 passes**:

### Pass 1: Static Analysis (no running app)
- Read every Rust source file
- Read every TypeScript/TSX file
- Cross-reference commands against UI invocations
- Cross-reference DB schema against Rust structs
- Find all `unwrap()`, `expect()`, `todo!()`, `unimplemented!()`
- Find all hardcoded values, magic numbers

### Pass 2: UI-to-Backend Trace
- For every button/menu/click in UI, trace to Tauri command
- For every Tauri command, verify it exists in `main.rs`
- For every command, verify it has error handling
- Mark: UI with no backend, backend with no UI

### Pass 3: Backend-to-Data Trace
- For every command, trace SQL queries
- Verify table names match schema
- Verify column names match structs
- Check for SQL injection (string interpolation)
- Mark: backend with no data source, data with no accessor

### Pass 4: Runtime Testing
- Start the app
- Click every button
- Test every error path
- Test with empty data, large data, malformed data
- Mark: runtime errors, freezes, crashes

---

## Output Format

For each issue found, we document:

```
### ISSUE-XXX: [Short description]
- **Phase:** [1-16]
- **Category:** BREAK | HARDCODED | NOT DYNAMIC | OLD | ERROR-PRONE | NO UI | NO BACKEND
- **File:** path/to/file.rs:123
- **What's wrong:** [description]
- **Impact:** [what breaks because of this]
- **Fix:** [suggested fix]
```

---

## Known Issues (Pre-Discovered)

| Issue | Phase | Category |
|-------|-------|----------|
| 2 duplicate evidence tables in IMAP UI | 5 | BREAK |
| Attachments not opening on click | 13 | BREAK |
| HTML body shows as plain text | 7 | NOT DYNAMIC |
| Seed phrase regex false positives | 11 | ERROR-PRONE |
| Duplicate entries on artifact pages | 11 | BREAK |
| Acquisition methods .md missing | 3 | OLD |

---

*Plan created: 2026-08-27*
*Auditor: Kilo*
*Status: Ready to execute Pass 1*
