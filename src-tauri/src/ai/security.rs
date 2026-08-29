use tauri::State;
use crate::AppState;
use super::types::{
    SpoofingAnalysis, SpoofingFinding, EmailResult, AuthResults, AttachmentMetadata,
    AttachmentTriage, AttachmentRisk
};
use super::tools::{ai_get_email, ai_get_authentication_results};

// === ENGINE 5: SPOOFING/PHISHING ANALYST ===

pub fn analyze_spoofing(
    email: &EmailResult,
    auth: &AuthResults,
) -> SpoofingAnalysis {
    let mut findings = Vec::new();
    let mut risk_score = 0;
    
    if auth.spf_result.as_deref() == Some("fail") {
        risk_score += 25;
        findings.push(SpoofingFinding {
            category: "authentication".to_string(),
            finding: "SPF check failed".to_string(),
            severity: "high".to_string(),
            evidence: format!("SPF result: {}", auth.spf_result.as_deref().unwrap_or("unknown")),
        });
    }
    
    if auth.dkim_result.as_deref() == Some("fail") || auth.dkim_result.as_deref() == Some("none") {
        risk_score += 20;
        findings.push(SpoofingFinding {
            category: "authentication".to_string(),
            finding: if auth.dkim_result.as_deref() == Some("fail") {
                "DKIM signature validation failed".to_string()
            } else {
                "No DKIM signature present".to_string()
            },
            severity: "medium".to_string(),
            evidence: format!("DKIM result: {}", auth.dkim_result.as_deref().unwrap_or("unknown")),
        });
    }
    
    if auth.dmarc_result.as_deref() == Some("fail") {
        risk_score += 30;
        findings.push(SpoofingFinding {
            category: "authentication".to_string(),
            finding: "DMARC validation failed".to_string(),
            severity: "high".to_string(),
            evidence: format!("DMARC result: {}", auth.dmarc_result.as_deref().unwrap_or("unknown")),
        });
    }
    
    if let Some(reply_to) = &email.from_display {
        if !reply_to.is_empty() && !email.from_addr.contains(reply_to) {
            risk_score += 15;
            findings.push(SpoofingFinding {
                category: "address".to_string(),
                finding: "Reply-To address differs from sender".to_string(),
                severity: "medium".to_string(),
                evidence: format!("From: {}, Display: {}", email.from_addr, reply_to),
            });
        }
    }
    
    if let Some(ip) = &auth.originating_ip {
        if ip.starts_with("10.") || ip.starts_with("192.168.") || ip.starts_with("172.") {
            risk_score += 10;
            findings.push(SpoofingFinding {
                category: "network".to_string(),
                finding: "Email originated from private IP range".to_string(),
                severity: "low".to_string(),
                evidence: format!("Originating IP: {}", ip),
            });
        }
    }
    
    if auth.received_chain.len() > 5 {
        risk_score += 5;
        findings.push(SpoofingFinding {
            category: "routing".to_string(),
            finding: format!("Unusually long received chain ({} hops)", auth.received_chain.len()),
            severity: "low".to_string(),
            evidence: format!("{} hops in received chain", auth.received_chain.len()),
        });
    }
    
    let overall_risk = match risk_score {
        0..=20 => "low".to_string(),
        21..=50 => "medium".to_string(),
        51..=75 => "high".to_string(),
        _ => "critical".to_string(),
    };
    
    let recommendations = generate_spoofing_recommendations(&overall_risk, &findings);
    
    SpoofingAnalysis {
        email_id: email.id.clone(),
        overall_risk,
        risk_score,
        findings,
        recommendations,
    }
}

fn generate_spoofing_recommendations(risk: &str, findings: &[SpoofingFinding]) -> Vec<String> {
    let mut recs = Vec::new();
    
    match risk {
        "critical" | "high" => {
            recs.push("Treat this email as potentially malicious".to_string());
            recs.push("Do not click any links or open attachments".to_string());
            recs.push("Verify sender through alternative channel".to_string());
        }
        "medium" => {
            recs.push("Exercise caution with this email".to_string());
            recs.push("Verify unexpected requests independently".to_string());
        }
        _ => {
            recs.push("Standard precautions apply".to_string());
        }
    }
    
    for finding in findings {
        if finding.category == "authentication" && finding.severity == "high" {
            recs.push("Authentication failure detected - verify sender identity".to_string());
        }
    }
    
    recs
}

// === ENGINE 6: ATTACHMENT TRIAGE ===

pub fn triage_attachments(attachments: &[AttachmentMetadata]) -> AttachmentTriage {
    let mut results = Vec::new();
    let mut critical_count = 0;
    let mut high_count = 0;
    let mut medium_count = 0;
    let mut low_count = 0;
    
    for att in attachments {
        let risk = assess_attachment_risk(att);
        match risk.risk_level.as_str() {
            "critical" => critical_count += 1,
            "high" => high_count += 1,
            "medium" => medium_count += 1,
            _ => low_count += 1,
        }
        results.push(risk);
    }
    
    results.sort_by(|a, b| b.risk_score.cmp(&a.risk_score));
    
    AttachmentTriage {
        attachments: results,
        critical_count,
        high_count,
        medium_count,
        low_count,
    }
}

fn assess_attachment_risk(att: &AttachmentMetadata) -> AttachmentRisk {
    let mut risk_score = 0;
    let mut reasons = Vec::new();
    let mut recommendations = Vec::new();
    
    let filename_lower = att.filename.as_deref().unwrap_or("").to_lowercase();
    let extension_count = filename_lower.matches('.').count();
    if extension_count > 1 {
        risk_score += 30;
        reasons.push("Double extension detected - may disguise true file type".to_string());
        recommendations.push("Verify file type using magic bytes".to_string());
    }
    
    if filename_lower.ends_with(".exe") || filename_lower.ends_with(".bat") || 
       filename_lower.ends_with(".cmd") || filename_lower.ends_with(".ps1") ||
       filename_lower.ends_with(".vbs") || filename_lower.ends_with(".js") ||
       filename_lower.ends_with(".scr") || filename_lower.ends_with(".msi") {
        risk_score += 40;
        reasons.push("Executable file type - can run code on target system".to_string());
        recommendations.push("Scan with antivirus before opening".to_string());
        recommendations.push("Consider sandboxed analysis".to_string());
    }
    
    if filename_lower.ends_with(".docm") || filename_lower.ends_with(".xlsm") || 
       filename_lower.ends_with(".pptm") {
        risk_score += 25;
        reasons.push("Macro-enabled office document - can execute automated scripts".to_string());
        recommendations.push("Disable macros before opening".to_string());
    }
    
    if let Some(entropy) = att.entropy {
        if entropy > 7.5 {
            risk_score += 20;
            reasons.push(format!("High entropy ({:.2}/8.0) - possibly encrypted or packed", entropy));
            recommendations.push("May contain hidden or obfuscated content".to_string());
        }
    }
    
    if let Some(filename) = &att.filename {
        let ext = filename.rsplit('.').next().unwrap_or("");
        let expected_mime = match ext.to_lowercase().as_str() {
            "pdf" => "application/pdf",
            "doc" | "docx" => "application/msword",
            "xls" | "xlsx" => "application/vnd.ms-excel",
            "jpg" | "jpeg" => "image/jpeg",
            "png" => "image/png",
            "zip" => "application/zip",
            _ => "",
        };
        if !expected_mime.is_empty() && !att.mime_type.contains(expected_mime) {
            risk_score += 15;
            reasons.push(format!("MIME type mismatch: extension .{} but MIME is {}", ext, att.mime_type));
        }
    }
    
    for flag in &att.risk_flags {
        match flag.as_str() {
            "executable" => risk_score += 40,
            "macro_enabled" => risk_score += 25,
            "high_entropy_encrypted" => risk_score += 20,
            "double_extension" => risk_score += 30,
            _ => risk_score += 5,
        }
    }
    
    if att.size_bytes > 10_000_000 {
        risk_score += 10;
        reasons.push(format!("Large file size: {} MB", att.size_bytes / 1_000_000));
    }
    
    let risk_level = match risk_score {
        0..=20 => "low".to_string(),
        21..=50 => "medium".to_string(),
        51..=75 => "high".to_string(),
        _ => "critical".to_string(),
    };
    
    if recommendations.is_empty() {
        recommendations.push("Standard handling procedures apply".to_string());
    }
    
    AttachmentRisk {
        attachment_id: att.id.clone(),
        filename: att.filename.clone().unwrap_or_else(|| "unknown".to_string()),
        risk_level,
        risk_score,
        reasons,
        recommendations,
    }
}

#[tauri::command]
pub async fn ai_analyze_spoofing(state: State<'_, AppState>, email_id: String) -> Result<SpoofingAnalysis, String> {
    let email = match ai_get_email(state.clone(), email_id.clone()).await? {
        Some(e) => e,
        None => return Err("Email not found".to_string()),
    };
    
    let auth = match ai_get_authentication_results(state, email_id).await? {
        Some(a) => a,
        None => return Err("Authentication results not found".to_string()),
    };
    
    Ok(analyze_spoofing(&email, &auth))
}

#[tauri::command]
pub async fn ai_triage_attachments(state: State<'_, AppState>, email_id: String) -> Result<AttachmentTriage, String> {
    let db = state.db.lock().await;
    
    let mut stmt = db.conn.prepare(
        "SELECT id, filename, mime_type, size_bytes, sha256, entropy, risk_flags FROM attachments WHERE email_id = ?1"
    ).map_err(|e| e.to_string())?;
    
    let attachments = stmt.query_map([&email_id], |row| {
        let risk_flags_str: Option<String> = row.get(6).ok();
        let risk_flags: Vec<String> = risk_flags_str
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default();
        
        Ok(AttachmentMetadata {
            id: row.get(0)?,
            filename: row.get(1)?,
            mime_type: row.get(2)?,
            size_bytes: row.get(3)?,
            sha256: row.get(4)?,
            entropy: row.get(5)?,
            risk_flags,
        })
    }).map_err(|e| e.to_string())?.collect::<Result<Vec<_>,_>>().map_err(|e| e.to_string())?;
    
    Ok(triage_attachments(&attachments))
}
