import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  ReportSection,
  Exhibit,
  ReportData,
  REPORT_SECTIONS,
} from "./report/types";
import { ReportDossierPreview } from "./report/ReportDossierPreview";
import { ReportTabsDetail } from "./report/ReportTabsDetail";

export function ReportView({ caseId, caseData, evidenceFilter }: { caseId: string; caseData: any; evidenceFilter?: string | null }) {
  const [sections, setSections] = useState<ReportSection[]>(REPORT_SECTIONS);
  const [reportData, setReportData] = useState<ReportData | null>(null);
  const [exhibits, setExhibits] = useState<Exhibit[]>([]);
  const [loading, setLoading] = useState(true);
  const [activeTab, setActiveTab] = useState<"preview" | "sources" | "folders" | "findings" | "ledger" | "exhibits" | "sections">("preview");
  const [copied, setCopied] = useState(false);
  const [toastMessage, setToastMessage] = useState<string | null>(null);

  const showToast = (msg: string) => {
    setToastMessage(msg);
    setTimeout(() => setToastMessage(null), 3500);
  };

  const loadReportData = useCallback(async () => {
    setLoading(true);
    try {
      const data = await invoke<ReportData>("generate_report_data", { input: { case_id: caseId, evidence_id: evidenceFilter || undefined } });
      setReportData(data);
    } catch (e) {
      console.error("Failed to load report data:", e);
    } finally {
      setLoading(false);
    }
  }, [caseId, evidenceFilter]);

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

  const handleExportHTML = async () => {
    try {
      const savedPath = await invoke<string>("export_report_pdf", {
        caseId,
        sections: sections.filter(s => s.enabled).map(s => s.id),
        exhibits
      });
      showToast(`📥 Exported standalone dossier to Downloads: ${savedPath}`);
    } catch (e: any) {
      console.error(e);
      showToast(`❌ Error exporting report: ${e}`);
    }
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

      {/* Top Action Bar */}
      <div className="row between mb-4">
        <div>
          <h2 style={{ fontSize: 22, fontWeight: 700, color: "var(--text-0)" }}>
            Forensic Investigation &amp; Source Data Report
          </h2>
          <p className="muted" style={{ fontSize: 12 }}>
            Belkasoft / Oxygen style comprehensive dossier with multi-source provenance, folder hierarchy, finding matrices, and itemized evidence ledgers.
          </p>
        </div>
        <div className="row gap-2">
          <button className="btn btn-ghost btn-sm" onClick={handleCopyMarkdown}>
            {copied ? "✓ Copied Markdown" : "📋 Copy Markdown"}
          </button>
          <button className="btn btn-ghost btn-sm" onClick={handleExportHTML} title="Export standalone self-contained HTML report to Downloads">
            📥 Export HTML Dossier
          </button>
          <button className="btn btn-primary btn-sm" onClick={handlePrint} title="Print or save as high-quality PDF">
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

      {activeTab === "preview" ? (
        <ReportDossierPreview
          reportData={reportData}
          caseData={caseData}
          enabledSections={enabledSections}
          exhibits={exhibits}
        />
      ) : (
        <ReportTabsDetail
          activeTab={activeTab}
          reportData={reportData}
          exhibits={exhibits}
          sections={sections}
          onAddExhibit={handleAddExhibit}
          onRemoveExhibit={removeExhibit}
          onUpdateExhibitNotes={(id, notes) => {
            setExhibits((prev) =>
              prev.map((item) => (item.id === id ? { ...item, notes } : item))
            );
          }}
          onToggleSection={toggleSection}
        />
      )}
    </div>
  );
}
