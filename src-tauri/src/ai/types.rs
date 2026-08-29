use serde::{Deserialize, Serialize};

/// Kilo.ai model info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KiloAIModel {
    pub id: String,
    pub name: String,
    pub description: String,
    pub context_length: Option<i64>,
    pub is_recommended: bool,
}

/// Search query for emails
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchQuery {
    pub text: Option<String>,
    pub subject: Option<String>,
    pub from: Option<String>,
    pub to: Option<String>,
    pub date_from: Option<String>,
    pub date_to: Option<String>,
    pub has_attachments: Option<bool>,
    pub attachment_types: Option<Vec<String>>,
    pub folder_category: Option<String>,
    pub risk_score_min: Option<i64>,
    pub entity_id: Option<String>,
    pub limit: i64,
    pub offset: i64,
}

impl Default for SearchQuery {
    fn default() -> Self {
        Self {
            text: None,
            subject: None,
            from: None,
            to: None,
            date_from: None,
            date_to: None,
            has_attachments: None,
            attachment_types: None,
            folder_category: None,
            risk_score_min: None,
            entity_id: None,
            limit: 50,
            offset: 0,
        }
    }
}

/// Email search result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailResult {
    pub id: String,
    pub message_id: Option<String>,
    pub from_addr: String,
    pub from_display: Option<String>,
    pub to_addrs: String,
    pub subject: Option<String>,
    pub date_sent: Option<String>,
    pub folder_category: String,
    pub risk_score: i64,
    pub has_attachments: bool,
}

/// Attachment metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttachmentMetadata {
    pub id: String,
    pub filename: Option<String>,
    pub mime_type: String,
    pub size_bytes: i64,
    pub sha256: String,
    pub entropy: Option<f64>,
    pub risk_flags: Vec<String>,
}

/// Authentication results for an email
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthResults {
    pub email_id: String,
    pub spf_result: Option<String>,
    pub dkim_result: Option<String>,
    pub dmarc_result: Option<String>,
    pub arc_result: Option<String>,
    pub received_chain: Vec<String>,
    pub originating_ip: Option<String>,
}

/// Entity data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityData {
    pub id: String,
    pub email_address: String,
    pub display_name: Option<String>,
    pub sent_count: i64,
    pub received_count: i64,
    pub first_seen: Option<String>,
    pub last_seen: Option<String>,
}

/// Timeline event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineEvent {
    pub id: String,
    pub timestamp: String,
    pub event_type: String,
    pub actor: Option<String>,
    pub summary: Option<String>,
    pub email_id: Option<String>,
}

/// Finding data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FindingData {
    pub id: String,
    pub finding_type: String,
    pub severity: String,
    pub title: String,
    pub description: Option<String>,
    pub status: String,
}

/// Case statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaseStats {
    pub total_emails: i64,
    pub total_entities: i64,
    pub total_attachments: i64,
    pub total_findings: i64,
    pub inbox_count: i64,
    pub sent_count: i64,
    pub deleted_count: i64,
    pub spam_count: i64,
    pub date_from: Option<String>,
    pub date_to: Option<String>,
}

/// Tool risk classification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ToolRiskLevel {
    Harmless,
    Sensitive,
    Expensive,
    Dangerous,
}

/// Investigation budget
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvestigationBudget {
    pub max_tool_calls: i64,
    pub max_runtime_seconds: i64,
    pub max_results: i64,
    pub max_tokens: i64,
    pub max_attachment_bytes: i64,
    pub max_graph_nodes: i64,
}

impl Default for InvestigationBudget {
    fn default() -> Self {
        Self {
            max_tool_calls: 50,
            max_runtime_seconds: 120,
            max_results: 1000,
            max_tokens: 10000,
            max_attachment_bytes: 10485760,
            max_graph_nodes: 500,
        }
    }
}

/// AI Evidence Gateway policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceGatewayPolicy {
    pub provider_type: AIProviderType,
    pub enable_body: bool,
    pub enable_headers: bool,
    pub enable_pii: bool,
    pub enable_credentials: bool,
    pub enable_attachment_text: bool,
    pub enable_attachment_binary: bool,
    pub enable_chain_of_custody: bool,
    pub enable_investigator_notes: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AIProviderType {
    Local,
    KiloAI,
    Online,
}

impl EvidenceGatewayPolicy {
    pub fn local() -> Self {
        Self {
            provider_type: AIProviderType::Local,
            enable_body: true,
            enable_headers: true,
            enable_pii: true,
            enable_credentials: false,
            enable_attachment_text: true,
            enable_attachment_binary: false,
            enable_chain_of_custody: false,
            enable_investigator_notes: false,
        }
    }
    
    pub fn remote() -> Self {
        Self {
            provider_type: AIProviderType::Online,
            enable_body: false,
            enable_headers: true,
            enable_pii: false,
            enable_credentials: false,
            enable_attachment_text: false,
            enable_attachment_binary: false,
            enable_chain_of_custody: false,
            enable_investigator_notes: false,
        }
    }
    
    pub fn kiloai() -> Self {
        Self {
            provider_type: AIProviderType::KiloAI,
            enable_body: true,
            enable_headers: true,
            enable_pii: true,
            enable_credentials: false,
            enable_attachment_text: true,
            enable_attachment_binary: false,
            enable_chain_of_custody: false,
            enable_investigator_notes: false,
        }
    }
}

/// Tool definition for AI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub risk_level: ToolRiskLevel,
    pub parameters: Vec<ToolParameter>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolParameter {
    pub name: String,
    pub param_type: String,
    pub required: bool,
    pub description: String,
}

/// Investigation plan step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvestigationStep {
    pub step_number: i32,
    pub title: String,
    pub description: String,
    pub tool_calls: Vec<String>,
    pub expected_output: String,
}

/// Investigation plan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvestigationPlan {
    pub objective: String,
    pub normalized_objective: String,
    pub available_evidence: Vec<String>,
    pub unavailable_evidence: Vec<String>,
    pub limitations: Vec<String>,
    pub steps: Vec<InvestigationStep>,
    pub estimated_runtime_seconds: i64,
}

/// Timeline interpretation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineInterpretation {
    pub events: Vec<TimelineEvent>,
    pub anomalies: Vec<TimelineAnomaly>,
    pub narrative: String,
    pub clock_skew_detected: bool,
    pub timestamp_reversals: Vec<(String, String)>,
}

/// Timeline anomaly
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineAnomaly {
    pub event_id: String,
    pub anomaly_type: String,
    pub description: String,
    pub severity: String,
    pub timestamp: String,
}

/// Spoofing analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpoofingAnalysis {
    pub email_id: String,
    pub overall_risk: String,
    pub risk_score: i32,
    pub findings: Vec<SpoofingFinding>,
    pub recommendations: Vec<String>,
}

/// Individual spoofing finding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpoofingFinding {
    pub category: String,
    pub finding: String,
    pub severity: String,
    pub evidence: String,
}

/// Attachment triage result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttachmentTriage {
    pub attachments: Vec<AttachmentRisk>,
    pub critical_count: usize,
    pub high_count: usize,
    pub medium_count: usize,
    pub low_count: usize,
}

/// Attachment risk assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttachmentRisk {
    pub attachment_id: String,
    pub filename: String,
    pub risk_level: String,
    pub risk_score: i32,
    pub reasons: Vec<String>,
    pub recommendations: Vec<String>,
}

/// Graph analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphAnalysis {
    pub central_entities: Vec<EntityCentrality>,
    pub communities: Vec<Vec<String>>,
    pub anomalies: Vec<GraphAnomaly>,
    pub recommendations: Vec<String>,
}

/// Entity centrality score
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityCentrality {
    pub entity_id: String,
    pub email_address: String,
    pub centrality_score: f64,
    pub connection_count: usize,
}

/// Graph anomaly
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphAnomaly {
    pub anomaly_type: String,
    pub description: String,
    pub entities_involved: Vec<String>,
    pub severity: String,
}

/// Entity resolution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityResolution {
    pub candidates: Vec<EntityCandidate>,
    pub total_entities: usize,
}

/// Entity merge candidate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityCandidate {
    pub entity_ids: Vec<String>,
    pub email_addresses: Vec<String>,
    pub display_names: Vec<String>,
    pub confidence: f64,
    pub evidence: Vec<String>,
    pub recommendation: String,
}

/// Anomaly detection result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnomalyDetection {
    pub anomalies: Vec<EmailAnomaly>,
    pub total_scanned: usize,
    pub scan_duration_ms: i64,
}

/// Email anomaly
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailAnomaly {
    pub email_id: String,
    pub anomaly_type: String,
    pub description: String,
    pub severity: String,
    pub confidence: f64,
}

/// Report section
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportSection {
    pub title: String,
    pub content: String,
    pub section_type: String,
    pub evidence_refs: Vec<String>,
}

/// Investigation report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvestigationReport {
    pub title: String,
    pub generated_at: String,
    pub generated_by: String,
    pub model: String,
    pub sections: Vec<ReportSection>,
    pub metadata: ReportMetadata,
}

/// Report metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportMetadata {
    pub total_emails: i64,
    pub total_findings: i64,
    pub total_entities: i64,
    pub scan_duration_ms: i64,
}
