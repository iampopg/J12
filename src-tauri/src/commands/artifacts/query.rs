use serde_json::Value;
use tauri::{AppHandle, State};
use tauri::ipc::Channel;

use crate::AppState;
use super::types::{TaxonomySubcategorySummary, TaxonomyDomainSummary, ForensicTaxonomyArtifact};
use super::scanner::{extract_all_taxonomy_artifacts, emit_scan_progress};

/// Case Artifacts Summary by Taxonomy Domains
#[tauri::command]
pub async fn case_artifacts_summary(
    state: State<'_, AppState>,
    input: Value,
) -> Result<Vec<TaxonomyDomainSummary>, String> {
    let case_id = input["case_id"].as_str()
        .or_else(|| input["caseId"].as_str())
        .or_else(|| input.as_str())
        .unwrap_or("")
        .to_string();

    let show_all = input["show_all"].as_bool().unwrap_or(false);
    let evidence_id = input["evidence_id"].as_str()
        .or_else(|| input["evidenceId"].as_str())
        .or_else(|| input["input"]["evidence_id"].as_str())
        .or_else(|| input["input"]["evidenceId"].as_str())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty() && s != "all");

    let mut all_artifacts = get_or_extract_artifacts(None, None, &state, &case_id, false).await?;

    if let Some(ref ev_id) = evidence_id {
        let email_ids_in_evidence: std::collections::HashSet<String> = {
            let db = state.db.lock().await;
            let mut stmt = db.conn.prepare("SELECT id FROM emails WHERE case_id = ?1 AND evidence_id = ?2").map_err(|e| e.to_string())?;
            let ids: std::collections::HashSet<String> = stmt.query_map(rusqlite::params![&case_id, ev_id], |row| row.get(0))
                .map_err(|e| e.to_string())?
                .filter_map(|r| r.ok())
                .collect();
            ids
        };
        all_artifacts = all_artifacts.into_iter().filter(|a| email_ids_in_evidence.contains(&a.email_id)).collect();
    }

    let domain_defs = [
        ("social_media", "Social Media & Communities", "🌐"),
        ("mobile_apps", "Mobile Apps & On-Demand", "📱"),
        ("crypto_platforms", "Crypto Exchanges & Web3", "🪙"),
        ("messaging_apps", "Encrypted & Instant Messengers", "💬"),
        ("dating_apps", "Dating & Romance Platforms", "❤️"),
        ("fintech_banking", "Fintech & Digital Banking", "🏦"),
        ("ecommerce", "E-Commerce & Marketplaces", "🛍️"),
        ("cloud_dev", "AI, Cloud & Developer Tools", "🤖"),
        ("vpn_privacy", "VPNs, Privacy & Anonymous Mail", "🛡️"),
        ("remote_access", "Remote Desktop & Collaboration", "🖥️"),
        ("gaming_gambling", "Gaming, Esports & Gambling", "🎮"),
        ("credentials", "Credentials & Secrets", "🔑"),
        ("crypto", "Cryptocurrency & Seeds", "🪙"),
        ("financial", "Financial & Banking Numbers", "💳"),
        ("phone_contacts", "Phone Numbers & Contact Cards", "📞"),
        ("identity_docs", "PII & Identity Documents", "🪪"),
        ("locations", "Locations, Travel & Addresses", "📍"),
        ("contraband", "Threats & Contraband", "🛑"),
        ("malware_threats", "Malware & Cyber IOCs", "🦠"),
        ("secrets", "Corporate & Legal Privileged", "📄"),
        ("phishing", "Phishing & Social Engineering", "🎣"),
        ("network", "Suspicious Network & URL Hooks", "🌐"),
        ("attachments", "Carved & Suspicious Files", "📎"),
        ("deleted_recovered", "Deleted & Carved Messages", "🗑️"),
        ("authentication", "Failed Authentication & Spoofing", "🔐"),
        ("calendar", "Calendar & Meetings (.ics)", "📅"),
        ("client", "Email Clients & Devices", "💻"),
        ("containers", "Mailboxes & Containers", "🗂️"),
        ("case_artifacts", "Evidence Integrity Seals", "⚖️"),
    ];

    let mut result = Vec::new();
    let mut handled_domains = std::collections::HashSet::new();
    handled_domains.insert("ecommerce_shopping".to_string());
    handled_domains.insert("ai_cloud_dev".to_string());
    handled_domains.insert("vpns_privacy".to_string());
    handled_domains.insert("remote_collab".to_string());

    for (dom_id, dom_name, dom_icon) in &domain_defs {
        handled_domains.insert(dom_id.to_string());
        let domain_artifacts: Vec<&ForensicTaxonomyArtifact> = all_artifacts.iter().filter(|a| {
            a.domain_id == *dom_id || match *dom_id {
                "ecommerce" => a.domain_id == "ecommerce_shopping",
                "cloud_dev" => a.domain_id == "ai_cloud_dev",
                "vpn_privacy" => a.domain_id == "vpns_privacy",
                "remote_access" => a.domain_id == "remote_collab",
                _ => false,
            }
        }).collect();
        let total_count = domain_artifacts.len();

        if !show_all && total_count == 0 {
            continue;
        }

        let mut sub_map: std::collections::BTreeMap<String, usize> = std::collections::BTreeMap::new();
        for a in &domain_artifacts {
            *sub_map.entry(a.subcategory_id.clone()).or_insert(0) += 1;
        }

        let mut subcategories: Vec<TaxonomySubcategorySummary> = sub_map.into_iter().filter(|(_, cnt)| *cnt > 0).map(|(k, v)| {
            let name = k.replace('_', " ").to_uppercase();
            TaxonomySubcategorySummary {
                subcategory_id: k,
                name,
                count: v,
            }
        }).collect();

        // Sort subcategories alphabetically by name
        subcategories.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

        result.push(TaxonomyDomainSummary {
            domain_id: dom_id.to_string(),
            name: dom_name.to_string(),
            icon: dom_icon.to_string(),
            total_count,
            subcategories,
        });
    }

    let mut dynamic_map: std::collections::BTreeMap<String, Vec<&ForensicTaxonomyArtifact>> = std::collections::BTreeMap::new();
    for a in &all_artifacts {
        if !handled_domains.contains(&a.domain_id) {
            dynamic_map.entry(a.domain_id.clone()).or_default().push(a);
        }
    }
    for (dom_id, items) in dynamic_map {
        let total_count = items.len();
        if !show_all && total_count == 0 {
            continue;
        }
        let dom_name = dom_id.replace('_', " ").to_uppercase();
        result.push(TaxonomyDomainSummary {
            domain_id: dom_id.clone(),
            name: dom_name,
            icon: "📁".to_string(),
            total_count,
            subcategories: vec![],
        });
    }

    // Sort all domains alphabetically by name
    result.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

    Ok(result)
}

/// Case Artifacts List filtered by domain, subcategory, search, or severity
#[tauri::command]
pub async fn case_artifacts_list(
    state: State<'_, AppState>,
    input: Value,
) -> Result<Vec<ForensicTaxonomyArtifact>, String> {
    let case_id = input["case_id"].as_str()
        .or_else(|| input["caseId"].as_str())
        .or_else(|| input.as_str())
        .unwrap_or("")
        .to_string();

    let domain = input["domain"].as_str()
        .or_else(|| input["category"].as_str())
        .unwrap_or("all");
    let subcategory = input["subcategory"].as_str().unwrap_or("all");
    let search = input["search"].as_str().unwrap_or("").to_lowercase();
    let artifact_type = input["artifact_type"].as_str().unwrap_or("all");
    let evidence_id = input["evidence_id"].as_str()
        .or_else(|| input["evidenceId"].as_str())
        .or_else(|| input["input"]["evidence_id"].as_str())
        .or_else(|| input["input"]["evidenceId"].as_str())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty() && s != "all");

    let mut all_artifacts = get_or_extract_artifacts(None, None, &state, &case_id, false).await?;

    if let Some(ref ev_id) = evidence_id {
        let email_ids_in_evidence: std::collections::HashSet<String> = {
            let db = state.db.lock().await;
            let mut stmt = db.conn.prepare("SELECT id FROM emails WHERE case_id = ?1 AND evidence_id = ?2").map_err(|e| e.to_string())?;
            let ids: std::collections::HashSet<String> = stmt.query_map(rusqlite::params![&case_id, ev_id], |row| row.get(0))
                .map_err(|e| e.to_string())?
                .filter_map(|r| r.ok())
                .collect();
            ids
        };
        all_artifacts = all_artifacts.into_iter().filter(|a| email_ids_in_evidence.contains(&a.email_id)).collect();
    }

    let filtered = all_artifacts.into_iter().filter(|item| {
        if domain != "all" && item.domain_id != domain && match domain {
            "ecommerce" => item.domain_id != "ecommerce_shopping",
            "cloud_dev" => item.domain_id != "ai_cloud_dev",
            "vpn_privacy" => item.domain_id != "vpns_privacy",
            "remote_access" => item.domain_id != "remote_collab",
            _ => true,
        } {
            return false;
        }
        if subcategory != "all" && item.subcategory_id != subcategory {
            return false;
        }
        if artifact_type != "all" && item.artifact_type != artifact_type {
            return false;
        }
        if !search.is_empty() {
            let val_m = item.primary_value.to_lowercase().contains(&search);
            let title_m = item.title.to_lowercase().contains(&search);
            let det_m = item.details.to_lowercase().contains(&search);
            let subj_m = item.email_subject.as_deref().unwrap_or("").to_lowercase().contains(&search);
            let from_m = item.email_from.to_lowercase().contains(&search);
            if !val_m && !title_m && !det_m && !subj_m && !from_m {
                return false;
            }
        }
        true
    }).collect();

    Ok(filtered)
}

#[tauri::command]
pub async fn rescan_case_artifacts(
    app: AppHandle,
    state: State<'_, AppState>,
    input: Value,
    on_event: Channel<Value>,
) -> Result<usize, String> {
    let case_id = input["case_id"].as_str()
        .or_else(|| input["caseId"].as_str())
        .or_else(|| input["input"]["case_id"].as_str())
        .or_else(|| input["input"]["caseId"].as_str())
        .or_else(|| input.as_str())
        .unwrap_or("")
        .to_string();

    let arts = get_or_extract_artifacts(Some(&app), Some(&on_event), &state, &case_id, true).await?;
    crate::audit_logger::log_forensic_event(
        &case_id,
        "ARTIFACT_TAXONOMY",
        "ARTIFACT_SCAN_COMPLETED",
        "System Pipeline",
        None,
        None,
        &format!("Extracted & categorized {} forensic taxonomy artifacts across all message corpora", arts.len())
    );
    Ok(arts.len())
}

pub async fn get_or_extract_artifacts(
    app: Option<&AppHandle>,
    on_event: Option<&Channel<Value>>,
    state: &State<'_, AppState>,
    case_id: &str,
    force_rescan: bool,
) -> Result<Vec<ForensicTaxonomyArtifact>, String> {
    if !force_rescan {
        let (cached, has_emails) = {
            let db = state.db.lock().await;
            let mut stmt = db.conn.prepare("
                SELECT id, domain_id, subcategory_id, title, primary_value, secondary_value,
                       details, severity, artifact_type, confidence, email_id, email_subject, email_from, date_sent_utc
                FROM forensic_artifacts
                WHERE case_id = ?1
            ").map_err(|e| e.to_string())?;

            let cached = stmt.query_map([case_id], |row| {
                Ok(ForensicTaxonomyArtifact {
                    id: row.get(0)?,
                    domain_id: row.get(1)?,
                    subcategory_id: row.get(2)?,
                    title: row.get(3)?,
                    primary_value: row.get(4)?,
                    secondary_value: row.get(5)?,
                    details: row.get(6)?,
                    severity: row.get(7)?,
                    artifact_type: row.get(8)?,
                    confidence: row.get(9)?,
                    email_id: row.get(10)?,
                    email_subject: row.get(11)?,
                    email_from: row.get(12)?,
                    date_sent_utc: row.get(13)?,
                })
            }).map_err(|e| e.to_string())?.filter_map(|r| r.ok()).collect::<Vec<_>>();

            let count: i64 = db.conn.query_row(
                "SELECT COUNT(*) FROM emails WHERE case_id = ?1",
                [case_id],
                |r| r.get(0)
            ).unwrap_or(0);

            (cached, count > 0)
        };

        if !cached.is_empty() || !has_emails {
            return Ok(cached);
        }
    }

    let extracted = extract_all_taxonomy_artifacts(app, on_event, state, case_id).await?;

    emit_scan_progress(
        app,
        on_event,
        0,
        0,
        92,
        extracted.len(),
        &format!("Writing {} extracted artifacts to forensic database...", extracted.len()),
    );

    let mut db = state.db.lock().await;
    let tx = db.conn.transaction().map_err(|e| e.to_string())?;
    let _ = tx.execute("DELETE FROM forensic_artifacts WHERE case_id = ?1", [case_id]);

    {
        let mut stmt = tx.prepare("
            INSERT OR REPLACE INTO forensic_artifacts (id, case_id, domain_id, subcategory_id, title, primary_value, secondary_value, details, severity, artifact_type, confidence, email_id, email_subject, email_from, date_sent_utc)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)
        ").map_err(|e| e.to_string())?;

        for art in &extracted {
            let _ = stmt.execute(rusqlite::params![
                art.id, case_id, art.domain_id, art.subcategory_id, art.title,
                art.primary_value, art.secondary_value, art.details, art.severity,
                art.artifact_type, art.confidence, art.email_id, art.email_subject,
                art.email_from, art.date_sent_utc
            ]);
        }
    }
    tx.commit().map_err(|e| e.to_string())?;

    emit_scan_progress(
        app,
        on_event,
        0,
        0,
        100,
        extracted.len(),
        &format!("Completed! Indexed {} forensic taxonomy artifacts.", extracted.len()),
    );

    Ok(extracted)
}
