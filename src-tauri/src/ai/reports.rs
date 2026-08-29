use tauri::State;
use crate::AppState;
use super::types::{ReportSection, InvestigationReport, ReportMetadata, FindingData, CaseStats};
use super::tools::ai_get_case_statistics;

// === ENGINE 10: REPORT ASSISTANT ===

pub fn generate_report(
    _case_data: &serde_json::Value,
    stats: &CaseStats,
    findings: &[FindingData],
    model: &str,
) -> InvestigationReport {
    let mut sections = Vec::new();
    
    sections.push(ReportSection {
        title: "Executive Summary".to_string(),
        content: format!(
            "This report presents the findings of a forensic investigation conducted on {} email messages. The investigation identified {} forensic findings and {} unique entities requiring review.",
            stats.total_emails, stats.total_findings, stats.total_entities
        ),
        section_type: "summary".to_string(),
        evidence_refs: vec![],
    });
    
    sections.push(ReportSection {
        title: "Scope".to_string(),
        content: "Analysis limited to acquired evidence only. Server-side logs, endpoint telemetry, and network flow data were not available for this investigation.".to_string(),
        section_type: "scope".to_string(),
        evidence_refs: vec![],
    });
    
    sections.push(ReportSection {
        title: "Methodology".to_string(),
        content: "Evidence was acquired, parsed, and analyzed using deterministic forensic analysis. Email authentication (SPF, DKIM, DMARC), attachment analysis, timeline reconstruction, and communication graph analysis were performed.".to_string(),
        section_type: "methodology".to_string(),
        evidence_refs: vec![],
    });
    
    let findings_content = if findings.is_empty() {
        "No critical findings were identified during this investigation.".to_string()
    } else {
        let mut content = String::from("The following findings were identified:\n\n");
        for finding in findings.iter().take(10) {
            content.push_str(&format!("- **[{}] {}**: {}\n", 
                finding.severity.to_uppercase(), 
                finding.title,
                finding.description.as_deref().unwrap_or("No description")
            ));
        }
        content
    };
    
    sections.push(ReportSection {
        title: "Findings".to_string(),
        content: findings_content,
        section_type: "findings".to_string(),
        evidence_refs: vec![],
    });
    
    sections.push(ReportSection {
        title: "Evidence Statistics".to_string(),
        content: format!(
            "- Total Emails: {}\n- Inbox: {}\n- Sent: {}\n- Deleted: {}\n- Spam: {}\n- Total Entities: {}\n- Total Attachments: {}",
            stats.total_emails,
            stats.inbox_count,
            stats.sent_count,
            stats.deleted_count,
            stats.spam_count,
            stats.total_entities,
            stats.total_attachments
        ),
        section_type: "statistics".to_string(),
        evidence_refs: vec![],
    });
    
    sections.push(ReportSection {
        title: "Limitations".to_string(),
        content: "1. Analysis limited to acquired evidence only.\n2. Mail server authentication logs not available.\n3. Endpoint telemetry not available.\n4. Cannot establish account compromise conclusively without server logs.".to_string(),
        section_type: "limitations".to_string(),
        evidence_refs: vec![],
    });
    
    InvestigationReport {
        title: "Forensic Investigation Report".to_string(),
        generated_at: chrono::Utc::now().to_rfc3339(),
        generated_by: "J12 AI Assistant".to_string(),
        model: model.to_string(),
        sections,
        metadata: ReportMetadata {
            total_emails: stats.total_emails,
            total_findings: stats.total_findings,
            total_entities: stats.total_entities,
            scan_duration_ms: 0,
        },
    }
}

#[tauri::command]
pub async fn ai_generate_report(state: State<'_, AppState>, case_id: String, model: String) -> Result<InvestigationReport, String> {
    let stats = ai_get_case_statistics(state.clone(), case_id.clone()).await?;
    let db = state.db.lock().await;
    
    let mut stmt = db.conn.prepare(
        "SELECT id, type, severity, title, description, status FROM findings WHERE case_id = ?1 ORDER BY severity, created_at"
    ).map_err(|e| e.to_string())?;
    
    let findings = stmt.query_map([&case_id], |row| {
        Ok(FindingData {
            id: row.get(0)?,
            finding_type: row.get(1)?,
            severity: row.get(2)?,
            title: row.get(3)?,
            description: row.get(4)?,
            status: row.get(5)?,
        })
    }).map_err(|e| e.to_string())?.collect::<Result<Vec<_>,_>>().map_err(|e| e.to_string())?;
    
    let case_data = serde_json::json!({});
    Ok(generate_report(&case_data, &stats, &findings, &model))
}
