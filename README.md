# J12 Forensic Investigation Platform

**A vendor-agnostic, court-admissible, open-source desktop email forensic investigation platform.**

![J12 Logo](j12-logo-v3.png)

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
[![Tauri](https://img.shields.io/badge/Tauri-2.0-blue.svg)](https://tauri.app)
[![Rust](https://img.shields.io/badge/Rust-1.70+-orange.svg)](https://www.rust-lang.org)
[![React](https://img.shields.io/badge/React-18-blue.svg)](https://react.dev)

## Overview

J12 is a desktop application for email forensic investigation. It ingests mailbox data (EML/MBOX/PST/OST/MSG) and provides timeline-first investigation workflow with communication graph, fraud/anomaly detection, and court-ready reporting.

**Key Principles:**
- **Local-first:** All data stored locally, no cloud dependency
- **Forensic integrity:** SHA-256 hashing, chain of custody, audit logging
- **AI as assistant:** AI provides suggestions, not determinations
- **Open source:** MIT licensed, community-driven

## Features

### Evidence Acquisition
- **File Import:** EML, MBOX, PST, OST, MSG, EMLX, TNEF formats
- **IMAP:** Live mailbox acquisition with streaming
- **POP3:** Legacy mailbox support
- **Forensic Imaging:** Disk imaging support (UI placeholder)

### Forensic Analysis
- **Header Analysis:** Received chain, timing anomalies, hop analysis
- **Authentication:** SPF, DKIM, DMARC, ARC verification
- **Spoofing Detection:** Display name spoofing, domain impersonation
- **Risk Scoring:** 0-100 score based on multiple factors
- **Entity Extraction:** Automatic people/organization identification

### Investigation Tools
- **Advanced Search:** 14+ search operators, filters
- **Entity Profiles:** Communication patterns, relationships
- **Timeline:** Chronological event visualization
- **Communication Graph:** Network relationship mapping
- **Artifacts Hub:** 12 domains, 80+ app signatures

### AI Assistance
- **Natural Language Search:** Ask questions in plain English
- **Evidence Explanation:** AI explains forensic findings
- **Investigation Planning:** Step-by-step investigation plans
- **Timeline Analysis:** Pattern and anomaly detection
- **Spoofing Analysis:** Deep email authentication analysis
- **Attachment Triage:** Risk assessment for attachments
- **Graph Analysis:** Communication pattern insights
- **Entity Resolution:** Identify duplicate entities
- **Anomaly Detection:** Unusual pattern identification
- **Report Generation:** Automated report drafting

### Court-Ready Reporting
- **PDF Export:** Professional PDF reports
- **12 Report Sections:** Executive summary, findings, timeline, etc.
- **Exhibits:** Attach specific emails as evidence
- **Chain of Custody:** Complete handling history
- **Audit Log:** All actions logged with timestamps

## Quick Start

### Prerequisites

- **Node.js** 20+ and npm 10+
- **Rust** 1.70+ (install via [rustup](https://rustup.rs/))
- **Tauri CLI** (`cargo install tauri-cli`)

See [Installation Guide](docs/INSTALLATION.md) for platform-specific requirements.

### Development Setup

```bash
# Clone the repository
git clone https://github.com/iampopg/J12.git
cd J12

# Install frontend dependencies
cd frontend && npm install && cd ..

# Run in development mode
cd frontend && cargo tauri dev
```

### Default Credentials

- **Username:** `admin`
- **Password:** `admin123`

**⚠️ Change default credentials in production!**

### Production Build

```bash
cd frontend && cargo tauri build
```

## Documentation

| Document | Description |
|----------|-------------|
| [README](README.md) | Project overview and quick start |
| [Installation](docs/INSTALLATION.md) | Detailed setup guide |
| [User Guide](docs/USER_GUIDE.md) | How to use the application |
| [Contributing](docs/CONTRIBUTING.md) | How to contribute |
| [Architecture](docs/ARCHITECTURE.md) | System architecture |
| [API Reference](docs/API_REFERENCE.md) | Backend API documentation |
| [Database Reference](docs/DATABASE_REFERENCE.md) | Complete database schema |
| [System Audit](docs/SYSTEM_AUDIT.md) | Complete system audit |
| [AI Architecture](docs/AI_ARCHITECTURE.md) | AI investigation engine design |
| [Security](docs/SECURITY.md) | Security policy |
| [Changelog](docs/CHANGELOG.md) | Version history |
| [Roadmap](docs/ROADMAP.md) | Planned features |

## Project Structure

```
J12/
├── frontend/              # React frontend
│   ├── src/
│   │   ├── components/    # Reusable UI components
│   │   ├── views/         # Page-level views
│   │   ├── pages/         # Top-level pages
│   │   └── utils/         # Utility functions
│   └── package.json
├── src-tauri/             # Rust backend
│   ├── src/
│   │   ├── commands/      # Tauri command handlers
│   │   ├── ai.rs          # AI integration
│   │   ├── analysis.rs    # Analysis engines
│   │   ├── db.rs          # Database layer
│   │   ├── models.rs      # Data structures
│   │   └── main.rs        # Application entry
│   └── Cargo.toml
├── docs/                  # Documentation
└── LICENSE                # MIT License
```

## Architecture

```
┌─────────────────────────────────────────────┐
│           Frontend (React + TypeScript)      │
│  Components → Views → Pages → Utils         │
└─────────────────────────────────────────────┘
                      │
              Tauri IPC Bridge
                      │
┌─────────────────────────────────────────────┐
│              Backend (Rust)                  │
│  Commands → Analysis → AI → Parser          │
└─────────────────────────────────────────────┘
                      │
┌─────────────────────────────────────────────┐
│            Database (SQLite)                 │
│  25 tables, 38 indexes, WAL mode            │
└─────────────────────────────────────────────┘
```

See [Architecture](docs/ARCHITECTURE.md) for detailed information.

## Database

- **25 tables** with full relationships
- **38 indexes** for performance optimization
- **WAL mode** for concurrent read/write
- **Migration system** for schema updates

See [Database Reference](docs/DATABASE_REFERENCE.md) for complete schema.

## AI Integration

J12 supports multiple AI providers:

| Provider | Type | Privacy |
|----------|------|---------|
| **Ollama** | Local | ✅ Fully private |
| **OpenAI** | Cloud | ⚠️ Data shared |
| **Anthropic** | Cloud | ⚠️ Data shared |
| **OpenRouter** | Cloud | ⚠️ Data shared |
| **kilo.ai** | Cloud | ⚠️ Data shared |

**Recommendation:** Use Ollama for sensitive investigations.

## Security

- All data stored locally
- No telemetry or tracking
- SHA-256/SHA-512 evidence hashing
- Chain of custody tracking
- Audit logging for all actions

See [Security Policy](docs/SECURITY.md) for more information.

## Contributing

We welcome contributions! See [Contributing Guide](docs/CONTRIBUTING.md) for:

- Development setup
- Coding standards
- Pull request process
- Code review guidelines

## Roadmap

See [Roadmap](docs/ROADMAP.md) for planned features:

- **v1.1:** Stability & polish
- **v1.2:** Enhanced analysis
- **v1.3:** Collaboration features
- **v2.0:** Database encryption
- **v2.1:** Advanced AI
- **v3.0:** Enterprise features

## License

This project is licensed under the MIT License - see [LICENSE](LICENSE) for details.

## Credits

Inspired by Abiola June 12 — Branding is green J + white 12.

## Support

- **Documentation:** See `docs/` directory
- **Issues:** Open a GitHub issue
- **Discussions:** Use GitHub discussions

---

*Built with Tauri, React, and Rust*
