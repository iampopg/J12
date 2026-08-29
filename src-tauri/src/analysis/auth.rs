use super::types::{AuthResults, AuthCheck, ArcSeal};

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

fn analyze_spf(
    _headers_raw: &str,
    from_domain: &str,
    _source_ip: Option<&str>,
    auth_results: &[String],
) -> AuthCheck {
    for result in auth_results {
        if result.to_lowercase().contains("spf=") {
            let lower = result.to_lowercase();
            
            if lower.contains("spf=pass") {
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
    
    AuthCheck {
        result: "none".to_string(),
        identity: Some(from_domain.to_string()),
        domain: Some(from_domain.to_string()),
        aligned: false,
        detail: "No explicit SPF result found in headers - cannot verify without DNS lookup".to_string(),
    }
}

fn analyze_dkim(headers_raw: &str, auth_results: &[String]) -> Vec<AuthCheck> {
    let mut results = Vec::new();
    
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

fn extract_dkim_domain(headers_raw: &str) -> Option<String> {
    for line in headers_raw.lines() {
        if line.to_lowercase().starts_with("dkim-signature:") {
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

fn analyze_dmarc(_headers_raw: &str, from_domain: &str, auth_results: &[String]) -> AuthCheck {
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
