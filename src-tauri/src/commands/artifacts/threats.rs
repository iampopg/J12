use std::collections::HashSet;
use crate::db::generate_id;
use super::types::ForensicTaxonomyArtifact;
use super::signatures::CompiledRegexes;
use super::validators::validate_ssn;

pub fn scan_identity_threats_and_phishing(
    artifacts: &mut Vec<ForensicTaxonomyArtifact>,
    seen: &mut HashSet<String>,
    re: &CompiledRegexes,
    eid: &str,
    from_addr: &str,
    subj_opt: &Option<String>,
    date_opt: &Option<String>,
    headers_lower: &str,
    full_text: &str,
    full_text_lower: &str,
    html: &str,
) {
    // 6. PII & IDENTITY
    if full_text.contains('-') {
        for cap in re.ssn.captures_iter(full_text) {
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
                        secondary_value: Some(from_addr.to_string()),
                        details: format!("Verified US Social Security Number: {}", ssn),
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

    if full_text_lower.contains("passport") {
        for cap in re.passport.captures_iter(full_text) {
            let pass = cap[1].to_string();
            let key = format!("passport:{}", pass);
            if seen.insert(key) {
                artifacts.push(ForensicTaxonomyArtifact {
                    id: generate_id(),
                    domain_id: "identity_docs".to_string(),
                    subcategory_id: "passport".to_string(),
                    title: "International Passport Number".to_string(),
                    primary_value: format!("Passport: {}", pass),
                    secondary_value: Some(from_addr.to_string()),
                    details: format!("Passport document identifier: {}", pass),
                    severity: "high".to_string(),
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

    if full_text_lower.contains("license") || full_text_lower.contains("licence") {
        for cap in re.driver_lic.captures_iter(full_text) {
            let dl = cap[1].trim().to_string();
            if dl.chars().any(|c| c.is_ascii_digit()) && dl.len() >= 6 {
                let key = format!("dl:{}", dl);
                if seen.insert(key) {
                    artifacts.push(ForensicTaxonomyArtifact {
                        id: generate_id(),
                        domain_id: "identity_docs".to_string(),
                        subcategory_id: "drivers_license".to_string(),
                        title: "Driver's License (DLN)".to_string(),
                        primary_value: format!("DL: {}", dl),
                        secondary_value: Some(from_addr.to_string()),
                        details: format!("Driver's license identifier: {}", dl),
                        severity: "high".to_string(),
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

    if full_text_lower.contains("ein") || full_text_lower.contains("tax id") {
        for cap in re.ein.captures_iter(full_text) {
            let ein = cap[1].to_string();
            let key = format!("ein:{}", ein);
            if seen.insert(key) {
                artifacts.push(ForensicTaxonomyArtifact {
                    id: generate_id(),
                    domain_id: "identity_docs".to_string(),
                    subcategory_id: "ein".to_string(),
                    title: "Employer Identification Number (EIN)".to_string(),
                    primary_value: format!("EIN: {}", ein),
                    secondary_value: Some(from_addr.to_string()),
                    details: format!("US Federal Employer Identification Number: {}", ein),
                    severity: "high".to_string(),
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

    // 7. TRAVEL & LOCATIONS
    if full_text_lower.contains("hotel") || full_text_lower.contains("flight") || full_text_lower.contains("reservation") || full_text_lower.contains("booking") {
        for cap in re.hotel_conf.captures_iter(full_text) {
            let conf = cap[1].to_string();
            let key = format!("hotel_conf:{}", conf);
            if seen.insert(key) {
                artifacts.push(ForensicTaxonomyArtifact {
                    id: generate_id(),
                    domain_id: "locations".to_string(),
                    subcategory_id: "hotel_booking".to_string(),
                    title: "Travel / Lodging Confirmation".to_string(),
                    primary_value: format!("Booking #: {}", conf),
                    secondary_value: Some(from_addr.to_string()),
                    details: format!("Travel lodging confirmation code: {}", conf),
                    severity: "high".to_string(),
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

    // 8. THREATS & CONTRABAND
    for cap in re.weapons.captures_iter(full_text) {
        let wpn = cap[1].to_string();
        let key = format!("wpn:{}", wpn.to_lowercase());
        if seen.insert(key) {
            artifacts.push(ForensicTaxonomyArtifact {
                id: generate_id(),
                domain_id: "contraband".to_string(),
                subcategory_id: "weapons".to_string(),
                title: format!("Firearms & Weapons ({})", wpn.to_uppercase()),
                primary_value: wpn.to_uppercase(),
                secondary_value: Some(from_addr.to_string()),
                details: format!("Firearm or weapons keyword: {}", wpn),
                severity: "critical".to_string(),
                artifact_type: "native".to_string(),
                confidence: Some("high".to_string()),
                email_id: eid.to_string(),
                email_subject: subj_opt.clone(),
                email_from: from_addr.to_string(),
                date_sent_utc: date_opt.clone(),
            });
            break;
        }
    }

    for cap in re.narcotics.captures_iter(full_text) {
        let drug = cap[1].to_string();
        let key = format!("drug:{}", drug.to_lowercase());
        if seen.insert(key) {
            artifacts.push(ForensicTaxonomyArtifact {
                id: generate_id(),
                domain_id: "contraband".to_string(),
                subcategory_id: "narcotics".to_string(),
                title: format!("Controlled Substances ({})", drug.to_uppercase()),
                primary_value: drug.to_uppercase(),
                secondary_value: Some(from_addr.to_string()),
                details: format!("Illicit drug mention: {}", drug),
                severity: "critical".to_string(),
                artifact_type: "native".to_string(),
                confidence: Some("high".to_string()),
                email_id: eid.to_string(),
                email_subject: subj_opt.clone(),
                email_from: from_addr.to_string(),
                date_sent_utc: date_opt.clone(),
            });
            break;
        }
    }

    for cap in re.explosives.captures_iter(full_text) {
        let exp = cap[1].to_string();
        let key = format!("exp:{}", exp.to_lowercase());
        if seen.insert(key) {
            artifacts.push(ForensicTaxonomyArtifact {
                id: generate_id(),
                domain_id: "contraband".to_string(),
                subcategory_id: "explosives".to_string(),
                title: format!("Explosives & IED Threat ({})", exp.to_uppercase()),
                primary_value: exp.to_uppercase(),
                secondary_value: Some(from_addr.to_string()),
                details: format!("Explosive material or detonator indicator: {}", exp),
                severity: "critical".to_string(),
                artifact_type: "native".to_string(),
                confidence: Some("high".to_string()),
                email_id: eid.to_string(),
                email_subject: subj_opt.clone(),
                email_from: from_addr.to_string(),
                date_sent_utc: date_opt.clone(),
            });
            break;
        }
    }

    for cap in re.terrorism.captures_iter(full_text) {
        let trr = cap[1].to_string();
        let key = format!("trr:{}", trr.to_lowercase());
        if seen.insert(key) {
            artifacts.push(ForensicTaxonomyArtifact {
                id: generate_id(),
                domain_id: "contraband".to_string(),
                subcategory_id: "terrorism".to_string(),
                title: format!("Violent Extremism ({})", trr.to_uppercase()),
                primary_value: trr.to_uppercase(),
                secondary_value: Some(from_addr.to_string()),
                details: format!("Extremist organization or threat keyword: {}", trr),
                severity: "critical".to_string(),
                artifact_type: "native".to_string(),
                confidence: Some("high".to_string()),
                email_id: eid.to_string(),
                email_subject: subj_opt.clone(),
                email_from: from_addr.to_string(),
                date_sent_utc: date_opt.clone(),
            });
            break;
        }
    }

    // 9. MALWARE & CYBER IOCs
    if full_text.contains("CVE-") {
        for cap in re.cve.captures_iter(full_text) {
            let cve = cap[1].to_string();
            let key = format!("cve:{}", cve);
            if seen.insert(key) {
                artifacts.push(ForensicTaxonomyArtifact {
                    id: generate_id(),
                    domain_id: "malware_threats".to_string(),
                    subcategory_id: "cve_vulnerability".to_string(),
                    title: format!("Common Vulnerability ({})", cve),
                    primary_value: cve.clone(),
                    secondary_value: Some(from_addr.to_string()),
                    details: format!("Vulnerability identifier: {}", cve),
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

    if full_text_lower.contains("command and control") || full_text_lower.contains("reverse shell") || full_text_lower.contains("meterpreter") {
        for cap in re.c2.captures_iter(full_text) {
            let c2 = cap[1].to_string();
            let key = format!("c2:{}", c2.to_lowercase());
            if seen.insert(key) {
                artifacts.push(ForensicTaxonomyArtifact {
                    id: generate_id(),
                    domain_id: "malware_threats".to_string(),
                    subcategory_id: "c2_indicators".to_string(),
                    title: "Command & Control (C2) Indicator".to_string(),
                    primary_value: c2.to_uppercase(),
                    secondary_value: Some(from_addr.to_string()),
                    details: format!("Command & Control terminology: {}", c2),
                    severity: "critical".to_string(),
                    artifact_type: "native".to_string(),
                    confidence: Some("high".to_string()),
                    email_id: eid.to_string(),
                    email_subject: subj_opt.clone(),
                    email_from: from_addr.to_string(),
                    date_sent_utc: date_opt.clone(),
                });
                break;
            }
        }
    }

    // 10. CORPORATE & LEGAL PRIVILEGED
    if full_text_lower.contains("confidential") || full_text_lower.contains("privilege") {
        for cap in re.confidential.captures_iter(full_text) {
            let conf = cap[1].to_string();
            let key = format!("confidential:{}", conf.to_lowercase());
            if seen.insert(key) {
                artifacts.push(ForensicTaxonomyArtifact {
                    id: generate_id(),
                    domain_id: "secrets".to_string(),
                    subcategory_id: "privileged_confidential".to_string(),
                    title: format!("Legal Privilege / Confidential ({})", conf.to_uppercase()),
                    primary_value: conf.to_uppercase(),
                    secondary_value: Some(from_addr.to_string()),
                    details: format!("Confidentiality or legal privilege notice: {}", conf),
                    severity: "high".to_string(),
                    artifact_type: "native".to_string(),
                    confidence: Some("high".to_string()),
                    email_id: eid.to_string(),
                    email_subject: subj_opt.clone(),
                    email_from: from_addr.to_string(),
                    date_sent_utc: date_opt.clone(),
                });
                break;
            }
        }
    }

    if full_text_lower.contains("non-disclosure") || full_text_lower.contains("nda") || full_text_lower.contains("proprietary") {
        for cap in re.nda.captures_iter(full_text) {
            let nda_val = cap[1].to_string();
            let key = format!("nda:{}", nda_val.to_lowercase());
            if seen.insert(key) {
                artifacts.push(ForensicTaxonomyArtifact {
                    id: generate_id(),
                    domain_id: "secrets".to_string(),
                    subcategory_id: "nda_agreements".to_string(),
                    title: "Non-Disclosure Agreement (NDA)".to_string(),
                    primary_value: nda_val.to_uppercase(),
                    secondary_value: Some(from_addr.to_string()),
                    details: format!("NDA or distribution restriction clause: {}", nda_val),
                    severity: "high".to_string(),
                    artifact_type: "native".to_string(),
                    confidence: Some("high".to_string()),
                    email_id: eid.to_string(),
                    email_subject: subj_opt.clone(),
                    email_from: from_addr.to_string(),
                    date_sent_utc: date_opt.clone(),
                });
                break;
            }
        }
    }

    // 11. PHISHING & SOCIAL ENGINEERING
    if full_text_lower.contains("verify") || full_text_lower.contains("password") || full_text_lower.contains("credentials") {
        for cap in re.phish_cred.captures_iter(full_text) {
            let cr = cap[1].to_string();
            let key = format!("phish_cred:{}", cr.to_lowercase());
            if seen.insert(key) {
                artifacts.push(ForensicTaxonomyArtifact {
                    id: generate_id(),
                    domain_id: "phishing".to_string(),
                    subcategory_id: "credential_requests".to_string(),
                    title: "Credential Harvesting Lure".to_string(),
                    primary_value: cr.to_uppercase(),
                    secondary_value: Some(from_addr.to_string()),
                    details: format!("Request for login credentials / password update: {}", cr),
                    severity: "critical".to_string(),
                    artifact_type: "derived".to_string(),
                    confidence: Some("high".to_string()),
                    email_id: eid.to_string(),
                    email_subject: subj_opt.clone(),
                    email_from: from_addr.to_string(),
                    date_sent_utc: date_opt.clone(),
                });
                break;
            }
        }
    }

    if full_text_lower.contains("wire") || full_text_lower.contains("gift card") {
        for cap in re.phish_finance.captures_iter(full_text) {
            let fin = cap[1].to_string();
            let key = format!("phish_fin:{}", fin.to_lowercase());
            if seen.insert(key) {
                artifacts.push(ForensicTaxonomyArtifact {
                    id: generate_id(),
                    domain_id: "phishing".to_string(),
                    subcategory_id: "financial_demands".to_string(),
                    title: "BEC / Financial Payment Demand".to_string(),
                    primary_value: fin.to_uppercase(),
                    secondary_value: Some(from_addr.to_string()),
                    details: format!("Fraudulent wire transfer or gift card demand: {}", fin),
                    severity: "critical".to_string(),
                    artifact_type: "derived".to_string(),
                    confidence: Some("high".to_string()),
                    email_id: eid.to_string(),
                    email_subject: subj_opt.clone(),
                    email_from: from_addr.to_string(),
                    date_sent_utc: date_opt.clone(),
                });
                break;
            }
        }
    }

    // 12. AUTHENTICATION FAILURES
    if headers_lower.contains("spf=fail") || headers_lower.contains("spf=softfail") {
        let key = "spf_fail".to_string();
        if seen.insert(key) {
            artifacts.push(ForensicTaxonomyArtifact {
                id: generate_id(),
                domain_id: "authentication".to_string(),
                subcategory_id: "spf_fail".to_string(),
                title: "SPF Authentication Failure".to_string(),
                primary_value: "SPF: FAIL".to_string(),
                secondary_value: Some(from_addr.to_string()),
                details: "Sender failed SPF domain authorization check".to_string(),
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

    if headers_lower.contains("dkim=fail") {
        let key = "dkim_fail".to_string();
        if seen.insert(key) {
            artifacts.push(ForensicTaxonomyArtifact {
                id: generate_id(),
                domain_id: "authentication".to_string(),
                subcategory_id: "dkim_fail".to_string(),
                title: "DKIM Cryptographic Signature Failure".to_string(),
                primary_value: "DKIM: FAIL".to_string(),
                secondary_value: Some(from_addr.to_string()),
                details: "Cryptographic signature validation failed on transport header".to_string(),
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

    if headers_lower.contains("dmarc=fail") && seen.insert("dmarc_fail".to_string()) {
        artifacts.push(ForensicTaxonomyArtifact {
            id: generate_id(),
            domain_id: "authentication".to_string(),
            subcategory_id: "dmarc_fail".to_string(),
            title: "DMARC Alignment Policy Failure".to_string(),
            primary_value: "DMARC: FAIL".to_string(),
            secondary_value: Some(from_addr.to_string()),
            details: "Message failed DMARC domain alignment policy".to_string(),
            severity: "critical".to_string(),
            artifact_type: "native".to_string(),
            confidence: Some("high".to_string()),
            email_id: eid.to_string(),
            email_subject: subj_opt.clone(),
            email_from: from_addr.to_string(),
            date_sent_utc: date_opt.clone(),
        });
    }

    // 13. Hidden 1x1 Web Tracking Beacons
    if (html.contains("width=\"1\" height=\"1\"") || html.contains("width='1' height='1'")) && seen.insert("tracking_pixel".to_string()) {
        artifacts.push(ForensicTaxonomyArtifact {
            id: generate_id(),
            domain_id: "network".to_string(),
            subcategory_id: "tracking_pixels".to_string(),
            title: "Hidden 1x1 Web Tracking Beacon".to_string(),
            primary_value: "1x1 Web Beacon".to_string(),
            secondary_value: Some(from_addr.to_string()),
            details: "Hidden 1x1 tracking beacon embedded in HTML".to_string(),
            severity: "medium".to_string(),
            artifact_type: "native".to_string(),
            confidence: Some("high".to_string()),
            email_id: eid.to_string(),
            email_subject: subj_opt.clone(),
            email_from: from_addr.to_string(),
            date_sent_utc: date_opt.clone(),
        });
    }
}
