<div align="center">

![J12 Logo](public/j12-logo.png)

# **J12**

### Email Forensic Investigation Platform

[![Status](https://img.shields.io/badge/status-under%20development-yellow?style=flat-square)]()
[![License](https://img.shields.io/badge/license-MIT-green?style=flat-square)]()
[![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux-blue?style=flat-square)]()
[![Tauri](https://img.shields.io/badge/Tauri-2.x-FFC131?style=flat-square&logo=tauri&logoColor=white)]()
[![Rust](https://img.shields.io/badge/Rust-1.70+-dea584?style=flat-square&logo=rust&logoColor=white)]()
[![React](https://img.shields.io/badge/React-18-61DAFB?style=flat-square&logo=react&logoColor=white)]()

**Forensic-grade email investigation with evidence provenance**

*Every conclusion traces back to the raw evidence.*

---

</div>

---

## 🎯 What is J12?

**J12** is a vendor-agnostic, court-admissible, multi-user desktop email forensic investigation platform. It ingests mailbox data from all major formats and provides investigators with timeline-first analysis tools, communication graph mapping, fraud/anomaly detection, and court-ready reporting.

The name **J12** is inspired by **Abiola June 12** — a reminder that behind every investigation are real people seeking truth and justice.

> **Important legal distinction:** No software can make evidence automatically "court-admissible." J12 is designed to produce **forensically defensible evidence and documentation that supports authentication/admissibility** following ISO 27037, SWGDE guidelines, and Daubert/FRE 702 standards.

---

## 🖥️ Platform Preview

### Main Dashboard
```
┌────────────────────────────────────────────────────────────────────────┐
│  [J12 Logo]  J12 · Fraud Investigation    ● 3,266 emails  1 source   │
├───────────┬────────────────────────────────────────────────────────────┤
│           │  Case Dashboard                                            │
│ ▶ Emails  │  ┌────────┐ ┌────────┐ ┌────────┐ ┌────────┐               │
│   Inbox  │  │ 3,266  │ │ 1,241  │ │  918   │ │   0    │  ← KPI Cards │
│   Sent   │  │ Emails │ │Entities│ │ Deleted│ │ Findings│               │
│   Deleted│  └────────┘ └────────┘ └────────┘ └────────┘               │
│   Drafts │                                                            │
│   Spam   │  ┌──────────────────────┐  ┌──────────────────────┐        │
│           │  │  Severity Breakdown  │  │  Top Correspondents  │        │
│ ▶Evidence│  │  Critical: 0         │  │  ████████████ 245    │        │
│  PST-001 │  │  High: 3             │  │  ████████   189      │        │
│           │  │  Medium: 12          │  │  ██████     134      │        │
│ ▶Invest. │  │  Low: 25             │  │  █████     98        │        │
│   Graph   │  └──────────────────────┘  └──────────────────────┘        │
│   Findings│                                                            │
│   Search  │  TIMELINE                                                 │
│   Timeline│  ═══════════════════════════════════════════════          │
│           │  ──●──●────●───●●●──────●───●──────►                     │
│ ▶Case    │                                                            │
│   Custody │  AUTHENTICATION SUMMARY                                  │
│   Notes   │  SPF: 89% pass │ DKIM: 76% pass │ DMARC: 82% pass       │
└───────────┴────────────────────────────────────────────────────────────┘
```

### Email Detail View with Forensic Analysis
```
┌────────────────────────────────────────────────────────────────────────┐
│  Subject: Urgent wire transfer needed                          [Close] │
│  From: john.smith@enron.com · Mon, 15 Jan 2001 09:15                  │
│  Source: enron_sample.mbox · Risk Score: 45/100 (MEDIUM)             │
├────────────────────────────────────────────────────────────────────────┤
│ [Overview] [Headers ▾] [Authentication ▾] [MIME] [Raw] [Attachments] │
├────────────────────────────────────────────────────────────────────────┤
│  AUTHENTICATION ANALYSIS                                               │
│  ┌─────────────────────────┐  ┌─────────────────────────┐             │
│  │ SPF:    [● PASS]        │  │ DKIM:   [● PASS]        │             │
│  │ Domain: enron.com       │  │ Selector: default       │             │
│  │ Aligned: Yes            │  │ Domain: enron.com       │             │
│  └─────────────────────────┘  └─────────────────────────┘             │
│  ┌─────────────────────────┐  ┌─────────────────────────┐             │
│  │ DMARC:  [● PASS]        │  │ ARC:    [○ N/A]         │             │
│  │ Policy: reject          │  │ No ARC seals present    │             │
│  │ Aligned: Yes            │  │                         │             │
│  └─────────────────────────┘  └─────────────────────────┘             │
│                                                                        │
│  RECEIVED CHAIN (3 hops, bottom = oldest)                             │
│  ┌ Hop 3: mail.enron.com ──► Hop 2: proxy.corp.com ──► Hop 1: inet  │
│  │  +0s                    │  +2.3s                   │  +0.1s        │
│  │  by: mail.enron.com     │  by: proxy.corp.com      │              │
│  │  with: ESMTP            │  with: HTTP              │              │
│  └────────────────────────────────────────────────────────────────────┘
│                                                                        │
│  ROUTING ANOMALIES: None detected                                     │
│  CLOCK SKEW: None detected                                            │
└────────────────────────────────────────────────────────────────────────┘
```

### Findings & Spoofing Detection
```
┌────────────────────────────────────────────────────────────────────────┐
│  FORENSIC FINDINGS                                        [▶ Run All] │
├────────────────────────────────────────────────────────────────────────┤
│  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────┐                   │
│  │ CRITICAL│  │  HIGH   │  │ MEDIUM  │  │   LOW   │                   │
│  │    0    │  │    3    │  │   12    │  │   25    │                   │
│  └─────────┘  └─────────┘  └─────────┘  └─────────┘                   │
│                                                                        │
│  SEVERITY    TYPE     TITLE                        STATUS    ACTIONS   │
│  ─────────────────────────────────────────────────────────────────────  │
│  [HIGH]    SPOOFING  Return-Path mismatch         [OPEN]    [✓] [✗]   │
│  [MEDIUM]  ANOMALY   Clock skew: 450s detected    [OPEN]    [✓] [✗]   │
│  [LOW]     ROUTING   Excessive hops (12)          [OPEN]    [✓] [✗]   │
│  [HIGH]    BEC       Brand impersonation (PayPal)  [OPEN]    [✓] [✗]   │
└────────────────────────────────────────────────────────────────────────┘
```

---

## 🏗️ Architecture

### Three-Layer Design

```
┌─────────────────────────────────────────────────────────────────────────┐
│                        LAYER 3: INVESTIGATOR UI                         │
│                                                                         │
│   React 18 + TypeScript │ Tauri IPC │ Tailwind-style CSS               │
│                                                                         │
│   Dashboard │ Evidence │ Emails │ Headers │ Auth │ Findings │ Timeline  │
│   Graph │ Entities │ Search │ Reports │ Custody                        │
├─────────────────────────────────────────────────────────────────────────┤
│                         LAYER 2: FORENSIC ENGINE                        │
│                                                                         │
│   Rust Core with Modular Crates (Trait-based Plugin System)            │
│                                                                         │
│   ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐ │
│   │ format-eml  │  │ format-msg  │  │ format-pst  │  │ search-eng  │ │
│   │ EML/MBOX    │  │ CFB/OLE     │  │ libpff FFI  │  │ FTS5/Tantivy│ │
│   └─────────────┘  └─────────────┘  └─────────────┘  └─────────────┘ │
│   ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐ │
│   │ analysis-hdr│  │ analysis-auth│ │ analysis-spf│  │ analysis-att│ │
│   │ Received    │  │ SPF/DKIM/   │  │ Displayname │  │ Magic bytes │ │
│   │ chain, skew │  │ DMARC/ARC   │  │ homoglyphs  │  │ entropy     │ │
│   └─────────────┘  └─────────────┘  └─────────────┘  └─────────────┘ │
│   ┌─────────────┐  ┌─────────────┐  ┌─────────────┐                   │
│   │ analysis-tml│  │ analysis-grph│ │ custody-eng │                   │
│   │ Event recon │  │ Entity/rel  │  │ Hash, audit │                   │
│   └─────────────┘  └─────────────┘  └─────────────┘                   │
├─────────────────────────────────────────────────────────────────────────┤
│                        LAYER 1: EVIDENCE STORAGE                        │
│                                                                         │
│   ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────────────┐ │
│   │  SQLite (FTS5)  │  │  Read-only FS   │  │  Content-addressable    │ │
│   │  metadata,      │  │  Original files │  │  Attachment store       │ │
│   │  relations,     │  │  chmod 444      │  │  ab/abcd1234...         │ │
│   │  audit, custody │  │  Never modified │  │  SHA-256 naming         │ │
│   └─────────────────┘  └─────────────────┘  └─────────────────────────┘ │
│                                                                         │
│   Every field traces: Display → Raw bytes → Offset → File → SHA-256   │
└─────────────────────────────────────────────────────────────────────────┘
```

### Evidence Provenance Chain

```
Display Field  →  Raw Bytes  →  Byte Offset  →  Evidence File  →  SHA-256
     ↓                                                                    
Chain of Custody Record (append-only)                                   
                                                                        
Every conclusion is reproducible from the source evidence.              
```

---

## 🔬 Forensic Methodology

### Header Analysis
| Technique | What We Detect |
|-----------|----------------|
| **Received Chain Parsing** | Full hop-by-hop reconstruction, bottom-up (oldest first) |
| **Clock Skew Detection** | Negative transit times, timestamp reversals, >5min jumps |
| **Originating IP Extraction** | Source IP from first Received hop or X-Originating-IP |
| **Routing Anomalies** | Excessive hops (>10), missing hops, suspicious relays |

### Authentication Verification (RFC 7208, 6376, 7489, 8617)
| Protocol | Analysis |
|----------|----------|
| **SPF** | Parse result from Authentication-Results, extract domain/alignment |
| **DKIM** | Signature presence, domain/selector extraction, pass/fail status |
| **DMARC** | Policy evaluation, alignment check (strict vs relaxed) |
| **ARC** | Chain of ARC-Seal headers, instance validation |

### Spoofing Detection
| Attack Vector | Detection Method |
|---------------|------------------|
| **Display Name Spoofing** | Display name contains email different from actual sender |
| **Brand Impersonation** | "PayPal", "Apple", "Bank" in display name but different domain |
| **From/Return-Path Mismatch** | Different domains between From: and Return-Path: headers |
| **Reply-To Pivot** | Reply-To domain differs from From domain |
| **Homoglyph Domains** | Mixed script detection (Latin + Cyrillic/Greek), punycode (xn--) |
| **Message-ID Anomaly** | Message-ID domain differs from From domain |

### Attachment Analysis
| Check | Purpose |
|-------|---------|
| **Magic Byte Detection** | Identify actual file type regardless of extension |
| **Extension Mismatch** | Detect .pdf files that are actually .exe |
| **Entropy Analysis** | Shannon entropy calculation (high = possibly encrypted/packed) |
| **Dangerous Extensions** | Flag .exe, .scr, .js, .vbs, .ps1, etc. |
| **Double Extensions** | Detect .pdf.exe, .doc.js patterns |
| **Office Macros** | Detect VBA project streams in OLE2 documents |

### Risk Scoring (0-100)
| Factor | Weight |
|--------|--------|
| SPF Failure | +15 |
| DKIM Failure | +15 |
| DMARC Failure | +20 |
| Spoofing Finding (critical) | +25 |
| Spoofing Finding (high) | +15 |
| Attachment Risk | up to +25 |
| Routing Anomalies | +5 each |

---

## 📋 Supported Formats

| Format | Extension | Parser | Deleted Recovery | Priority |
|--------|-----------|--------|------------------|----------|
| **EML** | `.eml` | RFC 5322 compliant | No | P0 ✅ |
| **MBOX** | `.mbox` | All variants (mboxo/rd/cl/cl2) | Partial | P0 ✅ |
| **EMLX** | `.emlx` | plist + RFC 822 | No | P0 ✅ |
| **MSG** | `.msg` | CFB/OLE parser | No | P1 🔜 |
| **PST** | `.pst` | libpff FFI | Yes (tombstones) | P1 🔜 |
| **OST** | `.ost` | libpff FFI | Yes (tombstones) | P1 🔜 |
| **Winmail.dat** | `.dat` | TNEF parser | No | P2 🔜 |
| **Exchange EDB** | `.edb` | Custom adapter | Yes | P3 🔜 |
| **NSF** | `.nsf` | Custom adapter | Yes | P3 🔜 |

---

## 🎯 Project Goals

### Primary Objectives
1. **Court-Admissible Evidence** — Read-only ingestion, SHA-256 at every transfer, append-only audit log, full chain of custody per ISO 27037
2. **Every Field Traceable** — Click any displayed fact → see raw evidence → byte offset → evidence file → SHA-256 → CoC record
3. **Timeline-First Investigation** — Events organized chronologically with zoom from year to hour
4. **Relationship Mapping** — Communication graph showing entities, message flows, hub detection
5. **Fraud Detection** — Automated spoofing detection, brand impersonation, BEC indicators
6. **Deleted Item Recovery** — Recover soft-deleted, tombstoned, and orphaned emails from PST/OST
7. **Court-Ready Reporting** — PDF reports with methodology, evidence maps, exhibit numbering, hash manifest

### Non-Goals
- Web-based interface (desktop-first for forensic integrity)
- Real-time email acquisition (batch processing for defensibility)
- AI/ML classification (rules-based for reproducibility)
- Mobile acquisition (separate tool category)

---

## 🛠️ Technology Stack

| Layer | Technology | Purpose |
|-------|------------|---------|
| **Desktop Shell** | Tauri 2.x | Low RAM, small bundle, native WebView, FFI support |
| **Frontend** | React 18 + TypeScript | UI framework, type safety |
| **Backend** | Rust | Performance, memory safety, forensic library FFI |
| **Database** | SQLite (FTS5) | Metadata, relations, audit, full-text search |
| **Search Index** | Tantivy (planned) | Fuzzy, regex, large-scale search |
| **Build Targets** | Windows, macOS, Linux | Cross-platform from single codebase |

### Why Tauri over Electron?
- **RAM**: Tauri uses ~100MB vs Electron's ~500MB for large cases
- **Bundle Size**: ~10MB vs ~150MB
- **Security**: Memory-safe Rust backend, no Node.js runtime exposure
- **FFI**: Direct C library integration (libpff for PST parsing)

---

## 🚀 Getting Started

### Prerequisites
- [Rust](https://www.ru.st-lang.org/tools/install) (1.70+)
- [Node.js](https://nodejs.org/) (v18+)
- [Tauri CLI](https://v2.tauri.app/start/prerequisites/)

### Development
```bash
# Clone the repository
git clone https://github.com/iampopg/J12.git
cd J12

# Install dependencies
npm install

# Run in development mode (hot reload)
npm run tauri dev

# Build for production (creates signed binaries)
npm run tauri build
```

### Test Credentials (Development)
```
Username: admin
Password: admin123
```

### Sample Data
Place EML/MBOX files in the `data/samples/` directory. The Enron corpus (3,266 emails) is recommended for testing.

---

## 🗺️ Roadmap

| Phase | Description | Status |
|-------|-------------|--------|
| **Phase 0** | Foundation (auth, cases, evidence, custody) | ✅ Complete |
| **Phase 1** | Core Email Parsing (EML/MBOX/EMLX) | ✅ Complete |
| **Phase 2** | Containers (PST/OST/MSG — needs libpff) | ⚠️ Partial |
| **Phase 3** | Analysis Engine (headers, auth, spoofing, attachments) | ✅ Complete |
| **Phase 4** | Investigation (search, entities, timeline, graph) | 🔜 Planned |
| **Phase 5** | Reporting (PDF, exhibits, hash manifest) | 🔜 Planned |
| **Phase 6** | Hardening (performance, large-scale testing, code signing) | 🔜 Planned |

---

## 🤝 Contributing

We are actively seeking contributors who care about digital forensics, evidence integrity, and open-source tooling.

### Areas We Need Help

| Area | Skill Level | Description |
|------|-------------|-------------|
| **PST/OST Parsing** | Advanced Rust | Integrate libpff FFI or build Rust PST parser |
| **UI/UX Design** | Intermediate | Better data visualization, responsive design, dark theme polish |
| **Forensic Test Corpus** | All levels | Build test suite with real-world samples (sanitized) |
| **Timeline View** | React + Canvas | Implement canvas-based timeline with pan/zoom |
| **Communication Graph** | React + WebGL | Force-directed graph for entity relationships |
| **PDF Reporting** | Rust/TypeScript | Court-ready report generation with exhibits |
| **Documentation** | Technical Writing | User guides, API docs, forensic methodology |
| **YARA Integration** | Intermediate | Malware signature scanning for attachments |
| **Code Signing** | DevOps | Windows EV, macOS notarization, Linux signing |

### How to Contribute

1. **Fork** the repository
2. Create a feature branch: `git checkout -b feature/amazing-feature`
3. Commit your changes: `git commit -m 'Add amazing feature'`
4. Push to the branch: `git push origin feature/amazing-feature`
5. Open a **Pull Request**

### Code Standards
- Rust: Follow `rustfmt` + `clippy` conventions
- TypeScript: Strict mode, no `any` types, full type annotations
- Commits: Conventional commits (`feat:`, `fix:`, `docs:`)
- Tests: All analysis functions must have unit tests

---

## 📊 Quality Gates

| ID | Gate | Criteria |
|----|------|----------|
| G1 | Parse Reliability | 100% of Enron corpus parses (3,266 emails, 0 errors) |
| G2 | Search Speed | <100ms for 100K emails |
| G3 | Timeline Performance | 10K events at 60fps |
| G4 | Graph Performance | 500 nodes interactive |
| G5 | Memory Usage | <2GB for 500K emails |
| G6 | Integrity | SHA-256 verify all evidence |
| G7 | Court Admissibility | Report passes Daubert checklist |
| G8 | Recovery | Deleted PST items recoverable |

---

## 📁 Project Structure

```
J12/
├── README.md                   # This file
├── index.html                  # Tauri entry point
├── package.json                # Frontend dependencies
├── public/
│   ├── j12-logo.png            # Brand logo
│   └── favicon.svg             # Browser favicon
├── src/                        # Frontend (React + TypeScript)
│   ├── App.tsx                 # Root component
│   ├── auth.tsx                # Authentication context
│   ├── main.tsx                # Entry point
│   ├── styles.css              # Global dark theme
│   ├── pages/
│   │   ├── LoginPage.tsx       # Authentication screen
│   │   ├── CaseListPage.tsx    # Case selection
│   │   └── CaseWorkspace.tsx   # Main investigation workspace
│   └── views/
│       ├── EmailListView.tsx   # Email list + 6-tab detail
│       ├── FindingsView.tsx    # Findings management
│       └── ...
├── src-tauri/                  # Backend (Rust)
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs             # Tauri entry point
│       ├── commands.rs         # Tauri IPC commands
│       ├── db.rs               # SQLite schema + migrations
│       ├── models.rs           # Data structures
│       ├── parser.rs           # EML/MBOX/EMLX parsers
│       ├── pst.rs              # PST/OST/MSG interfaces
│       └── analysis.rs         # Forensic analysis engine
└── data/
    └── samples/                # Test corpus
```

---

## 📜 License

This project is licensed under the MIT License — see [LICENSE](LICENSE) for details.

---

## 🙏 Acknowledgments

The name **J12** is inspired by **Abiola June 12** — a reminder that behind every investigation are real people seeking truth and justice.

Built with:
- [Tauri](https://tauri.app/) — Desktop framework
- [Rust](https://www.rust-lang.org/) — Systems programming
- [React](https://react.dev/) — UI framework
- [SQLite](https://sqlite.org/) — Embedded database

---

<div align="center">

**J12** — Forensic-grade email investigation

![J12](public/j12-logo.png)

*Under active development — seeking contributors*

</div>
