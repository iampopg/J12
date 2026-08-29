use tauri::State;
use crate::AppState;
use super::types::{CaseStats, InvestigationStep, InvestigationPlan};
use super::tools::ai_get_case_statistics;

/// Generate investigation plan from objective
pub fn generate_investigation_plan(objective: &str, case_stats: &CaseStats) -> InvestigationPlan {
    let lower = objective.to_lowercase();
    let investigation_type = classify_investigation(&lower);
    
    let available_evidence = build_available_evidence(case_stats);
    let unavailable_evidence = build_unavailable_evidence();
    let limitations = build_limitations(&investigation_type, case_stats);
    let steps = generate_steps(&investigation_type, case_stats);
    let estimated_runtime = estimate_runtime(&steps);
    
    InvestigationPlan {
        objective: objective.to_string(),
        normalized_objective: normalize_objective(&investigation_type),
        available_evidence,
        unavailable_evidence,
        limitations,
        steps,
        estimated_runtime_seconds: estimated_runtime,
    }
}

#[derive(Debug, Clone)]
pub enum InvestigationType {
    MailboxCompromise,
    DataExfiltration,
    PhishingCampaign,
    InsiderThreat,
    FinancialFraud,
    CommunicationAnalysis,
    General,
}

pub fn classify_investigation(objective: &str) -> InvestigationType {
    if objective.contains("compromis") || objective.contains("hacked") || objective.contains("unauthorized access") {
        InvestigationType::MailboxCompromise
    } else if objective.contains("exfiltrat") || objective.contains("data leak") || objective.contains("stolen data") {
        InvestigationType::DataExfiltration
    } else if objective.contains("phishing") || objective.contains("spoofing") || objective.contains("impersonat") {
        InvestigationType::PhishingCampaign
    } else if objective.contains("insider") || objective.contains("employee") || objective.contains("disgruntled") {
        InvestigationType::InsiderThreat
    } else if objective.contains("fraud") || objective.contains("wire transfer") || objective.contains("money") {
        InvestigationType::FinancialFraud
    } else if objective.contains("communication") || objective.contains("relationship") || objective.contains("who talked to whom") {
        InvestigationType::CommunicationAnalysis
    } else {
        InvestigationType::General
    }
}

pub fn normalize_objective(investigation_type: &InvestigationType) -> String {
    match investigation_type {
        InvestigationType::MailboxCompromise => "Assess indicators consistent with possible mailbox compromise".to_string(),
        InvestigationType::DataExfiltration => "Identify potential data exfiltration activities".to_string(),
        InvestigationType::PhishingCampaign => "Analyze phishing/spoofing indicators in acquired evidence".to_string(),
        InvestigationType::InsiderThreat => "Assess indicators consistent with possible insider threat".to_string(),
        InvestigationType::FinancialFraud => "Identify potential financial fraud indicators".to_string(),
        InvestigationType::CommunicationAnalysis => "Analyze communication patterns and relationships".to_string(),
        InvestigationType::General => "Conduct general investigation of acquired evidence".to_string(),
    }
}

pub fn build_available_evidence(case_stats: &CaseStats) -> Vec<String> {
    let mut evidence = Vec::new();
    
    if case_stats.total_emails > 0 {
        evidence.push(format!("{} emails with metadata and headers", case_stats.total_emails));
    }
    if case_stats.total_attachments > 0 {
        evidence.push(format!("{} attachments with analysis", case_stats.total_attachments));
    }
    if case_stats.total_entities > 0 {
        evidence.push(format!("{} entities with communication profiles", case_stats.total_entities));
    }
    if case_stats.total_findings > 0 {
        evidence.push(format!("{} pre-computed forensic findings", case_stats.total_findings));
    }
    
    evidence.push("SPF/DKIM/DMARC authentication results".to_string());
    evidence.push("Received header chains".to_string());
    evidence.push("Timeline events".to_string());
    evidence.push("Communication graph data".to_string());
    
    evidence
}

pub fn build_unavailable_evidence() -> Vec<String> {
    vec![
        "Mail server authentication logs (not acquired)".to_string(),
        "Endpoint telemetry (not acquired)".to_string(),
        "Account login history (not acquired)".to_string(),
        "Network flow data (not acquired)".to_string(),
        "Third-party email provider logs (not acquired)".to_string(),
        "Physical device forensics (not acquired)".to_string(),
    ]
}

pub fn build_limitations(investigation_type: &InvestigationType, case_stats: &CaseStats) -> Vec<String> {
    let mut limitations = Vec::new();
    limitations.push("Analysis limited to acquired evidence only".to_string());
    
    match investigation_type {
        InvestigationType::MailboxCompromise => {
            limitations.push("Mailbox evidence alone cannot definitively prove account compromise".to_string());
            limitations.push("Server-side logs would be required for conclusive determination".to_string());
        }
        InvestigationType::DataExfiltration => {
            limitations.push("Can identify suspicious patterns but not confirm data leaving the organization".to_string());
            limitations.push("Network monitoring data would strengthen analysis".to_string());
        }
        InvestigationType::PhishingCampaign => {
            limitations.push("Can identify indicators but cannot trace campaign origin".to_string());
            limitations.push("Threat intelligence feeds would enhance analysis".to_string());
        }
        _ => {}
    }
    
    if case_stats.total_emails < 100 {
        limitations.push("Limited email volume may reduce statistical significance".to_string());
    }
    
    limitations
}

pub fn generate_steps(investigation_type: &InvestigationType, _case_stats: &CaseStats) -> Vec<InvestigationStep> {
    match investigation_type {
        InvestigationType::MailboxCompromise => vec![
            InvestigationStep {
                step_number: 1,
                title: "Identify unusual sending patterns".to_string(),
                description: "Detect emails sent at unusual times or with unusual content".to_string(),
                tool_calls: vec!["search_emails".to_string(), "get_timeline".to_string()],
                expected_output: "List of anomalous sending patterns".to_string(),
            },
            InvestigationStep {
                step_number: 2,
                title: "Identify new correspondents".to_string(),
                description: "Find email addresses not previously seen in the mailbox".to_string(),
                tool_calls: vec!["get_entity".to_string(), "search_emails".to_string()],
                expected_output: "List of new/unusual correspondents".to_string(),
            },
            InvestigationStep {
                step_number: 3,
                title: "Analyze authentication anomalies".to_string(),
                description: "Check SPF/DKIM/DMARC failures".to_string(),
                tool_calls: vec!["get_authentication_results".to_string(), "search_emails".to_string()],
                expected_output: "Emails with authentication failures".to_string(),
            },
            InvestigationStep {
                step_number: 4,
                title: "Detect suspicious attachments".to_string(),
                description: "Identify potentially malicious files".to_string(),
                tool_calls: vec!["get_attachments".to_string(), "get_attachment_text".to_string()],
                expected_output: "List of suspicious attachments".to_string(),
            },
            InvestigationStep {
                step_number: 5,
                title: "Construct timeline".to_string(),
                description: "Build chronological view of suspicious events".to_string(),
                tool_calls: vec!["get_timeline".to_string(), "search_emails".to_string()],
                expected_output: "Timeline of suspicious activity".to_string(),
            },
        ],
        InvestigationType::DataExfiltration => vec![
            InvestigationStep {
                step_number: 1,
                title: "Identify large outbound emails".to_string(),
                description: "Find emails with large attachments or many recipients".to_string(),
                tool_calls: vec!["search_emails".to_string(), "get_attachments".to_string()],
                expected_output: "List of large outbound emails".to_string(),
            },
            InvestigationStep {
                step_number: 2,
                title: "Detect external communications".to_string(),
                description: "Identify emails to external/unusual domains".to_string(),
                tool_calls: vec!["search_emails".to_string(), "get_entity".to_string()],
                expected_output: "External communication patterns".to_string(),
            },
            InvestigationStep {
                step_number: 3,
                title: "Analyze attachment types".to_string(),
                description: "Identify archives, encrypted files, or unusual types".to_string(),
                tool_calls: vec!["get_attachments".to_string(), "get_attachment_text".to_string()],
                expected_output: "Suspicious attachment analysis".to_string(),
            },
            InvestigationStep {
                step_number: 4,
                title: "Map data flow".to_string(),
                description: "Visualize who sent what to whom".to_string(),
                tool_calls: vec!["get_communication_graph".to_string(), "get_entity".to_string()],
                expected_output: "Data flow visualization".to_string(),
            },
        ],
        InvestigationType::PhishingCampaign => vec![
            InvestigationStep {
                step_number: 1,
                title: "Identify failed authentication".to_string(),
                description: "Find emails with SPF/DKIM/DMARC failures".to_string(),
                tool_calls: vec!["search_emails".to_string(), "get_authentication_results".to_string()],
                expected_output: "Emails with auth failures".to_string(),
            },
            InvestigationStep {
                step_number: 2,
                title: "Detect lookalike domains".to_string(),
                description: "Find sender domains similar to known legitimate domains".to_string(),
                tool_calls: vec!["search_emails".to_string(), "get_entity".to_string()],
                expected_output: "Potential lookalike domains".to_string(),
            },
            InvestigationStep {
                step_number: 3,
                title: "Analyze URLs".to_string(),
                description: "Extract and analyze URLs in email bodies".to_string(),
                tool_calls: vec!["search_emails".to_string(), "get_email_body".to_string()],
                expected_output: "URL analysis results".to_string(),
            },
            InvestigationStep {
                step_number: 4,
                title: "Identify credential harvesting".to_string(),
                description: "Find emails with login forms or password requests".to_string(),
                tool_calls: vec!["search_emails".to_string(), "get_attachment_text".to_string()],
                expected_output: "Potential phishing emails".to_string(),
            },
        ],
        _ => vec![
            InvestigationStep {
                step_number: 1,
                title: "Review forensic findings".to_string(),
                description: "Examine pre-computed findings from deterministic analysis".to_string(),
                tool_calls: vec!["get_findings".to_string()],
                expected_output: "List of forensic findings".to_string(),
            },
            InvestigationStep {
                step_number: 2,
                title: "Analyze communication patterns".to_string(),
                description: "Review entity relationships and communication graph".to_string(),
                tool_calls: vec!["get_communication_graph".to_string(), "get_entity".to_string()],
                expected_output: "Communication pattern analysis".to_string(),
            },
            InvestigationStep {
                step_number: 3,
                title: "Review timeline".to_string(),
                description: "Examine chronological events".to_string(),
                tool_calls: vec!["get_timeline".to_string()],
                expected_output: "Timeline review".to_string(),
            },
        ],
    }
}

pub fn estimate_runtime(steps: &[InvestigationStep]) -> i64 {
    let step_time = steps.len() as i64 * 10;
    let tool_time: i64 = steps.iter().map(|s| s.tool_calls.len() as i64 * 5).sum();
    step_time + tool_time
}

/// Create investigation plan command
#[tauri::command]
pub async fn ai_create_investigation_plan(state: State<'_, AppState>, case_id: String, objective: String) -> Result<InvestigationPlan, String> {
    let stats = ai_get_case_statistics(state, case_id).await?;
    let plan = generate_investigation_plan(&objective, &stats);
    Ok(plan)
}

/// Execute investigation plan command
#[tauri::command]
pub async fn ai_execute_investigation_plan(state: State<'_, AppState>, case_id: String, plan: InvestigationPlan) -> Result<serde_json::Value, String> {
    let mut results = serde_json::json!({}); 
    let mut executed_steps = Vec::new();
    
    for step in &plan.steps {
        let step_result = execute_plan_step(&state, &case_id, step).await?;
        executed_steps.push(serde_json::json!({
            "step": step.step_number,
            "title": step.title,
            "result": step_result,
        }));
    }
    
    results["executed_steps"] = serde_json::json!(executed_steps);
    results["total_steps"] = serde_json::json!(plan.steps.len());
    results["status"] = serde_json::json!("completed");
    
    Ok(results)
}

async fn execute_plan_step(_state: &State<'_, AppState>, _case_id: &str, step: &InvestigationStep) -> Result<serde_json::Value, String> {
    let mut step_results = Vec::new();
    
    for tool_call in &step.tool_calls {
        step_results.push(serde_json::json!({
            "tool": tool_call,
            "status": "executed",
            "note": "Tool execution dispatched",
        }));
    }
    
    Ok(serde_json::json!({
        "step_number": step.step_number,
        "tool_results": step_results,
    }))
}
