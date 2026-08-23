import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { EmailListView } from "../views/EmailListView";
import { FindingsView } from "../views/FindingsView";
import { TargetProfileView } from "../views/TargetProfileView";
import { SearchView } from "../views/SearchView";
import { EntityDiveView } from "../views/EntityDiveView";

interface Case { id: string; title: string; case_number: string; description: string; status: string; target_email: string | null; target_name: string | null; target_organization: string | null; investigation_type: string; }
interface Evidence { id: string; case_id: string; filename: string; format: string; sha256: string; size_bytes: number; parse_status: string; message_count: number; deleted_recovered: number; acquired_at: string; source_description: string; parse_error: string | null; }
interface Dashboard { evidence_count: number; email_count: number; deleted_recovered: number; entity_count: number; finding_count: number; severity_breakdown: Record<string, number>; date_range: [string | null, string | null]; sent_count: number; inbox_count: number; soft_deleted_count: number; drafts_count: number; spam_count: number; other_count: number; high_risk_emails: number; }

type View = "dashboard" | "evidence" | "emails" | "sent" | "inbox" | "drafts" | "soft_deleted" | "hard_deleted" | "recoverable" | "spam" | "other" | "flagged" | "search" | "timeline" | "graph" | "entities" | "findings" | "custody" | "target";
type FolderFilter = "all" | "inbox" | "sent" | "drafts" | "soft_deleted" | "hard_deleted" | "recoverable" | "spam" | "other";

export function CaseWorkspace({ caseId, onBack }: { caseId: string; onBack: () => void }) {
  const [view, setView] = useState<View>("dashboard");
  const [caseData, setCaseData] = useState<Case | null>(null);
  const [dashboard, setDashboard] = useState<Dashboard | null>(null);
  const [evidence, setEvidence] = useState<Evidence[]>([]);
  const [loading, setLoading] = useState(true);
  const [sidebarCollapsed, setSidebarCollapsed] = useState(false);
  const [emailFolderOpen, setEmailFolderOpen] = useState(true);
  const [evidenceFolderOpen, setEvidenceFolderOpen] = useState(true);
  const [investigationFolderOpen, setInvestigationFolderOpen] = useState(true);
  const [folderFilter, setFolderFilter] = useState<FolderFilter>("all");

  const hasEvidence = evidence.length > 0;
  const hasDone = evidence.some((e) => e.parse_status === "done");

  const loadAll = useCallback(async () => {
    setLoading(true);
    try {
      const [c, ev, dash] = await Promise.all([
        invoke<Case>("case_get", { input: { case_id: caseId } }),
        invoke<Evidence[]>("evidence_list", { input: { case_id: caseId } }),
        invoke<Dashboard>("dashboard", { input: { case_id: caseId } }),
      ]);
      setCaseData(c);
      setEvidence(ev);
      setDashboard(dash);
    } catch (e) { console.error(e); }
    finally { setLoading(false); }
  }, [caseId]);

  useEffect(() => { loadAll(); }, [loadAll]);

  // Auto-refresh while parsing
  useEffect(() => {
    const hasParsing = evidence.some(e => e.parse_status === "parsing");
    if (!hasParsing) return;
    const interval = setInterval(loadAll, 3000);
    return () => clearInterval(interval);
  }, [evidence, loadAll]);

  // Get email counts from dashboard
  const getEmailCounts = () => {
    return {
      sent: dashboard?.sent_count || 0,
      inbox: dashboard?.inbox_count || 0,
      soft_deleted: dashboard?.soft_deleted_count || 0,
      drafts: dashboard?.drafts_count || 0,
      spam: dashboard?.spam_count || 0,
      other: dashboard?.other_count || 0,
      total: dashboard?.email_count || 0,
    };
  };

  const emailCounts = getEmailCounts();

  if (loading && !caseData) return <div className="app"><div className="empty">Loading case...</div></div>;

  return (
    <div className="app">
      <header className="topbar">
        <div className="row gap-4">
          <button className="btn btn-ghost btn-sm" onClick={onBack}>← Back to Cases</button>
          <div className="brand" style={{ cursor: "pointer" }}>
            <img src="/j12-logo.png" alt="J12" className="topbar-logo" />
            <div>
              <div className="brand-title"><span className="brand-j">J</span><span className="brand-12">12</span> · {caseData?.title || "Case"}</div>
              <div className="brand-sub">{caseData?.case_number || "No case number"}</div>
            </div>
          </div>
        </div>
        <div className="row gap-4">
          {caseData?.target_email && (
            <div style={{ textAlign: "right" }}>
              <div style={{ fontSize: 10, color: "var(--text-3)" }}>TARGET</div>
              <div style={{ fontSize: 12, color: "var(--accent)", fontFamily: "var(--mono)" }}>{caseData.target_email}</div>
            </div>
          )}
          {hasDone && <span className="badge badge-green">● {dashboard?.email_count?.toLocaleString() || 0} emails</span>}
          <span className="muted">{evidence.length} source(s)</span>
        </div>
      </header>

      <div className="body">
         {/* Case Navigator Sidebar */}
         <nav className="sidebar" style={{ width: sidebarCollapsed ? 50 : 220, minWidth: sidebarCollapsed ? 50 : 220 }}>
           <div className="sb-section">
             <button className="sb-toggle" onClick={() => setSidebarCollapsed(!sidebarCollapsed)}>
               {sidebarCollapsed ? "→" : "←"}
             </button>
           </div>

           {/* Case Dashboard - Always at top */}
           {!sidebarCollapsed && (
             <div className="sb-folder" style={{ marginBottom: 8 }}>
               <button className={`sb-item ${view === "dashboard" ? "active" : ""}`} onClick={() => setView("dashboard")} style={{ fontWeight: 600 }}>
                 <span className="sb-icon">◫</span> Case Dashboard
               </button>
             </div>
           )}

           {/* Target Profile */}
           {!sidebarCollapsed && (
             <div className="sb-folder" style={{ marginBottom: 8 }}>
               <button className={`sb-item ${view === "target" ? "active" : ""}`} onClick={() => setView("target")} style={{ fontWeight: 500 }}>
                 <span className="sb-icon">👤</span> Target Profile
                 {caseData?.target_email && <span className="sb-count" style={{ background: "var(--accent)" }}>●</span>}
               </button>
             </div>
           )}

          {/* Email Folders - Collapsible */}
          <div className="sb-folder">
            <div className="sb-folder-header" onClick={() => setEmailFolderOpen(!emailFolderOpen)}>
              <span className="sb-folder-arrow">{emailFolderOpen ? "▼" : "▶"}</span>
              <span className="sb-label" style={{ margin: 0 }}>Email Folders</span>
            </div>
            {emailFolderOpen && !sidebarCollapsed && (
              <div className="sb-folder-content">
                <button className={`sb-item ${folderFilter === "all" ? "active" : ""}`} onClick={() => { setFolderFilter("all"); setView("emails"); }} style={{ opacity: hasDone ? 1 : 0.4 }}>
                  <span className="sb-icon">📥</span> All Emails
                  <span className="sb-count">{emailCounts.total}</span>
                </button>
                <button className={`sb-item ${folderFilter === "inbox" ? "active" : ""}`} onClick={() => { setFolderFilter("inbox"); setView("inbox"); }} style={{ opacity: hasDone ? 1 : 0.4 }}>
                  <span className="sb-icon">📥</span> Inbox
                  <span className="sb-count">{emailCounts.inbox || 0}</span>
                </button>
                <button className={`sb-item ${folderFilter === "sent" ? "active" : ""}`} onClick={() => { setFolderFilter("sent"); setView("sent"); }} style={{ opacity: hasDone ? 1 : 0.4 }}>
                  <span className="sb-icon">📤</span> Sent
                  <span className="sb-count">{emailCounts.sent || 0}</span>
                </button>
                <button className={`sb-item ${folderFilter === "drafts" ? "active" : ""}`} onClick={() => { setFolderFilter("drafts"); setView("drafts"); }} style={{ opacity: hasDone ? 1 : 0.4 }}>
                  <span className="sb-icon">📝</span> Drafts
                  <span className="sb-count">{emailCounts.drafts || 0}</span>
                </button>
                <button className={`sb-item ${folderFilter === "soft_deleted" ? "active" : ""}`} onClick={() => { setFolderFilter("soft_deleted"); setView("soft_deleted"); }} style={{ opacity: hasDone ? 1 : 0.4 }}>
                  <span className="sb-icon">🗑️</span> Deleted (Recycle Bin)
                  <span className="sb-count">{emailCounts.soft_deleted || 0}</span>
                </button>
                <button className={`sb-item ${folderFilter === "hard_deleted" ? "active" : ""}`} onClick={() => { setFolderFilter("hard_deleted"); setView("hard_deleted"); }} style={{ opacity: 0.5 }}>
                  <span className="sb-icon">⚠</span> Permanently Deleted
                </button>
                <button className={`sb-item ${folderFilter === "recoverable" ? "active" : ""}`} onClick={() => { setFolderFilter("recoverable"); setView("recoverable"); }} style={{ opacity: 0.5 }}>
                  <span className="sb-icon">♻</span> Recoverable Items
                </button>
                <button className={`sb-item ${folderFilter === "spam" ? "active" : ""}`} onClick={() => { setFolderFilter("spam"); setView("spam"); }} style={{ opacity: hasDone ? 1 : 0.4 }}>
                  <span className="sb-icon">⚠</span> Spam/Junk
                  <span className="sb-count">{emailCounts.spam || 0}</span>
                </button>
                <button className={`sb-item ${folderFilter === "other" ? "active" : ""}`} onClick={() => { setFolderFilter("other"); setView("other"); }} style={{ opacity: hasDone ? 1 : 0.4 }}>
                  <span className="sb-icon">📁</span> Other
                  <span className="sb-count">{emailCounts.other || 0}</span>
                </button>
              </div>
            )}
          </div>

          {/* Evidence Sources - Collapsible */}
          <div className="sb-folder">
            <div className="sb-folder-header" onClick={() => setEvidenceFolderOpen(!evidenceFolderOpen)}>
              <span className="sb-folder-arrow">{evidenceFolderOpen ? "▼" : "▶"}</span>
              <span className="sb-label" style={{ margin: 0 }}>Evidence Sources</span>
            </div>
            {evidenceFolderOpen && !sidebarCollapsed && (
              <div className="sb-folder-content">
                {evidence.map((e) => (
                  <button key={e.id} className={`sb-item ${view === "evidence" ? "active" : ""}`} onClick={() => setView("evidence")}>
                    <span className="sb-icon">{e.format === "eml" ? "📧" : e.format === "mbox" ? "📦" : "📄"}</span>
                    <span className="sb-text-truncate">{e.filename}</span>
                    <span className={`sb-status sb-${e.parse_status}`}>{e.parse_status === "done" ? "✓" : e.parse_status === "error" ? "!" : "•"}</span>
                  </button>
                ))}
              </div>
            )}
          </div>

          {/* Investigation - Collapsible */}
          <div className="sb-folder">
            <div className="sb-folder-header" onClick={() => setInvestigationFolderOpen(!investigationFolderOpen)}>
              <span className="sb-folder-arrow">{investigationFolderOpen ? "▼" : "▶"}</span>
              <span className="sb-label" style={{ margin: 0 }}>Investigation</span>
            </div>
            {investigationFolderOpen && !sidebarCollapsed && (
              <div className="sb-folder-content">
                <button className={`sb-item ${view === "graph" ? "active" : ""}`} onClick={() => hasDone && setView("graph")} style={{ opacity: hasDone ? 1 : 0.4 }}>
                  <span className="sb-icon">◎</span> Graph
                </button>
                <button className={`sb-item ${view === "entities" ? "active" : ""}`} onClick={() => hasDone && setView("entities")} style={{ opacity: hasDone ? 1 : 0.4 }}>
                  <span className="sb-icon">◉</span> Entities
                </button>
                <button className={`sb-item ${view === "findings" ? "active" : ""}`} onClick={() => setView("findings")}>
                  <span className="sb-icon">⚠</span> Findings
                  {dashboard && dashboard.finding_count > 0 && <span className="sb-count">{dashboard.finding_count}</span>}
                </button>
                <button className={`sb-item ${view === "search" ? "active" : ""}`} onClick={() => hasDone && setView("search")} style={{ opacity: hasDone ? 1 : 0.4 }}>
                  <span className="sb-icon">🔍</span> Search
                </button>
                <button className={`sb-item ${view === "timeline" ? "active" : ""}`} onClick={() => hasDone && setView("timeline")} style={{ opacity: hasDone ? 1 : 0.4 }}>
                  <span className="sb-icon">◷</span> Timeline
                </button>
              </div>
            )}
          </div>

          {/* Case Management */}
          <div className="sb-folder">
            <div className="sb-folder-header">
              <span className="sb-folder-arrow">▼</span>
              <span className="sb-label" style={{ margin: 0 }}>Case Management</span>
            </div>
            {!sidebarCollapsed && (
              <div className="sb-folder-content">
                <button className={`sb-item ${view === "custody" ? "active" : ""}`} onClick={() => setView("custody")}>
                  <span className="sb-icon">📋</span> Chain of Custody
                </button>
                <button className="sb-item" style={{ opacity: 0.4 }}>
                  <span className="sb-icon">📝</span> Notes
                </button>
                <button className="sb-item" style={{ opacity: 0.4 }}>
                  <span className="sb-icon">📊</span> Reports
                </button>
              </div>
            )}
          </div>
        </nav>

        {/* Main content area */}
        <main className="content">
          {view === "dashboard" && dashboard && <DashboardView data={dashboard} evidence={evidence} caseData={caseData} />}
          {view === "evidence" && <EvidenceView evidence={evidence} caseId={caseId} onRefresh={loadAll} />}
          {view === "emails" && <EmailListView caseId={caseId} filter="all" />}
          {view === "sent" && <EmailListView caseId={caseId} filter="sent" />}
          {view === "inbox" && <EmailListView caseId={caseId} filter="inbox" />}
          {view === "drafts" && <EmailListView caseId={caseId} filter="drafts" />}
          {view === "soft_deleted" && <EmailListView caseId={caseId} filter="soft_deleted" />}
          {view === "hard_deleted" && <EmailListView caseId={caseId} filter="hard_deleted" />}
          {view === "recoverable" && <EmailListView caseId={caseId} filter="recoverable" />}
          {view === "spam" && <EmailListView caseId={caseId} filter="spam" />}
          {view === "other" && <EmailListView caseId={caseId} filter="other" />}
          {view === "search" && <SearchView caseId={caseId} />}
          {view === "entities" && <EntityDiveView caseId={caseId} />}
          {view === "timeline" && <div className="card"><div className="empty">Timeline — Phase 4</div></div>}
          {view === "graph" && <div className="card"><div className="empty">Communication graph — Phase 4</div></div>}
          {view === "findings" && <FindingsView caseId={caseId} />}
          {view === "target" && <TargetProfileView caseId={caseId} caseData={caseData} />}
          {view === "custody" && <CustodyView evidence={evidence} caseId={caseId} />}
        </main>
      </div>
    </div>
  );
}

function DashboardView({ data, evidence, caseData }: { data: Dashboard; evidence: Evidence[]; caseData: Case | null }) {
  const severityData = [
    { label: "Critical", value: data.severity_breakdown?.critical || 0, color: "#ef4444" },
    { label: "High", value: data.severity_breakdown?.high || 0, color: "#f97316" },
    { label: "Medium", value: data.severity_breakdown?.medium || 0, color: "#eab308" },
    { label: "Low", value: data.severity_breakdown?.low || 0, color: "#22c55e" },
  ];
  const totalFindings = severityData.reduce((sum, s) => sum + s.value, 0);
  const maxSeverity = Math.max(...severityData.map(s => s.value), 1);

  return (
    <div>
      <div className="row between mb-4">
        <div>
          <h2 style={{ fontSize: 24, fontWeight: 700, color: "var(--text-0)", marginBottom: 4 }}>Case Dashboard</h2>
          <p className="muted">Overview of evidence and investigation findings</p>
        </div>
        <button className="btn btn-primary" onClick={() => { /* trigger analysis refresh */ }}>
          ↻ Refresh
        </button>
      </div>

      {/* Target Info Card */}
      {(caseData?.target_name || caseData?.target_email || caseData?.target_organization) && (
        <div className="card mb-4" style={{ borderLeft: "4px solid var(--accent)" }}>
          <div className="row between">
            <div>
              <div style={{ fontSize: 10, fontWeight: 600, color: "var(--accent)", letterSpacing: "0.05em", marginBottom: 8 }}>INVESTIGATION TARGET</div>
              <div className="row gap-4" style={{ flexWrap: "wrap" }}>
                {caseData.target_name && (
                  <div>
                    <div style={{ fontSize: 11, color: "var(--text-3)" }}>Name</div>
                    <div style={{ fontSize: 14, fontWeight: 600, color: "var(--text-0)" }}>{caseData.target_name}</div>
                  </div>
                )}
                {caseData.target_email && (
                  <div>
                    <div style={{ fontSize: 11, color: "var(--text-3)" }}>Email</div>
                    <div style={{ fontSize: 14, fontWeight: 500, color: "var(--accent)", fontFamily: "var(--mono)" }}>{caseData.target_email}</div>
                  </div>
                )}
                {caseData.target_organization && (
                  <div>
                    <div style={{ fontSize: 11, color: "var(--text-3)" }}>Organization</div>
                    <div style={{ fontSize: 14, fontWeight: 500, color: "var(--text-1)" }}>{caseData.target_organization}</div>
                  </div>
                )}
              </div>
            </div>
            <div style={{ textAlign: "right" }}>
              <div style={{ fontSize: 11, color: "var(--text-3)" }}>Case Number</div>
              <div style={{ fontSize: 13, fontFamily: "var(--mono)", color: "var(--text-1)" }}>{caseData.case_number || "—"}</div>
            </div>
          </div>
        </div>
      )}

      {/* KPI Cards */}
      <div className="kpi-grid">
        <div className="kpi">
          <div className="kpi-val">{data.email_count.toLocaleString()}</div>
          <div className="kpi-label">Emails</div>
        </div>
        <div className="kpi">
          <div className="kpi-val" style={{ color: "var(--accent)" }}>{data.entity_count || 0}</div>
          <div className="kpi-label">Entities</div>
        </div>
        <div className="kpi">
          <div className="kpi-val" style={{ color: "var(--warning)" }}>{data.deleted_recovered}</div>
          <div className="kpi-label">Deleted Recovered</div>
        </div>
        <div className="kpi">
          <div className="kpi-val" style={{ color: totalFindings > 0 ? "var(--danger)" : "var(--text-0)" }}>{totalFindings}</div>
          <div className="kpi-label">Findings</div>
        </div>
        <div className="kpi">
          <div className="kpi-val" style={{ color: "var(--success)" }}>{data.evidence_count}</div>
          <div className="kpi-label">Evidence</div>
        </div>
      </div>

      {/* Folder Breakdown */}
      <div className="card mb-4">
        <h3 style={{ fontSize: 15, fontWeight: 600, marginBottom: 16 }}>Email Folder Breakdown</h3>
        <div className="row gap-4" style={{ flexWrap: "wrap" }}>
          {[
            { label: "Inbox", count: data.inbox_count, color: "#3b82f6" },
            { label: "Sent", count: data.sent_count, color: "#22c55e" },
            { label: "Deleted", count: data.soft_deleted_count, color: "#f97316" },
            { label: "Drafts", count: data.drafts_count, color: "#a855f7" },
            { label: "Spam", count: data.spam_count, color: "#ef4444" },
            { label: "Other", count: data.other_count, color: "#6b7280" },
          ].map(folder => (
            <div key={folder.label} style={{ flex: 1, minWidth: 120, padding: 12, background: "var(--bg-3)", borderRadius: "var(--r-sm)", textAlign: "center" }}>
              <div style={{ fontSize: 20, fontWeight: 700, color: folder.color }}>{folder.count.toLocaleString()}</div>
              <div style={{ fontSize: 10, color: "var(--text-3)", marginTop: 4 }}>{folder.label}</div>
            </div>
          ))}
        </div>
      </div>

      {/* Severity Breakdown */}
      {totalFindings > 0 && (
        <div className="card mb-4">
          <h3 style={{ fontSize: 15, fontWeight: 600, marginBottom: 16 }}>Findings by Severity</h3>
          <div style={{ display: "flex", flexDirection: "column", gap: 12 }}>
            {severityData.map(sev => (
              <div key={sev.label} className="row gap-4">
                <span style={{ width: 70, fontSize: 12, color: sev.color, fontWeight: 600 }}>{sev.label}</span>
                <div style={{ flex: 1, height: 24, background: "var(--bg-3)", borderRadius: "var(--r-sm)", overflow: "hidden" }}>
                  <div style={{ width: `${(sev.value / maxSeverity) * 100}%`, height: "100%", background: sev.color, borderRadius: "var(--r-sm)", opacity: 0.7 }} />
                </div>
                <span style={{ width: 40, textAlign: "right", fontSize: 13, fontWeight: 600, color: "var(--text-1)" }}>{sev.value}</span>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* Evidence Status */}
      {evidence.length > 0 && (
        <div className="card">
          <h3 style={{ fontSize: 15, fontWeight: 600, marginBottom: 16 }}>Evidence Status</h3>
          <table>
            <thead>
              <tr>
                <th className="th">File</th>
                <th className="th">Format</th>
                <th className="th">Status</th>
                <th className="th">Messages</th>
                <th className="th">SHA-256</th>
              </tr>
            </thead>
            <tbody>
              {evidence.map(e => (
                <tr key={e.id}>
                  <td className="td">{e.filename}</td>
                  <td className="td"><span className="badge badge-blue">{e.format}</span></td>
                  <td className="td"><span className={`badge badge-${e.parse_status === "done" ? "green" : e.parse_status === "error" ? "red" : e.parse_status === "parsing" ? "blue" : "gray"}`}>{e.parse_status}</span></td>
                  <td className="td">{e.message_count.toLocaleString()}</td>
                  <td className="td mono muted">{e.sha256.slice(0, 10)}…</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
    </div>
  );
}

function EvidenceView({ evidence, caseId, onRefresh }: { evidence: Evidence[]; caseId: string; onRefresh: () => void }) {
  const [uploading, setUploading] = useState(false);
  const [logs, setLogs] = useState<any[]>([]);
  const [selectedId, setSelectedId] = useState<string | null>(null);

  const addLog = (level: string, message: string) => {
    setLogs(prev => [...prev, { time: new Date().toLocaleTimeString(), level, message }]);
  };

  const handleUpload = async () => {
    try {
      const selected = await invoke<string | null>("open_file_dialog");
      if (!selected) return;
      setUploading(true);
      addLog("info", `Uploading: ${selected}`);
      const ev = await invoke<any>("evidence_upload", { input: { case_id: caseId, file_path: selected, source_description: null } });
      addLog("success", `Uploaded: ${ev.filename} (${ev.format}, ${(ev.size_bytes / 1024).toFixed(0)} KB)`);
      addLog("info", "Auto-parsing...");
      invoke("parse_evidence", { evidenceId: ev.id }).then((count: any) => {
        addLog("success", `Parsed ${count} emails`);
        onRefresh();
      }).catch((err: any) => {
        addLog("error", `Parse failed: ${err}`);
      });
      onRefresh();
    } catch (e: any) { addLog("error", `Upload failed: ${e}`); }
    setUploading(false);
  };

  const handleParse = async (evidenceId: string, filename: string) => {
    addLog("info", `Parsing ${filename}...`);
    try {
      const count = await invoke<number>("parse_evidence", { evidenceId });
      addLog("success", `Parsed ${count} emails from ${filename}`);
      onRefresh();
    } catch (e: any) {
      addLog("error", `Parse failed: ${e}`);
      onRefresh();
    }
  };

  const selectedEvidence = selectedId ? evidence.find(e => e.id === selectedId) : null;

  return (
    <div>
      <div className="row between mb-4">
        <div>
          <h2 style={{ fontSize: 22, fontWeight: 700, color: "var(--text-0)" }}>Evidence</h2>
          <p className="muted">Manage evidence sources for this case</p>
        </div>
        <button className="btn btn-primary" onClick={handleUpload} disabled={uploading}>
          {uploading ? "Uploading..." : "+ Add Evidence"}
        </button>
      </div>

      {logs.length > 0 && (
        <div className="card mb-4" style={{ maxHeight: 150, overflowY: "auto", background: "var(--bg-0)" }}>
          <div className="row between mb-4">
            <h4 style={{ fontSize: 12, fontWeight: 600 }}>Activity Log</h4>
            <button className="btn btn-ghost btn-sm" onClick={() => setLogs([])}>Clear</button>
          </div>
          {logs.map((log, i) => (
            <div key={i} className="row gap-2" style={{ fontSize: 11, fontFamily: "var(--mono)", marginBottom: 2 }}>
              <span className="muted">{log.time}</span>
              <span className={`badge badge-${log.level === "error" ? "red" : log.level === "success" ? "green" : "blue"}`}>{log.level}</span>
              <span>{log.message}</span>
            </div>
          ))}
        </div>
      )}

      {evidence.length === 0 ? (
        <div className="card" style={{ textAlign: "center", padding: "60px 40px" }}>
          <div style={{ fontSize: 48, marginBottom: 16 }}>📁</div>
          <h3 style={{ fontSize: 18, marginBottom: 8, color: "var(--text-0)" }}>No evidence yet</h3>
          <p className="muted mb-4">Upload email files to begin analysis.</p>
          <button className="btn btn-primary" onClick={handleUpload}>+ Upload Evidence</button>
        </div>
      ) : (
        <div className="card">
          <table>
            <thead>
              <tr>
                <th className="th">File</th>
                <th className="th">Format</th>
                <th className="th">Size</th>
                <th className="th">Status</th>
                <th className="th">Messages</th>
                <th className="th">SHA-256</th>
                <th className="th">Actions</th>
              </tr>
            </thead>
            <tbody>
              {evidence.map((e) => (
                <tr key={e.id} onClick={() => setSelectedId(selectedId === e.id ? null : e.id)} className="tr-click" style={{ background: selectedId === e.id ? "var(--bg-3)" : "transparent" }}>
                  <td className="td">{e.filename}</td>
                  <td className="td"><span className={`badge badge-${e.format === "eml" ? "blue" : e.format === "mbox" ? "green" : "orange"}`}>{e.format}</span></td>
                  <td className="td muted">{(e.size_bytes / 1024).toFixed(0)} KB</td>
                  <td className="td"><span className={`badge ${e.parse_status === "done" ? "badge-green" : e.parse_status === "error" ? "badge-red" : e.parse_status === "parsing" ? "badge-blue" : "badge-gray"}`}>{e.parse_status}</span></td>
                  <td className="td">{e.message_count}</td>
                  <td className="td mono muted">{e.sha256.slice(0, 12)}…</td>
                  <td className="td">
                    {e.parse_status === "pending" && <button className="btn btn-primary btn-sm" onClick={(ev) => { ev.stopPropagation(); handleParse(e.id, e.filename); }}>Parse</button>}
                    {e.parse_status === "parsing" && <span className="muted text-sm">Parsing...</span>}
                    {e.parse_status === "error" && <button className="btn btn-ghost btn-sm" onClick={(ev) => { ev.stopPropagation(); handleParse(e.id, e.filename); }}>Retry</button>}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}

      {selectedEvidence && (
        <div className="card mt-4">
          <div className="row between mb-4">
            <h4 style={{ fontSize: 14, fontWeight: 600 }}>Evidence Details</h4>
            <button className="btn btn-ghost btn-sm" onClick={() => setSelectedId(null)}>Close</button>
          </div>
          <div className="grid-2" style={{ fontSize: 13 }}>
            <div><span className="muted">File:</span> {selectedEvidence.filename}</div>
            <div><span className="muted">Format:</span> {selectedEvidence.format}</div>
            <div><span className="muted">Size:</span> {selectedEvidence.size_bytes} bytes</div>
            <div><span className="muted">Status:</span> <span className={`badge ${selectedEvidence.parse_status === "done" ? "badge-green" : selectedEvidence.parse_status === "error" ? "badge-red" : "badge-gray"}`}>{selectedEvidence.parse_status}</span></div>
            <div><span className="muted">Messages:</span> {selectedEvidence.message_count}</div>
            <div><span className="muted">Deleted Recovered:</span> {selectedEvidence.deleted_recovered}</div>
            <div><span className="muted">SHA-256:</span> <span className="mono">{selectedEvidence.sha256}</span></div>
            <div><span className="muted">Acquired:</span> {new Date(selectedEvidence.acquired_at).toLocaleString()}</div>
            <div style={{ gridColumn: "1 / -1" }}><span className="muted">Source:</span> {selectedEvidence.source_description || "—"}</div>
          </div>
          {selectedEvidence.parse_error && (
            <div style={{ marginTop: 12, padding: 12, background: "rgba(239,68,68,0.1)", borderRadius: 8, fontSize: 12, color: "var(--danger)" }}>
              <strong>Error:</strong> {selectedEvidence.parse_error}
            </div>
          )}
        </div>
      )}
    </div>
  );
}

function CustodyView({ evidence, caseId }: { evidence: Evidence[]; caseId: string }) {
  const [custody, setCustody] = useState<any[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    invoke<any[]>("custody_chain", { input: { case_id: caseId } }).then(events => { setCustody(events); setLoading(false); }).catch(() => setLoading(false));
  }, [caseId]);

  return (
    <div>
      <h2 style={{ fontSize: 22, fontWeight: 700, color: "var(--text-0)", marginBottom: 16 }}>Chain of Custody</h2>
      {loading ? <div className="empty">Loading...</div> : custody.length === 0 ? (
        <div className="card"><div className="empty">No custody events yet</div></div>
      ) : (
        <div className="card">
          <table>
            <thead><tr><th className="th">Action</th><th className="th">Timestamp</th><th className="th">Tool</th><th className="th">Detail</th><th className="th">Hash</th></tr></thead>
            <tbody>
              {custody.map((e, i) => (
                <tr key={i}>
                  <td className="td"><span className="badge badge-blue">{e.action}</span></td>
                  <td className="td muted">{new Date(e.timestamp).toLocaleString()}</td>
                  <td className="td">{e.tool} v{e.tool_version}</td>
                  <td className="td muted">{e.detail}</td>
                  <td className="td mono muted">{e.hash_after?.slice(0, 12) || "—"}…</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
    </div>
  );
}