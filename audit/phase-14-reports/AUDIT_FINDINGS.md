# Phase 14 Audit: Report Generation

> **Files Audited:**
> - `src-tauri/src/commands/reports.rs` (not fully read, based on SYSTEM_AUDIT.md)

---

## Findings

### ISSUE-111: PDF export may not work
- **Category:** NO BACKEND
- **File:** `src-tauri/src/commands/reports.rs`
- **What's wrong:** No PDF generation library in Cargo.toml dependencies. `export_report_pdf` likely returns HTML or fails.
- **Impact:** Cannot export PDF reports.
- **Fix:** Add PDF generation library (e.g., printpdf, wkhtmltopdf).

---

### ISSUE-112: Report data doesn't include attachments manifest
- **Category:** BREAK
- **File:** `src-tauri/src/commands/reports.rs`
- **What's wrong:** Report generation may not include attachment hashes or metadata.
- **Impact:** Incomplete reports.
- **Fix:** Add attachment manifest to report data.

---

### ISSUE-113: No report template customization
- **Category:** NO UI
- **File:** `src/pages/CaseWorkspace.tsx` (report view)
- **What's wrong:** No UI for customizing report template, adding logo, or changing sections.
- **Impact:** Reports are generic.
- **Fix:** Add report template editor.

---

## Summary

| Category | Count |
|----------|-------|
| BREAK | 1 |
| NO BACKEND | 1 |
| NO UI | 1 |
| **Total** | **3** |

