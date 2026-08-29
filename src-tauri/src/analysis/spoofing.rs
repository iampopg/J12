use super::types::{AuthResults, SpoofingFinding};

/// Detect spoofing attempts from email data
pub fn detect_spoofing(
    from_addr: &str,
    from_display: Option<&str>,
    headers_raw: &str,
    auth_results: &AuthResults,
) -> Vec<SpoofingFinding> {
    let mut findings = Vec::new();
    
    let from_domain = extract_domain(from_addr);
    
    // 1. Display name spoofing
    if let Some(display) = from_display {
        let display_lower = display.to_lowercase();
        if let Some(spoofed_email) = extract_email_from_display_name(&display_lower) {
            let spoofed_domain = extract_domain(&spoofed_email);
            if spoofed_domain != from_domain {
                findings.push(SpoofingFinding {
                    finding_type: "display_name_spoofing".to_string(),
                    severity: "high".to_string(),
                    confidence: "high".to_string(),
                    title: "Display name contains different email domain".to_string(),
                    description: format!(
                        "Display name '{}' contains email '{}' but actual sender is '{}'",
                        display, spoofed_email, from_addr
                    ),
                    indicator: format!("{} vs {}", spoofed_domain, from_domain),
                });
            }
        }
        
        let brands = [
            ("paypal", "paypal.com"),
            ("apple", "apple.com"),
            ("microsoft", "microsoft.com"),
            ("google", "google.com"),
            ("amazon", "amazon.com"),
            ("netflix", "netflix.com"),
            ("wells fargo", "wellsfargo.com"),
            ("chase", "chase.com"),
            ("bank of america", "bankofamerica.com"),
            ("citibank", "citi.com"),
        ];
        
        for (brand, canonical_domain) in &brands {
            let brand_clean = brand.replace([' ', '-', '_'], "");
            let domain_clean = from_domain.replace(['-', '_'], "");
            
            if display_lower.contains(brand) {
                let is_legit_brand_domain = from_domain.ends_with(canonical_domain) 
                    || from_domain == *canonical_domain
                    || domain_clean.contains(&brand_clean);

                if !is_legit_brand_domain {
                    findings.push(SpoofingFinding {
                        finding_type: "brand_impersonation".to_string(),
                        severity: "critical".to_string(),
                        confidence: "high".to_string(),
                        title: format!("Possible {} brand impersonation", brand),
                        description: format!(
                            "Display name '{}' contains brand '{}' but sender domain is '{}'",
                            display, brand, from_domain
                        ),
                        indicator: format!("{} in display name, not in domain {}", brand, from_domain),
                    });
                }
            }
        }
    }
    
    // 2. From/Return-Path mismatch
    if let Some(return_path_domain) = extract_return_path_domain(headers_raw) {
        if return_path_domain != from_domain {
            findings.push(SpoofingFinding {
                finding_type: "return_path_mismatch".to_string(),
                severity: "medium".to_string(),
                confidence: "high".to_string(),
                title: "Return-Path domain differs from From domain".to_string(),
                description: format!(
                    "From: {} (domain: {}), Return-Path: {}",
                    from_addr, from_domain, return_path_domain
                ),
                indicator: format!("{} vs {}", from_domain, return_path_domain),
            });
        }
    }
    
    // 3. Reply-To mismatch
    if let Some(reply_to_domain) = extract_reply_to_domain(headers_raw) {
        if reply_to_domain != from_domain {
            findings.push(SpoofingFinding {
                finding_type: "reply_to_mismatch".to_string(),
                severity: "medium".to_string(),
                confidence: "high".to_string(),
                title: "Reply-To domain differs from From domain".to_string(),
                description: format!(
                    "From: {} (domain: {}), Reply-To: {}",
                    from_addr, from_domain, reply_to_domain
                ),
                indicator: format!("{} vs {}", from_domain, reply_to_domain),
            });
        }
    }
    
    // 4. Message-ID domain anomaly
    if let Some(msg_id_domain) = extract_message_id_domain(headers_raw) {
        if msg_id_domain != from_domain {
            findings.push(SpoofingFinding {
                finding_type: "message_id_anomaly".to_string(),
                severity: "low".to_string(),
                confidence: "medium".to_string(),
                title: "Message-ID domain differs from From domain".to_string(),
                description: format!(
                    "From: {} (domain: {}), Message-ID domain: {}",
                    from_addr, from_domain, msg_id_domain
                ),
                indicator: format!("{} vs {}", from_domain, msg_id_domain),
            });
        }
    }
    
    // 5. Homoglyph/punycode detection
    if is_homoglyph_domain(&from_domain) || from_domain.starts_with("xn--") {
        findings.push(SpoofingFinding {
            finding_type: "homoglyph_domain".to_string(),
            severity: "critical".to_string(),
            confidence: "high".to_string(),
            title: "Possible homoglyph/punycode domain attack".to_string(),
            description: format!(
                "Domain '{}' may be using visual confusable characters to impersonate another domain",
                from_domain
            ),
            indicator: from_domain.clone(),
        });
    }
    
    // 6. Auth failure findings
    if auth_results.spf.result == "fail" {
        findings.push(SpoofingFinding {
            finding_type: "spf_failure".to_string(),
            severity: "high".to_string(),
            confidence: "high".to_string(),
            title: "SPF authentication failed".to_string(),
            description: "The sending server is not authorized to send email for this domain".to_string(),
            indicator: format!("SPF fail for {}", from_domain),
        });
    }
    
    if auth_results.dkim.iter().any(|d| d.result == "fail") {
        findings.push(SpoofingFinding {
            finding_type: "dkim_failure".to_string(),
            severity: "high".to_string(),
            confidence: "high".to_string(),
            title: "DKIM signature verification failed".to_string(),
            description: "The message's DKIM signature could not be verified".to_string(),
            indicator: "DKIM fail".to_string(),
        });
    }
    
    if auth_results.dmarc.result == "fail" {
        findings.push(SpoofingFinding {
            finding_type: "dmarc_failure".to_string(),
            severity: "critical".to_string(),
            confidence: "high".to_string(),
            title: "DMARC authentication failed".to_string(),
            description: "Both SPF and DKIM failed alignment checks".to_string(),
            indicator: format!("DMARC fail for {}", from_domain),
        });
    }
    
    findings
}

/// Detect deep content threats including BEC wire fraud, gift cards, and phishing lures
pub fn detect_content_threats(
    from_addr: &str,
    from_display: Option<&str>,
    subject: Option<&str>,
    body_text: Option<&str>,
) -> Vec<SpoofingFinding> {
    let mut findings = Vec::new();
    let subj = subject.unwrap_or("").to_lowercase();
    let body = body_text.unwrap_or("").to_lowercase();
    let full_content = format!("{} {}", subj, body);

    let from_domain = extract_domain(from_addr).to_lowercase();
    let display_str = from_display.unwrap_or("").to_lowercase();

    let wire_triggers = [
        "wire transfer", "updated bank details", "new bank account", "swift code", 
        "routing number", "ach transfer", "direct deposit change", "urgent payment", 
        "invoice overdue wire", "settlement fund payment", "remittance instruction",
        "fund transfer instruction", "account verification for wire"
    ];
    let urgency_triggers = [
        "urgent", "immediately", "asap", "confidential", "before end of day", 
        "do not call", "strictly private", "keep this between us", "wire today", "wire promptly"
    ];

    let has_wire = wire_triggers.iter().any(|&w| full_content.contains(w));
    let has_urgency = urgency_triggers.iter().any(|&u| full_content.contains(u));

    if has_wire {
        let severity = if has_urgency { "critical" } else { "high" };
        let matched_trigger = wire_triggers.iter().find(|&&w| full_content.contains(w)).unwrap_or(&"wire transfer");
        findings.push(SpoofingFinding {
            finding_type: "bec_wire_fraud".to_string(),
            severity: severity.to_string(),
            confidence: "high".to_string(),
            title: format!("BEC / Wire Transfer Request: '{}'", matched_trigger),
            description: format!(
                "Financial payment redirection pattern detected ('{}'). Urgency flagged: {}",
                matched_trigger, if has_urgency { "HIGH (Immediate action demanded)" } else { "Standard" }
            ),
            indicator: format!("wire_trigger: {}", matched_trigger),
        });
    }

    let giftcard_triggers = [
        "apple gift card", "itunes gift card", "google play card", "steam card", 
        "buy gift cards", "need you to run an errand", "discreet task for me"
    ];
    if let Some(&gc) = giftcard_triggers.iter().find(|&&g| full_content.contains(g)) {
        findings.push(SpoofingFinding {
            finding_type: "gift_card_fraud".to_string(),
            severity: "high".to_string(),
            confidence: "high".to_string(),
            title: format!("Gift Card Scam Pattern: '{}'", gc),
            description: format!("Message requests gift card purchasing or non-standard executive procurement ('{}')", gc),
            indicator: format!("gift_card: {}", gc),
        });
    }

    let cred_triggers = [
        "verify your account", "password reset requested", "mailbox size exceeded", 
        "account suspended", "mfa reset required", "login attempt blocked", 
        "update your credentials", "microsoft 365 security notice", "re-authenticate now",
        "verify banking details", "unlock your account"
    ];
    if let Some(&lure) = cred_triggers.iter().find(|&&c| full_content.contains(c)) {
        findings.push(SpoofingFinding {
            finding_type: "credential_phishing".to_string(),
            severity: "high".to_string(),
            confidence: "high".to_string(),
            title: format!("Credential Phishing Lure: '{}'", lure),
            description: format!("Phishing pattern detected attempting to harvest credentials or manipulate authentication ('{}')", lure),
            indicator: format!("phishing_lure: {}", lure),
        });
    }

    let legal_triggers = [
        "attorney-client privilege", "privileged and confidential", "strictly confidential", 
        "trade secret", "internal use only", "non-disclosure agreement", "material non-public"
    ];
    if let Some(&legal) = legal_triggers.iter().find(|&&l| full_content.contains(l)) {
        findings.push(SpoofingFinding {
            finding_type: "confidential_exfiltration".to_string(),
            severity: "medium".to_string(),
            confidence: "high".to_string(),
            title: format!("Privileged / Confidential Information: '{}'", legal),
            description: format!("Message contains legal privilege or corporate confidentiality notice ('{}')", legal),
            indicator: format!("confidential_marker: {}", legal),
        });
    }

    let free_webmail = ["gmail.com", "yahoo.com", "hotmail.com", "outlook.com", "aol.com", "icloud.com", "protonmail.com"];
    let vip_titles = ["ceo", "cfo", "chief executive", "president", "managing director", "general counsel", "vp", "vice president"];
    let is_webmail = free_webmail.iter().any(|&d| from_domain.contains(d));
    let has_vip_title = vip_titles.iter().any(|&v| display_str.contains(v));

    if is_webmail && has_vip_title {
        findings.push(SpoofingFinding {
            finding_type: "executive_impersonation".to_string(),
            severity: "critical".to_string(),
            confidence: "high".to_string(),
            title: "Executive Impersonation via Public Webmail Domain".to_string(),
            description: format!(
                "Display name '{}' claims executive rank but sender domain is public webmail ('{}')",
                from_display.unwrap_or(""), from_domain
            ),
            indicator: format!("{} on {}", display_str, from_domain),
        });
    }

    findings
}

pub fn extract_domain(email: &str) -> String {
    email.split('@').nth(1)
        .map(|d| d.trim_matches('>').trim().to_lowercase())
        .unwrap_or_default()
}

fn extract_email_from_display_name(display: &str) -> Option<String> {
    display.find('<').and_then(|start| {
        display[start..].find('>').map(|end| display[start + 1..start + end].to_string())
    })
}

fn extract_return_path_domain(headers_raw: &str) -> Option<String> {
    headers_raw.lines()
        .find(|l| l.to_lowercase().starts_with("return-path:"))
        .and_then(|l| l.split_once(':'))
        .map(|(_, v)| v.trim())
        .and_then(|v| {
            let email = v.trim_matches(|c| c == '<' || c == '>');
            if email.is_empty() {
                None
            } else {
                Some(extract_domain(email))
            }
        })
}

fn extract_reply_to_domain(headers_raw: &str) -> Option<String> {
    headers_raw.lines()
        .find(|l| l.to_lowercase().starts_with("reply-to:"))
        .and_then(|l| l.split_once(':'))
        .map(|(_, v)| v.trim())
        .and_then(|v| {
            extract_email_from_display_name(v)
                .or_else(|| {
                    let cleaned = v.trim_matches(|c| c == '<' || c == '>');
                    if cleaned.contains('@') {
                        Some(cleaned.to_string())
                    } else {
                        None
                    }
                })
        })
        .map(|e| extract_domain(&e))
}

fn extract_message_id_domain(headers_raw: &str) -> Option<String> {
    let domain = headers_raw.lines()
        .find(|l| l.to_lowercase().starts_with("message-id:"))
        .and_then(|l| l.split_once(':'))
        .map(|(_, v)| v.trim())
        .and_then(|v| {
            v.split('@').nth(1)
                .map(|d| d.trim_matches('>').to_lowercase())
        })?;
    
    if !domain.contains('.') {
        return None;
    }
    
    let internal_names = ["localhost", "mail", "mx", "smtp", "exchange", "domino"];
    if internal_names.contains(&domain.as_str()) {
        return None;
    }
    
    Some(domain)
}

fn is_homoglyph_domain(domain: &str) -> bool {
    if domain.starts_with("xn--") || domain.contains(".xn--") {
        return true;
    }
    
    let has_latin = domain.chars().any(|c| c.is_ascii_alphabetic());
    let has_cyrillic = domain.chars().any(|c| ('\u{0400}'..='\u{04FF}').contains(&c));
    let has_greek = domain.chars().any(|c| ('\u{0370}'..='\u{03FF}').contains(&c));
    
    (has_latin && has_cyrillic) || (has_latin && has_greek)
}
