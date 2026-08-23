import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";

interface Finding {
  id: string;
  case_id: string;
  type_: string;
  severity: string;
  confidence: string;
  title: string;
  description: string | null;
  email_ids: string;
  status: string;
  created_at: string;
  reviewed_by: string | null;
  reviewed_at: string | null;
}

interface Props {
  caseId: string;
}

export function FindingsView({ caseId }: Props) {
  const [findings, setFindings] = useState<Finding[]>([]);
  const [loading, setLoading] = useState(true);
  const [analyzing, setAnalyzing] = useState(false);
  const [filterSeverity, setFilterSeverity] = useState<string>("all");
  const [filterType, setFilterType] = useState<string>("all");
  const [filterStatus, setFilterStatus] = useState<string>("all");
  const [selectedFinding, setSelectedFinding] = useState<Finding | null>(null);
  const [noteText, setNoteText] = useState("");

  const loadFindings = useCallback(async () => {
    setLoading(true);
    try {
      const data = await invoke<Finding[]>("findings_list", { input: { case_id: caseId } });
      setFindings(data);
    } catch (e) { console.error(e); }
    finally { setLoading(false); }
  }, [caseId]);

  useEffect(() => { loadFindings(); }, [loadFindings]);

  const runAnalysis = async () => {
    setAnalyzing(true);
    try {
      const count = await invoke<number>("run_analysis", { caseId });
      alert(`Analysis complete: ${count} findings generated`);
      loadFindings();
    } catch (e: any) {
      alert(`Analysis failed: ${e}`);
    }
    setAnalyzing(false);
  };

  const updateStatus = async (id: string, newStatus: string) => {
    try {
      await invoke("update_finding_status", { findingId: id, newStatus, reviewedBy: "admin" });
      loadFindings();
    } catch (e: any) {
      alert(`Update failed: ${e}`);
    }
  };

  const addNote = async (id: string) => {
    if (!noteText.trim()) return;
    try {
      await invoke("add_finding_note", { findingId: id, note: noteText, author: "admin" });
      setNoteText("");
      loadFindings();
    } catch (e: any) {
      alert(`Note failed: ${e}`);
    }
  };

  // Filter findings
  const filtered = findings.filter(f => {
    if (filterSeverity !== "all" && f.severity !== filterSeverity) return false;
    if (filterType !== "all" && f.type_ !== filterType) return false;
    if (filterStatus !== "all" && f.status !== filterStatus) return false;
    return true;
  });

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
    ANOMALY: findings.filter(f => f.type_ === "ANOMALY").length,
    ATTACHMENT: findings.filter(f => f.type_ === "ATTACHMENT").length,
    ROUTING: findings.filter(f => f.type_ === "ROUTING").length,
  };

  const severityColor = (severity: string) => {
    switch (severity) {
      case "critical": return "#dc2626";
      case "high": return "#ea580c";
      case "medium": return "#ca8a04";
      default: return "#6b7280";
    }
  };

  const statusBadge = (status: string) => {
    switch (status) {
      case "open": return "badge-blue";
      case "confirmed": return "badge-green";
      case "rejected": return "badge-red";
      case "reviewed": return "badge-yellow";
      default: return "badge-gray";
    }
  };

  return (
    <div>
      <div className="row between mb-4">
        <div>
          <h2 style={{ fontSize: 22, fontWeight: 700, color: "var(--text-0)" }}>Forensic Findings</h2>
          <p className="muted">Automated analysis results and investigator review</p>
        </div>
        <button className="btn btn-primary" onClick={runAnalysis} disabled={analyzing}>
          {analyzing ? "Analyzing..." : "▶ Run Analysis"}
        </button>
      </div>

      {/* Severity Summary */}
      <div className="row gap-4 mb-4">
        <div style={{ flex: 1, padding: 16, background: "var(--bg-3)", borderRadius: "var(--r-md)", textAlign: "center", borderLeft: "4px solid #dc2626" }}>
          <div style={{ fontSize: 24, fontWeight: 700, color: "#dc2626" }}>{severityCounts.critical}</div>
          <div className="muted text-sm">Critical</div>
        </div>
        <div style={{ flex: 1, padding: 16, background: "var(--bg-3)", borderRadius: "var(--r-md)", textAlign: "center", borderLeft: "4px solid #ea580c" }}>
          <div style={{ fontSize: 24, fontWeight: 700, color: "#ea580c" }}>{severityCounts.high}</div>
          <div className="muted text-sm">High</div>
        </div>
        <div style={{ flex: 1, padding: 16, background: "var(--bg-3)", borderRadius: "var(--r-md)", textAlign: "center", borderLeft: "4px solid #ca8a04" }}>
          <div style={{ fontSize: 24, fontWeight: 700, color: "#ca8a04" }}>{severityCounts.medium}</div>
          <div className="muted text-sm">Medium</div>
        </div>
        <div style={{ flex: 1, padding: 16, background: "var(--bg-3)", borderRadius: "var(--r-md)", textAlign: "center", borderLeft: "4px solid #6b7280" }}>
          <div style={{ fontSize: 24, fontWeight: 700, color: "#6b7280" }}>{severityCounts.low}</div>
          <div className="muted text-sm">Low</div>
        </div>
      </div>

      {/* Type Summary */}
      <div className="row gap-2 mb-4" style={{ flexWrap: "wrap" }}>
        {Object.entries(typeCounts).map(([type, count]) => (
          count > 0 && (
            <span key={type} className="badge badge-gray" style={{ padding: "6px 12px", fontSize: 12 }}>
              {type}: {count}
            </span>
          )
        ))}
      </div>

      {/* Filters */}
      <div className="row gap-4 mb-4">
        <select className="input" style={{ width: "auto" }} value={filterSeverity} onChange={e => setFilterSeverity(e.target.value)}>
          <option value="all">All Severities</option>
          <option value="critical">Critical</option>
          <option value="high">High</option>
          <option value="medium">Medium</option>
          <option value="low">Low</option>
        </select>
        <select className="input" style={{ width: "auto" }} value={filterType} onChange={e => setFilterType(e.target.value)}>
          <option value="all">All Types</option>
          <option value="BEC">BEC</option>
          <option value="SPOOFING">Spoofing</option>
          <option value="ANOMALY">Anomaly</option>
          <option value="ATTACHMENT">Attachment</option>
          <option value="ROUTING">Routing</option>
        </select>
        <select className="input" style={{ width: "auto" }} value={filterStatus} onChange={e => setFilterStatus(e.target.value)}>
          <option value="all">All Statuses</option>
          <option value="open">Open</option>
          <option value="confirmed">Confirmed</option>
          <option value="rejected">Rejected</option>
          <option value="reviewed">Reviewed</option>
        </select>
        <span className="muted text-sm">{filtered.length} findings</span>
      </div>

      {loading ? (
        <div className="empty">Loading findings...</div>
      ) : findings.length === 0 ? (
        <div className="card" style={{ textAlign: "center", padding: "60px 40px" }}>
          <div style={{ fontSize: 48, marginBottom: 16 }}>🔍</div>
          <h3 style={{ fontSize: 18, marginBottom: 8, color: "var(--text-0)" }}>No findings yet</h3>
          <p className="muted mb-4">Run automated analysis to detect spoofing, authentication failures, and anomalies.</p>
          <button className="btn btn-primary" onClick={runAnalysis} disabled={analyzing}>
            {analyzing ? "Analyzing..." : "▶ Run Analysis"}
          </button>
        </div>
      ) : (
        <div className="card">
          <table>
            <thead>
              <tr>
                <th className="th" style={{ width: 80 }}>Severity</th>
                <th className="th" style={{ width: 80 }}>Type</th>
                <th className="th">Title</th>
                <th className="th" style={{ width: 100 }}>Status</th>
                <th className="th" style={{ width: 80 }}>Emails</th>
                <th className="th" style={{ width: 120 }}>Actions</th>
              </tr>
            </thead>
            <tbody>
              {filtered.map((f) => (
                <tr key={f.id} className="tr-click" onClick={() => setSelectedFinding(selectedFinding?.id === f.id ? null : f)}>
                  <td>
                    <span className="badge" style={{ background: severityColor(f.severity), color: "#fff", padding: "2px 8px", borderRadius: 4, fontSize: 11, fontWeight: 600 }}>
                      {f.severity?.toUpperCase()}
                    </span>
                  </td>
                  <td><span className="badge badge-gray">{f.type_}</span></td>
                  <td style={{ maxWidth: 300 }}>{f.title}</td>
                  <td><span className={`badge ${statusBadge(f.status)}`}>{f.status}</span></td>
                  <td className="mono">
                    {(() => {
                      try {
                        const ids = JSON.parse(f.email_ids || "[]");
                        return ids.length;
                      } catch { return 0; }
                    })()}
                  </td>
                  <td>
                    <div className="row gap-2" onClick={e => e.stopPropagation()}>
                      {f.status === "open" && (
                        <>
                          <button className="btn btn-ghost btn-sm" onClick={() => updateStatus(f.id, "confirmed")} title="Confirm finding">✓</button>
                          <button className="btn btn-ghost btn-sm" onClick={() => updateStatus(f.id, "rejected")} title="Reject finding">✗</button>
                        </>
                      )}
                      {f.status === "confirmed" && (
                        <button className="btn btn-ghost btn-sm" onClick={() => updateStatus(f.id, "reviewed")} title="Mark reviewed">👁</button>
                      )}
                    </div>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}

      {/* Finding Detail */}
      {selectedFinding && (
        <div className="card mt-4">
          <div className="row between mb-4">
            <h4 style={{ fontSize: 14, fontWeight: 600 }}>Finding Details</h4>
            <button className="btn btn-ghost btn-sm" onClick={() => setSelectedFinding(null)}>Close</button>
          </div>
          <div style={{ fontSize: 13 }}>
            <div className="mb-4">
              <span className="muted">Type:</span> <span className="badge badge-gray">{selectedFinding.type_}</span>
              <span className="muted" style={{ marginLeft: 16 }}>Severity:</span>
              <span className="badge" style={{ background: severityColor(selectedFinding.severity), color: "#fff" }}>{selectedFinding.severity}</span>
              <span className="muted" style={{ marginLeft: 16 }}>Confidence:</span> {selectedFinding.confidence}
            </div>
            <div className="mb-4">
              <h4 style={{ fontWeight: 600, marginBottom: 4 }}>{selectedFinding.title}</h4>
              <p style={{ color: "var(--text-1)" }}>{selectedFinding.description}</p>
            </div>
            <div className="mb-4">
              <span className="muted">Status:</span> <span className={`badge ${statusBadge(selectedFinding.status)}`}>{selectedFinding.status}</span>
              <span className="muted" style={{ marginLeft: 16 }}>Created:</span> {new Date(selectedFinding.created_at).toLocaleString()}
              {selectedFinding.reviewed_by && (
                <>
                  <span className="muted" style={{ marginLeft: 16 }}>Reviewed by:</span> {selectedFinding.reviewed_by}
                  {selectedFinding.reviewed_at && <span className="muted" style={{ marginLeft: 8 }}>at {new Date(selectedFinding.reviewed_at).toLocaleString()}</span>}
                </>
              )}
            </div>
            <div className="mb-4">
              <span className="muted">Related Emails:</span>
              <span className="mono">
                {(() => {
                  try {
                    const ids = JSON.parse(selectedFinding.email_ids || "[]");
                    return `${ids.length} email(s)`;
                  } catch { return "0"; }
                })()}
              </span>
            </div>
            <div className="mb-4">
              <span className="muted">Quick Actions:</span>
              <div className="row gap-2 mt-4">
                <button className="btn btn-primary btn-sm" onClick={() => updateStatus(selectedFinding.id, "confirmed")}>Confirm</button>
                <button className="btn btn-ghost btn-sm" onClick={() => updateStatus(selectedFinding.id, "rejected")}>Reject</button>
                <button className="btn btn-ghost btn-sm" onClick={() => updateStatus(selectedFinding.id, "reviewed")}>Mark Reviewed</button>
              </div>
            </div>
            <div>
              <span className="muted">Add Note:</span>
              <div className="row gap-2 mt-4">
                <input className="input" style={{ flex: 1 }} placeholder="Investigator note..." value={noteText} onChange={e => setNoteText(e.target.value)} />
                <button className="btn btn-primary btn-sm" onClick={() => addNote(selectedFinding.id)}>Add</button>
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
