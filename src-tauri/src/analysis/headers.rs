use super::types::{HeaderAnalysis, Hop, SkewEvent, Anomaly};

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
    
    for (idx, line) in received_lines.iter().enumerate() {
        let hop = parse_hop(line);
        
        if idx > 0 {
            let prev_ts = &hops[idx - 1].timestamp;
            if let (Some(ts), Some(prev_ts)) = (&hop.timestamp, prev_ts) {
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
    
    hops.reverse();
    
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
    
    let originating_ip = if hops_with_transit.is_empty() {
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
    
    if hops_with_transit.len() > 10 {
        anomalies.push(Anomaly {
            anomaly_type: "excessive_hops".to_string(),
            description: format!("Unusually high number of hops: {}", hops_with_transit.len()),
            severity: "medium".to_string(),
        });
    }
    
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
                if skew.abs() > 300 {
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

fn parse_hop(line: &str) -> Hop {
    let content = line.strip_prefix("Received:").unwrap_or(line).trim();
    
    let from = extract_field(content, "from").map(|s| s.trim().to_string());
    let by = extract_field(content, "by").map(|s| s.trim().to_string());
    let with = extract_field(content, "with").map(|s| s.trim().to_string());
    let id = extract_field(content, "id").map(|s| s.trim().to_string());
    let for_addr = extract_field(content, "for").map(|s| s.trim().to_string());
    
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

fn extract_field(content: &str, field: &str) -> Option<String> {
    let lower = content.to_lowercase();
    let pattern = format!("{} ", field);
    
    if let Some(start) = lower.find(&pattern) {
        let value_start = start + pattern.len();
        let remaining = &content[value_start..];
        
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

fn extract_ip_from_received(host: &str) -> Option<String> {
    if let (Some(start), Some(end)) = (host.find('['), host.find(']')) {
        if start < end {
            let ip = &host[start + 1..end];
            if ip.parse::<std::net::IpAddr>().is_ok() {
                return Some(ip.to_string());
            }
        }
    }
    let ip_regex = regex::Regex::new(r"\b(\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3})\b").ok()?;
    ip_regex.captures(host).and_then(|caps| caps.get(1)).map(|m| m.as_str().to_string())
}

fn parse_timestamp_to_seconds(ts: &str) -> Option<i64> {
    use chrono::DateTime;
    
    if let Ok(dt) = DateTime::parse_from_rfc2822(ts) {
        return Some(dt.timestamp());
    }
    if let Ok(dt) = DateTime::parse_from_str(ts, "%d %b %Y %H:%M:%S %z") {
        return Some(dt.timestamp());
    }
    if let Ok(dt) = DateTime::parse_from_rfc3339(ts) {
        return Some(dt.timestamp());
    }
    if let Ok(dt) = ts.parse::<DateTime<chrono::FixedOffset>>() {
        return Some(dt.timestamp());
    }
    None
}
