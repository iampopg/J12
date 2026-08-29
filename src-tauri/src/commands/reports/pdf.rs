use std::fs;
use std::path::PathBuf;
use chrono::Utc;
use serde_json::Value;
use tauri::State;

use crate::AppState;
use super::builder::generate_report_data;

#[tauri::command]
pub async fn export_report_pdf(
    state: State<'_, AppState>,
    case_id: String,
    _sections: Vec<String>,
    _exhibits: Vec<Value>,
) -> Result<String, String> {
    let report_data = generate_report_data(state, serde_json::json!({ "case_id": case_id })).await?;

    let downloads_dir = dirs::download_dir().unwrap_or_else(|| PathBuf::from("."));
    let case_title = report_data["case"]["title"].as_str().unwrap_or("Forensic_Report");
    let safe_name = case_title.replace(|c: char| !c.is_alphanumeric() && c != '_' && c != '-', "_");
    let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
    let html_filename = format!("{}_Forensic_Report_{}.html", safe_name, timestamp);
    let output_path = downloads_dir.join(&html_filename);

    let html_content = format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<title>Forensic Examination Report - {}</title>
<style>
  body {{ font-family: 'Helvetica Neue', Arial, sans-serif; margin: 40px; color: #1e293b; background: #fff; line-height: 1.6; }}
  .header {{ border-bottom: 3px solid #0f172a; padding-bottom: 20px; margin-bottom: 30px; }}
  .title {{ font-size: 26px; font-weight: 800; color: #0f172a; margin: 0; }}
  .badge {{ display: inline-block; padding: 4px 8px; border-radius: 4px; font-size: 11px; font-weight: 700; text-transform: uppercase; }}
  .badge-critical {{ background: #fef2f2; color: #ef4444; border: 1px solid #f87171; }}
  .badge-high {{ background: #fff7ed; color: #f97316; border: 1px solid #fdba74; }}
  .table {{ width: 100%; border-collapse: collapse; margin-top: 15px; font-size: 13px; }}
  .table th, .table td {{ border: 1px solid #e2e8f0; padding: 8px 12px; text-align: left; }}
  .table th {{ background: #f8fafc; font-weight: 700; }}
  .section {{ margin-bottom: 35px; page-break-inside: avoid; }}
  .section-title {{ font-size: 18px; font-weight: 700; color: #1e293b; border-bottom: 1px solid #cbd5e1; padding-bottom: 6px; margin-bottom: 15px; }}
  .hash {{ font-family: monospace; font-size: 11px; color: #475569; word-break: break-all; }}
</style>
</head>
<body>
  <div class="header">
    <h1 class="title">J12 EMAIL FORENSIC EXAMINATION DOSSIER</h1>
    <div style="font-size: 14px; color: #64748b; margin-top: 5px;">
      Case: <strong>{}</strong> | Case #: <strong>{}</strong> | Examiner: <strong>{}</strong> | Generated: <strong>{}</strong>
    </div>
  </div>

  <div class="section">
    <div class="section-title">1. EXECUTIVE SUMMARY &amp; SCOPE</div>
    <p>{}</p>
  </div>

  <div class="section">
    <div class="section-title">2. EVIDENCE SOURCES &amp; INTEGRITY HASHES</div>
    <table class="table">
      <thead><tr><th>Filename</th><th>Format</th><th>Size (Bytes)</th><th>Acquired At</th><th>SHA-256 Hash</th></tr></thead>
      <tbody>
        {}
      </tbody>
    </table>
  </div>

  <div class="section">
    <div class="section-title">3. FORENSIC FINDINGS &amp; THREAT INTELLIGENCE</div>
    <table class="table">
      <thead><tr><th>Severity</th><th>Title</th><th>Type</th><th>Description</th></tr></thead>
      <tbody>
        {}
      </tbody>
    </table>
  </div>

  <div class="section">
    <div class="section-title">4. CHAIN OF CUSTODY AUDIT LOG</div>
    <table class="table">
      <thead><tr><th>Timestamp</th><th>Action</th><th>Performed By</th><th>Notes</th></tr></thead>
      <tbody>
        {}
      </tbody>
    </table>
  </div>
</body>
</html>"#,
        case_title,
        report_data["case"]["title"].as_str().unwrap_or(""),
        report_data["case"]["case_number"].as_str().unwrap_or("N/A"),
        report_data["case"]["examiner_name"].as_str().unwrap_or("Examiner"),
        report_data["generated_at"].as_str().unwrap_or(""),
        report_data["executive_summary"].as_str().unwrap_or(""),
        report_data["evidence_summary"].as_array().unwrap_or(&vec![]).iter().map(|e| {
            format!(
                "<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td class=\"hash\">{}</td></tr>",
                e["filename"].as_str().unwrap_or(""),
                e["format"].as_str().unwrap_or(""),
                e["size_bytes"].as_i64().unwrap_or(0),
                e["acquired_at"].as_str().unwrap_or(""),
                e["sha256"].as_str().unwrap_or(""),
            )
        }).collect::<Vec<_>>().join(""),
        report_data["findings"].as_array().unwrap_or(&vec![]).iter().map(|f| {
            let sev = f["severity"].as_str().unwrap_or("low");
            let badge_class = if sev == "critical" { "badge badge-critical" } else { "badge badge-high" };
            format!(
                "<tr><td><span class=\"{}\">{}</span></td><td><strong>{}</strong></td><td>{}</td><td>{}</td></tr>",
                badge_class,
                sev,
                f["title"].as_str().unwrap_or(""),
                f["type"].as_str().unwrap_or(""),
                f["description"].as_str().unwrap_or(""),
            )
        }).collect::<Vec<_>>().join(""),
        report_data["chain_of_custody"].as_array().unwrap_or(&vec![]).iter().map(|c| {
            format!(
                "<tr><td>{}</td><td><strong>{}</strong></td><td>{}</td><td>{}</td></tr>",
                c["timestamp"].as_str().unwrap_or(""),
                c["action"].as_str().unwrap_or(""),
                c["performed_by"].as_str().unwrap_or(""),
                c["notes"].as_str().unwrap_or(""),
            )
        }).collect::<Vec<_>>().join(""),
    );

    fs::write(&output_path, html_content).map_err(|e| format!("Failed to write HTML report: {}", e))?;

    crate::audit_logger::log_forensic_event(
        &case_id,
        "REPORTING",
        "REPORT_GENERATED",
        "Examiner",
        None,
        None,
        &format!("Generated HTML forensic examination dossier at \"{}\"", output_path.display())
    );

    Ok(output_path.to_string_lossy().to_string())
}
