import { useState, useEffect, useCallback, useMemo } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  Finding,
  FindingEmailItem,
  FindingsProps,
} from "./findings/types";
import { FindingsSeverityCards } from "./findings/FindingsSeverityCards";
import { FindingsTable } from "./findings/FindingsTable";
import { FindingDetailPanel } from "./findings/FindingDetailPanel";

export function FindingsView({ caseId, evidenceFilter }: FindingsProps) {
  const [findings, setFindings] = useState<Finding[]>([]);
  const [loading, setLoading] = useState(true);
  const [analyzing, setAnalyzing] = useState(false);
  const [searchTerm, setSearchTerm] = useState("");
  const [filterSeverity, setFilterSeverity] = useState<string>("all");
  const [filterType, setFilterType] = useState<string>("all");
  const [filterStatus, setFilterStatus] = useState<string>("all");
  const [selectedFinding, setSelectedFinding] = useState<Finding | null>(null);
  const [toastMessage, setToastMessage] = useState<string | null>(null);
  
  const [relatedEmails, setRelatedEmails] = useState<FindingEmailItem[]>([]);
  const [loadingEmails, setLoadingEmails] = useState(false);
  const [inspectingEmail, setInspectingEmail] = useState<FindingEmailItem | null>(null);

  const [noteText, setNoteText] = useState("");
  const [authorName, setAuthorName] = useState("Investigator");
  const [savingNote, setSavingNote] = useState(false);

  const showToast = (msg: string) => {
    setToastMessage(msg);
    setTimeout(() => setToastMessage(null), 3500);
  };

  const loadFindings = useCallback(async () => {
    setLoading(true);
    try {
      const data = await invoke<Finding[]>("findings_list", { input: { case_id: caseId, evidence_id: evidenceFilter || undefined } });
      setFindings(data);
      if (selectedFinding) {
        const updated = data.find(f => f.id === selectedFinding.id);
        if (updated) setSelectedFinding(updated);
      }
    } catch (e) { 
      console.error(e); 
    } finally { 
      setLoading(false); 
    }
  }, [caseId, evidenceFilter, selectedFinding?.id]);

  useEffect(() => { loadFindings(); }, [loadFindings]);

  useEffect(() => {
    if (!selectedFinding) {
      setRelatedEmails([]);
      setInspectingEmail(null);
      return;
    }

    setLoadingEmails(true);
    invoke<FindingEmailItem[]>("finding_emails", { input: { finding_id: selectedFinding.id } })
      .then(emails => {
        setRelatedEmails(emails);
        if (emails.length > 0) {
          setInspectingEmail(emails[0]);
        } else {
          setInspectingEmail(null);
        }
      })
      .catch(err => {
        console.error("Failed to load finding emails:", err);
        setRelatedEmails([]);
      })
      .finally(() => setLoadingEmails(false));
  }, [selectedFinding?.id]);

  const runAnalysis = async () => {
    setAnalyzing(true);
    try {
      const count = await invoke<number>("run_analysis", { input: { case_id: caseId } });
      showToast(`⚡ Forensic analysis complete: ${count} threat findings indexed!`);
      await loadFindings();
    } catch (e: any) {
      console.error("Analysis failed:", e);
      showToast(`Analysis failed: ${e}`);
    } finally {
      setAnalyzing(false);
    }
  };

  const updateStatus = async (id: string, newStatus: string) => {
    try {
      await invoke("update_finding_status", { 
        input: { 
          finding_id: id, 
          new_status: newStatus, 
          reviewed_by: authorName 
        } 
      });
      showToast(`Finding marked as ${newStatus.toUpperCase()}`);
      loadFindings();
    } catch (e: any) {
      console.error("Update failed:", e);
    }
  };

  const addNote = async (e?: React.FormEvent) => {
    if (e) e.preventDefault();
    if (!selectedFinding || !noteText.trim()) return;
    setSavingNote(true);
    try {
      await invoke("add_finding_note", {
        input: {
          finding_id: selectedFinding.id,
          note: noteText.trim(),
          author: authorName.trim() || "Investigator",
        }
      });
      setNoteText("");
      showToast("Investigator note recorded in chain of custody");
      loadFindings();
    } catch (e: any) {
      console.error("Note failed:", e);
    } finally {
      setSavingNote(false);
    }
  };

  const filtered = useMemo(() => {
    return findings.filter(f => {
      if (filterSeverity !== "all" && f.severity !== filterSeverity) return false;
      if (filterType !== "all" && f.type_ !== filterType) return false;
      if (filterStatus !== "all" && f.status !== filterStatus) return false;
      if (searchTerm.trim()) {
        const q = searchTerm.toLowerCase();
        const matchTitle = f.title.toLowerCase().includes(q);
        const matchDesc = (f.description || "").toLowerCase().includes(q);
        const matchType = f.type_.toLowerCase().includes(q);
        if (!matchTitle && !matchDesc && !matchType) return false;
      }
      return true;
    });
  }, [findings, filterSeverity, filterType, filterStatus, searchTerm]);

  const severityCounts = {
    critical: findings.filter(f => f.severity === "critical").length,
    high: findings.filter(f => f.severity === "high").length,
    medium: findings.filter(f => f.severity === "medium").length,
    low: findings.filter(f => f.severity === "low").length,
  };

  const typeCounts = {
    BEC: findings.filter(f => f.type_ === "BEC").length,
    SPOOFING: findings.filter(f => f.type_ === "SPOOFING").length,
    PHISHING: findings.filter(f => f.type_ === "PHISHING").length,
    EXFILTRATION: findings.filter(f => f.type_ === "EXFILTRATION").length,
    ATTACHMENT: findings.filter(f => f.type_ === "ATTACHMENT").length,
    ROUTING: findings.filter(f => f.type_ === "ROUTING").length,
    ANOMALY: findings.filter(f => f.type_ === "ANOMALY").length,
  };

  const exportFindingsCSV = () => {
    const headers = ["Finding ID", "Severity", "Category", "Title", "Description", "Status", "Messages Count", "Reviewed By", "Created At"];
    const rows = filtered.map(f => {
      let msgCount = 0;
      try { msgCount = JSON.parse(f.email_ids || "[]").length; } catch { msgCount = 0; }
      return [
        f.id,
        f.severity.toUpperCase(),
        f.type_,
        `"${f.title.replace(/"/g, '""')}"`,
        `"${(f.description || "").replace(/"/g, '""')}"`,
        f.status.toUpperCase(),
        msgCount,
        f.reviewed_by || "",
        f.created_at,
      ].join(",");
    });

    const csvContent = "data:text/csv;charset=utf-8," + [headers.join(","), ...rows].join("\n");
    const encodedUri = encodeURI(csvContent);
    const link = document.createElement("a");
    link.setAttribute("href", encodedUri);
    link.setAttribute("download", `findings_matrix_${caseId.slice(0,8)}.csv`);
    document.body.appendChild(link);
    link.click();
    document.body.removeChild(link);
    showToast("📁 Findings matrix exported as CSV");
  };

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
            padding: "12px 20px",
            fontWeight: 600,
            fontSize: 13,
            boxShadow: "0 10px 25px rgba(0,0,0,0.5)",
            display: "flex",
            alignItems: "center",
            gap: 10,
          }}
        >
          <span>✓</span>
          <span>{toastMessage}</span>
        </div>
      )}

      {/* Header */}
      <div className="row between mb-4">
        <div>
          <h2 style={{ fontSize: 22, fontWeight: 700, color: "var(--text-0)" }}>
            Forensic Findings &amp; Threat Matrix
          </h2>
          <p className="muted">
            Automated BEC, spoofing, phishing, and evidence integrity analysis with reviewer chain of custody
          </p>
        </div>
        <div className="row gap-2">
          {findings.length > 0 && (
            <button className="btn btn-ghost" onClick={exportFindingsCSV} title="Export CSV for court exhibit">
              📥 Export CSV
            </button>
          )}
          <button className="btn btn-primary" onClick={runAnalysis} disabled={analyzing}>
            {analyzing ? "Analyzing Evidence..." : "▶ Re-Run Deep Analysis"}
          </button>
        </div>
      </div>

      {/* Severity Breakdown & Category Pills */}
      <FindingsSeverityCards
        findingsLength={findings.length}
        severityCounts={severityCounts}
        typeCounts={typeCounts}
        filterSeverity={filterSeverity}
        setFilterSeverity={setFilterSeverity}
        filterType={filterType}
        setFilterType={setFilterType}
      />

      {/* Findings Table */}
      <FindingsTable
        caseId={caseId}
        findings={findings}
        filtered={filtered}
        selectedFinding={selectedFinding}
        searchTerm={searchTerm}
        setSearchTerm={setSearchTerm}
        filterSeverity={filterSeverity}
        setFilterSeverity={setFilterSeverity}
        filterStatus={filterStatus}
        setFilterStatus={setFilterStatus}
        loading={loading}
        analyzing={analyzing}
        onSelectFinding={setSelectedFinding}
        onUpdateStatus={updateStatus}
        onRunAnalysis={runAnalysis}
      />

      {/* Detail Inspector Panel */}
      {selectedFinding && (
        <FindingDetailPanel
          caseId={caseId}
          selectedFinding={selectedFinding}
          relatedEmails={relatedEmails}
          loadingEmails={loadingEmails}
          inspectingEmail={inspectingEmail}
          setInspectingEmail={setInspectingEmail}
          noteText={noteText}
          setNoteText={setNoteText}
          authorName={authorName}
          setAuthorName={setAuthorName}
          savingNote={savingNote}
          onUpdateStatus={updateStatus}
          onAddNote={addNote}
          onClose={() => setSelectedFinding(null)}
        />
      )}
    </div>
  );
}
