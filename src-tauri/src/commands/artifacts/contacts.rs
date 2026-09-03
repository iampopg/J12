use std::collections::HashSet;
use crate::db::generate_id;
use super::types::ForensicTaxonomyArtifact;
use super::signatures::CompiledRegexes;

/// Resolves ITU-T Country Dial Code to Country Name and Flag Emoji
pub fn lookup_country(dial_prefix: &str) -> (&'static str, &'static str) {
    match dial_prefix {
        "1" => ("United States / Canada", "🇺🇸/🇨🇦"), "44" => ("United Kingdom", "🇬🇧"),
        "49" => ("Germany", "🇩🇪"), "33" => ("France", "🇫🇷"), "39" => ("Italy", "🇮🇹"),
        "34" => ("Spain", "🇪🇸"), "31" => ("Netherlands", "🇳🇱"), "41" => ("Switzerland", "🇨🇭"),
        "46" => ("Sweden", "🇸🇪"), "47" => ("Norway", "🇳🇴"), "45" => ("Denmark", "🇩🇰"),
        "358" => ("Finland", "🇫🇮"), "353" => ("Ireland", "🇮🇪"), "48" => ("Poland", "🇵🇱"),
        "43" => ("Austria", "🇦🇹"), "32" => ("Belgium", "🇧🇪"), "351" => ("Portugal", "🇵🇹"),
        "30" => ("Greece", "🇬🇷"), "90" => ("Turkey", "🇹🇷"), "7" => ("Russia / Kazakhstan", "🇷🇺/🇰🇿"),
        "971" => ("United Arab Emirates", "🇦🇪"), "966" => ("Saudi Arabia", "🇸🇦"),
        "972" => ("Israel", "🇮🇱"), "974" => ("Qatar", "🇶🇦"), "965" => ("Kuwait", "🇰🇼"),
        "968" => ("Oman", "🇴🇲"), "973" => ("Bahrain", "🇧🇭"), "962" => ("Jordan", "🇯🇴"),
        "961" => ("Lebanon", "🇱🇧"), "91" => ("India", "🇮🇳"), "92" => ("Pakistan", "🇵🇰"),
        "880" => ("Bangladesh", "🇧🇩"), "94" => ("Sri Lanka", "🇱🇰"), "86" => ("China", "🇨🇳"),
        "852" => ("Hong Kong", "🇭🇰"), "886" => ("Taiwan", "🇹🇼"), "81" => ("Japan", "🇯🇵"),
        "82" => ("South Korea", "🇰🇷"), "65" => ("Singapore", "🇸🇬"), "60" => ("Malaysia", "🇲🇾"),
        "62" => ("Indonesia", "🇮🇩"), "66" => ("Thailand", "🇹🇭"), "84" => ("Vietnam", "🇻🇳"),
        "63" => ("Philippines", "🇵🇭"), "61" => ("Australia", "🇦🇺"), "64" => ("New Zealand", "🇳🇿"),
        "234" => ("Nigeria", "🇳🇬"), "27" => ("South Africa", "🇿🇦"), "20" => ("Egypt", "🇪🇬"),
        "254" => ("Kenya", "🇰🇪"), "233" => ("Ghana", "🇬🇭"), "212" => ("Morocco", "🇲🇦"),
        "55" => ("Brazil", "🇧🇷"), "52" => ("Mexico", "🇲🇽"), "54" => ("Argentina", "🇦🇷"),
        "56" => ("Chile", "🇨🇱"), "57" => ("Colombia", "🇨🇴"), "58" => ("Venezuela", "🇻🇪"),
        "51" => ("Peru", "🇵🇪"), _ => ("International Line", "🌐"),
    }
}

/// Identifies National Mobile / Local formats (e.g. Nigerian 0814..., UK 07..., UAE 05...)
pub fn lookup_national_format(digits: &str) -> Option<(&'static str, &'static str, String)> {
    if digits.len() == 11 && (digits.starts_with("080") || digits.starts_with("081") || digits.starts_with("070") || digits.starts_with("090") || digits.starts_with("091")) {
        return Some(("Nigeria (National Mobile)", "🇳🇬", format!("+234 {}", &digits[1..])));
    }
    if digits.len() == 11 && digits.starts_with("07") {
        return Some(("United Kingdom (National Mobile)", "🇬🇧", format!("+44 {}", &digits[1..])));
    }
    if digits.len() == 10 && digits.starts_with("05") {
        return Some(("United Arab Emirates (Mobile)", "🇦🇪", format!("+971 {}", &digits[1..])));
    }
    if (digits.len() == 11 || digits.len() == 12) && (digits.starts_with("015") || digits.starts_with("016") || digits.starts_with("017")) {
        return Some(("Germany (National Mobile)", "🇩🇪", format!("+49 {}", &digits[1..])));
    }
    if digits.len() == 10 && (digits.starts_with("06") || digits.starts_with("07")) {
        return Some(("France (National Mobile)", "🇫🇷", format!("+33 {}", &digits[1..])));
    }
    None
}

/// Identifies Raw International Prefixes without leading + (e.g. 2348143893443, 447911123456)
pub fn lookup_raw_prefix(digits: &str) -> Option<(&'static str, &'static str, String)> {
    if digits.len() == 13 && digits.starts_with("234") {
        return Some(("Nigeria", "🇳🇬", format!("+{}", digits)));
    }
    if digits.len() == 12 && digits.starts_with("44") {
        return Some(("United Kingdom", "🇬🇧", format!("+{}", digits)));
    }
    if (digits.len() == 11 || digits.len() == 12) && digits.starts_with("971") {
        return Some(("United Arab Emirates", "🇦🇪", format!("+{}", digits)));
    }
    if digits.len() == 12 && digits.starts_with("91") {
        return Some(("India", "🇮🇳", format!("+{}", digits)));
    }
    if digits.len() == 13 && digits.starts_with("86") {
        return Some(("China", "🇨🇳", format!("+{}", digits)));
    }
    if (digits.len() == 12 || digits.len() == 13) && digits.starts_with("49") {
        return Some(("Germany", "🇩🇪", format!("+{}", digits)));
    }
    if digits.len() == 11 && digits.starts_with("33") {
        return Some(("France", "🇫🇷", format!("+{}", digits)));
    }
    if digits.len() == 11 && digits.starts_with('1') {
        return Some(("United States / Canada", "🇺🇸/🇨🇦", format!("+{}", digits)));
    }
    None
}

/// 5-Stage False-Positive Filter for >= 90% True Positive Rate
pub fn is_valid_phone_candidate(raw: &str, digits: &str, dial_code: Option<&str>, has_explicit_label: bool) -> bool {
    let d_len = digits.len();

    let min_len = if has_explicit_label { 7 } else { 9 };
    if d_len < min_len || d_len > 15 {
        return false;
    }

    // Reject all identical repeating digits (e.g. 0000000000)
    if digits.chars().all(|c| c == digits.chars().next().unwrap_or('0')) {
        return false;
    }

    // Reject sequential dummy runs
    if digits.contains("12345678") || digits.contains("98765432") {
        return false;
    }

    // Reject numbers with decimal points (e.g. +847715.590137096, 12345.678901)
    if raw.contains('.') {
        if raw.starts_with('+') {
            return false;
        }
        let dot_parts: Vec<&str> = raw.split('.').collect();
        if dot_parts.len() == 2 {
            let frac = dot_parts[1].trim();
            if frac.len() >= 4 {
                return false;
            }
        }
        for part in dot_parts {
            let part_digits = part.chars().filter(|c| c.is_ascii_digit()).count();
            if part_digits > 4 {
                return false;
            }
        }
    }

    // Reject ISO dates & timestamps (e.g. 2024-08-29, 2026/08/29, 5112-05)
    if !has_explicit_label {
        if (raw.starts_with("19") || raw.starts_with("20") || raw.starts_with("+19") || raw.starts_with("+20")) && (raw.contains('-') || raw.contains('/')) && d_len <= 8 {
            return false;
        }
        if raw.contains(':') && raw.chars().filter(|&c| c == ':').count() >= 2 {
            return false;
        }
        if raw.contains('.') && raw.chars().filter(|&c| c == '.').count() == 3 {
            return false;
        }
        if (raw.contains('-') || raw.contains('.')) && d_len <= 8 {
            return false;
        }
    }

    // Country-specific strict E.164 boundary rules
    if let Some(dc) = dial_code {
        match dc {
            "1" => {
                if d_len != 11 { return false; }
                let area_first = digits.chars().nth(1).unwrap_or('0');
                if area_first == '0' || area_first == '1' { return false; }
            }
            "44" => { if d_len < 11 || d_len > 12 { return false; } }
            "234" => { if d_len != 13 { return false; } }
            "971" => { if d_len < 11 || d_len > 12 { return false; } }
            "49" => { if d_len < 12 || d_len > 14 { return false; } }
            "33" => { if d_len != 11 { return false; } }
            "91" => { if d_len != 12 { return false; } }
            _ => {
                let dc_len = dc.len();
                if d_len <= dc_len + 6 { return false; }
            }
        }
    }

    true
}

/// Main Scanner for Phone Numbers, Signatures, Email Contacts, and vCards
pub fn scan_phone_numbers_and_contacts(
    artifacts: &mut Vec<ForensicTaxonomyArtifact>,
    seen: &mut HashSet<String>,
    re: &CompiledRegexes,
    eid: &str,
    from_addr: &str,
    from_disp: Option<&str>,
    subj_opt: &Option<String>,
    date_opt: &Option<String>,
    full_text: &str,
    full_text_lower: &str,
) {
    // 0. Email Account Contacts & Correspondents
    let clean_from = from_addr.trim();
    if !clean_from.is_empty() && clean_from.contains('@') {
        let key = format!("contact_email:{}", clean_from.to_lowercase());
        if seen.insert(key) {
            let disp = from_disp.unwrap_or("").trim();
            let label = if !disp.is_empty() && disp != clean_from {
                format!("👤 {} ({})", disp, clean_from)
            } else {
                format!("👤 Contact: {}", clean_from)
            };

            artifacts.push(ForensicTaxonomyArtifact {
                id: generate_id(),
                domain_id: "phone_contacts".to_string(),
                subcategory_id: "email_contacts".to_string(),
                title: label,
                primary_value: clean_from.to_string(),
                secondary_value: if !disp.is_empty() { Some(disp.to_string()) } else { None },
                details: format!("Account Correspondent: {} <{}>", disp, clean_from),
                severity: "info".to_string(),
                artifact_type: "native".to_string(),
                confidence: Some("high".to_string()),
                email_id: eid.to_string(),
                email_subject: subj_opt.clone(),
                email_from: from_addr.to_string(),
                date_sent_utc: date_opt.clone(),
            });
        }
    }

    // 1. vCard (.vcf) Contact Card Parsing (RFC 6350)
    if full_text.contains("BEGIN:VCARD") {
        for cap in re.vcard_block.captures_iter(full_text) {
            let card_body = &cap[1];
            let mut fn_name = String::new();
            let mut org = String::new();
            let mut title = String::new();
            let mut tel = String::new();
            let mut email = String::new();

            for line in card_body.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with("FN:") {
                    fn_name = trimmed.trim_start_matches("FN:").trim().to_string();
                } else if trimmed.starts_with("ORG:") {
                    org = trimmed.trim_start_matches("ORG:").replace(';', " - ").trim().to_string();
                } else if trimmed.starts_with("TITLE:") {
                    title = trimmed.trim_start_matches("TITLE:").trim().to_string();
                } else if trimmed.starts_with("TEL") && trimmed.contains(':') {
                    tel = trimmed.split_once(':').map(|(_, v)| v.trim().to_string()).unwrap_or_default();
                } else if trimmed.starts_with("EMAIL") && trimmed.contains(':') {
                    email = trimmed.split_once(':').map(|(_, v)| v.trim().to_string()).unwrap_or_default();
                }
            }

            if !fn_name.is_empty() || !tel.is_empty() {
                let key = format!("vcard:{}:{}:{}", fn_name, tel, email);
                if seen.insert(key) {
                    let display_val = if !fn_name.is_empty() && !tel.is_empty() {
                        format!("{} ({})", fn_name, tel)
                    } else if !fn_name.is_empty() {
                        fn_name.clone()
                    } else {
                        tel.clone()
                    };

                    artifacts.push(ForensicTaxonomyArtifact {
                        id: generate_id(),
                        domain_id: "phone_contacts".to_string(),
                        subcategory_id: "vcard_contacts".to_string(),
                        title: "vCard (.vcf) Contact Card".to_string(),
                        primary_value: display_val,
                        secondary_value: Some(org.clone()),
                        details: format!("vCard Contact: Name='{}', Org='{}', Title='{}', Phone='{}', Email='{}'", fn_name, org, title, tel, email),
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
        }
    }

    // 2. International Dialing Prefix Pattern (+1, +44, +49, +971, +234, etc.)
    if full_text.contains('+') || full_text.contains("00") {
        for cap in re.phone_intl.captures_iter(full_text) {
            let dial_code = cap[1].trim();
            let full_match = cap[0].trim();
            let digits: String = full_match.chars().filter(|c| c.is_ascii_digit()).collect();

            if is_valid_phone_candidate(full_match, &digits, Some(dial_code), false) {
                let key = format!("intl_phone:{}", digits);
                if seen.insert(key) {
                    let (country_name, flag) = lookup_country(dial_code);
                    let formatted_number = if full_match.starts_with('+') {
                        full_match.to_string()
                    } else {
                        format!("+{}", full_match.trim_start_matches("00"))
                    };

                    artifacts.push(ForensicTaxonomyArtifact {
                        id: generate_id(),
                        domain_id: "phone_contacts".to_string(),
                        subcategory_id: "intl_phones".to_string(),
                        title: format!("{} Phone ({})", flag, country_name),
                        primary_value: formatted_number.clone(),
                        secondary_value: Some(format!("{} {}", flag, country_name)),
                        details: format!("International Telephone Number: {} [Country: {} {}]", formatted_number, flag, country_name),
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
        }
    }

    // 3. National Mobile & Local Formats (e.g. Nigeria 08143893443, UK 07911 123456, UAE 050 123 4567)
    for cap in re.phone_national.captures_iter(full_text) {
        let full_match = cap[1].trim();
        let digits: String = full_match.chars().filter(|c| c.is_ascii_digit()).collect();

        if let Some((nat_name, flag, intl_norm)) = lookup_national_format(&digits) {
            if is_valid_phone_candidate(full_match, &digits, None, false) {
                let key = format!("nat_phone:{}", digits);
                if seen.insert(key) {
                    artifacts.push(ForensicTaxonomyArtifact {
                        id: generate_id(),
                        domain_id: "phone_contacts".to_string(),
                        subcategory_id: "intl_phones".to_string(),
                        title: format!("{} {} Line", flag, nat_name),
                        primary_value: full_match.to_string(),
                        secondary_value: Some(format!("Intl: {}", intl_norm)),
                        details: format!("National Number: {} (Normalized: {}) [Region: {} {}]", full_match, intl_norm, flag, nat_name),
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
        }
    }

    // 4. Raw International Prefixes without + (e.g. 2348143893443, 447911123456)
    for cap in re.phone_raw_prefix.captures_iter(full_text) {
        let digits = cap[1].trim();
        if let Some((raw_name, flag, intl_norm)) = lookup_raw_prefix(digits) {
            if is_valid_phone_candidate(digits, digits, None, false) {
                let key = format!("raw_intl:{}", digits);
                if seen.insert(key) {
                    artifacts.push(ForensicTaxonomyArtifact {
                        id: generate_id(),
                        domain_id: "phone_contacts".to_string(),
                        subcategory_id: "intl_phones".to_string(),
                        title: format!("{} Phone ({})", flag, raw_name),
                        primary_value: intl_norm.clone(),
                        secondary_value: Some(format!("{} {}", flag, raw_name)),
                        details: format!("Raw International Number: {} [Country: {} {}]", intl_norm, flag, raw_name),
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
        }
    }

    // 5. Contextual Signature & Labelled Phone Numbers (Tel:, Mobile:, Direct:, WhatsApp:)
    if full_text_lower.contains("tel")
        || full_text_lower.contains("phone")
        || full_text_lower.contains("mobile")
        || full_text_lower.contains("cell")
        || full_text_lower.contains("direct")
        || full_text_lower.contains("office")
        || full_text_lower.contains("whatsapp")
        || full_text_lower.contains("fax")
    {
        for cap in re.phone_contextual.captures_iter(full_text) {
            let num_str = cap[1].trim();
            let digits: String = num_str.chars().filter(|c| c.is_ascii_digit()).collect();

            if is_valid_phone_candidate(num_str, &digits, None, true) {
                let key = format!("sig_phone:{}", digits);
                if seen.insert(key) {
                    let dial_prefix = if num_str.starts_with('+') {
                        digits.chars().take(3).collect::<String>()
                    } else {
                        "".to_string()
                    };

                    let (country_name, flag) = if !dial_prefix.is_empty() {
                        lookup_country(&dial_prefix)
                    } else if digits.len() == 10 {
                        lookup_country("1") // US/Canada 10-digit default
                    } else {
                        ("Direct Phone Line", "📞")
                    };

                    artifacts.push(ForensicTaxonomyArtifact {
                        id: generate_id(),
                        domain_id: "phone_contacts".to_string(),
                        subcategory_id: "signature_contacts".to_string(),
                        title: format!("{} Signature Phone ({})", flag, country_name),
                        primary_value: num_str.to_string(),
                        secondary_value: Some(format!("From: {}", from_addr)),
                        details: format!("Extracted Contact Line from Signature: {} ({})", num_str, country_name),
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
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_country_lookups() {
        assert_eq!(lookup_country("1"), ("United States / Canada", "🇺🇸/🇨🇦"));
        assert_eq!(lookup_country("44"), ("United Kingdom", "🇬🇧"));
        assert_eq!(lookup_country("971"), ("United Arab Emirates", "🇦🇪"));
        assert_eq!(lookup_country("49"), ("Germany", "🇩🇪"));
        assert_eq!(lookup_country("91"), ("India", "🇮🇳"));
        assert_eq!(lookup_country("234"), ("Nigeria", "🇳🇬"));
    }

    #[test]
    fn test_nigerian_and_national_formats() {
        // Nigerian national format 08143893443 -> Nigeria 🇳🇬
        let nat = lookup_national_format("08143893443");
        assert!(nat.is_some());
        let (name, flag, intl) = nat.unwrap();
        assert_eq!(flag, "🇳🇬");
        assert_eq!(intl, "+234 8143893443");

        // Raw 234 format 2348143893443 -> Nigeria 🇳🇬
        let raw = lookup_raw_prefix("2348143893443");
        assert!(raw.is_some());
        let (r_name, r_flag, r_intl) = raw.unwrap();
        assert_eq!(r_flag, "🇳🇬");
        assert_eq!(r_intl, "+2348143893443");

        // UK national format 07911123456 -> UK 🇬🇧
        let uk = lookup_national_format("07911123456");
        assert!(uk.is_some());
        assert_eq!(uk.unwrap().1, "🇬🇧");
    }

    #[test]
    fn test_phone_validation_precision() {
        // Valid international numbers (True Positives)
        assert!(is_valid_phone_candidate("+1 568 344 3443", "15683443443", Some("1"), false));
        assert!(is_valid_phone_candidate("+234 814 389 3443", "2348143893443", Some("234"), false));
        assert!(is_valid_phone_candidate("08143893443", "08143893443", None, false));
        assert!(is_valid_phone_candidate("2348143893443", "2348143893443", None, false));
        assert!(is_valid_phone_candidate("+44 20 7946 0958", "442079460958", Some("44"), false));
        assert!(is_valid_phone_candidate("+971 50 123 4567", "971501234567", Some("971"), false));
        assert!(is_valid_phone_candidate("Direct: 555-0199", "5550199", None, true));

        // False positives to reject (Internal numbers, short codes, dates, tracking floats)
        assert!(!is_valid_phone_candidate("+847715.590137096", "847715590137096", Some("84"), false));
        assert!(!is_valid_phone_candidate("+852453.916601740", "852453916601740", Some("852"), false));
        assert!(!is_valid_phone_candidate("+824502.921953660", "824502921953660", Some("82"), false));
        assert!(!is_valid_phone_candidate("+764978.140215089", "764978140215089", Some("7"), false));
        assert!(!is_valid_phone_candidate("+5112-05", "511205", Some("51"), false));
        assert!(!is_valid_phone_candidate("+443016", "443016", Some("44"), false));
        assert!(!is_valid_phone_candidate("+387247", "387247", Some("387"), false));
        assert!(!is_valid_phone_candidate("+1823144", "1823144", Some("1"), false));
        assert!(!is_valid_phone_candidate("+109359162", "109359162", Some("1"), false));
        assert!(!is_valid_phone_candidate("2026-08-29", "20260829", None, false)); // Date
        assert!(!is_valid_phone_candidate("1999/12/31", "19991231", None, false)); // Date
        assert!(!is_valid_phone_candidate("14:30:00", "143000", None, false)); // Time
        assert!(!is_valid_phone_candidate("192.168.1.1", "19216811", None, false)); // IP
        assert!(!is_valid_phone_candidate("0000000000", "0000000000", None, false)); // Dummy repeating
        assert!(!is_valid_phone_candidate("123", "123", None, true)); // Too short
    }
}
