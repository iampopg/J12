use chrono::{DateTime, Utc};
use super::mime::decode_mime_word;

pub fn clean_raw_email(s: &str) -> String {
    s.trim_matches(|c: char| c == '<' || c == '>' || c == '"' || c == '\'' || c == ',' || c == ';' || c == ' ')
     .trim()
     .to_string()
}

pub fn extract_email(s: &str) -> String {
    let s = s.trim();
    if s.is_empty() { return String::new(); }

    if let Some(start) = s.find('<') {
        if let Some(end) = s.rfind('>') {
            if end > start {
                let inside = s[start + 1..end].trim();
                if !inside.is_empty() {
                    let cleaned = clean_raw_email(inside);
                    if cleaned.contains('@') {
                        return cleaned;
                    }
                }
            }
        }
    }

    if s.contains("/O=") || s.contains("CN=") || s.contains("/o=") || s.contains("cn=") {
        if let Some(cn_idx) = s.to_uppercase().rfind("CN=") {
            let name = &s[cn_idx + 3..];
            let name = name.split(';').next().unwrap_or(name).split('/').next().unwrap_or(name).trim();
            let name_clean = name.trim_matches(|c| c == '<' || c == '>' || c == '"' || c == '\'');
            if !name_clean.is_empty() {
                return format!("{}@enron.com", name_clean.to_lowercase());
            }
        }
    }

    if let Some(at_idx) = s.find('@') {
        let before = &s[..at_idx];
        let after = &s[at_idx + 1..];

        let user_start = before
            .rfind(|c: char| c == ' ' || c == '<' || c == '"' || c == ':' || c == ',')
            .map(|i| i + 1)
            .unwrap_or(0);
        let user = before[user_start..].trim();

        let domain_end = after
            .find(|c: char| c == ' ' || c == '>' || c == '"' || c == ';' || c == ',' || c == '\r' || c == '\n')
            .unwrap_or(after.len());
        let domain = after[..domain_end].trim();

        if !user.is_empty() && !domain.is_empty() {
            let candidate = format!("{}@{}", user, domain);
            return clean_raw_email(&candidate);
        }
    }

    clean_raw_email(s)
}

pub fn extract_display_name(s: &str) -> Option<String> {
    let s = s.trim();
    if s.is_empty() { return None; }

    if let Some(start) = s.find('<') {
        let name_part = s[..start].trim();
        if !name_part.is_empty() {
            if let Some(cleaned) = clean_display_name_str(name_part) {
                return Some(cleaned);
            }
        }
    }

    if s.contains("/O=") || s.contains("CN=") || s.contains("OU=") {
        if let Some(cn_idx) = s.to_uppercase().rfind("CN=") {
            let name = &s[cn_idx + 3..];
            let name = name.split(';').next().unwrap_or(name).split('/').next().unwrap_or(name).trim();
            if let Some(cleaned) = clean_display_name_str(name) {
                return Some(cleaned);
            }
        }
    }

    clean_display_name_str(s)
}

pub fn clean_display_name_str(s: &str) -> Option<String> {
    let mut name = s
        .trim_matches(|c| c == '<' || c == '>' || c == '"' || c == '\'' || c == ';' || c == ',')
        .trim()
        .to_string();

    if name.is_empty() { return None; }

    if let Some(idx) = name.find("IMCEANOTES-") {
        name = name[..idx].trim().to_string();
    }
    if let Some(idx) = name.find("@ENRON") {
        name = name[..idx].trim().to_string();
    }
    if let Some(idx) = name.find("@enron") {
        name = name[..idx].trim().to_string();
    }

    if name.contains(',') && !name.contains('@') {
        let parts: Vec<&str> = name.split(',').map(|p| p.trim()).collect();
        if parts.len() == 2 && !parts[0].is_empty() && !parts[1].is_empty() {
            name = format!("{} {}", parts[1], parts[0]);
        }
    }

    let trimmed = name
        .trim_matches(|c| c == '<' || c == '>' || c == '"' || c == '\'' || c == ',' || c == ';')
        .trim();

    if trimmed.is_empty() || (trimmed.contains('@') && !trimmed.contains(' ')) {
        return None;
    }

    Some(trimmed.to_string())
}

pub fn split_address_list(s: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    let mut bracket_depth = 0;

    let chars: Vec<char> = s.chars().collect();
    let len = chars.len();
    let mut i = 0;

    while i < len {
        let c = chars[i];
        if c == '"' {
            in_quotes = !in_quotes;
            current.push(c);
        } else if c == '<' {
            bracket_depth += 1;
            current.push(c);
        } else if c == '>' {
            if bracket_depth > 0 { bracket_depth -= 1; }
            current.push(c);
        } else if c == ';' {
            if !in_quotes && bracket_depth == 0 {
                let trimmed = current.trim().to_string();
                if !trimmed.is_empty() { parts.push(trimmed); }
                current.clear();
            } else {
                current.push(c);
            }
        } else if c == ',' {
            if in_quotes || bracket_depth > 0 {
                current.push(c);
            } else {
                let has_email_in_current = current.contains('@') || current.contains('<');
                let remaining: String = chars[i+1..].iter().collect();
                let next_delim = remaining.find(|ch| ch == ';' || ch == ',');
                let next_chunk = match next_delim {
                    Some(idx) => &remaining[..idx],
                    None => &remaining,
                };
                let next_has_bracket = next_chunk.contains('<') && next_chunk.contains('>');
                let next_has_at = next_chunk.contains('@');

                if !has_email_in_current && (next_has_bracket || next_has_at) {
                    current.push(c);
                } else {
                    let trimmed = current.trim().to_string();
                    if !trimmed.is_empty() { parts.push(trimmed); }
                    current.clear();
                }
            }
        } else {
            current.push(c);
        }
        i += 1;
    }

    let trimmed = current.trim().to_string();
    if !trimmed.is_empty() { parts.push(trimmed); }
    parts
}

pub fn clean_exchange_name(s: &str) -> String {
    clean_display_name_str(s).unwrap_or_else(|| s.trim().to_string())
}

pub fn extract_address_list(s: &str) -> Vec<String> {
    split_address_list(s)
        .into_iter()
        .map(|a| extract_email(&a))
        .filter(|a| !a.is_empty() && a != "unknown@unknown")
        .collect()
}

pub fn extract_address_list_with_names(s: &str) -> Vec<(String, Option<String>)> {
    split_address_list(s).into_iter().filter_map(|part| {
        let part = part.trim();
        if part.is_empty() { return None; }
        let email = extract_email(part);
        if email.is_empty() { return None; }
        let name = extract_display_name(part);
        Some((email, name))
    }).collect()
}

pub fn parse_date(s: &str) -> Option<DateTime<Utc>> {
    let s = s.trim();
    if let Ok(dt) = DateTime::parse_from_rfc2822(s) { return Some(dt.with_timezone(&Utc)); }
    for fmt in &["%a, %d %b %Y %H:%M:%S %z", "%d %b %Y %H:%M:%S %z", "%Y-%m-%dT%H:%M:%S%z", "%a, %d %b %Y %H:%M:%S"] {
        if let Ok(dt) = DateTime::parse_from_str(s, fmt) { return Some(dt.with_timezone(&Utc)); }
    }
    None
}

pub fn extract_ip(s: &str) -> String {
    if let Some(start) = s.find('[') {
        if let Some(end) = s[start..].find(']') {
            return s[start+1..start+end].to_string();
        }
    }
    for word in s.split(|c: char| c == ' ' || c == '\t' || c == '(' || c == ')' || c == ',') {
        let word = word.trim();
        if word.split('.').count() == 4 && word.chars().all(|c| c.is_digit(10) || c == '.') {
            return word.to_string();
        }
    }
    String::new()
}
