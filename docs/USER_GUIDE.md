# User Guide

## Getting Started

### 1. Create a Case

1. Click **"+ New Case"** on the case list page
2. Enter case details:
   - Title
   - Case number (auto-generated)
   - Description
   - Target information (name, email, organization)
3. Click **"Create Case"**

### 2. Acquire Evidence

1. Open your case
2. Click **"Evidence Acquisition"** in the sidebar
3. Select acquisition method:
   - **File Import**: Upload EML, MBOX, PST, MSG files
   - **Mail Server**: Connect via IMAP/POP3 (Coming Soon)
   - **Forensic Imaging**: Disk imaging (Coming Soon)
4. Click **"Select Files"** and choose your evidence
5. Click **"Parse"** to extract emails

### 3. Investigate

#### Email Messages
- View all emails with sorting and filtering
- Click any email to view details:
  - Overview
  - Headers
  - Authentication (SPF/DKIM/DMARC)
  - MIME structure
  - Attachments

#### Forensic Intelligence
- **Artifacts Hub**: Extracted credentials, URLs, PII, crypto addresses
- **Attachments & Files**: All attachments with risk analysis
- **Security Findings**: Automated threat detection

#### Investigation Tools
- **Advanced Search**: 14+ search operators
- **Entity Profiles**: People and organizations
- **Timeline**: Chronological event view
- **Graph**: Communication network visualization

### 4. Generate Report

1. Click **"Generate Report"** in sidebar
2. Select sections to include
3. Add exhibits (specific emails)
4. Click **"Generate PDF Report"**

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

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Cmd/Ctrl + F` | Search |
| `Cmd/Ctrl + N` | New case |
| `Cmd/Ctrl + S` | Save |

## Tips

- Use **Artifacts Hub** to quickly find credentials, URLs, and PII
- **Timeline** helps understand communication patterns
- **Graph** reveals hidden relationships between entities
- Export reports for court proceedings
