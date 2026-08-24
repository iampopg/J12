import { useState, useEffect, useCallback, useRef } from "react";
import { invoke, Channel } from "@tauri-apps/api/core";
import { EmailListView } from "../views/EmailListView";
import { FindingsView } from "../views/FindingsView";
import { TargetProfileView } from "../views/TargetProfileView";
import { SearchView } from "../views/SearchView";
import { EntityDiveView } from "../views/EntityDiveView";
import { TimelineView } from "../views/TimelineView";
import { GraphView } from "../views/GraphView";
import { NotesView } from "../views/NotesView";
import { ReportView } from "../views/ReportView";
import { ArtifactsView } from "../views/ArtifactsView";
import { AttachmentsView } from "../views/AttachmentsView";
import { J12Logo } from "../components/J12Logo";

interface Case { id: string; title: string; case_number: string; description: string; status: string; target_email: string | null; target_name: string | null; target_organization: string | null; investigation_type: string; working_dir?: string | null; }
interface Evidence { id: string; case_id: string; filename: string; format: string; sha256: string; size_bytes: number; parse_status: string; message_count: number; deleted_recovered: number; acquired_at: string; source_description: string; parse_error: string | null; }
interface Dashboard { evidence_count: number; email_count: number; deleted_recovered: number; entity_count: number; finding_count: number; severity_breakdown: Record<string, number>; date_range: [string | null, string | null]; sent_count: number; inbox_count: number; important_count?: number; soft_deleted_count: number; drafts_count: number; spam_count: number; other_count: number; high_risk_emails: number; }

type View = "dashboard" | "evidence" | "emails" | "sent" | "inbox" | "important" | "drafts" | "soft_deleted" | "hard_deleted" | "recoverable" | "spam" | "other" | "flagged" | "search" | "timeline" | "graph" | "entities" | "findings" | "custody" | "target" | "notes" | "case_manage" | "report" | "integrity" | "artifacts" | "attachments";
type FolderFilter = "all" | "inbox" | "important" | "sent" | "drafts" | "soft_deleted" | "hard_deleted" | "recoverable" | "spam" | "other";

function cleanDisplayName(name: string | null): string {
  if (!name) return "";
  let n = name.trim();
  n = n.replace(/^['"]+|['"]+$/g, "");
  if (n.startsWith("/O=") || n.startsWith("/o=")) {
    const parts = n.split("/");
    for (const part of parts) {
      if (part.toUpperCase().startsWith("CN=")) {
        return part.substring(3).trim();
      }
    }
    return n;
  }
  if (n.includes(",")) {
    const parts = n.split(",");
    if (parts.length === 2) {
      const last = parts[0].trim();
      const first = parts[1].trim();
      if (!first.includes(" ") && !last.includes(" ")) {
        return `${first} ${last}`;
      }
    }
  }
  n = n.replace(/<.*$/, "").trim();
  return n;
}

export function CaseWorkspace({ caseId, onBack }: { caseId: string; onBack: () => void }) {
  const [view, setViewState] = useState<View>(() => {
    const saved = localStorage.getItem(`last_view_${caseId}`);
    return (saved as View) || "dashboard";
  });

  const setView = (v: View) => {
    localStorage.setItem(`last_view_${caseId}`, v);
    setViewState(v);
  };

  const [notesCount, setNotesCount] = useState(0);
  const [caseData, setCaseData] = useState<Case | null>(null);
  const [dashboard, setDashboard] = useState<Dashboard | null>(null);
  const [evidence, setEvidence] = useState<Evidence[]>([]);
  const [loading, setLoading] = useState(true);

  // Collapsible folder states with localStorage persistence
  const [sidebarCollapsed, setSidebarCollapsedState] = useState(() => localStorage.getItem("sb_collapsed") === "true");
  const setSidebarCollapsed = (val: boolean) => {
    localStorage.setItem("sb_collapsed", String(val));
    setSidebarCollapsedState(val);
  };

  const [overviewOpen, setOverviewOpenState] = useState(() => localStorage.getItem("sb_overview_open") !== "false");
  const setOverviewOpen = (val: boolean) => {
    localStorage.setItem("sb_overview_open", String(val));
    setOverviewOpenState(val);
  };

  const [evidenceFolderOpen, setEvidenceFolderOpenState] = useState(() => localStorage.getItem("sb_evidence_open") !== "false");
  const setEvidenceFolderOpen = (val: boolean) => {
    localStorage.setItem("sb_evidence_open", String(val));
    setEvidenceFolderOpenState(val);
  };

  const [intelligenceOpen, setIntelligenceOpenState] = useState(() => localStorage.getItem("sb_intel_open") !== "false");
  const setIntelligenceOpen = (val: boolean) => {
    localStorage.setItem("sb_intel_open", String(val));
    setIntelligenceOpenState(val);
  };

  const [emailFolderOpen, setEmailFolderOpenState] = useState(() => localStorage.getItem("sb_email_open") !== "false");
  const setEmailFolderOpen = (val: boolean) => {
    localStorage.setItem("sb_email_open", String(val));
    setEmailFolderOpenState(val);
  };

  const [investigationFolderOpen, setInvestigationFolderOpenState] = useState(() => localStorage.getItem("sb_invest_open") !== "false");
  const setInvestigationFolderOpen = (val: boolean) => {
    localStorage.setItem("sb_invest_open", String(val));
    setInvestigationFolderOpenState(val);
  };

  const [caseManagementOpen, setCaseManagementOpenState] = useState(() => localStorage.getItem("sb_manage_open") !== "false");
  const setCaseManagementOpen = (val: boolean) => {
    localStorage.setItem("sb_manage_open", String(val));
    setCaseManagementOpenState(val);
  };

  const [folderFilter, setFolderFilter] = useState<FolderFilter>("all");
  const [showDeleteCase, setShowDeleteCase] = useState(false);
  const [deleteConfirmText, setDeleteConfirmText] = useState("");
  const [deletingCase, setDeletingCase] = useState(false);

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
      if (ev.length === 0 && !localStorage.getItem(`last_view_${caseId}`)) {
        setView("evidence");
      }
    } catch (e) { console.error(e); }
    finally { setLoading(false); }
  }, [caseId]);

  useEffect(() => { loadAll(); }, [loadAll]);

  const handleDeleteCase = async () => {
    if (!caseId) return;
    setDeletingCase(true);
    try {
      await invoke<boolean>("case_delete", { input: { case_id: caseId } });
      onBack();
    } catch (e) {
      console.error("Failed to delete case:", e);
    } finally {
      setDeletingCase(false);
      setShowDeleteCase(false);
    }
  };

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
      important: dashboard?.important_count || 0,
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
            <J12Logo size={30} />
            <div>
              <div className="brand-title">{caseData?.title || "Case"}</div>
              <div className="brand-sub">{caseData?.case_number ? `Case #${caseData.case_number}` : "Investigation"}</div>
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
          <button
            className={`btn ${view === "evidence" ? "btn-primary" : "btn-ghost"} btn-sm`}
            onClick={() => setView("evidence")}
            style={{ fontSize: 12 }}
          >
            📥 {evidence.length} Evidence Source(s)
          </button>
        </div>
      </header>

      <div className="body">
         {/* Case Navigator Sidebar */}
         <nav className="sidebar" style={{ width: sidebarCollapsed ? 50 : 230, minWidth: sidebarCollapsed ? 50 : 230 }}>
           <div className="sb-section" style={{ display: "flex", justifyContent: "flex-end", padding: "6px 8px" }}>
             <button className="sb-toggle" onClick={() => setSidebarCollapsed(!sidebarCollapsed)} title={sidebarCollapsed ? "Expand Sidebar" : "Collapse Sidebar"}>
               {sidebarCollapsed ? "→" : "←"}
             </button>
           </div>

           {/* 1. Overview & Dossier */}
           {!sidebarCollapsed && (
             <div className="sb-folder">
               <div className="sb-folder-header" onClick={() => setOverviewOpen(!overviewOpen)}>
                 <span className="sb-folder-arrow">{overviewOpen ? "▼" : "▶"}</span>
                 <span className="sb-label" style={{ margin: 0 }}>Case Overview</span>
               </div>
               {overviewOpen && (
                 <div className="sb-folder-content">
                   <button className={`sb-item ${view === "dashboard" ? "active" : ""}`} onClick={() => setView("dashboard")} style={{ fontWeight: 600 }}>
                     <span className="sb-icon">◫</span> Case Dashboard
                   </button>
                   <button className={`sb-item ${view === "target" ? "active" : ""}`} onClick={() => setView("target")}>
                     <span className="sb-icon">🎯</span> Target Dossier
                   </button>
                 </div>
               )}
             </div>
           )}

           {/* 2. Evidence Sources & Acquisition */}
           <div className="sb-folder">
             <div
               className="sb-folder-header"
               style={{ display: "flex", alignItems: "center", justifyContent: "space-between", cursor: "pointer" }}
               onClick={() => {
                 setEvidenceFolderOpen(!evidenceFolderOpen);
               }}
             >
               <div style={{ display: "flex", alignItems: "center", gap: 6 }}>
                 <span className="sb-folder-arrow" style={{ fontSize: 10 }}>
                   {evidenceFolderOpen ? "▼" : "▶"}
                 </span>
                 <span className="sb-label" style={{ margin: 0, padding: 0 }}>Evidence Sources</span>
               </div>
               {!sidebarCollapsed && (
                 <button
                   className="btn btn-ghost btn-sm"
                   style={{ padding: "2px 6px", fontSize: 10, height: 20 }}
                   onClick={(e) => {
                     e.stopPropagation();
                     setView("evidence");
                   }}
                 >
                   + Add
                 </button>
               )}
             </div>
             {evidenceFolderOpen && !sidebarCollapsed && (
               <div className="sb-folder-content">
                 <button
                   className={`sb-item ${view === "evidence" ? "active" : ""}`}
                   onClick={() => setView("evidence")}
                 >
                   <span className="sb-icon">📥</span>
                   <span>Acquire / Ingest</span>
                   <span className="sb-count">{evidence.length}</span>
                 </button>
                 {evidence.map((e) => (
                   <button
                     key={e.id}
                     className={`sb-item ${view === "evidence" ? "active" : ""}`}
                     onClick={() => setView("evidence")}
                     style={{ paddingLeft: 20 }}
                     title={e.filename}
                   >
                     <span className="sb-icon">{e.format === "imap" ? "☁️" : e.format === "eml" ? "📧" : e.format === "mbox" ? "📦" : "📄"}</span>
                     <span className="sb-text-truncate">{e.filename}</span>
                     <span className={`sb-status sb-${e.parse_status}`}>{e.parse_status === "done" ? "✓" : e.parse_status === "error" ? "!" : "•"}</span>
                   </button>
                 ))}
               </div>
             )}
           </div>

           {/* 3. Forensic Intelligence */}
           {!sidebarCollapsed && (
             <div className="sb-folder">
               <div className="sb-folder-header" onClick={() => setIntelligenceOpen(!intelligenceOpen)}>
                 <span className="sb-folder-arrow">{intelligenceOpen ? "▼" : "▶"}</span>
                 <span className="sb-label" style={{ margin: 0 }}>Forensic Intelligence</span>
               </div>
               {intelligenceOpen && (
                 <div className="sb-folder-content">
                   <button className={`sb-item ${view === "artifacts" ? "active" : ""}`} onClick={() => setView("artifacts")}>
                     <span className="sb-icon">🧩</span> Artifacts Hub
                   </button>
                   <button className={`sb-item ${view === "attachments" ? "active" : ""}`} onClick={() => setView("attachments")}>
                     <span className="sb-icon">📎</span> Attachments &amp; Files
                   </button>
                   <button className={`sb-item ${view === "findings" ? "active" : ""}`} onClick={() => setView("findings")}>
                     <span className="sb-icon">🚨</span> Security Findings
                     {dashboard && dashboard.finding_count > 0 && <span className="sb-count" style={{ background: "rgba(239, 68, 68, 0.2)", color: "#ef4444" }}>{dashboard.finding_count}</span>}
                   </button>
                 </div>
               )}
             </div>
           )}

           {/* 4. Email Folders - Collapsible */}
           <div className="sb-folder">
             <div className="sb-folder-header" onClick={() => setEmailFolderOpen(!emailFolderOpen)}>
               <span className="sb-folder-arrow">{emailFolderOpen ? "▼" : "▶"}</span>
               <span className="sb-label" style={{ margin: 0 }}>Email Messages</span>
             </div>
             {emailFolderOpen && !sidebarCollapsed && (
               <div className="sb-folder-content">
                 <button className={`sb-item ${folderFilter === "all" && view === "emails" ? "active" : ""}`} onClick={() => { setFolderFilter("all"); setView("emails"); }} style={{ opacity: hasDone ? 1 : 0.4 }}>
                   <span className="sb-icon">📬</span> All Emails
                   <span className="sb-count">{emailCounts.total || 0}</span>
                 </button>
                 <button className={`sb-item ${folderFilter === "inbox" && (view === "inbox" || view === "emails") ? "active" : ""}`} onClick={() => { setFolderFilter("inbox"); setView("inbox"); }} style={{ opacity: hasDone ? 1 : 0.4 }}>
                   <span className="sb-icon">📥</span> Inbox
                   <span className="sb-count">{emailCounts.inbox || 0}</span>
                 </button>
                 <button className={`sb-item ${folderFilter === "important" && (view === "important" || view === "emails") ? "active" : ""}`} onClick={() => { setFolderFilter("important"); setView("important"); }} style={{ opacity: hasDone ? 1 : 0.4 }}>
                   <span className="sb-icon">⭐</span> Important
                   <span className="sb-count">{emailCounts.important || 0}</span>
                 </button>
                 <button className={`sb-item ${folderFilter === "sent" && (view === "sent" || view === "emails") ? "active" : ""}`} onClick={() => { setFolderFilter("sent"); setView("sent"); }} style={{ opacity: hasDone ? 1 : 0.4 }}>
                   <span className="sb-icon">📤</span> Sent
                   <span className="sb-count">{emailCounts.sent || 0}</span>
                 </button>
                 <button className={`sb-item ${folderFilter === "drafts" && (view === "drafts" || view === "emails") ? "active" : ""}`} onClick={() => { setFolderFilter("drafts"); setView("drafts"); }} style={{ opacity: hasDone ? 1 : 0.4 }}>
                   <span className="sb-icon">📝</span> Drafts
                   <span className="sb-count">{emailCounts.drafts || 0}</span>
                 </button>
                 <button className={`sb-item ${folderFilter === "soft_deleted" && (view === "soft_deleted" || view === "emails") ? "active" : ""}`} onClick={() => { setFolderFilter("soft_deleted"); setView("soft_deleted"); }} style={{ opacity: hasDone ? 1 : 0.4 }}>
                   <span className="sb-icon">🗑️</span> Deleted (Recycle Bin)
                   <span className="sb-count">{emailCounts.soft_deleted || 0}</span>
                 </button>
                 <button className={`sb-item ${folderFilter === "spam" && (view === "spam" || view === "emails") ? "active" : ""}`} onClick={() => { setFolderFilter("spam"); setView("spam"); }} style={{ opacity: hasDone ? 1 : 0.4 }}>
                   <span className="sb-icon">⚠</span> Spam / Junk
                   <span className="sb-count">{emailCounts.spam || 0}</span>
                 </button>
                 <button className={`sb-item ${folderFilter === "other" && (view === "other" || view === "emails") ? "active" : ""}`} onClick={() => { setFolderFilter("other"); setView("other"); }} style={{ opacity: hasDone ? 1 : 0.4 }}>
                   <span className="sb-icon">📁</span> Other Folders
                   <span className="sb-count">{emailCounts.other || 0}</span>
                 </button>
               </div>
             )}
           </div>

           {/* 5. Investigation & Analytics - Collapsible */}
           <div className="sb-folder">
             <div className="sb-folder-header" onClick={() => setInvestigationFolderOpen(!investigationFolderOpen)}>
               <span className="sb-folder-arrow">{investigationFolderOpen ? "▼" : "▶"}</span>
               <span className="sb-label" style={{ margin: 0 }}>Investigation &amp; Graph</span>
             </div>
             {investigationFolderOpen && !sidebarCollapsed && (
               <div className="sb-folder-content">
                 <button className={`sb-item ${view === "search" ? "active" : ""}`} onClick={() => setView("search")}>
                   <span className="sb-icon">🔍</span> Advanced Search
                 </button>
                 <button className={`sb-item ${view === "graph" ? "active" : ""}`} onClick={() => hasDone && setView("graph")} style={{ opacity: hasDone ? 1 : 0.4 }}>
                   <span className="sb-icon">🕸️</span> Network Graph
                 </button>
                 <button className={`sb-item ${view === "entities" ? "active" : ""}`} onClick={() => hasDone && setView("entities")} style={{ opacity: hasDone ? 1 : 0.4 }}>
                   <span className="sb-icon">👥</span> Entity Profiles
                 </button>
                 <button className={`sb-item ${view === "timeline" ? "active" : ""}`} onClick={() => hasDone && setView("timeline")} style={{ opacity: hasDone ? 1 : 0.4 }}>
                   <span className="sb-icon">📅</span> Incident Timeline
                 </button>
               </div>
             )}
           </div>

           {/* 6. Case Management & Integrity - Collapsible */}
           <div className="sb-folder">
             <div className="sb-folder-header" onClick={() => setCaseManagementOpen(!caseManagementOpen)}>
               <span className="sb-folder-arrow">{caseManagementOpen ? "▼" : "▶"}</span>
               <span className="sb-label" style={{ margin: 0 }}>Case Management</span>
             </div>
             {caseManagementOpen && !sidebarCollapsed && (
               <div className="sb-folder-content">
                 <button className={`sb-item ${view === "case_manage" ? "active" : ""}`} onClick={() => setView("case_manage")}>
                   <span className="sb-icon">⚙️</span> Manage Case &amp; Directory
                 </button>
                 <button className={`sb-item ${view === "custody" ? "active" : ""}`} onClick={() => setView("custody")}>
                   <span className="sb-icon">📋</span> Chain of Custody
                 </button>
                 <button className={`sb-item ${view === "integrity" ? "active" : ""}`} onClick={() => setView("integrity")}>
                   <span className="sb-icon">🔒</span> Verify Integrity &amp; Hashes
                 </button>
                 <button className={`sb-item ${view === "notes" ? "active" : ""}`} onClick={() => setView("notes")}>
                   <span className="sb-icon">📝</span> Case Notes
                   {notesCount > 0 && <span className="sb-count">{notesCount}</span>}
                 </button>
                 <button className={`sb-item ${view === "report" ? "active" : ""}`} onClick={() => setView("report")}>
                   <span className="sb-icon">📄</span> Generate Report
                 </button>
                 <button className="sb-item" style={{ color: "var(--red)" }} onClick={() => setShowDeleteCase(true)}>
                   <span className="sb-icon">🗑️</span> Delete Case
                 </button>
               </div>
             )}
           </div>
         </nav>

        {/* Main content area */}
        <main className="content">
          {view === "dashboard" && dashboard && (
            <DashboardView
              data={dashboard}
              evidence={evidence}
              caseData={caseData}
              caseId={caseId}
              onNavigate={(v) => setView(v)}
              onRefresh={loadAll}
            />
          )}
          {view === "evidence" && <EvidenceView evidence={evidence} caseId={caseId} onRefresh={loadAll} />}
           {view === "emails" && <EmailListView caseId={caseId} filter={folderFilter} onViewEntity={(email) => setView("entities")} />}
           {view === "sent" && <EmailListView caseId={caseId} filter="sent" onViewEntity={(email) => setView("entities")} />}
           {view === "inbox" && <EmailListView caseId={caseId} filter="inbox" onViewEntity={(email) => setView("entities")} />}
           {view === "important" && <EmailListView caseId={caseId} filter="important" onViewEntity={(email) => setView("entities")} />}
           {view === "drafts" && <EmailListView caseId={caseId} filter="drafts" onViewEntity={(email) => setView("entities")} />}
           {view === "soft_deleted" && <EmailListView caseId={caseId} filter="soft_deleted" onViewEntity={(email) => setView("entities")} />}
           {view === "hard_deleted" && <EmailListView caseId={caseId} filter="hard_deleted" onViewEntity={(email) => setView("entities")} />}
           {view === "recoverable" && <EmailListView caseId={caseId} filter="recoverable" onViewEntity={(email) => setView("entities")} />}
           {view === "spam" && <EmailListView caseId={caseId} filter="spam" onViewEntity={(email) => setView("entities")} />}
            {view === "other" && <EmailListView caseId={caseId} filter="other" onViewEntity={(email) => setView("entities")} />}
           {view === "search" && <SearchView caseId={caseId} onViewEntity={(email) => { setView("entities"); }} />}
           {view === "entities" && <EntityDiveView caseId={caseId} />}
           {view === "timeline" && <TimelineView caseId={caseId} />}
           {view === "graph" && <GraphView caseId={caseId} />}
           {view === "findings" && <FindingsView caseId={caseId} onGoToEvidence={() => setView("evidence")} />}
           {view === "artifacts" && <ArtifactsView caseId={caseId} onSelectEmail={() => { setView("emails"); }} />}
           {view === "attachments" && <AttachmentsView caseId={caseId} onSelectEmail={() => { setView("emails"); }} />}
           {view === "target" && <TargetProfileView caseId={caseId} caseData={caseData} />}
            {view === "custody" && <CustodyView evidence={evidence} caseId={caseId} />}
            {view === "notes" && <NotesView caseId={caseId} onNotesCountChange={setNotesCount} />}
            {view === "case_manage" && <CaseManageView caseData={caseData} caseId={caseId} onUpdate={loadAll} onBack={() => setView("dashboard")} />}
             {view === "report" && <ReportView caseId={caseId} caseData={caseData} />}
             {view === "integrity" && <IntegrityView caseId={caseId} />}
         </main>
       </div>

       {/* Delete Case Confirmation Modal */}
       {showDeleteCase && (
         <div style={{ position: "fixed", inset: 0, background: "rgba(0,0,0,0.75)", backdropFilter: "blur(4px)", display: "flex", alignItems: "center", justifyContent: "center", zIndex: 10000 }}>
           <div className="card" style={{ maxWidth: 460, width: "90%", padding: 24, border: "1px solid rgba(239, 68, 68, 0.4)", boxShadow: "0 20px 50px rgba(0,0,0,0.7)" }}>
             <div style={{ display: "flex", alignItems: "center", gap: 12, marginBottom: 16 }}>
               <span style={{ fontSize: 32 }}>⚠️</span>
               <div>
                 <h3 style={{ fontSize: 18, fontWeight: 700, color: "var(--danger)", margin: 0 }}>Delete Case</h3>
                 <p className="muted" style={{ fontSize: 12, margin: "4px 0 0" }}>Permanent &amp; Irreversible Destruction</p>
               </div>
             </div>
             <p style={{ fontSize: 13, color: "var(--text-1)", marginBottom: 12, lineHeight: 1.6 }}>
               Are you sure you want to delete case <strong>"{caseData?.title}"</strong>? This will permanently erase:
             </p>
             <ul style={{ fontSize: 12, color: "var(--text-2)", marginBottom: 16, paddingLeft: 20, lineHeight: 1.8 }}>
               <li>All evidence sources ({evidence.length} files)</li>
               <li>All parsed emails ({dashboard?.email_count?.toLocaleString() || 0} messages)</li>
               <li>All extracted forensic artifacts and security findings</li>
               <li>Chain of custody and audit records</li>
             </ul>

             <div style={{ marginBottom: 18 }}>
               <label className="label" style={{ color: "var(--danger)", fontWeight: 700, fontSize: 11 }}>
                 Type <span style={{ textDecoration: "underline" }}>DELETE</span> to confirm:
               </label>
               <input
                 className="input"
                 style={{ borderColor: deleteConfirmText === "DELETE" ? "var(--danger)" : "var(--border)", fontWeight: 700, letterSpacing: "0.08em" }}
                 placeholder="Type DELETE"
                 value={deleteConfirmText}
                 onChange={e => setDeleteConfirmText(e.target.value)}
                 autoFocus
               />
             </div>

             <div className="row gap-2" style={{ justifyContent: "flex-end" }}>
               <button className="btn btn-ghost" onClick={() => { setShowDeleteCase(false); setDeleteConfirmText(""); }} disabled={deletingCase}>
                 Cancel
               </button>
               <button 
                 className="btn btn-danger" 
                 style={{ background: "#dc2626", color: "#fff", fontWeight: 700 }} 
                 onClick={handleDeleteCase} 
                 disabled={deletingCase || deleteConfirmText.trim() !== "DELETE"}
               >
                 {deletingCase ? "Deleting Case..." : "Delete Case Permanently"}
               </button>
             </div>
           </div>
         </div>
       )}
     </div>
   );
}

function DashboardView({
  data,
  evidence,
  caseData,
  caseId,
  onNavigate,
  onRefresh,
}: {
  data: Dashboard;
  evidence: Evidence[];
  caseData: Case | null;
  caseId: string;
  onNavigate: (view: View) => void;
  onRefresh: () => void;
}) {
  const [criticalFindings, setCriticalFindings] = useState<any[]>([]);
  const [targetPartners, setTargetPartners] = useState<any[]>([]);
  const [analyzing, setAnalyzing] = useState(false);

  useEffect(() => {
    // Load top critical findings
    invoke<any[]>("findings_list", { input: { case_id: caseId } })
      .then((res) => {
        const critical = (res || []).filter((f) => f.severity === "critical" || f.severity === "high");
        setCriticalFindings(critical.slice(0, 3));
      })
      .catch(() => setCriticalFindings([]));

    // Load target partners if target_email exists
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

      {/* Interactive 5-Metric Command Center Cards (Clickable) */}
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

          {/* Top Correspondents for Target */}
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
          <h3 style={{ fontSize: 15, fontWeight: 600, marginBottom: 16 }}>Evidence Containers & Provenance</h3>
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

function EvidenceView({ evidence, caseId, onRefresh }: { evidence: Evidence[]; caseId: string; onRefresh: () => void }) {
  const [uploading, setUploading] = useState(false);
  const [logs, setLogs] = useState<any[]>([]);
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [dragOver, setDragOver] = useState(false);
  const [acqMethod, setAcqMethod] = useState<"file" | "server" | "client" | "imaging">("file");

  const addLog = (level: string, message: string) => {
    setLogs(prev => [...prev, { time: new Date().toLocaleTimeString(), level, message }]);
  };

  const handleUpload = async () => {
    try {
      const selected = await invoke<string | null>("open_file_dialog");
      if (!selected) return;
      processFile(selected);
    } catch (e: any) { addLog("error", `Upload failed: ${e}`); }
  };

  const processFile = async (path: string) => {
    setUploading(true);
    addLog("info", `Uploading: ${path}`);
    try {
      const ev = await invoke<any>("evidence_upload", { input: { case_id: caseId, file_path: path, source_description: null } });
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

  const handleDrop = async (e: React.DragEvent) => {
    e.preventDefault();
    setDragOver(false);
    addLog("info", "Please use the upload button to select files (browser security restricts drag-drop paths)");
  };

  const handleDragOver = (e: React.DragEvent) => {
    e.preventDefault();
    setDragOver(true);
  };

  const handleDragLeave = () => {
    setDragOver(false);
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

  const [deletingId, setDeletingId] = useState<string | null>(null);
  const [confirmDeleteModal, setConfirmDeleteModal] = useState<{ id: string; filename: string } | null>(null);
  const [deleteEvidenceConfirmText, setDeleteEvidenceConfirmText] = useState("");

  const handleDeleteEvidence = async (evidenceId: string, filename: string) => {
    setDeletingId(evidenceId);
    try {
      await invoke("evidence_delete", { input: { evidence_id: evidenceId } });
      addLog("success", `Deleted evidence source "${filename}" and its associated emails.`);
      if (selectedId === evidenceId) setSelectedId(null);
      setConfirmDeleteModal(null);
      setDeleteEvidenceConfirmText("");
      onRefresh();
    } catch (e: any) {
      addLog("error", `Failed to delete evidence: ${e}`);
    } finally {
      setDeletingId(null);
    }
  };

  const selectedEvidence = selectedId ? evidence.find(e => e.id === selectedId) : null;

  return (
    <div>
      {/* Confirmation Modal for Evidence Deletion */}
      {confirmDeleteModal && (
        <div
          style={{
            position: "fixed",
            inset: 0,
            background: "rgba(0,0,0,0.75)",
            backdropFilter: "blur(4px)",
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
            zIndex: 9999,
          }}
          onClick={() => { setConfirmDeleteModal(null); setDeleteEvidenceConfirmText(""); }}
        >
          <div
            className="card"
            style={{
              maxWidth: 480,
              width: "92%",
              padding: 24,
              border: "1px solid rgba(239, 68, 68, 0.4)",
              boxShadow: "0 20px 40px rgba(0,0,0,0.6)",
              background: "var(--bg-1)",
            }}
            onClick={(e) => e.stopPropagation()}
          >
            <div style={{ display: "flex", alignItems: "center", gap: 12, marginBottom: 16 }}>
              <span style={{ fontSize: 32 }}>⚠️</span>
              <div>
                <h3 style={{ fontSize: 16, fontWeight: 700, color: "var(--danger)", margin: 0 }}>
                  Delete Evidence Source?
                </h3>
                <p className="muted" style={{ fontSize: 12, margin: "4px 0 0" }}>
                  Irreversible Forensic Action
                </p>
              </div>
            </div>
            <p style={{ fontSize: 13, lineHeight: 1.5, color: "var(--text-1)", marginBottom: 16 }}>
              Are you sure you want to permanently delete <strong>"{confirmDeleteModal.filename}"</strong>? 
              This will remove all associated emails, extracted attachments, and chain-of-custody records for this container.
            </p>

            <div style={{ marginBottom: 18 }}>
              <label className="label" style={{ color: "var(--danger)", fontWeight: 700, fontSize: 11 }}>
                Type <span style={{ textDecoration: "underline" }}>DELETE</span> to confirm:
              </label>
              <input
                className="input"
                style={{ borderColor: deleteEvidenceConfirmText === "DELETE" ? "var(--danger)" : "var(--border)", fontWeight: 700, letterSpacing: "0.08em" }}
                placeholder="Type DELETE"
                value={deleteEvidenceConfirmText}
                onChange={e => setDeleteEvidenceConfirmText(e.target.value)}
                autoFocus
              />
            </div>

            <div style={{ display: "flex", justifyContent: "flex-end", gap: 10 }}>
              <button className="btn btn-ghost" onClick={() => { setConfirmDeleteModal(null); setDeleteEvidenceConfirmText(""); }}>
                Cancel
              </button>
              <button
                className="btn btn-danger"
                style={{ background: "#dc2626", color: "#fff", fontWeight: 700 }}
                onClick={() => handleDeleteEvidence(confirmDeleteModal.id, confirmDeleteModal.filename)}
                disabled={deletingId !== null || deleteEvidenceConfirmText.trim() !== "DELETE"}
              >
                {deletingId ? "Deleting..." : "Delete Evidence Source"}
              </button>
            </div>
          </div>
        </div>
      )}

      <div className="row between mb-4">
        <div>
          <h2 style={{ fontSize: 22, fontWeight: 700, color: "var(--text-0)" }}>Evidence Acquisition</h2>
          <p className="muted">Import evidence into this case</p>
        </div>
      </div>

      {/* Acquisition Method Tabs */}
      <div className="card mb-4">
        <div className="row gap-2 mb-4" style={{ borderBottom: "1px solid var(--border)", paddingBottom: 0 }}>
          {[
            { key: "file", label: "📁 File Import", desc: "EML, MBOX, PST, MSG", active: true },
            { key: "server", label: "☁️ Mail Server", desc: "IMAP, OAuth, Cloud", active: false },
            { key: "client", label: "💻 Mail Client", desc: "Outlook, Apple Mail, Thunderbird", active: false },
            { key: "imaging", label: "💾 Forensic Imaging", desc: "Disk, Device, E01", active: false },
          ].map(tab => (
            <button
              key={tab.key}
              className={`btn btn-sm ${acqMethod === tab.key ? "btn-primary" : "btn-ghost"}`}
              style={{ borderRadius: "6px 6px 0 0", display: "flex", flexDirection: "column", alignItems: "center", padding: "8px 14px", opacity: tab.active ? 1 : 0.7 }}
              onClick={() => setAcqMethod(tab.key as any)}
            >
              <span style={{ fontSize: 12, fontWeight: 600 }}>{tab.label}</span>
              <span style={{ fontSize: 10, opacity: 0.7 }}>{tab.desc}</span>
              {!tab.active && <span style={{ fontSize: 9, color: "var(--text-3)" }}>Coming Soon</span>}
            </button>
          ))}
        </div>

        {/* 1. File Import Method (ACTIVE) */}
        {acqMethod === "file" && (
          <div>
            <div
              style={{
                textAlign: "center",
                padding: "40px 20px",
                border: dragOver ? "2px dashed var(--accent)" : "2px dashed var(--border)",
                borderRadius: "var(--r-md)",
                background: dragOver ? "var(--accent-subtle)" : "transparent",
              }}
              onDragOver={handleDragOver}
              onDragLeave={handleDragLeave}
              onDrop={handleDrop}
            >
              <div style={{ fontSize: 40, marginBottom: 12 }}>📧</div>
              <h4 style={{ fontSize: 15, fontWeight: 600, marginBottom: 6 }}>Upload Email Files</h4>
              <p className="muted mb-4" style={{ fontSize: 12 }}>Supports: EML, MBOX, PST, OST, MSG, EMLX</p>
              <button className="btn btn-primary" onClick={handleUpload} disabled={uploading}>
                {uploading ? "Uploading..." : "+ Select Files"}
              </button>
            </div>
          </div>
        )}

        {/* 2. Direct Mail Server / Cloud (Coming Soon) */}
        {/* 2. Mail Server Acquisition (IMAP) */}
        {acqMethod === "server" && (
          <ImapAcquisition caseId={caseId} onComplete={onRefresh} />
        )}

        {/* 3. Local Mail Client Extraction (Coming Soon) */}
        {acqMethod === "client" && (
          <div style={{ textAlign: "center", padding: "40px 20px" }}>
            <div style={{ fontSize: 40, marginBottom: 12 }}>💻</div>
            <h4 style={{ fontSize: 15, fontWeight: 600, marginBottom: 6 }}>Local Mail Client Extraction</h4>
            <p className="muted mb-4" style={{ fontSize: 12 }}>
              Auto-detect and extract from installed mail applications on this computer
            </p>
            <div className="row gap-2" style={{ justifyContent: "center", flexWrap: "wrap" }}>
              <div className="card" style={{ padding: 16, minWidth: 180 }}>
                <div style={{ fontSize: 24, marginBottom: 8 }}>📮</div>
                <h5 style={{ fontSize: 13, fontWeight: 600, marginBottom: 4 }}>Apple Mail</h5>
                <p className="muted" style={{ fontSize: 11 }}>macOS native mail app</p>
                <span className="badge badge-gray mt-2">Coming Soon</span>
              </div>
              <div className="card" style={{ padding: 16, minWidth: 180 }}>
                <div style={{ fontSize: 24, marginBottom: 8 }}>🔷</div>
                <h5 style={{ fontSize: 13, fontWeight: 600, marginBottom: 4 }}>Outlook</h5>
                <p className="muted" style={{ fontSize: 11 }}>Mac/Windows, OST/PST</p>
                <span className="badge badge-gray mt-2">Coming Soon</span>
              </div>
              <div className="card" style={{ padding: 16, minWidth: 180 }}>
                <div style={{ fontSize: 24, marginBottom: 8 }}>🦅</div>
                <h5 style={{ fontSize: 13, fontWeight: 600, marginBottom: 4 }}>Thunderbird</h5>
                <p className="muted" style={{ fontSize: 11 }}>MBOX-based storage</p>
                <span className="badge badge-gray mt-2">Coming Soon</span>
              </div>
            </div>
          </div>
        )}

        {/* 4. Forensic Imaging (Coming Soon) */}
        {acqMethod === "imaging" && (
          <div style={{ textAlign: "center", padding: "40px 20px" }}>
            <div style={{ fontSize: 40, marginBottom: 12 }}>💾</div>
            <h4 style={{ fontSize: 15, fontWeight: 600, marginBottom: 6 }}>Forensic Physical & Logical Imaging</h4>
            <p className="muted mb-4" style={{ fontSize: 12 }}>
              Extract email stores from physical drives, device dumps, and E01 forensic images
            </p>
            <span className="badge badge-gray">Coming Soon</span>
          </div>
        )}
      </div>

      {/* Activity Log */}
      {logs.length > 0 && (
        <div className="card mb-4" style={{ maxHeight: 150, overflowY: "auto", fontFamily: "monospace", fontSize: 12 }}>
          <div className="row between mb-4">
            <h4 style={{ fontSize: 12, fontWeight: 600 }}>Activity Log</h4>
            <button className="btn btn-ghost btn-sm" onClick={() => setLogs([])}>Clear</button>
          </div>
          {logs.map((log, i) => (
            <div key={i} className={`log-${log.level}`} style={{ padding: "2px 0" }}>
              <span className="muted">[{log.time}]</span>{" "}
              <span className={`badge badge-${log.level === "error" ? "red" : log.level === "success" ? "green" : "blue"}`}>{log.level}</span>
              <span>{log.message}</span>
            </div>
          ))}
        </div>
      )}

      {/* Evidence List */}
      {evidence.length > 0 && (
        <div className="card">
          <h3 style={{ fontSize: 15, fontWeight: 600, marginBottom: 16 }}>Evidence Sources ({evidence.length})</h3>
          <table>
            <thead>
              <tr>
                <th className="th">File</th>
                <th className="th">Format</th>
                <th className="th">Size</th>
                <th className="th">Status</th>
                <th className="th">Messages</th>
                <th className="th">SHA-256</th>
                <th className="th" style={{ textAlign: "right" }}>Actions</th>
              </tr>
            </thead>
            <tbody>
              {evidence.map((e) => (
                <tr key={e.id} onClick={() => setSelectedId(selectedId === e.id ? null : e.id)} className="tr-click" style={{ background: selectedId === e.id ? "var(--bg-3)" : "transparent" }}>
                  <td className="td" style={{ fontWeight: 600 }}>{e.filename}</td>
                  <td className="td"><span className={`badge badge-${e.format === "eml" ? "blue" : e.format === "mbox" ? "green" : e.format === "imap" ? "purple" : "orange"}`}>{e.format}</span></td>
                  <td className="td muted">{(e.size_bytes / 1024).toFixed(0)} KB</td>
                  <td className="td"><span className={`badge ${e.parse_status === "done" ? "badge-green" : e.parse_status === "error" ? "badge-red" : e.parse_status === "parsing" || e.parse_status === "ingesting" ? "badge-blue" : "badge-gray"}`}>{e.parse_status}</span></td>
                  <td className="td">{e.message_count}</td>
                  <td className="td mono muted">{e.sha256 ? `${e.sha256.slice(0, 12)}…` : "—"}</td>
                  <td className="td" style={{ textAlign: "right" }}>
                    <div style={{ display: "inline-flex", gap: 6, alignItems: "center" }}>
                      {e.parse_status === "pending" && <button className="btn btn-primary btn-sm" onClick={(ev) => { ev.stopPropagation(); handleParse(e.id, e.filename); }}>Parse</button>}
                      {e.parse_status === "parsing" && <span className="muted text-sm">Parsing...</span>}
                      {e.parse_status === "error" && <button className="btn btn-ghost btn-sm" onClick={(ev) => { ev.stopPropagation(); handleParse(e.id, e.filename); }}>Retry</button>}
                      <button
                        className="btn btn-danger btn-sm"
                        style={{ padding: "4px 8px", fontSize: 12, background: "rgba(239, 68, 68, 0.15)", color: "#ef4444", border: "1px solid rgba(239, 68, 68, 0.3)" }}
                        title={`Delete evidence source: ${e.filename}`}
                        onClick={(ev) => {
                          ev.stopPropagation();
                          setConfirmDeleteModal({ id: e.id, filename: e.filename });
                        }}
                        disabled={deletingId === e.id}
                      >
                        {deletingId === e.id ? "Deleting..." : "🗑️ Delete"}
                      </button>
                    </div>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}

      {/* Selected Evidence Details */}
      {selectedEvidence && (
        <div className="card mt-4">
          <div className="row between mb-4">
            <h4 style={{ fontSize: 14, fontWeight: 600 }}>Evidence Details</h4>
            <div style={{ display: "flex", gap: 8 }}>
              <button
                className="btn btn-danger btn-sm"
                style={{ padding: "4px 10px", fontSize: 12, background: "rgba(239, 68, 68, 0.15)", color: "#ef4444", border: "1px solid rgba(239, 68, 68, 0.3)" }}
                onClick={() => setConfirmDeleteModal({ id: selectedEvidence.id, filename: selectedEvidence.filename })}
              >
                🗑️ Delete Evidence Source
              </button>
              <button className="btn btn-ghost btn-sm" onClick={() => setSelectedId(null)}>Close</button>
            </div>
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

function CaseManageView({ caseData, caseId, onUpdate, onBack }: { caseData: Case | null; caseId: string; onUpdate: () => void; onBack: () => void }) {
  const [title, setTitle] = useState(caseData?.title || "");
  const [description, setDescription] = useState(caseData?.description || "");
  const [status, setStatus] = useState(caseData?.status || "open");
  const [targetName, setTargetName] = useState(caseData?.target_name || "");
  const [targetEmail, setTargetEmail] = useState(caseData?.target_email || "");
  const [targetOrg, setTargetOrg] = useState(caseData?.target_organization || "");
  const [saving, setSaving] = useState(false);
  const [showDelete, setShowDelete] = useState(false);
  const [deleteConfirmText, setDeleteConfirmText] = useState("");
  const [deleting, setDeleting] = useState(false);

  const handleSave = async () => {
    setSaving(true);
    try {
      await invoke("case_update", {
        input: {
          case_id: caseId,
          title,
          description,
          status,
          target_name: targetName,
          target_email: targetEmail,
          target_organization: targetOrg,
        }
      });
      onUpdate();
    } catch (e) {
      console.error("Failed to update case:", e);
    } finally {
      setSaving(false);
    }
  };

  const handleDelete = async () => {
    setDeleting(true);
    try {
      await invoke("case_delete", { input: { case_id: caseId } });
      onBack();
    } catch (e) {
      console.error("Failed to delete case:", e);
    } finally {
      setDeleting(false);
    }
  };

  return (
    <div>
      <div className="row between mb-4">
        <div>
          <h2 style={{ fontSize: 22, fontWeight: 700, color: "var(--text-0)" }}>Manage Case</h2>
          <p className="muted">Edit case details or delete the case</p>
        </div>
        <button className="btn btn-ghost btn-sm" onClick={onBack}>← Back to Dashboard</button>
      </div>

      <div className="card mb-4">
        <h3 style={{ fontSize: 15, fontWeight: 600, marginBottom: 16 }}>Case Information</h3>
        <div className="field">
          <label className="label">Case ID (not editable)</label>
          <input className="input" value={caseId} disabled style={{ opacity: 0.6 }} />
        </div>
        <div className="field">
          <label className="label">Case Number</label>
          <input className="input" value={caseData?.case_number || "—"} disabled style={{ opacity: 0.6 }} />
        </div>
        <div className="field">
          <label className="label">Title</label>
          <input className="input" value={title} onChange={(e) => setTitle(e.target.value)} />
        </div>
        <div className="field">
          <label className="label">Description</label>
          <textarea className="textarea" value={description} onChange={(e) => setDescription(e.target.value)} rows={3} />
        </div>
        <div className="field">
          <label className="label">Status</label>
          <select className="input" value={status} onChange={(e) => setStatus(e.target.value)}>
            <option value="open">Open</option>
            <option value="closed">Closed</option>
            <option value="archived">Archived</option>
          </select>
        </div>
      </div>

      <div className="card mb-4">
        <h3 style={{ fontSize: 15, fontWeight: 600, marginBottom: 16 }}>Investigation Target</h3>
        <div className="field">
          <label className="label">Target Name</label>
          <input className="input" value={targetName} onChange={(e) => setTargetName(e.target.value)} placeholder="e.g. John Doe" />
        </div>
        <div className="field">
          <label className="label">Target Email</label>
          <input className="input" value={targetEmail} onChange={(e) => setTargetEmail(e.target.value)} placeholder="e.g. john@example.com" />
        </div>
        <div className="field">
          <label className="label">Target Organization</label>
          <input className="input" value={targetOrg} onChange={(e) => setTargetOrg(e.target.value)} placeholder="e.g. Acme Corp" />
        </div>
      </div>

      <div className="row gap-2">
        <button className="btn btn-primary" onClick={handleSave} disabled={saving}>
          {saving ? "Saving..." : "Save Changes"}
        </button>
        <button className="btn btn-ghost" onClick={onBack}>Cancel</button>
      </div>

      <div className="card mt-4" style={{ borderColor: "var(--red)", border: "1px solid var(--red)" }}>
        <h3 style={{ fontSize: 15, fontWeight: 600, color: "var(--red)", marginBottom: 12 }}>Danger Zone</h3>
        <p style={{ fontSize: 13, color: "var(--text-2)", marginBottom: 16 }}>
          Deleting this case will permanently remove all evidence, emails, findings, and chain of custody records.
        </p>
        <button className="btn" style={{ background: "var(--red)", color: "#fff" }} onClick={() => setShowDelete(true)}>
          Delete Case & All Data
        </button>
      </div>

      {showDelete && (
        <div style={{ position: "fixed", inset: 0, background: "rgba(0,0,0,0.75)", backdropFilter: "blur(4px)", display: "flex", alignItems: "center", justifyContent: "center", zIndex: 10000 }}>
          <div className="card" style={{ maxWidth: 460, width: "90%", padding: 24, border: "1px solid rgba(239, 68, 68, 0.4)", boxShadow: "0 20px 50px rgba(0,0,0,0.7)" }}>
            <div style={{ display: "flex", alignItems: "center", gap: 12, marginBottom: 16 }}>
              <span style={{ fontSize: 32 }}>⚠️</span>
              <div>
                <h3 style={{ fontSize: 18, fontWeight: 700, color: "var(--danger)", margin: 0 }}>Delete Case</h3>
                <p className="muted" style={{ fontSize: 12, margin: "4px 0 0" }}>Permanent &amp; Irreversible Destruction</p>
              </div>
            </div>
            <p style={{ fontSize: 13, color: "var(--text-1)", marginBottom: 12, lineHeight: 1.6 }}>
              Are you sure you want to delete case <strong>"{caseData?.title}"</strong>? This will permanently remove:
            </p>
            <ul style={{ fontSize: 12, color: "var(--text-2)", marginBottom: 16, paddingLeft: 20, lineHeight: 1.8 }}>
              <li>All evidence sources</li>
              <li>All parsed emails</li>
              <li>All findings, artifacts, and analysis results</li>
              <li>Chain of custody records</li>
            </ul>

            <div style={{ marginBottom: 18 }}>
              <label className="label" style={{ color: "var(--danger)", fontWeight: 700, fontSize: 11 }}>
                Type <span style={{ textDecoration: "underline" }}>DELETE</span> to confirm:
              </label>
              <input
                className="input"
                style={{ borderColor: deleteConfirmText === "DELETE" ? "var(--danger)" : "var(--border)", fontWeight: 700, letterSpacing: "0.08em" }}
                placeholder="Type DELETE"
                value={deleteConfirmText}
                onChange={e => setDeleteConfirmText(e.target.value)}
                autoFocus
              />
            </div>

            <div className="row gap-2" style={{ justifyContent: "flex-end" }}>
              <button className="btn btn-ghost" onClick={() => { setShowDelete(false); setDeleteConfirmText(""); }} disabled={deleting}>
                Cancel
              </button>
              <button 
                className="btn btn-danger" 
                style={{ background: "#dc2626", color: "#fff", fontWeight: 700 }} 
                onClick={handleDelete} 
                disabled={deleting || deleteConfirmText.trim() !== "DELETE"}
              >
                {deleting ? "Deleting..." : "Delete Permanently"}
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}

function IntegrityView({ caseId }: { caseId: string }) {
  const [verification, setVerification] = useState<any>(null);
  const [chainCheck, setChainCheck] = useState<any>(null);
  const [loading, setLoading] = useState(false);

  const verifyHashes = async () => {
    setLoading(true);
    try {
      const result = await invoke<any>("verify_evidence_hashes", { input: { case_id: caseId } });
      setVerification(result);
    } catch (e) { console.error(e); }
    setLoading(false);
  };

  const checkChain = async () => {
    setLoading(true);
    try {
      const result = await invoke<any>("check_custody_chain", { input: { case_id: caseId } });
      setChainCheck(result);
    } catch (e) { console.error(e); }
    setLoading(false);
  };

  const exportAudit = async () => {
    try {
      const path = await invoke<string>("export_audit_log", { input: { case_id: caseId } });
      alert(`Audit log exported to: ${path}`);
    } catch (e) { console.error(e); }
  };

  return (
    <div>
      <h2 style={{ fontSize: 22, fontWeight: 700, color: "var(--text-0)", marginBottom: 16 }}>Integrity & Verification</h2>
      <p className="muted mb-4">Verify evidence integrity and export audit logs</p>

      <div className="row gap-2 mb-4">
        <button className="btn btn-primary" onClick={verifyHashes} disabled={loading}>🔍 Verify Evidence Hashes</button>
        <button className="btn btn-ghost" onClick={checkChain} disabled={loading}>🔗 Check Custody Chain</button>
        <button className="btn btn-ghost" onClick={exportAudit}>📥 Export Audit Log</button>
      </div>

      {verification && (
        <div className="card mb-4">
          <h3 style={{ fontSize: 15, fontWeight: 600, marginBottom: 16 }}>Hash Verification Results</h3>
          <div className="grid-3 mb-4">
            <div className="card" style={{ textAlign: "center", padding: 16 }}>
              <div style={{ fontSize: 24, fontWeight: 700, color: "var(--success)" }}>{verification.verified}</div>
              <div className="muted">Verified</div>
            </div>
            <div className="card" style={{ textAlign: "center", padding: 16 }}>
              <div style={{ fontSize: 24, fontWeight: 700, color: "var(--red)" }}>{verification.failed}</div>
              <div className="muted">Modified</div>
            </div>
            <div className="card" style={{ textAlign: "center", padding: 16 }}>
              <div style={{ fontSize: 24, fontWeight: 700, color: "var(--text-2)" }}>{verification.missing}</div>
              <div className="muted">Missing</div>
            </div>
          </div>
          <table>
            <thead><tr><th>Filename</th><th>Status</th></tr></thead>
            <tbody>
              {verification.results.map((r: any, i: number) => (
                <tr key={i}>
                  <td>{r.filename}</td>
                  <td><span className={`badge ${r.status === "verified" ? "badge-green" : r.status === "modified" ? "badge-red" : "badge-gray"}`}>{r.status}</span></td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}

      {chainCheck && (
        <div className="card">
          <h3 style={{ fontSize: 15, fontWeight: 600, marginBottom: 16 }}>Custody Chain Check</h3>
          <p>Chain Intact: <span className={`badge ${chainCheck.chain_intact ? "badge-green" : "badge-red"}`}>{chainCheck.chain_intact ? "YES" : "NO"}</span></p>
          {chainCheck.gaps.length > 0 && (
            <div style={{ marginTop: 12 }}>
              <strong>Gaps Found:</strong>
              <ul style={{ paddingLeft: 20, marginTop: 8 }}>
                 {chainCheck.gaps.map((g: any, i: number) => (
                  <li key={i}>{g.evidence}: {g.issue}</li>
                ))}
              </ul>
            </div>
          )}
        </div>
      )}
    </div>
  );
}

function ImapAcquisition({ caseId, onComplete }: { caseId: string; onComplete: () => void }) {
  const getSaved = () => {
    try {
      return JSON.parse(localStorage.getItem(`imap_creds_${caseId}`) || "{}");
    } catch { return {}; }
  };
  const saved = getSaved();

  const [username, setUsername] = useState(saved.username || "");
  const [password, setPassword] = useState(saved.password || "");
  const [server, setServer] = useState(saved.server || "imap.gmail.com");
  const [port, setPort] = useState(saved.port || "993");
  const [useSsl, setUseSsl] = useState(saved.useSsl !== undefined ? saved.useSsl : true);
  const [mailboxScope, setMailboxScope] = useState(saved.mailboxScope || "ALL");
  const [showAdvanced, setShowAdvanced] = useState(false);
  const [mailboxes, setMailboxes] = useState<string[]>(saved.mailboxes || []);
  const [connecting, setConnecting] = useState(false);
  const [fetching, setFetching] = useState(false);
  const [result, setResult] = useState<any>(saved.result || null);
  const [logs, setLogs] = useState<string[]>(saved.logs || []);
  const [progress, setProgress] = useState<{
    folder?: string;
    folderIndex?: number;
    totalFolders?: number;
    msgSeq?: number;
    folderTotal?: number;
    ingested?: number;
    duplicatesSkipped?: number;
    subject?: string;
    from?: string;
  } | null>(null);

  const logsEndRef = useRef<HTMLDivElement | null>(null);

  useEffect(() => {
    localStorage.setItem(`imap_creds_${caseId}`, JSON.stringify({
      username, password, server, port, useSsl, mailboxScope, mailboxes, result, logs
    }));
  }, [caseId, username, password, server, port, useSsl, mailboxScope, mailboxes, result, logs]);

  useEffect(() => {
    logsEndRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [logs]);

  // Automatically detect IMAP server settings based on email domain
  const handleEmailChange = (val: string) => {
    setUsername(val);
    const domain = val.includes("@") ? val.split("@")[1].toLowerCase().trim() : "";
    if (domain.includes("gmail") || domain.includes("googlemail")) {
      setServer("imap.gmail.com");
      setPort("993");
      setUseSsl(true);
    } else if (domain.includes("outlook") || domain.includes("hotmail") || domain.includes("live.com") || domain.includes("office365")) {
      setServer("outlook.office365.com");
      setPort("993");
      setUseSsl(true);
    } else if (domain.includes("yahoo") || domain.includes("ymail") || domain.includes("rocketmail")) {
      setServer("imap.mail.yahoo.com");
      setPort("993");
      setUseSsl(true);
    } else if (domain.includes("icloud") || domain.includes("me.com") || domain.includes("mac.com")) {
      setServer("imap.mail.me.com");
      setPort("993");
      setUseSsl(true);
    } else if (domain.includes("zoho")) {
      setServer("imap.zoho.com");
      setPort("993");
      setUseSsl(true);
    } else if (domain.includes("aol.com")) {
      setServer("imap.aol.com");
      setPort("993");
      setUseSsl(true);
    } else if (domain.includes("fastmail")) {
      setServer("imap.fastmail.com");
      setPort("993");
      setUseSsl(true);
    } else if (domain.includes("gmx")) {
      setServer("imap.gmx.com");
      setPort("993");
      setUseSsl(true);
    } else if (domain.includes("mail.com")) {
      setServer("imap.mail.com");
      setPort("993");
      setUseSsl(true);
    } else if (domain.includes("proton")) {
      setServer("127.0.0.1");
      setPort("1143");
      setUseSsl(false);
    } else if (domain.includes(".") && !domain.endsWith(".")) {
      setServer(`imap.${domain}`);
      setPort("993");
      setUseSsl(true);
    }
  };

  const addLog = (msg: string) => setLogs(prev => [...prev, `[${new Date().toLocaleTimeString()}] ${msg}`]);

  const testConnection = async () => {
    const cleanUser = username.trim();
    const cleanPass = password.trim().replace(/\\+$/, "").trim();
    const effectivePass = cleanUser.toLowerCase().includes("gmail") ? cleanPass.replace(/\s+/g, "") : cleanPass;
    setConnecting(true);
    setLogs([]);
    addLog(`Testing connection to ${server}:${port} (SSL: ${useSsl ? "YES" : "NO"})...`);
    try {
      const boxes = await invoke<string[]>("imap_list_mailboxes", {
        input: {
          server: server.trim(), 
          port: parseInt(port) || 993, 
          username: cleanUser, 
          password: effectivePass, 
          use_ssl: useSsl,
          useSsl
        }
      });
      setMailboxes(boxes);
      addLog(`✓ Connection & Authentication Successful!`);
      addLog(`Discovered ${boxes.length} account folders: ${boxes.join(", ")}`);
    } catch (e: any) {
      addLog(`✗ Connection failed: ${e}`);
    }
    setConnecting(false);
  };

  const acquireEmails = async () => {
    const cleanUser = username.trim();
    const cleanPass = password.trim().replace(/\\+$/, "").trim();
    const effectivePass = cleanUser.toLowerCase().includes("gmail") ? cleanPass.replace(/\s+/g, "") : cleanPass;
    setFetching(true);
    setLogs([]);
    setProgress(null);
    addLog(`Starting forensic streaming acquisition for account: ${cleanUser}...`);
    addLog(`Scope: ${mailboxScope === "ALL" ? "Entire Account (All Mailboxes)" : mailboxScope}`);
    addLog(`⚡ Real-time incremental deduplication active (previously ingested emails will be preserved)`);

    const onEvent = new Channel<any>();
    onEvent.onmessage = (p: any) => {
      if (p?.log) {
        setLogs(prev => {
          const last = prev[prev.length - 1];
          const newLog = `[${new Date().toLocaleTimeString()}] ${p.log}`;
          if (last === newLog) return prev;
          return [...prev, newLog];
        });
      }
      if (p?.status === "ingested" || p?.status === "folder_discovered" || p?.status === "duplicate_skipped") {
        setProgress(prev => ({
          ...prev,
          folder: p.folder || prev?.folder,
          folderIndex: p.folder_index || prev?.folderIndex,
          totalFolders: p.total_folders || prev?.totalFolders,
          msgSeq: p.msg_seq || prev?.msgSeq,
          folderTotal: p.folder_total || prev?.folderTotal,
          ingested: p.ingested_count !== undefined ? p.ingested_count : prev?.ingested,
          duplicatesSkipped: p.duplicates_skipped !== undefined ? p.duplicates_skipped : prev?.duplicatesSkipped,
          subject: p.subject || prev?.subject,
          from: p.from || prev?.from,
        }));
      }
    };

    try {
      const res = await invoke<any>("imap_fetch_emails", {
        input: {
          case_id: caseId,
          caseId,
          evidence_id: `imap_${Date.now()}`,
          evidenceId: `imap_${Date.now()}`,
          server: server.trim(), 
          port: parseInt(port) || 993, 
          username: cleanUser, 
          password: effectivePass, 
          use_ssl: useSsl,
          useSsl,
          mailbox: mailboxScope,
          max_messages: null
        },
        on_event: onEvent,
        onEvent
      });
      setResult(res);
      addLog(`✓ Acquisition Finished: Ingested ${res.downloaded} new emails (${res.duplicates_skipped || 0} duplicates skipped) across ${res.folders_acquired?.length || 1} folders`);
      onComplete();
    } catch (e: any) {
      addLog(`✗ Acquisition error: ${e}`);
    }
    setFetching(false);
  };

  const stopAcquisition = async () => {
    try {
      await invoke("imap_cancel_acquisition");
      addLog("⏹ Stop requested. Wrapping up current message and committing all downloaded emails to database...");
    } catch (e: any) {
      addLog(`Error stopping: ${e}`);
    }
  };

  const percent = (progress?.folderTotal && progress?.msgSeq) ? Math.min(100, Math.round((progress.msgSeq / progress.folderTotal) * 100)) : 0;

  return (
    <div>
      <div className="row between mb-3">
        <div>
          <h3 style={{ fontSize: 16, fontWeight: 600, color: "var(--text-0)" }}>Live IMAP Account Acquisition</h3>
          <p className="muted" style={{ fontSize: 12 }}>
            Forensic multi-folder streaming extraction over TLS with live deduplication &amp; disk payload storage
          </p>
        </div>
        <div className="row gap-2">
          <span className="badge badge-green">RESUME / DEDUPLICATION READY</span>
          <span className="badge badge-blue">TLS 1.3 / SSL VERIFIED</span>
        </div>
      </div>
      
      <div className="card mb-4">
        <div className="grid-2">
          <div className="field">
            <label className="label">Target Email / Account *</label>
            <input 
              className="input" 
              value={username} 
              onChange={e => handleEmailChange(e.target.value)} 
              placeholder="e.g. suspect@gmail.com, target@company.com" 
              required
            />
          </div>
          <div className="field">
            <label className="label">Password / App-Specific Password *</label>
            <input 
              className="input" 
              type="password" 
              value={password} 
              onChange={e => setPassword(e.target.value)} 
              placeholder="••••••••••••••••" 
              required
            />
          </div>
        </div>

        <div className="grid-2" style={{ marginTop: 12 }}>
          <div className="field">
            <label className="label">Acquisition Scope</label>
            <select className="input" value={mailboxScope} onChange={e => setMailboxScope(e.target.value)}>
              <option value="ALL">📦 Entire Account (All Folders: Inbox, Sent, Trash, Spam, Archive)</option>
              <option value="INBOX">📥 Inbox Only</option>
              {mailboxes.filter(b => b.toUpperCase() !== "INBOX").map(b => (
                <option key={b} value={b}>📁 {b}</option>
              ))}
            </select>
          </div>

          <div className="field" style={{ display: "flex", flexDirection: "column", justifyContent: "flex-end" }}>
            <button 
              type="button" 
              className="btn btn-ghost" 
              style={{ fontSize: 12, textAlign: "left", width: "fit-content", padding: "8px 12px" }}
              onClick={() => setShowAdvanced(!showAdvanced)}
            >
              {showAdvanced ? "▲ Hide Server Configuration" : "⚙️ Custom Server Settings (Auto-Configured)"}
            </button>
          </div>
        </div>

        {/* Collapsible Advanced Server Configuration */}
        {showAdvanced && (
          <div style={{ marginTop: 16, padding: 14, background: "var(--bg-3)", borderRadius: "var(--r-md)" }}>
            <h5 style={{ fontSize: 12, fontWeight: 600, color: "var(--text-1)", marginBottom: 10 }}>IMAP Server Parameters</h5>
            <div className="grid-3">
              <div className="field">
                <label className="label">Host / Server</label>
                <input className="input" value={server} onChange={e => setServer(e.target.value)} placeholder="imap.server.com" />
              </div>
              <div className="field">
                <label className="label">Port</label>
                <input className="input" value={port} onChange={e => setPort(e.target.value)} placeholder="993" />
              </div>
              <div className="field" style={{ display: "flex", alignItems: "center", paddingTop: 20 }}>
                <label className="row gap-2" style={{ cursor: "pointer" }}>
                  <input type="checkbox" checked={useSsl} onChange={e => setUseSsl(e.target.checked)} />
                  <span style={{ fontSize: 12, fontWeight: 500 }}>Use SSL / TLS (Port 993)</span>
                </label>
              </div>
            </div>
          </div>
        )}

        <div className="row between" style={{ marginTop: 20 }}>
          <div className="row gap-2">
            <button 
              type="button" 
              className="btn btn-ghost" 
              onClick={testConnection} 
              disabled={connecting || fetching || !username || !password}
            >
              {connecting ? "Testing Connection..." : "🔗 Test Connection & Enumerate Folders"}
            </button>
            <button 
              type="button" 
              className="btn btn-primary" 
              onClick={acquireEmails} 
              disabled={fetching || connecting || !username || !password}
            >
              {fetching ? "⏳ Acquiring Live Account..." : "📥 Acquire & Ingest Live Emails"}
            </button>
          </div>

          {fetching && (
            <button 
              type="button" 
              className="btn btn-danger" 
              onClick={stopAcquisition}
              style={{ background: "#dc2626", color: "#fff", borderColor: "#dc2626" }}
            >
              ⏹ Stop / Pause Acquisition
            </button>
          )}
        </div>
      </div>

      {/* Live Streaming Progress HUD */}
      {fetching && progress && (
        <div className="card mb-4" style={{ border: "1px solid var(--accent)", background: "var(--bg-2)" }}>
          <div className="row between mb-2">
            <div className="row gap-2" style={{ alignItems: "center" }}>
              <span className="badge badge-blue">FOLDER {progress.folderIndex || 1} OF {progress.totalFolders || 1}</span>
              <strong style={{ fontSize: 13, color: "var(--text-0)" }}>{progress.folder || "Scanning folder..."}</strong>
            </div>
            <div style={{ fontSize: 12, fontWeight: 700, color: "var(--accent)" }}>
              {percent}% ({progress.msgSeq || 0}/{progress.folderTotal || 0} messages)
            </div>
          </div>

          {/* Animated Progress Bar */}
          <div style={{ width: "100%", height: 8, background: "var(--bg-3)", borderRadius: 4, overflow: "hidden", marginBottom: 12 }}>
            <div style={{ width: `${percent}%`, height: "100%", background: "linear-gradient(90deg, #3b82f6, #6366f1)", transition: "width 0.2s ease" }} />
          </div>

          <div className="grid-3" style={{ fontSize: 12 }}>
            <div>
              <span className="muted">Total Ingested:</span> <strong style={{ color: "var(--success)" }}>{progress.ingested || 0}</strong>
            </div>
            <div>
              <span className="muted">Duplicates Skipped:</span> <strong style={{ color: "#38bdf8" }}>{progress.duplicatesSkipped || 0}</strong>
            </div>
            <div style={{ overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
              <span className="muted">Current:</span> <span>{progress.subject || "..."}</span>
            </div>
          </div>
        </div>
      )}

      {result && (
        <div className="card mb-4">
          <h4 style={{ fontSize: 14, fontWeight: 600, color: "var(--text-0)", marginBottom: 12 }}>
            {result.was_cancelled ? "⏹ Acquisition Paused / Stopped" : "✓ Acquisition Results"}
          </h4>
          <div className="grid-3 mb-3">
            <div className="card" style={{ textAlign: "center", padding: 12 }}>
              <div style={{ fontSize: 22, fontWeight: 700, color: "var(--accent)" }}>{result.total_found}</div>
              <div className="muted text-sm">Discovered on Server</div>
            </div>
            <div className="card" style={{ textAlign: "center", padding: 12 }}>
              <div style={{ fontSize: 22, fontWeight: 700, color: "var(--success)" }}>{result.downloaded}</div>
              <div className="muted text-sm">Ingested &amp; Saved to DB</div>
            </div>
            <div className="card" style={{ textAlign: "center", padding: 12 }}>
              <div style={{ fontSize: 22, fontWeight: 700, color: "#38bdf8" }}>{result.duplicates_skipped || 0}</div>
              <div className="muted text-sm">Skipped (Saved Bandwidth)</div>
            </div>
          </div>
          {result.folders_acquired && result.folders_acquired.length > 0 && (
            <div style={{ fontSize: 12, color: "var(--text-2)" }}>
              <strong>Folders Acquired:</strong> {result.folders_acquired.join(", ")}
            </div>
          )}
        </div>
      )}

      {logs.length > 0 && (
        <div className="card" style={{ maxHeight: 260, overflowY: "auto", background: "#0b0f19", border: "1px solid #1e293b", padding: 14 }}>
          <div className="row between mb-2">
            <h4 style={{ fontSize: 12, fontWeight: 700, color: "#94a3b8", letterSpacing: "0.05em", margin: 0 }}>
              📡 LIVE FORENSIC ACQUISITION AUDIT STREAM
            </h4>
            <span style={{ fontSize: 10, color: "#64748b" }}>{logs.length} EVENTS</span>
          </div>
          {logs.map((log, i) => {
            const isSuccess = log.includes("✓") || log.includes("Ingested");
            const isError = log.includes("✗") || log.includes("Error") || log.includes("failed");
            const isSkip = log.includes("Skipped") || log.includes("duplicate");
            return (
              <div 
                key={i} 
                style={{ 
                  fontSize: 11, 
                  fontFamily: "var(--mono)", 
                  marginBottom: 3, 
                  lineHeight: 1.4,
                  color: isSuccess ? "#4ade80" : isError ? "#f87171" : isSkip ? "#38bdf8" : "#cbd5e1" 
                }}
              >
                {log}
              </div>
            );
          })}
          <div ref={logsEndRef} />
        </div>
      )}
    </div>
  );
}