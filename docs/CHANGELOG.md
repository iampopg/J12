# Changelog

All notable changes to J12 Forensic will be documented in this file.

## [Unreleased]

### Added
- Complete database documentation (DATABASE_REFERENCE.md)
- Complete system audit (SYSTEM_AUDIT.md)
- Contributing guidelines (CONTRIBUTING.md)
- Architecture documentation (ARCHITECTURE.md)

## [1.0.0] - 2026-08-26

### Added
- **Evidence Acquisition**
  - File import (EML, MBOX, PST, OST, MSG, EMLX, TNEF)
  - IMAP live acquisition
  - POP3 live acquisition
  - Forensic imaging (UI placeholder)

- **Email Parsing**
  - RFC 2822 compliant EML parsing
  - MBOX folder parsing
  - PST/OST file parsing (via libpff)
  - MSG file parsing
  - Attachment extraction

- **Forensic Analysis**
  - Header analysis (Received chain, delays)
  - Authentication verification (SPF, DKIM, DMARC, ARC)
  - Spoofing detection
  - Attachment risk analysis
  - Risk score calculation (0-100)
  - Entity extraction and resolution

- **Investigation Tools**
  - Advanced search with operators
  - Entity profiles with communication analysis
  - Timeline visualization
  - Communication graph
  - Bookmark system
  - Notes and tags

- **Artifact Extraction**
  - Credentials and secrets (passwords, API keys, tokens)
  - Financial data (credit cards, bank accounts, routing numbers)
  - Cryptocurrency (BTC, ETH, SOL, TRON, LTC, DOGE, XMR)
  - PII (SSN, passport, driver's license)
  - Threats (weapons, narcotics, explosives, terrorism)
  - Malware IOCs (CVE, C2)
  - URLs and app signatures

- **AI Integration**
  - Natural language search
  - Evidence explanation
  - Investigation planning
  - Timeline analysis
  - Spoofing analysis
  - Attachment triage
  - Graph analysis
  - Entity resolution
  - Anomaly detection
  - Report generation
  - Support for Ollama, OpenAI, Anthropic, OpenRouter

- **Reporting**
  - PDF report generation
  - 12 report sections
  - Exhibit management
  - Chain of custody export
  - Audit log export

- **Case Management**
  - Create, update, delete cases
  - Target profile auto-detection
  - Evidence status tracking
  - Integrity verification

- **User Interface**
  - Login page
  - Case list and management
  - Case workspace with tabs
  - Email list with sorting/filtering
  - Email detail modal
  - Search view
  - Entity dive view
  - Timeline view
  - Graph view
  - Attachments view
  - Artifacts hub
  - Findings view
  - Notes view
  - Report view
  - AI setup page
  - AI chat widget
  - Evidence locker (bookmarks)

### Security
- SHA-256/SHA-512 evidence hashing
- Chain of custody tracking
- Audit logging
- Local data storage

### Database
- 25 tables with full relationships
- 38 indexes for performance
- WAL mode for concurrent access
- Migration system for schema updates

---

## [0.9.0] - 2026-08-XX

### Added
- Initial AI architecture (v3.1)
- AI database tables (8 tables)
- AI command handlers (25+ commands)
- AI setup UI
- AI chat widget

---

## [0.8.0] - 2026-08-XX

### Added
- Artifact extraction engine
- 12 artifact domains
- 80+ app signatures
- 40+ regex patterns
- False positive reduction validators

---

## [0.7.0] - 2026-08-XX

### Added
- POP3 acquisition
- IMAP acquisition improvements
- Streaming email fetch

---

## [0.6.0] - 2026-08-XX

### Added
- Performance optimization (25+ indexes)
- Folder category migration
- Entity extraction improvements

---

## [0.5.0] - 2026-08-XX

### Added
- Report generation
- Bookmarks system
- Notes and tags

---

## [0.4.0] - 2026-08-XX

### Added
- Timeline view
- Graph view
- Entity dive view
- Advanced search

---

## [0.3.0] - 2026-08-XX

### Added
- Header analysis
- Authentication verification
- Spoofing detection
- Risk scoring
- Findings generation

---

## [0.2.0] - 2026-08-XX

### Added
- Email parsing (EML, MBOX)
- Attachment extraction
- Basic search
- Case management

---

## [0.1.0] - 2026-08-XX

### Added
- Initial project setup
- Tauri + React + TypeScript
- SQLite database
- Login page
- Case list page
- Basic email list view

---

[Unreleased]: https://github.com/iampopg/J12/compare/v1.0.0...HEAD
[1.0.0]: https://github.com/iampopg/J12/compare/v0.9.0...v1.0.0
[0.9.0]: https://github.com/iampopg/J12/compare/v0.8.0...v0.9.0
[0.8.0]: https://github.com/iampopg/J12/compare/v0.7.0...v0.8.0
[0.7.0]: https://github.com/iampopg/J12/compare/v0.6.0...v0.7.0
[0.6.0]: https://github.com/iampopg/J12/compare/v0.5.0...v0.6.0
[0.5.0]: https://github.com/iampopg/J12/compare/v0.4.0...v0.5.0
[0.4.0]: https://github.com/iampopg/J12/compare/v0.3.0...v0.4.0
[0.3.0]: https://github.com/iampopg/J12/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/iampopg/J12/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/iampopg/J12/releases/tag/v0.1.0
