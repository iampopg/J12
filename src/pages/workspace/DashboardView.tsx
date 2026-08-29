import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Dashboard, Evidence, Case, View, cleanDisplayName } from "./types";

interface Props {
  data: Dashboard;
  evidence: Evidence[];
  caseData: Case | null;
  caseId: string;
  onNavigate: (view: View) => void;
  onRefresh: () => void;
}

export function DashboardView({
  data,
  evidence,
  caseData,
  caseId,
  onNavigate,
  onRefresh,
}: Props) {
  const [criticalFindings, setCriticalFindings] = useState<any[]>([]);
  const [targetPartners, setTargetPartners] = useState<any[]>([]);
  const [analyzing, setAnalyzing] = useState(false);

  useEffect(() => {
    invoke<any[]>("findings_list", { input: { case_id: caseId } })
      .then((res) => {
        const critical = (res || []).filter((f) => f.severity === "critical" || f.severity === "high");
        setCriticalFindings(critical.slice(0, 3));
      })
      .catch(() => setCriticalFindings([]));

    if (caseData?.target_email) {
      invoke<any>("entity_dive", { input: { case_id: caseId, email: caseData.target_email } })
        .then((res) => {
          if (res?.top_sent_to || res?.top_received_from) {
            const combined = [...(res.top_sent_to || []), ...(res.top_received_from || [])];
            setTargetPartners(combined.slice(0, 4));
          }
        })
        .catch(() => setTargetPartners([]));
    }
  }, [caseId, caseData?.target_email]);

  const severityData = [
    { label: "Critical", value: data.severity_breakdown?.critical || 0, color: "#ef4444" },
    { label: "High", value: data.severity_breakdown?.high || 0, color: "#f97316" },
    { label: "Medium", value: data.severity_breakdown?.medium || 0, color: "#eab308" },
    { label: "Low", value: data.severity_breakdown?.low || 0, color: "#22c55e" },
  ];
  const totalFindings = severityData.reduce((sum, s) => sum + s.value, 0);
  const maxSeverity = Math.max(...severityData.map((s) => s.value), 1);

  const handleRunAnalysis = async () => {
    setAnalyzing(true);
    try {
      await invoke("run_analysis", { input: { case_id: caseId } });
      onRefresh();
    } catch (e) {
      console.error("Analysis failed:", e);
    } finally {
      setAnalyzing(false);
    }
  };

  return (
    <div>
      {/* Top Header & Investigation Quick Actions Bar */}
      <div className="row between mb-4" style={{ flexWrap: "wrap", gap: 12 }}>
        <div>
          <h2 style={{ fontSize: 24, fontWeight: 800, color: "var(--text-0)", marginBottom: 4 }}>
            Case Investigation Command Center
          </h2>
          <p className="muted" style={{ fontSize: 12 }}>
            Central intelligence hub for evidence triage, threat detection, entity profiling, and case reporting.
          </p>
        </div>
        <div className="row gap-2">
          <button
            className="btn btn-ghost btn-sm"
            onClick={handleRunAnalysis}
            disabled={analyzing}
            title="Run forensic rules & brand impersonation checks"
          >
            {analyzing ? "⚡ Analyzing..." : "⚡ Run Analysis"}
          </button>
          <button className="btn btn-primary btn-sm" onClick={onRefresh}>
            ↻ Refresh
          </button>
        </div>
      </div>

      {/* Investigation Action Shortcuts Bar */}
      <div
        className="card mb-4"
        style={{
          padding: "10px 14px",
          display: "flex",
          alignItems: "center",
          gap: 8,
          flexWrap: "wrap",
          background: "var(--bg-2)",
          border: "1px solid var(--border)",
        }}
      >
        <span style={{ fontSize: 11, fontWeight: 700, color: "var(--text-3)", marginRight: 6 }}>
          QUICK TOOLS:
        </span>
        <button
          className="btn btn-ghost btn-sm"
          style={{ fontSize: 11, padding: "4px 10px" }}
          onClick={() => onNavigate("search")}
        >
          🔍 Advanced Search
        </button>
        <button
          className="btn btn-ghost btn-sm"
          style={{ fontSize: 11, padding: "4px 10px" }}
          onClick={() => onNavigate("graph")}
        >
          🕸️ Network Graph
        </button>
        <button
          className="btn btn-ghost btn-sm"
          style={{ fontSize: 11, padding: "4px 10px" }}
          onClick={() => onNavigate("timeline")}
        >
          📅 Incident Timeline
        </button>
        <button
          className="btn btn-ghost btn-sm"
          style={{ fontSize: 11, padding: "4px 10px" }}
          onClick={() => onNavigate("entities")}
        >
          👤 Entity Profiles
        </button>
        <button
          className="btn btn-ghost btn-sm"
          style={{ fontSize: 11, padding: "4px 10px" }}
          onClick={() => onNavigate("findings")}
        >
          🛡️ Findings Matrix
        </button>
        <button
          className="btn btn-primary btn-sm"
          style={{ fontSize: 11, padding: "4px 12px", marginLeft: "auto" }}
          onClick={() => onNavigate("report")}
        >
          📄 Generate Report
        </button>
      </div>

      {/* Interactive 5-Metric Command Center Cards */}
      <div className="kpi-grid mb-4">
        <div
          className="kpi tr-click"
          style={{ cursor: "pointer" }}
          onClick={() => onNavigate("search")}
          title="Click to search all messages"
        >
          <div className="kpi-val">{data.email_count.toLocaleString()}</div>
          <div className="kpi-label">✉️ Processed Emails →</div>
        </div>

        <div
          className="kpi tr-click"
          style={{ cursor: "pointer" }}
          onClick={() => onNavigate("entities")}
          title="Click to explore entity profiles"
        >
          <div className="kpi-val" style={{ color: "var(--accent)" }}>
            {data.entity_count || 0}
          </div>
          <div className="kpi-label">👥 Entities Discovered →</div>
        </div>

        <div
          className="kpi tr-click"
          style={{ cursor: "pointer" }}
          onClick={() => onNavigate("soft_deleted")}
          title="Click to inspect deleted & recovered emails"
        >
          <div className="kpi-val" style={{ color: "var(--danger)" }}>
            {data.deleted_recovered.toLocaleString()}
          </div>
          <div className="kpi-label">🗑️ Deleted Recovered →</div>
        </div>

        <div
          className="kpi tr-click"
          style={{ cursor: "pointer" }}
          onClick={() => onNavigate("findings")}
          title="Click to review security findings"
        >
          <div
            className="kpi-val"
            style={{ color: totalFindings > 0 ? "var(--warning)" : "var(--text-0)" }}
          >
            {totalFindings}
          </div>
          <div className="kpi-label">🚨 Security Findings →</div>
        </div>

        <div
          className="kpi tr-click"
          style={{ cursor: "pointer" }}
          onClick={() => onNavigate("evidence")}
          title="Click to manage evidence containers"
        >
          <div className="kpi-val" style={{ color: "var(--success)" }}>
            {data.evidence_count}
          </div>
          <div className="kpi-label">📁 Evidence Containers →</div>
        </div>
      </div>

      {/* Target Subject Dossier & Active Security Threats Grid */}
      <div
        style={{
          display: "grid",
          gridTemplateColumns: "1fr 1fr",
          gap: 16,
          marginBottom: 16,
        }}
      >
        {/* Left: Investigation Target Dossier */}
        <div className="card mb-0" style={{ borderLeft: "4px solid var(--accent)", padding: 16 }}>
          <div className="row between mb-3">
            <span style={{ fontSize: 11, fontWeight: 700, color: "var(--accent)", letterSpacing: "0.06em" }}>
              🎯 CASE TARGET DOSSIER
            </span>
            <span className="badge badge-blue" style={{ fontSize: 10 }}>
              CASE #{caseData?.case_number || "J12-001"}
            </span>
          </div>

          <div style={{ fontSize: 16, fontWeight: 700, color: "var(--text-0)", marginBottom: 4 }}>
            {caseData?.target_name || "Target Not Set"}
          </div>
          <div style={{ fontSize: 12, color: "var(--accent)", fontFamily: "var(--mono)", marginBottom: 8 }}>
            {caseData?.target_email || "No primary email assigned"}
          </div>
          <div style={{ fontSize: 12, color: "var(--text-2)", marginBottom: 12 }}>
            Organization: <strong>{caseData?.target_organization || "N/A"}</strong>
          </div>

          {targetPartners.length > 0 && (
            <div>
              <div style={{ fontSize: 11, fontWeight: 600, color: "var(--text-3)", marginBottom: 6 }}>
                FREQUENT CORRESPONDENTS:
              </div>
              <div style={{ display: "flex", flexDirection: "column", gap: 4 }}>
                {targetPartners.map((p, i) => (
                  <div
                    key={i}
                    className="row between tr-click"
                    style={{
                      padding: "4px 8px",
                      background: "var(--bg-3)",
                      borderRadius: "var(--r-xs)",
                      fontSize: 11,
                    }}
                    onClick={() => onNavigate("entities")}
                  >
                    <span style={{ color: "var(--text-1)" }}>
                      {cleanDisplayName(p.display_name) || p.email}
                    </span>
                    <span className="badge badge-blue" style={{ fontSize: 9 }}>
                      {p.count} messages
                    </span>
                  </div>
                ))}
              </div>
            </div>
          )}
        </div>

        {/* Right: Active Threats & Security Alerts */}
        <div className="card mb-0" style={{ borderLeft: "4px solid #ef4444", padding: 16 }}>
          <div className="row between mb-3">
            <span style={{ fontSize: 11, fontWeight: 700, color: "#ef4444", letterSpacing: "0.06em" }}>
              🚨 CRITICAL SECURITY FINDINGS ({totalFindings})
            </span>
            <button
              className="btn btn-ghost btn-sm"
              style={{ fontSize: 10, padding: "2px 6px" }}
              onClick={() => onNavigate("findings")}
            >
              View All →
            </button>
          </div>

          {criticalFindings.length > 0 ? (
            <div style={{ display: "flex", flexDirection: "column", gap: 8 }}>
              {criticalFindings.map((f: any) => (
                <div
                  key={f.id}
                  className="tr-click"
                  style={{
                    padding: 10,
                    background: "var(--bg-3)",
                    borderRadius: "var(--r-xs)",
                    borderLeft: "3px solid #ef4444",
                  }}
                  onClick={() => onNavigate("findings")}
                >
                  <div className="row between mb-1">
                    <strong style={{ fontSize: 12, color: "var(--text-0)" }}>{f.title}</strong>
                    <span className="badge badge-red" style={{ fontSize: 9 }}>
                      {f.severity.toUpperCase()}
                    </span>
                  </div>
                  <div style={{ fontSize: 11, color: "var(--text-3)", overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
                    {f.description}
                  </div>
                </div>
              ))}
            </div>
          ) : (
            <div className="empty" style={{ padding: 24, fontSize: 12 }}>
              No critical threat violations flagged. Run analysis to scan archive.
            </div>
          )}
        </div>
      </div>

      {/* Interactive Folder Breakdown Tiles */}
      <div className="card mb-4">
        <h3 style={{ fontSize: 15, fontWeight: 600, marginBottom: 14 }}>
          Mailbox Folder Tally (Click to inspect folder)
        </h3>
        <div className="row gap-3" style={{ flexWrap: "wrap" }}>
          {[
            { label: "Inbox", count: data.inbox_count, color: "#3b82f6", view: "inbox" as View },
            { label: "Sent Items", count: data.sent_count, color: "#22c55e", view: "sent" as View },
            { label: "Deleted / Trash", count: data.soft_deleted_count, color: "#f97316", view: "soft_deleted" as View },
            { label: "Drafts", count: data.drafts_count, color: "#a855f7", view: "drafts" as View },
            { label: "Spam / Junk", count: data.spam_count, color: "#ef4444", view: "spam" as View },
            { label: "Other Folders", count: data.other_count, color: "#6b7280", view: "other" as View },
          ].map((folder) => (
            <div
              key={folder.label}
              className="tr-click"
              style={{
                flex: 1,
                minWidth: 120,
                padding: 12,
                background: "var(--bg-3)",
                borderRadius: "var(--r-sm)",
                textAlign: "center",
                cursor: "pointer",
              }}
              onClick={() => onNavigate(folder.view)}
            >
              <div style={{ fontSize: 20, fontWeight: 700, color: folder.color }}>
                {folder.count.toLocaleString()}
              </div>
              <div style={{ fontSize: 10, color: "var(--text-3)", marginTop: 4 }}>
                {folder.label} →
              </div>
            </div>
          ))}
        </div>
      </div>

      {/* Severity Breakdown */}
      {totalFindings > 0 && (
        <div className="card mb-4">
          <h3 style={{ fontSize: 15, fontWeight: 600, marginBottom: 16 }}>Findings by Severity</h3>
          <div style={{ display: "flex", flexDirection: "column", gap: 12 }}>
            {severityData.map((sev) => (
              <div key={sev.label} className="row gap-4">
                <span style={{ width: 70, fontSize: 12, color: sev.color, fontWeight: 600 }}>
                  {sev.label}
                </span>
                <div
                  style={{
                    flex: 1,
                    height: 24,
                    background: "var(--bg-3)",
                    borderRadius: "var(--r-sm)",
                    overflow: "hidden",
                  }}
                >
                  <div
                    style={{
                      width: `${(sev.value / maxSeverity) * 100}%`,
                      height: "100%",
                      background: sev.color,
                      borderRadius: "var(--r-sm)",
                      opacity: 0.7,
                    }}
                  />
                </div>
                <span
                  style={{
                    width: 40,
                    textAlign: "right",
                    fontSize: 13,
                    fontWeight: 600,
                    color: "var(--text-1)",
                  }}
                >
                  {sev.value}
                </span>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* Evidence Status */}
      {evidence.length > 0 && (
        <div className="card">
          <h3 style={{ fontSize: 15, fontWeight: 600, marginBottom: 16 }}>Evidence Containers &amp; Provenance</h3>
          <table>
            <thead>
              <tr>
                <th className="th">Container File</th>
                <th className="th" style={{ width: 80 }}>Format</th>
                <th className="th" style={{ width: 90 }}>Status</th>
                <th className="th" style={{ width: 90 }}>Messages</th>
                <th className="th">SHA-256 Acquisition Hash</th>
              </tr>
            </thead>
            <tbody>
              {evidence
                .reduce((unique: Evidence[], e) => {
                  const existing = unique.find((u) => u.filename === e.filename);
                  if (!existing) unique.push(e);
                  else if (e.message_count > existing.message_count) {
                    const idx = unique.indexOf(existing);
                    unique[idx] = e;
                  }
                  return unique;
                }, [])
                .map((e) => (
                  <tr key={e.id}>
                    <td className="td">
                      <strong>{e.filename}</strong>
                    </td>
                    <td className="td">
                      <span className="badge badge-blue">{e.format.toUpperCase()}</span>
                    </td>
                    <td className="td">
                      <span
                        className={`badge badge-${
                          e.parse_status === "done"
                            ? "green"
                            : e.parse_status === "error"
                            ? "red"
                            : e.parse_status === "parsing"
                            ? "blue"
                            : "gray"
                        }`}
                      >
                        {e.parse_status}
                      </span>
                    </td>
                    <td className="td">{e.message_count.toLocaleString()}</td>
                    <td className="td mono muted" style={{ fontSize: 11, color: "var(--accent)" }}>
                      {e.sha256}
                    </td>
                  </tr>
                ))}
            </tbody>
          </table>
        </div>
      )}
    </div>
  );
}
