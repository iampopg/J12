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
| `search` | `SearchInput` | `EmailMessage[]` | Advanced search |

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
