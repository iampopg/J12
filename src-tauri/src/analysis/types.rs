use serde::{Deserialize, Serialize};

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
