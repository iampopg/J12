import { View, FolderFilter, Evidence, Dashboard } from "./types";
import { FooterSignature } from "../../components/FooterSignature";

interface Props {
  sidebarCollapsed: boolean;
  setSidebarCollapsed: (b: boolean) => void;
  overviewOpen: boolean;
  setOverviewOpen: (b: boolean) => void;
  intelligenceOpen: boolean;
  setIntelligenceOpen: (b: boolean) => void;
  emailFolderOpen: boolean;
  setEmailFolderOpen: (b: boolean) => void;
  investigationFolderOpen: boolean;
  setInvestigationFolderOpen: (b: boolean) => void;
  caseManagementOpen: boolean;
  setCaseManagementOpen: (b: boolean) => void;
  helpFolderOpen: boolean;
  setHelpFolderOpen: (b: boolean) => void;
  view: View;
  setView: (v: View) => void;
  folderFilter: FolderFilter;
  setFolderFilter: (f: FolderFilter) => void;
  setActiveEvidenceId: (id: string | null) => void;
  evidence: Evidence[];
  dashboard: Dashboard | null;
  emailCounts: Record<string, number>;
  notesCount: number;
  hasDone: boolean;
  onOpenExport: () => void;
  onOpenDelete: () => void;
}

export function WorkspaceSidebar({
  sidebarCollapsed,
  setSidebarCollapsed,
  overviewOpen,
  setOverviewOpen,
  intelligenceOpen,
  setIntelligenceOpen,
  emailFolderOpen,
  setEmailFolderOpen,
  investigationFolderOpen,
  setInvestigationFolderOpen,
  caseManagementOpen,
  setCaseManagementOpen,
  helpFolderOpen,
  setHelpFolderOpen,
  view,
  setView,
  folderFilter,
  setFolderFilter,
  setActiveEvidenceId,
  evidence,
  dashboard,
  emailCounts,
  notesCount,
  hasDone,
  onOpenExport,
  onOpenDelete,
}: Props) {
  return (
    <nav className="sidebar" style={{ width: sidebarCollapsed ? 50 : 230, minWidth: sidebarCollapsed ? 50 : 230 }}>
      <div className="sb-section" style={{ display: "flex", justifyContent: "flex-end", padding: "6px 8px" }}>
        <button className="sb-toggle" onClick={() => setSidebarCollapsed(!sidebarCollapsed)} title={sidebarCollapsed ? "Expand Sidebar" : "Collapse Sidebar"}>
          {sidebarCollapsed ? "→" : "←"}
        </button>
      </div>

      {/* 1. 📁 Case Overview */}
      {!sidebarCollapsed && (
        <div className="sb-folder">
          <div className="sb-folder-header" onClick={() => setOverviewOpen(!overviewOpen)}>
            <span className="sb-folder-arrow">{overviewOpen ? "▼" : "▶"}</span>
            <span className="sb-label" style={{ margin: 0 }}>📁 Case Overview</span>
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

      {/* 2. 📥 Evidence & Ingest Hub */}
      {!sidebarCollapsed ? (
        <div style={{ padding: "0 6px", marginBottom: 6 }}>
          <button
            className={`sb-item ${view === "evidence" ? "active" : ""}`}
            onClick={() => {
              setActiveEvidenceId(null);
              setView("evidence");
            }}
            style={{
              width: "100%",
              display: "flex",
              alignItems: "center",
              justifyContent: "space-between",
              fontWeight: 600,
            }}
            title="Evidence Sources & Ingest Hub"
          >
            <div style={{ display: "flex", alignItems: "center", gap: 8 }}>
              <span className="sb-icon">📥</span>
              <span>Evidence &amp; Ingest</span>
            </div>
            {evidence.length > 0 && (
              <span className="sb-count">{evidence.length}</span>
            )}
          </button>
        </div>
      ) : (
        <button
          className={`sb-item ${view === "evidence" ? "active" : ""}`}
          onClick={() => {
            setActiveEvidenceId(null);
            setView("evidence");
          }}
          title="Evidence & Ingest Hub"
        >
          <span className="sb-icon">📥</span>
        </button>
      )}

      {/* 3. 📁 Forensic Intelligence */}
      {!sidebarCollapsed && (
        <div className="sb-folder">
          <div className="sb-folder-header" onClick={() => setIntelligenceOpen(!intelligenceOpen)}>
            <span className="sb-folder-arrow">{intelligenceOpen ? "▼" : "▶"}</span>
            <span className="sb-label" style={{ margin: 0 }}>📁 Forensic Intelligence</span>
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
              <button className={`sb-item ${view === "locker" ? "active" : ""}`} onClick={() => setView("locker")}>
                <span className="sb-icon">🏷️</span> Tagged Evidence Locker
              </button>
            </div>
          )}
        </div>
      )}

      {/* 4. 📁 Email Messages */}
      <div className="sb-folder">
        <div className="sb-folder-header" onClick={() => setEmailFolderOpen(!emailFolderOpen)}>
          <span className="sb-folder-arrow">{emailFolderOpen ? "▼" : "▶"}</span>
          <span className="sb-label" style={{ margin: 0 }}>📁 Email Messages</span>
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

      {/* 5. 📁 Investigation & Graph */}
      <div className="sb-folder">
        <div className="sb-folder-header" onClick={() => setInvestigationFolderOpen(!investigationFolderOpen)}>
          <span className="sb-folder-arrow">{investigationFolderOpen ? "▼" : "▶"}</span>
          <span className="sb-label" style={{ margin: 0 }}>📁 Investigation &amp; Graph</span>
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

      {/* 6. 📁 Case Management */}
      <div className="sb-folder">
        <div className="sb-folder-header" onClick={() => setCaseManagementOpen(!caseManagementOpen)}>
          <span className="sb-folder-arrow">{caseManagementOpen ? "▼" : "▶"}</span>
          <span className="sb-label" style={{ margin: 0 }}>📁 Case Management</span>
        </div>
        {caseManagementOpen && !sidebarCollapsed && (
          <div className="sb-folder-content">
            <button className={`sb-item ${view === "case_manage" ? "active" : ""}`} onClick={() => setView("case_manage")}>
              <span className="sb-icon">⚙️</span> Case Settings
            </button>
            <button className={`sb-item ${view === "custody" ? "active" : ""}`} onClick={() => setView("custody")}>
              <span className="sb-icon">📋</span> Chain of Custody
            </button>
            <button className={`sb-item ${view === "integrity" ? "active" : ""}`} onClick={() => setView("integrity")}>
              <span className="sb-icon">🔒</span> Verify Evidence Integrity
            </button>
            <button className={`sb-item ${view === "notes" ? "active" : ""}`} onClick={() => setView("notes")}>
              <span className="sb-icon">📝</span> Case Notes
              {notesCount > 0 && <span className="sb-count">{notesCount}</span>}
            </button>
            <button className={`sb-item ${view === "report" ? "active" : ""}`} onClick={() => setView("report")}>
              <span className="sb-icon">📄</span> Generate Report
            </button>
            <button className="sb-item" style={{ opacity: 0.5, cursor: "not-allowed" }} disabled title="Coming Soon">
              <span className="sb-icon">🤖</span> AI Setup
              <span className="sb-count badge-gray">Soon</span>
            </button>
            <button className="sb-item" onClick={onOpenExport}>
              <span className="sb-icon">📦</span> Export Case
            </button>
            <button className="sb-item" style={{ color: "var(--red)" }} onClick={onOpenDelete}>
              <span className="sb-icon">🗑️</span> Delete Case
            </button>
          </div>
        )}
      </div>

      {/* 7. 📁 Help */}
      <div className="sb-folder">
        <div className="sb-folder-header" onClick={() => setHelpFolderOpen(!helpFolderOpen)}>
          <span className="sb-folder-arrow">{helpFolderOpen ? "▼" : "▶"}</span>
          <span className="sb-label" style={{ margin: 0 }}>📁 Help</span>
        </div>
        {helpFolderOpen && !sidebarCollapsed && (
          <div className="sb-folder-content">
            <button className={`sb-item ${view === "docs" ? "active" : ""}`} onClick={() => setView("docs")}>
              <span className="sb-icon">📖</span> Documentation
            </button>
          </div>
        )}
      </div>

      {/* Sidebar Footer Signature */}
      {!sidebarCollapsed && <FooterSignature compact style={{ marginTop: "auto" }} />}
    </nav>
  );
}
