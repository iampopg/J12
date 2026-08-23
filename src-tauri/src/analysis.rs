//! Forensic Analysis Engine
//! 
//! Implements Phase 3 analysis modules:
//! - Header analysis (Received chain, clock skew, routing anomalies)
//! - Authentication (SPF/DKIM/DMARC/ARC verification)
//! - Spoofing detection (display name, homoglyph, From/Return-Path mismatch)
//! - Attachment analysis (magic bytes, entropy, extension mismatch)
//! - Findings engine (type, severity, confidence, status tracking)

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Analysis result for a single email
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisResult {
    pub email_id: String,
    pub header_analysis: HeaderAnalysis,
    pub auth_results: AuthResults,
    pub spoof_findings: Vec<SpoofingFinding>,
    pub attachment_analysis: Vec<AttachmentAnalysis>,
    pub risk_score: u8, // 0-100
    pub flags: Vec<String>,
}

/// Header analysis output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeaderAnalysis {
    pub received_chain: Vec<Hop>,
    pub clock_skew: Vec<SkewEvent>,
    pub originating_ip: Option<String>,
    pub routing_anomalies: Vec<Anomaly>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hop {
    pub from: Option<String>,
    pub by: Option<String>,
    pub with: Option<String>,
    pub id: Option<String>,
    pub for_addr: Option<String>,
    pub timestamp: Option<String>,
    pub transit_time_seconds: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkewEvent {
    pub hop_from: String,
    pub hop_to: String,
    pub expected_order: String,
    pub actual_order: String,
    pub skew_seconds: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Anomaly {
    pub anomaly_type: String,
    pub description: String,
    pub severity: String, // low|medium|high|critical
}

/// Authentication results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthResults {
    pub spf: AuthCheck,
    pub dkim: Vec<AuthCheck>,
    pub dmarc: AuthCheck,
    pub arc: Vec<ArcSeal>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthCheck {
    pub result: String, // pass|fail|none|neutral|permerror|temperror
    pub identity: Option<String>,
    pub domain: Option<String>,
    pub aligned: bool,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArcSeal {
    pub instance: u32,
    pub result: String,
    pub cv: String,
}

/// Spoofing finding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpoofingFinding {
    pub finding_type: String,
    pub severity: String,
    pub confidence: String,
    pub title: String,
    pub description: String,
    pub indicator: String,
}

/// Attachment analysis output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttachmentAnalysis {
    pub filename: Option<String>,
    pub declared_mime: String,
    pub detected_type: String,
    pub extension_match: bool,
    pub entropy: f64,
    pub risk_flags: Vec<String>,
    pub risk_score: u8,
}

// ─────────────────────────────────────────────────────────────────────────────
// HEADER ANALYSIS
// ─────────────────────────────────────────────────────────────────────────────

/// Parse the Received chain from raw headers
pub fn analyze_headers(headers_raw: &str) -> HeaderAnalysis {
    let mut hops: Vec<Hop> = Vec::new();
    let mut anomalies = Vec::new();
    
    // Extract all Received lines (they're in reverse chronological order in raw headers)
    let received_lines: Vec<&str> = headers_raw.lines()
        .filter(|l| l.starts_with("Received:"))
        .collect();
    
    if received_lines.is_empty() {
        // Check if this might be an internal message (e.g., Exchange)
        let has_transport_headers = headers_raw.lines().any(|l| {
            let lower = l.to_lowercase();
            lower.starts_with("x-from:") || lower.starts_with("x-to:") || 
            lower.starts_with("x-cc:") || lower.starts_with("x-message-id:") ||
            lower.starts_with("x-mailer:")
        });
        
        if has_transport_headers {
            // Internal message (Exchange, Notes, etc.) — no Received headers is normal
            anomalies.push(Anomaly {
                anomaly_type: "no_received_internal".to_string(),
                description: "No Received headers — message appears to be internal (Exchange/Notes transport headers present)".to_string(),
                severity: "low".to_string(),
            });
        } else {
            anomalies.push(Anomaly {
                anomaly_type: "missing_received".to_string(),
                description: "No Received headers found — possible direct injection or header manipulation".to_string(),
                severity: "medium".to_string(),
            });
        }
    }
    
    // Parse each hop (bottom-up = oldest first)
    for (idx, line) in received_lines.iter().enumerate() {
        let hop = parse_hop(line);
        
        // Check for transit time anomalies
        if idx > 0 {
            let prev_ts = &hops[idx - 1].timestamp;
            if let (Some(ts), Some(prev_ts)) = (&hop.timestamp, prev_ts) {
                // In a valid chain, hop timestamps should be increasing as we go up
                // (each later hop is more recent)
                if let (Some(curr_secs), Some(prev_secs)) = (
                    parse_timestamp_to_seconds(ts),
                    parse_timestamp_to_seconds(prev_ts)
                ) {
                    let transit = curr_secs - prev_secs;
                    if transit < 0 {
                        anomalies.push(Anomaly {
                            anomaly_type: "timestamp_reversal".to_string(),
                            description: format!(
                                "Negative transit time between hop {} and {}: {} seconds",
                                idx, idx - 1, transit
                            ),
                            severity: "medium".to_string(),
                        });
                    } else if transit > 3600 {
                        anomalies.push(Anomaly {
                            anomaly_type: "long_transit".to_string(),
                            description: format!(
                                "Unusually long transit time between hop {} and {}: {} seconds",
                                idx, idx - 1, transit
                            ),
                            severity: "low".to_string(),
                        });
                    }
                }
            }
        }
        
        hops.push(hop);
    }
    
    // Reverse to get bottom-up order (oldest first)
    hops.reverse();
    
    // Calculate transit times
    let mut hops_with_transit: Vec<Hop> = Vec::new();
    for (idx, mut hop) in hops.into_iter().enumerate() {
        if idx > 0 {
            let prev_ts = &hops_with_transit[idx - 1].timestamp;
            if let (Some(ts), Some(prev_ts)) = (&hop.timestamp, prev_ts) {
                if let (Some(curr_secs), Some(prev_secs)) = (
                    parse_timestamp_to_seconds(ts),
                    parse_timestamp_to_seconds(prev_ts)
                ) {
                    hop.transit_time_seconds = Some(curr_secs - prev_secs);
                }
            }
        }
        hops_with_transit.push(hop);
    }
    
    // Extract originating IP (from the first hop = last in raw order)
    let originating_ip = if hops_with_transit.is_empty() {
        // No Received headers — try X-Originating-IP
        headers_raw.lines()
            .find(|l| l.to_lowercase().starts_with("x-originating-ip:"))
            .and_then(|l| l.split_once(':'))
            .map(|(_, v)| v.trim().to_string())
    } else {
        hops_with_transit.first()
            .and_then(|hop| {
                let from = hop.from.as_deref().unwrap_or("");
                if from.is_empty() {
                    None
                } else {
                    extract_ip_from_received(from)
                }
            })
    };
    
    // Detect suspicious relay patterns
    if hops_with_transit.len() > 10 {
        anomalies.push(Anomaly {
            anomaly_type: "excessive_hops".to_string(),
            description: format!("Unusually high number of hops: {}", hops_with_transit.len()),
            severity: "medium".to_string(),
        });
    }
    
    // Check for missing hops (gaps in Received chain)
    let mut clock_skew = Vec::new();
    for window in hops_with_transit.windows(2) {
        let curr_ts = &window[0].timestamp;
        let prev_ts = &window[1].timestamp;
        if let (Some(curr_ts), Some(prev_ts)) = (curr_ts, prev_ts) {
            if let (Some(curr_secs), Some(prev_secs)) = (
                parse_timestamp_to_seconds(curr_ts),
                parse_timestamp_to_seconds(prev_ts)
            ) {
                let skew = prev_secs - curr_secs;
                if skew.abs() > 300 { // > 5 minutes
                    clock_skew.push(SkewEvent {
                        hop_from: window[0].from.clone().unwrap_or_default(),
                        hop_to: window[1].from.clone().unwrap_or_default(),
                        expected_order: "increasing".to_string(),
                        actual_order: if skew > 0 { "reversed".to_string() } else { "large_gap".to_string() },
                        skew_seconds: skew,
                    });
                }
            }
        }
    }
    
    HeaderAnalysis {
        received_chain: hops_with_transit,
        clock_skew,
        originating_ip,
        routing_anomalies: anomalies,
    }
}

/// Parse a single Received header line into a Hop
fn parse_hop(line: &str) -> Hop {
    let content = line.strip_prefix("Received:").unwrap_or(line).trim();
    
    let from = extract_field(content, "from").map(|s| s.trim().to_string());
    let by = extract_field(content, "by").map(|s| s.trim().to_string());
    let with = extract_field(content, "with").map(|s| s.trim().to_string());
    let id = extract_field(content, "id").map(|s| s.trim().to_string());
    let for_addr = extract_field(content, "for").map(|s| s.trim().to_string());
    
    // Extract timestamp (after semicolon at end)
    let timestamp = content.rfind(';').map(|idx| content[idx + 1..].trim().to_string());
    
    Hop {
        from,
        by,
        with,
        id,
        for_addr,
        timestamp,
        transit_time_seconds: None,
    }
}

/// Extract a field from a Received header
fn extract_field(content: &str, field: &str) -> Option<String> {
    let lower = content.to_lowercase();
    let pattern = format!("{} ", field);
    
    if let Some(start) = lower.find(&pattern) {
        let value_start = start + pattern.len();
        let remaining = &content[value_start..];
        
        // Find the end of this value (next field keyword, semicolon, or end)
        let end_markers = [" ;", "  ", "\t"];
        let mut end = remaining.len();
        for marker in &end_markers {
            if let Some(pos) = remaining.find(marker) {
                if pos < end {
                    end = pos;
                }
            }
        }
        
        let value = remaining[..end].trim().trim_end_matches(';').trim();
        if !value.is_empty() {
            Some(value.to_string())
        } else {
            None
        }
    } else {
        None
    }
}

/// Extract IP address from a hostname or direct IP
fn extract_ip_from_received(host: &str) -> Option<String> {
    // Look for IP in brackets: [192.168.1.1]
    if let (Some(start), Some(end)) = (host.find('['), host.find(']')) {
        if start < end {
            let ip = &host[start + 1..end];
            if ip.parse::<std::net::IpAddr>().is_ok() {
                return Some(ip.to_string());
            }
        }
    }
    // Look for IPv4 pattern
    let ip_regex = regex::Regex::new(r"\b(\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3})\b").ok()?;
    ip_regex.captures(host).and_then(|caps| caps.get(1)).map(|m| m.as_str().to_string())
}

/// Parse timestamp string to seconds since epoch (approximate for comparison)
fn parse_timestamp_to_seconds(ts: &str) -> Option<i64> {
    // Try common email date formats
    use chrono::DateTime;
    
    // RFC 2822 format: "Mon, 15 Jan 2001 09:15:00 -0500"
    if let Ok(dt) = DateTime::parse_from_rfc2822(ts) {
        return Some(dt.timestamp());
    }
    
    // Try without day name: "15 Jan 2001 09:15:00 -0500"
    if let Ok(dt) = DateTime::parse_from_str(ts, "%d %b %Y %H:%M:%S %z") {
        return Some(dt.timestamp());
    }
    
    // Try ISO 8601
    if let Ok(dt) = DateTime::parse_from_rfc3339(ts) {
        return Some(dt.timestamp());
    }
    
    // Fallback: just try chrono's flexible parsing
    if let Ok(dt) = ts.parse::<DateTime<chrono::FixedOffset>>() {
        return Some(dt.timestamp());
    }
    
    None
}

// ─────────────────────────────────────────────────────────────────────────────
// AUTHENTICATION ANALYSIS
// ─────────────────────────────────────────────────────────────────────────────

/// Analyze email authentication from headers
pub fn analyze_authentication(
    headers_raw: &str,
    from_domain: &str,
    source_ip: Option<&str>,
) -> AuthResults {
    let auth_results_header = extract_authentication_results(headers_raw);
    
    AuthResults {
        spf: analyze_spf(headers_raw, from_domain, source_ip, &auth_results_header),
        dkim: analyze_dkim(headers_raw, &auth_results_header),
        dmarc: analyze_dmarc(headers_raw, from_domain, &auth_results_header),
        arc: analyze_arc(headers_raw),
    }
}

/// Find Authentication-Results headers (these contain the receiving server's evaluation)
fn extract_authentication_results(headers_raw: &str) -> Vec<String> {
    headers_raw.lines()
        .filter(|l| l.to_lowercase().starts_with("authentication-results:"))
        .map(|l| l.strip_prefix("Authentication-Results:")
            .or_else(|| l.strip_prefix("authentication-results:"))
            .unwrap_or(l)
            .trim()
            .to_string())
        .collect()
}

/// Analyze SPF (Sender Policy Framework)
fn analyze_spf(
    headers_raw: &str,
    from_domain: &str,
    _source_ip: Option<&str>,
    auth_results: &[String],
) -> AuthCheck {
    // Check for explicit SPF result in Authentication-Results
    for result in auth_results {
        if result.to_lowercase().contains("spf=") {
            let lower = result.to_lowercase();
            
            if lower.contains("spf=pass") {
                // Extract domain
                let domain = lower.split("spf=pass")
                    .nth(1)
                    .and_then(|s| s.split_whitespace().next())
                    .and_then(|s| s.split('@').nth(1))
                    .map(|s| s.trim_matches(|c: char| !c.is_alphanumeric() && c != '.' && c != '-').to_string());
                
                return AuthCheck {
                    result: "pass".to_string(),
                    identity: domain.clone(),
                    domain,
                    aligned: true,
                    detail: "SPF validation passed".to_string(),
                };
            }
            
            if lower.contains("spf=fail") {
                return AuthCheck {
                    result: "fail".to_string(),
                    identity: Some(from_domain.to_string()),
                    domain: Some(from_domain.to_string()),
                    aligned: false,
                    detail: "SPF validation failed - sender not authorized".to_string(),
                };
            }
            
            if lower.contains("spf=softfail") {
                return AuthCheck {
                    result: "softfail".to_string(),
                    identity: Some(from_domain.to_string()),
                    domain: Some(from_domain.to_string()),
                    aligned: false,
                    detail: "SPF softfail - sender likely unauthorized".to_string(),
                };
            }
            
            if lower.contains("spf=none") {
                return AuthCheck {
                    result: "none".to_string(),
                    identity: Some(from_domain.to_string()),
                    domain: Some(from_domain.to_string()),
                    aligned: false,
                    detail: "No SPF record published for this domain".to_string(),
                };
            }
        }
    }
    
    // Check if the sending IP is authorized (simplified - would need DNS lookup for full SPF)
    // For now, check if the source IP is in the Received chain and matches the domain
    AuthCheck {
        result: "none".to_string(),
        identity: Some(from_domain.to_string()),
        domain: Some(from_domain.to_string()),
        aligned: false,
        detail: "No explicit SPF result found in headers - cannot verify without DNS lookup".to_string(),
    }
}

/// Analyze DKIM (DomainKeys Identified Mail)
fn analyze_dkim(headers_raw: &str, auth_results: &[String]) -> Vec<AuthCheck> {
    let mut results = Vec::new();
    
    // Check Authentication-Results for DKIM
    for result in auth_results {
        if result.to_lowercase().contains("dkim=") {
            let lower = result.to_lowercase();
            
            let dkim_result = if lower.contains("dkim=pass") {
                "pass"
            } else if lower.contains("dkim=fail") {
                "fail"
            } else if lower.contains("dkim=none") {
                "none"
            } else {
                "temperror"
            };
            
            // Extract domain from DKIM signature
            let domain = extract_dkim_domain(headers_raw);
            
            results.push(AuthCheck {
                result: dkim_result.to_string(),
                identity: domain.clone(),
                domain,
                aligned: dkim_result == "pass",
                detail: format!("DKIM validation: {}", dkim_result),
            });
        }
    }
    
    // Check for DKIM-Signature header (even if no Authentication-Results)
    if !headers_raw.lines().any(|l| l.to_lowercase().starts_with("dkim-signature:")) {
        if results.is_empty() {
            results.push(AuthCheck {
                result: "none".to_string(),
                identity: None,
                domain: None,
                aligned: false,
                detail: "No DKIM-Signature header present".to_string(),
            });
        }
    }
    
    results
}

/// Extract DKIM domain from DKIM-Signature header
fn extract_dkim_domain(headers_raw: &str) -> Option<String> {
    for line in headers_raw.lines() {
        if line.to_lowercase().starts_with("dkim-signature:") {
            // Look for d= tag
            for part in line.split(';') {
                let trimmed = part.trim();
                if trimmed.starts_with("d=") {
                    return Some(trimmed[2..].trim().to_string());
                }
            }
        }
    }
    None
}

/// Analyze DMARC (Domain-based Message Authentication, Reporting & Conformance)
fn analyze_dmarc(headers_raw: &str, from_domain: &str, auth_results: &[String]) -> AuthCheck {
    // Check for DMARC result in Authentication-Results
    for result in auth_results {
        if result.to_lowercase().contains("dmarc=") {
            let lower = result.to_lowercase();
            
            let dmarc_result = if lower.contains("dmarc=pass") {
                "pass"
            } else if lower.contains("dmarc=fail") {
                "fail"
            } else if lower.contains("dmarc=none") {
                "none"
            } else {
                "temperror"
            };
            
            // Extract alignment info
            let aligned = lower.contains("dmarc=pass") || lower.contains("spf=pass") && lower.contains("dkim=pass");
            
            return AuthCheck {
                result: dmarc_result.to_string(),
                identity: Some(from_domain.to_string()),
                domain: Some(from_domain.to_string()),
                aligned,
                detail: format!("DMARC validation: {}", dmarc_result),
            };
        }
    }
    
    AuthCheck {
        result: "none".to_string(),
        identity: Some(from_domain.to_string()),
        domain: Some(from_domain.to_string()),
        aligned: false,
        detail: "No DMARC result in headers - check requires DNS lookup".to_string(),
    }
}

/// Analyze ARC (Authenticated Received Chain)
fn analyze_arc(headers_raw: &str) -> Vec<ArcSeal> {
    let mut seals = Vec::new();
    
    let arc_seals: Vec<&str> = headers_raw.lines()
        .filter(|l| l.to_lowercase().starts_with("arc-seal:"))
        .collect();
    
    let arc_results: Vec<&str> = headers_raw.lines()
        .filter(|l| l.to_lowercase().starts_with("arc-authentication-results:"))
        .collect();
    
    for (idx, seal) in arc_seals.iter().enumerate() {
        let content = seal.strip_prefix("ARC-Seal:").unwrap_or(seal).trim();
        
        let cv = content.split(';')
            .find_map(|p| {
                let trimmed = p.trim();
                if trimmed.starts_with("cv=") {
                    Some(trimmed[3..].trim().to_string())
                } else {
                    None
                }
            }).unwrap_or_else(|| "none".to_string());
        
        // Check corresponding ARC result
        let result = arc_results.get(idx)
            .map(|r| {
                let lower = r.to_lowercase();
                if lower.contains("pass") { "pass" } else { "fail" }
            })
            .unwrap_or("unknown");
        
        seals.push(ArcSeal {
            instance: (idx + 1) as u32,
            result: result.to_string(),
            cv,
        });
    }
    
    seals
}

// ─────────────────────────────────────────────────────────────────────────────
// SPOOFING DETECTION
// ─────────────────────────────────────────────────────────────────────────────

/// Detect spoofing attempts from email data
pub fn detect_spoofing(
    from_addr: &str,
    from_display: Option<&str>,
    headers_raw: &str,
    auth_results: &AuthResults,
) -> Vec<SpoofingFinding> {
    let mut findings = Vec::new();
    
    let from_domain = extract_domain(from_addr);
    
    // 1. Display name spoofing: display name contains email different from actual sender
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
        
        // Check for brand impersonation in display name
        let brands = ["paypal", "apple", "microsoft", "google", "amazon", "netflix", "bank", "wells fargo", "chase"];
        for brand in &brands {
            if display_lower.contains(brand) && !from_domain.contains(brand) {
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

// ─────────────────────────────────────────────────────────────────────────────
// ATTACHMENT ANALYSIS
// ─────────────────────────────────────────────────────────────────────────────

/// Analyze an attachment for forensic indicators
pub fn analyze_attachment(
    filename: Option<&str>,
    declared_mime: &str,
    data: &[u8],
) -> AttachmentAnalysis {
    let detected_type = detect_file_type(data);
    let extension_match = check_extension_match(filename, &detected_type);
    let entropy = calculate_entropy(data);
    
    let mut risk_flags = Vec::new();
    let mut risk_score: u8 = 0;
    
    // Flag: extension mismatch
    if !extension_match && filename.is_some() {
        risk_flags.push("extension_mismatch".to_string());
        risk_score += 20;
    }
    
    // Flag: dangerous extensions
    if let Some(name) = filename {
        let lower = name.to_lowercase();
        let dangerous_exts = [".exe", ".scr", ".pif", ".cmd", ".bat", ".com", ".vbs", ".js", ".wsf", ".ps1", ".msi"];
        for ext in &dangerous_exts {
            if lower.ends_with(ext) {
                risk_flags.push(format!("dangerous_extension: {}", ext));
                risk_score += 30;
                break;
            }
        }
        
        // Flag: double extension
        let parts: Vec<&str> = lower.split('.').collect();
        if parts.len() > 2 {
            let second_ext = format!(".{}", parts[parts.len() - 2]);
            if dangerous_exts.contains(&second_ext.as_str()) || second_ext == ".pdf" || second_ext == ".doc" || second_ext == ".xls" {
                risk_flags.push("double_extension".to_string());
                risk_score += 40;
            }
        }
        
        // Flag: extensionless file
        if !lower.contains('.') {
            risk_flags.push("no_extension".to_string());
            risk_score += 10;
        }
    }
    
    // Flag: high entropy (possibly encrypted/packed)
    if entropy > 7.5 {
        risk_flags.push("high_entropy: possibly encrypted".to_string());
        risk_score += 25;
    } else if entropy > 7.0 {
        risk_flags.push("elevated_entropy".to_string());
        risk_score += 10;
    }
    
    // Flag: macro-enabled Office document
    if is_office_document_with_macros(data) {
        risk_flags.push("office_macros_detected".to_string());
        risk_score += 35;
    }
    
    // Flag: executable disguised as document
    if detected_type == "application/x-dosexec" && 
       (declared_mime.contains("pdf") || declared_mime.contains("office") || declared_mime.contains("document")) {
        risk_flags.push("executable_disguised_as_document".to_string());
        risk_score += 50;
    }
    
    AttachmentAnalysis {
        filename: filename.map(|s| s.to_string()),
        declared_mime: declared_mime.to_string(),
        detected_type,
        extension_match,
        entropy,
        risk_flags,
        risk_score: risk_score.min(100),
    }
}

/// Detect file type from magic bytes
fn detect_file_type(data: &[u8]) -> String {
    if data.len() < 4 {
        return "application/octet-stream".to_string();
    }
    
    // Magic byte signatures
    match &data[0..4] {
        [0x25, 0x50, 0x44, 0x46] => "application/pdf".to_string(),           // %PDF
        [0x50, 0x4b, 0x03, 0x04] => {
            // ZIP-based (could be Office docx/xlsx, JAR, etc.)
            if data.windows(4).any(|w| w == b"[Con") {
                return "application/vnd.openxmlformats-officedocument.wordprocessingml.document".to_string();
            }
            "application/zip".to_string()
        },
        [0xd0, 0xcf, 0x11, 0xe0] => {
            // OLE2 compound document (old Office .doc/.xls/.msg)
            "application/vnd.ms-office".to_string()
        },
        [0x7f, 0x45, 0x4c, 0x46] => "application/x-elf".to_string(),           // ELF binary
        [0x4d, 0x5a, 0x90, 0x00] | [0x4d, 0x5a, 0x00, 0x00] => "application/x-dosexec".to_string(), // MZ executable
        [0x52, 0x61, 0x72, 0x21] => "application/x-rar-compressed".to_string(), // RAR
        [0x1f, 0x8b, 0x08, _] => "application/gzip".to_string(),              // GZIP
        [0x42, 0x5a, 0x68, _] => "application/x-bzip2".to_string(),          // BZIP2
        [0xFD, 0x37, 0x7A, 0x58] => "application/x-xz".to_string(),          // XZ
        [0x89, 0x50, 0x4E, 0x47] => "image/png".to_string(),                 // PNG
        [0xFF, 0xD8, 0xFF, _] => "image/jpeg".to_string(),                   // JPEG
        [0x47, 0x49, 0x46, 0x38] => "image/gif".to_string(),                 // GIF
        [0x42, 0x4d, _, _] => "image/bmp".to_string(),                       // BMP
        [0x49, 0x20, 0x49, 0x00] | [0x49, 0x49, 0x2a, 0x00] => "image/tiff".to_string(), // TIFF
        _ => {
            // Check for text content
            if data.iter().all(|&b| b == b'\n' || b == b'\r' || b == b'\t' || (b >= 0x20 && b < 0x7f)) {
                "text/plain".to_string()
            } else {
                "application/octet-stream".to_string()
            }
        }
    }
}

/// Check if filename extension matches detected type
fn check_extension_match(filename: Option<&str>, detected_type: &str) -> bool {
    let ext = match filename {
        Some(name) => {
            name.rfind('.').map(|i| &name[i..]).unwrap_or("")
        }
        None => return true, // No filename, can't mismatch
    };
    
    let ext_lower = ext.to_lowercase();
    
    match detected_type {
        "application/pdf" => ext_lower == ".pdf",
        "application/zip" => ext_lower == ".zip" || ext_lower == ".jar" || ext_lower == ".war",
        "application/vnd.openxmlformats-officedocument.wordprocessingml.document" => 
            ext_lower == ".docx" || ext_lower == ".xlsx" || ext_lower == ".pptx",
        "application/vnd.ms-office" => ext_lower == ".doc" || ext_lower == ".xls" || ext_lower == ".ppt" || ext_lower == ".msg",
        "application/x-dosexec" => ext_lower == ".exe" || ext_lower == ".dll" || ext_lower == ".sys",
        "application/x-rar-compressed" => ext_lower == ".rar",
        "application/gzip" => ext_lower == ".gz" || ext_lower == ".tgz" || ext_lower.ends_with(".tar.gz"),
        "image/png" => ext_lower == ".png",
        "image/jpeg" => ext_lower == ".jpg" || ext_lower == ".jpeg",
        "image/gif" => ext_lower == ".gif",
        "text/plain" => ext_lower == ".txt" || ext_lower == ".log" || ext_lower == ".csv" || ext_lower == ".text",
        _ => true, // Unknown type, don't flag
    }
}

/// Calculate Shannon entropy of data (0.0 - 8.0 for bytes)
fn calculate_entropy(data: &[u8]) -> f64 {
    if data.is_empty() {
        return 0.0;
    }
    
    let mut freq = [0u64; 256];
    for &byte in data {
        freq[byte as usize] += 1;
    }
    
    let len = data.len() as f64;
    let mut entropy = 0.0;
    
    for &count in &freq {
        if count > 0 {
            let p = count as f64 / len;
            entropy -= p * p.log2();
        }
    }
    
    entropy
}

/// Check if data contains Office document with macros (simplified)
fn is_office_document_with_macros(_data: &[u8]) -> bool {
    // Full OLE2/VBA parsing requires oletools or similar
    // For now, detect OLE2 + "macros" string presence
    if _data.len() > 8 && _data[0..4] == [0xd0, 0xcf, 0x11, 0xe0] {
        // Look for VBA project stream indicator
        _data.windows(20).any(|window| {
            window.windows(11).any(|w| w.eq_ignore_ascii_case(b"VBAProject"))
        })
    } else {
        false
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// FINDINGS ENGINE
// ─────────────────────────────────────────────────────────────────────────────

/// Generate findings from analysis results
pub fn generate_findings(
    email_id: &str,
    header_analysis: &HeaderAnalysis,
    auth_results: &AuthResults,
    spoof_findings: &[SpoofingFinding],
    attachment_analysis: &[AttachmentAnalysis],
) -> Vec<NewFinding> {
    let mut findings = Vec::new();
    
    // Header-based findings - skip expected MBOX behavior
    for anomaly in &header_analysis.routing_anomalies {
        // Don't create findings for expected MBOX archive behavior
        match anomaly.anomaly_type.as_str() {
            "no_received_internal" | "missing_received" => continue,
            _ => {}
        }
        
        let severity = match anomaly.severity.as_str() {
            "critical" => "critical",
            "high" => "high",
            "medium" => "medium",
            _ => "low",
        };
        
        findings.push(NewFinding {
            type_: "ROUTING".to_string(),
            severity: severity.to_string(),
            confidence: "high".to_string(),
            title: format!("Routing anomaly: {}", anomaly.anomaly_type),
            description: anomaly.description.clone(),
            email_ids: vec![email_id.to_string()],
            indicator: anomaly.anomaly_type.clone(),
        });
    }
    
    // Clock skew findings
    for skew in &header_analysis.clock_skew {
        findings.push(NewFinding {
            type_: "ANOMALY".to_string(),
            severity: "medium".to_string(),
            confidence: "medium".to_string(),
            title: format!("Clock skew detected: {} seconds", skew.skew_seconds),
            description: format!(
                "Time anomaly between {} and {}: {} seconds",
                skew.hop_from, skew.hop_to, skew.skew_seconds
            ),
            email_ids: vec![email_id.to_string()],
            indicator: "clock_skew".to_string(),
        });
    }
    
    // Spoofing findings
    for spoof in spoof_findings {
        let severity = match spoof.severity.as_str() {
            "critical" => "critical",
            "high" => "high",
            "medium" => "medium",
            _ => "low",
        };
        
        let type_ = match spoof.finding_type.as_str() {
            "spf_failure" | "dkim_failure" | "dmarc_failure" => "SPOOFING",
            "display_name_spoofing" | "brand_impersonation" => "BEC",
            "homoglyph_domain" => "SPOOFING",
            "return_path_mismatch" | "reply_to_mismatch" => "SPOOFING",
            "message_id_anomaly" => "ANOMALY",
            _ => "ANOMALY",
        };
        
        findings.push(NewFinding {
            type_: type_.to_string(),
            severity: severity.to_string(),
            confidence: spoof.confidence.clone(),
            title: spoof.title.clone(),
            description: spoof.description.clone(),
            email_ids: vec![email_id.to_string()],
            indicator: spoof.indicator.clone(),
        });
    }
    
    // Attachment findings
    for att in attachment_analysis {
        if !att.risk_flags.is_empty() {
            let severity = if att.risk_score >= 50 { "high" } 
                else if att.risk_score >= 25 { "medium" } 
                else { "low" };
            
            findings.push(NewFinding {
                type_: "ATTACHMENT".to_string(),
                severity: severity.to_string(),
                confidence: "high".to_string(),
                title: format!(
                    "Attachment risk: {}",
                    att.filename.as_deref().unwrap_or("unknown")
                ),
                description: format!(
                    "Risk flags: {}. Entropy: {:.2}",
                    att.risk_flags.join(", "),
                    att.entropy
                ),
                email_ids: vec![email_id.to_string()],
                indicator: att.risk_flags.join(", "),
            });
        }
    }
    
    findings
}

/// Calculate overall risk score for an email
pub fn calculate_risk_score(
    header_analysis: &HeaderAnalysis,
    auth_results: &AuthResults,
    spoof_findings: &[SpoofingFinding],
    attachment_analysis: &[AttachmentAnalysis],
) -> u8 {
    let mut score: u8 = 0;
    
    // Auth failures
    if auth_results.spf.result == "fail" {
        score += 15;
    }
    if auth_results.dkim.iter().any(|d| d.result == "fail") {
        score += 15;
    }
    if auth_results.dmarc.result == "fail" {
        score += 20;
    }
    
    // Spoofing findings
    for spoof in spoof_findings {
        score += match spoof.severity.as_str() {
            "critical" => 25,
            "high" => 15,
            "medium" => 10,
            _ => 5,
        };
    }
    
    // Attachment risk
    for att in attachment_analysis {
        score += att.risk_score / 4; // Max 25 from attachments
    }
    
    // Routing anomalies - only count actual anomalies, not expected MBOX behavior
    for anomaly in &header_analysis.routing_anomalies {
        // Skip anomalies that are expected for MBOX archives
        match anomaly.anomaly_type.as_str() {
            "missing_received" | "no_received_internal" => {
                // Not a risk for MBOX files - Received headers are stripped during archival
            }
            "timestamp_reversal" => score += 10,
            "long_transit" => score += 5,
            "excessive_hops" => score += 10,
            _ => score += 5,
        }
    }
    
    score.min(100)
}

/// New finding for database insertion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewFinding {
    pub type_: String,
    pub severity: String,
    pub confidence: String,
    pub title: String,
    pub description: String,
    pub email_ids: Vec<String>,
    pub indicator: String,
}

// ─────────────────────────────────────────────────────────────────────────────
// HELPER FUNCTIONS
// ─────────────────────────────────────────────────────────────────────────────

/// Extract domain from email address
fn extract_domain(email: &str) -> String {
    email.split('@').nth(1)
        .map(|d| d.trim_matches('>').trim().to_lowercase())
        .unwrap_or_default()
}

/// Extract email from display name like "Name <email@domain.com>"
fn extract_email_from_display_name(display: &str) -> Option<String> {
    display.find('<').and_then(|start| {
        display[start..].find('>').map(|end| display[start + 1..start + end].to_string())
    })
}

/// Extract Return-Path domain
fn extract_return_path_domain(headers_raw: &str) -> Option<String> {
    headers_raw.lines()
        .find(|l| l.to_lowercase().starts_with("return-path:"))
        .and_then(|l| l.split_once(':'))
        .map(|(_, v)| v.trim())
        .and_then(|v| {
            let email = v.trim_matches(|c| c == '<' || c == '>');
            if email.is_empty() {
                None // <> = null return path
            } else {
                Some(extract_domain(email))
            }
        })
}

/// Extract Reply-To domain
fn extract_reply_to_domain(headers_raw: &str) -> Option<String> {
    headers_raw.lines()
        .find(|l| l.to_lowercase().starts_with("reply-to:"))
        .and_then(|l| l.split_once(':'))
        .map(|(_, v)| v.trim())
        .map(|v| {
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
        .flatten()
        .map(|e| extract_domain(&e))
}

/// Extract Message-ID domain (only if it looks like a real domain, not a mail server hostname)
fn extract_message_id_domain(headers_raw: &str) -> Option<String> {
    let domain = headers_raw.lines()
        .find(|l| l.to_lowercase().starts_with("message-id:"))
        .and_then(|l| l.split_once(':'))
        .map(|(_, v)| v.trim())
        .map(|v| {
            v.split('@').nth(1)
                .map(|d| d.trim_matches('>').to_lowercase())
        })
        .flatten()?;
    
    // Filter out single-label hostnames (e.g., "thyme", "mail", "localhost")
    // These are mail server hostnames, not real domains
    if !domain.contains('.') {
        return None; // Single word = mail server hostname, not a domain
    }
    
    // Filter out obviously internal hostnames
    let internal_names = ["localhost", "mail", "mx", "smtp", "exchange", "domino"];
    if internal_names.contains(&domain.as_str()) {
        return None;
    }
    
    Some(domain)
}

/// Check if domain contains homoglyph characters or punycode
fn is_homoglyph_domain(domain: &str) -> bool {
    // Check for punycode
    if domain.starts_with("xn--") || domain.contains(".xn--") {
        return true;
    }
    
    // Check for mixed scripts (homoglyph indicator)
    let has_latin = domain.chars().any(|c| c.is_ascii_alphabetic());
    let has_cyrillic = domain.chars().any(|c| ('\u{0400}'..='\u{04FF}').contains(&c));
    let has_greek = domain.chars().any(|c| ('\u{0370}'..='\u{03FF}').contains(&c));
    
    (has_latin && has_cyrillic) || (has_latin && has_greek)
}
