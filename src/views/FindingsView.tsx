import { useState, useEffect, useCallback, useMemo } from "react";
import { invoke } from "@tauri-apps/api/core";

interface Finding {
  id: string;
  case_id: string;
  type_: string;
  severity: string;
  confidence: string;
  title: string;
  description: string | null;
  evidence_refs: string;
  email_ids: string;
  status: string;
  created_at: string;
  reviewed_by: string | null;
  reviewed_at: string | null;
  notes: string | null;
}

interface EmailItem {
  id: string;
  evidence_id: string;
  from_addr: string;
  from_display: string | null;
  to_addrs: string;
  cc_addrs: string;
  subject: string | null;
  date_sent: string | null;
  body_text: string | null;
  body_html: string | null;
  headers_raw: string | null;
  folder_name: string | null;
  folder_category: string;
  risk_score: number;
}

interface Props {
  caseId: string;
  evidenceFilter?: string | null;
  onGoToEvidence?: () => void;
}

export function FindingsView({ caseId, evidenceFilter }: Props) {
  const [findings, setFindings] = useState<Finding[]>([]);
  const [loading, setLoading] = useState(true);
  const [analyzing, setAnalyzing] = useState(false);
  const [searchTerm, setSearchTerm] = useState("");
  const [filterSeverity, setFilterSeverity] = useState<string>("all");
  const [filterType, setFilterType] = useState<string>("all");
  const [filterStatus, setFilterStatus] = useState<string>("all");
  const [selectedFinding, setSelectedFinding] = useState<Finding | null>(null);
  const [toastMessage, setToastMessage] = useState<string | null>(null);
  
  // Related emails state
  const [relatedEmails, setRelatedEmails] = useState<EmailItem[]>([]);
  const [loadingEmails, setLoadingEmails] = useState(false);
  const [inspectingEmail, setInspectingEmail] = useState<EmailItem | null>(null);

  // Note composer state
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
      const data = await invoke<Finding[]>("findings_list", { 
        input: { 
          case_id: caseId,
          evidence_id: evidenceFilter || undefined
        } 
      });
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

  useEffect(() => { loadFindings(); }, [caseId, evidenceFilter, loadFindings]);

  // Load related emails when selectedFinding changes
  useEffect(() => {
    if (!selectedFinding) {
      setRelatedEmails([]);
      setInspectingEmail(null);
      return;
    }

    setLoadingEmails(true);
    invoke<EmailItem[]>("finding_emails", { input: { finding_id: selectedFinding.id } })
      .then(emails => {
        setRelatedEmails(emails);
        if (emails.length > 0) {
          setInspectingEmail(emails[0]); // Auto-inspect first related email
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

  // Filter findings
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

  // Severity breakdown
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

  const severityColor = (severity: string) => {
    switch (severity.toLowerCase()) {
      case "critical": return "var(--danger)";
      case "high": return "#f97316";
      case "medium": return "#eab308";
      case "low": return "#3b82f6";
      default: return "#6b7280";
    }
  };

  const statusBadge = (status: string) => {
    switch (status.toLowerCase()) {
      case "open": return "badge-blue";
      case "confirmed": return "badge-green";
      case "rejected": return "badge-red";
      case "reviewed": return "badge-yellow";
      default: return "badge-gray";
    }
  };

  // Export filtered findings as CSV
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

  // Parse notes list
  const parsedNotes = (selectedFinding?.notes || "").split("\n---\n").filter(Boolean);

  return (
    <div>
      {/* Toast Notification */}
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

      {/* Severity Summary Cards */}
      <div className="row gap-4 mb-4" style={{ flexWrap: "wrap" }}>
        <div 
          style={{ 
            flex: 1, 
            minWidth: 140, 
            padding: 16, 
            background: "var(--bg-3)", 
            borderRadius: "var(--r-md)", 
            textAlign: "center", 
            borderLeft: "4px solid var(--danger)",
            cursor: "pointer"
          }}
          onClick={() => setFilterSeverity(filterSeverity === "critical" ? "all" : "critical")}
        >
          <div style={{ fontSize: 26, fontWeight: 800, color: "var(--danger)" }}>{severityCounts.critical}</div>
          <div className="muted text-sm" style={{ fontWeight: 600 }}>Critical Severity</div>
        </div>

        <div 
          style={{ 
            flex: 1, 
            minWidth: 140, 
            padding: 16, 
            background: "var(--bg-3)", 
            borderRadius: "var(--r-md)", 
            textAlign: "center", 
            borderLeft: "4px solid #f97316",
            cursor: "pointer"
          }}
          onClick={() => setFilterSeverity(filterSeverity === "high" ? "all" : "high")}
        >
          <div style={{ fontSize: 26, fontWeight: 800, color: "#f97316" }}>{severityCounts.high}</div>
          <div className="muted text-sm" style={{ fontWeight: 600 }}>High Threats</div>
        </div>

        <div 
          style={{ 
            flex: 1, 
            minWidth: 140, 
            padding: 16, 
            background: "var(--bg-3)", 
            borderRadius: "var(--r-md)", 
            textAlign: "center", 
            borderLeft: "4px solid #eab308",
            cursor: "pointer"
          }}
          onClick={() => setFilterSeverity(filterSeverity === "medium" ? "all" : "medium")}
        >
          <div style={{ fontSize: 26, fontWeight: 800, color: "#eab308" }}>{severityCounts.medium}</div>
          <div className="muted text-sm" style={{ fontWeight: 600 }}>Medium Risks</div>
        </div>

        <div 
          style={{ 
            flex: 1, 
            minWidth: 140, 
            padding: 16, 
            background: "var(--bg-3)", 
            borderRadius: "var(--r-md)", 
            textAlign: "center", 
            borderLeft: "4px solid #3b82f6",
            cursor: "pointer"
          }}
          onClick={() => setFilterSeverity(filterSeverity === "low" ? "all" : "low")}
        >
          <div style={{ fontSize: 26, fontWeight: 800, color: "#3b82f6" }}>{severityCounts.low}</div>
          <div className="muted text-sm" style={{ fontWeight: 600 }}>Low / Info</div>
        </div>
      </div>

      {/* Category Breakdown Pills */}
      <div className="row gap-2 mb-4" style={{ flexWrap: "wrap" }}>
        <button
          className={`btn btn-sm ${filterType === "all" ? "btn-primary" : "btn-ghost"}`}
          onClick={() => setFilterType("all")}
        >
          All Categories ({findings.length})
        </button>
        {Object.entries(typeCounts).map(([type, count]) => (
          <button
            key={type}
            className={`btn btn-sm ${filterType === type ? "btn-primary" : "btn-ghost"}`}
            onClick={() => setFilterType(filterType === type ? "all" : type)}
            style={{ opacity: count === 0 ? 0.5 : 1 }}
          >
            {type}: {count}
          </button>
        ))}
      </div>

      {/* Filter & Search Toolbar */}
      <div className="card mb-4" style={{ padding: "12px 16px" }}>
        <div className="row between" style={{ flexWrap: "wrap", gap: 12 }}>
          <div style={{ flex: 1, minWidth: 260 }}>
            <input
              className="input"
              style={{ width: "100%", padding: "6px 12px", fontSize: 13 }}
              placeholder="Search findings by keyword, indicator, brand, domain..."
              value={searchTerm}
              onChange={e => setSearchTerm(e.target.value)}
            />
          </div>

          <div className="row gap-2" style={{ flexWrap: "wrap" }}>
            <span className="muted" style={{ fontSize: 12, alignSelf: "center" }}>Severity:</span>
            {["all", "critical", "high", "medium", "low"].map(s => (
              <button
                key={s}
                className={`btn btn-sm ${filterSeverity === s ? "btn-primary" : "btn-ghost"}`}
                style={{ fontSize: 11, padding: "4px 8px" }}
                onClick={() => setFilterSeverity(s)}
              >
                {s.toUpperCase()}
              </button>
            ))}
          </div>

          <div className="row gap-2" style={{ flexWrap: "wrap" }}>
            <span className="muted" style={{ fontSize: 12, alignSelf: "center" }}>Status:</span>
            {["all", "open", "confirmed", "rejected", "reviewed"].map(st => (
              <button
                key={st}
                className={`btn btn-sm ${filterStatus === st ? "btn-primary" : "btn-ghost"}`}
                style={{ fontSize: 11, padding: "4px 8px" }}
                onClick={() => setFilterStatus(st)}
              >
                {st.toUpperCase()}
              </button>
            ))}
          </div>
        </div>
      </div>

      {/* Findings Table */}
      {loading ? (
        <div className="empty">Loading forensic findings...</div>
      ) : filtered.length === 0 ? (
        <div className="card" style={{ textAlign: "center", padding: "50px 30px" }}>
          <div style={{ fontSize: 40, marginBottom: 12 }}>🛡️</div>
          <h3 style={{ fontSize: 18, color: "var(--text-0)", marginBottom: 6 }}>
            {findings.length === 0 ? "No Findings Generated Yet" : "No findings match your filter criteria"}
          </h3>
          <p className="muted mb-4">
            {findings.length === 0
              ? "Run automated deep analysis to scan email headers, wire fraud indicators, brand spoofing, and file attachments."
              : "Try resetting your search query or severity filters above."}
          </p>
          {findings.length === 0 && (
            <button className="btn btn-primary" onClick={runAnalysis} disabled={analyzing}>
              {analyzing ? "Analyzing..." : "▶ Run Analysis Now"}
            </button>
          )}
        </div>
      ) : (
        <div className="card" style={{ padding: 0, overflow: "hidden", marginBottom: 20 }}>
          <div className="row between" style={{ padding: "10px 16px", background: "var(--bg-3)", borderBottom: "1px solid var(--border)", fontSize: 12, fontWeight: 600, color: "var(--text-1)" }}>
            <div>
              {filtered.length} Forensic Finding{filtered.length === 1 ? "" : "s"} — Select a finding to inspect evidentiary emails &amp; record notes
            </div>
            <div className="muted text-sm">
              Showing {filtered.length} of {findings.length} total
            </div>
          </div>
          <div style={{ overflowX: "auto" }}>
            <table>
              <thead>
                <tr>
                  <th className="th" style={{ width: 95 }}>Severity</th>
                  <th className="th" style={{ width: 110 }}>Category</th>
                  <th className="th">Finding Description &amp; Indicator</th>
                  <th className="th" style={{ width: 105 }}>Status</th>
                  <th className="th" style={{ width: 75 }}>Evidentiary</th>
                  <th className="th" style={{ width: 150 }}>Review Action</th>
                </tr>
              </thead>
              <tbody>
                {filtered.map(f => (
                  <tr
                    key={f.id}
                    className="tr-click"
                    style={{
                      background: selectedFinding?.id === f.id ? "rgba(59,130,246,0.12)" : undefined,
                      borderLeft: selectedFinding?.id === f.id ? "4px solid var(--accent)" : "4px solid transparent",
                    }}
                    onClick={() => setSelectedFinding(f)}
                  >
                    <td>
                      <span className="badge" style={{ background: `${severityColor(f.severity)}22`, color: severityColor(f.severity), border: `1px solid ${severityColor(f.severity)}44`, fontWeight: 700 }}>
                        {f.severity.toUpperCase()}
                      </span>
                    </td>
                    <td><span className="badge badge-gray" style={{ fontWeight: 600 }}>{f.type_}</span></td>
                    <td style={{ maxWidth: 380 }}>
                      <div style={{ fontWeight: 600, color: "var(--text-0)", fontSize: 13 }}>{f.title}</div>
                      {f.description && (
                        <div className="muted text-sm" style={{ marginTop: 2, whiteSpace: "nowrap", overflow: "hidden", textOverflow: "ellipsis" }}>
                          {f.description}
                        </div>
                      )}
                    </td>
                    <td><span className={`badge ${statusBadge(f.status)}`}>{f.status.toUpperCase()}</span></td>
                    <td className="mono" style={{ fontSize: 12 }}>
                      {(() => {
                        try {
                          const ids = JSON.parse(f.email_ids || "[]");
                          return `${ids.length} msg`;
                        } catch { return "0"; }
                      })()}
                    </td>
                    <td>
                      <div className="row gap-2" onClick={e => e.stopPropagation()}>
                        {f.status === "open" && (
                          <>
                            <button className="btn btn-ghost btn-sm" style={{ color: "var(--success)", padding: "2px 6px" }} onClick={() => updateStatus(f.id, "confirmed")} title="Confirm finding">
                              ✓ Confirm
                            </button>
                            <button className="btn btn-ghost btn-sm" style={{ color: "var(--danger)", padding: "2px 6px" }} onClick={() => updateStatus(f.id, "rejected")} title="Reject (False Positive)">
                              ✗ Reject
                            </button>
                          </>
                        )}
                        {f.status === "confirmed" && (
                          <button className="btn btn-ghost btn-sm" style={{ color: "var(--warning)", padding: "2px 6px" }} onClick={() => updateStatus(f.id, "reviewed")} title="Mark reviewed">
                            👁 Review
                          </button>
                        )}
                        {f.status !== "open" && (
                          <button className="btn btn-ghost btn-sm" style={{ padding: "2px 6px", fontSize: 11 }} onClick={() => updateStatus(f.id, "open")} title="Reopen finding">
                            Reopen
                          </button>
                        )}
                      </div>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </div>
      )}

      {/* Comprehensive Finding Investigation & Email Inspector Panel */}
      {selectedFinding && (
        <div className="card" style={{ border: "1px solid var(--accent)", boxShadow: "0 8px 30px rgba(0,0,0,0.25)" }}>
          <div className="row between mb-4" style={{ borderBottom: "1px solid var(--border)", paddingBottom: 12 }}>
            <div className="row gap-2" style={{ alignItems: "center", flexWrap: "wrap" }}>
              <span className="badge" style={{ background: `${severityColor(selectedFinding.severity)}22`, color: severityColor(selectedFinding.severity), border: `1px solid ${severityColor(selectedFinding.severity)}44`, fontWeight: 700 }}>
                {selectedFinding.severity.toUpperCase()}
              </span>
              <span className="badge badge-gray" style={{ fontWeight: 700 }}>{selectedFinding.type_}</span>
              <span className={`badge ${statusBadge(selectedFinding.status)}`}>{selectedFinding.status.toUpperCase()}</span>
              <h3 style={{ fontSize: 16, fontWeight: 700, color: "var(--text-0)", margin: 0 }}>
                {selectedFinding.title}
              </h3>
            </div>
            <button className="btn btn-ghost btn-sm" onClick={() => setSelectedFinding(null)}>✕ Close Panel</button>
          </div>

          {/* Top Quick Actions Bar */}
          <div className="row between mb-4" style={{ background: "var(--bg-0)", padding: "10px 14px", borderRadius: "var(--r-md)", border: "1px solid var(--border)", flexWrap: "wrap", gap: 10 }}>
            <div className="row gap-2" style={{ alignItems: "center", fontSize: 12, color: "var(--text-2)", flexWrap: "wrap" }}>
              <span>Investigator Decision:</span>
              <button
                className={`btn btn-sm ${selectedFinding.status === "confirmed" ? "btn-primary" : "btn-ghost"}`}
                style={{ background: selectedFinding.status === "confirmed" ? "var(--success)" : undefined, color: selectedFinding.status === "confirmed" ? "#fff" : "var(--success)", borderColor: "var(--success)" }}
                onClick={() => updateStatus(selectedFinding.id, "confirmed")}
              >
                ✓ Confirm Threat
              </button>
              <button
                className={`btn btn-sm ${selectedFinding.status === "rejected" ? "btn-primary" : "btn-ghost"}`}
                style={{ background: selectedFinding.status === "rejected" ? "var(--danger)" : undefined, color: selectedFinding.status === "rejected" ? "#fff" : "var(--danger)", borderColor: "var(--danger)" }}
                onClick={() => updateStatus(selectedFinding.id, "rejected")}
              >
                ✗ Reject (False Positive)
              </button>
              <button
                className={`btn btn-sm ${selectedFinding.status === "reviewed" ? "btn-primary" : "btn-ghost"}`}
                style={{ background: selectedFinding.status === "reviewed" ? "var(--warning)" : undefined, color: selectedFinding.status === "reviewed" ? "#000" : "var(--warning)", borderColor: "var(--warning)" }}
                onClick={() => updateStatus(selectedFinding.id, "reviewed")}
              >
                👁 Mark Reviewed
              </button>
            </div>
            <div style={{ fontSize: 11, color: "var(--text-3)" }}>
              Recorded: {new Date(selectedFinding.created_at).toLocaleString()}
              {selectedFinding.reviewed_by && ` · Reviewed by: ${selectedFinding.reviewed_by}`}
            </div>
          </div>

          {/* Analysis Rationale Box */}
          <div className="mb-4" style={{ padding: 14, background: "rgba(239, 68, 68, 0.05)", border: "1px solid rgba(239, 68, 68, 0.2)", borderRadius: "var(--r-md)" }}>
            <h4 style={{ fontSize: 13, fontWeight: 700, color: "var(--danger)", marginBottom: 6 }}>
              🛡️ Automated Forensic Detection Rationale
            </h4>
            <p style={{ fontSize: 13, color: "var(--text-1)", lineHeight: 1.6, margin: 0 }}>
              {selectedFinding.description || "No automated rationale specified."}
            </p>
          </div>

          {/* Related Emails Section */}
          <div className="mb-4">
            <h4 style={{ fontSize: 14, fontWeight: 600, color: "var(--text-0)", marginBottom: 8 }}>
              Associated Evidentiary Email Messages ({relatedEmails.length})
            </h4>
            <p className="muted mb-3" style={{ fontSize: 12 }}>
              Inspect the exact email body, headers, and sender details that triggered this finding:
            </p>

            {loadingEmails ? (
              <div className="empty">Loading associated emails...</div>
            ) : relatedEmails.length === 0 ? (
              <div className="empty">No associated email messages found in database.</div>
            ) : (
              <div style={{ display: "flex", flexDirection: "column", gap: 12 }}>
                {/* Email Selector Tabs if multiple */}
                {relatedEmails.length > 1 && (
                  <div className="row gap-2 mb-2" style={{ flexWrap: "wrap" }}>
                    {relatedEmails.map((em, idx) => (
                      <button
                        key={em.id}
                        className={`btn btn-sm ${inspectingEmail?.id === em.id ? "btn-primary" : "btn-ghost"}`}
                        style={{ fontSize: 12 }}
                        onClick={() => setInspectingEmail(em)}
                      >
                        Email #{idx + 1}: {em.subject || "(no subject)"}
                      </button>
                    ))}
                  </div>
                )}

                {/* Inspected Email Preview Box */}
                {inspectingEmail && (
                  <div style={{ background: "var(--bg-0)", borderRadius: "var(--r-md)", border: "1px solid var(--border)", overflow: "hidden" }}>
                    <div style={{ padding: "12px 16px", background: "var(--bg-3)", borderBottom: "1px solid var(--border)" }}>
                      <div className="row between">
                        <div>
                          <strong style={{ fontSize: 14, color: "var(--text-0)" }}>{inspectingEmail.subject || "(no subject)"}</strong>
                          <div className="muted" style={{ fontSize: 12, marginTop: 2 }}>
                            From: <strong>{inspectingEmail.from_display || inspectingEmail.from_addr}</strong> ({inspectingEmail.from_addr})
                          </div>
                          <div className="muted" style={{ fontSize: 11, marginTop: 2 }}>
                            Date: {inspectingEmail.date_sent ? new Date(inspectingEmail.date_sent).toLocaleString() : "—"} · Folder: <span className="badge badge-gray">{inspectingEmail.folder_category}</span>
                          </div>
                        </div>
                        <div style={{ textAlign: "right" }}>
                          <span className="badge badge-red" style={{ fontSize: 11 }}>Risk Score: {inspectingEmail.risk_score}/100</span>
                        </div>
                      </div>
                    </div>

                    {/* Email Body Content */}
                    <div style={{ padding: 16 }}>
                      <div className="muted text-sm mb-2" style={{ fontWeight: 600 }}>Message Body Preview:</div>
                      <pre style={{
                        background: "var(--bg-1)",
                        border: "1px solid var(--border)",
                        borderRadius: "var(--r-sm)",
                        padding: 14,
                        fontSize: 12,
                        color: "var(--text-1)",
                        maxHeight: 240,
                        overflowY: "auto",
                        whiteSpace: "pre-wrap",
                        lineHeight: 1.5,
                        margin: 0,
                      }}>
                        {inspectingEmail.body_text || inspectingEmail.body_html || "No body content available in message."}
                      </pre>

                      {/* Raw Headers Preview Toggle */}
                      {inspectingEmail.headers_raw && (
                        <details style={{ marginTop: 12 }}>
                          <summary style={{ fontSize: 12, color: "var(--accent)", cursor: "pointer", fontWeight: 500 }}>
                            View Raw Transport Headers ({inspectingEmail.headers_raw.split('\n').length} lines)
                          </summary>
                          <pre className="mono" style={{
                            fontSize: 10,
                            background: "var(--bg-1)",
                            padding: 12,
                            borderRadius: "var(--r-sm)",
                            border: "1px solid var(--border)",
                            maxHeight: 180,
                            overflowY: "auto",
                            marginTop: 8,
                            color: "var(--text-2)",
                          }}>
                            {inspectingEmail.headers_raw}
                          </pre>
                        </details>
                      )}
                    </div>
                  </div>
                )}
              </div>
            )}
          </div>

          <hr style={{ borderColor: "var(--border)", margin: "20px 0" }} />

          {/* Investigator Notes on Finding */}
          <div>
            <h4 style={{ fontSize: 14, fontWeight: 600, color: "var(--text-0)", marginBottom: 8 }}>
              Investigator Review Notes &amp; Justification
            </h4>
            <p className="muted mb-3" style={{ fontSize: 12 }}>
              Document why this finding was confirmed, rejected, or flagged for inclusion in the final court report:
            </p>

            {/* Note Composer */}
            <form onSubmit={addNote} className="mb-4">
              <div className="row gap-2 mb-2">
                <input
                  className="input"
                  style={{ maxWidth: 200, padding: "6px 10px", fontSize: 12 }}
                  placeholder="Investigator name"
                  value={authorName}
                  onChange={e => setAuthorName(e.target.value)}
                />
                <input
                  className="input"
                  style={{ flex: 1, padding: "6px 12px", fontSize: 12 }}
                  placeholder="Record your observation or justification..."
                  value={noteText}
                  onChange={e => setNoteText(e.target.value)}
                />
                <button type="submit" className="btn btn-primary btn-sm" disabled={savingNote || !noteText.trim()}>
                  {savingNote ? "Saving..." : "+ Add Note"}
                </button>
              </div>
            </form>

            {/* Existing Notes List */}
            {parsedNotes.length === 0 ? (
              <div style={{ padding: 12, background: "var(--bg-0)", borderRadius: "var(--r-md)", border: "1px solid var(--border)", color: "var(--text-3)", fontSize: 12, textAlign: "center" }}>
                No investigator review notes recorded on this finding yet.
              </div>
            ) : (
              <div style={{ display: "flex", flexDirection: "column", gap: 8 }}>
                {parsedNotes.map((nt, idx) => (
                  <div key={idx} style={{ padding: "10px 14px", background: "var(--bg-0)", borderRadius: "var(--r-md)", border: "1px solid var(--border)", fontSize: 12 }}>
                    <span style={{ color: "var(--accent)", fontWeight: 600 }}>📝 Note #{idx + 1}</span>
                    <div style={{ color: "var(--text-1)", marginTop: 4, whiteSpace: "pre-wrap" }}>{nt}</div>
                  </div>
                ))}
              </div>
            )}
          </div>
        </div>
      )}
    </div>
  );
}
