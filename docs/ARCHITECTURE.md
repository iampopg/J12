# J12 Forensic - Architecture

## System Overview

J12 Forensic is a desktop email forensic investigation platform built with Tauri (Rust backend + React frontend). It provides court-admissible email analysis with AI-powered investigation assistance.

## Technology Stack

| Layer | Technology | Purpose |
|-------|------------|---------|
| **Frontend** | React 18 + TypeScript | User interface |
| **Build Tool** | Vite 6 | Fast development and building |
| **Backend** | Rust + Tauri 2 | Native desktop app with system access |
| **Database** | SQLite 3 with FTS5 | Local data storage + full-text search |
| **Email Parsing** | Custom Rust parsers | EML, MBOX, PST, MSG support |
| **IMAP** | imap crate + OAuth2 | Live email acquisition (Google, M365) |
| **Analysis** | Custom Rust engines | Header analysis, spoofing detection |
| **AI** | Ollama, OpenAI, Anthropic, OpenRouter | Investigation assistance |
| **OCR** | macOS Vision + Tesseract | Image text extraction |
| **Documents** | Custom Rust extractors | PDF, DOCX, XLSX, PPTX parsing |

## Application Layers

### 1. Presentation Layer (Frontend)

```
┌─────────────────────────────────────────────────────────────────┐
│                     React Application                           │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐          │
│  │   Pages      │  │   Views      │  │  Components  │          │
│  │              │  │              │  │              │          │
│  │ LoginPage    │  │ EmailList    │  │ AIChatWidget │          │
│  │ CaseList     │  │ SearchView   │  │ BookmarkBtn  │          │
│  │ CaseWorkspace│  │ EntityDive   │  │ EmailModal   │          │
│  │              │  │ Timeline     │  │ Logo         │          │
│  │              │  │ Graph        │  │ RichViewer   │          │
│  │              │  │ Artifacts    │  │ Footer       │          │
│  │              │  │ Findings     │  │              │          │
│  │              │  │ Report       │  │              │          │
│  └──────────────┘  └──────────────┘  └──────────────┘          │
│                                                                  │
│  ┌──────────────┐  ┌──────────────┐                             │
│  │ Auth Context │  │ Scan State   │                             │
│  └──────────────┘  └──────────────┘                             │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

### 2. Command Layer (Tauri Bridge)

```
┌─────────────────────────────────────────────────────────────────┐
│                    Tauri IPC Bridge                              │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  invoke('case_create', { title: '...' })                         │
│  invoke('email_list', { caseId: '...' })                         │
│  invoke('ai_chat', { message: '...' })                           │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

### 3. Business Logic Layer (Rust Backend)

```
┌─────────────────────────────────────────────────────────────────┐
│                    Rust Backend                                  │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │                    Command Handlers                        │   │
│  │  cases/ │ emails/ │ analysis/ │ artifacts/ │ attachments/  │   │
│  │  imap.rs │ pop3.rs │ evidence.rs │ bookmarks.rs │ reports/ │   │
│  └──────────────────────────────────────────────────────────┘   │
│                                                                  │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │                    Analysis Engines                        │   │
│  │  Header Analysis │ Auth Verification │ Spoofing Detection │   │
│  │  Risk Scoring    │ Entity Extraction │ Artifact Scanner   │   │
│  │  FTS5 Search     │ Graph Analysis     │ Findings Manager  │   │
│  └──────────────────────────────────────────────────────────┘   │
│                                                                  │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │                    Document Processing                     │   │
│  │  PDF Extractor │ DOCX Parser │ XLSX Parser │ PPTX Parser  │   │
│  │  OCR Engine (macOS Vision + Tesseract fallback)           │   │
│  │  Attachment Text Extraction                               │   │
│  └──────────────────────────────────────────────────────────┘   │
│                                                                  │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │                    Email Acquisition                       │   │
│  │  IMAP Client (Password + OAuth2) │ POP3 Client            │   │
│  │  OAuth2: Google Workspace │ Microsoft 365                  │   │
│  │  SASL XOAUTH2 (RFC 7628) │ Device Code Flow (RFC 8628)    │   │
│  └──────────────────────────────────────────────────────────┘   │
│                                                                  │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │                    AI Integration                          │   │
│  │  Chat │ Search │ Explain │ Plan │ Analyze │ Generate     │   │
│  │  Context Manager │ Tool Runner │ Report Generator         │   │
│  └──────────────────────────────────────────────────────────┘   │
│                                                                  │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │                    Data Access Layer                       │   │
│  │  db/ (schema, migrations, utils)                          │   │
│  └──────────────────────────────────────────────────────────┘   │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

### 4. Data Layer (SQLite + FTS5)

```
┌─────────────────────────────────────────────────────────────────┐
                    SQLite Database + FTS5 Full-Text Search
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │                    Core Tables                             │   │
│  │  cases │ evidence_items │ emails │ attachments            │   │
│  └──────────────────────────────────────────────────────────┘   │
│                                                                  │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │                    FTS5 Virtual Table                      │   │
│  │  emails_fts (subject, from_addr, to_addrs, body_text,     │   │
│  │              attachment_text)                             │   │
│  │  - Porter Stemmer algorithm                               │   │
│  │  - Boolean queries (AND, OR, NOT)                         │   │
│  │  - Proximity search (NEAR/5)                              │   │
│  │  - Hit highlighting snippets                              │   │
│  │  - BM25 relevance ranking                                 │   │
│  │  - Auto-sync triggers (INSERT, UPDATE, DELETE)            │   │
│  └──────────────────────────────────────────────────────────┘   │
│                                                                  │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │                    Analysis Tables                         │   │
│  │  findings │ entities │ communication_edges │ timeline     │   │
│  └──────────────────────────────────────────────────────────┘   │
│                                                                  │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │                    Artifact Tables                         │   │
│  │  forensic_artifacts │ artifacts_cache                     │   │
│  └──────────────────────────────────────────────────────────┘   │
│                                                                  │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │                    AI Tables                               │   │
│  │  ai_sessions │ ai_messages │ ai_tool_calls │ ai_audit_log │   │
│  │  ai_context_snapshots │ ai_search_index                    │   │
│  │  ai_entity_resolutions │ ai_investigation_plans            │   │
│  └──────────────────────────────────────────────────────────┘   │
│                                                                  │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │                    Custody Tables                          │   │
│  │  custody_events │ chain_of_custody │ audit_log            │   │
│  └──────────────────────────────────────────────────────────┘   │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

## Key Design Decisions

### 1. Local-First Architecture
- All data stored locally in SQLite
- No cloud dependency for core functionality
- Optional AI features can use local or remote providers

### 2. Forensic Integrity
- SHA-256/SHA-512 hashing for all evidence
- Chain of custody tracking
- Audit logging for all actions
- Read-only evidence access

### 3. AI as Assistant, Not Authority
- AI provides suggestions, not determinations
- All AI outputs require human validation
- Evidence citations for AI claims
- Privacy-first: local AI recommended

### 4. Modular Analysis
- Pluggable analysis engines
- Extensible artifact taxonomy
- Configurable risk scoring

### 5. FTS5 Full-Text Search
- **Porter Stemmer**: Search "transfer" matches "transfers", "transferring", "transferred"
- **Boolean Queries**: `fraud AND (wire OR offshore) NOT payroll`
- **Proximity Search**: `NEAR("wire" "transfer", 5)` finds words within 5 words
- **Wildcard Search**: `crypt*` matches "crypto", "cryptocurrency", "cryptography"
- **Exact Phrase**: `"strictly confidential"` matches exact phrase
- **Hit Highlighting**: Returns snippets with `<mark>` tags around matches
- **BM25 Ranking**: Results ordered by relevance score
- **Performance**: < 5ms on 100,000+ emails (vs 30+ seconds with LIKE)

### 6. Document Processing Pipeline
- **Attachment Text Extraction**: PDF, DOCX, XLSX, PPTX, CSV, RTF, TXT
- **OCR Engine**: macOS Vision framework (native) + Tesseract (fallback)
- **Extracted Text Storage**: Stored in `attachments.extracted_text` column
- **FTS Integration**: Extracted text indexed in FTS5 for searchability

### 7. OAuth2 Authentication
- **Google Workspace**: Device Code Flow (RFC 8628) + SASL XOAUTH2 (RFC 7628)
- **Microsoft 365**: Azure AD token exchange
- **Token Management**: Automatic refresh, secure storage
- **Fallback**: Traditional password auth still supported

## Data Flow Diagrams

### Evidence Ingestion Flow
```
┌─────────────────────────────────────────────────────────────────┐
│                    Evidence Ingestion                            │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌─────────┐    ┌─────────┐    ┌─────────┐    ┌─────────┐      │
│  │  File   │    │  Parse  │    │  Store  │    │ Extract │      │
│  │  Upload │───►│  Email  │───►│  in DB  │───│ Entities│      │
│  └─────────┘    └─────────┘    └─────────┘    └─────────┘      │
│       │              │              │              │             │
│       ▼              ▼              ▼              ▼             │
│  ┌─────────┐    ┌─────────┐    ┌─────────┐    ┌─────────┐      │
│  │ Compute │    │ RawEmail│    │ emails  │    │entities │      │
│  │  Hash   │    │ Struct  │    │  table  │    │  table  │      │
│  └─────────┘    └─────────┘    └─────────┘    └─────────┘      │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

### Analysis Flow
```
┌─────────────────────────────────────────────────────────────────┐
│                    Analysis Pipeline                             │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌─────────┐    ┌─────────┐    ┌─────────┐    ┌─────────┐      │
│  │ Header  │    │  Auth   │    │Spoofing │    │  Risk   │      │
│  │ Analysis│───►│ Verify  │───►│ Detect  │───►│  Score  │      │
│  └─────────┘    └─────────┘    └─────────┘    └─────────┘      │
│       │              │              │              │             │
│       ▼              ▼              ▼              ▼             │
│  ┌─────────┐    ┌─────────┐    ┌─────────┐    ┌─────────┐      │
│  │  Hop    │    │SPF/DKIM │    │ Findings│    │ 0-100   │      │
│  │ Chain   │    │ DMARC   │    │  List   │    │  Score  │      │
│  └─────────┘    └─────────┘    └─────────┘    └─────────┘      │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

### AI Investigation Flow
```
┌─────────────────────────────────────────────────────────────────┐
│                    AI Investigation                              │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌─────────┐    ┌─────────┐    ┌─────────┐    ┌─────────┐      │
│  │  User   │    │Context  │    │   AI    │    │ Output  │      │
│  │ Query   │───►│ Gather  │───►│ Process │───►│ Format  │      │
│  └─────────┘    └─────────┘    └─────────┘    └─────────┘      │
│       │              │              │              │             │
│       ▼              ▼              ▼              ▼             │
│  ┌─────────┐    ┌─────────┐    ┌─────────┐    ┌─────────┐      │
│  │Natural  │    │Emails   │    │ Ollama  │    │ Cited   │      │
│  │Language │    │Entities │    │ OpenAI  │    │ Response│      │
│  │         │    │Findings │    │Anthropic│    │         │      │
│  └─────────┘    └─────────┘    └─────────┘    └─────────┘      │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

## Risk Score Calculation

### Formula
```
Risk Score = (
  header_anomalies * 0.2 +
  auth_failures * 0.3 +
  spoofing_findings * 0.25 +
  attachment_risks * 0.15 +
  content_threats * 0.1
) * 100
```

### Severity Levels
| Score | Severity | Color |
|-------|----------|-------|
| 0-25 | Low | 🟢 Green |
| 26-50 | Medium | 🟡 Yellow |
| 51-75 | High | 🟠 Orange |
| 76-100 | Critical | 🔴 Red |

## Search Operators

| Operator | Example | Description |
|----------|---------|-------------|
| `from:` | `from:john@example.com` | Sender contains |
| `to:` | `to:finance@company.com` | Recipient contains |
| `subject:` | `subject:invoice` | Subject contains |
| `after:` | `after:2024-01-01` | Sent after date |
| `before:` | `before:2024-06-01` | Sent before date |
| `has:attachment` | `has:attachment` | Has attachments |
| `risk:>50` | `risk:>50` | Risk score above 50 |
| `folder:` | `folder:sent` | In specific folder |
| `type:` | `type:fraud` | Finding type |

## Security Considerations

### Data Protection
- All data stored locally
- No telemetry or tracking
- Optional AI with privacy controls
- Hash verification for evidence integrity

### Authentication
- Local user accounts
- Session-based authentication
- Role-based access (future)

### Forensic Admissibility
- Chain of custody logging
- SHA-256/SHA-512 hashing
- Audit trail for all actions
- Read-only evidence handling

## Performance Optimization

### Database Indexes
- 38 indexes for fast queries
- Composite indexes for common filters
- Foreign key indexes

### Caching
- Artifacts cache table
- Dashboard statistics caching
- Frontend state management

### Streaming
- IMAP streaming for large mailboxes
- Pagination for email lists
- Lazy loading for attachments

## Future Architecture Plans

1. **Multi-user support** - Role-based access
2. **Distributed analysis** - Parallel processing
3. **Plugin system** - Custom analysis engines
4. **Cloud sync** - Optional encrypted backup
5. **Mobile companion** - Evidence capture app

---

*Last updated: 2026-08-26*
