# Phase 1 Audit: Authentication & Session

> **Files Audited:**
> - `src/auth.tsx` (156 lines)
> - `src/pages/LoginPage.tsx` (293 lines)
> - `src-tauri/src/main.rs` (112 lines)
> - `src-tauri/src/db.rs` (schema)

---

## Findings

### ISSUE-001: Passwords stored in plaintext
- **Category:** ERROR-PRONE
- **File:** `src/auth.tsx:32`, `src/auth.tsx:121`
- **What's wrong:** The default admin password is stored as `"admin123"` and compared directly. New user passwords are stored in plaintext in `passwordHash` field name but it's not actually hashed.
- **Impact:** Anyone with access to localStorage can read all passwords. This is a forensic tool that handles evidence - plaintext passwords are unacceptable.
- **Fix:** Use a proper password hashing library (bcrypt, argon2) before storing. Never store plaintext.

---

### ISSUE-002: No session expiration
- **Category:** ERROR-PRONE
- **File:** `src/auth.tsx:45-51`, `src/auth.tsx:85`
- **What's wrong:** Once logged in, the user stays logged in forever via `localStorage.getItem("j12_current_user")`. No expiry, no token refresh, no idle timeout.
- **Impact:** If an examiner walks away from their workstation, anyone can access the forensic evidence. No forensic integrity.
- **Fix:** Add session timeout (e.g., 30 minutes idle), store session token with expiry, re-authenticate for sensitive operations.

---

### ISSUE-003: No backend authentication at all
- **Category:** BREAK
- **File:** `src-tauri/src/main.rs` (entire invoke_handler)
- **What's wrong:** Authentication is 100% frontend-side in localStorage. There is NO backend auth check on ANY command. The Rust backend has no `users` table, no `auth` module, no middleware.
- **Impact:** Any command can be invoked without authentication. Evidence can be accessed/modified/deleted without login. Complete security bypass.
- **Fix:** Add backend authentication middleware. Verify session/token on every command. Add `users` table to database.

---

### ISSUE-004: Default credentials hardcoded and visible
- **Category:** HARDCODED
- **File:** `src/pages/LoginPage.tsx:11-12`, `src/pages/LoginPage.tsx:164`
- **What's wrong:** Login form pre-filled with `admin`/`admin123`. The footer shows "Default Local Admin: admin / admin123" in plain sight.
- **Impact:** Anyone who opens the app knows the default credentials. For a forensic tool, this is a critical security flaw.
- **Fix:** Remove pre-filled credentials. Force password change on first login. Remove credential hint from UI.

---

### ISSUE-005: No brute force protection
- **Category:** ERROR-PRONE
- **File:** `src/auth.tsx:66-90`
- **What's wrong:** Login function has no rate limiting, no account lockout, no CAPTCHA. An attacker can try unlimited passwords.
- **Impact:** Passwords can be brute-forced. Even a 4-character minimum is trivially crackable.
- **Fix:** Add rate limiting (max 5 attempts per minute), account lockout after 10 failures, exponential backoff.

---

### ISSUE-006: No password strength requirements
- **Category:** ERROR-PRONE
- **File:** `src/auth.tsx:105-107`
- **What's wrong:** Password minimum is 4 characters. No complexity requirement (uppercase, lowercase, number, special char).
- **Impact:** Users can set password as "1234" or "aaaa". Trivial to guess.
- **Fix:** Minimum 8 characters, require 3 of 4 character classes (upper, lower, digit, special).

---

### ISSUE-007: Agency field hardcoded on registration
- **Category:** HARDCODED
- **File:** `src/pages/LoginPage.tsx:69`
- **What's wrong:** Registration form passes `agency: "Digital Forensics Unit"` hardcoded. The user cannot choose their agency.
- **Impact:** All registered users show same agency. Not dynamic.
- **Fix:** Make agency a user-editable field in the registration form.

---

### ISSUE-008: All new users get role "examiner"
- **Category:** HARDCODED
- **File:** `src/auth.tsx:120`
- **What's wrong:** Role is hardcoded to `"examiner"` for all new registrations. No admin can be created through UI.
- **Impact:** Role-based access control is meaningless when everyone has the same role.
- **Fix:** Implement role hierarchy. First user = admin. Admin can assign roles.

---

### ISSUE-009: No logout confirmation
- **Category:** ERROR-PRONE
- **File:** `src/auth.tsx:142-145`
- **What's wrong:** Logout immediately clears session with no confirmation dialog. No "Are you sure?" prompt.
- **Impact:** Accidental click logs out examiner. No forensic session continuity.
- **Fix:** Add confirmation dialog before logout.

---

### ISSUE-010: No audit logging for auth events
- **Category:** NO BACKEND
- **File:** `src/auth.tsx` (login, register, logout functions)
- **What's wrong:** Login, registration, and logout events are never written to the `audit_log` table. No record of who accessed the system.
- **Impact:** No chain of custody for authentication events. Cannot track who logged in or when.
- **Fix:** Call `audit_logger` on every auth event (login success, login failure, logout, registration).

---

## Reconfirmation

I re-read `auth.tsx` and `LoginPage.tsx` in full. Findings confirmed:
- Passwords are plaintext (line 32, 121)
- Session stored in localStorage with no expiry (line 45-51)
- Default credentials pre-filled (line 11-12)
- No backend auth module exists in Rust code

Cross-referenced with `main.rs`:
- No `mod ai;` declaration
- No auth commands registered
- No middleware for auth checking

Cross-referenced with `db.rs`:
- No `users` table in schema
- 19 CREATE TABLE statements, none for users

**All 10 findings confirmed.**

---

## Summary

| Category | Count |
|----------|-------|
| BREAK | 1 |
| HARDCODED | 3 |
| ERROR-PRONE | 5 |
| NO BACKEND | 1 |
| **Total** | **10** |

**Severity:** CRITICAL - Authentication is completely frontend-only with no backend enforcement. This is a forensic tool that must maintain chain of custody - anyone can access all evidence without login by invoking commands directly.

