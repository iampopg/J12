use tauri::State;
use crate::AppState;
use super::types::SearchQuery;
use super::tools::ai_search_emails;

/// Natural language query parser
pub fn parse_natural_language_query(input: &str) -> SearchQuery {
    let lower = input.to_lowercase();
    let mut query = SearchQuery::default();
    
    if let Some(from) = extract_pattern(&lower, &[
        "from ", "from:", "sender ", "sent by ",
    ]) {
        query.from = Some(from.to_string());
    }
    
    if let Some(to) = extract_pattern(&lower, &[
        "to ", "to:", "recipient ", "sent to ",
    ]) {
        query.to = Some(to.to_string());
    }
    
    if let Some(subject) = extract_pattern(&lower, &[
        "subject:", "about ", "regarding ", "re:",
    ]) {
        query.subject = Some(subject.to_string());
    }
    
    if lower.contains("before") {
        if let Some(date) = extract_date_after_keyword(&lower, "before") {
            query.date_to = Some(date);
        }
    }
    if lower.contains("after") {
        if let Some(date) = extract_date_after_keyword(&lower, "after") {
            query.date_from = Some(date);
        }
    }
    if lower.contains("between") {
        if let Some((from, to)) = extract_date_range(&lower, "between", "and") {
            query.date_from = Some(from);
            query.date_to = Some(to);
        }
    }
    if lower.contains("last week") || lower.contains("past week") {
        query.date_from = Some("7_days_ago".to_string());
    }
    if lower.contains("last month") || lower.contains("past month") {
        query.date_from = Some("30_days_ago".to_string());
    }
    if lower.contains("last year") || lower.contains("past year") {
        query.date_from = Some("365_days_ago".to_string());
    }
    
    if lower.contains("attachment") || lower.contains("attached") {
        query.has_attachments = Some(true);
        
        if lower.contains("pdf") {
            query.attachment_types = Some(vec!["application/pdf".to_string()]);
        } else if lower.contains("image") || lower.contains("photo") || lower.contains("picture") {
            query.attachment_types = Some(vec!["image/jpeg".to_string(), "image/png".to_string(), "image/gif".to_string()]);
        } else if lower.contains("document") || lower.contains("doc") {
            query.attachment_types = Some(vec!["application/msword".to_string(), "application/vnd.openxmlformats-officedocument.wordprocessingml.document".to_string()]);
        } else if lower.contains("spreadsheet") || lower.contains("excel") {
            query.attachment_types = Some(vec!["application/vnd.ms-excel".to_string(), "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet".to_string()]);
        }
    }
    
    if lower.contains("suspicious") || lower.contains("risky") || lower.contains("dangerous") {
        query.risk_score_min = Some(50);
    }
    if lower.contains("high risk") || lower.contains("critical") {
        query.risk_score_min = Some(75);
    }
    
    if lower.contains("inbox") {
        query.folder_category = Some("inbox".to_string());
    } else if lower.contains("sent") {
        query.folder_category = Some("sent".to_string());
    } else if lower.contains("deleted") || lower.contains("trash") {
        query.folder_category = Some("soft_deleted".to_string());
    } else if lower.contains("spam") || lower.contains("junk") {
        query.folder_category = Some("spam".to_string());
    } else if lower.contains("draft") {
        query.folder_category = Some("drafts".to_string());
    }
    
    let keywords: Vec<&str> = lower
        .split_whitespace()
        .filter(|w| !is_stop_word(w) && !is_query_operator(w))
        .collect();
    
    if !keywords.is_empty() {
        query.text = Some(keywords.join(" "));
    }
    
    query
}

fn extract_pattern(input: &str, patterns: &[&str]) -> Option<String> {
    for pattern in patterns {
        if let Some(pos) = input.find(pattern) {
            let start = pos + pattern.len();
            let remaining = &input[start..];
            let end = remaining.find(|c: char| c == ' ' || c == ',' || c == '.')
                .unwrap_or(remaining.len());
            let value = remaining[..end].trim();
            if !value.is_empty() {
                return Some(value.to_string());
            }
        }
    }
    None
}

fn extract_date_after_keyword(input: &str, keyword: &str) -> Option<String> {
    if let Some(pos) = input.find(keyword) {
        let after = &input[pos + keyword.len()..];
        let date_str = after.trim().split_whitespace().next()?;
        if date_str.contains('-') && date_str.len() >= 8 {
            return Some(date_str.to_string());
        }
    }
    None
}

fn extract_date_range(input: &str, start_kw: &str, end_kw: &str) -> Option<(String, String)> {
    if let Some(start_pos) = input.find(start_kw) {
        let after_start = &input[start_pos + start_kw.len()..];
        if let Some(end_pos) = after_start.find(end_kw) {
            let from_date = after_start[..end_pos].trim();
            let to_date = after_start[end_pos + end_kw.len()..].trim().split_whitespace().next()?;
            return Some((from_date.to_string(), to_date.to_string()));
        }
    }
    None
}

fn is_stop_word(word: &str) -> bool {
    matches!(word, "the" | "a" | "an" | "is" | "are" | "was" | "were" | "be" | "been" | "being" | "have" | "has" | "had" | "do" | "does" | "did" | "will" | "would" | "could" | "should" | "may" | "might" | "can" | "find" | "show" | "get" | "me" | "my" | "we" | "our" | "you" | "your" | "they" | "their" | "it" | "its" | "this" | "that" | "these" | "those" | "i" | "he" | "she" | "all" | "any" | "each" | "every" | "both" | "few" | "more" | "most" | "other" | "some" | "such" | "no" | "nor" | "not" | "only" | "own" | "same" | "so" | "than" | "too" | "very" | "just" | "because" | "as" | "until" | "while" | "of" | "at" | "by" | "for" | "with" | "about" | "against" | "between" | "into" | "through" | "during" | "before" | "after" | "above" | "below" | "to" | "from" | "up" | "down" | "in" | "out" | "on" | "off" | "over" | "under" | "again" | "further" | "then" | "once" | "here" | "there" | "when" | "where" | "why" | "how" | "what" | "which" | "who" | "whom")
}

fn is_query_operator(word: &str) -> bool {
    matches!(word, "emails" | "email" | "messages" | "message" | "mail" | "mails" | "with" | "and" | "or" | "containing" | "that" | "have" | "been" | "sent" | "received")
}

/// Evidence explainer
pub fn explain_evidence(evidence_type: &str, evidence_data: &serde_json::Value) -> String {
    match evidence_type {
        "authentication_results" => explain_authentication(evidence_data),
        "received_header" => explain_received_header(evidence_data),
        "spf_result" => explain_spf(evidence_data),
        "dkim_result" => explain_dkim(evidence_data),
        "dmarc_result" => explain_dmarc(evidence_data),
        "attachment_analysis" => explain_attachment(evidence_data),
        "email_headers" => explain_headers(evidence_data),
        _ => format!("Evidence type '{}' is not recognized. Cannot provide explanation.", evidence_type),
    }
}

fn explain_authentication(data: &serde_json::Value) -> String {
    let spf = data.get("spf_result").and_then(|v| v.as_str()).unwrap_or("unknown");
    let dkim = data.get("dkim_result").and_then(|v| v.as_str()).unwrap_or("unknown");
    let dmarc = data.get("dmarc_result").and_then(|v| v.as_str()).unwrap_or("unknown");
    
    let mut explanation = String::from("## Authentication Results Explanation\n\n");
    
    explanation.push_str(&format!("**SPF (Sender Policy Framework):** {}\n", explain_spf_value(spf)));
    explanation.push_str(&format!("**DKIM (DomainKeys Identified Mail):** {}\n", explain_dkim_value(dkim)));
    explanation.push_str(&format!("**DMARC (Domain-based Message Authentication):** {}\n\n", explain_dmarc_value(dmarc)));
    
    if spf == "pass" && dkim == "pass" && dmarc == "pass" {
        explanation.push_str("**Overall:** All authentication checks passed. The email appears to be legitimately sent from the claimed domain.\n");
    } else if spf == "fail" || dkim == "fail" || dmarc == "fail" {
        explanation.push_str("**Overall:** One or more authentication checks failed. This email may be spoofed or sent from an unauthorized server.\n");
    } else {
        explanation.push_str("**Overall:** Some authentication checks are missing or inconclusive. Exercise caution with this email.\n");
    }
    
    explanation
}

fn explain_spf_value(result: &str) -> &str {
    match result {
        "pass" => "PASS - The sending IP is authorized by the sender domain's SPF record.",
        "fail" => "FAIL - The sending IP is NOT authorized by the sender domain's SPF record.",
        "softfail" => "SOFTFAIL - The sending IP is not explicitly authorized.",
        "none" => "NONE - No SPF record exists for the sender domain.",
        "neutral" => "NEUTRAL - The SPF record explicitly states no assertion about this IP.",
        _ => "UNKNOWN - Could not determine SPF result.",
    }
}

fn explain_dkim_value(result: &str) -> &str {
    match result {
        "pass" => "PASS - The email's DKIM signature is valid and matches the sender's published key.",
        "fail" => "FAIL - The DKIM signature is invalid or missing.",
        "none" => "NONE - No DKIM signature was applied.",
        _ => "UNKNOWN - Could not determine DKIM result.",
    }
}

fn explain_dmarc_value(result: &str) -> &str {
    match result {
        "pass" => "PASS - DMARC validation passed.",
        "fail" => "FAIL - DMARC validation failed.",
        "none" => "NONE - No DMARC policy exists for the sender domain.",
        _ => "UNKNOWN - Could not determine DMARC result.",
    }
}

fn explain_received_header(data: &serde_json::Value) -> String {
    let mut explanation = String::from("## Received Header Explanation\n\n");
    explanation.push_str("The `Received` header traces the path an email took from sender to receiver.\n\n");
    
    if let Some(chain) = data.get("received_chain").and_then(|v| v.as_array()) {
        explanation.push_str("**Email Path:**\n\n");
        for (i, hop) in chain.iter().enumerate() {
            if let Some(hop_str) = hop.as_str() {
                explanation.push_str(&format!("{}. {}\n", i + 1, hop_str));
            }
        }
    }
    
    explanation
}

fn explain_spf(data: &serde_json::Value) -> String {
    let result = data.get("result").and_then(|v| v.as_str()).unwrap_or("unknown");
    format!("## SPF Result: {}\n\n{}", result.to_uppercase(), explain_spf_value(result))
}

fn explain_dkim(data: &serde_json::Value) -> String {
    let result = data.get("result").and_then(|v| v.as_str()).unwrap_or("unknown");
    format!("## DKIM Result: {}\n\n{}", result.to_uppercase(), explain_dkim_value(result))
}

fn explain_dmarc(data: &serde_json::Value) -> String {
    let result = data.get("result").and_then(|v| v.as_str()).unwrap_or("unknown");
    format!("## DMARC Result: {}\n\n{}", result.to_uppercase(), explain_dmarc_value(result))
}

fn explain_attachment(data: &serde_json::Value) -> String {
    let mut explanation = String::from("## Attachment Analysis\n\n");
    
    if let Some(filename) = data.get("filename").and_then(|v| v.as_str()) {
        explanation.push_str(&format!("**Filename:** {}\n", filename));
    }
    if let Some(mime) = data.get("mime_type").and_then(|v| v.as_str()) {
        explanation.push_str(&format!("**MIME Type:** {}\n", mime));
    }
    if let Some(size) = data.get("size_bytes").and_then(|v| v.as_i64()) {
        explanation.push_str(&format!("**Size:** {} bytes\n", size));
    }
    if let Some(entropy) = data.get("entropy").and_then(|v| v.as_f64()) {
        explanation.push_str(&format!("**Entropy:** {:.2}/8.0\n", entropy));
    }
    if let Some(risk_flags) = data.get("risk_flags").and_then(|v| v.as_array()) {
        explanation.push_str("\n**Risk Flags:**\n");
        for flag in risk_flags {
            if let Some(f) = flag.as_str() {
                explanation.push_str(&format!("- {}\n", explain_risk_flag(f)));
            }
        }
    }
    
    explanation
}

fn explain_risk_flag(flag: &str) -> &str {
    match flag {
        "executable" => "Executable file - Can run code on the target system",
        "macro_enabled" => "Contains macros - Can execute automated scripts",
        "high_entropy_encrypted" => "High entropy - Possibly encrypted or packed",
        "double_extension" => "Double extension - May disguise true file type",
        _ => flag,
    }
}

fn explain_headers(data: &serde_json::Value) -> String {
    let mut explanation = String::from("## Email Headers Explanation\n\n");
    
    if let Some(from) = data.get("from").and_then(|v| v.as_str()) {
        explanation.push_str(&format!("**From:** {}\n", from));
    }
    if let Some(to) = data.get("to").and_then(|v| v.as_str()) {
        explanation.push_str(&format!("**To:** {}\n", to));
    }
    if let Some(subject) = data.get("subject").and_then(|v| v.as_str()) {
        explanation.push_str(&format!("**Subject:** {}\n", subject));
    }
    if let Some(date) = data.get("date").and_then(|v| v.as_str()) {
        explanation.push_str(&format!("**Date:** {}\n", date));
    }
    
    explanation
}

/// Natural language search command
#[tauri::command]
pub async fn ai_natural_language_search(state: State<'_, AppState>, query: String) -> Result<serde_json::Value, String> {
    let search_query = parse_natural_language_query(&query);
    let parsed_query_json = serde_json::to_value(&search_query).unwrap_or_default();
    let results = ai_search_emails(state, search_query).await?;
    
    Ok(serde_json::json!({
        "query": query,
        "parsed_query": parsed_query_json,
        "results": results,
        "total": results.len(),
    }))
}

/// Explain evidence command
#[tauri::command]
pub async fn ai_explain_evidence(evidence_type: String, evidence_data: serde_json::Value) -> Result<String, String> {
    Ok(explain_evidence(&evidence_type, &evidence_data))
}
