use super::types::{
    HeaderAnalysis, AuthResults, SpoofingFinding, AttachmentAnalysis, NewFinding
};

/// Generate findings from analysis results
pub fn generate_findings(
    email_id: &str,
    header_analysis: &HeaderAnalysis,
    _auth_results: &AuthResults,
    spoof_findings: &[SpoofingFinding],
    attachment_analysis: &[AttachmentAnalysis],
) -> Vec<NewFinding> {
    let mut findings = Vec::new();
    
    // Header-based findings - skip expected MBOX behavior
    for anomaly in &header_analysis.routing_anomalies {
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
            "spf_failure" | "dkim_failure" | "dmarc_failure" | "homoglyph_domain" | "return_path_mismatch" | "reply_to_mismatch" => "SPOOFING",
            "display_name_spoofing" | "brand_impersonation" | "bec_wire_fraud" | "gift_card_fraud" | "executive_impersonation" => "BEC",
            "credential_phishing" => "PHISHING",
            "confidential_exfiltration" => "EXFILTRATION",
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
    let mut score: u32 = 0;
    
    if auth_results.spf.result == "fail" {
        score += 15;
    }
    if auth_results.dkim.iter().any(|d| d.result == "fail") {
        score += 15;
    }
    if auth_results.dmarc.result == "fail" {
        score += 20;
    }
    
    for spoof in spoof_findings {
        score += match spoof.severity.as_str() {
            "critical" => 25,
            "high" => 15,
            "medium" => 10,
            _ => 5,
        };
    }
    
    for att in attachment_analysis {
        score += (att.risk_score as u32) / 4;
    }
    
    for anomaly in &header_analysis.routing_anomalies {
        match anomaly.anomaly_type.as_str() {
            "missing_received" | "no_received_internal" => {}
            "timestamp_reversal" => score += 10,
            "long_transit" => score += 5,
            "excessive_hops" => score += 10,
            _ => score += 5,
        }
    }
    
    score.min(100) as u8
}
