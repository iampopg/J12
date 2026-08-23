# J12

> **Email Forensic Investigation Platform**

![Status](https://img.shields.io/badge/status-under%20development-yellow)
![License](https://img.shields.io/badge/license-MIT-green)
![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux-blue)

---

## Overview

**J12** is a vendor-agnostic, court-admissible, multi-user desktop email forensic investigation platform. It ingests mailbox data (EML, MBOX, PST, OST, MSG, EMLX) and provides a timeline-first investigation workflow with communication graph analysis, fraud/anomaly detection, and court-ready reporting.

Born from the need for transparent, reproducible forensic tooling — **J12** traces every conclusion back to the raw evidence.

---

## Project Goal

Build a forensic-grade email investigation platform that:

- **Ingests** all major email formats (EML, MBOX, PST, OST, MSG, EMLX)
- **Analyzes** headers, authentication (SPF/DKIM/DMARC/ARC), and content for spoofing, fraud, and anomalies
- **Visualizes** communication patterns through timelines, entity graphs, and relationship maps
- **Produces** court-ready reports with full evidence provenance and chain of custody
- **Preserves** read-only evidence handling with SHA-256 verification at every transfer

Every displayed fact traces to: **Field → Raw bytes → Byte offset → Evidence file → SHA-256 → CoC record**

---

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│                  INVESTIGATOR UI                        │
│         React + TypeScript | Tauri Shell                │
├─────────────────────────────────────────────────────────┤
│                FORENSIC ENGINE                          │
│     Rust Core | Parser Registry | Analyzer Traits       │
├─────────────────────────────────────────────────────────┤
│               EVIDENCE STORAGE                          │
│   SQLite | Read-only Files | Content-addressable Store  │
└─────────────────────────────────────────────────────────┘
```

**Tech Stack:**
- **Frontend**: React 18, TypeScript, Vite
- **Backend**: Rust (Tauri 2.x)
- **Database**: SQLite (FTS5 full-text search)
- **Build**: Cross-platform (Windows, macOS, Linux)

---

## Current Status

| Phase | Description | Status |
|-------|-------------|--------|
| Phase 0 | Foundation (auth, cases, evidence, custody) | ✅ Complete |
| Phase 1 | Core Email Parsing (EML/MBOX/EMLX) | ✅ Complete |
| Phase 2 | Containers (PST/OST/MSG partial) | ⚠️ Partial |
| Phase 3 | Analysis Engine | ✅ Complete |
| Phase 4 | Investigation (search, entities, timeline, graph) | 🔜 Planned |
| Phase 5 | Reporting | 🔜 Planned |
| Phase 6 | Hardening | 🔜 Planned |

---

## Features Implemented

- **Evidence Ingestion**: Upload via file picker, SHA-256 hashing, chain of custody logging
- **Email Parsing**: RFC 5322 compliant EML/MBOX/EMLX parsers with X-Folder categorization
- **Header Analysis**: Received chain parsing, clock skew detection, routing anomaly detection
- **Authentication**: SPF/DKIM/DMARC/ARC verification from Authentication-Results headers
- **Spoofing Detection**: Display-name attacks, homoglyph domains, brand impersonation
- **Attachment Analysis**: Magic byte detection, entropy analysis, extension mismatch flagging
- **Findings Engine**: Automated finding generation with severity, confidence, and review workflow
- **Risk Scoring**: Per-email risk score (0-100) based on auth failures and anomalies

---

## Getting Started

### Prerequisites
- [Rust](https://www.rust-lang.org/tools/install) (latest stable)
- [Node.js](https://nodejs.org/) (v18+)
- [Tauri CLI](https://v2.tauri.app/start/prerequisites/)

### Development

```bash
# Clone the repository
git clone https://github.com/iampopg/J12.git
cd J12

# Install dependencies
npm install

# Run in development mode
npm run tauri dev

# Build for production
npm run tauri build
```

---

## Contributing

We are actively seeking contributors! Areas where help is especially welcome:

- **PST/OST/MSG parsing**: Integration with libpff or alternative parsers
- **UI/UX improvements**: Better data visualization, responsive design
- **Test corpus**: Building a comprehensive test suite with real-world samples
- **Documentation**: User guides, API documentation, forensic methodology
- **Performance**: Scaling to 500K+ email cases

See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines (coming soon).

To contribute:
1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

---

## License

This project is licensed under the MIT License — see [LICENSE](LICENSE) for details.

---

## Acknowledgments

The name **J12** is inspired by **Abiola June 12** — a reminder that behind every investigation are real people seeking truth and justice.

---

> **Disclaimer**: No software can make evidence automatically "court-admissible." J12 is designed to produce forensically defensible evidence and documentation that supports authentication/admissibility. Courts determine admissibility based on jurisdiction, facts, authentication, hearsay rules, expert methodology, and other factors.
