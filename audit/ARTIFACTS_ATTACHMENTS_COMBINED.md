# Artifacts + Attachments System - Complete Audit

> **Date:** 2026-08-28
> **Scope:** How artifacts and attachments work together, load, display, and connect

---

## 1. System Overview

```
┌─────────────────────────────────────────────────────────────────────────┐
│                        CASE WORKSPACE                                   │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  Sidebar Tabs:                                                          │
│  ┌──────────────┐  ┌──────────────┐                                    │
│  │  Artifacts   │  │  Attachments │  ← Separate tabs, same case data   │
│  └──────┬───────┘  └──────┬───────┘                                    │
│         │                 │                                             │
│         ▼                 ▼                                             │
│  ┌──────────────┐  ┌──────────────┐                                    │
│  │ ArtifactsView│  │AttachmentsView│                                   │
│  └──────┬───────┘  └──────┬───────┘                                    │
│         │                 │                                             │
│         ▼                 ▼                                             │
│  ┌──────────────┐  ┌──────────────┐                                    │
│  │ forensic_    │  │ attachments  │  ← Different tables               │
│  │ artifacts    │  │ table        │                                    │
│  │ table        │  │              │                                    │
│  └──────────────┘  └──────────────┘                                    │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## 2. Artifacts System

### 2.1 What Are Artifacts?

Artifacts are **extracted intelligence** from email content:
- Credentials (usernames, passwords, API keys)
- Financial data (credit cards, bank accounts, routing numbers)
- Cryptocurrency (BTC/ETH addresses, seed phrases)
- PII (SSN, passport, driver's license)
- Threats (weapons, narcotics, terrorism)
- App signatures (social media, messaging, fintech)

### 2.2 How Artifacts Are Created

**Flow:**
```
Email Ingestion → Artifact Scanner → Regex Matching → Validation → DB Storage
```

**Code Path:**
1. `rescan_case_artifacts()` (`artifacts.rs:650-674`) - Entry point
2. `get_or_extract_artifacts()` (`artifacts.rs:676-748`) - Cache or extract
3. `extract_all_taxonomy_artifacts()` - Scans all emails with regex patterns
4. Each match validated (Luhn for CC, BIP-39 for seeds, etc.)
5. Stored in `forensic_artifacts` table

**Artifact Domains (12 categories):**
| Domain | Icon | Subcategories |
|--------|------|---------------|
| credentials | 🔑 | credential_pair, password_standalone, api_keys, bearer_token, jwt, ssh_key, seed_phrase, private_key |
| financial | 💳 | credit_card_spaced, credit_card_raw, routing_number, iban, swift, bank_account, sort_code |
| cryptocurrency | ₿ | btc_legacy, btc_bech32, ethereum, tron, solana, litecoin, dogecoin, monero, crypto_uri |
| pii | 🪪 | ssn, passport, driver_license, ein |
| locations | 📍 | street_address, hotel_confirmation, gps |
| threats | ⚠️ | weapons, narcotics, explosives, terrorism |
| malware | 🦠 | cve, c2 |
| corporate | 🏢 | confidential, nda |
| phishing | 🎣 | phishing_credentials, phishing_finance |
| phones | 📞 | phone_nigeria, phone_international |
| african_ids | 🇳🇬 | bvn, nin, tin |
| urls | 🔗 | url |

### 2.3 Artifacts View (UI)

**Component:** `ArtifactsView.tsx`

**Loading:**
```
User clicks "Artifacts" tab → loadTaxonomy() + loadArtifacts()
→ invoke("case_artifacts_summary") → Domain counts
→ invoke("case_artifacts_list") → Filtered artifacts
```

**Features:**
- Domain filter (sidebar with counts)
- Subcategory filter
- Artifact type filter (native/recovered/derived)
- Search (filters by primary_value, title)
- Deduplication toggle (unique vs all occurrences)
- CSV export
- Click artifact → View source email

**Deduplication:**
```typescript
// ArtifactsView.tsx:134-150
const displayedArtifacts = useMemo(() => {
  if (!dedupUnique) return artifacts;
  const map = new Map();
  for (const a of artifacts) {
    const key = `${a.domain_id}|${a.subcategory_id}|${(a.primary_value || a.title || "").trim().toLowerCase()}`;
    // ... count occurrences
  }
  return Array.from(map.values()).map(({ item, count }) => ({
    ...item,
    occurrenceCount: count,
  }));
}, [artifacts, dedupUnique]);
```

**⚠️ BUG: Deduplication is client-side only**
- Server returns ALL artifacts
- Frontend filters duplicates
- With 10k+ artifacts, this will be slow

### 2.4 Artifact Scanning Process

**Trigger:** Automatic after email ingestion OR manual "Rescan" button

**Progress:**
```
15% - Reading emails from database
15-92% - Progress bar increments every 200ms
100% - Complete
```

**Scanning Steps:**
1. Fetch all emails for case
2. For each email:
   - Combine subject + body text
   - Run all regex patterns
   - Validate matches (Luhn, BIP-39, etc.)
   - Check for duplicates (HashSet)
3. Delete old artifacts for case
4. Insert new artifacts in transaction
5. Return count

**Performance:**
- Scans ALL emails every time (no incremental)
- No progress reporting to backend
- Frontend simulates progress with setInterval

**⚠️ BUG: No incremental scanning**
- Re-scans all emails even if only one new email added
- Could be slow for large cases (100k+ emails)

---

## 3. Attachments System

### 3.1 What Are Attachments?

Attachments are **binary files** extracted from email MIME parts:
- Documents (PDF, DOCX, XLSX)
- Images (PNG, JPG, GIF)
- Archives (ZIP, RAR, 7Z)
- Executables (EXE, BAT, PS1)
- Media (MP3, MP4, MOV)

### 3.2 How Attachments Are Created

**Flow:**
```
Email Ingestion → MIME Parser → Extract Binary → Save to Disk + DB
```

**Code Path:**
1. `parse_evidence()` (`evidence.rs:202-378`) - For file import
2. `imap_fetch_emails()` (`imap.rs:52-435`) - For IMAP
3. `pop3_fetch_emails()` (`pop3.rs:212-585`) - For POP3

**Storage:**
- Database: `attachments` table (metadata only)
- Disk: `<data_dir>/cases/<case_id>/attachments/<att_id>_<filename>`

### 3.3 Attachments View (UI)

**Component:** `AttachmentsView.tsx`

**Loading:**
```
User clicks "Attachments" tab → loadSummary() + loadData()
→ invoke("case_attachments_summary") → Category counts
→ invoke("case_attachments_list") → Filtered attachments
```

**Features:**
- Category filter (all, dangerous, documents, images, archives, media)
- Search (filename, subject, from)
- Table/Grid view toggle
- Image preview (thumbnail + zoom)
- Open in system
- Reveal in Finder
- Export to Downloads
- Click attachment → View source email

**Thumbnail Loading:**
```typescript
// AttachmentsView.tsx:724-758
const AttachmentThumbnail = ({ attachmentId, storedPath, filename, category, onZoom }) => {
  const [src, setSrc] = useState<string | null>(null);
  
  useEffect(() => {
    if (isImg) {
      invoke<string | null>("get_attachment_preview", { 
        input: { attachment_id: attachmentId, stored_path: storedPath } 
      }).then((data) => {
        if (data) setSrc(data);
      });
    }
  }, [attachmentId, storedPath, isImg]);
};
```

**⚠️ BUG: Thumbnail loads ALL images immediately**
- No lazy loading
- Opens 50+ images at once → memory spike
- No pagination

---

## 4. How Artifacts and Attachments Connect

### 4.1 Shared Data

Both views share:
- Same `case_id`
- Same `evidenceFilter` (optional evidence container filter)
- Same email data (linked via `email_id`)

### 4.2 Navigation Between Views

```
Artifacts View                          Attachments View
     │                                       │
     │  Click email link                      │  Click email link
     ▼                                       ▼
┌─────────────────────────────────────────────────────────────┐
│                     Email Detail Modal                       │
│  - Shows email body                                          │
│  - Shows attachments for this email (email_attachments)      │
│  - Shows tags, notes, bookmarks                              │
└─────────────────────────────────────────────────────────────┘
```

### 4.3 Combined Pipeline

```
Email Ingestion
     │
     ├──→ Parse emails → Store in `emails` table
     │
     ├──→ Extract attachments → Store in `attachments` table + disk
     │
     └──→ Scan for artifacts → Store in `forensic_artifacts` table
```

**All three happen during ingestion, but are displayed in separate tabs.**

---

## 5. Bugs in Combined System

### BUG-1: Schema Missing `is_inline` Column (CRITICAL)
**File:** `db.rs:148-159` vs `evidence.rs:358`

```rust
// evidence.rs:358 - INSERT includes is_inline
"INSERT INTO attachments (..., is_inline, ...) VALUES (..., ?9, ...)"

// db.rs:148-159 - Schema does NOT have is_inline
CREATE TABLE attachments (
    id TEXT PRIMARY KEY,
    email_id TEXT NOT NULL,
    filename TEXT,
    sha256 TEXT NOT NULL,
    mime_type TEXT,
    size_bytes INTEGER NOT NULL,
    stored_path TEXT,
    entropy REAL,
    risk_flags TEXT DEFAULT '[]',
    created_at TEXT NOT NULL
    -- is_inline is MISSING!
);
```

**Impact:** File import with attachments will FAIL.

### BUG-2: IMAP Saves to Wrong Directory (HIGH)
**File:** `imap.rs:304-306`

```rust
let att_dir = dirs::data_dir()
    .unwrap_or_else(|| std::path::PathBuf::from("."))
    .join("j12-forensic")  // ← WRONG!
    .join("evidence")
    .join(&case_id)
    .join("attachments");
```

**Should be:**
```rust
let att_dir = Database::get_data_dir()
    .join("cases")
    .join(&case_id)
    .join("attachments");
```

**Impact:** IMAP attachments can't be opened.

### BUG-3: POP3 Attachments Not Saved to Disk (HIGH)
**File:** `pop3.rs:478-489`

POP3 attachments are inserted into DB but `stored_path` is empty.

**Impact:** POP3 attachments can't be opened or exported.

### BUG-4: No Lazy Loading for Attachments (MEDIUM)
**File:** `AttachmentsView.tsx:724-758`

All image thumbnails load immediately when tab opens.

**Impact:** Memory spike with many attachments.

### BUG-5: Artifact Scanning Not Incremental (MEDIUM)
**File:** `artifacts.rs:728`

```rust
let _ = tx.execute("DELETE FROM forensic_artifacts WHERE case_id = ?1", [case_id]);
// Then re-scan ALL emails
```

**Impact:** Slow for large cases.

### BUG-6: No Pagination (LOW)
Both views load ALL records at once.

**Impact:** Slow with 10k+ artifacts/attachments.

---

## 6. Data Flow Diagram

```
┌─────────────────────────────────────────────────────────────────────────┐
│                     EMAIL INGESTION PIPELINE                            │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  ┌─────────┐    ┌─────────┐    ┌─────────┐                            │
│  │  File   │    │  IMAP   │    │  POP3   │                            │
│  │  Import │    │  Fetch  │    │  Fetch  │                            │
│  └────┬────┘    └────┬────┘    └────┬────┘                            │
│       │              │              │                                   │
│       └──────────────┼──────────────┘                                  │
│                      ▼                                                  │
│            ┌─────────────────┐                                         │
│            │  MIME Parser    │                                         │
│            │  (parser.rs)    │                                         │
│            └────────┬────────┘                                         │
│                     │                                                   │
│        ┌────────────┼────────────┐                                     │
│        ▼            ▼            ▼                                     │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐                              │
│  │  Email   │ │Attachment│ │ Artifact │                              │
│  │  Store   │ │  Store   │ │  Scanner │                              │
│  └────┬─────┘ └────┬─────┘ └────┬─────┘                              │
│       │            │            │                                      │
│       ▼            ▼            ▼                                      │
│  ┌──────────┐ ┌──────────┐ ┌──────────────┐                          │
│  │  emails  │ │attachments│ │forensic_     │                          │
│  │  table   │ │  table    │ │artifacts     │                          │
│  │          │ │  + disk   │ │  table       │                          │
│  └──────────┘ └──────────┘ └──────────────┘                          │
│       │            │            │                                      │
│       └────────────┼────────────┘                                     │
│                    ▼                                                    │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │                     CASE WORKSPACE UI                            │   │
│  │                                                                  │   │
│  │  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐          │   │
│  │  │   Emails     │  │  Artifacts   │  │  Attachments │          │   │
│  │  │   View       │  │  View        │  │  View        │          │   │
│  │  └──────────────┘  └──────────────┘  └──────────────┘          │   │
│  │         │                  │                  │                 │   │
│  │         └──────────────────┼──────────────────┘                │   │
│  │                            ▼                                   │   │
│  │                   ┌──────────────┐                             │   │
│  │                   │    Email     │                             │   │
│  │                   │    Detail    │                             │   │
│  │                   │    Modal     │                             │   │
│  │                   └──────────────┘                             │   │
│  └─────────────────────────────────────────────────────────────────┘   │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## 7. Summary

| Bug | Severity | Component | Impact |
|-----|----------|-----------|--------|
| Schema missing `is_inline` | 🔴 CRITICAL | Attachments | File import fails |
| IMAP wrong directory | 🟠 HIGH | Attachments | IMAP attachments can't open |
| POP3 not saved to disk | 🟠 HIGH | Attachments | POP3 attachments can't open |
| No lazy loading | 🟡 MEDIUM | Attachments UI | Memory spike |
| No incremental scan | 🟡 MEDIUM | Artifacts | Slow re-scanning |
| No pagination | 🟡 MEDIUM | Both | Slow with many records |

---

*Audit completed: 2026-08-28*

