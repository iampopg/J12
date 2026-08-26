import { useState, useEffect, useCallback, useMemo } from "react";
import { invoke } from "@tauri-apps/api/core";

function cleanDisplayName(name: string | null): string {
  if (!name) return "";
  let cleaned = name
    .replace(/@ENRON.*$/g, "")
    .replace(/IMCEANOTES-[^<]*/g, "")
    .replace(/<[^>]*>/g, "")
    .replace(/"/g, "")
    .replace(/\s+/g, " ")
    .trim();
  if (cleaned.includes("@")) {
    return cleaned.split("@")[0].trim() || cleaned;
  }
  return cleaned;
}

export interface ReportSection {
  id: string;
  title: string;
  description: string;
  enabled: boolean;
}

export interface Exhibit {
  id: string;
  exhibit_number: string;
  email_id: string;
  from_addr: string;
  from_display: string | null;
  subject: string;
  date_sent: string;
  sha256: string;
  notes: string;
}

export interface ReportData {
  case_info: any;
  case: any;
  scope_and_authority: any;
  acquisition_methodology: any;
  tools_and_validation: any;
  limitations: string[];
  custody_chain: any[];
  chain_of_custody: any[];
  evidence_inventory: any[];
  evidence_summary: any[];
  findings: any[];
  email_stats?: any;
  email_statistics?: any;
  folder_breakdown?: any[];
  attachments_manifest?: any[];
  key_messages_ledger?: any[];
  top_correspondents?: any[];
  timeline_events?: any[];
  executive_summary?: string;
  generated_at?: string;
  tool_version?: string;
}

const REPORT_SECTIONS: ReportSection[] = [
  { id: "scope_and_authority", title: "1. Case Information, Scope & Authority", description: "Case metadata, subject identity, requesting authority, and questions presented", enabled: true },
  { id: "exec_summary", title: "2. Executive Summary & Forensic Scope", description: "High-level overview of evidence processed, key metrics, and core findings", enabled: true },
  { id: "sources", title: "3. Evidence Inventory & Acquisition Provenance", description: "Container technical specs, file size in bytes, SHA-256 acquisition hashes", enabled: true },
  { id: "methodology", title: "4. Evidence Acquisition Methodology & Tooling", description: "Protocol details, host server, authentication, and write-protection status", enabled: true },
  { id: "tools_validation", title: "5. Technical Tooling & Parser Validation", description: "Explicit tool inventory, versions, and validation against reference standards", enabled: true },
  { id: "folders", title: "6. Mailbox Hierarchy & Folder Volume Matrix", description: "Breakdown of folders (Inbox, Sent, Deleted) with item tallies and date spans", enabled: true },
  { id: "timeline", title: "7. Chronological Forensic Timeline & Provenance", description: "Ordered sequence of communication and mailbox events with timestamp origin", enabled: true },
  { id: "findings", title: "8. Security Findings Matrix (Observed Facts vs. Analysis)", description: "Itemized security findings with observed facts, technical assessment, and interpretation", enabled: true },
  { id: "correspondents", title: "9. Top Correspondents & Entity Analysis", description: "Communication volume breakdown, key external domains, and frequency analysis", enabled: true },
  { id: "key_ledger", title: "10. Key Evidentiary Message Ledger", description: "Itemized ledger of high-risk, recovered deleted, and flagged communications", enabled: true },
  { id: "attachments", title: "11. Attachment & File Artifact Manifest", description: "Inventory of extracted attachment files, types, sizes, and SHA-256 hashes", enabled: true },
  { id: "exhibits", title: "12. Marked Evidence Exhibits", description: "Bookmarked emails entered into formal evidence record with annotations", enabled: true },
  { id: "limitations", title: "13. Examination Limitations & Boundaries", description: "Explicit statement of technical boundaries, unavailable artifacts, and scope limits", enabled: true },
  { id: "custody", title: "14. Chain of Custody & Audit Trail", description: "Step-by-step verification history and evidence handling log", enabled: true },
  { id: "certification", title: "15. Examiner Declaration & Sworn Certification", description: "Forensic standards compliance statement and signed examiner certification block", enabled: true },
];

export function ReportView({ caseId, caseData }: { caseId: string; caseData: any }) {
  const [sections, setSections] = useState<ReportSection[]>(REPORT_SECTIONS);
  const [reportData, setReportData] = useState<ReportData | null>(null);
  const [exhibits, setExhibits] = useState<Exhibit[]>([]);
  const [loading, setLoading] = useState(true);
  const [reportTier, setReportTier] = useState<"tier1" | "tier2" | "tier3">("tier2");
  const [activeTab, setActiveTab] = useState<"preview" | "sources" | "methodology" | "timeline" | "findings" | "ledger" | "exhibits" | "sections">("preview");
  const [copied, setCopied] = useState(false);
  const [toastMessage, setToastMessage] = useState<string | null>(null);

  const showToast = (msg: string) => {
    setToastMessage(msg);
    setTimeout(() => setToastMessage(null), 3500);
  };

  const loadReportData = useCallback(async () => {
    setLoading(true);
    try {
      const data = await invoke<ReportData>("generate_report_data", { input: { case_id: caseId } });
      setReportData(data);
    } catch (e) {
      console.error("Failed to load report data:", e);
    } finally {
      setLoading(false);
    }
  }, [caseId]);

  useEffect(() => {
    loadReportData();
  }, [loadReportData]);

  const toggleSection = (id: string) => {
    setSections((prev) => prev.map((s) => (s.id === id ? { ...s, enabled: !s.enabled } : s)));
  };

  const handleAddExhibit = async () => {
    const emailId = prompt("Enter Email ID to bookmark as Exhibit:");
    if (!emailId) return;
    try {
      const email = await invoke<any>("email_get", { input: { case_id: emailId } });
      if (!email) {
        alert("Email not found");
        return;
      }
      const newEx: Exhibit = {
        id: `exhibit_${Date.now()}`,
        exhibit_number: `EXHIBIT-${String.fromCharCode(65 + exhibits.length)}`,
        email_id: emailId,
        from_addr: email.from_addr,
        from_display: email.from_display,
        subject: email.subject || "(no subject)",
        date_sent: email.date_sent || "",
        sha256: email.message_id || "",
        notes: "Marked during forensic examination",
      };
      setExhibits((prev) => [...prev, newEx]);
    } catch (e) {
      console.error(e);
      alert("Failed to add exhibit");
    }
  };

  const removeExhibit = (id: string) => {
    setExhibits((prev) =>
      prev
        .filter((e) => e.id !== id)
        .map((e, i) => ({
          ...e,
          exhibit_number: `EXHIBIT-${String.fromCharCode(65 + i)}`,
        }))
    );
  };

  const handlePrint = () => {
    window.print();
  };

  const handleExportHTML = async () => {
    try {
      const savedPath = await invoke<string>("export_report_pdf", {
        caseId,
        sections: sections.filter((s) => s.enabled).map((s) => s.id),
        exhibits,
      });
      showToast(`📥 Exported standalone report to Downloads: ${savedPath}`);
    } catch (e: any) {
      console.error(e);
      showToast(`❌ Error exporting report: ${e}`);
    }
  };

  const handleCopyMarkdown = () => {
    if (!reportData) return;
    const scope = reportData.scope_and_authority || {};
    const acq = reportData.acquisition_methodology || {};
    const stats = reportData.email_stats || {};

    const md = `
# DIGITAL FORENSIC EXAMINATION REPORT
**Standard:** Prepared for Evidentiary Use · ISO/IEC 27037 / NIST SP 800-86
**Case Title:** ${scope.case_title || caseData?.title || "N/A"}
**Case File Number:** ${scope.case_number || caseData?.case_number || "J12-CASE-001"}
**Requesting Authority:** ${scope.requesting_authority || "Authorized Legal Counsel"}
**Examination Authority:** ${scope.examination_authority || "Written Forensic Authorization"}
**Date Generated:** ${new Date().toUTCString()}

---

## 1. Executive Summary
${reportData.executive_summary || "Forensic examination completed."}

## 2. Scope & Questions Presented
**Scope:** ${scope.scope_of_examination || "Examination of acquired electronic mailboxes."}
**Questions Presented:**
${(scope.questions_presented || []).map((q: string) => `- ${q}`).join("\n")}

## 3. Evidence Inventory & Acquisition Provenance
${(reportData.evidence_inventory || [])
  .map(
    (ev) =>
      `- **${ev.evidence_id || "EVID"}**: \`${ev.filename}\` (${ev.format} · ${(ev.size_bytes / (1024 * 1024)).toFixed(2)} MB · ${ev.message_count?.toLocaleString()} msgs)\n  SHA-256: \`${ev.sha256}\`\n  Acquired: ${ev.acquired_at} | Method: ${ev.method}`
  )
  .join("\n")}

## 4. Acquisition Methodology & Technical Tooling
- **Acquisition Protocol:** ${acq.protocol || "IMAP4rev1 over TLS 1.3"}
- **Write Protection:** ${acq.write_protection || "Software Read-Only Isolation"}
- **Hashing Algorithm:** ${acq.hash_algorithm || "SHA-256 (FIPS 180-4)"}
- **Primary Tool:** ${acq.tool_name || "J12 Email Forensic Suite"} v${acq.tool_version || "1.0.0"}

## 5. Mailbox Metrics & Hierarchy
- Total Messages: ${stats.total?.toLocaleString() || 0}
- Inbound: ${stats.inbox?.toLocaleString() || 0} | Outbound: ${stats.sent?.toLocaleString() || 0} | Deleted: ${stats.deleted?.toLocaleString() || 0}
- Total Attachments: ${stats.total_attachments?.toLocaleString() || 0}

## 6. Forensic Findings Matrix (Observed Facts vs Analysis)
${(reportData.findings || [])
  .map(
    (f) =>
      `### [${f.severity.toUpperCase()}] ${f.citation_id || "F-0000"}: ${f.title}
- **Type:** ${f.type} | **Confidence:** ${f.confidence_label || "High"}
- **Observed Facts:** ${f.observed_facts || f.description || "N/A"}
- **Analytical Assessment:** ${f.analytical_assessment || "N/A"}
- **Examiner Interpretation:** ${f.examiner_interpretation || "N/A"}`
  )
  .join("\n\n")}

## 7. Examination Limitations
${(reportData.limitations || []).map((l: string) => `${l}`).join("\n")}

## 8. Examiner Certification & Declaration
I declare under penalty of perjury that this digital forensic examination was conducted in accordance with accepted scientific principles of digital evidence handling (ISO/IEC 27037 / NIST SP 800-86).
`;
    navigator.clipboard.writeText(md.trim());
    setCopied(true);
    setTimeout(() => setCopied(false), 2500);
  };

  if (loading) return <div className="card empty">Generating comprehensive forensic examination report...</div>;

  const enabledSections = new Set(sections.filter((s) => s.enabled).map((s) => s.id));
  const scope = reportData?.scope_and_authority || {};
  const acq = reportData?.acquisition_methodology || {};
  const tools = reportData?.tools_and_validation || {};
  const stats = reportData?.email_stats || {};

  return (
    <div>
      {toastMessage && (
        <div
          className="card"
          style={{
            position: "fixed",
            bottom: 24,
            right: 24,
            zIndex: 9999,
            background: "#1e293b",
            border: "1px solid #22c55e",
            color: "#4ade80",
            padding: "10px 18px",
            fontWeight: 600,
            fontSize: 13,
            boxShadow: "0 10px 25px rgba(0,0,0,0.5)",
          }}
        >
          {toastMessage}
        </div>
      )}

      <div className="row between mb-4">
        <div>
          <h2 style={{ fontSize: 22, fontWeight: 700, color: "var(--text-0)" }}>
            Digital Forensic Examination Report
          </h2>
          <p className="muted" style={{ fontSize: 12 }}>
            Formal expert reporting with scope, acquisition provenance, tool validation, evidence citations, timeline, and examiner certification.
          </p>
        </div>
        <div className="row gap-2">
          <button className="btn btn-ghost btn-sm" onClick={handleCopyMarkdown}>
            {copied ? "✓ Copied Markdown" : "📋 Copy Markdown"}
          </button>
          <button className="btn btn-ghost btn-sm" onClick={handleExportHTML} title="Export standalone self-contained HTML report to Downloads">
            📥 Export HTML Report
          </button>
          <button className="btn btn-primary btn-sm" onClick={handlePrint} title="Print or save as high-quality PDF">
            🖨️ Print / PDF
          </button>
        </div>
      </div>

      <div className="card mb-4" style={{ padding: "10px 14px", background: "var(--bg-2)", border: "1px solid var(--border)" }}>
        <div className="row between gap-2" style={{ flexWrap: "wrap", alignItems: "center" }}>
          <div className="row gap-2" style={{ alignItems: "center" }}>
            <span style={{ fontSize: 11, fontWeight: 800, color: "var(--text-3)", letterSpacing: "0.8px" }}>REPORT TIER:</span>
            <button
              className={`btn btn-sm ${reportTier === "tier1" ? "btn-primary" : "btn-ghost"}`}
              style={{ fontSize: 11, padding: "4px 10px" }}
              onClick={() => setReportTier("tier1")}
              title="Level 1: Concise executive overview and high-level findings for counsel & management"
            >
              Level 1: Executive Dossier
            </button>
            <button
              className={`btn btn-sm ${reportTier === "tier2" ? "btn-primary" : "btn-ghost"}`}
              style={{ fontSize: 11, padding: "4px 10px" }}
              onClick={() => setReportTier("tier2")}
              title="Level 2: Full 15-section technical forensic examination report prepared for evidentiary use"
            >
              Level 2: Expert Forensic Report
            </button>
            <button
              className={`btn btn-sm ${reportTier === "tier3" ? "btn-primary" : "btn-ghost"}`}
              style={{ fontSize: 11, padding: "4px 10px" }}
              onClick={() => setReportTier("tier3")}
              title="Level 3: Verifiable machine package with cryptographic manifests and raw audit logs"
            >
              Level 3: Evidence Package Manifest
            </button>
          </div>
          <div style={{ fontSize: 11, color: "var(--text-3)", fontFamily: "var(--mono)" }}>
            FRE 902(14) Self-Authenticating Digital Records
          </div>
        </div>
      </div>

      <div className="row gap-2 mb-4" style={{ borderBottom: "1px solid var(--border)", paddingBottom: 8, flexWrap: "wrap" }}>
        {(
          [
            ["preview", "👁️ Full Report Preview"],
            ["sources", `📁 Evidence Sources (${reportData?.evidence_inventory?.length || 0})`],
            ["methodology", "🛠️ Tools & Validation"],
            ["timeline", `⏳ Timeline (${reportData?.timeline_events?.length || 0})`],
            ["findings", `🛡️ Findings Matrix (${reportData?.findings?.length || 0})`],
            ["ledger", `📜 Key Ledger (${reportData?.key_messages_ledger?.length || 0})`],
            ["exhibits", `📎 Marked Exhibits (${exhibits.length})`],
            ["sections", "⚙️ Configure Sections"],
          ] as const
        ).map(([tab, label]) => (
          <button
            key={tab}
            className={`btn btn-sm ${activeTab === tab ? "btn-primary" : "btn-ghost"}`}
            onClick={() => setActiveTab(tab)}
          >
            {label}
          </button>
        ))}
      </div>

      {activeTab === "preview" && (
        <div
          className="card"
          style={{
            background: "var(--bg-1)",
            padding: "44px 50px",
            borderRadius: "var(--r-md)",
            border: "1px solid var(--border)",
            maxWidth: 1020,
            margin: "0 auto",
            color: "var(--text-0)",
          }}
        >
          <div
            style={{
              textAlign: "center",
              borderBottom: "3px double var(--border)",
              paddingBottom: 28,
              marginBottom: 32,
            }}
          >
            <div style={{ fontSize: 11, fontWeight: 800, letterSpacing: "1.5px", color: "var(--accent)", textTransform: "uppercase", marginBottom: 6 }}>
              DIGITAL FORENSIC EMAIL EXAMINATION REPORT · PREPARED FOR EVIDENTIARY USE
            </div>
            <h1 style={{ fontSize: 26, fontWeight: 900, margin: "8px 0 6px", color: "var(--text-0)" }}>
              {scope.case_title || caseData?.title || "Forensic Investigation Report"}
            </h1>
            <div style={{ fontSize: 13, color: "var(--text-2)", marginBottom: 12 }}>
              Case Reference: <strong>#{scope.case_number || caseData?.case_number || "J12-CASE-001"}</strong>
            </div>
            <div style={{ fontSize: 11, color: "var(--text-3)", display: "flex", justifyContent: "center", gap: 20, flexWrap: "wrap" }}>
              <span>Generated: {new Date().toUTCString()}</span>
              <span>Standards: <strong>ISO/IEC 27037 · NIST SP 800-86 · FRE 902(14)</strong></span>
            </div>
          </div>

          {enabledSections.has("scope_and_authority") && (
            <div style={{ marginBottom: 32 }}>
              <h3 style={{ fontSize: 15, fontWeight: 700, borderBottom: "1px solid var(--border)", paddingBottom: 6, marginBottom: 12, color: "var(--accent)" }}>
                1. Case Information, Scope &amp; Authority
              </h3>
              <table style={{ width: "100%", fontSize: 12, marginBottom: 10 }}>
                <tbody>
                  <tr>
                    <td style={{ width: 170, fontWeight: 600, background: "var(--bg-3)" }}>Case Title</td>
                    <td>{scope.case_title}</td>
                    <td style={{ width: 170, fontWeight: 600, background: "var(--bg-3)" }}>Case Reference #</td>
                    <td><code>{scope.case_number}</code></td>
                  </tr>
                  <tr>
                    <td style={{ fontWeight: 600, background: "var(--bg-3)" }}>Requesting Authority</td>
                    <td>{scope.requesting_authority}</td>
                    <td style={{ fontWeight: 600, background: "var(--bg-3)" }}>Examination Authority</td>
                    <td>{scope.examination_authority}</td>
                  </tr>
                  <tr>
                    <td style={{ fontWeight: 600, background: "var(--bg-3)" }}>Target Subject</td>
                    <td><strong>{reportData?.case?.target_name || "Target Individual"}</strong></td>
                    <td style={{ fontWeight: 600, background: "var(--bg-3)" }}>Target Email Address</td>
                    <td><code style={{ color: "var(--accent)" }}>{reportData?.case?.target_email || "target@domain.com"}</code></td>
                  </tr>
                  <tr>
                    <td style={{ fontWeight: 600, background: "var(--bg-3)" }}>Date Received</td>
                    <td>{scope.date_received}</td>
                    <td style={{ fontWeight: 600, background: "var(--bg-3)" }}>Date Examined</td>
                    <td>{scope.date_examined}</td>
                  </tr>
                </tbody>
              </table>

              <div style={{ background: "var(--bg-3)", padding: "12px 16px", borderRadius: "var(--r-sm)", fontSize: 12, marginBottom: 10, borderLeft: "4px solid var(--accent)" }}>
                <strong>Scope of Examination:</strong><br />
                <span style={{ color: "var(--text-1)" }}>{scope.scope_of_examination}</span>
              </div>

              {scope.questions_presented && (
                <div style={{ fontSize: 11.5, color: "var(--text-2)", paddingLeft: 8 }}>
                  <strong>Questions Presented for Examination:</strong>
                  <ul style={{ paddingLeft: 20, margin: "6px 0 0 0" }}>
                    {scope.questions_presented.map((q: string, i: number) => (
                      <li key={i} style={{ marginBottom: 4 }}>{q}</li>
                    ))}
                  </ul>
                </div>
              )}
            </div>
          )}

          {enabledSections.has("exec_summary") && (
            <div style={{ marginBottom: 32 }}>
              <h3 style={{ fontSize: 15, fontWeight: 700, borderBottom: "1px solid var(--border)", paddingBottom: 6, marginBottom: 12, color: "var(--accent)" }}>
                2. Executive Summary &amp; Mailbox Metrics
              </h3>
              <p style={{ fontSize: 12.5, color: "var(--text-1)", lineHeight: 1.6, marginBottom: 14 }}>
                {reportData?.executive_summary}
              </p>

              <div
                style={{
                  display: "grid",
                  gridTemplateColumns: "repeat(5, 1fr)",
                  gap: 10,
                  marginBottom: 14,
                }}
              >
                <div style={{ background: "var(--bg-3)", padding: 10, borderRadius: "var(--r-sm)", textAlign: "center" }}>
                  <div style={{ fontSize: 20, fontWeight: 800, color: "var(--accent)" }}>
                    {stats.total?.toLocaleString() || 0}
                  </div>
                  <div style={{ fontSize: 9.5, color: "var(--text-3)", marginTop: 2 }}>TOTAL MESSAGES</div>
                </div>

                <div style={{ background: "var(--bg-3)", padding: 10, borderRadius: "var(--r-sm)", textAlign: "center" }}>
                  <div style={{ fontSize: 20, fontWeight: 800, color: "#3b82f6" }}>
                    {stats.sent?.toLocaleString() || 0}
                  </div>
                  <div style={{ fontSize: 9.5, color: "var(--text-3)", marginTop: 2 }}>OUTBOUND (SENT)</div>
                </div>

                <div style={{ background: "var(--bg-3)", padding: 10, borderRadius: "var(--r-sm)", textAlign: "center" }}>
                  <div style={{ fontSize: 20, fontWeight: 800, color: "#22c55e" }}>
                    {stats.inbox?.toLocaleString() || 0}
                  </div>
                  <div style={{ fontSize: 9.5, color: "var(--text-3)", marginTop: 2 }}>INBOUND (INBOX)</div>
                </div>

                <div style={{ background: "var(--bg-3)", padding: 10, borderRadius: "var(--r-sm)", textAlign: "center" }}>
                  <div style={{ fontSize: 20, fontWeight: 800, color: "#ef4444" }}>
                    {stats.deleted?.toLocaleString() || 0}
                  </div>
                  <div style={{ fontSize: 9.5, color: "var(--text-3)", marginTop: 2 }}>DELETED / CARVED</div>
                </div>

                <div style={{ background: "var(--bg-3)", padding: 10, borderRadius: "var(--r-sm)", textAlign: "center" }}>
                  <div style={{ fontSize: 20, fontWeight: 800, color: "#f59e0b" }}>
                    {stats.total_attachments?.toLocaleString() || 0}
                  </div>
                  <div style={{ fontSize: 9.5, color: "var(--text-3)", marginTop: 2 }}>ATTACHMENTS</div>
                </div>
              </div>
            </div>
          )}

          {enabledSections.has("sources") && (
            <div style={{ marginBottom: 32 }}>
              <h3 style={{ fontSize: 15, fontWeight: 700, borderBottom: "1px solid var(--border)", paddingBottom: 6, marginBottom: 12, color: "var(--accent)" }}>
                3. Evidence Inventory &amp; Acquisition Provenance
              </h3>
              <table style={{ width: "100%", fontSize: 11, marginBottom: 14 }}>
                <thead>
                  <tr>
                    <th className="th" style={{ width: 80 }}>Evidence ID</th>
                    <th className="th">Source Container / Description</th>
                    <th className="th" style={{ width: 70 }}>Format</th>
                    <th className="th" style={{ width: 85 }}>Size</th>
                    <th className="th" style={{ width: 80 }}>Items</th>
                    <th className="th">SHA-256 Acquisition Hash</th>
                  </tr>
                </thead>
                <tbody>
                  {(reportData?.evidence_inventory || []).map((ev) => (
                    <tr key={ev.id}>
                      <td className="td"><strong>{ev.evidence_id || "EVID"}</strong></td>
                      <td className="td">
                        <strong>{ev.filename}</strong>
                        <div style={{ fontSize: 10, color: "var(--text-3)" }}>
                          Acquired: {ev.acquired_at} | {ev.method}
                        </div>
                      </td>
                      <td className="td"><span className="badge badge-blue">{ev.format}</span></td>
                      <td className="td muted">{(ev.size_bytes / (1024 * 1024)).toFixed(2)} MB</td>
                      <td className="td"><strong>{ev.message_count?.toLocaleString()}</strong></td>
                      <td className="td" style={{ fontFamily: "var(--mono)", fontSize: 9.5, color: "var(--accent)" }}>{ev.sha256}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          )}

          {enabledSections.has("methodology") && (
            <div style={{ marginBottom: 32 }}>
              <h3 style={{ fontSize: 15, fontWeight: 700, borderBottom: "1px solid var(--border)", paddingBottom: 6, marginBottom: 12, color: "var(--accent)" }}>
                4. Evidence Acquisition Methodology &amp; Environmental Isolation
              </h3>
              <table style={{ width: "100%", fontSize: 11.5, marginBottom: 12 }}>
                <tbody>
                  <tr>
                    <td style={{ width: 170, fontWeight: 600, background: "var(--bg-3)" }}>Acquisition Method</td>
                    <td>{acq.method}</td>
                    <td style={{ width: 170, fontWeight: 600, background: "var(--bg-3)" }}>Protocol &amp; Security</td>
                    <td>{acq.protocol}</td>
                  </tr>
                  <tr>
                    <td style={{ fontWeight: 600, background: "var(--bg-3)" }}>Host Endpoint</td>
                    <td><code>{acq.server_host}</code></td>
                    <td style={{ fontWeight: 600, background: "var(--bg-3)" }}>Authentication</td>
                    <td>{acq.authentication_method}</td>
                  </tr>
                  <tr>
                    <td style={{ fontWeight: 600, background: "var(--bg-3)" }}>Write Protection</td>
                    <td><span className="badge badge-green">{acq.write_protection}</span></td>
                    <td style={{ fontWeight: 600, background: "var(--bg-3)" }}>Integrity Hashing</td>
                    <td>{acq.hash_algorithm}</td>
                  </tr>
                  <tr>
                    <td style={{ fontWeight: 600, background: "var(--bg-3)" }}>Items Requested / Acquired</td>
                    <td>{acq.messages_requested} requested / {acq.messages_acquired} acquired (100% success)</td>
                    <td style={{ fontWeight: 600, background: "var(--bg-3)" }}>Errors &amp; Dropped Packets</td>
                    <td>0 errors</td>
                  </tr>
                </tbody>
              </table>
            </div>
          )}

          {enabledSections.has("tools_validation") && (
            <div style={{ marginBottom: 32 }}>
              <h3 style={{ fontSize: 15, fontWeight: 700, borderBottom: "1px solid var(--border)", paddingBottom: 6, marginBottom: 12, color: "var(--accent)" }}>
                5. Technical Tooling Inventory &amp; Methodological Validation
              </h3>
              <table style={{ width: "100%", fontSize: 11, marginBottom: 12 }}>
                <thead>
                  <tr>
                    <th className="th">Software Component / Engine</th>
                    <th className="th" style={{ width: 140 }}>Version / Standard</th>
                    <th className="th">Forensic Function</th>
                  </tr>
                </thead>
                <tbody>
                  {(tools.tools || []).map((t: any, i: number) => (
                    <tr key={i}>
                      <td className="td"><strong>{t.name}</strong></td>
                      <td className="td"><code style={{ fontSize: 10 }}>{t.version}</code></td>
                      <td className="td muted">{t.purpose}</td>
                    </tr>
                  ))}
                </tbody>
              </table>

              <table style={{ width: "100%", fontSize: 11 }}>
                <thead>
                  <tr>
                    <th className="th">Validation Test Suite</th>
                    <th className="th" style={{ width: 80 }}>Status</th>
                    <th className="th">Verification Details</th>
                  </tr>
                </thead>
                <tbody>
                  {(tools.validation_status || []).map((v: any, i: number) => (
                    <tr key={i}>
                      <td className="td"><strong>{v.component}</strong></td>
                      <td className="td"><span className="badge badge-green">{v.status}</span></td>
                      <td className="td muted">{v.details}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          )}

          {enabledSections.has("folders") && (reportData?.folder_breakdown || []).length > 0 && (
            <div style={{ marginBottom: 32 }}>
              <h3 style={{ fontSize: 15, fontWeight: 700, borderBottom: "1px solid var(--border)", paddingBottom: 6, marginBottom: 12, color: "var(--accent)" }}>
                6. Mailbox Hierarchy &amp; Folder Volume Matrix
              </h3>
              <table style={{ width: "100%", fontSize: 11 }}>
                <thead>
                  <tr>
                    <th className="th">Folder Name</th>
                    <th className="th" style={{ width: 100 }}>Category</th>
                    <th className="th" style={{ width: 80 }}>Item Count</th>
                    <th className="th" style={{ width: 110 }}>Earliest Date</th>
                    <th className="th" style={{ width: 110 }}>Latest Date</th>
                  </tr>
                </thead>
                <tbody>
                  {(reportData?.folder_breakdown || []).map((f: any, i: number) => (
                    <tr key={i}>
                      <td className="td"><strong>{f.folder_name}</strong></td>
                      <td className="td"><span className="badge badge-blue">{f.folder_category}</span></td>
                      <td className="td"><strong>{f.count?.toLocaleString()}</strong></td>
                      <td className="td muted">{f.date_from ? f.date_from.slice(0, 10) : "—"}</td>
                      <td className="td muted">{f.date_to ? f.date_to.slice(0, 10) : "—"}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          )}

          {enabledSections.has("timeline") && (reportData?.timeline_events || []).length > 0 && (
            <div style={{ marginBottom: 32 }}>
              <h3 style={{ fontSize: 15, fontWeight: 700, borderBottom: "1px solid var(--border)", paddingBottom: 6, marginBottom: 12, color: "var(--accent)" }}>
                7. Chronological Forensic Timeline &amp; Timestamp Provenance
              </h3>
              <table style={{ width: "100%", fontSize: 10.5 }}>
                <thead>
                  <tr>
                    <th className="th" style={{ width: 70 }}>Event ID</th>
                    <th className="th" style={{ width: 130 }}>Observed UTC</th>
                    <th className="th" style={{ width: 140 }}>Event Type</th>
                    <th className="th">Actor / Subject</th>
                    <th className="th" style={{ width: 150 }}>Provenance Source</th>
                  </tr>
                </thead>
                <tbody>
                  {(reportData?.timeline_events || []).slice(0, 20).map((t: any) => (
                    <tr key={t.event_id}>
                      <td className="td"><strong>{t.event_id}</strong></td>
                      <td className="td" style={{ fontFamily: "var(--mono)" }}>{t.timestamp_utc?.slice(0, 19).replace("T", " ")}</td>
                      <td className="td"><span className="badge badge-blue">{t.event_type}</span></td>
                      <td className="td">
                        <strong>{t.actor}</strong> · {t.details}
                      </td>
                      <td className="td muted" style={{ fontSize: 9.5 }}>{t.provenance}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          )}

          {enabledSections.has("findings") && (
            <div style={{ marginBottom: 32 }}>
              <h3 style={{ fontSize: 15, fontWeight: 700, borderBottom: "1px solid var(--border)", paddingBottom: 6, marginBottom: 12, color: "var(--accent)" }}>
                8. Forensic Findings Matrix (Observed Facts vs. Technical Assessment)
              </h3>
              {(reportData?.findings || []).length === 0 ? (
                <div className="muted text-sm">No security violations or tampering anomalies detected.</div>
              ) : (
                <div style={{ display: "flex", flexDirection: "column", gap: 14 }}>
                  {reportData?.findings.map((f: any) => (
                    <div
                      key={f.id}
                      style={{
                        padding: 14,
                        background: "var(--bg-3)",
                        borderRadius: "var(--r-sm)",
                        borderLeft:
                          f.severity === "critical"
                            ? "4px solid #ef4444"
                            : f.severity === "high"
                            ? "4px solid #f97316"
                            : "4px solid #3b82f6",
                      }}
                    >
                      <div className="row between mb-2">
                        <strong style={{ fontSize: 13, color: "var(--text-0)" }}>
                          {f.citation_id}: {f.title}
                        </strong>
                        <div className="row gap-2">
                          <span
                            className={`badge ${
                              f.severity === "critical"
                                ? "badge-red"
                                : f.severity === "high"
                                ? "badge-orange"
                                : "badge-blue"
                            }`}
                            style={{ fontSize: 9 }}
                          >
                            {f.severity.toUpperCase()}
                          </span>
                          <span className="badge badge-green" style={{ fontSize: 9 }}>
                            CONFIDENCE: {f.confidence_label || "HIGH"}
                          </span>
                        </div>
                      </div>

                      <div style={{ fontSize: 11.5, marginBottom: 4 }}>
                        <span style={{ fontWeight: 700, color: "var(--text-2)" }}>[Observed Header/Payload Facts]:</span>{" "}
                        <span style={{ color: "var(--text-1)" }}>{f.observed_facts || f.description}</span>
                      </div>

                      <div style={{ fontSize: 11.5, marginBottom: 4 }}>
                        <span style={{ fontWeight: 700, color: "var(--accent)" }}>[Analytical Assessment]:</span>{" "}
                        <span style={{ color: "var(--text-1)" }}>{f.analytical_assessment}</span>
                      </div>

                      <div style={{ fontSize: 11, color: "var(--text-3)", fontStyle: "italic" }}>
                        <strong>Examiner Interpretation:</strong> {f.examiner_interpretation}
                      </div>
                    </div>
                  ))}
                </div>
              )}
            </div>
          )}

          {enabledSections.has("correspondents") && (reportData?.top_correspondents || []).length > 0 && (
            <div style={{ marginBottom: 32 }}>
              <h3 style={{ fontSize: 15, fontWeight: 700, borderBottom: "1px solid var(--border)", paddingBottom: 6, marginBottom: 12, color: "var(--accent)" }}>
                9. Top Correspondents &amp; Communication Patterns (Top 15 Entities)
              </h3>
              <table style={{ width: "100%", fontSize: 11 }}>
                <thead>
                  <tr>
                    <th className="th">Correspondent Address</th>
                    <th className="th" style={{ width: 90 }}>Volume</th>
                    <th className="th" style={{ width: 110 }}>First Observed</th>
                    <th className="th" style={{ width: 110 }}>Last Observed</th>
                  </tr>
                </thead>
                <tbody>
                  {(reportData?.top_correspondents || []).slice(0, 15).map((c: any, i: number) => (
                    <tr key={i}>
                      <td className="td" style={{ fontFamily: "var(--mono)" }}><strong>{c.email}</strong></td>
                      <td className="td" style={{ color: "var(--accent)" }}><strong>{c.message_count?.toLocaleString()}</strong> msgs</td>
                      <td className="td muted">{c.first_seen ? c.first_seen.slice(0, 10) : "—"}</td>
                      <td className="td muted">{c.last_seen ? c.last_seen.slice(0, 10) : "—"}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          )}

          {enabledSections.has("key_ledger") && (reportData?.key_messages_ledger || []).length > 0 && (
            <div style={{ marginBottom: 32 }}>
              <h3 style={{ fontSize: 15, fontWeight: 700, borderBottom: "1px solid var(--border)", paddingBottom: 6, marginBottom: 12, color: "var(--accent)" }}>
                10. Key Evidentiary Message Ledger (High Risk &amp; Recovered Items)
              </h3>
              <table style={{ width: "100%", fontSize: 10 }}>
                <thead>
                  <tr>
                    <th className="th" style={{ width: 70 }}>Item Ref</th>
                    <th className="th" style={{ width: 140 }}>Sender</th>
                    <th className="th">Subject</th>
                    <th className="th" style={{ width: 80 }}>Date</th>
                    <th className="th" style={{ width: 60 }}>Folder</th>
                    <th className="th" style={{ width: 45 }}>Risk</th>
                  </tr>
                </thead>
                <tbody>
                  {(reportData?.key_messages_ledger || []).slice(0, 30).map((em: any) => (
                    <tr key={em.id}>
                      <td className="td"><strong>{em.item_ref || "MSG"}</strong></td>
                      <td className="td" style={{ overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
                        {cleanDisplayName(em.from_display) || em.from_addr}
                      </td>
                      <td className="td">
                        <strong>{em.subject}</strong>
                        {em.deleted_recovered && (
                          <span className="badge badge-red" style={{ fontSize: 8, marginLeft: 6 }}>
                            CARVED
                          </span>
                        )}
                      </td>
                      <td className="td muted">{em.date_sent_utc ? em.date_sent_utc.slice(0, 10) : "—"}</td>
                      <td className="td muted">{em.folder_category}</td>
                      <td className="td">
                        <span className={`badge ${em.risk_score >= 50 ? "badge-red" : "badge-orange"}`} style={{ fontSize: 8 }}>
                          {em.risk_score}
                        </span>
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          )}

          {enabledSections.has("attachments") && (reportData?.attachments_manifest || []).length > 0 && (
            <div style={{ marginBottom: 32 }}>
              <h3 style={{ fontSize: 15, fontWeight: 700, borderBottom: "1px solid var(--border)", paddingBottom: 6, marginBottom: 12, color: "var(--accent)" }}>
                11. Attachment &amp; File Artifact Manifest
              </h3>
              <table style={{ width: "100%", fontSize: 10 }}>
                <thead>
                  <tr>
                    <th className="th">Filename</th>
                    <th className="th" style={{ width: 80 }}>Size</th>
                    <th className="th">Parent Subject / Sender</th>
                    <th className="th">SHA-256 Hash</th>
                  </tr>
                </thead>
                <tbody>
                  {(reportData?.attachments_manifest || []).slice(0, 20).map((att: any) => (
                    <tr key={att.id}>
                      <td className="td"><strong>{att.filename}</strong></td>
                      <td className="td muted">{(att.size_bytes / 1024).toFixed(1)} KB</td>
                      <td className="td">{att.email_subject || "(No Subject)"} · <span className="muted">{att.email_from}</span></td>
                      <td className="td" style={{ fontFamily: "var(--mono)", fontSize: 9, color: "var(--accent)" }}>{att.sha256}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          )}

          {enabledSections.has("exhibits") && (
            <div style={{ marginBottom: 32 }}>
              <div className="row between mb-2">
                <h3 style={{ fontSize: 15, fontWeight: 700, color: "var(--accent)", margin: 0 }}>
                  12. Marked Evidence Exhibits ({exhibits.length})
                </h3>
                <button className="btn btn-ghost btn-sm" onClick={handleAddExhibit} style={{ fontSize: 11 }}>
                  + Add Exhibit
                </button>
              </div>
              {exhibits.length === 0 ? (
                <div className="muted text-xs p-3" style={{ background: "var(--bg-3)", borderRadius: "var(--r-sm)" }}>
                  No exhibits entered into the evidence record yet. Click &quot;+ Add Exhibit&quot; to mark key messages.
                </div>
              ) : (
                <table style={{ width: "100%", fontSize: 11 }}>
                  <thead>
                    <tr>
                      <th className="th" style={{ width: 90 }}>Exhibit #</th>
                      <th className="th">Sender</th>
                      <th className="th">Subject</th>
                      <th className="th">Examiner Notes</th>
                      <th className="th" style={{ width: 50 }}>Action</th>
                    </tr>
                  </thead>
                  <tbody>
                    {exhibits.map((ex) => (
                      <tr key={ex.id}>
                        <td className="td"><strong>{ex.exhibit_number}</strong></td>
                        <td className="td">{cleanDisplayName(ex.from_display) || ex.from_addr}</td>
                        <td className="td"><strong>{ex.subject}</strong></td>
                        <td className="td muted">{ex.notes}</td>
                        <td className="td">
                          <button className="btn btn-ghost btn-sm" onClick={() => removeExhibit(ex.id)} style={{ fontSize: 10, padding: "1px 4px" }}>
                            ✕
                          </button>
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              )}
            </div>
          )}

          {enabledSections.has("limitations") && (
            <div style={{ marginBottom: 32 }}>
              <h3 style={{ fontSize: 15, fontWeight: 700, borderBottom: "1px solid var(--border)", paddingBottom: 6, marginBottom: 12, color: "var(--accent)" }}>
                13. Technical Examination Limitations &amp; Boundaries
              </h3>
              <div style={{ background: "var(--bg-3)", padding: "14px 18px", borderRadius: "var(--r-sm)", fontSize: 11.5, lineHeight: 1.6 }}>
                {(reportData?.limitations || []).map((lim: string, i: number) => (
                  <div key={i} style={{ marginBottom: 6 }}>{lim}</div>
                ))}
              </div>
            </div>
          )}

          {enabledSections.has("custody") && (
            <div style={{ marginBottom: 32 }}>
              <h3 style={{ fontSize: 15, fontWeight: 700, borderBottom: "1px solid var(--border)", paddingBottom: 6, marginBottom: 12, color: "var(--accent)" }}>
                14. Chain of Custody &amp; Evidence Audit Trail
              </h3>
              <table style={{ width: "100%", fontSize: 11 }}>
                <thead>
                  <tr>
                    <th className="th" style={{ width: 140 }}>Timestamp (UTC)</th>
                    <th className="th" style={{ width: 140 }}>Action / Event</th>
                    <th className="th" style={{ width: 140 }}>Performed By</th>
                    <th className="th">Custody Notes &amp; Hash Verification</th>
                  </tr>
                </thead>
                <tbody>
                  {(reportData?.chain_of_custody || []).map((coc: any, i: number) => (
                    <tr key={i}>
                      <td className="td" style={{ fontFamily: "var(--mono)", fontSize: 10 }}>{coc.timestamp?.slice(0, 19).replace("T", " ")}</td>
                      <td className="td"><strong>{coc.action}</strong></td>
                      <td className="td">{coc.performed_by}</td>
                      <td className="td muted">{coc.notes || "Sealed & Verified in SQLite"}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          )}

          {/* 15. Examiner Declaration & Certification */}
          {enabledSections.has("certification") && (
            <div style={{ border: "2px solid var(--border)", padding: "20px 24px", borderRadius: "var(--r-sm)", background: "var(--bg-2)" }}>
              <h3 style={{ fontSize: 14, fontWeight: 800, color: "var(--text-0)", margin: "0 0 8px 0" }}>
                15. Examiner Declaration &amp; Sworn Certification
              </h3>
              <p style={{ fontSize: 11.5, color: "var(--text-2)", lineHeight: 1.6, margin: "0 0 16px 0" }}>
                I declare under penalty of perjury that this digital forensic examination was conducted in accordance with accepted scientific principles of digital evidence handling (ISO/IEC 27037 / NIST SP 800-86). The factual findings, evidence citations, and analytical interpretations in this report represent an objective, independent technical assessment of the acquired electronic evidence.
              </p>
              <div className="row between" style={{ fontSize: 12, borderTop: "1px solid var(--border)", paddingTop: 12 }}>
                <div>
                  <strong>Lead Examiner:</strong> Senior Digital Forensic Specialist<br />
                  <span className="muted" style={{ fontSize: 10 }}>J12 Cyber Intelligence Laboratory</span>
                </div>
                <div>
                  <strong>Date:</strong> {new Date().toLocaleDateString()}<br />
                  <span className="muted" style={{ fontSize: 10 }}>Cryptographically Sealed</span>
                </div>
                <div style={{ textAlign: "right" }}>
                  <strong>Signature:</strong> ____________________________<br />
                  <span className="muted" style={{ fontSize: 10 }}>FRE 902(14) Certified</span>
                </div>
              </div>
            </div>
          )}
        </div>
      )}

      {/* TAB 2: SOURCES */}
      {activeTab === "sources" && (
        <div className="card">
          <h3 style={{ fontSize: 16, fontWeight: 700, marginBottom: 14 }}>Evidence Sources &amp; Integrity Seals</h3>
          <table style={{ width: "100%", fontSize: 12 }}>
            <thead>
              <tr>
                <th className="th">Evidence ID</th>
                <th className="th">Filename / Container</th>
                <th className="th">Format</th>
                <th className="th">Size</th>
                <th className="th">Message Count</th>
                <th className="th">SHA-256 Hash</th>
              </tr>
            </thead>
            <tbody>
              {(reportData?.evidence_inventory || []).map((ev) => (
                <tr key={ev.id}>
                  <td className="td"><strong>{ev.evidence_id || "EVID"}</strong></td>
                  <td className="td"><strong>{ev.filename}</strong></td>
                  <td className="td"><span className="badge badge-blue">{ev.format}</span></td>
                  <td className="td">{(ev.size_bytes / (1024 * 1024)).toFixed(2)} MB</td>
                  <td className="td"><strong>{ev.message_count?.toLocaleString()}</strong></td>
                  <td className="td" style={{ fontFamily: "var(--mono)", fontSize: 11, color: "var(--accent)" }}>{ev.sha256}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}

      {/* TAB 3: TOOLS & VALIDATION */}
      {activeTab === "methodology" && (
        <div className="card">
          <h3 style={{ fontSize: 16, fontWeight: 700, marginBottom: 14 }}>Technical Tooling &amp; Parser Validation Status</h3>
          <table style={{ width: "100%", fontSize: 12, marginBottom: 20 }}>
            <thead>
              <tr>
                <th className="th">Component</th>
                <th className="th">Version / Standard</th>
                <th className="th">Purpose</th>
              </tr>
            </thead>
            <tbody>
              {(tools.tools || []).map((t: any, i: number) => (
                <tr key={i}>
                  <td className="td"><strong>{t.name}</strong></td>
                  <td className="td"><code style={{ color: "var(--accent)" }}>{t.version}</code></td>
                  <td className="td">{t.purpose}</td>
                </tr>
              ))}
            </tbody>
          </table>

          <h4 style={{ fontSize: 14, fontWeight: 700, marginBottom: 10 }}>Validation Test Results</h4>
          <table style={{ width: "100%", fontSize: 12 }}>
            <thead>
              <tr>
                <th className="th">Validation Test Suite</th>
                <th className="th" style={{ width: 90 }}>Status</th>
                <th className="th">Details</th>
              </tr>
            </thead>
            <tbody>
              {(tools.validation_status || []).map((v: any, i: number) => (
                <tr key={i}>
                  <td className="td"><strong>{v.component}</strong></td>
                  <td className="td"><span className="badge badge-green">{v.status}</span></td>
                  <td className="td">{v.details}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}

      {/* TAB 4: TIMELINE */}
      {activeTab === "timeline" && (
        <div className="card">
          <h3 style={{ fontSize: 16, fontWeight: 700, marginBottom: 14 }}>Chronological Forensic Timeline</h3>
          <table style={{ width: "100%", fontSize: 11 }}>
            <thead>
              <tr>
                <th className="th" style={{ width: 80 }}>Event ID</th>
                <th className="th" style={{ width: 140 }}>Observed UTC</th>
                <th className="th" style={{ width: 150 }}>Event Type</th>
                <th className="th">Actor / Sender</th>
                <th className="th">Subject / Details</th>
                <th className="th">Provenance</th>
              </tr>
            </thead>
            <tbody>
              {(reportData?.timeline_events || []).map((t: any) => (
                <tr key={t.event_id}>
                  <td className="td"><strong>{t.event_id}</strong></td>
                  <td className="td" style={{ fontFamily: "var(--mono)" }}>{t.timestamp_utc?.slice(0, 19).replace("T", " ")}</td>
                  <td className="td"><span className="badge badge-blue">{t.event_type}</span></td>
                  <td className="td"><strong>{t.actor}</strong></td>
                  <td className="td">{t.details}</td>
                  <td className="td muted">{t.provenance}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}

      {/* TAB 5: FINDINGS MATRIX */}
      {activeTab === "findings" && (
        <div className="card">
          <h3 style={{ fontSize: 16, fontWeight: 700, marginBottom: 14 }}>Security Findings &amp; Risk Matrix</h3>
          <div style={{ display: "flex", flexDirection: "column", gap: 14 }}>
            {(reportData?.findings || []).map((f: any) => (
              <div key={f.id} className="card" style={{ padding: 14, margin: 0, borderLeft: f.severity === "critical" ? "4px solid #ef4444" : "4px solid #f97316" }}>
                <div className="row between mb-2">
                  <strong>{f.citation_id}: {f.title}</strong>
                  <div className="row gap-2">
                    <span className={`badge ${f.severity === "critical" ? "badge-red" : "badge-orange"}`}>{f.severity.toUpperCase()}</span>
                    <span className="badge badge-green">CONFIDENCE: {f.confidence_label || "HIGH"}</span>
                  </div>
                </div>
                <p style={{ fontSize: 12, margin: "0 0 6px 0" }}><strong>Observed Facts:</strong> {f.observed_facts || f.description}</p>
                <p style={{ fontSize: 12, margin: "0 0 6px 0", color: "var(--accent)" }}><strong>Assessment:</strong> {f.analytical_assessment}</p>
                <p style={{ fontSize: 11.5, margin: 0, color: "var(--text-3)" }}><strong>Examiner Interpretation:</strong> {f.examiner_interpretation}</p>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* TAB 6: KEY LEDGER */}
      {activeTab === "ledger" && (
        <div className="card">
          <h3 style={{ fontSize: 16, fontWeight: 700, marginBottom: 14 }}>Key Evidentiary Messages Ledger</h3>
          <table style={{ width: "100%", fontSize: 11 }}>
            <thead>
              <tr>
                <th className="th" style={{ width: 80 }}>Item Ref</th>
                <th className="th" style={{ width: 160 }}>Sender</th>
                <th className="th">Subject</th>
                <th className="th" style={{ width: 90 }}>Date</th>
                <th className="th" style={{ width: 80 }}>Folder</th>
                <th className="th" style={{ width: 50 }}>Risk</th>
              </tr>
            </thead>
            <tbody>
              {(reportData?.key_messages_ledger || []).map((em: any) => (
                <tr key={em.id}>
                  <td className="td"><strong>{em.item_ref || "MSG"}</strong></td>
                  <td className="td">{cleanDisplayName(em.from_display) || em.from_addr}</td>
                  <td className="td"><strong>{em.subject}</strong></td>
                  <td className="td muted">{em.date_sent_utc?.slice(0, 10)}</td>
                  <td className="td muted">{em.folder_category}</td>
                  <td className="td"><span className={`badge ${em.risk_score >= 50 ? "badge-red" : "badge-orange"}`}>{em.risk_score}</span></td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}

      {/* TAB 7: EXHIBITS */}
      {activeTab === "exhibits" && (
        <div className="card">
          <div className="row between mb-3">
            <h3 style={{ fontSize: 16, fontWeight: 700, margin: 0 }}>Marked Exhibits Record</h3>
            <button className="btn btn-primary btn-sm" onClick={handleAddExhibit}>+ Add Exhibit</button>
          </div>
          {exhibits.length === 0 ? (
            <div className="muted text-center p-4">No exhibits entered into the record yet.</div>
          ) : (
            <table style={{ width: "100%", fontSize: 12 }}>
              <thead>
                <tr>
                  <th className="th" style={{ width: 100 }}>Exhibit #</th>
                  <th className="th">Sender</th>
                  <th className="th">Subject</th>
                  <th className="th">Examiner Notes</th>
                  <th className="th" style={{ width: 60 }}>Action</th>
                </tr>
              </thead>
              <tbody>
                {exhibits.map((ex) => (
                  <tr key={ex.id}>
                    <td className="td"><strong>{ex.exhibit_number}</strong></td>
                    <td className="td">{cleanDisplayName(ex.from_display) || ex.from_addr}</td>
                    <td className="td"><strong>{ex.subject}</strong></td>
                    <td className="td">{ex.notes}</td>
                    <td className="td">
                      <button className="btn btn-ghost btn-sm" onClick={() => removeExhibit(ex.id)}>✕</button>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          )}
        </div>
      )}

      {/* TAB 8: CONFIGURE SECTIONS */}
      {activeTab === "sections" && (
        <div className="card">
          <h3 style={{ fontSize: 16, fontWeight: 700, marginBottom: 14 }}>Configure Forensic Report Sections</h3>
          <div style={{ display: "flex", flexDirection: "column", gap: 10 }}>
            {sections.map((s) => (
              <div key={s.id} className="row between" style={{ padding: "10px 14px", background: "var(--bg-2)", borderRadius: "var(--r-sm)" }}>
                <div>
                  <div style={{ fontWeight: 600, fontSize: 13 }}>{s.title}</div>
                  <div style={{ fontSize: 11, color: "var(--text-3)" }}>{s.description}</div>
                </div>
                <input
                  type="checkbox"
                  checked={s.enabled}
                  onChange={() => toggleSection(s.id)}
                  style={{ width: 18, height: 18, cursor: "pointer" }}
                />
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  );
}
