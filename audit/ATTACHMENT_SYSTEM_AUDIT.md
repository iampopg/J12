# Attachment System - Complete Audit

> **Date:** 2026-08-28
> **Scope:** Full lifecycle of attachments from acquisition to display

---

## 1. How Attachments Are Acquired

### 1.1 File Import (EML/MBOX)

**Flow:**
```
User selects file → evidence_upload() → parse_evidence() → Parser extracts → Saved to disk + DB
```

**Code Path:**
1. `evidence_upload()` (`evidence.rs:15-60`) - Creates evidence record
2. `parse_evidence()` (`evidence.rs:202-378`) - Parses file, extracts emails + attachments
3. Parser (`parser.rs`) - `parse_eml()` / `parse_mbox()` returns `RawEmail` with `Vec<RawAttachment>`
4. For each attachment:
   - SHA-256 computed
   - Entropy calculated
   - File saved to: `<data_dir>/cases/<case_id>/attachments/<att_id>_<filename>`
   - `stored_path` set in database

**Storage Location:**
```
~/Library/Application Support/email-forensic/cases/<case_id>/attachments/<att_id>_<filename>
```

### 1.2 IMAP Acquisition

**Flow:**
```
IMAP fetch → parse_rfc5322() → Extract attachments → Save to disk + DB
```

**Code Path:**
1. `imap_fetch_emails()` (`imap.rs:52-435`) - Streams emails from IMAP
2. Each email parsed via `parser::parse_rfc5322()`
3. Attachments saved to: `~/Library/Application Support/j12-forensic/evidence/<case_id>/attachments/`

**⚠️ BUG: Path Inconsistency**
- File evidence uses: `Database::get_data_dir()/cases/<case_id>/attachments/`
- IMAP uses: `dirs::data_dir()/j12-forensic/evidence/<case_id>/attachments/`
- These are DIFFERENT directories!

### 1.3 POP3 Acquisition

**Flow:**
```
POP3 fetch → Parse email → Extract attachments → DB only (not saved to disk!)
```

**Code Path:**
1. `pop3_fetch_emails()` (`pop3.rs:212-585`) - Fetches emails
2. Attachments inserted into DB but `stored_path` is EMPTY

**⚠️ BUG: POP3 attachments NOT saved to disk**
```rust
// pop3.rs line 478-489 - No disk write!
let _ = db.conn.execute(
    "INSERT OR REPLACE INTO attachments (id, email_id, filename, sha256, mime_type, size_bytes, stored_path, entropy, risk_flags, created_at)
     VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10)",
    rusqlite::params![
        att_id,
        email_id,
        att.filename,
        sha256,
        att.content_type,
        att.data.len() as i64,
        stored_path,  // ← This is empty string!
        ...
    ],
);
```

---

## 2. How Attachments Are Stored

### 2.1 Database Schema

```sql
CREATE TABLE attachments (
    id TEXT PRIMARY KEY,
    email_id TEXT NOT NULL REFERENCES emails(id),
    filename TEXT,
    sha256 TEXT NOT NULL,
    mime_type TEXT,
    size_bytes INTEGER NOT NULL,
    stored_path TEXT,          -- ← NULL if not saved to disk
    entropy REAL,
    risk_flags TEXT DEFAULT '[]',
    created_at TEXT NOT NULL
);
-- NOTE: is_inline column is in evidence.rs INSERT but NOT in schema!
```

**⚠️ BUG: Schema mismatch**
- `evidence.rs:358` inserts `is_inline` column
- But `db.rs:148-159` schema doesn't have `is_inline` column
- This will cause a runtime error on file import!

### 2.2 Disk Storage

**File Import:**
```
~/Library/Application Support/email-forensic/cases/<case_id>/attachments/<att_id>_<filename>
```

**IMAP:**
```
~/Library/Application Support/j12-forensic/evidence/<case_id>/attachments/<filename>
```

**POP3:**
```
(Not saved to disk - stored_path is empty)
```

---

## 3. How Attachments Are Loaded/Displayed

### 3.1 Attachments View (Dedicated Page)

**Component:** `AttachmentsView.tsx`

**Flow:**
```
User opens Attachments tab → loadSummary() + loadData()
→ invoke("case_attachments_summary") + invoke("case_attachments_list")
→ Display in table/grid with category filters
```

**Data Source:** `case_attachments_list` command
- Queries `attachments` table JOIN `emails` table
- Returns `CaseAttachmentItem[]` with metadata
- Does NOT load actual file data (just metadata)

**Loading:** Immediate - loads all attachment metadata for the case at once.

### 3.2 Email Detail Modal

**Component:** `EmailDetailModal.tsx`

**Flow:**
```
User clicks email → Modal opens → invoke("email_attachments")
→ Display attachment list below email body
```

**Data Source:** `email_attachments` command
- Queries `attachments` table WHERE `email_id = ?`
- Returns `Attachment[]` with metadata only

**Loading:** On-demand - only when email modal opens.

### 3.3 Rich Email Body Viewer

**Component:** `RichEmailBodyViewer.tsx`

**Flow:**
```
Email body rendered → parseMimeBody() extracts inline images
→ Base64 images embedded directly in HTML
```

**Inline Image Handling:**
- Parses raw MIME body for `Content-Type: image/*` parts
- Extracts base64 data
- Creates `data:` URLs for direct embedding
- Matches `Content-ID` or `X-Attachment-Id` for CID references

---

## 4. How Attachments Are Opened/Previewed

### 4.1 Preview (get_attachment_preview)

**Flow:**
```
User clicks attachment → invoke("get_attachment_preview")
→ Try read from stored_path
→ If file exists: return base64 data URL
→ If not: carve from parent email's MIME body
```

**Code Path:** `attachments.rs:397-503`

**Two Methods:**
1. **Direct file read** - Reads from `stored_path` on disk
2. **MIME carving fallback** - Searches parent email's raw body for filename, extracts base64

**⚠️ BUG: MIME carving is fragile**
- Searches for filename in raw body
- Takes everything after `\r\n\r\n` as base64
- No validation that extracted data is actually the attachment
- Can produce corrupted files

### 4.2 Open in System (open_attachment_in_system)

**Flow:**
```
User double-clicks → invoke("open_attachment_in_system")
→ Query stored_path from DB
→ If file exists: open with OS default app
→ If not: error "Attachment file path not found on disk"
```

**Code Path:** `attachments.rs:505-564`

**⚠️ Will fail for:**
- POP3 attachments (stored_path is empty)
- IMAP attachments (wrong path - `j12-forensic` vs `email-forensic`)
- Files moved/deleted externally

### 4.3 Export (export_attachment)

**Flow:**
```
User clicks export → invoke("export_attachment")
→ Copy from stored_path to Downloads
→ If no stored_path: write text receipt file
```

**Code Path:** `attachments.rs:318-395`

**⚠️ BUG: Export creates fake file if no stored_path**
```rust
// Line 379-380 - Writes a text receipt instead of actual file
std::fs::write(&target_file, format!("Attachment Export Receipt: {}\nID: {}\nExtracted with J12 Forensic Suite.", filename, attachment_id))
```

---

## 5. Bugs Found

### BUG-1: Schema Mismatch (CRITICAL)
**File:** `db.rs:148-159` vs `evidence.rs:358`

The `attachments` table schema is missing `is_inline` column, but `parse_evidence` tries to insert it.

**Impact:** File import will fail with "table attachments has no column named is_inline"

**Fix:** Add `is_inline INTEGER DEFAULT 0` to schema.

### BUG-2: IMAP Path Inconsistency (HIGH)
**File:** `imap.rs:304-306`

IMAP saves attachments to `j12-forensic/evidence/` but file import uses `email-forensic/cases/`.

**Impact:** IMAP attachments cannot be opened from Attachments view because `stored_path` points to wrong directory.

**Fix:** Use `Database::get_data_dir()` consistently.

### BUG-3: POP3 Attachments Not Saved to Disk (HIGH)
**File:** `pop3.rs:478-489`

POP3 attachments are inserted into DB but never saved to disk.

**Impact:** Cannot open or export POP3 attachments.

**Fix:** Add disk write logic like file import has.

### BUG-4: MIME Carving Fallback Fragile (MEDIUM)
**File:** `attachments.rs:456-500`

The fallback method to carve attachments from raw MIME body is unreliable.

**Impact:** May produce corrupted preview data for attachments not on disk.

**Fix:** Store attachment binary data in database as BLOB, or ensure all attachments are saved to disk.

### BUG-5: Export Creates Fake File (LOW)
**File:** `attachments.rs:379-380`

When no `stored_path`, export writes a text receipt instead of actual file.

**Impact:** User gets a text file instead of the actual attachment.

**Fix:** Return error or use MIME carving as fallback.

---

## 6. Data Flow Diagram

```
┌─────────────────────────────────────────────────────────────────────────┐
│                         ATTACHMENT LIFECYCLE                            │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  ACQUISITION                                                            │
│  ┌──────────┐   ┌──────────┐   ┌──────────┐                           │
│  │ File     │   │ IMAP     │   │ POP3     │                           │
│  │ Import   │   │ Fetch    │   │ Fetch    │                           │
│  └────┬─────┘   └────┬─────┘   └────┬─────┘                           │
│       │              │              │                                   │
│       ▼              ▼              ▼                                   │
│  ┌─────────────────────────────────────────┐                           │
│  │         Parser (parser.rs)              │                           │
│  │  - Extracts RawAttachment from MIME     │                           │
│  │  - Sets is_inline flag                  │                           │
│  └────────────────┬────────────────────────┘                           │
│                   │                                                     │
│       ┌───────────┼───────────┐                                        │
│       ▼           ▼           ▼                                        │
│  ┌─────────┐ ┌─────────┐ ┌─────────┐                                  │
│  │ Save to │ │ Save to │ │ DB ONLY │  ← BUG: POP3 not saved to disk   │
│  │ disk    │ │ disk    │ │ (no     │                                  │
│  │ (cases) │ │ (evid)  │ │  disk)  │  ← BUG: Different path           │
│  └────┬────┘ └────┬────┘ └────┬────┘                                  │
│       │           │           │                                        │
│       ▼           ▼           ▼                                        │
│  ┌─────────────────────────────────────────┐                           │
│  │         SQLite Database                  │                           │
│  │  - attachments table                     │                           │
│  │  - stored_path, sha256, entropy          │                           │
│  └────────────────┬────────────────────────┘                           │
│                   │                                                     │
│  DISPLAY          │                                                     │
│  ┌────────────────┼────────────────┐                                   │
│  ▼                ▼                ▼                                   │
│  ┌──────────┐ ┌──────────┐ ┌──────────────┐                           │
│  │ Attach-  │ │ Email    │ │ RichEmail    │                           │
│  │ ments    │ │ Detail   │ │ BodyViewer   │                           │
│  │ View     │ │ Modal    │ │ (inline img) │                           │
│  └──────────┘ └──────────┘ └──────────────┘                           │
│       │              │                │                                 │
│       └──────────────┼────────────────┘                                │
│                      ▼                                                  │
│  ┌─────────────────────────────────────────┐                           │
│  │         Opening/Preview                  │                           │
│  │  1. Read from stored_path                │                           │
│  │  2. Fallback: MIME carving              │
│  │  3. Fallback: Error                     │                           │
│  └─────────────────────────────────────────┘                           │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## 7. Summary of Issues

| Bug | Severity | Impact | Fix |
|-----|----------|--------|-----|
| Schema missing `is_inline` | CRITICAL | File import fails | Add column to schema |
| IMAP path wrong | HIGH | IMAP attachments can't open | Use `Database::get_data_dir()` |
| POP3 not saved to disk | HIGH | POP3 attachments can't open | Add disk write logic |
| MIME carving fragile | MEDIUM | Corrupted previews | Store BLOB in DB or ensure disk |
| Export creates fake file | LOW | Wrong file exported | Return error or carve |

---

*Audit completed: 2026-08-28*

