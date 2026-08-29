use std::collections::HashSet;
use rayon::prelude::*;
use serde_json::{json, Value};
use tauri::{AppHandle, Emitter, State};
use tauri::ipc::Channel;

use crate::AppState;
use super::super::attachments::classify_attachment_category;
use super::types::ForensicTaxonomyArtifact;
use super::signatures::{get_regexes, CompiledRegexes};
use super::apps::scan_apps_and_services;
use super::credentials::scan_credentials_and_crypto;
use super::financial::scan_financial_and_wallets;
use super::threats::scan_identity_threats_and_phishing;
use super::contacts::scan_phone_numbers_and_contacts;

pub fn emit_scan_progress(
    app: Option<&AppHandle>,
    on_event: Option<&Channel<Value>>,
    current: usize,
    total: usize,
    percent: u32,
    artifacts_found: usize,
    stage: &str,
) {
    let payload = json!({
        "current": current,
        "total": total,
        "percent": percent,
        "artifacts_found": artifacts_found,
        "stage": stage,
        "scanning": percent < 100
    });
    if let Some(ch) = on_event {
        let _ = ch.send(payload.clone());
    }
    if let Some(app) = app {
        let _ = app.emit("artifact_scan_progress", payload.clone());
        let _ = app.emit_to("main", "artifact_scan_progress", payload);
    }
}

type EmailRecord = (
    String,         // 0: id
    String,         // 1: from_addr
    Option<String>, // 2: from_display
    String,         // 3: to_addrs
    Option<String>, // 4: cc_addrs
    Option<String>, // 5: reply_to
    Option<String>, // 6: subject
    Option<String>, // 7: body_text
    Option<String>, // 8: body_html
    Option<String>, // 9: headers_raw
    Option<String>, // 10: date_sent_utc
    u8,             // 11: risk_score
    bool,           // 12: is_deleted
    bool,           // 13: deleted_recovered
    Option<String>, // 14: folder_category
    Option<String>, // 15: message_id
    Option<String>, // 16: in_reply_to
    Option<String>, // 17: msg_references
);

#[inline]
fn strip_base64_and_markup(raw: &str) -> String {
    if !raw.contains('<') && !raw.contains("data:image") && !raw.contains("base64") && raw.len() < 1000 {
        return raw.to_string();
    }
    
    let mut out = String::with_capacity(raw.len().min(64_000));
    let mut in_tag = false;
    let mut current_word = String::with_capacity(64);

    for ch in raw.chars() {
        if ch == '<' {
            in_tag = true;
            if !current_word.is_empty() {
                if current_word.len() <= 50 || !current_word.chars().all(|c| c.is_ascii_alphanumeric() || c == '+' || c == '/' || c == '=') {
                    out.push_str(&current_word);
                }
                current_word.clear();
            }
            continue;
        }
        if ch == '>' {
            in_tag = false;
            out.push(' ');
            continue;
        }
        if in_tag {
            continue;
        }

        if ch.is_whitespace() {
            if !current_word.is_empty() {
                if current_word.len() <= 50 || !current_word.chars().all(|c| c.is_ascii_alphanumeric() || c == '+' || c == '/' || c == '=') {
                    out.push_str(&current_word);
                }
                current_word.clear();
            }
            out.push(ch);
        } else {
            current_word.push(ch);
        }

        if out.len() >= 64_000 {
            break;
        }
    }

    if !current_word.is_empty() {
        if current_word.len() <= 50 || !current_word.chars().all(|c| c.is_ascii_alphanumeric() || c == '+' || c == '/' || c == '=') {
            out.push_str(&current_word);
        }
    }

    out
}

#[inline]
fn scan_single_email(
    email: &EmailRecord,
    re: &'static CompiledRegexes,
    artifacts: &mut Vec<ForensicTaxonomyArtifact>,
    seen: &mut HashSet<String>,
) {
    let (eid, from_addr, _from_disp, to_addrs, _cc_addrs, _reply_to, subj_opt, body_opt, html_opt, headers_raw_opt, date_opt, _risk, is_del, is_soft_del, folder_opt, msg_id_opt, _in_reply_to_opt, _ref_opt) = email;
    let from_lower = from_addr.to_lowercase();
    let subj = subj_opt.as_deref().unwrap_or("");
    let subj_lower = subj.to_lowercase();
    let body = body_opt.as_deref().unwrap_or("");
    let html = html_opt.as_deref().unwrap_or("");
    let headers_raw = headers_raw_opt.as_deref().unwrap_or("");
    let headers_lower = headers_raw.to_lowercase();
    let folder = folder_opt.as_deref().unwrap_or("inbox");
    
    let sanitized_body = strip_base64_and_markup(body);
    let full_text = if !headers_raw.is_empty() {
        format!("{} {}\n{}", subj, sanitized_body, headers_raw)
    } else {
        format!("{} {}", subj, sanitized_body)
    };
    let full_text_lower = full_text.to_lowercase();

    // 0. APPS & SERVICES SIGNATURE ENGINE
    scan_apps_and_services(
        artifacts,
        seen,
        eid,
        from_addr,
        to_addrs,
        subj_opt,
        date_opt,
        &from_lower,
        &headers_lower,
        &subj_lower,
        &full_text_lower,
        subj,
    );

    // 1. DELETED & CARVED MESSAGES
    let is_deleted = *is_del || *is_soft_del || folder == "trash" || folder == "deleted items" || folder == "soft_deleted";
    if is_deleted {
        artifacts.push(ForensicTaxonomyArtifact {
            id: crate::db::generate_id(),
            domain_id: "deleted_recovered".to_string(),
            subcategory_id: "dumpster_carved".to_string(),
            title: "Deleted / Dumpster Carved Message".to_string(),
            primary_value: if subj.is_empty() { "(No Subject)".to_string() } else { subj.to_string() },
            secondary_value: Some(from_addr.clone()),
            details: format!("Recovered from folder: {} | MsgID: {}", folder, msg_id_opt.as_deref().unwrap_or("")),
            severity: "high".to_string(),
            artifact_type: "recovered".to_string(),
            confidence: Some("high".to_string()),
            email_id: eid.clone(),
            email_subject: subj_opt.clone(),
            email_from: from_addr.clone(),
            date_sent_utc: date_opt.clone(),
        });
    }

    // 2. CALENDAR & MEETINGS (.ics)
    if headers_lower.contains("text/calendar") || full_text_lower.contains("begin:vcalendar") || subj_lower.contains("invitation:") {
        artifacts.push(ForensicTaxonomyArtifact {
            id: crate::db::generate_id(),
            domain_id: "calendar".to_string(),
            subcategory_id: "meetings_ics".to_string(),
            title: "Calendar Meeting Invitation (.ics)".to_string(),
            primary_value: if subj.is_empty() { "Calendar Event".to_string() } else { subj.to_string() },
            secondary_value: Some(from_addr.clone()),
            details: "iCalendar / Outlook meeting request object".to_string(),
            severity: "info".to_string(),
            artifact_type: "native".to_string(),
            confidence: Some("high".to_string()),
            email_id: eid.clone(),
            email_subject: subj_opt.clone(),
            email_from: from_addr.clone(),
            date_sent_utc: date_opt.clone(),
        });
    }

    // 3. CREDENTIALS & CRYPTO SECRETS
    scan_credentials_and_crypto(
        artifacts,
        seen,
        re,
        eid,
        from_addr,
        subj_opt,
        date_opt,
        &full_text,
        &full_text_lower,
    );

    // 4. FINANCIAL & CRYPTO WALLETS
    scan_financial_and_wallets(
        artifacts,
        seen,
        re,
        eid,
        from_addr,
        subj_opt,
        date_opt,
        &from_lower,
        &subj_lower,
        &full_text,
        &full_text_lower,
        subj,
    );

    // 5. IDENTITY, THREATS, MALWARE, CORPORATE PRIVILEGE, PHISHING, AUTH & TRACKING
    scan_identity_threats_and_phishing(
        artifacts,
        seen,
        re,
        eid,
        from_addr,
        subj_opt,
        date_opt,
        &headers_lower,
        &full_text,
        &full_text_lower,
        html,
    );

    // 6. PHONE NUMBERS & vCARD CONTACTS (ALL 195+ COUNTRIES)
    scan_phone_numbers_and_contacts(
        artifacts,
        seen,
        re,
        eid,
        from_addr,
        subj_opt,
        date_opt,
        &full_text,
        &full_text_lower,
    );
}

pub async fn extract_all_taxonomy_artifacts(
    app: Option<&AppHandle>,
    on_event: Option<&Channel<Value>>,
    state: &State<'_, AppState>,
    case_id: &str,
) -> Result<Vec<ForensicTaxonomyArtifact>, String> {
    emit_scan_progress(
        app,
        on_event,
        0,
        0,
        2,
        0,
        "Reading email records and attachment indices from database...",
    );

    let (emails, attachments, evidence_items) = {
        let db = state.db.lock().await;

        let mut stmt = db.conn.prepare("
            SELECT id, from_addr, from_display, to_addrs, cc_addrs, reply_to, subject, 
                   substr(body_text, 1, 65536), 
                   substr(body_html, 1, 65536), 
                   substr(headers_raw, 1, 32768), 
                   date_sent_utc, risk_score, is_deleted, deleted_recovered, folder_category, message_id, in_reply_to, msg_references
            FROM emails
            WHERE case_id = ?1
            ORDER BY date_sent_utc DESC
        ").map_err(|e| e.to_string())?;

        let emails = stmt.query_map([case_id], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, Option<String>>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, Option<String>>(4)?,
                row.get::<_, Option<String>>(5)?,
                row.get::<_, Option<String>>(6)?,
                row.get::<_, Option<String>>(7)?,
                row.get::<_, Option<String>>(8)?,
                row.get::<_, Option<String>>(9)?,
                row.get::<_, Option<String>>(10)?,
                row.get::<_, Option<i64>>(11)?.unwrap_or(0) as u8,
                row.get::<_, Option<i64>>(12)?.unwrap_or(0) != 0,
                row.get::<_, Option<i64>>(13)?.unwrap_or(0) != 0,
                row.get::<_, Option<String>>(14)?,
                row.get::<_, Option<String>>(15)?,
                row.get::<_, Option<String>>(16)?,
                row.get::<_, Option<String>>(17)?,
            ))
        }).map_err(|e| e.to_string())?.filter_map(|r| r.ok()).collect::<Vec<_>>();

        let mut att_stmt = db.conn.prepare("
            SELECT a.id, a.email_id, a.filename, a.sha256, a.mime_type, a.size_bytes, a.entropy, a.risk_flags,
                   e.subject, e.from_addr, e.date_sent_utc, a.extracted_text
            FROM attachments a
            JOIN emails e ON a.email_id = e.id
            WHERE e.case_id = ?1
        ").map_err(|e| e.to_string())?;

        let attachments = att_stmt.query_map([case_id], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, Option<String>>(2)?.unwrap_or_else(|| "attachment.bin".to_string()),
                row.get::<_, String>(3)?,
                row.get::<_, Option<String>>(4)?.unwrap_or_else(|| "application/octet-stream".to_string()),
                row.get::<_, i64>(5)? as u64,
                row.get::<_, Option<f64>>(6)?,
                row.get::<_, Option<String>>(7)?,
                row.get::<_, Option<String>>(8)?,
                row.get::<_, Option<String>>(9)?.unwrap_or_default(),
                row.get::<_, Option<String>>(10)?,
                row.get::<_, Option<String>>(11)?,
            ))
        }).map_err(|e| e.to_string())?.filter_map(|r| r.ok()).collect::<Vec<_>>();

        let mut ev_stmt = db.conn.prepare("
            SELECT id, filename, format, sha256, size_bytes, source_description, acquired_at
            FROM evidence_items
            WHERE case_id = ?1
        ").map_err(|e| e.to_string())?;

        let evidence_items = ev_stmt.query_map([case_id], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, Option<String>>(1)?.unwrap_or_else(|| "Evidence".to_string()),
                row.get::<_, Option<String>>(2)?.unwrap_or_else(|| "unknown".to_string()),
                row.get::<_, Option<String>>(3)?.unwrap_or_else(|| "unsealed".to_string()),
                row.get::<_, i64>(4)? as u64,
                row.get::<_, Option<String>>(5)?,
                row.get::<_, Option<String>>(6)?.unwrap_or_default(),
            ))
        }).map_err(|e| e.to_string())?.filter_map(|r| r.ok()).collect::<Vec<_>>();

        (emails, attachments, evidence_items)
    };

    let total_emails = emails.len();
    let mut artifacts: Vec<ForensicTaxonomyArtifact> = Vec::with_capacity(evidence_items.len() * 2 + attachments.len() + total_emails);

    // 0. Case Evidence Containers & Hashes
    for (ev_id, filename, format, sha256, size_bytes, source_desc, acquired_at) in evidence_items {
        artifacts.push(ForensicTaxonomyArtifact {
            id: format!("ev-{}", ev_id),
            domain_id: "containers".to_string(),
            subcategory_id: format.to_lowercase(),
            title: format!("Evidence Container ({})", format.to_uppercase()),
            primary_value: filename.clone(),
            secondary_value: Some(format!("SHA-256: {}", sha256)),
            details: format!("Format: {} | Size: {} B | Acquired: {} | Source: {}", format, size_bytes, acquired_at, source_desc.unwrap_or_default()),
            severity: "info".to_string(),
            artifact_type: "native".to_string(),
            confidence: Some("high".to_string()),
            email_id: String::new(),
            email_subject: Some(format!("Evidence Container: {}", filename)),
            email_from: "Case Evidence Store".to_string(),
            date_sent_utc: Some(acquired_at.clone()),
        });

        artifacts.push(ForensicTaxonomyArtifact {
            id: format!("hash-{}", ev_id),
            domain_id: "case_artifacts".to_string(),
            subcategory_id: "sha256_hash".to_string(),
            title: "Cryptographic SHA-256 Integrity Seal".to_string(),
            primary_value: sha256.clone(),
            secondary_value: Some(filename),
            details: format!("Cryptographic SHA-256 seal established at acquisition on {}", acquired_at),
            severity: "info".to_string(),
            artifact_type: "native".to_string(),
            confidence: Some("high".to_string()),
            email_id: String::new(),
            email_subject: Some("Chain of Custody Hash Seal".to_string()),
            email_from: "Forensic Acquisition Engine".to_string(),
            date_sent_utc: Some(acquired_at),
        });
    }

    let re = get_regexes();

    // Process attachments artifacts & extracted text
    for (att_id, email_id, filename, sha256, mime, size, entropy, risk_flags, subj, from_addr, date_sent, ext_text_opt) in attachments {
        let cat = classify_attachment_category(&filename, &mime, entropy, risk_flags.as_deref());
        let is_dangerous = cat == "dangerous";
        let ent_val = entropy.unwrap_or(0.0);
        let is_high_entropy = ent_val > 7.5;

        if is_dangerous || is_high_entropy || cat == "archives" {
            artifacts.push(ForensicTaxonomyArtifact {
                id: format!("att-{}", att_id),
                domain_id: "attachments".to_string(),
                subcategory_id: if is_high_entropy { "high_entropy".to_string() } else { cat.clone() },
                title: format!("Carved File: {}", filename),
                primary_value: filename.clone(),
                secondary_value: Some(format!("SHA-256: {}", sha256)),
                details: format!("MIME: {} | Size: {} B | Entropy: {:.2}{}", mime, size, ent_val, if is_high_entropy { " [HIGH ENTROPY / PACKED]" } else { "" }),
                severity: if is_dangerous || is_high_entropy { "critical".to_string() } else { "info".to_string() },
                artifact_type: "native".to_string(),
                confidence: Some("high".to_string()),
                email_id: email_id.clone(),
                email_subject: subj.clone(),
                email_from: from_addr.clone(),
                date_sent_utc: date_sent.clone(),
            });
        }

        if let Some(ref text) = ext_text_opt {
            if !text.trim().is_empty() {
                let pseudo_email: EmailRecord = (
                    email_id.clone(), format!("Doc: {}", filename), None, String::new(), None, None,
                    subj.clone(), Some(text.clone()), None, None, date_sent.clone(), 0, false, false,
                    Some("attachment".to_string()), None, None, None,
                );
                let mut att_arts = Vec::with_capacity(4);
                let mut att_seen = HashSet::new();
                scan_single_email(&pseudo_email, re, &mut att_arts, &mut att_seen);
                artifacts.extend(att_arts);
            }
        }
    }

    emit_scan_progress(
        app,
        on_event,
        0,
        total_emails,
        5,
        artifacts.len(),
        &format!("Loaded {} messages. Starting parallel multithreaded forensic scan...", total_emails),
    );

    let chunk_size = 500;
    let mut scanned_count = 0;

    for chunk in emails.chunks(chunk_size) {
        let chunk_artifacts: Vec<ForensicTaxonomyArtifact> = chunk
            .par_iter()
            .flat_map(|email| {
                let mut local_arts = Vec::with_capacity(4);
                let mut local_seen = HashSet::new();
                scan_single_email(email, re, &mut local_arts, &mut local_seen);
                local_arts
            })
            .collect();

        artifacts.extend(chunk_artifacts);
        scanned_count += chunk.len();

        let pct = if total_emails > 0 {
            5 + (((scanned_count as f64 / total_emails as f64) * 85.0) as u32)
        } else {
            90
        };

        emit_scan_progress(
            app,
            on_event,
            scanned_count,
            total_emails,
            pct,
            artifacts.len(),
            &format!("Scanning messages ({}/{}) • Found {} artifacts", scanned_count, total_emails, artifacts.len()),
        );
    }

    Ok(artifacts)
}
