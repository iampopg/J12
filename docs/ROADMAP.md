# Roadmap

This document outlines the planned features and improvements for J12 Forensic.

## Current Status: v1.1.0

Core forensic investigation capabilities with AI integration.

---

## Short Term (v1.1 - v1.3)

### v1.1 - Stability & Polish ✅ COMPLETE
- [x] Fix attachment viewing (click-to-open)
- [x] Fix HTML content rendering
- [x] Fix crypto seed phrase regex false positives (BIP-39 validation)
- [x] Add deduplication to artifact pages
- [x] Improve first-visit performance for artifacts page (caching)
- [x] Add loading indicators for long operations
- [x] Fix schema mismatch (is_inline column)
- [x] Fix IMAP path inconsistency
- [x] Save POP3 attachments to disk
- [x] Unify chain of custody tables
- [x] Register AI module and commands
- [x] Create AI database tables

### v1.2 - Enhanced Search & Analysis 🔄 IN PROGRESS
- [ ] **SQLite FTS5 Full-Text Search Engine**
  - [ ] Create emails_fts virtual table with porter unicode61 tokenizer
  - [ ] Auto-sync triggers (INSERT, UPDATE, DELETE)
  - [ ] Boolean search (AND, OR, NOT)
  - [ ] Proximity search (NEAR/5)
  - [ ] Stemming & wildcard support
  - [ ] Hit highlighting snippets
  - [ ] Sub-millisecond response times
- [ ] **Deep Attachment Text Extraction**
  - [ ] PDF text extraction
  - [ ] Word (.docx) parsing
  - [ ] Excel (.xlsx) parsing
  - [ ] PowerPoint (.pptx) parsing
  - [ ] CSV/RTF text extraction
- [ ] **Image OCR Engine**
  - [ ] Native macOS Vision framework
  - [ ] Linux/Windows fallback
  - [ ] Image-only PDF OCR

### v1.3 - Modern Authentication 📋 PLANNED
- [ ] **OAuth2 IMAP Authentication**
  - [ ] Google Workspace SASL XOAUTH2
  - [ ] Microsoft 365 / Azure AD
  - [ ] Device Code Authorization (RFC 8628)
  - [ ] Token refresh handling
- [ ] Multi-user authentication
- [ ] Role-based access control (Admin, Investigator, Viewer)

---

## Medium Term (v2.0 - v2.5)

### v2.0 - Database Encryption
- [ ] AES-256 encryption for database at rest
- [ ] Encrypted attachments storage
- [ ] Secure key management
- [ ] Password-based unlock

### v2.1 - Advanced AI
- [ ] Custom AI model fine-tuning
- [ ] Investigation playbooks
- [ ] Automated report generation
- [ ] Multi-modal analysis (images, PDFs)
- [ ] Voice-to-text for notes

### v2.2 - Collaboration Features
- [ ] Case sharing between users
- [ ] Activity feed per case
- [ ] Investigator notes with @mentions
- [ ] Real-time collaboration

### v2.3 - Plugin System
- [ ] Plugin API for custom analysis engines
- [ ] Community plugin marketplace
- [ ] Custom artifact extractors
- [ ] Custom report templates

### v2.4 - Internationalization
- [ ] Multi-language support
- [ ] Locale-specific artifact patterns
- [ ] Right-to-left text support
- [ ] Regional date/time formats

---

## Long Term (v3.0+)

### v3.0 - Enterprise Features
- [ ] LDAP/Active Directory integration
- [ ] SSO support
- [ ] Centralized deployment
- [ ] Admin dashboard
- [ ] Usage analytics

### v3.1 - AI Investigation Agent
- [ ] Autonomous investigation workflows
- [ ] Proactive anomaly detection
- [ ] Predictive analytics
- [ ] Natural language case queries

### v3.2 - Forensic Lab Integration
- [ ] Laboratory Information Management System (LIMS) integration
- [ ] Evidence chain automation
- [ ] Court submission workflows
- [ ] Digital signature support

---

## Completed Features

### v1.0 (Initial Release)
- ✅ Evidence acquisition (File, IMAP, POP3)
- ✅ Email parsing (EML, MBOX)
- ✅ Forensic analysis (headers, auth, spoofing)
- ✅ Investigation tools (search, timeline, graph)
- ✅ Artifact extraction (12 domains, 80+ signatures)
- ✅ AI integration (11 engines, multiple providers)
- ✅ Court-ready reporting (HTML export)
- ✅ Case management
- ✅ Chain of custody tracking
- ✅ Audit logging
- ✅ Bookmark system
- ✅ Notes and tags

### v1.1 (Current)
- ✅ Attachment disk extraction & inline tracking
- ✅ BIP-39 seed phrase validation
- ✅ AI subsystem integration (25 commands)
- ✅ Unified chain of custody
- ✅ Database schema fixes
- ✅ IMAP/POP3 attachment storage

---

## Architecture

### Tech Stack
- **Backend:** Rust + Tauri 2
- **Frontend:** React + TypeScript + Vite
- **Database:** SQLite with FTS5
- **AI:** Multiple providers (OpenAI, Anthropic, Ollama, etc.)

### Project Structure
```
J12-forensic/
├── src/                    # React frontend
│   ├── pages/              # Page components
│   ├── views/              # View components
│   ├── components/         # Reusable components
│   ├── context/            # React context
│   └── utils/              # Utilities
├── src-tauri/              # Rust backend
│   ├── src/
│   │   ├── commands/       # Tauri commands
│   │   ├── analysis/       # Analysis engines
│   │   ├── ai/             # AI integration
│   │   └── imap_acquisition/ # IMAP client
│   └── Cargo.toml
└── docs/                   # Documentation
```

---

## How to Contribute

See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines on how to contribute to any of these features.

---

## Feature Requests

Have an idea? Open an issue with the `enhancement` label and describe:
1. What problem does it solve?
2. Who would benefit?
3. Any implementation ideas?

---

*Last updated: 2026-08-28*

