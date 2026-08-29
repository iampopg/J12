use std::collections::HashSet;
use crate::db::generate_id;
use super::types::ForensicTaxonomyArtifact;
use super::signatures::CompiledRegexes;

pub fn scan_credentials_and_crypto(
    artifacts: &mut Vec<ForensicTaxonomyArtifact>,
    seen: &mut HashSet<String>,
    re: &CompiledRegexes,
    eid: &str,
    from_addr: &str,
    subj_opt: &Option<String>,
    date_opt: &Option<String>,
    full_text: &str,
    full_text_lower: &str,
) {
    if full_text_lower.contains("password") || full_text_lower.contains("passwd") || full_text_lower.contains("passcode") || full_text_lower.contains("login") || full_text_lower.contains("user") {
        for cap in re.cred_pair.captures_iter(full_text) {
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
                    secondary_value: Some(from_addr.to_string()),
                    details: format!("Extracted Account Login: User='{}', Pass='{}'", user_val, pass_val),
                    severity: "critical".to_string(),
                    artifact_type: "native".to_string(),
                    confidence: Some("high".to_string()),
                    email_id: eid.to_string(),
                    email_subject: subj_opt.clone(),
                    email_from: from_addr.to_string(),
                    date_sent_utc: date_opt.clone(),
                });
            }
        }

        for cap in re.pass_standalone.captures_iter(full_text) {
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
                        secondary_value: Some(from_addr.to_string()),
                        details: format!("Standalone password value: {}", pass_val),
                        severity: "critical".to_string(),
                        artifact_type: "native".to_string(),
                        confidence: Some("high".to_string()),
                        email_id: eid.to_string(),
                        email_subject: subj_opt.clone(),
                        email_from: from_addr.to_string(),
                        date_sent_utc: date_opt.clone(),
                    });
                }
            }
        }
    }

    if full_text.contains("AKIA") || full_text.contains("sk_live_") || full_text.contains("ghp_") || full_text.contains("AIza") {
        for cap in re.api_keys.captures_iter(full_text) {
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
                    secondary_value: Some(from_addr.to_string()),
                    details: format!("Extracted {} credential token", provider),
                    severity: "critical".to_string(),
                    artifact_type: "native".to_string(),
                    confidence: Some("high".to_string()),
                    email_id: eid.to_string(),
                    email_subject: subj_opt.clone(),
                    email_from: from_addr.to_string(),
                    date_sent_utc: date_opt.clone(),
                });
            }
        }
    }

    if full_text.contains("Bearer ") {
        for cap in re.bearer.captures_iter(full_text) {
            let token = cap[1].to_string();
            let key = format!("bearer:{}", token);
            if seen.insert(key) {
                artifacts.push(ForensicTaxonomyArtifact {
                    id: generate_id(),
                    domain_id: "credentials".to_string(),
                    subcategory_id: "bearer_tokens".to_string(),
                    title: "Bearer Authorization Token".to_string(),
                    primary_value: format!("Bearer {}", &token[..token.len().min(40)]),
                    secondary_value: Some(from_addr.to_string()),
                    details: format!("OAuth/Bearer authorization token: {}", token),
                    severity: "critical".to_string(),
                    artifact_type: "native".to_string(),
                    confidence: Some("high".to_string()),
                    email_id: eid.to_string(),
                    email_subject: subj_opt.clone(),
                    email_from: from_addr.to_string(),
                    date_sent_utc: date_opt.clone(),
                });
            }
        }
    }

    if full_text.contains("eyJ") {
        for cap in re.jwt.captures_iter(full_text) {
            let jwt = cap[1].to_string();
            let key = format!("jwt:{}", jwt);
            if seen.insert(key) {
                artifacts.push(ForensicTaxonomyArtifact {
                    id: generate_id(),
                    domain_id: "credentials".to_string(),
                    subcategory_id: "jwt_tokens".to_string(),
                    title: "JSON Web Token (JWT)".to_string(),
                    primary_value: format!("JWT: {}", &jwt[..jwt.len().min(45)]),
                    secondary_value: Some(from_addr.to_string()),
                    details: format!("JSON Web Token (JWT) session credential: {}", jwt),
                    severity: "critical".to_string(),
                    artifact_type: "native".to_string(),
                    confidence: Some("high".to_string()),
                    email_id: eid.to_string(),
                    email_subject: subj_opt.clone(),
                    email_from: from_addr.to_string(),
                    date_sent_utc: date_opt.clone(),
                });
            }
        }
    }

    if full_text.contains("BEGIN") && full_text.contains("PRIVATE KEY") && re.ssh_key.is_match(full_text) {
        let key = "ssh_key_block".to_string();
        if seen.insert(key) {
            artifacts.push(ForensicTaxonomyArtifact {
                id: generate_id(),
                domain_id: "credentials".to_string(),
                subcategory_id: "ssh_keys".to_string(),
                title: "SSH / OpenSSH Private Key Block".to_string(),
                primary_value: "-----BEGIN PRIVATE KEY-----".to_string(),
                secondary_value: Some(from_addr.to_string()),
                details: "Private cryptographic SSH key block exposed in message".to_string(),
                severity: "critical".to_string(),
                artifact_type: "native".to_string(),
                confidence: Some("high".to_string()),
                email_id: eid.to_string(),
                email_subject: subj_opt.clone(),
                email_from: from_addr.to_string(),
                date_sent_utc: date_opt.clone(),
            });
        }
    }

    if full_text_lower.contains("seed phrase") || full_text_lower.contains("recovery phrase") || full_text_lower.contains("mnemonic") || full_text_lower.contains("wallet seed") || full_text_lower.contains("backup phrase") || full_text_lower.contains("secret words") {
        let anchor_keywords = ["seed phrase", "recovery phrase", "mnemonic", "wallet seed", "backup phrase", "secret words"];
        for kw in &anchor_keywords {
            if let Some(pos) = full_text_lower.find(kw) {
                let snippet = &full_text[pos..full_text.len().min(pos + 350)];
                let words: Vec<&str> = snippet.split(|c: char| !c.is_alphabetic()).filter(|w| w.len() >= 3 && w.len() <= 12).collect();
                if let Some(valid_phrase) = crate::bip39_wordlist::validate_bip39_phrase(&words) {
                    let word_count = valid_phrase.split_whitespace().count();
                    let key = format!("seed:{}", valid_phrase);
                    if seen.insert(key) {
                        artifacts.push(ForensicTaxonomyArtifact {
                            id: generate_id(),
                            domain_id: "crypto".to_string(),
                            subcategory_id: "seed_phrases".to_string(),
                            title: format!("BIP-39 Mnemonic Seed Phrase ({} words)", word_count),
                            primary_value: valid_phrase.clone(),
                            secondary_value: Some(from_addr.to_string()),
                            details: format!("Cryptocurrency BIP-39 dictionary-verified seed phrase ({} words)", word_count),
                            severity: "critical".to_string(),
                            artifact_type: "native".to_string(),
                            confidence: Some("high".to_string()),
                            email_id: eid.to_string(),
                            email_subject: subj_opt.clone(),
                            email_from: from_addr.to_string(),
                            date_sent_utc: date_opt.clone(),
                        });
                    }
                }
            }
        }
    }

    if full_text_lower.contains("private key") || full_text_lower.contains("privkey") {
        for cap in re.privkey.captures_iter(full_text) {
            let pkey = cap[1].trim().to_string();
            let key = format!("privkey:{}", pkey);
            if seen.insert(key) {
                artifacts.push(ForensicTaxonomyArtifact {
                    id: generate_id(),
                    domain_id: "crypto".to_string(),
                    subcategory_id: "private_keys".to_string(),
                    title: "Cryptocurrency Hex Private Key".to_string(),
                    primary_value: pkey.clone(),
                    secondary_value: Some(from_addr.to_string()),
                    details: format!("Raw hex private key: {}", pkey),
                    severity: "critical".to_string(),
                    artifact_type: "native".to_string(),
                    confidence: Some("high".to_string()),
                    email_id: eid.to_string(),
                    email_subject: subj_opt.clone(),
                    email_from: from_addr.to_string(),
                    date_sent_utc: date_opt.clone(),
                });
            }
        }
    }
}
