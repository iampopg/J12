◊# Artifact Scanning Performance Analysis

> **Date:** 2026-08-28
> **Problem:** User reports artifact scanning takes hours with no feedback

---

## Why It's Slow

### The Scanning Process

```
┌─────────────────────────────────────────────────────────────────────────┐
│                     ARTIFACT SCANNING PIPELINE                          │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  1. Load ALL emails from DB (14,000 emails)                            │
│     └── Each email: subject + body + headers (could be MB each)        │
│                                                                         │
│  2. Load ALL attachments from DB                                        │
│                                                                         │
│  3. Load ALL evidence items from DB                                     │
│                                                                         │
│  4. For EACH email (14,000 times):                                     │
│     ├── Check APP_SIGNATURES (80+ signatures)                           │
│     ├── Check DELETED status                                            │
│     ├── Check CALENDAR                                                 │
│     ├── If contains "password": run cred_pair regex                    │
│     ├── If contains "password": run pass_standalone regex              │
│     ├── If contains "AKIA": run api_keys regex                         │
│     ├── If contains "Bearer": run bearer regex                         │
│     ├── If contains "eyJ": run jwt regex                               │
│     ├── If contains "private": run ssh_key regex                       │
│     ├── If contains "private": run privkey regex                       │
│     ├── If contains credit card patterns: run cc regex                 │
│     ├── If contains routing: run routing regex                         │
│     ├── If contains IBAN: run iban regex                               │
│     ├── If contains SWIFT: run swift regex                             │
│     ├── If contains account: run account regex                         │
│     ├── If contains sort: run sort_code regex                          │
│     ├── If contains BTC: run btc_legacy regex                          │
│     ├── If contains BTC: run btc_bech32 regex                          │
│     ├── If contains ETH: run eth regex                                 │
│     ├── If contains TRON: run tron regex                               │
│     ├── If contains SOL: run sol regex                                 │
│     ├── If contains LTC: run ltc regex                                 │
│     ├── If contains DOGE: run doge regex                               │
│     ├── If contains XMR: run xmr regex                                 │
│     ├── If contains crypto: run crypto_uri regex                       │
│     ├── If contains SSN: run ssn regex                                 │
│     ├── If contains passport: run passport regex                       │
│     ├── If contains driver: run driver_lic regex                       │
│     ├── If contains EIN: run ein regex                                 │
│     ├── If contains address: run street_addr regex                     │
│     ├── If contains hotel: run hotel_conf regex                        │
│     ├── If contains GPS: run gps regex                                 │
│     ├── If contains weapons: run weapons regex                         │
│     ├── If contains narcotics: run narcotics regex                     │
│     ├── If contains explosives: run explosives regex                   │
│     ├── If contains terrorism: run terrorism regex                     │
│     ├── If contains CVE: run cve regex                                 │
│     ├── If contains C2: run c2 regex                                   │
│     ├── If contains confidential: run confidential regex               │
│     ├── If contains NDA: run nda regex                                 │
│     ├── If contains phishing: run phish_cred regex                     │
│     ├── If contains phishing: run phish_finance regex                  │
│     ├── If contains phone: run phone regex                             │
│     ├── If contains BVN: run bvn regex                                 │
│     ├── If contains NIN: run nin regex                                 │
│     ├── If contains TIN: run tin regex                                 │
│     └── If contains URL: run url regex                                 │
│                                                                         │
│  5. Delete ALL old artifacts for case                                  │
│                                                                         │
│  6. Insert ALL new artifacts in transaction                            │
│                                                                         │
│  7. Return count to frontend                                           │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## Performance Breakdown

### Step 1: Loading Data

```rust
// artifacts.rs:757-786 - Loads ALL emails
let emails = stmt.query_map([case_id], |row| {
    Ok((
        row.get::<_, String>(0)?,   // id
        row.get::<_, String>(1)?,   // from_addr
        // ... 17 fields per email
    ))
}).collect::<Vec<_>>();

// For 14,000 emails:
// - Each email could be 1KB-100KB (body + headers)
// - Total: 14MB - 1.4GB loaded into RAM
```

**Time estimate:**
- 14,000 small emails: ~2 seconds
- 14,000 large emails: ~10-30 seconds

### Step 2: Scanning with Regex

```rust
// For each of 14,000 emails:
// - Up to 40+ regex patterns checked
// - Each regex runs against 64KB text sample

// Best case: Early substring filter skips most regex
// Worst case: Email contains "password" → runs ALL credential regexes
```

**Time estimate:**
- Best case (clean emails): ~5 seconds
- Average case: ~30-60 seconds
- Worst case (spam with many keywords): **5-15 minutes**

### Step 3: Database Write

```rust
// artifacts.rs:728-745
let _ = tx.execute("DELETE FROM forensic_artifacts WHERE case_id = ?1", [case_id]);
// ... insert each artifact in transaction
tx.commit()
```

**Time estimate:**
- DELETE: ~1 second
- INSERT 10,000 artifacts: ~5-10 seconds
- COMMIT: ~1 second

### Step 4: Frontend Deduplication

```typescript
// ArtifactsView.tsx:134-150
const displayedArtifacts = useMemo(() => {
  // Runs after ALL artifacts loaded
  // Creates Map with all artifacts
  // Counts occurrences
}, [artifacts, dedupUnique]);
```

**Time estimate:**
- 10,000 artifacts: ~100ms
- 100,000 artifacts: ~1-2 seconds

---

## Total Time Estimates

| Scenario | Emails | Time | Why |
|----------|--------|------|-----|
| Small case | 100 | 5-10 sec | Quick load, few regex matches |
| Medium case | 1,000 | 30-60 sec | More emails, more matches |
| Large case | 10,000 | 2-5 min | Many emails, many matches |
| Your case | 14,000 | 3-10 min | Large emails + many matches |
| Worst case | 100,000+ | 30+ min | All emails contain keywords |

---

## The FAKE Progress Bar

**The progress bar is FAKE.** It doesn't reflect actual scanning progress:

```typescript
// ArtifactsView.tsx:160-165
const progressInterval = setInterval(() => {
  currentProgress = Math.min(currentProgress + 10, 92);
  scanStore.setState({ progress: currentProgress });
}, 200);  // ← Just increments every 200ms regardless of actual progress
```

**What happens:**
1. Progress bar goes 15% → 92% over ~4 seconds
2. Then STALLS at 92% while actual scanning happens
3. User thinks it's stuck at 92%
4. Scanning could take 5 MORE MINUTES with no feedback
5. Finally jumps to 100% when done

---

## Root Causes of Slowness

### 1. No Incremental Scanning
```rust
// artifacts.rs:728 - Deletes ALL artifacts then re-scans ALL emails
let _ = tx.execute("DELETE FROM forensic_artifacts WHERE case_id = ?1", [case_id]);
```

**Problem:** If you add 1 email, it re-scans ALL 14,000 emails.

**Fix:** Track last scan timestamp, only scan new/modified emails.

### 2. Fake Progress Bar
```typescript
// Progress bar is just setInterval - not connected to actual scanning
```

**Fix:** Send real progress events from backend during scanning.

### 3. All Data in Memory
```rust
// artifacts.rs:757-831 - Loads everything into Vec
let emails = stmt.query_map(...).collect::<Vec<_>>();
```

**Fix:** Stream emails in chunks.

### 4. No Parallelism
Scanning is single-threaded.

**Fix:** Use rayon or async to scan emails in parallel.

### 5. Client-Side Deduplication
All artifacts sent to frontend, then deduplicated.

**Fix:** Deduplicate in SQL or backend.

---

## What You See vs What's Happening

```
What You See:                    What's Actually Happening:
                                 
[15%] Reading emails...          Loading 14,000 emails from DB
[25%] ...                        (2-5 seconds)
[35%] ...                        
[45%] ...                        Done loading, starting regex scan
[55%] ...                        Scanning email 1/14,000...
[65%] ...                        Scanning email 500/14,000...
[75%] ...                        Scanning email 2000/14,000...
[85%] ...                        Scanning email 5000/14,000...
[92%] ...                        Scanning email 10000/14,000...
[92%] ... (stalled)              Scanning email 12000/14,000...
[92%] ... (stalled)              Scanning email 13500/14,000...
[92%] ... (stalled)              Done scanning, inserting to DB...
[100%] Complete!                  Done!
```

---

## Recommended Fixes (Priority Order)

### 1. Real Progress Events (HIGH)
Send actual progress from backend:
```rust
// In extract_all_taxonomy_artifacts:
for (i, email) in emails.iter().enumerate() {
    if i % 100 == 0 {
        let progress = (i * 100) / emails.len();
        app.emit("artifact_scan_progress", progress)?;
    }
    // ... scan email
}
```

### 2. Incremental Scanning (HIGH)
Only scan new emails:
```sql
-- Add last_scanned_at column to emails
SELECT * FROM emails 
WHERE case_id = ?1 
  AND (last_scanned_at IS NULL OR updated_at > last_scanned_at)
```

### 3. Streaming/Chunking (MEDIUM)
Process emails in batches:
```rust
for chunk in emails.chunks(1000) {
    // Process 1000 emails at a time
    // Emit progress
}
```

### 4. Parallel Scanning (LOW)
Use rayon for parallel iteration:
```rust
use rayon::prelude::*;
emails.par_iter().for_each(|email| {
    // Scan email (thread-safe)
});
```

---

## Summary

| Issue | Impact | Fix Priority |
|-------|--------|--------------|
| Fake progress bar | User thinks it's stuck | 🔴 HIGH |
| No incremental scan | Re-scans everything | 🔴 HIGH |
| All data in memory | RAM spike | 🟡 MEDIUM |
| Single-threaded | Slow on multi-core | 🟢 LOW |
| Client-side dedup | Slow frontend | 🟢 LOW |

---

*Audit completed: 2026-08-28*

