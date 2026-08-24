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
    pub email_id: String,
    pub email_subject: Option<String>,
    pub email_from: String,
    pub date_sent_utc: Option<String>,
}

/// Luhn algorithm for validating credit card numbers
pub fn luhn_check(num_str: &str) -> bool {
    let digits: Vec<u32> = num_str.chars().filter_map(|c| c.to_digit(10)).collect();
    if digits.len() < 13 || digits.len() > 19 {
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

/// Case Artifacts Summary by Taxonomy Domains (Complete 34-Domain Forensic Taxonomy)
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

    let all_artifacts = extract_all_taxonomy_artifacts(&state, &case_id).await?;

    let domain_defs = [
        ("messages", "Email Messages", "📧"),
        ("people", "People & Identities", "👤"),
        ("contacts", "Contacts & Address Books", "📇"),
        ("threads", "Conversations & Threads", "🧵"),
        ("network", "Network & Infrastructure", "🌐"),
        ("web", "URLs & Web Intelligence", "🔗"),
        ("authentication", "Authentication & Security", "🔐"),
        ("attachments", "Attachments & Files", "📎"),
        ("calendar", "Calendar & Meetings", "📅"),
        ("client", "Email Clients & Devices", "💻"),
        ("containers", "Mailboxes & Containers", "🗂️"),
        ("headers_meta", "Headers & Metadata", "🧬"),
        ("timeline_events", "Timeline Events", "🕐"),
        ("graph_network", "Communication Graph", "🕸️"),
        ("credentials", "Credentials & Secrets", "🔑"),
        ("security_otp", "2FA & Account Recovery", "🛡️"),
        ("financial", "Banking & Financial", "🏦"),
        ("crypto", "Cryptocurrency & Seeds", "🪙"),
        ("identity_docs", "Identity Documents (SSN/Passports)", "🪪"),
        ("corporate_intel", "Corporate Intelligence", "🏢"),
        ("messaging_apps", "Messaging App Relays", "💬"),
        ("fraud_bec", "Fraud & BEC Wire Demands", "🚨"),
        ("phishing", "Phishing & Social Engineering", "🎣"),
        ("spoofing", "Spoofing & Impersonation", "🎭"),
        ("malware_threats", "Malware & Threats", "🦠"),
        ("remote_access", "Remote Access Tools", "🖥️"),
        ("legal_docs", "Documents & Legal", "📄"),
        ("locations", "Locations, Travel & Addresses", "📍"),
        ("phone_numbers", "Phone Numbers & VoIP", "📞"),
        ("named_entities", "Named Entities", "🧠"),
        ("campaigns", "Campaigns & Bulk Blasts", "🎯"),
        ("anomalies", "Anomalies & Anti-Forensics", "⚠️"),
        ("deleted_recovered", "Deleted & Recovered", "🗑️"),
        ("case_artifacts", "Evidence & Case Integrity", "⚖️"),
        ("dating_romance", "Romance & Dating Scams", "❤️"),
        ("gift_cards", "Gift Card Laundering", "🎁"),
        ("contraband", "Narcotics, Weapons & Violent Crime", "🛑"),
        ("secrets", "Classified, NDA & Secrets", "🤫"),
    ];

    let mut result = Vec::new();

    for (dom_id, dom_name, dom_icon) in &domain_defs {
        let domain_artifacts: Vec<&ForensicTaxonomyArtifact> = all_artifacts.iter().filter(|a| a.domain_id == *dom_id).collect();
        let total_count = domain_artifacts.len();

        let mut sub_map: std::collections::BTreeMap<String, usize> = std::collections::BTreeMap::new();
        for a in &domain_artifacts {
            *sub_map.entry(a.subcategory_id.clone()).or_insert(0) += 1;
        }

        let subcategories = sub_map.into_iter().map(|(k, v)| {
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

        // Fetch evidence containers
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

    // 0. Case Evidence & Containers
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
            email_id: String::new(),
            email_subject: Some("Chain of Custody Hash Seal".to_string()),
            email_from: "Forensic Acquisition Engine".to_string(),
            date_sent_utc: Some(acquired_at),
        });
    }

    // Comprehensive Regex Matchers
    let re_phone = regex::Regex::new(r"(\+?[0-9]{1,4}[\s\-\.]?\(?[0-9]{2,4}\)?[\s\-\.]?[0-9]{3,4}[\s\-\.]?[0-9]{3,5})").ok();
    let re_ip = regex::Regex::new(r"\b([0-9]{1,3}\.[0-9]{1,3}\.[0-9]{1,3}\.[0-9]{1,3})\b").ok();
    let re_url = regex::Regex::new(r"(https?://[^\s<>'\x22]+)").ok();
    let re_auth_url = regex::Regex::new(r"https?://([^:\s/@]+):([^@\s/]+)@([^\s/]+)").ok();
    let re_btc = regex::Regex::new(r"\b([13][a-km-zA-HJ-NP-Z1-9]{25,34}|bc1[a-zA-HJ-NP-Z0-9]{39,59})\b").ok();
    let re_eth = regex::Regex::new(r"\b(0x[a-fA-F0-9]{40})\b").ok();
    let re_tron = regex::Regex::new(r"\b(T[A-Za-z1-9]{33})\b").ok();
    let re_sol = regex::Regex::new(r"\b([1-9A-HJ-NP-Za-km-z]{32,44})\b").ok();
    let re_seed = regex::Regex::new(r"(?i)(?:seed\s*phrase|recovery\s*phrase|mnemonic(?:\s*phrase)?|secret\s*phrase|passphrase|private\s*key)\s*[:=\-]?\s*([a-z\s]{20,200})").ok();
    let re_cred_pair = regex::Regex::new(r"(?i)(?:username|user|login|email|usr|id)\s*[:=]\s*([^\s\r\n,;]{2,50})\s*(?:and\s+|,|;|\n|\r)?\s*(?:password|passwd|pwd|pass|pin)\s*[:=]\s*([^\s\r\n,;]{3,50})").ok();
    let re_pass_standalone = regex::Regex::new(r"(?i)(?:password|passwd|pwd|passcode|secret\s*key|api\s*key|access\s*token|pin\s*code)\s*[:=]\s*([^\s\r\n,;]{3,60})").ok();
    let re_api_keys = regex::Regex::new(r"\b(AKIA[0-9A-Z]{16}|sk_live_[0-9a-zA-Z]{24,40}|ghp_[0-9a-zA-Z]{36}|AIza[0-9A-Za-z\-_]{35}|xox[baprs]-[0-9a-zA-Z]{10,48}|Bearer\s+[A-Za-z0-9\-\._~\+\/]{20,}=*|eyJ[A-Za-z0-9-_=]{15,}\.[A-Za-z0-9-_=]{15,}\.?[A-Za-z0-9-_.+/=]*)\b").ok();
    let re_routing = regex::Regex::new(r"(?i)(?:routing(?:\s*number|\s*#)?|aba(?:\s*#|\s*no)?)\s*[:#=]?\s*(\b(?:0[1-9]|[123][0-9]|6[1-9]|7[0-2]|80)\d{7}\b)").ok();
    let re_swift = regex::Regex::new(r"(?i)(?:swift(?:\s*code|\s*bic)?|bic(?:\s*code)?)\s*[:#=]?\s*(\b[A-Z]{6}[A-Z0-9]{2}(?:[A-Z0-9]{3})?\b)").ok();
    let re_iban = regex::Regex::new(r"(?i)(?:iban)\s*[:#=]?\s*(\b[A-Z]{2}[0-9]{2}[A-Z0-9]{4}[0-9]{7}(?:[A-Z0-9]?){0,16}\b)").ok();
    let re_account = regex::Regex::new(r"(?i)(?:account(?:\s*number|\s*#|s)?|acct(?:\s*#|\s*no)?|acc\s*#?)\s*[:#=]?\s*([0-9]{8,17})\b").ok();
    let re_cc_spaced = regex::Regex::new(r"\b((?:4[0-9]{3}|5[1-5][0-9]{2}|6011|3[47][0-9]{2})[\s\-][0-9]{4}[\s\-][0-9]{4}[\s\-][0-9]{4})\b").ok();
    let re_cc_raw = regex::Regex::new(r"\b(4[0-9]{12}(?:[0-9]{3})?|5[1-5][0-9]{14}|3[47][0-9]{13}|6011[0-9]{12})\b").ok();
    let re_cashtag = regex::Regex::new(r"(\$[a-zA-Z][a-zA-Z0-9_]{1,19})\b").ok();
    let re_weapons = regex::Regex::new(r"(?i)\b(glock|beretta|ar-15|ak-47|kalashnikov|silencer|suppressor|ghost\s*gun|switch|auto\s*sear|ammunition|magazine|firearm|pistol|carbine|smg|shotgun|rifle)\b").ok();
    let re_narcotics = regex::Regex::new(r"(?i)\b(cocaine|coke|heroin|fentanyl|methamphetamine|crystal\s*meth|mdma|ecstasy|oxycodone|percocet|xanax|alprazolam|ketamine|codeine|lean|promethazine|suboxone)\b").ok();
    let re_threats_terror = regex::Regex::new(r"(?i)\b(bomb|explosive|detonator|c4|assassination|hitman|terrorist|jihad|IED|suicide\s*vest|pipe\s*bomb|anthrax|ricin|poison)\b").ok();
    let re_secrets = regex::Regex::new(r"(?i)\b(strictly\s+confidential|top\s+secret|confidential\s+attorney-client|non-disclosure\s+agreement|\bnda\b|internal\s+use\s+only|classified\s+material|proprietary\s+and\s+confidential|do\s+not\s+distribute|restricted\s+leak)\b").ok();
    
    // Identity Documents
    let re_ssn = regex::Regex::new(r"\b([0-9]{3}-[0-9]{2}-[0-9]{4})\b").ok();
    let re_passport = regex::Regex::new(r"(?i)(?:passport(?:\s*#|\s*no|\s*number)?)\s*[:#=]?\s*([A-PR-WYa-pr-wy][0-9]{7,8})\b").ok();
    let re_driver_lic = regex::Regex::new(r"(?i)(?:driver'?s?\s*license|dl\s*#?|license\s*#)\s*[:#=]?\s*([A-Z0-9\-]{6,16})\b").ok();

    // Corporate Intel
    let re_ein = regex::Regex::new(r"(?i)(?:ein|tax\s*id|federal\s*id)\s*[:#=]?\s*([0-9]{2}-[0-9]{7})\b").ok();
    let re_corp_names = regex::Regex::new(r"(?i)\b([A-Z][a-zA-Z0-9\s&,.\-]{2,40}\s+(?:Inc\.?|LLC|Ltd\.?|Corp\.?|Corporation|GmbH|Co\.?|Holdings|Capital|Group|Ventures))\b").ok();

    // Locations, Travel & Addresses
    let re_street_addr = regex::Regex::new(r"\b([0-9]{1,5}\s+[A-Z][a-zA-Z0-9\s.,]{2,30}\s+(?:Street|St\.?|Avenue|Ave\.?|Road|Rd\.?|Boulevard|Blvd\.?|Lane|Ln\.?|Drive|Dr\.?|Way|Court|Ct\.?|Parkway|Pkwy\.?|Suite|Ste\.?|Apt\.?))\b").ok();
    let re_travel = regex::Regex::new(r"(?i)(?:flight(?:\s*#|\s*no)?|booking\s*ref|pnr|reservation\s*code|ticket\s*#)\s*[:#=]?\s*([A-Z0-9]{5,10})\b").ok();
    let re_gps = regex::Regex::new(r"\b(-?[0-9]{1,2}\.[0-9]{4,8}\s*,\s*-?[0-9]{1,3}\.[0-9]{4,8})\b").ok();

    // Process attachments artifacts
    for (att_id, email_id, filename, sha256, mime, size, entropy, risk_flags, subj, from_addr, date_sent) in attachments {
        let cat = classify_attachment_category(&filename, &mime, entropy, risk_flags.as_deref());
        let is_dangerous = cat == "dangerous";
        artifacts.push(ForensicTaxonomyArtifact {
            id: format!("att-{}", att_id),
            domain_id: "attachments".to_string(),
            subcategory_id: cat.clone(),
            title: format!("Attachment: {}", filename),
            primary_value: filename.clone(),
            secondary_value: Some(format!("SHA-256: {}", sha256)),
            details: format!("MIME: {} | Size: {} B | Entropy: {:.2}", mime, size, entropy.unwrap_or(0.0)),
            severity: if is_dangerous { "critical".to_string() } else { "info".to_string() },
            artifact_type: "native".to_string(),
            email_id,
            email_subject: subj,
            email_from: from_addr,
            date_sent_utc: date_sent,
        });
    }

    for (eid, from_addr, from_disp, to_addrs, cc_addrs, _reply_to, subj_opt, body_opt, html_opt, headers_raw_opt, date_opt, _risk, is_del, is_soft_del, folder_opt, msg_id_opt, in_reply_to_opt, ref_opt) in emails {
        let from_lower = from_addr.to_lowercase();
        let disp_lower = from_disp.as_deref().unwrap_or("").to_lowercase();
        let subj = subj_opt.as_deref().unwrap_or("");
        let subj_lower = subj.to_lowercase();
        let body = body_opt.as_deref().unwrap_or("");
        let body_lower = body.to_lowercase();
        let html = html_opt.as_deref().unwrap_or("");
        let headers_raw = headers_raw_opt.as_deref().unwrap_or("");
        let folder = folder_opt.as_deref().unwrap_or("inbox");
        let full_text = format!("{} {}", subj_lower, body_lower);

        // 1. MESSAGES & RECOVERED
        let is_reply = subj_lower.starts_with("re:") || in_reply_to_opt.is_some();
        let is_fwd = subj_lower.starts_with("fwd:") || subj_lower.starts_with("fw:") || full_text.contains("forwarded message");
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
                email_id: eid.clone(),
                email_subject: subj_opt.clone(),
                email_from: from_addr.clone(),
                date_sent_utc: date_opt.clone(),
            });
        }

        artifacts.push(ForensicTaxonomyArtifact {
            id: generate_id(),
            domain_id: "messages".to_string(),
            subcategory_id: folder.to_lowercase(),
            title: format!("Message: {}", if subj.is_empty() { "(No Subject)" } else { subj }),
            primary_value: if subj.is_empty() { "(No Subject)".to_string() } else { subj.to_string() },
            secondary_value: Some(from_addr.clone()),
            details: format!("Folder: {} | Date: {}", folder, date_opt.as_deref().unwrap_or("Unknown")),
            severity: "info".to_string(),
            artifact_type: "native".to_string(),
            email_id: eid.clone(),
            email_subject: subj_opt.clone(),
            email_from: from_addr.clone(),
            date_sent_utc: date_opt.clone(),
        });

        // 2. CONVERSATION THREADS
        if is_reply || is_fwd || ref_opt.is_some() {
            artifacts.push(ForensicTaxonomyArtifact {
                id: generate_id(),
                domain_id: "threads".to_string(),
                subcategory_id: if is_reply { "reply_chain".to_string() } else { "forward_chain".to_string() },
                title: if is_reply { "Conversation Thread Reply (Re:)".to_string() } else { "Forwarded Thread Chain (Fwd:)".to_string() },
                primary_value: subj.to_string(),
                secondary_value: in_reply_to_opt.clone(),
                details: format!("In-Reply-To: {} | References: {}", in_reply_to_opt.as_deref().unwrap_or("None"), ref_opt.as_deref().unwrap_or("None")),
                severity: "info".to_string(),
                artifact_type: "derived".to_string(),
                email_id: eid.clone(),
                email_subject: subj_opt.clone(),
                email_from: from_addr.clone(),
                date_sent_utc: date_opt.clone(),
            });
        }

        // 3. PEOPLE & IDENTITIES
        artifacts.push(ForensicTaxonomyArtifact {
            id: generate_id(),
            domain_id: "people".to_string(),
            subcategory_id: "identities".to_string(),
            title: format!("Email Identity: {}", from_disp.as_deref().unwrap_or(&from_addr)),
            primary_value: from_addr.clone(),
            secondary_value: from_disp.clone(),
            details: format!("Sender Identity | Display Name: {}", from_disp.as_deref().unwrap_or("None")),
            severity: "info".to_string(),
            artifact_type: "native".to_string(),
            email_id: eid.clone(),
            email_subject: subj_opt.clone(),
            email_from: from_addr.clone(),
            date_sent_utc: date_opt.clone(),
        });

        // 4. PHONE NUMBERS
        if let Some(ref re) = re_phone {
            for cap in re.captures_iter(&body) {
                let p = cap[1].trim().to_string();
                if p.len() >= 9 && p.len() <= 22 && !p.contains('@') {
                    artifacts.push(ForensicTaxonomyArtifact {
                        id: generate_id(),
                        domain_id: "phone_numbers".to_string(),
                        subcategory_id: "telephony".to_string(),
                        title: "Extracted Phone Number".to_string(),
                        primary_value: p.clone(),
                        secondary_value: Some(from_addr.clone()),
                        details: format!("Found in message body from {}", from_addr),
                        severity: "medium".to_string(),
                        artifact_type: "native".to_string(),
                        email_id: eid.clone(),
                        email_subject: subj_opt.clone(),
                        email_from: from_addr.clone(),
                        date_sent_utc: date_opt.clone(),
                    });
                    break;
                }
            }
        }

        // 5. CONTACTS & ADDRESS BOOKS (vCard / Signatures)
        let sig_triggers = ["best regards", "kind regards", "sincerely", "thanks & regards", "warm regards"];
        for sig in &sig_triggers {
            if let Some(idx) = body_lower.find(sig) {
                let sig_block: String = body[idx..].chars().take(160).collect();
                artifacts.push(ForensicTaxonomyArtifact {
                    id: generate_id(),
                    domain_id: "contacts".to_string(),
                    subcategory_id: "signatures".to_string(),
                    title: "Email Signature Contact Card".to_string(),
                    primary_value: sig_block.lines().next().unwrap_or("Signature").to_string(),
                    secondary_value: Some(from_addr.clone()),
                    details: sig_block,
                    severity: "info".to_string(),
                    artifact_type: "native".to_string(),
                    email_id: eid.clone(),
                    email_subject: subj_opt.clone(),
                    email_from: from_addr.clone(),
                    date_sent_utc: date_opt.clone(),
                });
                break;
            }
        }

        // 6. CALENDAR & MEETINGS (.ics)
        if headers_raw_opt.as_deref().unwrap_or("").to_lowercase().contains("text/calendar") || full_text.contains("begin:vcalendar") || subj_lower.contains("invitation:") || subj_lower.contains("meeting request") {
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
                email_id: eid.clone(),
                email_subject: subj_opt.clone(),
                email_from: from_addr.clone(),
                date_sent_utc: date_opt.clone(),
            });
        }

        // 7. CREDENTIALS & SECRETS
        if let Some(ref re) = re_cred_pair {
            for cap in re.captures_iter(&body) {
                let user_val = cap[1].trim().to_string();
                let pass_val = cap[2].trim().to_string();
                artifacts.push(ForensicTaxonomyArtifact {
                    id: generate_id(),
                    domain_id: "credentials".to_string(),
                    subcategory_id: "credentials_pair".to_string(),
                    title: "Credential Pair (Username + Password)".to_string(),
                    primary_value: format!("User: {} | Pass: {}", user_val, pass_val),
                    secondary_value: Some(from_addr.clone()),
                    details: format!("Extracted Account Login: User='{}', Pass='{}'", user_val, pass_val),
                    severity: "critical".to_string(),
                    artifact_type: "native".to_string(),
                    email_id: eid.clone(),
                    email_subject: subj_opt.clone(),
                    email_from: from_addr.clone(),
                    date_sent_utc: date_opt.clone(),
                });
            }
        }

        if let Some(ref re) = re_pass_standalone {
            for cap in re.captures_iter(&body) {
                let pass_val = cap[1].trim().to_string();
                if pass_val.len() >= 4 && !pass_val.contains(' ') {
                    artifacts.push(ForensicTaxonomyArtifact {
                        id: generate_id(),
                        domain_id: "credentials".to_string(),
                        subcategory_id: "passwords".to_string(),
                        title: "Extracted Password / Secret".to_string(),
                        primary_value: format!("Password: {}", pass_val),
                        secondary_value: Some(from_addr.clone()),
                        details: format!("Standalone password value: {}", pass_val),
                        severity: "critical".to_string(),
                        artifact_type: "native".to_string(),
                        email_id: eid.clone(),
                        email_subject: subj_opt.clone(),
                        email_from: from_addr.clone(),
                        date_sent_utc: date_opt.clone(),
                    });
                }
            }
        }

        if let Some(ref re) = re_auth_url {
            for cap in re.captures_iter(&body) {
                artifacts.push(ForensicTaxonomyArtifact {
                    id: generate_id(),
                    domain_id: "credentials".to_string(),
                    subcategory_id: "auth_urls".to_string(),
                    title: "URL with Embedded Credentials".to_string(),
                    primary_value: format!("{}:{}@{}", &cap[1], &cap[2], &cap[3]),
                    secondary_value: Some(from_addr.clone()),
                    details: format!("Authenticated URI Target: host={}", &cap[3]),
                    severity: "critical".to_string(),
                    artifact_type: "native".to_string(),
                    email_id: eid.clone(),
                    email_subject: subj_opt.clone(),
                    email_from: from_addr.clone(),
                    date_sent_utc: date_opt.clone(),
                });
            }
        }

        if let Some(ref re) = re_api_keys {
            for cap in re.captures_iter(&body) {
                let token = cap[1].to_string();
                artifacts.push(ForensicTaxonomyArtifact {
                    id: generate_id(),
                    domain_id: "credentials".to_string(),
                    subcategory_id: "api_keys".to_string(),
                    title: "API Key / JWT Bearer Secret".to_string(),
                    primary_value: token.clone(),
                    secondary_value: Some(from_addr.clone()),
                    details: format!("Secret token extracted from message payload: {}", &token[..token.len().min(30)]),
                    severity: "critical".to_string(),
                    artifact_type: "native".to_string(),
                    email_id: eid.clone(),
                    email_subject: subj_opt.clone(),
                    email_from: from_addr.clone(),
                    date_sent_utc: date_opt.clone(),
                });
            }
        }

        // 8. 2FA & ACCOUNT RECOVERY
        if full_text.contains("verification code") || full_text.contains("your otp is") || full_text.contains("security code is") || full_text.contains("one-time password") {
            let mut extracted_token = "2FA / OTP Code".to_string();
            for word in full_text.split_whitespace() {
                let clean = word.trim_matches(|c: char| !c.is_numeric());
                if (clean.len() == 6 || clean.len() == 4 || clean.len() == 8) && clean.chars().all(|c| c.is_numeric()) {
                    extracted_token = format!("OTP: {}", clean);
                    break;
                }
            }
            artifacts.push(ForensicTaxonomyArtifact {
                id: generate_id(),
                domain_id: "security_otp".to_string(),
                subcategory_id: "otp_codes".to_string(),
                title: "Authentication Token / OTP Code".to_string(),
                primary_value: extracted_token,
                secondary_value: Some(from_addr.clone()),
                details: body.chars().take(200).collect(),
                severity: "high".to_string(),
                artifact_type: "native".to_string(),
                email_id: eid.clone(),
                email_subject: subj_opt.clone(),
                email_from: from_addr.clone(),
                date_sent_utc: date_opt.clone(),
            });
        }

        // 9. FINANCIAL, BANKING & CREDIT CARDS
        if let Some(ref re) = re_routing {
            for cap in re.captures_iter(&body) {
                let routing_no = cap[1].trim().to_string();
                artifacts.push(ForensicTaxonomyArtifact {
                    id: generate_id(),
                    domain_id: "financial".to_string(),
                    subcategory_id: "routing_numbers".to_string(),
                    title: "ABA Bank Routing Number".to_string(),
                    primary_value: format!("Routing #: {}", routing_no),
                    secondary_value: Some(from_addr.clone()),
                    details: format!("US 9-digit ABA Bank Routing Number: {}", routing_no),
                    severity: "critical".to_string(),
                    artifact_type: "native".to_string(),
                    email_id: eid.clone(),
                    email_subject: subj_opt.clone(),
                    email_from: from_addr.clone(),
                    date_sent_utc: date_opt.clone(),
                });
            }
        }

        if let Some(ref re) = re_swift {
            for cap in re.captures_iter(&body) {
                let swift_code = cap[1].trim().to_string();
                artifacts.push(ForensicTaxonomyArtifact {
                    id: generate_id(),
                    domain_id: "financial".to_string(),
                    subcategory_id: "swift_bic".to_string(),
                    title: "SWIFT / BIC Bank Identifier".to_string(),
                    primary_value: format!("SWIFT: {}", swift_code),
                    secondary_value: Some(from_addr.clone()),
                    details: format!("International Bank SWIFT/BIC Code: {}", swift_code),
                    severity: "high".to_string(),
                    artifact_type: "native".to_string(),
                    email_id: eid.clone(),
                    email_subject: subj_opt.clone(),
                    email_from: from_addr.clone(),
                    date_sent_utc: date_opt.clone(),
                });
            }
        }

        if let Some(ref re) = re_iban {
            for cap in re.captures_iter(&body) {
                let iban = cap[1].trim().to_string();
                artifacts.push(ForensicTaxonomyArtifact {
                    id: generate_id(),
                    domain_id: "financial".to_string(),
                    subcategory_id: "iban".to_string(),
                    title: "IBAN International Account Number".to_string(),
                    primary_value: format!("IBAN: {}", iban),
                    secondary_value: Some(from_addr.clone()),
                    details: format!("International Bank Account Number (IBAN): {}", iban),
                    severity: "critical".to_string(),
                    artifact_type: "native".to_string(),
                    email_id: eid.clone(),
                    email_subject: subj_opt.clone(),
                    email_from: from_addr.clone(),
                    date_sent_utc: date_opt.clone(),
                });
            }
        }

        if let Some(ref re) = re_account {
            for cap in re.captures_iter(&body) {
                let acc_no = cap[1].trim().to_string();
                artifacts.push(ForensicTaxonomyArtifact {
                    id: generate_id(),
                    domain_id: "financial".to_string(),
                    subcategory_id: "account_numbers".to_string(),
                    title: "Bank Account Number".to_string(),
                    primary_value: format!("Account #: {}", acc_no),
                    secondary_value: Some(from_addr.clone()),
                    details: format!("Extracted Financial Account Number: {}", acc_no),
                    severity: "critical".to_string(),
                    artifact_type: "native".to_string(),
                    email_id: eid.clone(),
                    email_subject: subj_opt.clone(),
                    email_from: from_addr.clone(),
                    date_sent_utc: date_opt.clone(),
                });
            }
        }

        // Credit Cards (Luhn validated)
        if let Some(ref re) = re_cc_spaced {
            for cap in re.captures_iter(&body) {
                let cc_raw = cap[1].replace([' ', '-'], "");
                if luhn_check(&cc_raw) {
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
                        email_id: eid.clone(),
                        email_subject: subj_opt.clone(),
                        email_from: from_addr.clone(),
                        date_sent_utc: date_opt.clone(),
                    });
                }
            }
        }

        if let Some(ref re) = re_cc_raw {
            for cap in re.captures_iter(&body) {
                let cc_raw = cap[1].to_string();
                if luhn_check(&cc_raw) {
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
                        email_id: eid.clone(),
                        email_subject: subj_opt.clone(),
                        email_from: from_addr.clone(),
                        date_sent_utc: date_opt.clone(),
                    });
                }
            }
        }

        if let Some(ref re) = re_cashtag {
            for cap in re.captures_iter(&body) {
                artifacts.push(ForensicTaxonomyArtifact {
                    id: generate_id(),
                    domain_id: "financial".to_string(),
                    subcategory_id: "neobanks".to_string(),
                    title: "CashApp Cashtag Handle".to_string(),
                    primary_value: cap[1].to_string(),
                    secondary_value: Some(from_addr.clone()),
                    details: format!("CashApp Payment Tag: {}", &cap[1]),
                    severity: "high".to_string(),
                    artifact_type: "native".to_string(),
                    email_id: eid.clone(),
                    email_subject: subj_opt.clone(),
                    email_from: from_addr.clone(),
                    date_sent_utc: date_opt.clone(),
                });
            }
        }

        // 10. CRYPTO & SEED PHRASES
        if let Some(ref re) = re_btc {
            for cap in re.captures_iter(&body) {
                artifacts.push(ForensicTaxonomyArtifact {
                    id: generate_id(),
                    domain_id: "crypto".to_string(),
                    subcategory_id: "bitcoin".to_string(),
                    title: "Bitcoin Wallet Address".to_string(),
                    primary_value: cap[1].to_string(),
                    secondary_value: Some(from_addr.clone()),
                    details: format!("Bitcoin (BTC) Public Address: {}", &cap[1]),
                    severity: "high".to_string(),
                    artifact_type: "native".to_string(),
                    email_id: eid.clone(),
                    email_subject: subj_opt.clone(),
                    email_from: from_addr.clone(),
                    date_sent_utc: date_opt.clone(),
                });
            }
        }

        if let Some(ref re) = re_eth {
            for cap in re.captures_iter(&body) {
                artifacts.push(ForensicTaxonomyArtifact {
                    id: generate_id(),
                    domain_id: "crypto".to_string(),
                    subcategory_id: "ethereum".to_string(),
                    title: "Ethereum / ERC-20 Wallet Address".to_string(),
                    primary_value: cap[1].to_string(),
                    secondary_value: Some(from_addr.clone()),
                    details: format!("Ethereum / EVM Address: {}", &cap[1]),
                    severity: "high".to_string(),
                    artifact_type: "native".to_string(),
                    email_id: eid.clone(),
                    email_subject: subj_opt.clone(),
                    email_from: from_addr.clone(),
                    date_sent_utc: date_opt.clone(),
                });
            }
        }

        if let Some(ref re) = re_tron {
            for cap in re.captures_iter(&body) {
                artifacts.push(ForensicTaxonomyArtifact {
                    id: generate_id(),
                    domain_id: "crypto".to_string(),
                    subcategory_id: "usdt_tron".to_string(),
                    title: "TRON / USDT TRC-20 Wallet Address".to_string(),
                    primary_value: cap[1].to_string(),
                    secondary_value: Some(from_addr.clone()),
                    details: format!("TRON USDT TRC-20 Address: {}", &cap[1]),
                    severity: "high".to_string(),
                    artifact_type: "native".to_string(),
                    email_id: eid.clone(),
                    email_subject: subj_opt.clone(),
                    email_from: from_addr.clone(),
                    date_sent_utc: date_opt.clone(),
                });
            }
        }

        if let Some(ref re) = re_sol {
            for cap in re.captures_iter(&body) {
                let sol_addr = cap[1].to_string();
                if sol_addr.len() >= 32 && sol_addr.len() <= 44 && !sol_addr.contains('@') {
                    artifacts.push(ForensicTaxonomyArtifact {
                        id: generate_id(),
                        domain_id: "crypto".to_string(),
                        subcategory_id: "solana".to_string(),
                        title: "Solana Wallet Address".to_string(),
                        primary_value: sol_addr.clone(),
                        secondary_value: Some(from_addr.clone()),
                        details: format!("Solana (SOL) Address: {}", sol_addr),
                        severity: "high".to_string(),
                        artifact_type: "native".to_string(),
                        email_id: eid.clone(),
                        email_subject: subj_opt.clone(),
                        email_from: from_addr.clone(),
                        date_sent_utc: date_opt.clone(),
                    });
                    break;
                }
            }
        }

        if let Some(ref re) = re_seed {
            for cap in re.captures_iter(&body) {
                let seed_phrase = cap[1].trim().to_string();
                artifacts.push(ForensicTaxonomyArtifact {
                    id: generate_id(),
                    domain_id: "crypto".to_string(),
                    subcategory_id: "seed_phrases".to_string(),
                    title: "BIP-39 Crypto Seed Phrase / Recovery Mnemonic".to_string(),
                    primary_value: seed_phrase.clone(),
                    secondary_value: Some(from_addr.clone()),
                    details: format!("Extracted crypto wallet recovery phrase: {}", seed_phrase),
                    severity: "critical".to_string(),
                    artifact_type: "native".to_string(),
                    email_id: eid.clone(),
                    email_subject: subj_opt.clone(),
                    email_from: from_addr.clone(),
                    date_sent_utc: date_opt.clone(),
                });
            }
        }

        // 11. IDENTITY DOCUMENTS (SSN, Passport, Driver's License)
        if let Some(ref re) = re_ssn {
            for cap in re.captures_iter(&body) {
                artifacts.push(ForensicTaxonomyArtifact {
                    id: generate_id(),
                    domain_id: "identity_docs".to_string(),
                    subcategory_id: "ssn".to_string(),
                    title: "Social Security Number (SSN)".to_string(),
                    primary_value: cap[1].to_string(),
                    secondary_value: Some(from_addr.clone()),
                    details: format!("Extracted US Social Security Number: {}", &cap[1]),
                    severity: "critical".to_string(),
                    artifact_type: "native".to_string(),
                    email_id: eid.clone(),
                    email_subject: subj_opt.clone(),
                    email_from: from_addr.clone(),
                    date_sent_utc: date_opt.clone(),
                });
            }
        }

        if let Some(ref re) = re_passport {
            for cap in re.captures_iter(&body) {
                artifacts.push(ForensicTaxonomyArtifact {
                    id: generate_id(),
                    domain_id: "identity_docs".to_string(),
                    subcategory_id: "passports".to_string(),
                    title: "Passport Number Indicator".to_string(),
                    primary_value: cap[1].to_string(),
                    secondary_value: Some(from_addr.clone()),
                    details: format!("Passport document identifier: {}", &cap[1]),
                    severity: "high".to_string(),
                    artifact_type: "native".to_string(),
                    email_id: eid.clone(),
                    email_subject: subj_opt.clone(),
                    email_from: from_addr.clone(),
                    date_sent_utc: date_opt.clone(),
                });
            }
        }

        if let Some(ref re) = re_driver_lic {
            for cap in re.captures_iter(&body) {
                artifacts.push(ForensicTaxonomyArtifact {
                    id: generate_id(),
                    domain_id: "identity_docs".to_string(),
                    subcategory_id: "driver_license".to_string(),
                    title: "Driver's License Identifier".to_string(),
                    primary_value: cap[1].to_string(),
                    secondary_value: Some(from_addr.clone()),
                    details: format!("Driver's License number: {}", &cap[1]),
                    severity: "high".to_string(),
                    artifact_type: "native".to_string(),
                    email_id: eid.clone(),
                    email_subject: subj_opt.clone(),
                    email_from: from_addr.clone(),
                    date_sent_utc: date_opt.clone(),
                });
            }
        }

        // 12. CORPORATE INTELLIGENCE (EIN, Corporate Entities)
        if let Some(ref re) = re_ein {
            for cap in re.captures_iter(&body) {
                artifacts.push(ForensicTaxonomyArtifact {
                    id: generate_id(),
                    domain_id: "corporate_intel".to_string(),
                    subcategory_id: "ein_tax_id".to_string(),
                    title: "Federal Tax ID / EIN".to_string(),
                    primary_value: format!("EIN: {}", &cap[1]),
                    secondary_value: Some(from_addr.clone()),
                    details: format!("Employer Identification Number (EIN): {}", &cap[1]),
                    severity: "high".to_string(),
                    artifact_type: "native".to_string(),
                    email_id: eid.clone(),
                    email_subject: subj_opt.clone(),
                    email_from: from_addr.clone(),
                    date_sent_utc: date_opt.clone(),
                });
            }
        }

        if let Some(ref re) = re_corp_names {
            for cap in re.captures_iter(&body) {
                let corp_name = cap[1].trim().to_string();
                artifacts.push(ForensicTaxonomyArtifact {
                    id: generate_id(),
                    domain_id: "corporate_intel".to_string(),
                    subcategory_id: "companies".to_string(),
                    title: format!("Corporate Entity: {}", corp_name),
                    primary_value: corp_name.clone(),
                    secondary_value: Some(from_addr.clone()),
                    details: format!("Identified business corporation: {}", corp_name),
                    severity: "info".to_string(),
                    artifact_type: "native".to_string(),
                    email_id: eid.clone(),
                    email_subject: subj_opt.clone(),
                    email_from: from_addr.clone(),
                    date_sent_utc: date_opt.clone(),
                });
                break;
            }
        }

        // 13. LOCATIONS, TRAVEL & ADDRESSES
        if let Some(ref re) = re_street_addr {
            for cap in re.captures_iter(&body) {
                let addr = cap[1].trim().to_string();
                artifacts.push(ForensicTaxonomyArtifact {
                    id: generate_id(),
                    domain_id: "locations".to_string(),
                    subcategory_id: "physical_addresses".to_string(),
                    title: "Physical Street Address".to_string(),
                    primary_value: addr.clone(),
                    secondary_value: Some(from_addr.clone()),
                    details: format!("Street address identified in message: {}", addr),
                    severity: "medium".to_string(),
                    artifact_type: "native".to_string(),
                    email_id: eid.clone(),
                    email_subject: subj_opt.clone(),
                    email_from: from_addr.clone(),
                    date_sent_utc: date_opt.clone(),
                });
                break;
            }
        }

        if let Some(ref re) = re_travel {
            for cap in re.captures_iter(&body) {
                let pnr = cap[1].trim().to_string();
                artifacts.push(ForensicTaxonomyArtifact {
                    id: generate_id(),
                    domain_id: "locations".to_string(),
                    subcategory_id: "travel_bookings".to_string(),
                    title: "Flight / Travel Booking PNR".to_string(),
                    primary_value: format!("PNR: {}", pnr),
                    secondary_value: Some(from_addr.clone()),
                    details: format!("Travel airline reservation or booking code: {}", pnr),
                    severity: "high".to_string(),
                    artifact_type: "native".to_string(),
                    email_id: eid.clone(),
                    email_subject: subj_opt.clone(),
                    email_from: from_addr.clone(),
                    date_sent_utc: date_opt.clone(),
                });
            }
        }

        if let Some(ref re) = re_gps {
            for cap in re.captures_iter(&body) {
                artifacts.push(ForensicTaxonomyArtifact {
                    id: generate_id(),
                    domain_id: "locations".to_string(),
                    subcategory_id: "gps_coordinates".to_string(),
                    title: "GPS Geographic Coordinates".to_string(),
                    primary_value: cap[1].to_string(),
                    secondary_value: Some(from_addr.clone()),
                    details: format!("Latitude / Longitude coordinates: {}", &cap[1]),
                    severity: "high".to_string(),
                    artifact_type: "native".to_string(),
                    email_id: eid.clone(),
                    email_subject: subj_opt.clone(),
                    email_from: from_addr.clone(),
                    date_sent_utc: date_opt.clone(),
                });
            }
        }

        // 14. FRAUD & BEC WIRE DEMANDS
        if full_text.contains("wire transfer immediately") || full_text.contains("urgent payment") || full_text.contains("send gift card") || full_text.contains("compromised account") || full_text.contains("direct deposit form") {
            artifacts.push(ForensicTaxonomyArtifact {
                id: generate_id(),
                domain_id: "fraud_bec".to_string(),
                subcategory_id: "wire_fraud".to_string(),
                title: "BEC Wire Fraud / Urgent Extortion".to_string(),
                primary_value: "Urgent Wire Payment Demand".to_string(),
                secondary_value: Some(from_addr.clone()),
                details: body.chars().take(240).collect(),
                severity: "critical".to_string(),
                artifact_type: "derived".to_string(),
                email_id: eid.clone(),
                email_subject: subj_opt.clone(),
                email_from: from_addr.clone(),
                date_sent_utc: date_opt.clone(),
            });
        }

        // 15. SPOOFING & IMPERSONATION
        if let Some(ref disp) = from_disp {
            if (disp.to_lowercase().contains("ceo") || disp.to_lowercase().contains("director") || disp.to_lowercase().contains("executive") || disp.to_lowercase().contains("president")) && (from_lower.contains("gmail.com") || from_lower.contains("yahoo.com") || from_lower.contains("hotmail.com")) {
                artifacts.push(ForensicTaxonomyArtifact {
                    id: generate_id(),
                    domain_id: "spoofing".to_string(),
                    subcategory_id: "display_name_spoof".to_string(),
                    title: "Executive Display Name Impersonation".to_string(),
                    primary_value: format!("{} <{}>", disp, from_addr),
                    secondary_value: Some(from_addr.clone()),
                    details: "VIP Executive display name paired with freemail address (Classic BEC tactic)".to_string(),
                    severity: "critical".to_string(),
                    artifact_type: "derived".to_string(),
                    email_id: eid.clone(),
                    email_subject: subj_opt.clone(),
                    email_from: from_addr.clone(),
                    date_sent_utc: date_opt.clone(),
                });
            }
        }

        // 16. PHISHING & SOCIAL ENGINEERING
        if full_text.contains("verify your account") || full_text.contains("password expires") || full_text.contains("suspended account") || full_text.contains("click here to unlock") {
            artifacts.push(ForensicTaxonomyArtifact {
                id: generate_id(),
                domain_id: "phishing".to_string(),
                subcategory_id: "credential_harvesting".to_string(),
                title: "Phishing Social Engineering Lure".to_string(),
                primary_value: "Urgent Account Verification Hook".to_string(),
                secondary_value: Some(from_addr.clone()),
                details: body.chars().take(200).collect(),
                severity: "critical".to_string(),
                artifact_type: "derived".to_string(),
                email_id: eid.clone(),
                email_subject: subj_opt.clone(),
                email_from: from_addr.clone(),
                date_sent_utc: date_opt.clone(),
            });
        }

        // 17. MALWARE & THREATS (Office Macros / Dangerous Scripts)
        if full_text.contains(".docm") || full_text.contains(".xlsm") || full_text.contains("enable macro") || full_text.contains("powershell -enc") {
            artifacts.push(ForensicTaxonomyArtifact {
                id: generate_id(),
                domain_id: "malware_threats".to_string(),
                subcategory_id: "macro_payloads".to_string(),
                title: "VBA Macro Payload / PowerShell Threat".to_string(),
                primary_value: "VBA Macro Execution Indicator".to_string(),
                secondary_value: Some(from_addr.clone()),
                details: "Email references macro-enabled document or encoded PowerShell payload".to_string(),
                severity: "critical".to_string(),
                artifact_type: "derived".to_string(),
                email_id: eid.clone(),
                email_subject: subj_opt.clone(),
                email_from: from_addr.clone(),
                date_sent_utc: date_opt.clone(),
            });
        }

        // 18. CONTRABAND: WEAPONS, NARCOTICS & THREATS
        if let Some(ref re) = re_weapons {
            for cap in re.captures_iter(&body) {
                let weapon = cap[1].to_string();
                artifacts.push(ForensicTaxonomyArtifact {
                    id: generate_id(),
                    domain_id: "contraband".to_string(),
                    subcategory_id: "weapons_firearms".to_string(),
                    title: format!("Firearms / Weapons Indicator: {}", weapon),
                    primary_value: weapon.to_uppercase(),
                    secondary_value: Some(from_addr.clone()),
                    details: format!("Firearm or weapon keyword matched in context: {}", body.chars().take(160).collect::<String>()),
                    severity: "critical".to_string(),
                    artifact_type: "native".to_string(),
                    email_id: eid.clone(),
                    email_subject: subj_opt.clone(),
                    email_from: from_addr.clone(),
                    date_sent_utc: date_opt.clone(),
                });
                break;
            }
        }

        if let Some(ref re) = re_narcotics {
            for cap in re.captures_iter(&body) {
                let drug = cap[1].to_string();
                artifacts.push(ForensicTaxonomyArtifact {
                    id: generate_id(),
                    domain_id: "contraband".to_string(),
                    subcategory_id: "narcotics_drugs".to_string(),
                    title: format!("Narcotics / Controlled Substance: {}", drug),
                    primary_value: drug.to_uppercase(),
                    secondary_value: Some(from_addr.clone()),
                    details: format!("Illicit drug or controlled pharmaceutical mention: {}", drug),
                    severity: "critical".to_string(),
                    artifact_type: "native".to_string(),
                    email_id: eid.clone(),
                    email_subject: subj_opt.clone(),
                    email_from: from_addr.clone(),
                    date_sent_utc: date_opt.clone(),
                });
                break;
            }
        }

        if let Some(ref re) = re_threats_terror {
            for cap in re.captures_iter(&body) {
                let threat = cap[1].to_string();
                artifacts.push(ForensicTaxonomyArtifact {
                    id: generate_id(),
                    domain_id: "contraband".to_string(),
                    subcategory_id: "terrorism_threats".to_string(),
                    title: format!("Violent Crime / Explosives Threat: {}", threat),
                    primary_value: threat.to_uppercase(),
                    secondary_value: Some(from_addr.clone()),
                    details: format!("Violent extremism or explosives keyword: {}", threat),
                    severity: "critical".to_string(),
                    artifact_type: "native".to_string(),
                    email_id: eid.clone(),
                    email_subject: subj_opt.clone(),
                    email_from: from_addr.clone(),
                    date_sent_utc: date_opt.clone(),
                });
                break;
            }
        }

        // 19. CLASSIFIED, NDA & CORPORATE SECRETS
        if let Some(ref re) = re_secrets {
            for cap in re.captures_iter(&body) {
                let secret_tag = cap[1].to_string();
                artifacts.push(ForensicTaxonomyArtifact {
                    id: generate_id(),
                    domain_id: "secrets".to_string(),
                    subcategory_id: "classified_leaks".to_string(),
                    title: format!("Confidential / Secret Indicator: {}", secret_tag),
                    primary_value: secret_tag.to_uppercase(),
                    secondary_value: Some(from_addr.clone()),
                    details: format!("Sensitive disclosure header or confidentiality marking: {}", secret_tag),
                    severity: "high".to_string(),
                    artifact_type: "native".to_string(),
                    email_id: eid.clone(),
                    email_subject: subj_opt.clone(),
                    email_from: from_addr.clone(),
                    date_sent_utc: date_opt.clone(),
                });
                break;
            }
        }

        // 20. NETWORK & INFRASTRUCTURE
        if let Some(ref re) = re_ip {
            for cap in re.captures_iter(headers_raw) {
                let ip = cap[1].to_string();
                if !ip.starts_with("127.") && !ip.starts_with("0.") && !ip.starts_with("255.") && !ip.starts_with("10.") && !ip.starts_with("192.168.") {
                    artifacts.push(ForensicTaxonomyArtifact {
                        id: generate_id(),
                        domain_id: "network".to_string(),
                        subcategory_id: "ip_addresses".to_string(),
                        title: "Relay / Originating IP Address".to_string(),
                        primary_value: ip.clone(),
                        secondary_value: Some(from_addr.clone()),
                        details: format!("Extracted from headers of email '{}'", subj),
                        severity: "medium".to_string(),
                        artifact_type: "native".to_string(),
                        email_id: eid.clone(),
                        email_subject: subj_opt.clone(),
                        email_from: from_addr.clone(),
                        date_sent_utc: date_opt.clone(),
                    });
                    break;
                }
            }
        }

        // 21. AUTHENTICATION PROOFS
        let headers_lower = headers_raw.to_lowercase();
        if headers_lower.contains("spf=pass") || headers_lower.contains("spf=fail") || headers_lower.contains("received-spf") {
            let res = if headers_lower.contains("spf=pass") { "PASS" } else if headers_lower.contains("spf=fail") { "FAIL" } else { "NEUTRAL" };
            artifacts.push(ForensicTaxonomyArtifact {
                id: generate_id(),
                domain_id: "authentication".to_string(),
                subcategory_id: "spf".to_string(),
                title: format!("SPF Authentication Result: {}", res),
                primary_value: format!("SPF: {}", res),
                secondary_value: Some(from_addr.clone()),
                details: format!("Sender Domain: {}", from_lower.split('@').nth(1).unwrap_or("")),
                severity: if res == "FAIL" { "critical".to_string() } else { "info".to_string() },
                artifact_type: "native".to_string(),
                email_id: eid.clone(),
                email_subject: subj_opt.clone(),
                email_from: from_addr.clone(),
                date_sent_utc: date_opt.clone(),
            });
        }

        if headers_lower.contains("dkim=pass") || headers_lower.contains("dkim=fail") || headers_lower.contains("dkim-signature") {
            let res = if headers_lower.contains("dkim=pass") { "PASS" } else if headers_lower.contains("dkim=fail") { "FAIL" } else { "PRESENT" };
            artifacts.push(ForensicTaxonomyArtifact {
                id: generate_id(),
                domain_id: "authentication".to_string(),
                subcategory_id: "dkim".to_string(),
                title: format!("DKIM Signature Verification: {}", res),
                primary_value: format!("DKIM: {}", res),
                secondary_value: Some(from_addr.clone()),
                details: "Cryptographic signature header".to_string(),
                severity: if res == "FAIL" { "critical".to_string() } else { "info".to_string() },
                artifact_type: "native".to_string(),
                email_id: eid.clone(),
                email_subject: subj_opt.clone(),
                email_from: from_addr.clone(),
                date_sent_utc: date_opt.clone(),
            });
        }

        // 22. WEB & HYPERLINKS
        if let Some(ref re) = re_url {
            let mut url_count = 0;
            for cap in re.captures_iter(&body) {
                let u = cap[1].to_string();
                artifacts.push(ForensicTaxonomyArtifact {
                    id: generate_id(),
                    domain_id: "web".to_string(),
                    subcategory_id: "urls".to_string(),
                    title: "Hyperlink / URL Indicator".to_string(),
                    primary_value: u.clone(),
                    secondary_value: Some(from_addr.clone()),
                    details: format!("Target URL extracted from message body: {}", u),
                    severity: if u.contains("login") || u.contains("verify") || u.contains("secure") { "high".to_string() } else { "info".to_string() },
                    artifact_type: "native".to_string(),
                    email_id: eid.clone(),
                    email_subject: subj_opt.clone(),
                    email_from: from_addr.clone(),
                    date_sent_utc: date_opt.clone(),
                });
                url_count += 1;
                if url_count >= 5 { break; }
            }
        }

        // Tracking Pixels (1x1 images)
        if html.contains("width=\"1\" height=\"1\"") || html.contains("width='1' height='1'") || html.contains("display:none") {
            artifacts.push(ForensicTaxonomyArtifact {
                id: generate_id(),
                domain_id: "web".to_string(),
                subcategory_id: "tracking_pixels".to_string(),
                title: "Tracking Pixel / Hidden Web Beacon".to_string(),
                primary_value: "1x1 Tracking Pixel / Beacon".to_string(),
                secondary_value: Some(from_addr.clone()),
                details: "Email contains hidden tracking image to log recipient open event & IP address".to_string(),
                severity: "medium".to_string(),
                artifact_type: "native".to_string(),
                email_id: eid.clone(),
                email_subject: subj_opt.clone(),
                email_from: from_addr.clone(),
                date_sent_utc: date_opt.clone(),
            });
        }

        // 23. CLIENT & MAILER SOFTWARE
        let mut client_found: Option<&str> = None;
        if headers_lower.contains("microsoft outlook") || headers_lower.contains("x-mailer: microsoft") {
            client_found = Some("Microsoft Outlook");
        } else if headers_lower.contains("apple mail") || headers_lower.contains("mac os x mail") {
            client_found = Some("Apple Mail");
        } else if headers_lower.contains("thunderbird") {
            client_found = Some("Mozilla Thunderbird");
        } else if headers_lower.contains("iphone mail") || headers_lower.contains("ipad mail") {
            client_found = Some("iOS Mail (iPhone/iPad)");
        } else if headers_lower.contains("sendgrid") {
            client_found = Some("SendGrid Mail Relay");
        } else if headers_lower.contains("mailgun") {
            client_found = Some("Mailgun Cloud Mailer");
        } else if headers_lower.contains("exchange server") || headers_lower.contains("x-ms-exchange") {
            client_found = Some("Microsoft Exchange Server");
        }

        if let Some(client_name) = client_found {
            artifacts.push(ForensicTaxonomyArtifact {
                id: generate_id(),
                domain_id: "client".to_string(),
                subcategory_id: "clients".to_string(),
                title: format!("Email Client / Mailer: {}", client_name),
                primary_value: client_name.to_string(),
                secondary_value: Some(from_addr.clone()),
                details: format!("Identified from X-Mailer / User-Agent headers on email '{}'", subj),
                severity: "info".to_string(),
                artifact_type: "native".to_string(),
                email_id: eid.clone(),
                email_subject: subj_opt.clone(),
                email_from: from_addr.clone(),
                date_sent_utc: date_opt.clone(),
            });
        }

        // 24. MESSAGING APPS
        if from_lower.contains("voice.google.com") || from_lower.contains("voice-noreply@google.com") || full_text.contains("google voice") {
            let mut phone = "Google Voice Relay".to_string();
            if let Some(idx) = subj.find("from (") {
                phone = subj[idx + 5..].trim().to_string();
            }
            artifacts.push(ForensicTaxonomyArtifact {
                id: generate_id(),
                domain_id: "messaging_apps".to_string(),
                subcategory_id: "google_voice".to_string(),
                title: "Google Voice SMS / Call Transcript".to_string(),
                primary_value: phone,
                secondary_value: Some(from_addr.clone()),
                details: body.chars().take(200).collect(),
                severity: "high".to_string(),
                artifact_type: "native".to_string(),
                email_id: eid.clone(),
                email_subject: subj_opt.clone(),
                email_from: from_addr.clone(),
                date_sent_utc: date_opt.clone(),
            });
        }

        if from_lower.contains("textnow") || full_text.contains("textnow") || from_lower.contains("pinger") {
            artifacts.push(ForensicTaxonomyArtifact {
                id: generate_id(),
                domain_id: "messaging_apps".to_string(),
                subcategory_id: "burner_voip".to_string(),
                title: "TextNow / Burner Virtual SMS Activity".to_string(),
                primary_value: "Burner VoIP SMS".to_string(),
                secondary_value: Some(from_addr.clone()),
                details: body.chars().take(200).collect(),
                severity: "high".to_string(),
                artifact_type: "native".to_string(),
                email_id: eid.clone(),
                email_subject: subj_opt.clone(),
                email_from: from_addr.clone(),
                date_sent_utc: date_opt.clone(),
            });
        }

        if from_lower.contains("whatsapp") || full_text.contains("whatsapp web") {
            artifacts.push(ForensicTaxonomyArtifact {
                id: generate_id(),
                domain_id: "messaging_apps".to_string(),
                subcategory_id: "whatsapp".to_string(),
                title: "WhatsApp Messenger Notification / Web Session".to_string(),
                primary_value: "WhatsApp Notification".to_string(),
                secondary_value: Some(from_addr.clone()),
                details: body.chars().take(200).collect(),
                severity: "medium".to_string(),
                artifact_type: "native".to_string(),
                email_id: eid.clone(),
                email_subject: subj_opt.clone(),
                email_from: from_addr.clone(),
                date_sent_utc: date_opt.clone(),
            });
        }

        // 25. REMOTE ACCESS TOOLS
        let rat_tools = [("anydesk", "AnyDesk"), ("teamviewer", "TeamViewer"), ("rustdesk", "RustDesk")];
        for (rkey, rlabel) in &rat_tools {
            if from_lower.contains(rkey) || full_text.contains(rkey) {
                artifacts.push(ForensicTaxonomyArtifact {
                    id: generate_id(),
                    domain_id: "remote_access".to_string(),
                    subcategory_id: "remote_sessions".to_string(),
                    title: format!("Remote Access Session ({})", rlabel),
                    primary_value: rlabel.to_string(),
                    secondary_value: Some(from_addr.clone()),
                    details: body.chars().take(200).collect(),
                    severity: "critical".to_string(),
                    artifact_type: "native".to_string(),
                    email_id: eid.clone(),
                    email_subject: subj_opt.clone(),
                    email_from: from_addr.clone(),
                    date_sent_utc: date_opt.clone(),
                });
                break;
            }
        }

        // 26. DATING & ROMANCE
        let dating_apps = ["tinder", "match.com", "bumble", "zoosk", "pof.com", "christianmingle", "okcupid"];
        for d in &dating_apps {
            if from_lower.contains(d) || full_text.contains(d) {
                artifacts.push(ForensicTaxonomyArtifact {
                    id: generate_id(),
                    domain_id: "dating_romance".to_string(),
                    subcategory_id: "dating_profiles".to_string(),
                    title: format!("Dating Profile Activity ({})", d),
                    primary_value: format!("Dating App: {}", d),
                    secondary_value: Some(from_addr.clone()),
                    details: body.chars().take(200).collect(),
                    severity: "high".to_string(),
                    artifact_type: "native".to_string(),
                    email_id: eid.clone(),
                    email_subject: subj_opt.clone(),
                    email_from: from_addr.clone(),
                    date_sent_utc: date_opt.clone(),
                });
                break;
            }
        }

        // 27. GIFT CARDS
        let gift_cards = ["apple gift card", "itunes gift card", "steam card", "amazon gift card", "google play card", "razer gold"];
        for gc in &gift_cards {
            if full_text.contains(gc) {
                artifacts.push(ForensicTaxonomyArtifact {
                    id: generate_id(),
                    domain_id: "gift_cards".to_string(),
                    subcategory_id: "gift_card_codes".to_string(),
                    title: format!("Gift Card / Voucher Code ({})", gc),
                    primary_value: format!("Gift Card: {}", gc),
                    secondary_value: Some(from_addr.clone()),
                    details: body.chars().take(200).collect(),
                    severity: "critical".to_string(),
                    artifact_type: "native".to_string(),
                    email_id: eid.clone(),
                    email_subject: subj_opt.clone(),
                    email_from: from_addr.clone(),
                    date_sent_utc: date_opt.clone(),
                });
                break;
            }
        }

        // 28. HEADERS & METADATA
        if headers_lower.contains("x-originating-ip") || headers_lower.contains("x-mailer") {
            artifacts.push(ForensicTaxonomyArtifact {
                id: generate_id(),
                domain_id: "headers_meta".to_string(),
                subcategory_id: "x_headers".to_string(),
                title: "Forensic Transport X-Header".to_string(),
                primary_value: format!("Email: {}", if subj.is_empty() { "(No Subject)" } else { subj }),
                secondary_value: Some(from_addr.clone()),
                details: headers_raw.lines().filter(|l| l.to_lowercase().starts_with("x-")).take(5).collect::<Vec<_>>().join("\n"),
                severity: "info".to_string(),
                artifact_type: "native".to_string(),
                email_id: eid.clone(),
                email_subject: subj_opt.clone(),
                email_from: from_addr.clone(),
                date_sent_utc: date_opt.clone(),
            });
        }

        // 29. TIMELINE EVENTS
        if let Some(ref d) = date_opt {
            artifacts.push(ForensicTaxonomyArtifact {
                id: generate_id(),
                domain_id: "timeline_events".to_string(),
                subcategory_id: "message_timestamp".to_string(),
                title: "Email Dispatch / Receipt Timestamp".to_string(),
                primary_value: d.clone(),
                secondary_value: Some(from_addr.clone()),
                details: format!("Message event logged at UTC timestamp: {}", d),
                severity: "info".to_string(),
                artifact_type: "native".to_string(),
                email_id: eid.clone(),
                email_subject: subj_opt.clone(),
                email_from: from_addr.clone(),
                date_sent_utc: date_opt.clone(),
            });
        }

        // 30. COMMUNICATION GRAPH
        artifacts.push(ForensicTaxonomyArtifact {
            id: generate_id(),
            domain_id: "graph_network".to_string(),
            subcategory_id: "counterparty_edge".to_string(),
            title: format!("Communication Edge: {} ➔ {}", from_addr, to_addrs),
            primary_value: format!("{} ➔ {}", from_addr, to_addrs),
            secondary_value: Some(from_addr.clone()),
            details: format!("Direct communication path between {} and recipients: {}", from_addr, to_addrs),
            severity: "info".to_string(),
            artifact_type: "derived".to_string(),
            email_id: eid.clone(),
            email_subject: subj_opt.clone(),
            email_from: from_addr.clone(),
            date_sent_utc: date_opt.clone(),
        });
    }

    Ok(artifacts)
}
