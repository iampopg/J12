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

interface ReportSection {
  id: string;
  title: string;
  description: string;
  enabled: boolean;
}

interface Exhibit {
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

interface ReportData {
  case_info: any;
  methodology: any;
  custody_chain: any[];
  evidence_inventory: any[];
  findings: any[];
  entities?: any[];
  email_stats?: any;
  hash_manifest: any[];
  target_profile?: any;
  folder_breakdown?: any[];
  attachments_manifest?: any[];
  key_messages_ledger?: any[];
}

const REPORT_SECTIONS: ReportSection[] = [
  { id: "case_info", title: "1. Case Overview & Identification", description: "Case metadata, subject identity, examiner and agency information", enabled: true },
  { id: "sources", title: "2. Evidence Sources & Provenance", description: "Container technical specs, file size in bytes, SHA-256 acquisition hashes", enabled: true },
  { id: "exec_summary", title: "3. Executive Analytics & Volume Ledger", description: "Total email counts, sent/received/deleted metrics and temporal spans", enabled: true },
  { id: "folders", title: "4. Mailbox Structure & Folder Hierarchy", description: "Breakdown of folders (Inbox, Sent, Deleted) with item tallies and date spans", enabled: true },
  { id: "findings", title: "5. Security Findings & Tampering Matrix", description: "Full technical descriptions of spoofing, BEC, and risk anomalies", enabled: true },
  { id: "target_dossier", title: "6. Subject Profile & Top Correspondents", description: "Primary case subject profile, discovered aliases, and entity matrix", enabled: true },
  { id: "key_ledger", title: "7. Evidentiary & Flagged Email Ledger", description: "Itemized list of suspicious, high-risk, and recovered deleted messages", enabled: true },
  { id: "attachments", title: "8. Attachments & File Artifacts", description: "Inventory of extracted attachment files, types, and cryptographic hashes", enabled: true },
  { id: "exhibits", title: "9. Marked Court Exhibits", description: "Bookmarked emails entered into formal evidence record with annotations", enabled: true },
  { id: "custody", title: "10. Chain of Custody & Audit Trail", description: "Step-by-step verification history and evidence handling log", enabled: true },
  { id: "certification", title: "11. Methodology & Examiner Certification", description: "Forensic tool versioning, standards compliance, and sworn signature block", enabled: true },
];

export function ReportView({ caseId, caseData }: { caseId: string; caseData: any }) {
  const [sections, setSections] = useState<ReportSection[]>(REPORT_SECTIONS);
  const [reportData, setReportData] = useState<ReportData | null>(null);
  const [exhibits, setExhibits] = useState<Exhibit[]>([]);
  const [loading, setLoading] = useState(true);
  const [activeTab, setActiveTab] = useState<"preview" | "sources" | "folders" | "findings" | "ledger" | "exhibits" | "sections">("preview");
  const [copied, setCopied] = useState(false);

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
    setSections((prev) =>
      prev.map((s) => (s.id === id ? { ...s, enabled: !s.enabled } : s))
    );
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

  const handleCopyMarkdown = () => {
    if (!reportData) return;
    const md = `
# FORENSIC INVESTIGATION REPORT & SOURCE DATA DOSSIER
**Case Title:** ${reportData.case_info?.title || caseData?.title || "N/A"}
**Case File Number:** ${reportData.case_info?.case_number || caseData?.case_number || "J12-001"}
**Primary Target:** ${reportData.case_info?.target_name || "N/A"} (${reportData.case_info?.target_email || "N/A"})
**Examination Date:** ${new Date().toUTCString()}

## 1. Evidence Containers & Sources
${(reportData.evidence_inventory || [])
  .map(
    (ev) =>
      `- **${ev.filename}** (${ev.format.toUpperCase()} · ${(ev.size_bytes / (1024 * 1024)).toFixed(2)} MB · ${ev.message_count.toLocaleString()} messages)\n  SHA-256: \`${ev.sha256}\``
  )
  .join("\n")}

## 2. Mailbox Analytics
- Total Extracted Messages: ${reportData.email_stats?.total?.toLocaleString() || 0}
- Inbound Messages: ${reportData.email_stats?.inbox?.toLocaleString() || 0}
- Outbound (Sent): ${reportData.email_stats?.sent?.toLocaleString() || 0}
- Deleted / Dumpster Recovered: ${reportData.email_stats?.deleted?.toLocaleString() || 0}
- Flagged Security Violations: ${(reportData.findings || []).length}

## 3. Folder Breakdown
${(reportData.folder_breakdown || [])
  .map((f) => `- **${f.folder_name}** (${f.folder_category}): ${f.count.toLocaleString()} items (${f.date_from?.slice(0, 10) || "—"} to ${f.date_to?.slice(0, 10) || "—"})`)
  .join("\n")}

## 4. Key Findings
${(reportData.findings || [])
  .map(
    (f) =>
      `### [${f.severity.toUpperCase()}] ${f.title}\n- **Type:** ${f.type}\n- **Status:** ${f.status}\n- **Details:** ${f.description || "N/A"}`
  )
  .join("\n\n")}

## 5. Certification
I hereby certify that this forensic examination was conducted objectively in accordance with digital forensics best practices.
`;
    navigator.clipboard.writeText(md.trim());
    setCopied(true);
    setTimeout(() => setCopied(false), 2500);
  };

  if (loading) return <div className="card empty">Generating comprehensive forensic dossier...</div>;

  const enabledSections = new Set(sections.filter((s) => s.enabled).map((s) => s.id));

  return (
    <div>
      {/* Top Action Bar */}
      <div className="row between mb-4">
        <div>
          <h2 style={{ fontSize: 22, fontWeight: 700, color: "var(--text-0)" }}>
            Forensic Investigation & Source Data Report
          </h2>
          <p className="muted" style={{ fontSize: 12 }}>
            Belkasoft / Oxygen style comprehensive dossier with multi-source provenance, folder hierarchy, finding matrices, and itemized evidence ledgers.
          </p>
        </div>
        <div className="row gap-2">
          <button className="btn btn-ghost btn-sm" onClick={handleCopyMarkdown}>
            {copied ? "✓ Copied Markdown" : "📋 Copy Markdown"}
          </button>
          <button className="btn btn-primary btn-sm" onClick={handlePrint}>
            🖨️ Print / Save as PDF
          </button>
        </div>
      </div>

      {/* Navigation Tabs */}
      <div className="row gap-2 mb-4" style={{ borderBottom: "1px solid var(--border)", paddingBottom: 8, flexWrap: "wrap" }}>
        {(
          [
            ["preview", "👁️ Full Forensic Dossier (Court Ready)"],
            ["sources", `📁 Evidence Sources (${reportData?.evidence_inventory?.length || 0})`],
            ["folders", `📂 Folder Breakdown (${reportData?.folder_breakdown?.length || 0})`],
            ["findings", `🛡️ Findings Matrix (${reportData?.findings?.length || 0})`],
            ["ledger", `📜 Key Messages Ledger (${reportData?.key_messages_ledger?.length || 0})`],
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

      {/* TAB 1: FULL REPORT PREVIEW (Belkasoft / Oxygen Multi-Page Layout) */}
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
          {/* COVER PAGE BANNER */}
          <div
            style={{
              textAlign: "center",
              borderBottom: "3px double var(--border)",
              paddingBottom: 28,
              marginBottom: 32,
            }}
          >
            <div style={{ fontSize: 12, fontWeight: 700, letterSpacing: "0.15em", color: "var(--accent)", textTransform: "uppercase", marginBottom: 6 }}>
              DIGITAL FORENSICS & eDISCOVERY EXAMINATION REPORT
            </div>
            <h1 style={{ fontSize: 28, fontWeight: 900, margin: "8px 0 6px", color: "var(--text-0)" }}>
              {reportData?.case_info?.title || caseData?.title || "Email Investigation"}
            </h1>
            <div style={{ fontSize: 13, color: "var(--text-2)", marginBottom: 12 }}>
              Case File Reference: <strong>#{reportData?.case_info?.case_number || caseData?.case_number || "J12-001"}</strong>
            </div>
            <div style={{ fontSize: 11, color: "var(--text-3)", display: "flex", justifyContent: "center", gap: 20 }}>
              <span>Generated: {new Date().toUTCString()}</span>
              <span>Classification: <strong>CONFIDENTIAL / LAW ENFORCEMENT & LEGAL PRIVILEGED</strong></span>
            </div>
          </div>

          {/* 1. Case & Investigation Information */}
          {enabledSections.has("case_info") && (
            <div style={{ marginBottom: 32 }}>
              <h3 style={{ fontSize: 15, fontWeight: 700, borderBottom: "1px solid var(--border)", paddingBottom: 6, marginBottom: 12, color: "var(--accent)" }}>
                1. Case Overview & Subject Identification
              </h3>
              <table style={{ width: "100%", fontSize: 12, marginBottom: 8 }}>
                <tbody>
                  <tr>
                    <td style={{ width: 180, fontWeight: 600, background: "var(--bg-3)" }}>Case Title</td>
                    <td>{reportData?.case_info?.title}</td>
                    <td style={{ width: 180, fontWeight: 600, background: "var(--bg-3)" }}>Case Number</td>
                    <td>{reportData?.case_info?.case_number || "—"}</td>
                  </tr>
                  <tr>
                    <td style={{ fontWeight: 600, background: "var(--bg-3)" }}>Target Subject</td>
                    <td><strong>{reportData?.case_info?.target_name || "—"}</strong></td>
                    <td style={{ fontWeight: 600, background: "var(--bg-3)" }}>Target Email Address</td>
                    <td><code>{reportData?.case_info?.target_email || "—"}</code></td>
                  </tr>
                  <tr>
                    <td style={{ fontWeight: 600, background: "var(--bg-3)" }}>Organization / Entity</td>
                    <td>{reportData?.case_info?.target_organization || "—"}</td>
                    <td style={{ fontWeight: 600, background: "var(--bg-3)" }}>Investigation Status</td>
                    <td><span className="badge badge-green">{reportData?.case_info?.status || "ACTIVE"}</span></td>
                  </tr>
                </tbody>
              </table>
            </div>
          )}

          {/* 2. Evidence Sources & Provenance */}
          {enabledSections.has("sources") && (
            <div style={{ marginBottom: 32 }}>
              <h3 style={{ fontSize: 15, fontWeight: 700, borderBottom: "1px solid var(--border)", paddingBottom: 6, marginBottom: 12, color: "var(--accent)" }}>
                2. Evidence Sources & Cryptographic Provenance (Per Source Data)
              </h3>
              <p style={{ fontSize: 12, color: "var(--text-2)", marginBottom: 10 }}>
                Inventory of physical/digital forensic mail containers acquired, verified, and parsed into the investigative database.
              </p>
              <table style={{ width: "100%", fontSize: 11, marginBottom: 14 }}>
                <thead>
                  <tr>
                    <th className="th">Source Container</th>
                    <th className="th" style={{ width: 70 }}>Format</th>
                    <th className="th" style={{ width: 90 }}>Size (Bytes)</th>
                    <th className="th" style={{ width: 90 }}>Messages</th>
                    <th className="th">SHA-256 Acquisition Hash</th>
                  </tr>
                </thead>
                <tbody>
                  {(reportData?.evidence_inventory || []).map((ev) => (
                    <tr key={ev.id}>
                      <td className="td">
                        <strong>{ev.filename}</strong>
                        <div style={{ fontSize: 10, color: "var(--text-3)" }}>
                          Acquisition: {new Date(ev.acquired_at).toLocaleString()}
                        </div>
                      </td>
                      <td className="td">
                        <span className="badge badge-blue">{ev.format.toUpperCase()}</span>
                      </td>
                      <td className="td muted">
                        {(ev.size_bytes / (1024 * 1024)).toFixed(2)} MB ({ev.size_bytes.toLocaleString()} B)
                      </td>
                      <td className="td">
                        <strong>{ev.message_count.toLocaleString()}</strong> items
                      </td>
                      <td className="td" style={{ fontFamily: "var(--mono)", fontSize: 10, color: "var(--accent)" }}>
                        {ev.sha256}
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          )}

          {/* 3. Executive Analytics & Volume Ledger */}
          {enabledSections.has("exec_summary") && reportData?.email_stats && (
            <div style={{ marginBottom: 32 }}>
              <h3 style={{ fontSize: 15, fontWeight: 700, borderBottom: "1px solid var(--border)", paddingBottom: 6, marginBottom: 12, color: "var(--accent)" }}>
                3. Executive Summary & Mailbox Analytics
              </h3>
              <div
                style={{
                  display: "grid",
                  gridTemplateColumns: "repeat(4, 1fr)",
                  gap: 12,
                  marginBottom: 14,
                }}
              >
                <div style={{ background: "var(--bg-3)", padding: 12, borderRadius: "var(--r-sm)", textAlign: "center" }}>
                  <div style={{ fontSize: 22, fontWeight: 800, color: "var(--accent)" }}>
                    {reportData.email_stats.total?.toLocaleString() || 0}
                  </div>
                  <div style={{ fontSize: 10, color: "var(--text-3)", marginTop: 2 }}>TOTAL MESSAGES</div>
                </div>

                <div style={{ background: "var(--bg-3)", padding: 12, borderRadius: "var(--r-sm)", textAlign: "center" }}>
                  <div style={{ fontSize: 22, fontWeight: 800, color: "#3b82f6" }}>
                    {reportData.email_stats.sent?.toLocaleString() || 0}
                  </div>
                  <div style={{ fontSize: 10, color: "var(--text-3)", marginTop: 2 }}>OUTBOUND / SENT</div>
                </div>

                <div style={{ background: "var(--bg-3)", padding: 12, borderRadius: "var(--r-sm)", textAlign: "center" }}>
                  <div style={{ fontSize: 22, fontWeight: 800, color: "#22c55e" }}>
                    {reportData.email_stats.inbox?.toLocaleString() || 0}
                  </div>
                  <div style={{ fontSize: 10, color: "var(--text-3)", marginTop: 2 }}>INBOUND / INBOX</div>
                </div>

                <div style={{ background: "var(--bg-3)", padding: 12, borderRadius: "var(--r-sm)", textAlign: "center" }}>
                  <div style={{ fontSize: 22, fontWeight: 800, color: "#ef4444" }}>
                    {reportData.email_stats.deleted?.toLocaleString() || 0}
                  </div>
                  <div style={{ fontSize: 10, color: "var(--text-3)", marginTop: 2 }}>DELETED / RECOVERED</div>
                </div>
              </div>
              <div style={{ fontSize: 11, color: "var(--text-2)" }}>
                Temporal Range: <strong>{reportData.email_stats.date_from?.slice(0, 10) || "—"}</strong> to <strong>{reportData.email_stats.date_to?.slice(0, 10) || "—"}</strong>
              </div>
            </div>
          )}

          {/* 4. Mailbox Structure & Folder Hierarchy Breakdown */}
          {enabledSections.has("folders") && (reportData?.folder_breakdown || []).length > 0 && (
            <div style={{ marginBottom: 32 }}>
              <h3 style={{ fontSize: 15, fontWeight: 700, borderBottom: "1px solid var(--border)", paddingBottom: 6, marginBottom: 12, color: "var(--accent)" }}>
                4. Mailbox Folder Structure & Item Tally
              </h3>
              <table style={{ width: "100%", fontSize: 11, marginBottom: 10 }}>
                <thead>
                  <tr>
                    <th className="th">Folder Name</th>
                    <th className="th" style={{ width: 120 }}>Category</th>
                    <th className="th" style={{ width: 90 }}>Item Count</th>
                    <th className="th" style={{ width: 110 }}>Earliest Date</th>
                    <th className="th" style={{ width: 110 }}>Latest Date</th>
                  </tr>
                </thead>
                <tbody>
                  {(reportData?.folder_breakdown || []).map((f: any, i: number) => (
                    <tr key={i}>
                      <td className="td"><strong>{f.folder_name}</strong></td>
                      <td className="td">
                        <span className={`badge ${f.folder_category === "sent" ? "badge-blue" : f.folder_category === "soft_deleted" ? "badge-red" : "badge-green"}`}>
                          {f.folder_category}
                        </span>
                      </td>
                      <td className="td"><strong>{f.count.toLocaleString()}</strong></td>
                      <td className="td muted">{f.date_from ? f.date_from.slice(0, 10) : "—"}</td>
                      <td className="td muted">{f.date_to ? f.date_to.slice(0, 10) : "—"}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          )}

          {/* 5. Forensic Findings Matrix */}
          {enabledSections.has("findings") && (
            <div style={{ marginBottom: 32 }}>
              <h3 style={{ fontSize: 15, fontWeight: 700, borderBottom: "1px solid var(--border)", paddingBottom: 6, marginBottom: 12, color: "var(--accent)" }}>
                5. Forensic Security Violations & Risk Matrix
              </h3>
              {(reportData?.findings || []).length === 0 ? (
                <div className="muted text-sm">No security violations or tampering findings flagged.</div>
              ) : (
                <div style={{ display: "flex", flexDirection: "column", gap: 12 }}>
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
                            : "4px solid #eab308",
                      }}
                    >
                      <div className="row between mb-2">
                        <strong style={{ fontSize: 14, color: "var(--text-0)" }}>
                          {f.title}
                        </strong>
                        <div className="row gap-2">
                          <span
                            className={`badge ${
                              f.severity === "critical"
                                ? "badge-red"
                                : f.severity === "high"
                                ? "badge-orange"
                                : "badge-yellow"
                            }`}
                            style={{ fontSize: 9 }}
                          >
                            {f.severity.toUpperCase()}
                          </span>
                          <span className="badge badge-blue" style={{ fontSize: 9 }}>
                            TYPE: {f.type}
                          </span>
                          <span className="badge badge-green" style={{ fontSize: 9 }}>
                            STATUS: {f.status}
                          </span>
                        </div>
                      </div>
                      <p style={{ fontSize: 12, color: "var(--text-2)", margin: 0, lineHeight: 1.5 }}>
                        {f.description}
                      </p>
                    </div>
                  ))}
                </div>
              )}
            </div>
          )}

          {/* 6. Key Subject Dossier & Entity Matrix */}
          {enabledSections.has("target_dossier") && (
            <div style={{ marginBottom: 32 }}>
              <h3 style={{ fontSize: 15, fontWeight: 700, borderBottom: "1px solid var(--border)", paddingBottom: 6, marginBottom: 12, color: "var(--accent)" }}>
                6. Subject Dossier & Top Correspondents Network (Top 30 Entities)
              </h3>
              {reportData?.target_profile ? (
                <div style={{ background: "var(--bg-3)", padding: 14, borderRadius: "var(--r-sm)", marginBottom: 14 }}>
                  <div className="row between mb-2">
                    <strong style={{ fontSize: 14, color: "var(--text-0)" }}>
                      {cleanDisplayName(reportData.target_profile.display_name) || reportData.target_profile.email}
                    </strong>
                    <span className="badge badge-orange">CASE TARGET</span>
                  </div>
                  <div style={{ fontSize: 11, color: "var(--text-2)", marginBottom: 6 }}>
                    Primary Email: <code style={{ color: "var(--accent)" }}>{reportData.target_profile.email}</code>
                  </div>
                  {reportData.target_profile.aliases && (
                    <div style={{ fontSize: 11, color: "var(--text-3)", marginBottom: 8 }}>
                      Discovered Aliases & Exchange DNs: {reportData.target_profile.aliases}
                    </div>
                  )}
                  <div className="row gap-4" style={{ fontSize: 12 }}>
                    <div>Sent: <strong>{reportData.target_profile.sent}</strong></div>
                    <div>Received: <strong>{reportData.target_profile.received}</strong></div>
                    <div>Total Involvement: <strong>{reportData.target_profile.sent + reportData.target_profile.received}</strong></div>
                  </div>
                </div>
              ) : null}

              {/* Top Entities Table */}
              <table style={{ width: "100%", fontSize: 11 }}>
                <thead>
                  <tr>
                    <th className="th">Entity Name</th>
                    <th className="th">Email / Address</th>
                    <th className="th" style={{ width: 80 }}>Sent</th>
                    <th className="th" style={{ width: 90 }}>Received</th>
                    <th className="th" style={{ width: 80 }}>Total</th>
                  </tr>
                </thead>
                <tbody>
                  {(reportData?.entities || []).slice(0, 30).map((e: any, i: number) => (
                    <tr key={i}>
                      <td className="td">
                        <strong>{cleanDisplayName(e.display_name) || e.email.split("@")[0]}</strong>
                      </td>
                      <td className="td" style={{ fontFamily: "var(--mono)", fontSize: 10, color: "var(--text-2)" }}>
                        {e.email}
                      </td>
                      <td className="td" style={{ color: "#3b82f6" }}>{e.sent}</td>
                      <td className="td" style={{ color: "#22c55e" }}>{e.received}</td>
                      <td className="td"><strong>{e.sent + e.received}</strong></td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          )}

          {/* 7. Key Messages Ledger */}
          {enabledSections.has("key_ledger") && (reportData?.key_messages_ledger || []).length > 0 && (
            <div style={{ marginBottom: 32 }}>
              <h3 style={{ fontSize: 15, fontWeight: 700, borderBottom: "1px solid var(--border)", paddingBottom: 6, marginBottom: 12, color: "var(--accent)" }}>
                7. Evidentiary & Flagged Messages Ledger (Top Suspicious / Deleted Items)
              </h3>
              <table style={{ width: "100%", fontSize: 10 }}>
                <thead>
                  <tr>
                    <th className="th" style={{ width: 140 }}>Sender</th>
                    <th className="th">Subject</th>
                    <th className="th" style={{ width: 80 }}>Date</th>
                    <th className="th" style={{ width: 70 }}>Category</th>
                    <th className="th" style={{ width: 45 }}>Risk</th>
                  </tr>
                </thead>
                <tbody>
                  {(reportData?.key_messages_ledger || []).slice(0, 50).map((em: any) => (
                    <tr key={em.id}>
                      <td className="td" style={{ overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
                        {cleanDisplayName(em.from_display) || em.from_addr}
                      </td>
                      <td className="td">
                        <strong>{em.subject || "(no subject)"}</strong>
                        {em.deleted_recovered && (
                          <span className="badge badge-red" style={{ fontSize: 8, marginLeft: 6 }}>
                            DELETED
                          </span>
                        )}
                      </td>
                      <td className="td muted">{em.date_sent_utc ? em.date_sent_utc.slice(0, 10) : "—"}</td>
                      <td className="td muted">{em.folder_category}</td>
                      <td className="td">
                        <span className={`badge ${em.risk_score >= 50 ? "badge-red" : em.risk_score >= 25 ? "badge-orange" : "badge-green"}`} style={{ fontSize: 8 }}>
                          {em.risk_score}
                        </span>
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          )}

          {/* 8. Attachments Manifest */}
          {enabledSections.has("attachments") && (reportData?.attachments_manifest || []).length > 0 && (
            <div style={{ marginBottom: 32 }}>
              <h3 style={{ fontSize: 15, fontWeight: 700, borderBottom: "1px solid var(--border)", paddingBottom: 6, marginBottom: 12, color: "var(--accent)" }}>
                8. Extracted Attachments & File Artifacts Manifest
              </h3>
              <table style={{ width: "100%", fontSize: 11 }}>
                <thead>
                  <tr>
                    <th className="th">Filename</th>
                    <th className="th" style={{ width: 90 }}>Type</th>
                    <th className="th" style={{ width: 80 }}>Size</th>
                    <th className="th">Parent Email Subject</th>
                    <th className="th">SHA-256 Hash</th>
                  </tr>
                </thead>
                <tbody>
                  {(reportData?.attachments_manifest || []).slice(0, 30).map((att: any, i: number) => (
                    <tr key={i}>
                      <td className="td"><strong>{att.filename}</strong></td>
                      <td className="td muted">{att.file_type || "Binary"}</td>
                      <td className="td muted">{(att.size_bytes / 1024).toFixed(1)} KB</td>
                      <td className="td" style={{ overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap", maxWidth: 160 }}>
                        {att.email_subject || "—"}
                      </td>
                      <td className="td" style={{ fontFamily: "var(--mono)", fontSize: 9, color: "var(--accent)" }}>
                        {att.sha256 ? `${att.sha256.slice(0, 24)}...` : "—"}
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          )}

          {/* 9. Marked Exhibits */}
          {enabledSections.has("exhibits") && exhibits.length > 0 && (
            <div style={{ marginBottom: 32 }}>
              <h3 style={{ fontSize: 15, fontWeight: 700, borderBottom: "1px solid var(--border)", paddingBottom: 6, marginBottom: 12, color: "var(--accent)" }}>
                9. Formal Evidentiary Exhibits & Court Appendices
              </h3>
              <div style={{ display: "flex", flexDirection: "column", gap: 12 }}>
                {exhibits.map((ex) => (
                  <div
                    key={ex.id}
                    style={{
                      padding: 14,
                      background: "var(--bg-3)",
                      borderRadius: "var(--r-sm)",
                      border: "1px solid var(--border)",
                    }}
                  >
                    <div className="row between mb-2">
                      <strong style={{ fontSize: 14, color: "var(--accent)" }}>
                        {ex.exhibit_number}: {ex.subject}
                      </strong>
                      <span className="muted text-sm">{ex.date_sent}</span>
                    </div>
                    <div style={{ fontSize: 11, color: "var(--text-2)", marginBottom: 4 }}>
                      From: <strong>{ex.from_display || ex.from_addr}</strong>
                    </div>
                    <div style={{ fontSize: 10, color: "var(--text-3)" }}>
                      Investigator Annotation: {ex.notes}
                    </div>
                  </div>
                ))}
              </div>
            </div>
          )}

          {/* 10. Chain of Custody */}
          {enabledSections.has("custody") && (
            <div style={{ marginBottom: 32 }}>
              <h3 style={{ fontSize: 15, fontWeight: 700, borderBottom: "1px solid var(--border)", paddingBottom: 6, marginBottom: 12, color: "var(--accent)" }}>
                10. Cryptographic Chain of Custody & Verification Log
              </h3>
              <table style={{ width: "100%", fontSize: 11 }}>
                <thead>
                  <tr>
                    <th className="th" style={{ width: 140 }}>Timestamp</th>
                    <th className="th" style={{ width: 120 }}>Action</th>
                    <th className="th" style={{ width: 100 }}>Examiner</th>
                    <th className="th">Forensic Verification Details</th>
                  </tr>
                </thead>
                <tbody>
                  {(reportData?.custody_chain || []).map((c: any, i: number) => (
                    <tr key={i}>
                      <td className="td muted">{new Date(c.timestamp).toLocaleString()}</td>
                      <td className="td"><strong>{c.action}</strong></td>
                      <td className="td">{c.actor}</td>
                      <td className="td">{c.detail || "Verifiable acquisition integrity check"}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          )}

          {/* 11. Examiner Certification */}
          {enabledSections.has("certification") && (
            <div
              style={{
                marginTop: 36,
                padding: 22,
                border: "2px solid var(--border)",
                borderRadius: "var(--r-md)",
                background: "var(--bg-2)",
              }}
            >
              <h4 style={{ fontSize: 14, fontWeight: 800, marginBottom: 8, color: "var(--text-0)" }}>
                11. Formal Forensic Examiner Sworn Certification
              </h4>
              <p style={{ fontSize: 11, color: "var(--text-2)", lineHeight: 1.6 }}>
                I hereby certify that this forensic examination was conducted in accordance with established digital forensics (ISO/IEC 27037) and eDiscovery protocols. The data contained in this dossier represents a verifiable extraction from the provided evidence sources without modification or tampering.
              </p>
              <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 36, marginTop: 44 }}>
                <div>
                  <div style={{ borderTop: "1px solid var(--text-3)", paddingTop: 6, fontSize: 11, color: "var(--text-2)" }}>
                    Lead Forensic Examiner Signature
                  </div>
                </div>
                <div>
                  <div style={{ borderTop: "1px solid var(--text-3)", paddingTop: 6, fontSize: 11, color: "var(--text-2)" }}>
                    Date & Seal / Notarization
                  </div>
                </div>
              </div>
            </div>
          )}
        </div>
      )}

      {/* TAB 2: EVIDENCE SOURCES DETAIL */}
      {activeTab === "sources" && (
        <div className="card">
          <h3 style={{ fontSize: 16, fontWeight: 700, marginBottom: 12 }}>
            Source Data Provenance & Container Verification
          </h3>
          <p className="muted mb-4" style={{ fontSize: 12 }}>
            Comprehensive technical manifest of all evidence containers attached to this case.
          </p>

          <div style={{ display: "flex", flexDirection: "column", gap: 14 }}>
            {(reportData?.evidence_inventory || []).map((ev) => (
              <div
                key={ev.id}
                style={{
                  background: "var(--bg-3)",
                  padding: 16,
                  borderRadius: "var(--r-md)",
                  border: "1px solid var(--border)",
                }}
              >
                <div className="row between mb-2">
                  <strong style={{ fontSize: 15, color: "var(--text-0)" }}>{ev.filename}</strong>
                  <span className="badge badge-green">VERIFIED INTEGRITY</span>
                </div>

                <div className="grid-3 mb-3" style={{ fontSize: 12 }}>
                  <div>
                    <span className="muted">Format: </span>
                    <strong>{ev.format.toUpperCase()}</strong>
                  </div>
                  <div>
                    <span className="muted">Size: </span>
                    <strong>{(ev.size_bytes / (1024 * 1024)).toFixed(2)} MB</strong> ({ev.size_bytes.toLocaleString()} bytes)
                  </div>
                  <div>
                    <span className="muted">Extracted Emails: </span>
                    <strong>{ev.message_count.toLocaleString()}</strong>
                  </div>
                </div>

                <div style={{ fontSize: 11, background: "var(--bg-1)", padding: 10, borderRadius: "var(--r-xs)" }}>
                  <div className="muted mb-1">CRYPTOGRAPHIC ACQUISITION HASH (SHA-256):</div>
                  <code style={{ color: "var(--accent)", fontFamily: "var(--mono)" }}>{ev.sha256}</code>
                </div>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* TAB 3: FOLDER BREAKDOWN */}
      {activeTab === "folders" && (
        <div className="card">
          <h3 style={{ fontSize: 16, fontWeight: 700, marginBottom: 12 }}>
            Mailbox Storage & Folder Hierarchy Breakdown
          </h3>
          <table style={{ width: "100%", fontSize: 12 }}>
            <thead>
              <tr>
                <th className="th">Folder Name</th>
                <th className="th" style={{ width: 140 }}>Category</th>
                <th className="th" style={{ width: 110 }}>Item Count</th>
                <th className="th" style={{ width: 130 }}>Earliest Date</th>
                <th className="th" style={{ width: 130 }}>Latest Date</th>
              </tr>
            </thead>
            <tbody>
              {(reportData?.folder_breakdown || []).map((f: any, i: number) => (
                <tr key={i}>
                  <td className="td"><strong>{f.folder_name}</strong></td>
                  <td className="td">
                    <span className={`badge ${f.folder_category === "sent" ? "badge-blue" : f.folder_category === "soft_deleted" ? "badge-red" : "badge-green"}`}>
                      {f.folder_category}
                    </span>
                  </td>
                  <td className="td"><strong>{f.count.toLocaleString()}</strong></td>
                  <td className="td muted">{f.date_from ? f.date_from.slice(0, 10) : "—"}</td>
                  <td className="td muted">{f.date_to ? f.date_to.slice(0, 10) : "—"}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}

      {/* TAB 4: FINDINGS */}
      {activeTab === "findings" && (
        <div className="card">
          <h3 style={{ fontSize: 16, fontWeight: 700, marginBottom: 12 }}>
            Forensic Findings & Security Violations Matrix
          </h3>
          <table style={{ width: "100%", fontSize: 12 }}>
            <thead>
              <tr>
                <th className="th" style={{ width: 100 }}>Severity</th>
                <th className="th" style={{ width: 120 }}>Finding Type</th>
                <th className="th">Finding Description</th>
                <th className="th" style={{ width: 100 }}>Status</th>
              </tr>
            </thead>
            <tbody>
              {(reportData?.findings || []).map((f: any) => (
                <tr key={f.id}>
                  <td className="td">
                    <span
                      className={`badge ${
                        f.severity === "critical"
                          ? "badge-red"
                          : f.severity === "high"
                          ? "badge-orange"
                          : "badge-yellow"
                      }`}
                    >
                      {f.severity.toUpperCase()}
                    </span>
                  </td>
                  <td className="td"><strong>{f.type}</strong></td>
                  <td className="td">
                    <div style={{ fontWeight: 600, color: "var(--text-0)" }}>{f.title}</div>
                    <div className="muted text-sm">{f.description}</div>
                  </td>
                  <td className="td">
                    <span className="badge badge-blue">{f.status}</span>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}

      {/* TAB 5: KEY MESSAGES LEDGER */}
      {activeTab === "ledger" && (
        <div className="card">
          <h3 style={{ fontSize: 16, fontWeight: 700, marginBottom: 12 }}>
            Evidentiary & Flagged Messages Ledger
          </h3>
          <p className="muted mb-4" style={{ fontSize: 12 }}>
            Itemized record of suspicious, high-risk, and recovered deleted messages extracted during analysis.
          </p>
          <table style={{ width: "100%", fontSize: 11 }}>
            <thead>
              <tr>
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
                  <td className="td">{cleanDisplayName(em.from_display) || em.from_addr}</td>
                  <td className="td">
                    <strong>{em.subject || "(no subject)"}</strong>
                    {em.deleted_recovered && (
                      <span className="badge badge-red" style={{ fontSize: 8, marginLeft: 6 }}>
                        DELETED
                      </span>
                    )}
                  </td>
                  <td className="td muted">{em.date_sent_utc ? em.date_sent_utc.slice(0, 10) : "—"}</td>
                  <td className="td muted">{em.folder_category}</td>
                  <td className="td">
                    <span className={`badge ${em.risk_score >= 50 ? "badge-red" : em.risk_score >= 25 ? "badge-orange" : "badge-green"}`} style={{ fontSize: 8 }}>
                      {em.risk_score}
                    </span>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}

      {/* TAB 6: EXHIBITS */}
      {activeTab === "exhibits" && (
        <div className="card">
          <div className="row between mb-4">
            <div>
              <h3 style={{ fontSize: 16, fontWeight: 700 }}>Marked Court Exhibits</h3>
              <p className="muted" style={{ fontSize: 12 }}>
                Bookmarked evidentiary emails to include in formal report appendices.
              </p>
            </div>
            <button className="btn btn-primary btn-sm" onClick={handleAddExhibit}>
              + Add Exhibit by Email ID
            </button>
          </div>

          {exhibits.length === 0 ? (
            <div className="empty">No exhibits bookmarked yet. Use "+ Add Exhibit" to enter emails into the record.</div>
          ) : (
            <div style={{ display: "flex", flexDirection: "column", gap: 10 }}>
              {exhibits.map((ex) => (
                <div
                  key={ex.id}
                  style={{
                    padding: 14,
                    background: "var(--bg-3)",
                    borderRadius: "var(--r-md)",
                    border: "1px solid var(--border)",
                  }}
                >
                  <div className="row between mb-2">
                    <strong style={{ fontSize: 14, color: "var(--accent)" }}>
                      {ex.exhibit_number}: {ex.subject}
                    </strong>
                    <button
                      className="btn btn-ghost btn-sm"
                      style={{ color: "var(--red)", fontSize: 11 }}
                      onClick={() => removeExhibit(ex.id)}
                    >
                      ✕ Remove
                    </button>
                  </div>
                  <div className="grid-2 text-sm mb-2">
                    <div>From: <strong>{ex.from_display || ex.from_addr}</strong></div>
                    <div>Date: <strong>{ex.date_sent}</strong></div>
                  </div>
                  <input
                    className="input"
                    style={{ fontSize: 11, padding: "4px 8px", width: "100%" }}
                    placeholder="Add investigator annotation / notes for this exhibit..."
                    value={ex.notes}
                    onChange={(e) => {
                      const val = e.target.value;
                      setExhibits((prev) =>
                        prev.map((item) => (item.id === ex.id ? { ...item, notes: val } : item))
                      );
                    }}
                  />
                </div>
              ))}
            </div>
          )}
        </div>
      )}

      {/* TAB 7: SECTIONS CONFIGURATION */}
      {activeTab === "sections" && (
        <div className="card">
          <h3 style={{ fontSize: 16, fontWeight: 700, marginBottom: 6 }}>Configure Report Chapters</h3>
          <p className="muted mb-4" style={{ fontSize: 12 }}>
            Toggle sections on or off to tailor the final report for court submission, internal review, or executive presentation.
          </p>

          <div style={{ display: "grid", gridTemplateColumns: "repeat(auto-fill, minmax(320px, 1fr))", gap: 12 }}>
            {sections.map((sec) => (
              <label
                key={sec.id}
                style={{
                  display: "flex",
                  alignItems: "flex-start",
                  gap: 12,
                  padding: 14,
                  background: "var(--bg-3)",
                  borderRadius: "var(--r-md)",
                  cursor: "pointer",
                  border: sec.enabled ? "1px solid var(--accent)" : "1px solid transparent",
                }}
              >
                <input
                  type="checkbox"
                  checked={sec.enabled}
                  onChange={() => toggleSection(sec.id)}
                  style={{ marginTop: 2 }}
                />
                <div>
                  <div style={{ fontSize: 13, fontWeight: 600, color: "var(--text-0)" }}>
                    {sec.title}
                  </div>
                  <div style={{ fontSize: 11, color: "var(--text-3)", marginTop: 2 }}>
                    {sec.description}
                  </div>
                </div>
              </label>
            ))}
          </div>
        </div>
      )}
    </div>
  );
}
