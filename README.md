# J12 Forensic Investigation Platform

**A vendor-agnostic, court-admissible, multi-user desktop email forensic investigation platform.**

![J12 Logo](j12-logo-v3.png)

## Overview

J12 is a desktop application for email forensic investigation. It ingests mailbox data (EML/MBOX/PST/OST/MSG) and provides timeline-first investigation workflow with communication graph, fraud/anomaly detection, and court-ready reporting.

## Features

- **Evidence Acquisition**: File upload, IMAP/POP3 live acquisition, forensic imaging
- **Email Parsing**: EML, MBOX, PST, OST, MSG formats
- **Forensic Analysis**: Header analysis, authentication verification (SPF/DKIM/DMARC), spoofing detection
- **Investigation Tools**: Advanced search, entity profiles, timeline, communication graph
- **AI Assistance**: Natural language search, evidence explanation, investigation planning (Coming Soon)
- **Court-Ready Reports**: PDF export with exhibits, hash manifest, chain of custody

## Quick Start

```bash
# Install dependencies
cd frontend && npm install

# Run development server
npx tauri dev

# Build for production
npx tauri build
```

## Documentation

- [AI Architecture](docs/AI_ARCHITECTURE.md) - AI investigation engine design
- [Installation](docs/INSTALLATION.md) - Setup and installation guide
- [User Guide](docs/USER_GUIDE.md) - How to use the application
- [API Reference](docs/API_REFERENCE.md) - Backend API documentation

## License

MIT License - See [LICENSE](LICENSE) for details.

## Credits

Inspired by Abiola June 12 — Branding is green J + white 12.
