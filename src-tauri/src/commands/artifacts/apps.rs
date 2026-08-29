use std::collections::HashSet;
use crate::db::generate_id;
use super::types::ForensicTaxonomyArtifact;
use super::signatures::{APP_SIGNATURES, extract_domain};

pub fn scan_apps_and_services(
    artifacts: &mut Vec<ForensicTaxonomyArtifact>,
    seen: &mut HashSet<String>,
    eid: &str,
    from_addr: &str,
    to_addrs: &str,
    subj_opt: &Option<String>,
    date_opt: &Option<String>,
    from_lower: &str,
    headers_lower: &str,
    subj_lower: &str,
    full_text_lower: &str,
    subj: &str,
) {
    let mut app_matched = false;
    for sig in APP_SIGNATURES {
        let matched_header = sig.keywords.iter().any(|&kw| {
            from_lower.contains(kw) || headers_lower.contains(kw) || subj_lower.contains(kw)
        });

        let matched = matched_header || (!app_matched && sig.keywords.iter().any(|&kw| full_text_lower.contains(kw)));

        if matched {
            app_matched = true;
            let key = format!("app:{}:{}", sig.domain_id, sig.name);
            if seen.insert(key) {
                artifacts.push(ForensicTaxonomyArtifact {
                    id: generate_id(),
                    domain_id: sig.domain_id.to_string(),
                    subcategory_id: sig.subcategory.to_string(),
                    title: sig.category_title.to_string(),
                    primary_value: sig.name.to_string(),
                    secondary_value: Some(format!("User/Recipient: {}", to_addrs)),
                    details: format!("Target account footprint detected on '{}'. Subject: '{}'", sig.name, subj),
                    severity: "medium".to_string(),
                    artifact_type: "derived".to_string(),
                    confidence: Some("high".to_string()),
                    email_id: eid.to_string(),
                    email_subject: subj_opt.clone(),
                    email_from: from_addr.to_string(),
                    date_sent_utc: date_opt.clone(),
                });
            }
        }
    }

    if !app_matched {
        if let Some(domain) = extract_domain(from_addr) {
            let generic_providers = ["gmail.com", "yahoo.com", "hotmail.com", "outlook.com", "aol.com", "icloud.com", "enron.com"];
            if !generic_providers.iter().any(|&gp| domain.ends_with(gp)) && domain.contains('.') {
                let parts: Vec<&str> = domain.split('.').collect();
                if parts.len() >= 2 {
                    let brand = parts[parts.len() - 2];
                    if brand.len() >= 3 && !brand.chars().all(|c| c.is_ascii_digit()) {
                        let brand_cap = format!("{}{}", brand[..1].to_uppercase(), &brand[1..]);
                        let key = format!("dynamic_app:{}", domain);
                        if seen.insert(key) {
                            artifacts.push(ForensicTaxonomyArtifact {
                                id: generate_id(),
                                domain_id: "mobile_apps".to_string(),
                                subcategory_id: "external_services".to_string(),
                                title: format!("Web & Cloud Service ({})", brand_cap),
                                primary_value: domain.clone(),
                                secondary_value: Some(format!("Target: {}", to_addrs)),
                                details: format!("Digital account correspondence from '{}' to '{}'", domain, to_addrs),
                                severity: "info".to_string(),
                                artifact_type: "derived".to_string(),
                                confidence: Some("medium".to_string()),
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
    }
}
