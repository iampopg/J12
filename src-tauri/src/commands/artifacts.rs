use std::collections::{BTreeMap, HashSet};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri::State;

use crate::AppState;
use crate::db::generate_id;
use super::attachments::classify_attachment_category;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TaxonomySubcategorySummary {
    pub subcategory_id: String,
    pub name: String,
    pub count: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TaxonomyDomainSummary {
    pub domain_id: String,
    pub name: String,
    pub icon: String,
    pub total_count: usize,
    pub subcategories: Vec<TaxonomySubcategorySummary>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ForensicTaxonomyArtifact {
    pub id: String,
    pub domain_id: String,
    pub subcategory_id: String,
    pub title: String,
    pub primary_value: String,
    pub secondary_value: Option<String>,
    pub details: String,
    pub severity: String,
    pub artifact_type: String,
    pub confidence: Option<String>,
    pub email_id: String,
    pub email_subject: Option<String>,
    pub email_from: String,
    pub date_sent_utc: Option<String>,
}

// ─────────────────────────────────────────────────────────────────────────────
// HIGH-PRECISION FALSE POSITIVE (FP) REDUCTION VALIDATORS (>80% TRUE POSITIVES)
// ─────────────────────────────────────────────────────────────────────────────

/// Luhn algorithm for validating credit card numbers
pub fn luhn_check(num_str: &str) -> bool {
    let digits: Vec<u32> = num_str.chars().filter_map(|c| c.to_digit(10)).collect();
    if digits.len() < 13 || digits.len() > 19 {
        return false;
    }
    // Reject repeated digits like 0000-0000-0000-0000 or 1111-1111-1111-1111
    if digits.iter().all(|&d| d == digits[0]) {
        return false;
    }
    let mut sum = 0;
    let mut double = false;
    for &d in digits.iter().rev() {
        let val = if double {
            let doubled = d * 2;
            if doubled > 9 { doubled - 9 } else { doubled }
        } else {
            d
        };
        sum += val;
        double = !double;
    }
    sum % 10 == 0
}

/// US 9-Digit ABA Bank Routing Checksum Validator
pub fn validate_routing_number(num_str: &str) -> bool {
    let digits: Vec<u32> = num_str.chars().filter_map(|c| c.to_digit(10)).collect();
    if digits.len() != 9 || digits.iter().all(|&d| d == digits[0]) {
        return false;
    }
    let sum = 3 * (digits[0] + digits[3] + digits[6])
            + 7 * (digits[1] + digits[4] + digits[7])
            + 1 * (digits[2] + digits[5] + digits[8]);
    sum % 10 == 0
}

/// US Social Security Number (SSN) Structure Validator
pub fn validate_ssn(ssn_str: &str) -> bool {
    let clean: String = ssn_str.chars().filter(|c| c.is_ascii_digit()).collect();
    if clean.len() != 9 || clean.chars().all(|c| c == clean.chars().next().unwrap()) {
        return false;
    }
    let area: u32 = clean[0..3].parse().unwrap_or(0);
    let group: u32 = clean[3..5].parse().unwrap_or(0);
    let serial: u32 = clean[5..9].parse().unwrap_or(0);
    if area == 0 || area == 666 || area >= 900 || group == 0 || serial == 0 {
        return false;
    }
    if clean == "123456789" || clean == "987654321" {
        return false;
    }
    true
}

/// Base58 Bitcoin Address Character Validator
pub fn validate_btc_base58(addr: &str) -> bool {
    if addr.len() < 26 || addr.len() > 35 { return false; }
    let forbidden = ['0', 'O', 'I', 'l'];
    !addr.chars().any(|c| forbidden.contains(&c)) && (addr.starts_with('1') || addr.starts_with('3'))
}

/// Phone Number Sanitizer & Quality Check
pub fn validate_phone(p: &str) -> bool {
    let digits: Vec<char> = p.chars().filter(|c| c.is_ascii_digit()).collect();
    if digits.len() < 10 || digits.len() > 15 { return false; }
    if digits.iter().all(|&c| c == digits[0]) { return false; }
    // Exclude date patterns like 2024-05-12 or 2023-11-04
    if (p.starts_with("19") || p.starts_with("20")) && p.contains('-') && digits.len() <= 8 {
        return false;
    }
    true
}

/// Case Artifacts Summary by Taxonomy Domains (Hides 0-count domains by default)
#[tauri::command]
pub async fn case_artifacts_summary(
    state: State<'_, AppState>,
    input: Value,
) -> Result<Vec<TaxonomyDomainSummary>, String> {
    let case_id = input["case_id"].as_str()
        .or_else(|| input["caseId"].as_str())
        .or_else(|| input.as_str())
        .unwrap_or("")
        .to_string();

    let show_all = input["show_all"].as_bool().unwrap_or(false);
    let all_artifacts = extract_all_taxonomy_artifacts(&state, &case_id).await?;

    let domain_defs = [
        ("credentials", "Credentials & Secrets", "🔑"),
        ("financial", "Financial & Banking", "🏦"),
        ("crypto", "Cryptocurrency & Seeds", "🪙"),
        ("identity_docs", "PII & Identity Documents", "🪪"),
        ("locations", "Locations, Travel & Addresses", "📍"),
        ("contraband", "Threats & Contraband", "🛑"),
        ("malware_threats", "Malware & Cyber Threats", "🦠"),
        ("secrets", "Corporate & Legal Privileged", "📄"),
        ("phishing", "Phishing & Social Engineering", "🎣"),
        ("network", "Suspicious Network & URL Hooks", "🌐"),
        ("messaging_apps", "Messaging App Relays", "💬"),
        ("attachments", "Carved & Suspicious Files", "📎"),
        ("deleted_recovered", "Deleted & Carved Messages", "🗑️"),
        ("authentication", "Failed Authentication & Spoofing", "🔐"),
        ("calendar", "Calendar & Meetings (.ics)", "📅"),
        ("client", "Email Clients & Devices", "💻"),
        ("containers", "Mailboxes & Containers", "🗂️"),
        ("case_artifacts", "Evidence Integrity Seals", "⚖️"),
    ];

    let mut result = Vec::new();

    for (dom_id, dom_name, dom_icon) in &domain_defs {
        let domain_artifacts: Vec<&ForensicTaxonomyArtifact> = all_artifacts.iter().filter(|a| a.domain_id == *dom_id).collect();
        let total_count = domain_artifacts.len();

        if total_count == 0 && !show_all {
            continue;
        }

        let mut sub_map: BTreeMap<String, usize> = BTreeMap::new();
        for a in &domain_artifacts {
            *sub_map.entry(a.subcategory_id.clone()).or_insert(0) += 1;
        }

        let subcategories = sub_map.into_iter().filter(|(_, cnt)| *cnt > 0).map(|(k, v)| {
            let name = k.replace('_', " ").to_uppercase();
            TaxonomySubcategorySummary {
                subcategory_id: k,
                name,
                count: v,
            }
        }).collect();

        result.push(TaxonomyDomainSummary {
            domain_id: dom_id.to_string(),
            name: dom_name.to_string(),
            icon: dom_icon.to_string(),
            total_count,
            subcategories,
        });
    }

    Ok(result)
}

/// Case Artifacts List filtered by domain, subcategory, search, or severity
#[tauri::command]
pub async fn case_artifacts_list(
    state: State<'_, AppState>,
    input: Value,
) -> Result<Vec<ForensicTaxonomyArtifact>, String> {
    let case_id = input["case_id"].as_str()
        .or_else(|| input["caseId"].as_str())
        .or_else(|| input.as_str())
        .unwrap_or("")
        .to_string();

    let domain = input["domain"].as_str()
        .or_else(|| input["category"].as_str())
        .unwrap_or("all");
    let subcategory = input["subcategory"].as_str().unwrap_or("all");
    let search = input["search"].as_str().unwrap_or("").to_lowercase();
    let artifact_type = input["artifact_type"].as_str().unwrap_or("all");

    let all_artifacts = extract_all_taxonomy_artifacts(&state, &case_id).await?;

    let filtered = all_artifacts.into_iter().filter(|item| {
        if domain != "all" && item.domain_id != domain {
            return false;
        }
        if subcategory != "all" && item.subcategory_id != subcategory {
            return false;
        }
        if artifact_type != "all" && item.artifact_type != artifact_type {
            return false;
        }
        if !search.is_empty() {
            let val_m = item.primary_value.to_lowercase().contains(&search);
            let title_m = item.title.to_lowercase().contains(&search);
            let det_m = item.details.to_lowercase().contains(&search);
            let subj_m = item.email_subject.as_deref().unwrap_or("").to_lowercase().contains(&search);
            let from_m = item.email_from.to_lowercase().contains(&search);
            if !val_m && !title_m && !det_m && !subj_m && !from_m {
                return false;
            }
        }
        true
    }).collect();

    Ok(filtered)
}

async fn extract_all_taxonomy_artifacts(
    state: &State<'_, AppState>,
    case_id: &str,
) -> Result<Vec<ForensicTaxonomyArtifact>, String> {
    let (emails, attachments, evidence_items) = {
        let db = state.db.lock().await;

        let mut stmt = db.conn.prepare("
            SELECT id, from_addr, from_display, to_addrs, cc_addrs, reply_to, subject, body_text, body_html, headers_raw, 
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

        // Fetch case attachments
        let mut att_stmt = db.conn.prepare("
            SELECT a.id, a.email_id, a.filename, a.sha256, a.mime_type, a.size_bytes, a.entropy, a.risk_flags,
                   e.subject, e.from_addr, e.date_sent_utc
            FROM attachments a
            JOIN emails e ON a.email_id = e.id
            WHERE e.case_id = ?1
        ").map_err(|e| e.to_string())?;

        let attachments = att_stmt.query_map([case_id], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, i64>(5)? as u64,
                row.get::<_, Option<f64>>(6)?,
                row.get::<_, Option<String>>(7)?,
                row.get::<_, Option<String>>(8)?,
                row.get::<_, String>(9)?,
                row.get::<_, Option<String>>(10)?,
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
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, i64>(4)? as u64,
                row.get::<_, Option<String>>(5)?,
                row.get::<_, String>(6)?,
            ))
        }).map_err(|e| e.to_string())?.filter_map(|r| r.ok()).collect::<Vec<_>>();

        (emails, attachments, evidence_items)
    };

    let mut artifacts: Vec<ForensicTaxonomyArtifact> = Vec::new();

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
            details: format!("Cryptographic SHA-256 evidence integrity seal established at acquisition on {}", acquired_at),
            severity: "info".to_string(),
            artifact_type: "native".to_string(),
            confidence: Some("high".to_string()),
            email_id: String::new(),
            email_subject: Some("Chain of Custody Hash Seal".to_string()),
            email_from: "Forensic Acquisition Engine".to_string(),
            date_sent_utc: Some(acquired_at),
        });
    }

    // ─────────────────────────────────────────────────────────────────────────────
    // STRICT FORENSIC REGEX MATCHERS (Zero False-Alarm Target)
    // ─────────────────────────────────────────────────────────────────────────────
    
    // 1. Credentials & Secrets
    let re_cred_pair = regex::Regex::new(r"(?i)(?:username|user|login|email)[:=\s]+([a-zA-Z0-9._%+\-@]{3,50})\s*(?:password|pwd|pass)[:=\s]+([^\s,;]{4,50})").ok();
    let re_pass_standalone = regex::Regex::new(r"(?i)(?:password|passwd|passcode|secret\s*key)[:=\s]+([^\s,;]{6,60})").ok();
    let re_api_keys = regex::Regex::new(r"\b(AKIA[0-9A-Z]{16}|sk_live_[0-9a-zA-Z]{24,40}|ghp_[0-9a-zA-Z]{36}|AIza[0-9A-Za-z\-_]{35})\b").ok();
    let re_bearer = regex::Regex::new(r"Bearer\s+([A-Za-z0-9\-\._~\+\/]{25,}=*)").ok();
    let re_jwt = regex::Regex::new(r"(eyJ[A-Za-z0-9-_=]{15,}\.[A-Za-z0-9-_=]{15,}\.[A-Za-z0-9-_.+/=]{10,})").ok();
    let re_ssh_key = regex::Regex::new(r"-----BEGIN (?:RSA|DSA|EC|OPENSSH) PRIVATE KEY-----").ok();
    // BIP-39 Seed Phrase: MUST be preceded by seed phrase indicator AND contain at least 12 lowercase words
    let re_seed = regex::Regex::new(r"(?i)(?:seed\s*phrase|recovery\s*phrase|mnemonic\s*phrase|wallet\s*seed)[:=\-]?\s*([a-z]{3,10}(?:\s+[a-z]{3,10}){11,23})").ok();
    let re_privkey = regex::Regex::new(r"(?i)(?:private\s*key|privkey)[:=\s]+([0-9a-fA-F]{64})\b").ok();

    // 2. Financial & Banking (Strict Contextual Prefixes)
    let re_cc_spaced = regex::Regex::new(r"\b((?:4[0-9]{3}|5[1-5][0-9]{2}|6011|3[47][0-9]{2})[\s\-][0-9]{4}[\s\-][0-9]{4}[\s\-][0-9]{4})\b").ok();
    let re_cc_raw = regex::Regex::new(r"\b(4[0-9]{12}(?:[0-9]{3})?|5[1-5][0-9]{14}|3[47][0-9]{13}|6011[0-9]{12})\b").ok();
    let re_routing = regex::Regex::new(r"(?i)(?:routing(?:\s*number|#)?|aba(?:\s*#|\s*no)?)\s*[:#=]?\s*(\b(?:0[1-9]|[123][0-9]|6[1-9]|7[0-2]|80)\d{7}\b)").ok();
    let re_iban = regex::Regex::new(r"(?i)(?:iban)\s*[:#=]?\s*([A-Z]{2}[0-9]{2}[A-Z0-9]{4}[0-9]{7}(?:[A-Z0-9]?){0,16})\b").ok();
    let re_swift = regex::Regex::new(r"(?i)(?:swift(?:\s*code|\s*bic)?|bic(?:\s*code)?|swift/bic)\s*[:#=]?\s*([A-Z]{6}[A-Z0-9]{2}(?:[A-Z0-9]{3})?)\b").ok();
    let re_account = regex::Regex::new(r"(?i)(?:bank\s*account(?:\s*number|\s*#)?|acct(?:\s*number|\s*#))\s*[:#=]?\s*([0-9]{8,17})\b").ok();
    let re_sort_code = regex::Regex::new(r"(?i)(?:sort\s*code|sort-code|sortcode)\s*[:#=]?\s*(\d{2}[-\s]?\d{2}[-\s]?\d{2})\b").ok();

    // 3. Cryptocurrency (Strict Address Check)
    let re_btc_legacy = regex::Regex::new(r"\b([13][a-km-zA-HJ-NP-Z1-9]{25,34})\b").ok();
    let re_btc_bech32 = regex::Regex::new(r"\b(bc1[a-zA-HJ-NP-Z0-9]{39,59})\b").ok();
    let re_eth = regex::Regex::new(r"\b(0x[a-fA-F0-9]{40})\b").ok();
    let re_tron = regex::Regex::new(r"\b(T[A-Za-z1-9]{33})\b").ok();
    let re_sol = regex::Regex::new(r"\b([1-9A-HJ-NP-Za-km-z]{32,44})\b").ok();
    let re_crypto_uri = regex::Regex::new(r"(?i)\b((?:bitcoin|ethereum|litecoin|doge|solana|monero):[a-zA-Z0-9?=_&%-]+)\b").ok();

    // 4. PII & Identity (Strict Context)
    let re_ssn = regex::Regex::new(r"\b(\d{3}-\d{2}-\d{4})\b").ok();
    let re_passport = regex::Regex::new(r"(?i)(?:passport(?:\s*#|\s*no|\s*number)?)\s*[:#=]?\s*([A-PR-WYa-pr-wy][0-9]{7,8})\b").ok();
    // Driver's license: MUST be preceded by "driver license" and contain numbers (prevent DL = distribution list)
    let re_driver_lic = regex::Regex::new(r"(?i)(?:driver'?s?\s*license|driving\s*licence)\s*(?:#|no|number)?[:=\s]*([A-Z0-9]{6,14})\b").ok();
    let re_ein = regex::Regex::new(r"(?i)(?:ein|federal\s*tax\s*id)\s*[:#=]?\s*(\d{2}-\d{7})\b").ok();

    // 5. Locations, Travel & Addresses
    let re_street_addr = regex::Regex::new(r"\b([0-9]{1,5}\s+[A-Z][a-zA-Z0-9\s.,]{2,30}\s+(?:Street|St\.|Avenue|Ave\.|Road|Rd\.|Boulevard|Blvd\.|Lane|Ln\.|Drive|Dr\.|Way|Court|Ct\.|Parkway|Pkwy\.|Suite\s+[0-9A-Z]+|Apt\.\s+[0-9A-Z]+))\b").ok();
    let re_hotel_conf = regex::Regex::new(r"(?i)(?:hotel|lodging|flight)\s*(?:confirmation|booking|reservation)\s*(?:#|no|number)?[:=\s]*([A-Z0-9]{6,12})\b").ok();
    let re_gps = regex::Regex::new(r"\b(-?[0-9]{1,2}\.[0-9]{4,8}\s*,\s*-?[0-9]{1,3}\.[0-9]{4,8})\b").ok();

    // 6. Threats & Contraband (High Precision Words)
    let re_weapons = regex::Regex::new(r"(?i)\b(glock|beretta|ar-15|ak-47|silencer|ghost\s*gun|auto\s*sear|firearm|pistol|carbine|smg|shotgun|rifle|revolver)\b").ok();
    let re_narcotics = regex::Regex::new(r"(?i)\b(cocaine|coke|heroin|fentanyl|methamphetamine|crystal\s*meth|mdma|ecstasy|oxycodone|percocet|xanax|alprazolam|ketamine)\b").ok();
    let re_explosives = regex::Regex::new(r"(?i)\b(bomb|explosive|detonator|c4|ied|suicide\s*vest|pipe\s*bomb|anthrax|ricin|poison)\b").ok();
    let re_terrorism = regex::Regex::new(r"(?i)\b(al-qaeda|al-qa'ida|boko\s*haram|hezbollah|hamas|jihadist|suicide\s*bomber|terrorist\s*cell|taliban)\b").ok();

    // 7. Malware & Cyber IOCs
    let re_cve = regex::Regex::new(r"(CVE-\d{4}-\d{4,7})").ok();
    let re_c2 = regex::Regex::new(r"(?i)\b(command\s*and\s*control|c2\s*server|c&c\s*server|reverse\s*shell|meterpreter)\b").ok();

    // 8. Corporate & Legal Privileged
    let re_confidential = regex::Regex::new(r"(?i)\b(strictly\s+confidential|attorney[- ]client\s+privilege|privileged\s*and\s*confidential|work\s*product\s*doctrine)\b").ok();
    let re_nda = regex::Regex::new(r"(?i)\b(non[- ]disclosure\s*agreement|\bnda\b|proprietary\s+and\s+confidential)\b").ok();

    // 9. Phishing & Social Engineering
    let re_phish_cred = regex::Regex::new(r"(?i)\b(verify\s*your\s*identity|confirm\s*your\s*password|update\s*your\s*account\s*credentials)\b").ok();
    let re_phish_finance = regex::Regex::new(r"(?i)\b(wire\s*transfer\s*urgently|urgent\s*wire\s*payment|purchase\s*gift\s*cards)\b").ok();

    // Process attachments artifacts
    for (att_id, email_id, filename, sha256, mime, size, entropy, risk_flags, subj, from_addr, date_sent) in attachments {
        let cat = classify_attachment_category(&filename, &mime, entropy, risk_flags.as_deref());
        let is_dangerous = cat == "dangerous";
        let ent_val = entropy.unwrap_or(0.0);
        let is_high_entropy = ent_val > 7.5;

        // Only include dangerous, high entropy, or suspicious files in Artifacts
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
                email_id,
                email_subject: subj,
                email_from: from_addr,
                date_sent_utc: date_sent,
            });
        }
    }

    for (eid, from_addr, _from_disp, _to_addrs, _cc_addrs, _reply_to, subj_opt, body_opt, html_opt, headers_raw_opt, date_opt, _risk, is_del, is_soft_del, folder_opt, msg_id_opt, _in_reply_to_opt, _ref_opt) in emails {
        let subj = subj_opt.as_deref().unwrap_or("");
        let subj_lower = subj.to_lowercase();
        let body = body_opt.as_deref().unwrap_or("");
        let html = html_opt.as_deref().unwrap_or("");
        let headers_raw = headers_raw_opt.as_deref().unwrap_or("");
        let headers_lower = headers_raw.to_lowercase();
        let folder = folder_opt.as_deref().unwrap_or("inbox");
        let full_text = format!("{} {}", subj, body);

        let mut seen: HashSet<String> = HashSet::new();

        // 1. DELETED & CARVED MESSAGES (Only if actually deleted/recovered)
        let is_deleted = is_del || is_soft_del || folder == "trash" || folder == "deleted items" || folder == "soft_deleted";
        if is_deleted {
            artifacts.push(ForensicTaxonomyArtifact {
                id: generate_id(),
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
        if headers_lower.contains("text/calendar") || full_text.to_lowercase().contains("begin:vcalendar") || subj_lower.contains("invitation:") {
            artifacts.push(ForensicTaxonomyArtifact {
                id: generate_id(),
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

        // ─────────────────────────────────────────────────────────────────────────
        // 1. CREDENTIALS & SECRETS
        // ─────────────────────────────────────────────────────────────────────────
        if let Some(ref re) = re_cred_pair {
            for cap in re.captures_iter(&full_text) {
                let user_val = cap[1].trim().to_string();
                let pass_val = cap[2].trim().to_string();
                let key = format!("cred_pair:{}:{}", user_val, pass_val);
                if seen.insert(key) {
                    artifacts.push(ForensicTaxonomyArtifact {
                        id: generate_id(),
                        domain_id: "credentials".to_string(),
                        subcategory_id: "credentials_pair".to_string(),
                        title: "Credential Pair (User + Pass)".to_string(),
                        primary_value: format!("User: {} | Pass: {}", user_val, pass_val),
                        secondary_value: Some(from_addr.clone()),
                        details: format!("Extracted Account Login: User='{}', Pass='{}'", user_val, pass_val),
                        severity: "critical".to_string(),
                        artifact_type: "native".to_string(),
                        confidence: Some("high".to_string()),
                        email_id: eid.clone(),
                        email_subject: subj_opt.clone(),
                        email_from: from_addr.clone(),
                        date_sent_utc: date_opt.clone(),
                    });
                }
            }
        }

        if let Some(ref re) = re_pass_standalone {
            for cap in re.captures_iter(&full_text) {
                let pass_val = cap[1].trim().to_string();
                if pass_val.len() >= 6 && !pass_val.contains(' ') {
                    let key = format!("pass:{}", pass_val);
                    if seen.insert(key) {
                        artifacts.push(ForensicTaxonomyArtifact {
                            id: generate_id(),
                            domain_id: "credentials".to_string(),
                            subcategory_id: "passwords".to_string(),
                            title: "Standalone Password".to_string(),
                            primary_value: format!("Password: {}", pass_val),
                            secondary_value: Some(from_addr.clone()),
                            details: format!("Standalone password value: {}", pass_val),
                            severity: "critical".to_string(),
                            artifact_type: "native".to_string(),
                            confidence: Some("high".to_string()),
                            email_id: eid.clone(),
                            email_subject: subj_opt.clone(),
                            email_from: from_addr.clone(),
                            date_sent_utc: date_opt.clone(),
                        });
                    }
                }
            }
        }

        if let Some(ref re) = re_api_keys {
            for cap in re.captures_iter(&full_text) {
                let key_val = cap[1].to_string();
                let key = format!("api:{}", key_val);
                if seen.insert(key) {
                    let provider = if key_val.starts_with("AKIA") { "AWS Access Key" } else if key_val.starts_with("sk_live_") { "Stripe Live Key" } else if key_val.starts_with("ghp_") { "GitHub Token" } else { "Google Cloud API Key" };
                    artifacts.push(ForensicTaxonomyArtifact {
                        id: generate_id(),
                        domain_id: "credentials".to_string(),
                        subcategory_id: "api_keys".to_string(),
                        title: format!("API Key ({})", provider),
                        primary_value: key_val.clone(),
                        secondary_value: Some(from_addr.clone()),
                        details: format!("Extracted {} credential token", provider),
                        severity: "critical".to_string(),
                        artifact_type: "native".to_string(),
                        confidence: Some("high".to_string()),
                        email_id: eid.clone(),
                        email_subject: subj_opt.clone(),
                        email_from: from_addr.clone(),
                        date_sent_utc: date_opt.clone(),
                    });
                }
            }
        }

        if let Some(ref re) = re_bearer {
            for cap in re.captures_iter(&full_text) {
                let token = cap[1].to_string();
                let key = format!("bearer:{}", token);
                if seen.insert(key) {
                    artifacts.push(ForensicTaxonomyArtifact {
                        id: generate_id(),
                        domain_id: "credentials".to_string(),
                        subcategory_id: "bearer_tokens".to_string(),
                        title: "Bearer Authorization Token".to_string(),
                        primary_value: format!("Bearer {}", &token[..token.len().min(40)]),
                        secondary_value: Some(from_addr.clone()),
                        details: format!("OAuth/Bearer authorization token: {}", token),
                        severity: "critical".to_string(),
                        artifact_type: "native".to_string(),
                        confidence: Some("high".to_string()),
                        email_id: eid.clone(),
                        email_subject: subj_opt.clone(),
                        email_from: from_addr.clone(),
                        date_sent_utc: date_opt.clone(),
                    });
                }
            }
        }

        if let Some(ref re) = re_jwt {
            for cap in re.captures_iter(&full_text) {
                let jwt = cap[1].to_string();
                let key = format!("jwt:{}", jwt);
                if seen.insert(key) {
                    artifacts.push(ForensicTaxonomyArtifact {
                        id: generate_id(),
                        domain_id: "credentials".to_string(),
                        subcategory_id: "jwt_tokens".to_string(),
                        title: "JSON Web Token (JWT)".to_string(),
                        primary_value: format!("JWT: {}", &jwt[..jwt.len().min(45)]),
                        secondary_value: Some(from_addr.clone()),
                        details: format!("JSON Web Token (JWT) session credential: {}", jwt),
                        severity: "critical".to_string(),
                        artifact_type: "native".to_string(),
                        confidence: Some("high".to_string()),
                        email_id: eid.clone(),
                        email_subject: subj_opt.clone(),
                        email_from: from_addr.clone(),
                        date_sent_utc: date_opt.clone(),
                    });
                }
            }
        }

        if let Some(ref re) = re_ssh_key {
            if re.is_match(&full_text) {
                let key = "ssh_key_block".to_string();
                if seen.insert(key) {
                    artifacts.push(ForensicTaxonomyArtifact {
                        id: generate_id(),
                        domain_id: "credentials".to_string(),
                        subcategory_id: "ssh_keys".to_string(),
                        title: "SSH / OpenSSH Private Key Block".to_string(),
                        primary_value: "-----BEGIN PRIVATE KEY-----".to_string(),
                        secondary_value: Some(from_addr.clone()),
                        details: "Private cryptographic SSH key block exposed in message".to_string(),
                        severity: "critical".to_string(),
                        artifact_type: "native".to_string(),
                        confidence: Some("high".to_string()),
                        email_id: eid.clone(),
                        email_subject: subj_opt.clone(),
                        email_from: from_addr.clone(),
                        date_sent_utc: date_opt.clone(),
                    });
                }
            }
        }

        // BIP-39 Seed phrase: Must have 12+ words, no dictionary newsletters
        if let Some(ref re) = re_seed {
            for cap in re.captures_iter(&full_text) {
                let seed = cap[1].trim().to_string();
                let words: Vec<&str> = seed.split_whitespace().collect();
                if words.len() >= 12 && !seed.contains("word of the day") && !seed.contains("merriam") {
                    let key = format!("seed:{}", seed);
                    if seen.insert(key) {
                        artifacts.push(ForensicTaxonomyArtifact {
                            id: generate_id(),
                            domain_id: "crypto".to_string(),
                            subcategory_id: "seed_phrases".to_string(),
                            title: "BIP-39 Mnemonic Seed Phrase (12+ words)".to_string(),
                            primary_value: seed.clone(),
                            secondary_value: Some(from_addr.clone()),
                            details: format!("Cryptocurrency recovery seed phrase ({} words): {}", words.len(), seed),
                            severity: "critical".to_string(),
                            artifact_type: "native".to_string(),
                            confidence: Some("high".to_string()),
                            email_id: eid.clone(),
                            email_subject: subj_opt.clone(),
                            email_from: from_addr.clone(),
                            date_sent_utc: date_opt.clone(),
                        });
                    }
                }
            }
        }

        if let Some(ref re) = re_privkey {
            for cap in re.captures_iter(&full_text) {
                let pkey = cap[1].trim().to_string();
                let key = format!("privkey:{}", pkey);
                if seen.insert(key) {
                    artifacts.push(ForensicTaxonomyArtifact {
                        id: generate_id(),
                        domain_id: "crypto".to_string(),
                        subcategory_id: "private_keys".to_string(),
                        title: "Cryptocurrency Hex Private Key".to_string(),
                        primary_value: pkey.clone(),
                        secondary_value: Some(from_addr.clone()),
                        details: format!("Raw hex private key: {}", pkey),
                        severity: "critical".to_string(),
                        artifact_type: "native".to_string(),
                        confidence: Some("high".to_string()),
                        email_id: eid.clone(),
                        email_subject: subj_opt.clone(),
                        email_from: from_addr.clone(),
                        date_sent_utc: date_opt.clone(),
                    });
                }
            }
        }

        // ─────────────────────────────────────────────────────────────────────────
        // 2. FINANCIAL & BANKING (Luhn + ABA Routing Validated)
        // ─────────────────────────────────────────────────────────────────────────
        if let Some(ref re) = re_cc_spaced {
            for cap in re.captures_iter(&full_text) {
                let cc_raw = cap[1].replace([' ', '-'], "");
                if luhn_check(&cc_raw) {
                    let key = format!("cc:{}", cc_raw);
                    if seen.insert(key) {
                        let card_type = if cc_raw.starts_with('4') { "Visa" } else if cc_raw.starts_with("34") || cc_raw.starts_with("37") { "Amex" } else if cc_raw.starts_with("6011") { "Discover" } else { "MasterCard" };
                        artifacts.push(ForensicTaxonomyArtifact {
                            id: generate_id(),
                            domain_id: "financial".to_string(),
                            subcategory_id: "credit_cards".to_string(),
                            title: format!("Credit Card ({})", card_type),
                            primary_value: cap[1].to_string(),
                            secondary_value: Some(from_addr.clone()),
                            details: format!("Luhn-Verified Credit Card Number ({})", card_type),
                            severity: "critical".to_string(),
                            artifact_type: "native".to_string(),
                            confidence: Some("high".to_string()),
                            email_id: eid.clone(),
                            email_subject: subj_opt.clone(),
                            email_from: from_addr.clone(),
                            date_sent_utc: date_opt.clone(),
                        });
                    }
                }
            }
        }

        if let Some(ref re) = re_cc_raw {
            for cap in re.captures_iter(&full_text) {
                let cc_raw = cap[1].to_string();
                if luhn_check(&cc_raw) {
                    let key = format!("cc:{}", cc_raw);
                    if seen.insert(key) {
                        let card_type = if cc_raw.starts_with('4') { "Visa" } else if cc_raw.starts_with("34") || cc_raw.starts_with("37") { "Amex" } else if cc_raw.starts_with("6011") { "Discover" } else { "MasterCard" };
                        artifacts.push(ForensicTaxonomyArtifact {
                            id: generate_id(),
                            domain_id: "financial".to_string(),
                            subcategory_id: "credit_cards".to_string(),
                            title: format!("Credit Card ({})", card_type),
                            primary_value: cc_raw.clone(),
                            secondary_value: Some(from_addr.clone()),
                            details: format!("Luhn-Verified Card Number: {}", cc_raw),
                            severity: "critical".to_string(),
                            artifact_type: "native".to_string(),
                            confidence: Some("high".to_string()),
                            email_id: eid.clone(),
                            email_subject: subj_opt.clone(),
                            email_from: from_addr.clone(),
                            date_sent_utc: date_opt.clone(),
                        });
                    }
                }
            }
        }

        if let Some(ref re) = re_routing {
            for cap in re.captures_iter(&full_text) {
                let r_no = cap[1].trim().to_string();
                if validate_routing_number(&r_no) {
                    let key = format!("routing:{}", r_no);
                    if seen.insert(key) {
                        artifacts.push(ForensicTaxonomyArtifact {
                            id: generate_id(),
                            domain_id: "financial".to_string(),
                            subcategory_id: "routing_numbers".to_string(),
                            title: "US ABA Bank Routing Number".to_string(),
                            primary_value: format!("Routing #: {}", r_no),
                            secondary_value: Some(from_addr.clone()),
                            details: format!("Verified US 9-digit ABA Bank Routing Number: {}", r_no),
                            severity: "critical".to_string(),
                            artifact_type: "native".to_string(),
                            confidence: Some("high".to_string()),
                            email_id: eid.clone(),
                            email_subject: subj_opt.clone(),
                            email_from: from_addr.clone(),
                            date_sent_utc: date_opt.clone(),
                        });
                    }
                }
            }
        }

        if let Some(ref re) = re_iban {
            for cap in re.captures_iter(&full_text) {
                let iban = cap[1].trim().to_string();
                let key = format!("iban:{}", iban);
                if seen.insert(key) {
                    artifacts.push(ForensicTaxonomyArtifact {
                        id: generate_id(),
                        domain_id: "financial".to_string(),
                        subcategory_id: "iban".to_string(),
                        title: "IBAN Bank Account Number".to_string(),
                        primary_value: format!("IBAN: {}", iban),
                        secondary_value: Some(from_addr.clone()),
                        details: format!("International Bank Account Number (IBAN): {}", iban),
                        severity: "critical".to_string(),
                        artifact_type: "native".to_string(),
                        confidence: Some("high".to_string()),
                        email_id: eid.clone(),
                        email_subject: subj_opt.clone(),
                        email_from: from_addr.clone(),
                        date_sent_utc: date_opt.clone(),
                    });
                }
            }
        }

        if let Some(ref re) = re_swift {
            for cap in re.captures_iter(&full_text) {
                let swift = cap[1].trim().to_string();
                let key = format!("swift:{}", swift);
                if seen.insert(key) {
                    artifacts.push(ForensicTaxonomyArtifact {
                        id: generate_id(),
                        domain_id: "financial".to_string(),
                        subcategory_id: "swift_bic".to_string(),
                        title: "SWIFT / BIC Bank Code".to_string(),
                        primary_value: format!("SWIFT: {}", swift),
                        secondary_value: Some(from_addr.clone()),
                        details: format!("International Bank SWIFT/BIC Identifier: {}", swift),
                        severity: "high".to_string(),
                        artifact_type: "native".to_string(),
                        confidence: Some("high".to_string()),
                        email_id: eid.clone(),
                        email_subject: subj_opt.clone(),
                        email_from: from_addr.clone(),
                        date_sent_utc: date_opt.clone(),
                    });
                }
            }
        }

        if let Some(ref re) = re_account {
            for cap in re.captures_iter(&full_text) {
                let acc = cap[1].trim().to_string();
                let key = format!("acct:{}", acc);
                if seen.insert(key) {
                    artifacts.push(ForensicTaxonomyArtifact {
                        id: generate_id(),
                        domain_id: "financial".to_string(),
                        subcategory_id: "account_numbers".to_string(),
                        title: "Bank Account Number".to_string(),
                        primary_value: format!("Account #: {}", acc),
                        secondary_value: Some(from_addr.clone()),
                        details: format!("Extracted financial account number: {}", acc),
                        severity: "critical".to_string(),
                        artifact_type: "native".to_string(),
                        confidence: Some("high".to_string()),
                        email_id: eid.clone(),
                        email_subject: subj_opt.clone(),
                        email_from: from_addr.clone(),
                        date_sent_utc: date_opt.clone(),
                    });
                }
            }
        }

        if let Some(ref re) = re_sort_code {
            for cap in re.captures_iter(&full_text) {
                let sort = cap[1].trim().to_string();
                let key = format!("sort:{}", sort);
                if seen.insert(key) {
                    artifacts.push(ForensicTaxonomyArtifact {
                        id: generate_id(),
                        domain_id: "financial".to_string(),
                        subcategory_id: "sort_code".to_string(),
                        title: "UK / Ireland Bank Sort Code".to_string(),
                        primary_value: format!("Sort Code: {}", sort),
                        secondary_value: Some(from_addr.clone()),
                        details: format!("Bank clearing sort code: {}", sort),
                        severity: "high".to_string(),
                        artifact_type: "native".to_string(),
                        confidence: Some("high".to_string()),
                        email_id: eid.clone(),
                        email_subject: subj_opt.clone(),
                        email_from: from_addr.clone(),
                        date_sent_utc: date_opt.clone(),
                    });
                }
            }
        }

        // ─────────────────────────────────────────────────────────────────────────
        // 3. CRYPTOCURRENCY (Validated Base58 / Bech32 / EVM)
        // ─────────────────────────────────────────────────────────────────────────
        if let Some(ref re) = re_btc_legacy {
            for cap in re.captures_iter(&full_text) {
                let btc = cap[1].to_string();
                if validate_btc_base58(&btc) {
                    let key = format!("btc:{}", btc);
                    if seen.insert(key) {
                        artifacts.push(ForensicTaxonomyArtifact {
                            id: generate_id(),
                            domain_id: "crypto".to_string(),
                            subcategory_id: "bitcoin_p2pkh".to_string(),
                            title: "Bitcoin Legacy (P2PKH) Address".to_string(),
                            primary_value: btc.clone(),
                            secondary_value: Some(from_addr.clone()),
                            details: format!("Verified Bitcoin Base58 address: {}", btc),
                            severity: "high".to_string(),
                            artifact_type: "native".to_string(),
                            confidence: Some("high".to_string()),
                            email_id: eid.clone(),
                            email_subject: subj_opt.clone(),
                            email_from: from_addr.clone(),
                            date_sent_utc: date_opt.clone(),
                        });
                    }
                }
            }
        }

        if let Some(ref re) = re_btc_bech32 {
            for cap in re.captures_iter(&full_text) {
                let btc = cap[1].to_string();
                let key = format!("btc_bech:{}", btc);
                if seen.insert(key) {
                    artifacts.push(ForensicTaxonomyArtifact {
                        id: generate_id(),
                        domain_id: "crypto".to_string(),
                        subcategory_id: "bitcoin_bech32".to_string(),
                        title: "Bitcoin SegWit (Bech32) Address".to_string(),
                        primary_value: btc.clone(),
                        secondary_value: Some(from_addr.clone()),
                        details: format!("Bitcoin SegWit Native Bech32 Address: {}", btc),
                        severity: "high".to_string(),
                        artifact_type: "native".to_string(),
                        confidence: Some("high".to_string()),
                        email_id: eid.clone(),
                        email_subject: subj_opt.clone(),
                        email_from: from_addr.clone(),
                        date_sent_utc: date_opt.clone(),
                    });
                }
            }
        }

        if let Some(ref re) = re_eth {
            for cap in re.captures_iter(&full_text) {
                let eth = cap[1].to_string();
                if eth != "0x0000000000000000000000000000000000000000" {
                    let key = format!("eth:{}", eth);
                    if seen.insert(key) {
                        artifacts.push(ForensicTaxonomyArtifact {
                            id: generate_id(),
                            domain_id: "crypto".to_string(),
                            subcategory_id: "ethereum".to_string(),
                            title: "Ethereum / ERC-20 Wallet Address".to_string(),
                            primary_value: eth.clone(),
                            secondary_value: Some(from_addr.clone()),
                            details: format!("Ethereum / EVM Address: {}", eth),
                            severity: "high".to_string(),
                            artifact_type: "native".to_string(),
                            confidence: Some("high".to_string()),
                            email_id: eid.clone(),
                            email_subject: subj_opt.clone(),
                            email_from: from_addr.clone(),
                            date_sent_utc: date_opt.clone(),
                        });
                    }
                }
            }
        }

        if let Some(ref re) = re_tron {
            for cap in re.captures_iter(&full_text) {
                let trx = cap[1].to_string();
                let key = format!("trx:{}", trx);
                if seen.insert(key) {
                    artifacts.push(ForensicTaxonomyArtifact {
                        id: generate_id(),
                        domain_id: "crypto".to_string(),
                        subcategory_id: "tron".to_string(),
                        title: "TRON (TRX / USDT-TRC20) Address".to_string(),
                        primary_value: trx.clone(),
                        secondary_value: Some(from_addr.clone()),
                        details: format!("TRON Network Address: {}", trx),
                        severity: "high".to_string(),
                        artifact_type: "native".to_string(),
                        confidence: Some("high".to_string()),
                        email_id: eid.clone(),
                        email_subject: subj_opt.clone(),
                        email_from: from_addr.clone(),
                        date_sent_utc: date_opt.clone(),
                    });
                }
            }
        }

        if let Some(ref re) = re_sol {
            for cap in re.captures_iter(&full_text) {
                let sol = cap[1].to_string();
                if sol.len() >= 32 && sol.len() <= 44 && !sol.contains('@') {
                    let key = format!("sol:{}", sol);
                    if seen.insert(key) {
                        artifacts.push(ForensicTaxonomyArtifact {
                            id: generate_id(),
                            domain_id: "crypto".to_string(),
                            subcategory_id: "solana".to_string(),
                            title: "Solana (SOL) Wallet Address".to_string(),
                            primary_value: sol.clone(),
                            secondary_value: Some(from_addr.clone()),
                            details: format!("Solana Blockchain Public Address: {}", sol),
                            severity: "high".to_string(),
                            artifact_type: "native".to_string(),
                            confidence: Some("high".to_string()),
                            email_id: eid.clone(),
                            email_subject: subj_opt.clone(),
                            email_from: from_addr.clone(),
                            date_sent_utc: date_opt.clone(),
                        });
                    }
                }
            }
        }

        if let Some(ref re) = re_crypto_uri {
            for cap in re.captures_iter(&full_text) {
                let uri = cap[1].to_string();
                let key = format!("crypto_uri:{}", uri);
                if seen.insert(key) {
                    artifacts.push(ForensicTaxonomyArtifact {
                        id: generate_id(),
                        domain_id: "crypto".to_string(),
                        subcategory_id: "qr_wallet_uris".to_string(),
                        title: "Cryptocurrency Wallet URI".to_string(),
                        primary_value: uri.clone(),
                        secondary_value: Some(from_addr.clone()),
                        details: format!("Payment URI schema: {}", uri),
                        severity: "high".to_string(),
                        artifact_type: "native".to_string(),
                        confidence: Some("high".to_string()),
                        email_id: eid.clone(),
                        email_subject: subj_opt.clone(),
                        email_from: from_addr.clone(),
                        date_sent_utc: date_opt.clone(),
                    });
                }
            }
        }

        // ─────────────────────────────────────────────────────────────────────────
        // 4. PERSONAL IDENTIFIABLE INFORMATION (PII) (Validated SSN / Passport / Real DL)
        // ─────────────────────────────────────────────────────────────────────────
        if let Some(ref re) = re_ssn {
            for cap in re.captures_iter(&full_text) {
                let ssn = cap[1].to_string();
                if validate_ssn(&ssn) {
                    let key = format!("ssn:{}", ssn);
                    if seen.insert(key) {
                        artifacts.push(ForensicTaxonomyArtifact {
                            id: generate_id(),
                            domain_id: "identity_docs".to_string(),
                            subcategory_id: "ssn".to_string(),
                            title: "US Social Security Number (SSN)".to_string(),
                            primary_value: ssn.clone(),
                            secondary_value: Some(from_addr.clone()),
                            details: format!("Verified US Social Security Number: {}", ssn),
                            severity: "critical".to_string(),
                            artifact_type: "native".to_string(),
                            confidence: Some("high".to_string()),
                            email_id: eid.clone(),
                            email_subject: subj_opt.clone(),
                            email_from: from_addr.clone(),
                            date_sent_utc: date_opt.clone(),
                        });
                    }
                }
            }
        }

        if let Some(ref re) = re_passport {
            for cap in re.captures_iter(&full_text) {
                let pass = cap[1].to_string();
                let key = format!("passport:{}", pass);
                if seen.insert(key) {
                    artifacts.push(ForensicTaxonomyArtifact {
                        id: generate_id(),
                        domain_id: "identity_docs".to_string(),
                        subcategory_id: "passport".to_string(),
                        title: "International Passport Number".to_string(),
                        primary_value: format!("Passport: {}", pass),
                        secondary_value: Some(from_addr.clone()),
                        details: format!("Passport document identifier: {}", pass),
                        severity: "high".to_string(),
                        artifact_type: "native".to_string(),
                        confidence: Some("high".to_string()),
                        email_id: eid.clone(),
                        email_subject: subj_opt.clone(),
                        email_from: from_addr.clone(),
                        date_sent_utc: date_opt.clone(),
                    });
                }
            }
        }

        if let Some(ref re) = re_driver_lic {
            for cap in re.captures_iter(&full_text) {
                let dl = cap[1].trim().to_string();
                // Must contain digits to avoid corporate words like ACTIVE, kinning, etc.
                if dl.chars().any(|c| c.is_ascii_digit()) && dl.len() >= 6 {
                    let key = format!("dl:{}", dl);
                    if seen.insert(key) {
                        artifacts.push(ForensicTaxonomyArtifact {
                            id: generate_id(),
                            domain_id: "identity_docs".to_string(),
                            subcategory_id: "drivers_license".to_string(),
                            title: "Driver's License (DLN)".to_string(),
                            primary_value: format!("DL: {}", dl),
                            secondary_value: Some(from_addr.clone()),
                            details: format!("Driver's license identifier: {}", dl),
                            severity: "high".to_string(),
                            artifact_type: "native".to_string(),
                            confidence: Some("high".to_string()),
                            email_id: eid.clone(),
                            email_subject: subj_opt.clone(),
                            email_from: from_addr.clone(),
                            date_sent_utc: date_opt.clone(),
                        });
                    }
                }
            }
        }

        if let Some(ref re) = re_ein {
            for cap in re.captures_iter(&full_text) {
                let ein = cap[1].to_string();
                let key = format!("ein:{}", ein);
                if seen.insert(key) {
                    artifacts.push(ForensicTaxonomyArtifact {
                        id: generate_id(),
                        domain_id: "identity_docs".to_string(),
                        subcategory_id: "ein".to_string(),
                        title: "Employer Identification Number (EIN)".to_string(),
                        primary_value: format!("EIN: {}", ein),
                        secondary_value: Some(from_addr.clone()),
                        details: format!("US Federal Employer Identification Number: {}", ein),
                        severity: "high".to_string(),
                        artifact_type: "native".to_string(),
                        confidence: Some("high".to_string()),
                        email_id: eid.clone(),
                        email_subject: subj_opt.clone(),
                        email_from: from_addr.clone(),
                        date_sent_utc: date_opt.clone(),
                    });
                }
            }
        }

        // ─────────────────────────────────────────────────────────────────────────
        // 5. LOCATIONS, TRAVEL & ADDRESSES
        // ─────────────────────────────────────────────────────────────────────────
        if let Some(ref re) = re_street_addr {
            for cap in re.captures_iter(&body) {
                let addr = cap[1].trim().to_string();
                let key = format!("addr:{}", addr);
                if seen.insert(key) {
                    artifacts.push(ForensicTaxonomyArtifact {
                        id: generate_id(),
                        domain_id: "locations".to_string(),
                        subcategory_id: "street_address".to_string(),
                        title: "Physical Street Address".to_string(),
                        primary_value: addr.clone(),
                        secondary_value: Some(from_addr.clone()),
                        details: format!("Street address: {}", addr),
                        severity: "medium".to_string(),
                        artifact_type: "native".to_string(),
                        confidence: Some("high".to_string()),
                        email_id: eid.clone(),
                        email_subject: subj_opt.clone(),
                        email_from: from_addr.clone(),
                        date_sent_utc: date_opt.clone(),
                    });
                    break;
                }
            }
        }

        if let Some(ref re) = re_hotel_conf {
            for cap in re.captures_iter(&full_text) {
                let conf = cap[1].to_string();
                let key = format!("hotel_conf:{}", conf);
                if seen.insert(key) {
                    artifacts.push(ForensicTaxonomyArtifact {
                        id: generate_id(),
                        domain_id: "locations".to_string(),
                        subcategory_id: "hotel_booking".to_string(),
                        title: "Travel / Lodging Confirmation".to_string(),
                        primary_value: format!("Booking #: {}", conf),
                        secondary_value: Some(from_addr.clone()),
                        details: format!("Travel lodging confirmation code: {}", conf),
                        severity: "high".to_string(),
                        artifact_type: "native".to_string(),
                        confidence: Some("high".to_string()),
                        email_id: eid.clone(),
                        email_subject: subj_opt.clone(),
                        email_from: from_addr.clone(),
                        date_sent_utc: date_opt.clone(),
                    });
                }
            }
        }

        if let Some(ref re) = re_gps {
            for cap in re.captures_iter(&full_text) {
                let gps = cap[1].to_string();
                let key = format!("gps:{}", gps);
                if seen.insert(key) {
                    artifacts.push(ForensicTaxonomyArtifact {
                        id: generate_id(),
                        domain_id: "locations".to_string(),
                        subcategory_id: "gps_coordinates".to_string(),
                        title: "GPS Geographic Coordinates".to_string(),
                        primary_value: gps.clone(),
                        secondary_value: Some(from_addr.clone()),
                        details: format!("Latitude / Longitude coordinates: {}", gps),
                        severity: "high".to_string(),
                        artifact_type: "native".to_string(),
                        confidence: Some("high".to_string()),
                        email_id: eid.clone(),
                        email_subject: subj_opt.clone(),
                        email_from: from_addr.clone(),
                        date_sent_utc: date_opt.clone(),
                    });
                }
            }
        }

        // ─────────────────────────────────────────────────────────────────────────
        // 6. THREATS & CONTRABAND (Precise Keywords)
        // ─────────────────────────────────────────────────────────────────────────
        if let Some(ref re) = re_weapons {
            for cap in re.captures_iter(&full_text) {
                let wpn = cap[1].to_string();
                let key = format!("wpn:{}", wpn.to_lowercase());
                if seen.insert(key) {
                    artifacts.push(ForensicTaxonomyArtifact {
                        id: generate_id(),
                        domain_id: "contraband".to_string(),
                        subcategory_id: "weapons".to_string(),
                        title: format!("Firearms & Weapons ({})", wpn.to_uppercase()),
                        primary_value: wpn.to_uppercase(),
                        secondary_value: Some(from_addr.clone()),
                        details: format!("Firearm or weapons keyword: {}", wpn),
                        severity: "critical".to_string(),
                        artifact_type: "native".to_string(),
                        confidence: Some("high".to_string()),
                        email_id: eid.clone(),
                        email_subject: subj_opt.clone(),
                        email_from: from_addr.clone(),
                        date_sent_utc: date_opt.clone(),
                    });
                    break;
                }
            }
        }

        if let Some(ref re) = re_narcotics {
            for cap in re.captures_iter(&full_text) {
                let drug = cap[1].to_string();
                let key = format!("drug:{}", drug.to_lowercase());
                if seen.insert(key) {
                    artifacts.push(ForensicTaxonomyArtifact {
                        id: generate_id(),
                        domain_id: "contraband".to_string(),
                        subcategory_id: "narcotics".to_string(),
                        title: format!("Controlled Substances ({})", drug.to_uppercase()),
                        primary_value: drug.to_uppercase(),
                        secondary_value: Some(from_addr.clone()),
                        details: format!("Illicit drug mention: {}", drug),
                        severity: "critical".to_string(),
                        artifact_type: "native".to_string(),
                        confidence: Some("high".to_string()),
                        email_id: eid.clone(),
                        email_subject: subj_opt.clone(),
                        email_from: from_addr.clone(),
                        date_sent_utc: date_opt.clone(),
                    });
                    break;
                }
            }
        }

        if let Some(ref re) = re_explosives {
            for cap in re.captures_iter(&full_text) {
                let exp = cap[1].to_string();
                let key = format!("exp:{}", exp.to_lowercase());
                if seen.insert(key) {
                    artifacts.push(ForensicTaxonomyArtifact {
                        id: generate_id(),
                        domain_id: "contraband".to_string(),
                        subcategory_id: "explosives".to_string(),
                        title: format!("Explosives & IED Threat ({})", exp.to_uppercase()),
                        primary_value: exp.to_uppercase(),
                        secondary_value: Some(from_addr.clone()),
                        details: format!("Explosive material or detonator indicator: {}", exp),
                        severity: "critical".to_string(),
                        artifact_type: "native".to_string(),
                        confidence: Some("high".to_string()),
                        email_id: eid.clone(),
                        email_subject: subj_opt.clone(),
                        email_from: from_addr.clone(),
                        date_sent_utc: date_opt.clone(),
                    });
                    break;
                }
            }
        }

        if let Some(ref re) = re_terrorism {
            for cap in re.captures_iter(&full_text) {
                let trr = cap[1].to_string();
                let key = format!("trr:{}", trr.to_lowercase());
                if seen.insert(key) {
                    artifacts.push(ForensicTaxonomyArtifact {
                        id: generate_id(),
                        domain_id: "contraband".to_string(),
                        subcategory_id: "terrorism".to_string(),
                        title: format!("Violent Extremism ({})", trr.to_uppercase()),
                        primary_value: trr.to_uppercase(),
                        secondary_value: Some(from_addr.clone()),
                        details: format!("Terrorist organization or extremist keyword: {}", trr),
                        severity: "critical".to_string(),
                        artifact_type: "native".to_string(),
                        confidence: Some("high".to_string()),
                        email_id: eid.clone(),
                        email_subject: subj_opt.clone(),
                        email_from: from_addr.clone(),
                        date_sent_utc: date_opt.clone(),
                    });
                    break;
                }
            }
        }

        // ─────────────────────────────────────────────────────────────────────────
        // 7. MALWARE & CYBER IOCs
        // ─────────────────────────────────────────────────────────────────────────
        if let Some(ref re) = re_cve {
            for cap in re.captures_iter(&full_text) {
                let cve = cap[1].to_string();
                let key = format!("cve:{}", cve);
                if seen.insert(key) {
                    artifacts.push(ForensicTaxonomyArtifact {
                        id: generate_id(),
                        domain_id: "malware_threats".to_string(),
                        subcategory_id: "cve_vulnerability".to_string(),
                        title: format!("Common Vulnerability ({})", cve),
                        primary_value: cve.clone(),
                        secondary_value: Some(from_addr.clone()),
                        details: format!("Vulnerability identifier: {}", cve),
                        severity: "critical".to_string(),
                        artifact_type: "native".to_string(),
                        confidence: Some("high".to_string()),
                        email_id: eid.clone(),
                        email_subject: subj_opt.clone(),
                        email_from: from_addr.clone(),
                        date_sent_utc: date_opt.clone(),
                    });
                }
            }
        }

        if let Some(ref re) = re_c2 {
            for cap in re.captures_iter(&full_text) {
                let c2 = cap[1].to_string();
                let key = format!("c2:{}", c2.to_lowercase());
                if seen.insert(key) {
                    artifacts.push(ForensicTaxonomyArtifact {
                        id: generate_id(),
                        domain_id: "malware_threats".to_string(),
                        subcategory_id: "c2_indicators".to_string(),
                        title: "Command & Control (C2) Indicator".to_string(),
                        primary_value: c2.to_uppercase(),
                        secondary_value: Some(from_addr.clone()),
                        details: format!("Command & Control terminology: {}", c2),
                        severity: "critical".to_string(),
                        artifact_type: "native".to_string(),
                        confidence: Some("high".to_string()),
                        email_id: eid.clone(),
                        email_subject: subj_opt.clone(),
                        email_from: from_addr.clone(),
                        date_sent_utc: date_opt.clone(),
                    });
                    break;
                }
            }
        }

        // ─────────────────────────────────────────────────────────────────────────
        // 8. CORPORATE & LEGAL PRIVILEGED
        // ─────────────────────────────────────────────────────────────────────────
        if let Some(ref re) = re_confidential {
            for cap in re.captures_iter(&full_text) {
                let conf = cap[1].to_string();
                let key = format!("confidential:{}", conf.to_lowercase());
                if seen.insert(key) {
                    artifacts.push(ForensicTaxonomyArtifact {
                        id: generate_id(),
                        domain_id: "secrets".to_string(),
                        subcategory_id: "privileged_confidential".to_string(),
                        title: format!("Legal Privilege / Confidential ({})", conf.to_uppercase()),
                        primary_value: conf.to_uppercase(),
                        secondary_value: Some(from_addr.clone()),
                        details: format!("Confidentiality or legal privilege notice: {}", conf),
                        severity: "high".to_string(),
                        artifact_type: "native".to_string(),
                        confidence: Some("high".to_string()),
                        email_id: eid.clone(),
                        email_subject: subj_opt.clone(),
                        email_from: from_addr.clone(),
                        date_sent_utc: date_opt.clone(),
                    });
                    break;
                }
            }
        }

        if let Some(ref re) = re_nda {
            for cap in re.captures_iter(&full_text) {
                let nda_val = cap[1].to_string();
                let key = format!("nda:{}", nda_val.to_lowercase());
                if seen.insert(key) {
                    artifacts.push(ForensicTaxonomyArtifact {
                        id: generate_id(),
                        domain_id: "secrets".to_string(),
                        subcategory_id: "nda_agreements".to_string(),
                        title: "Non-Disclosure Agreement (NDA)".to_string(),
                        primary_value: nda_val.to_uppercase(),
                        secondary_value: Some(from_addr.clone()),
                        details: format!("NDA or distribution restriction clause: {}", nda_val),
                        severity: "high".to_string(),
                        artifact_type: "native".to_string(),
                        confidence: Some("high".to_string()),
                        email_id: eid.clone(),
                        email_subject: subj_opt.clone(),
                        email_from: from_addr.clone(),
                        date_sent_utc: date_opt.clone(),
                    });
                    break;
                }
            }
        }

        // ─────────────────────────────────────────────────────────────────────────
        // 9. PHISHING & SOCIAL ENGINEERING
        // ─────────────────────────────────────────────────────────────────────────
        if let Some(ref re) = re_phish_cred {
            for cap in re.captures_iter(&full_text) {
                let cr = cap[1].to_string();
                let key = format!("phish_cred:{}", cr.to_lowercase());
                if seen.insert(key) {
                    artifacts.push(ForensicTaxonomyArtifact {
                        id: generate_id(),
                        domain_id: "phishing".to_string(),
                        subcategory_id: "credential_requests".to_string(),
                        title: "Credential Harvesting Lure".to_string(),
                        primary_value: cr.to_uppercase(),
                        secondary_value: Some(from_addr.clone()),
                        details: format!("Request for login credentials / password update: {}", cr),
                        severity: "critical".to_string(),
                        artifact_type: "derived".to_string(),
                        confidence: Some("high".to_string()),
                        email_id: eid.clone(),
                        email_subject: subj_opt.clone(),
                        email_from: from_addr.clone(),
                        date_sent_utc: date_opt.clone(),
                    });
                    break;
                }
            }
        }

        if let Some(ref re) = re_phish_finance {
            for cap in re.captures_iter(&full_text) {
                let fin = cap[1].to_string();
                let key = format!("phish_fin:{}", fin.to_lowercase());
                if seen.insert(key) {
                    artifacts.push(ForensicTaxonomyArtifact {
                        id: generate_id(),
                        domain_id: "phishing".to_string(),
                        subcategory_id: "financial_demands".to_string(),
                        title: "BEC / Financial Payment Demand".to_string(),
                        primary_value: fin.to_uppercase(),
                        secondary_value: Some(from_addr.clone()),
                        details: format!("Fraudulent wire transfer or gift card demand: {}", fin),
                        severity: "critical".to_string(),
                        artifact_type: "derived".to_string(),
                        confidence: Some("high".to_string()),
                        email_id: eid.clone(),
                        email_subject: subj_opt.clone(),
                        email_from: from_addr.clone(),
                        date_sent_utc: date_opt.clone(),
                    });
                    break;
                }
            }
        }

        // ─────────────────────────────────────────────────────────────────────────
        // 10. AUTHENTICATION FAILURES (Only report actual FAILURES, not every email)
        // ─────────────────────────────────────────────────────────────────────────
        if headers_lower.contains("spf=fail") || headers_lower.contains("spf=softfail") {
            let key = "spf_fail".to_string();
            if seen.insert(key) {
                artifacts.push(ForensicTaxonomyArtifact {
                    id: generate_id(),
                    domain_id: "authentication".to_string(),
                    subcategory_id: "spf_fail".to_string(),
                    title: "SPF Authentication Failure".to_string(),
                    primary_value: "SPF: FAIL".to_string(),
                    secondary_value: Some(from_addr.clone()),
                    details: "Sender failed SPF domain authorization check".to_string(),
                    severity: "critical".to_string(),
                    artifact_type: "native".to_string(),
                    confidence: Some("high".to_string()),
                    email_id: eid.clone(),
                    email_subject: subj_opt.clone(),
                    email_from: from_addr.clone(),
                    date_sent_utc: date_opt.clone(),
                });
            }
        }

        if headers_lower.contains("dkim=fail") {
            let key = "dkim_fail".to_string();
            if seen.insert(key) {
                artifacts.push(ForensicTaxonomyArtifact {
                    id: generate_id(),
                    domain_id: "authentication".to_string(),
                    subcategory_id: "dkim_fail".to_string(),
                    title: "DKIM Cryptographic Signature Failure".to_string(),
                    primary_value: "DKIM: FAIL".to_string(),
                    secondary_value: Some(from_addr.clone()),
                    details: "Cryptographic signature validation failed on transport header".to_string(),
                    severity: "critical".to_string(),
                    artifact_type: "native".to_string(),
                    confidence: Some("high".to_string()),
                    email_id: eid.clone(),
                    email_subject: subj_opt.clone(),
                    email_from: from_addr.clone(),
                    date_sent_utc: date_opt.clone(),
                });
            }
        }

        if headers_lower.contains("dmarc=fail") {
            let key = "dmarc_fail".to_string();
            if seen.insert(key) {
                artifacts.push(ForensicTaxonomyArtifact {
                    id: generate_id(),
                    domain_id: "authentication".to_string(),
                    subcategory_id: "dmarc_fail".to_string(),
                    title: "DMARC Alignment Policy Failure".to_string(),
                    primary_value: "DMARC: FAIL".to_string(),
                    secondary_value: Some(from_addr.clone()),
                    details: "Message failed DMARC domain alignment policy".to_string(),
                    severity: "critical".to_string(),
                    artifact_type: "native".to_string(),
                    confidence: Some("high".to_string()),
                    email_id: eid.clone(),
                    email_subject: subj_opt.clone(),
                    email_from: from_addr.clone(),
                    date_sent_utc: date_opt.clone(),
                });
            }
        }

        // 11. Hidden 1x1 Web Tracking Beacons
        if html.contains("width=\"1\" height=\"1\"") || html.contains("width='1' height='1'") {
            let key = "tracking_pixel".to_string();
            if seen.insert(key) {
                artifacts.push(ForensicTaxonomyArtifact {
                    id: generate_id(),
                    domain_id: "network".to_string(),
                    subcategory_id: "tracking_pixels".to_string(),
                    title: "Hidden 1x1 Web Tracking Beacon".to_string(),
                    primary_value: "1x1 Web Beacon".to_string(),
                    secondary_value: Some(from_addr.clone()),
                    details: "Hidden 1x1 tracking beacon embedded in HTML".to_string(),
                    severity: "medium".to_string(),
                    artifact_type: "native".to_string(),
                    confidence: Some("high".to_string()),
                    email_id: eid.clone(),
                    email_subject: subj_opt.clone(),
                    email_from: from_addr.clone(),
                    date_sent_utc: date_opt.clone(),
                });
            }
        }
    }

    Ok(artifacts)
}
