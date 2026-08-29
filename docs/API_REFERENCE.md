# API Reference

## Tauri Commands (Backend)

### Case Management

| Command | Input | Output | Description |
|---------|-------|--------|-------------|
| `case_create` | `CaseCreateInput` | `Case` | Create new case |
| `case_list` | - | `Case[]` | List all cases |
| `case_get` | `EmptyInput` | `Case?` | Get case by ID |
| `case_update` | `CaseUpdateInput` | `void` | Update case details |
| `case_delete` | `EmptyInput` | `bool` | Delete case and all data |

### Evidence

| Command | Input | Output | Description |
|---------|-------|--------|-------------|
| `evidence_upload` | `EvidenceUploadInput` | `Evidence` | Upload evidence file |
| `evidence_list` | `EmptyInput` | `Evidence[]` | List evidence for case |
| `parse_evidence` | `evidenceId: String` | `u32` | Parse evidence and extract emails |

### Emails

| Command | Input | Output | Description |
|---------|-------|--------|-------------|
| `email_list` | `EmailListInput` | `EmailMessage[]` | List emails with filters |
| `email_get` | `EmptyInput` | `EmailMessage?` | Get email by ID |
| `search` | `SearchInput` | `EmailMessage[]` | Basic search (LIKE) |
| `fts_search` | `FtsSearchInput` | `FtsSearchResponse` | **FTS5 full-text search** |

### FTS5 Full-Text Search

**Command:** `fts_search`

**Input:**
```typescript
interface FtsSearchInput {
  case_id: string;
  query: string;
  limit?: number;
  offset?: number;
  evidence_id?: string;
}
```

**Output:**
```typescript
interface FtsSearchResponse {
  total_hits: number;
  execution_ms: number;
  query_parsed: string;
  items: FtsSearchResultItem[];
}

interface FtsSearchResultItem {
  id: string;
  evidence_id: string;
  case_id: string;
  message_id: string | null;
  from_addr: string;
  from_display: string | null;
  to_addrs: string;
  cc_addrs: string;
  subject: string | null;
  date_sent: string | null;
  date_sent_utc: string | null;
  folder_category: string;
  is_deleted: boolean;
  deleted_recovered: boolean;
  risk_score: number;
  flags: string | null;
  snippet: string | null;  // Highlighted match snippet
  match_rank: number;      // BM25 relevance score
}
```

**Query Syntax:**
| Syntax | Example | Description |
|--------|---------|-------------|
| Plain words | `wire transfer` | All words (implicit AND) |
| AND | `fraud AND wire` | Both words |
| OR | `wire OR offshore` | Either word |
| NOT | `wire NOT domestic` | Exclude word |
| Phrase | `"strictly confidential"` | Exact phrase |
| Wildcard | `crypt*` | Prefix match |
| Proximity | `NEAR("wire" "transfer", 5)` | Words within 5 words |
| Stemming | `transfer` | Matches transfers, transferring |

### Attachments

| Command | Input | Output | Description |
|---------|-------|--------|-------------|
| `email_attachments` | `emailId: String` | `Attachment[]` | Get attachments for email |
| `case_attachments_list` | `AttachmentListInput` | `CaseAttachmentItem[]` | List case attachments |
| `case_attachments_summary` | `caseId: String` | `AttachmentCategoryCounts` | Attachment counts by category |
| `get_attachment_preview` | `attachmentId: String` | `string?` | Get base64 preview |
| `open_attachment_in_system` | `attachmentId: String` | `string` | Open in OS viewer |
| `reveal_in_finder` | `attachmentId: String` | `string` | Reveal in file manager |
| `export_attachment` | `attachmentId: String` | `string` | Export to Downloads |
| `extract_attachment_text` | `attachmentId: String` | `string` | **Extract text from document** |
| `batch_extract_case_attachments` | `caseId: String` | `number` | **Batch extract all attachments** |

### Document Extraction & OCR

| Command | Input | Output | Description |
|---------|-------|--------|-------------|
| `extract_attachment_text` | `attachmentId: String` | `string` | Extract text from PDF/DOCX/XLSX/PPTX |
| `batch_extract_case_attachments` | `caseId: String` | `number` | Extract text from all case attachments |
| `ocr_attachment_image` | `attachmentId: String` | `string` | OCR image to text |

**Supported Formats:**
- **PDF:** Object streams, text chunks, bookmarks
- **Word (.docx):** Paragraphs, tables, headers
- **Excel (.xlsx):** Cell values, shared strings
- **PowerPoint (.pptx):** Slide text, presenter notes
- **Images:** PNG, JPG, TIFF, BMP, WEBP (via macOS Vision or Tesseract)
- **Text:** TXT, CSV, RTF, HTML, XML, JSON

### IMAP Acquisition

| Command | Input | Output | Description |
|---------|-------|--------|-------------|
| `imap_list_mailboxes` | `ImapConfigInput` | `ImapMailbox[]` | List IMAP folders |
| `imap_fetch_emails` | `ImapFetchInput` | `ImapAcquisitionResult` | Fetch emails via IMAP |
| `imap_cancel_acquisition` | - | `void` | Cancel active IMAP fetch |
| `imap_test_connection` | `ImapConfigInput` | `bool` | Test IMAP connection |

**Authentication:**
- **Password:** Traditional LOGIN authentication
- **OAuth2:** SASL XOAUTH2 (RFC 7628) for Google Workspace and Microsoft 365

### POP3 Acquisition

| Command | Input | Output | Description |
|---------|-------|--------|-------------|
| `pop3_test_connection` | `Pop3ConfigInput` | `bool` | Test POP3 connection |
| `pop3_fetch_emails` | `Pop3FetchInput` | `Pop3AcquisitionResult` | Fetch emails via POP3 |

### Analysis

| Command | Input | Output | Description |
|---------|-------|--------|-------------|
| `dashboard` | `EmptyInput` | `Dashboard` | Get case statistics |
| `entity_list` | `EmptyInput` | `Entity[]` | List entities |
| `entity_dive` | `address: String` | `EntityDetail` | Entity deep dive |
| `graph_data` | `EmptyInput` | `GraphData` | Communication graph |
| `timeline_data` | `EmptyInput` | `TimelineEvent[]` | Timeline events |
| `findings` | `EmptyInput` | `Finding[]` | Security findings |

### AI Commands

| Command | Input | Output | Description |
|---------|-------|--------|-------------|
| `ai_get_case_statistics` | `caseId: String` | `CaseStats` | Get case statistics |
| `ai_search_emails` | `SearchQuery` | `EmailResult[]` | Search emails for AI |
| `ai_get_findings` | `caseId: String` | `FindingData[]` | Get findings for AI |
| `ai_chat` | `AIChatInput` | `String` | Chat with AI |
| `fetch_kiloai_models` | - | `KiloAIModel[]` | Fetch kilo.ai free models |
| `fetch_openrouter_models` | - | `KiloAIModel[]` | Fetch OpenRouter free models |

## Data Types

### EmailMessage
```typescript
interface EmailMessage {
  id: string;
  evidence_id: string;
  case_id: string;
  message_id: string | null;
  from_addr: string;
  from_display: string | null;
  to_addrs: string;
  subject: string | null;
  date_sent: string | null;
  headers_raw: string | null;
  body_text: string | null;
  body_html: string | null;
  folder_category: string;
  risk_score: number;
}
```

### Entity
```typescript
interface Entity {
  id: string;
  email_address: string;
  display_name: string | null;
  sent_count: number;
  received_count: number;
  first_seen: string | null;
  last_seen: string | null;
}
```

### Finding
```typescript
interface Finding {
  id: string;
  type: string;        // FRAUD, SPOOF, ATTACH, ANOMALY
  severity: string;    // critical, high, medium, low
  title: string;
  description: string;
  status: string;      // open, reviewed, confirmed, rejected
}
```

## Database Schema

See `src-tauri/src/db.rs` for complete schema.

### Key Tables
- `cases` - Case records
- `evidence_items` - Evidence sources
- `emails` - Parsed emails
- `attachments` - Email attachments
- `entities` - People/organizations
- `findings` - Security findings
- `forensic_artifacts` - Extracted artifacts
- `timeline_events` - Timeline entries
- `communication_edges` - Graph relationships
- `chain_of_custody` - Audit trail
