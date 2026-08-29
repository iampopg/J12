# J12 Forensic - Complete System Audit

> **Total Documentation:** DATABASE_REFERENCE.md + SYSTEM_AUDIT.md
> **Coverage:** Database, Rust types, TypeScript types, Commands, Components, Configuration

---

## Table of Contents

1. [Tauri Command Registry](#tauri-command-registry)
2. [TypeScript Types & Interfaces](#typescript-types--interfaces)
3. [Component Props](#component-props)
4. [Configuration Files](#configuration-files)
5. [Utility Functions](#utility-functions)
6. [Forensic Regex Patterns](#forensic-regex-patterns)
7. [Artifact Taxonomy](#artifact-taxonomy)
8. [Error Handling](#error-handling)
9. [State Management](#state-management)
10. [Application Workflows](#application-workflows)

---

## Tauri Command Registry

### Case Management Commands (10)

| Command | Input | Output | Description |
|---------|-------|--------|-------------|
| `case_create` | `CaseCreateInput` | `Case` | Create new case |
| `case_list` | - | `Vec<Case>` | List all cases |
| `case_get` | `EmptyInput` | `Option<Case>` | Get case by ID |
| `case_update` | `CaseUpdateInput` | `()` | Update case details |
| `case_delete` | `Value` | `bool` | Delete case and all data |
| `auto_detect_targets` | `Value` | `Value` | Auto-detect targets from emails |
| `target_profile` | `Value` | `Value` | Get target profile |
| `open_external_url` | `String` | `()` | Open URL in system browser |
| `evidence_upload` | `EvidenceUploadInput` | `Evidence` | Upload evidence file |
| `evidence_list` | `EmptyInput` | `Vec<Evidence>` | List evidence for case |

### Evidence Commands (8)

| Command | Input | Output | Description |
|---------|-------|--------|-------------|
| `evidence_status` | `Value` | `Value` | Get evidence parsing status |
| `evidence_delete` | `Value` | `bool` | Delete evidence item |
| `write_temp_file` | `Value` | `String` | Write file to temp location |
| `open_file_dialog` | - | `Option<String>` | Open file picker dialog |
| `open_folder_dialog` | - | `Option<String>` | Open folder picker dialog |
| `read_file` | `String` | `Vec<u8>` | Read file contents |
| `parse_evidence` | `Value` | `u32` | Parse evidence and extract emails |
| `verify_evidence_hashes` | `Value` | `Value` | Verify evidence integrity |

### Email Commands (12)

| Command | Input | Output | Description |
|---------|-------|--------|-------------|
| `email_list` | `EmailListInput` | `Vec<EmailMessage>` | List emails with filters |
| `email_get` | `Value` | `Option<EmailMessage>` | Get email by ID |
| `email_headers` | `String` | `Value` | Get email headers |
| `search` | `SearchInput` | `Vec<EmailMessage>` | Basic search |
| `advanced_search` | `SearchInput` | `Vec<EmailMessage>` | Advanced search with operators |
| `emails_by_date` | `Value` | `Vec<EmailMessage>` | Get emails by date |
| `emails_between` | `Value` | `Vec<EmailMessage>` | Get emails between dates |
| `get_case_email_count` | `Value` | `i64` | Get total email count |
| `email_attachments` | `Value` | `Vec<Attachment>` | Get email attachments |
| `get_email_inline_images` | `Value` | `Vec<InlineImageData>` | Get inline images |
| `email_tags_list` | `Value` | `Vec<EmailTag>` | List email tags |
| `email_tag_add` | `EmailTagAddInput` | `EmailTag` | Add tag to email |

### Tag & Note Commands (11)

| Command | Input | Output | Description |
|---------|-------|--------|-------------|
| `email_tags_list` | `Value` | `Vec<EmailTag>` | List tags for an email |
| `email_tag_add` | `Value` | `EmailTag` | Add tag to email |
| `email_tag_remove` | `Value` | `()` | Remove tag from email |
| `email_notes_list` | `String` | `Vec<EmailNote>` | List email notes |
| `email_note_add` | `EmailNoteInput` | `EmailNote` | Add note to email |
| `email_note_delete` | `String` | `()` | Delete email note |
| `case_notes_list` | `EmptyInput` | `Vec<CaseNote>` | List case notes |
| `case_note_create` | `CaseNoteCreateInput` | `CaseNote` | Create case note |
| `case_note_update` | `CaseNoteUpdateInput` | `()` | Update case note |
| `case_note_toggle_pin` | `String` | `bool` | Toggle note pinned state |
| `case_note_delete` | `String` | `()` | Delete case note |

### Analysis Commands (10)

| Command | Input | Output | Description |
|---------|-------|--------|-------------|
| `findings_list` | `Value` | `Vec<Finding>` | List findings |
| `dashboard` | `Value` | `DashboardData` | Get dashboard statistics |
| `custody_chain` | `EmptyInput` | `Vec<CustodyEvent>` | Get custody chain |
| `run_analysis` | `Value` | `u32` | Run automated analysis |
| `update_finding_status` | `Value` | `()` | Update finding status |
| `add_finding_note` | `Value` | `()` | Add note to finding |
| `finding_emails` | `Value` | `Vec<EmailMessage>` | Get finding related emails |
| `extract_entities` | `Value` | `u32` | Extract entities from emails |
| `entity_list` | `Value` | `Vec<Entity>` | List entities |
| `entity_dive` | `EntityInput` | `Value` | Get entity details |

### Entity Commands (5)

| Command | Input | Output | Description |
|---------|-------|--------|-------------|
| `entity_emails` | `Value` | `Vec<EmailMessage>` | Get entity emails |
| `entity_heatmap` | `EntityInput` | `Value` | Get entity activity heatmap |
| `timeline_data` | `Value` | `Value` | Get timeline data |
| `graph_data` | `Value` | `Value` | Get communication graph data |
| `extract_entities` | `Value` | `u32` | Extract entities (duplicate) |

### Attachment Commands (7)

| Command | Input | Output | Description |
|---------|-------|--------|-------------|
| `case_attachments_summary` | `Value` | `AttachmentCategoryCounts` | Get attachment summary |
| `case_attachments_list` | `Value` | `Vec<CaseAttachmentItem>` | List case attachments |
| `export_attachment` | `Value` | `String` | Export attachment to disk |
| `get_attachment_preview` | `Value` | `Value` | Get attachment preview |
| `open_attachment_in_system` | `Value` | `()` | Open attachment in default app |
| `reveal_in_finder` | `Value` | `()` | Reveal file in system finder |
| `email_attachments` | `Value` | `Vec<Attachment>` | Get email attachments |

### Artifact Commands (4)

| Command | Input | Output | Description |
|---------|-------|--------|-------------|
| `case_artifacts_summary` | `Value` | `TaxonomyDomainSummary` | Get artifacts summary |
| `case_artifacts_list` | `Value` | `Vec<ForensicTaxonomyArtifact>` | List artifacts |
| `rescan_case_artifacts` | `Value` | `u32` | Rescan case for artifacts |
| `case_artifacts_summary` | `Value` | `Value` | Get artifacts summary |

### Bookmark Commands (5)

| Command | Input | Output | Description |
|---------|-------|--------|-------------|
| `bookmark_add` | `Value` | `ItemBookmark` | Add bookmark |
| `bookmark_remove` | `Value` | `()` | Remove bookmark |
| `bookmarks_list` | `Value` | `Vec<ItemBookmark>` | List bookmarks |
| `bookmark_check` | `Value` | `bool` | Check if item is bookmarked |
| `bookmark_add` | `Value` | `ItemBookmark` | Add bookmark (duplicate) |

### Report Commands (4)

| Command | Input | Output | Description |
|---------|-------|--------|-------------|
| `generate_report_data` | `Value` | `Value` | Generate report data |
| `export_report_pdf` | `Value` | `String` | Export report as PDF |
| `export_audit_log` | `Value` | `String` | Export audit log |
| `check_custody_chain` | `Value` | `Value` | Check custody chain integrity |

### IMAP Commands (4)

| Command | Input | Output | Description |
|---------|-------|--------|-------------|
| `imap_list_mailboxes` | `Value` | `Vec<String>` | List IMAP mailboxes |
| `imap_fetch_emails` | `Value` | `ImapAcquisitionResult>` | Fetch emails via IMAP |
| `imap_cancel_acquisition` | - | `()` | Cancel IMAP acquisition |
| `imap_test_connection` | `Value` | `bool` | Test IMAP connection |

### POP3 Commands (2)

| Command | Input | Output | Description |
|---------|-------|--------|-------------|
| `pop3_test_connection` | `Value` | `bool` | Test POP3 connection |
| `pop3_fetch_emails` | `Value` | `Pop3AcquisitionResult>` | Fetch emails via POP3 |

### AI Commands (25)

| Command | Input | Output | Description |
|---------|-------|--------|-------------|
| `ai_get_case_statistics` | `String` | `CaseStats` | Get case statistics |
| `ai_search_emails` | `SearchQuery` | `Vec<EmailResult>` | Search emails for AI |
| `ai_get_email` | `String` | `Option<EmailResult>` | Get email for AI |
| `ai_get_authentication_results` | `String` | `Option<AuthResults>` | Get auth results |
| `ai_get_entity` | `Value` | `Option<EntityData>` | Get entity data |
| `ai_get_timeline` | `Value` | `Vec<TimelineEvent>` | Get timeline |
| `ai_get_findings` | `String` | `Vec<FindingData>` | Get findings |
| `ai_get_case_context` | `String` | `Value` | Get case context |
| `ai_create_session` | `Value` | `String` | Create AI session |
| `ai_get_session_history` | `String` | `Vec<Value>` | Get session history |
| `ai_clear_session` | `String` | `()` | Clear AI session |
| `ai_natural_language_search` | `String` | `Value` | Natural language search |
| `ai_explain_evidence` | `Value` | `String` | Explain evidence |
| `ai_create_investigation_plan` | `Value` | `InvestigationPlan>` | Create investigation plan |
| `ai_execute_investigation_plan` | `Value` | `Value` | Execute investigation plan |
| `ai_analyze_timeline` | `Value` | `TimelineInterpretation>` | Analyze timeline |
| `ai_analyze_spoofing` | `String` | `SpoofingAnalysis>` | Analyze spoofing |
| `ai_triage_attachments` | `String` | `AttachmentTriage>` | Triage attachments |
| `ai_analyze_graph` | `Value` | `GraphAnalysis>` | Analyze communication graph |
| `fetch_kiloai_models` | - | `Vec<KiloAIModel>>` | Fetch kilo.ai models |
| `fetch_openrouter_models` | - | `Vec<KiloAIModel>>` | Fetch OpenRouter models |
| `ai_chat` | `Value` | `String` | Chat with AI |
| `ai_resolve_entities` | `String` | `EntityResolution>` | Resolve entities |
| `ai_detect_anomalies` | `Value` | `AnomalyDetection>` | Detect anomalies |
| `ai_generate_report` | `Value` | `InvestigationReport>` | Generate investigation report |

---

## TypeScript Types & Interfaces

### Authentication (auth.tsx)

#### User
```typescript
export interface User {
    username: string;
    password: string;
    role: string;
}
```

#### StoredAccount
```typescript
export interface StoredAccount extends User {
    created_at: string;
}
```

#### AuthState
```typescript
interface AuthState {
    isAuthenticated: boolean;
    user: User | null;
    error: string | null;
}
```

### Scan State (utils/scanState.ts)

#### ScanState
```typescript
export interface ScanState {
    status: 'idle' | 'scanning' | 'completed' | 'error';
    progress: number;
    currentEmail: number;
    totalEmails: number;
    artifactsFound: number;
    startTime: string | null;
    endTime: string | null;
    error: string | null;
}
```

### Email Types (views/EmailListView.tsx)

#### Email
```typescript
interface Email {
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
    headers_raw: string | null;
    body_text: string | null;
    body_html: string | null;
    folder_name: string | null;
    folder_category: string;
    is_deleted: boolean;
    deleted_recovered: boolean;
    risk_score: number;
    flags: string;
    attachment_count: number;
    image_count: number;
}
```

#### ColumnSettings
```typescript
export interface ColumnSettings {
    from: boolean;
    to: boolean;
    subject: boolean;
    date: boolean;
    risk: boolean;
    folder: boolean;
    attachments: boolean;
}
```

#### EmailTag
```typescript
export interface EmailTag {
    id: string;
    case_id: string;
    email_id: string;
    tag: string;
    color: string;
    created_by?: string;
    created_at: string;
}
```

#### Preset Tags
```typescript
const PRESET_TAGS = [
    { name: "Key Evidence", color: "#ef4444" },
    { name: "Privileged", color: "#8b5cf6" },
    { name: "Hot", color: "#f97316" },
    { name: "Responsive", color: "#22c55e" },
    { name: "Suspicious", color: "#eab308" },
    { name: "Reviewed", color: "#3b82f6" },
];
```

#### Evidence
```typescript
interface Evidence {
    id: string;
    filename: string;
    format: string;
    message_count: number;
}
```

#### SortField
```typescript
export type SortField = "date" | "name" | "from" | "subject" | "risk" | "folder";
```

#### SortDir
```typescript
export type SortDir = "asc" | "desc";
```

### Entity Types (views/EntityDiveView.tsx)

#### Entity
```typescript
interface Entity {
    id: string;
    email_address: string;
    display_name: string | null;
    sent_count: number;
    received_count: number;
    first_seen: string | null;
    last_seen: string | null;
    role: string;
}
```

#### EntityDetail
```typescript
interface EntityDetail {
    email: string;
    display_name: string | null;
    first_seen: string | null;
    last_seen: string | null;
    sent_count: number;
    received_count: number;
    deleted_count: number;
    flagged_count: number;
    total_count: number;
    aliases: string[];
    sent_to: [string, number][];
    received_from: [string, number][];
    top_subjects: [string, number][];
}
```

#### EntityEmail
```typescript
interface EntityEmail {
    id: string;
    from_addr: string;
    to_addrs: string;
    subject: string | null;
    date_sent_utc: string | null;
    risk_score: number;
    attachment_count: number;
    image_count: number;
}
```

#### TabType
```typescript
type TabType = "all" | "sent" | "received" | "deleted" | "flagged" | "partners";
```

#### EntityTier
```typescript
type EntityTier = "people" | "organizations" | "all";
```

### Attachment Types (views/AttachmentsView.tsx)

#### CaseAttachmentItem
```typescript
export interface CaseAttachmentItem {
    id: string;
    email_id: string;
    filename: string | null;
    sha256: string;
    mime_type: string | null;
    size_bytes: number;
    stored_path: string;
    entropy: number | null;
    risk_flags: string;
    email_subject: string | null;
    email_from: string | null;
    date_sent: string | null;
}
```

### Artifact Types (views/ArtifactsView.tsx)

#### TaxonomySubcategorySummary
```typescript
export interface TaxonomySubcategorySummary {
    subcategory_id: string;
    name: string;
    count: number;
}
```

#### TaxonomyDomainSummary
```typescript
export interface TaxonomyDomainSummary {
    domain_id: string;
    name: string;
    icon: string;
    total_count: number;
    subcategories: TaxonomySubcategorySummary[];
}
```

#### ForensicTaxonomyArtifact
```typescript
export interface ForensicTaxonomyArtifact {
    id: string;
    domain_id: string;
    subcategory_id: string;
    title: string;
    primary_value: string;
    secondary_value: string | null;
    details: string;
    severity: string;
    artifact_type: string;
    confidence: string | null;
    email_id: string;
    email_subject: string | null;
    email_from: string;
    date_sent_utc: string | null;
}
```

#### EmailMessage
```typescript
export interface EmailMessage {
    id: string;
    from_addr: string;
    to_addrs: string;
    subject: string | null;
    date_sent_utc: string | null;
    body_text: string | null;
    body_html: string | null;
    risk_score: number;
    folder_category: string;
}
```

### Finding Types (views/FindingsView.tsx)

#### Finding
```typescript
interface Finding {
    id: string;
    case_id: string;
    type: string;
    severity: string;
    confidence: string;
    title: string;
    description: string | null;
    evidence_refs: string;
    email_ids: string;
    status: string;
    created_at: string;
    reviewed_by: string | null;
    reviewed_at: string | null;
    notes: string | null;
}
```

#### EmailItem
```typescript
interface EmailItem {
    id: string;
    from_addr: string;
    to_addrs: string;
    subject: string | null;
    date_sent_utc: string | null;
    body_text: string | null;
    risk_score: number;
}
```

### Timeline Types (views/TimelineView.tsx)

#### DailyRecord
```typescript
interface DailyRecord {
    date: string;
    total_count: number;
    high_risk_count: number;
    deleted_count: number;
}
```

#### MonthlyRecord
```typescript
interface MonthlyRecord {
    month: string;
    total_count: number;
    high_risk_count: number;
    deleted_count: number;
}
```

#### TimelineEmail
```typescript
interface TimelineEmail {
    id: string;
    from_addr: string;
    to_addrs: string;
    subject: string | null;
    date_sent_utc: string | null;
    risk_score: number;
    is_deleted: boolean;
}
```

#### FilterCategory
```typescript
type FilterCategory = "all" | "sent" | "received" | "deleted" | "flagged" | "after_hours";
```

### Graph Types (views/GraphView.tsx)

#### GraphNode
```typescript
interface GraphNode {
    id: string;
    label: string;
    email: string;
    sent_count: number;
    received_count: number;
    risk_score: number;
    role: string;
}
```

#### GraphEdge
```typescript
interface GraphEdge {
    source: string;
    target: string;
    weight: number;
}
```

#### ExchangedEmail
```typescript
interface ExchangedEmail {
    id: string;
    from_addr: string;
    to_addrs: string;
    subject: string | null;
    date_sent_utc: string | null;
}
```

### Search Types (views/SearchView.tsx)

#### Email
```typescript
interface Email {
    id: string;
    from_addr: string;
    to_addrs: string;
    subject: string | null;
    date_sent_utc: string | null;
    body_text: string | null;
    risk_score: number;
    folder_category: string;
    attachment_count: number;
}
```

#### SortField
```typescript
type SortField = "date" | "from" | "subject" | "risk";
```

### Report Types (views/ReportView.tsx)

#### ReportSection
```typescript
export interface ReportSection {
    id: string;
    title: string;
    description: string;
    enabled: boolean;
    content: string | null;
}
```

#### Exhibit
```typescript
export interface Exhibit {
    id: string;
    email_id: string;
    description: string;
    included: boolean;
}
```

### Target Profile Types (views/TargetProfileView.tsx)

#### TargetProfile
```typescript
export interface TargetProfile {
    target_name: string | null;
    target_email: string | null;
    target_organization: string | null;
    investigation_type: string;
    total_emails: number;
    total_entities: number;
    total_findings: number;
    total_attachments: number;
    date_range: [string | null, string | null];
}
```

#### DetectedTarget
```typescript
export interface DetectedTarget {
    name: string;
    email: string;
    organization: string;
    confidence: number;
    evidence: string[];
}
```

### AI Types (views/AISetupPage.tsx)

#### AIConfig
```typescript
interface AIConfig {
    provider: string;
    model: string;
    apiKey: string;
    baseUrl: string;
}
```

#### DetectedInstance
```typescript
interface DetectedInstance {
    name: string;
    port: number;
    status: string;
    version: string;
}
```

#### KiloAIModel
```typescript
interface KiloAIModel {
    id: string;
    name: string;
    provider: string;
    is_free: boolean;
}
```

### Bookmark Types (views/EvidenceLockerView.tsx)

#### ItemBookmark
```typescript
export interface ItemBookmark {
    id: string;
    case_id: string;
    item_id: string;
    item_type: string;
    label: string;
    color: string;
    note: string;
    created_at: string;
    item_title?: string;
    item_subject?: string;
    item_from?: string;
    item_date?: string;
}
```

### Component Props Types

#### RichEmailBodyViewer Props
```typescript
interface Props {
    bodyHtml: string | null;
    bodyText: string | null;
    inlineImages: InlineImageData[];
}
```

#### ParsedEmailBody
```typescript
interface ParsedEmailBody {
    segments: BodySegment[];
    hasQuotedText: boolean;
    hasSignature: boolean;
}
```

#### InlineImageData
```typescript
interface InlineImageData {
    content_id: string;
    mime_type: string;
    data: string; // base64
    filename: string | null;
}
```

#### EmailDetailModal Props
```typescript
interface Props {
    email: EmailModalData | null;
    onClose: () => void;
    onTagAdd: (emailId: string, tag: string) => void;
    onTagRemove: (emailId: string, tag: string) => void;
    onNoteAdd: (emailId: string, content: string) => void;
    onBookmarkAdd: (emailId: string) => void;
}
```

#### EmailModalData
```typescript
export interface EmailModalData {
    id: string;
    from_addr: string;
    from_display: string | null;
    to_addrs: string;
    cc_addrs: string;
    subject: string | null;
    date_sent_utc: string | null;
    headers_raw: string | null;
    body_text: string | null;
    body_html: string | null;
    risk_score: number;
    flags: string;
    attachment_count: number;
    image_count: number;
}
```

#### AttachmentItem
```typescript
interface AttachmentItem {
    id: string;
    filename: string | null;
    mime_type: string | null;
    size_bytes: number;
    sha256: string;
    entropy: number | null;
    risk_flags: string;
}
```

#### AIChatWidget Props
```typescript
interface Props {
    caseId: string;
    onClose: () => void;
}
```

#### AIMessage
```typescript
interface AIMessage {
    role: "user" | "assistant" | "system";
    content: string;
    timestamp: string;
    evidence_refs?: string[];
}
```

#### BookmarkButton Props
```typescript
interface Props {
    itemId: string;
    itemType: string;
    itemTitle?: string;
    caseId: string;
}
```

#### ItemBookmark
```typescript
interface ItemBookmark {
    id: string;
    case_id: string;
    item_id: string;
    item_type: string;
    label: string;
    color: string;
    note: string;
    created_at: string;
}
```

#### J12Logo Props
```typescript
interface J12LogoProps {
    size?: number;
    className?: string;
}
```

#### FooterSignature Props
```typescript
interface FooterProps {
    variant?: "default" | "compact";
}
```

#### LogEntry (PlaceholderViews)
```typescript
interface LogEntry {
    timestamp: string;
    level: string;
    message: string;
}
```

---

## Configuration Files

### Cargo.toml (Rust Dependencies)

| Dependency | Version | Purpose |
|------------|---------|---------|
| `tauri` | 2.0 | Desktop app framework |
| `tauri-plugin-dialog` | 2.0 | File dialogs |
| `tauri-plugin-fs` | 2.0 | File system access |
| `rusqlite` | 0.37 | SQLite database |
| `serde` | 1.0 | Serialization |
| `serde_json` | 1.0 | JSON handling |
| `chrono` | 0.4 | Date/time handling |
| `uuid` | 1.0 | UUID generation |
| `sha2` | 0.10 | SHA-256/SHA-512 hashing |
| `regex` | 1.0 | Regular expressions |
| `tokio` | 1.0 | Async runtime |
| `lazy_static` | 1.5 | Static initialization |
| `dirs` | 6.0 | System directories |
| `reqwest` | 0.12 | HTTP client |
| `md5` | 0.7 | MD5 hashing |
| `base64` | 0.22 | Base64 encoding |
| `bytes` | 1.0 | Byte handling |
| `futures` | 0.3 | Async utilities |
| `imap` | 3.0 | IMAP client |
| `mailparse` | 0.15 | Email parsing |
| `lettre` | 0.11 | SMTP client |

### package.json (Frontend Dependencies)

| Dependency | Version | Purpose |
|------------|---------|---------|
| `react` | 18.3 | UI framework |
| `react-dom` | 18.3 | DOM rendering |
| `@tauri-apps/api` | 2.0 | Tauri API |
| `@tauri-apps/plugin-dialog` | 2.0 | Dialog plugin |
| `@tauri-apps/plugin-fs` | 2.0 | FS plugin |
| `typescript` | 5.6 | Type safety |
| `vite` | 6.0 | Build tool |
| `@vitejs/plugin-react` | 4.3 | React plugin |

### tauri.conf.json

| Setting | Value |
|---------|-------|
| `productName` | J12 Forensic |
| `version` | 1.0.0 |
| `identifier` | com.j12.forensic |
| `bundle` | macOS, Windows, Linux |
| `frontendDist` | ../dist |
| `devUrl` | http://localhost:5173 |

---

## Utility Functions

### Database Utilities (db.rs)

| Function | Signature | Description |
|----------|-----------|-------------|
| `parse_dt` | `fn(s: &str) -> DateTime<Utc>` | Parse ISO 8601 timestamp |
| `compute_sha256` | `fn(path: &PathBuf) -> Result<String>` | Compute SHA-256 hash |
| `compute_sha512` | `fn(path: &PathBuf) -> Result<String>` | Compute SHA-512 hash |
| `detect_format` | `fn(filename: &str) -> String` | Detect email file format |
| `generate_id` | `fn() -> String` | Generate UUID v4 |

### Analysis Utilities (analysis.rs)

| Function | Signature | Description |
|----------|-----------|-------------|
| `analyze_headers` | `fn(headers_raw: &str) -> HeaderAnalysis` | Analyze email headers |
| `analyze_authentication` | `fn(headers: &str, domain: &str, dns: Option<&str>) -> AuthResults` | Analyze SPF/DKIM/DMARC |
| `detect_spoofing` | `fn(from: &str, display: Option<&str>, headers: &str, auth: &AuthResults) -> Vec<SpoofingFinding>` | Detect spoofing |
| `analyze_attachment` | `fn(filename: Option<&str>, mime: Option<&str>, size: u64, entropy: Option<f64>, risk_flags: Option<&str>) -> AttachmentAnalysis` | Analyze attachment |
| `analyze_attachment_metadata` | `fn(...) -> AttachmentAnalysis` | Analyze attachment metadata |
| `detect_content_threats` | `fn(body: &str) -> Vec<String>` | Detect content threats |
| `generate_findings` | `fn(email_id: &str, ...) -> Vec<NewFinding>` | Generate findings |
| `calculate_risk_score` | `fn(...) -> u8` | Calculate risk score |

### Artifact Utilities (commands/artifacts.rs)

| Function | Signature | Description |
|----------|-----------|-------------|
| `luhn_check` | `fn(num_str: &str) -> bool` | Validate credit card (Luhn) |
| `validate_routing_number` | `fn(num_str: &str) -> bool` | Validate US routing number |
| `validate_ssn` | `fn(ssn_str: &str) -> bool` | Validate US SSN |
| `clean_email_body_for_forensic_scan` | `fn(raw_body: &str) -> String` | Clean body for scanning |
| `is_automated_or_noise_email` | `fn(addr: &str) -> bool` | Detect automated emails |
| `validate_base58check` | `fn(addr: &str, versions: &[u8]) -> bool` | Validate BTC/LTC/DOGE |
| `validate_solana_address` | `fn(addr: &str, context: &str) -> bool` | Validate Solana address |
| `validate_btc_bech32` | `fn(addr: &str) -> bool` | Validate BTC Bech32 |
| `validate_eth_address` | `fn(addr: &str) -> bool` | Validate ETH address |

### Parser Utilities (parser.rs, pst.rs)

| Function | Signature | Description |
|----------|-----------|-------------|
| `parse_file` | `fn(path: &Path) -> Result<Vec<RawEmail>>` | Parse email file |
| `parse_msg` | `fn(path: &Path) -> Result<Vec<RawEmail>>` | Parse MSG file |
| `parse_emlx` | `fn(path: &Path) -> Result<Vec<RawEmail>>` | Parse EMLX file |
| `PstParser::can_parse` | `fn(path: &Path) -> bool` | Check if PST parseable |
| `PstParser::parse` | `fn(path: &Path) -> Result<Vec<RawEmail>>` | Parse PST file |
| `PstParser::get_folder_hierarchy` | `fn(path: &Path) -> Result<Vec<PstFolder>>` | Get PST folder structure |
| `PstParser::recover_deleted` | `fn(path: &Path) -> Result<Vec<RawEmail>>` | Recover deleted from PST |

### IMAP Utilities (imap_acquisition.rs)

| Function | Signature | Description |
|----------|-----------|-------------|
| `categorize_imap_folder` | `fn(folder_name: &str) -> String` | Categorize IMAP folder |
| `list_mailboxes` | `fn(config: &ImapConfig) -> Result<Vec<String>>` | List IMAP mailboxes |
| `fetch_emails_streaming` | `fn<F, G>(...) -> Result<ImapAcquisitionResult>` | Fetch emails with streaming |

---

## Forensic Regex Patterns

### Credentials & Secrets

| Pattern | Regex | Description |
|---------|-------|-------------|
| Credential Pair | `(?i)(?:username\|user\|login\|email)[:=\s]+([a-zA-Z0-9._%+\-@]{3,50})\s*(?:password\|pwd\|pass)[:=\s]+([^\s,;]{4,50})` | Username + password |
| Password Standalone | `(?i)(?:password\|passwd\|passcode\|secret\s*key)[:=\s]+([^\s,;]{6,60})` | Password only |
| API Keys | `\b(AKIA[0-9A-Z]{16}\|sk_live_[0-9a-zA-Z]{24,40}\|ghp_[0-9a-zA-Z]{36}\|AIza[0-9A-Za-z\-_]{35})\b` | AWS/GitHub/Google API keys |
| Bearer Token | `Bearer\s+([A-Za-z0-9\-\._~\+\/]{25,}=*)` | OAuth bearer tokens |
| JWT | `(eyJ[A-Za-z0-9-_=]{15,}\.[A-Za-z0-9-_=]{15,}\.[A-Za-z0-9-_.+/=]{10,})` | JSON Web Tokens |
| SSH Key | `-----BEGIN (?:RSA\|DSA\|EC\|OPENSSH) PRIVATE KEY-----` | SSH private keys |
| Seed Phrase | `(?i)(?:seed\s*phrase\|recovery\s*phrase\|mnemonic\s*phrase\|wallet\s*seed)[:=\-]?\s*([a-z]{3,10}(?:\s+[a-z]{3,10}){11,23})` | Crypto seed phrases |
| Private Key | `(?i)(?:private\s*key\|privkey)[:=\s]+([0-9a-fA-F]{64})\b` | Hex private keys |

### Financial & Banking

| Pattern | Regex | Description |
|---------|-------|-------------|
| Credit Card (spaced) | `\b((?:4[0-9]{3}\|5[1-5][0-9]{2}\|6011\|3[47][0-9]{2})[\s\-][0-9]{4}[\s\-][0-9]{4}[\s\-][0-9]{4})\b` | CC with spaces/dashes |
| Credit Card (raw) | `\b(4[0-9]{12}(?:[0-9]{3})?\|5[1-5][0-9]{14}\|3[47][0-9]{13}\|6011[0-9]{12})\b` | CC without spaces |
| Routing Number | `(?i)(?:routing(?:\s*number\|#)?\|aba(?:\s*#\|\s*no)?)\s*[:#=]?\s*(\b(?:0[1-9]\|[123][0-9]\|6[1-9]\|7[0-2]\|80)\d{7}\b)` | US ABA routing |
| IBAN | `(?i)(?:iban)\s*[:#=]?\s*([A-Z]{2}[0-9]{2}[A-Z0-9]{4}[0-9]{7}(?:[A-Z0-9]?){0,16})\b` | International Bank Account |
| SWIFT/BIC | `(?i)(?:swift(?:\s*code\|\s*bic)?\|bic(?:\s*code)?\|swift/bic)\s*[:#=]?\s*([A-Z]{6}[A-Z0-9]{2}(?:[A-Z0-9]{3})?)\b` | SWIFT/BIC codes |
| Bank Account | `(?i)(?:bank\s*account(?:\s*number\|\s*#)?\|acct(?:\s*number\|\s*#))\s*[:#=]?\s*([0-9]{8,17})\b` | Bank account numbers |
| Sort Code | `(?i)(?:sort\s*code\|sort-code\|sortcode)\s*[:#=]?\s*(\d{2}[-\s]?\d{2}[-\s]?\d{2})\b` | UK sort codes |

### Cryptocurrency

| Pattern | Regex | Description |
|---------|-------|-------------|
| BTC Legacy | `\b([13][a-km-zA-HJ-NP-Z1-9]{25,34})\b` | Bitcoin legacy addresses |
| BTC Bech32 | `\b(bc1[a-zA-HJ-NP-Z0-9]{39,59})\b` | Bitcoin SegWit |
| Ethereum | `\b(0x[a-fA-F0-9]{40})\b` | Ethereum addresses |
| TRON | `\b(T[A-Za-z1-9]{33})\b` | Tron addresses |
| Solana | `\b([1-9A-HJ-NP-Za-km-z]{32,44})\b` | Solana addresses |
| Litecoin | `\b([LM3][a-km-zA-HJ-NP-Z1-9]{25,34})\b` | Litecoin addresses |
| Dogecoin | `\b(D[A-Za-z1-9]{33})\b` | Dogecoin addresses |
| Monero | `\b(4[0-9AB][1-9A-HJ-NP-Za-km-z]{93})\b` | Monero addresses |
| Crypto URI | `(?i)\b((?:bitcoin\|ethereum\|litecoin\|doge\|solana\|monero):[a-zA-Z0-9?=_&%-]+)\b` | Crypto URIs |

### PII & Identity

| Pattern | Regex | Description |
|---------|-------|-------------|
| SSN | `\b(\d{3}-\d{2}-\d{4})\b` | US Social Security Number |
| Passport | `(?i)(?:passport(?:\s*#\|\s*no\|\s*number)?)\s*[:#=]?\s*([A-PR-WYa-pr-wy][0-9]{7,8})\b` | Passport numbers |
| Driver License | `(?i)(?:driver'?s?\s*license\|driving\s*licence)\s*(?:#\|no\|number)?[:=\s]*([A-Z0-9]{6,14})\b` | Driver's license |
| EIN | `(?i)(?:ein\|federal\s*tax\s*id)\s*[:#=]?\s*(\d{2}-\d{7})\b` | Employer ID |

### Locations & Travel

| Pattern | Regex | Description |
|---------|-------|-------------|
| Street Address | `\b([0-9]{1,5}\s+[A-Z][a-zA-Z0-9\s.,]{2,30}\s+(?:Street\|St\.\|...)` | US street addresses |
| Hotel Confirmation | `(?i)(?:hotel\|lodging\|flight)\s*(?:confirmation\|booking\|reservation)\s*(?:#\|no\|number)?[:=\s]*([A-Z0-9]{6,12})\b` | Booking confirmations |
| GPS Coordinates | `\b(-?[0-9]{1,2}\.[0-9]{4,8}\s*,\s*-?[0-9]{1,3}\.[0-9]{4,8})\b` | GPS coordinates |

### Threats & Contraband

| Pattern | Regex | Description |
|---------|-------|-------------|
| Weapons | `(?i)\b(glock\|beretta\|ar-15\|ak-47\|silencer\|ghost\s*gun\|...)` | Firearms/weapons |
| Narcotics | `(?i)\b(cocaine\|coke\|heroin\|fentanyl\|methamphetamine\|...)` | Illegal drugs |
| Explosives | `(?i)\b(bomb\|explosive\|detonator\|c4\|ied\|...)` | Explosives |
| Terrorism | `(?i)\b(al-qaeda\|boko\s*haram\|hezbollah\|hamas\|...)` | Terrorist organizations |

### Malware & Cyber IOCs

| Pattern | Regex | Description |
|---------|-------|-------------|
| CVE | `(CVE-\d{4}-\d{4,7})` | Common Vulnerabilities |
| C2 | `(?i)\b(command\s*and\s*control\|c2\s*server\|...)` | Command & control |

### Corporate & Legal

| Pattern | Regex | Description |
|---------|-------|-------------|
| Confidential | `(?i)\b(strictly\s+confidential\|attorney[- ]client\s+privilege\|...)` | Legal confidentiality |
| NDA | `(?i)\b(non[- ]disclosure\s*agreement\|\bnda\b\|...)` | Non-disclosure agreements |

### Phishing & Social Engineering

| Pattern | Regex | Description |
|---------|-------|-------------|
| Phishing Credentials | `(?i)\b(verify\s*your\s*identity\|confirm\s*your\s*password\|...)` | Credential phishing |
| Phishing Finance | `(?i)\b(wire\s*transfer\s*urgently\|urgent\s*wire\s*payment\|...)` | Financial phishing |

### Phone Numbers

| Pattern | Regex | Description |
|---------|-------|-------------|
| Nigeria Phone | `(?i)\b(?:\+?234\|0)\s?[789][01](?:[\s.-]?\d){8}\b` | Nigerian phone numbers |
| International Phone | `(?i)(?:phone\|tel\|mobile\|call\|whatsapp\|cell\|contact)[:=\s]*([+]?[1-9]\d{0,2}[-\s]?(?:\(?\d{1,4}\)?[-\s]?)?\d{3,4}[-\s]?\d{3,4})\b` | International phones |

### African & Global IDs

| Pattern | Regex | Description |
|---------|-------|-------------|
| BVN | `(?i)\b(?:bvn\|bank\s*verification\s*number)[:=\s]*([0-9]{11})\b` | Nigerian Bank Verification |
| NIN | `(?i)\b(?:nin\|national\s*(?:identity\|id)\s*(?:number\|no)?\|national\s*id)[:=\s]*([0-9]{11})\b` | Nigerian National ID |
| TIN | `(?i)\b(?:tin\|tax\s*(?:identity\|id\|identification)\s*number)[:=\s]*([0-9]{8,12})\b` | Tax ID |

### URLs

| Pattern | Regex | Description |
|---------|-------|-------------|
| URL | `https?://[a-zA-Z0-9\-._~:/?#\[\]@!$&'()*+,;=%]+` | HTTP/HTTPS URLs |

---

## Artifact Taxonomy

### Domain Categories

| Domain ID | Name | Icon | Subcategories |
|-----------|------|------|---------------|
| `credentials` | Credentials & Secrets | 🔑 | credential_pair, password_standalone, api_keys, bearer_token, jwt, ssh_key, seed_phrase, private_key |
| `financial` | Financial & Banking | 💳 | credit_card_spaced, credit_card_raw, routing_number, iban, swift, bank_account, sort_code |
| `cryptocurrency` | Cryptocurrency | ₿ | btc_legacy, btc_bech32, ethereum, tron, solana, litecoin, dogecoin, monero, crypto_uri |
| `pii` | PII & Identity | 🪪 | ssn, passport, driver_license, ein |
| `locations` | Locations & Travel | 📍 | street_address, hotel_confirmation, gps |
| `threats` | Threats & Contraband | ⚠️ | weapons, narcotics, explosives, terrorism |
| `malware` | Malware & Cyber IOCs | 🦠 | cve, c2 |
| `corporate` | Corporate & Legal | 🏢 | confidential, nda |
| `phishing` | Phishing & Social Engineering | 🎣 | phishing_credentials, phishing_finance |
| `phones` | Phone Numbers | 📞 | phone_nigeria, phone_international |
| `african_ids` | African & Global IDs | 🇳🇬 | bvn, nin, tin |
| `urls` | URLs | 🔗 | url |

### App/Signatures Categories

| Domain ID | Name | Apps |
|-----------|------|------|
| `social_media` | Social Media & Communities | Snapchat, Twitter/X, Instagram, Facebook, TikTok, LinkedIn, Reddit, Pinterest, YouTube, Twitch, Tumblr, Threads, Bluesky, VKontakte |
| `messaging_apps` | Messaging & Encrypted Chat | Telegram, Signal, WhatsApp, Discord, Session, Threema, Wickr, Element/Matrix, Viber, WeChat, Line, Skype, Kik |
| `crypto_platforms` | Crypto Platforms & Web3 | Binance, Coinbase, Kraken, KuCoin, MetaMask, Trust Wallet, Ledger, Trezor, Bybit, OKX, Bitfinex, Uniswap, Phantom, Exodus, Paxful, Gemini, OpenSea, BitMEX, Paybis, Zypto, CoinCodex, Quidax, Luno, Busha, Yellow Card, Roqqu |
| `dating_apps` | Dating & Romance | Tinder, Bumble, Hinge, Badoo, Grindr, Match.com, OkCupid, Ashley Madison, OnlyFans, Fansly |
| `fintech_banking` | Fintech & Banking | Huntington Bank, Chase, Simmons Bank, Armed Forces Bank, Bank of America, Wells Fargo, Citibank, Capital One, PNC Bank, ApexPay, TaxAct, PayPal, Stripe, Venmo, Cash App, Revolut, Wise, Payoneer, Zelle, Robinhood, eToro, Monzo, N26, Klarna, Western Union, GTBank, Access Bank, Zenith Bank, First Bank, UBA, Kuda Bank, OPay, PalmPay, Moniepoint, Stanbic IBTC, Fidelity Bank, Wema Bank, Sterling Bank, Union Bank, Ecobank, Flutterwave, Paystack, PiggyVest, Cowrywise, Carbon, FairMoney, Chipper Cash, Remita, M-Pesa, MTN MoMo, Airtel Money |
| `mobile_apps` | Mobile & On-Demand | Uber, Lyft, Bolt, InDrive, DoorDash, Deliveroo, Chowdeck, Glovo, Instacart, Airbnb, Booking.com, Spotify, Netflix, Disney+, Apple Services, Google Play, MTN, Airtel, Glo, 9mobile, Duolingo, Strava |
| `ecommerce` | E-Commerce & Marketplaces | Amazon, Jumia, Konga, eBay, AliExpress, Temu, Shein, Etsy, Walmart, Vinted, StockX |
| `cloud_dev` | AI, Cloud & Developer | OpenAI, Anthropic, Midjourney, GitHub, GitLab, AWS, GCP, Vercel, Cloudflare |
| `vpn_privacy` | VPNs & Privacy | ProtonMail, Tutanota, SimpleLogin, DuckDuckGo, NordVPN, ExpressVPN, Mullvad, 1Password, Bitwarden |
| `remote_access` | Remote Access & Productivity | AnyDesk, TeamViewer, RustDesk, Zoom, Slack, Notion |

---

## Error Handling

### Rust Error Patterns

| Pattern | Usage | Example |
|---------|-------|---------|
| `Result<T, String>` | Most commands | `Result<Case, String>` |
| `Result<T, std::io::Error>` | File operations | `Result<String, std::io::Error>` |
| `Option<T>` | Optional values | `Option<Case>` |
| `unwrap_or_default()` | Default values | `row.get::<_, Option<String>>(x)?.unwrap_or_default()` |
| `map_err(|e| e.to_string())` | Error conversion | `.map_err(|e| e.to_string())?` |
| `ok()` | Ignore errors | `conn.execute(...).ok()` |

### Frontend Error Patterns

| Pattern | Usage | Example |
|---------|-------|---------|
| `try/catch` | Async operations | `try { await invoke(...) } catch (e) { ... }` |
| Error state | Component state | `const [error, setError] = useState<string \| null>(null)` |
| Error boundary | React error handling | Not implemented |

---

## State Management

### Application State (main.rs)

```rust
pub struct AppState {
    pub db: Mutex<Database>,
    pub data_dir: PathBuf,
    pub cancel_imap: Arc<AtomicBool>,
}
```

### Authentication State (auth.tsx)

```typescript
interface AuthState {
    isAuthenticated: boolean;
    user: User | null;
    error: string | null;
}
```

### Local Storage Keys

| Key | Purpose |
|-----|---------|
| `j12_user` | Stored authenticated user |
| `j12_accounts` | All user accounts |
| `j12_ai_config` | AI provider configuration |
| `j12_scan_state` | Artifact scan progress |

---

## Application Workflows

### 1. Authentication Flow
```
Login Page → Validate Credentials → Store Session → Case List Page
```

### 2. Case Creation Flow
```
Case List → New Case Form → Create Case → Case Workspace
```

### 3. Evidence Import Flow
```
Case Workspace → Evidence Acquisition → Select Files → Parse → Emails Extracted
```

### 4. Analysis Flow
```
Case Workspace → Run Analysis → Extract Entities → Generate Findings → Update Risk Scores
```

### 5. Investigation Flow
```
Emails → Search/Filter → Entity Dive → Timeline → Graph → Findings
```

### 6. Report Flow
```
Case Workspace → Generate Report → Select Sections → Add Exhibits → Export PDF
```

### 7. AI Investigation Flow
```
AI Setup → Configure Provider → Create Session → Chat/Investigate → Generate Report
```

### 8. Artifact Extraction Flow
```
Emails → Scan for Artifacts → Categorize → Display in Artifacts Hub
```

---

## File Statistics

| Category | Count |
|----------|-------|
| Database Tables | 25 |
| Database Indexes | 64 |
| Rust Structs | 75+ |
| Rust Enums | 3 |
| Rust Functions | 100+ |
| Tauri Commands | 90 |
| TypeScript Interfaces | 50+ |
| TypeScript Types | 20+ |
| React Components | 20+ |
| Regex Patterns | 40+ |
| Artifact Domains | 12 |
| App Signatures | 80+ |

---

*Generated: 2026-08-26*
*Database: J12 Forensic Email Investigation Platform*
*Version: 1.0.0*
