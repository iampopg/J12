import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useAcquisition } from "../context/AcquisitionContext";
import { EmailListView } from "../views/EmailListView";
import { FindingsView } from "../views/FindingsView";
import { TargetProfileView } from "../views/TargetProfileView";
import { SearchView } from "../views/SearchView";
import { EntityDiveView } from "../views/EntityDiveView";
import { TimelineView } from "../views/TimelineView";
import { GraphView } from "../views/GraphView";
import { NotesView } from "../views/NotesView";
import { ReportView } from "../views/ReportView";
import { AISetupPage } from "../views/AISetupPage";
import { ArtifactsView } from "../views/ArtifactsView";
import { AttachmentsView } from "../views/AttachmentsView";
import { DocumentationView } from "../views/DocumentationView";
import { EvidenceLockerView } from "../views/EvidenceLockerView";

import { Case, Evidence, Dashboard, View, FolderFilter } from "./workspace/types";
import { WorkspaceHeader } from "./workspace/WorkspaceHeader";
import { WorkspaceSidebar } from "./workspace/WorkspaceSidebar";
import { ExportCaseModal, DeleteCaseModal } from "./workspace/WorkspaceModals";
import { DashboardView } from "./workspace/DashboardView";
import { EvidenceView } from "./workspace/EvidenceView";
import { CustodyView } from "./workspace/CustodyView";
import { CaseManageView } from "./workspace/CaseManageView";
import { IntegrityView } from "./workspace/IntegrityView";

export function CaseWorkspace({ caseId, onBack }: { caseId: string; onBack: () => void }) {
  const { isAcquiring, pipelineStep } = useAcquisition();

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

  const [intelligenceOpen, setIntelligenceOpenState] = useState(() => localStorage.getItem("sb_intel_open") !== "false");
  const setIntelligenceOpen = (val: boolean) => {
    localStorage.setItem("sb_intel_open", String(val));
    setIntelligenceOpenState(val);
  };

  const [emailFolderOpen, setEmailFolderOpenState] = useState(() => localStorage.getItem("sb_email_open") === "true");
  const setEmailFolderOpen = (val: boolean) => {
    localStorage.setItem("sb_email_open", String(val));
    setEmailFolderOpenState(val);
  };

  const [investigationFolderOpen, setInvestigationFolderOpenState] = useState(() => localStorage.getItem("sb_invest_open") === "true");
  const setInvestigationFolderOpen = (val: boolean) => {
    localStorage.setItem("sb_invest_open", String(val));
    setInvestigationFolderOpenState(val);
  };

  const [caseManagementOpen, setCaseManagementOpenState] = useState(() => localStorage.getItem("sb_manage_open") === "true");
  const setCaseManagementOpen = (val: boolean) => {
    localStorage.setItem("sb_manage_open", String(val));
    setCaseManagementOpenState(val);
  };

  const [helpFolderOpen, setHelpFolderOpenState] = useState(() => localStorage.getItem("sb_help_open") === "true");
  const setHelpFolderOpen = (val: boolean) => {
    localStorage.setItem("sb_help_open", String(val));
    setHelpFolderOpenState(val);
  };

  const [folderFilter, setFolderFilter] = useState<FolderFilter>("all");
  const [activeEvidenceId, setActiveEvidenceId] = useState<string | null>(null);
  const [showEvidenceDropdown, setShowEvidenceDropdown] = useState(false);

  const [showDeleteCase, setShowDeleteCase] = useState(false);
  const [deleteConfirmText, setDeleteConfirmText] = useState("");
  const [deletingCase, setDeletingCase] = useState(false);
  const [showExportModal, setShowExportModal] = useState(false);
  const [exportToast, setExportToast] = useState<string | null>(null);

  const hasDone = evidence.some((e) => e.parse_status === "done" || e.parse_status === "parsed");

  const loadAll = useCallback(async () => {
    setLoading(true);
    try {
      const [c, ev, dash] = await Promise.all([
        invoke<Case>("case_get", { input: { case_id: caseId } }),
        invoke<Evidence[]>("evidence_list", { input: { case_id: caseId } }),
        invoke<Dashboard>("dashboard", {
          input: {
            case_id: caseId,
            evidence_id: activeEvidenceId || undefined,
          },
        }),
      ]);
      setCaseData(c);
      setEvidence(ev);
      setDashboard(dash);
      if (ev.length === 0 && !localStorage.getItem(`last_view_${caseId}`)) {
        setView("evidence");
      }
    } catch (e) {
      console.error(e);
    } finally {
      setLoading(false);
    }
  }, [caseId, activeEvidenceId]);

  useEffect(() => {
    loadAll();
  }, [loadAll]);

  useEffect(() => {
    invoke<Dashboard>("dashboard", {
      input: {
        case_id: caseId,
        evidence_id: activeEvidenceId || undefined,
      },
    })
      .then((dash) => setDashboard(dash))
      .catch((e) => console.error("Failed to update dashboard for evidence filter:", e));
  }, [caseId, activeEvidenceId]);

  const handleDeleteCase = async () => {
    if (!caseId) return;
    setDeletingCase(true);
    try {
      await invoke<boolean>("case_delete", { input: { case_id: caseId } });
      try {
        Object.keys(localStorage).forEach((key) => {
          if (key.includes(caseId)) {
            localStorage.removeItem(key);
          }
        });
      } catch (storageErr) {
        console.warn("Failed to purge localStorage for deleted case:", storageErr);
      }
      onBack();
    } catch (e) {
      console.error("Failed to delete case:", e);
    } finally {
      setDeletingCase(false);
      setShowDeleteCase(false);
    }
  };

  useEffect(() => {
    const hasParsing = evidence.some((e) => e.parse_status === "parsing" || e.parse_status === "ingesting");
    if (!hasParsing && !isAcquiring && pipelineStep === "idle") return;
    const interval = setInterval(loadAll, 2500);
    return () => clearInterval(interval);
  }, [evidence, isAcquiring, pipelineStep, loadAll]);

  useEffect(() => {
    if (pipelineStep === "complete" || pipelineStep === "artifacts" || pipelineStep === "analysis") {
      loadAll();
    }
  }, [pipelineStep, loadAll]);

  const emailCounts = {
    sent: dashboard?.sent_count || 0,
    inbox: dashboard?.inbox_count || 0,
    important: dashboard?.important_count || 0,
    soft_deleted: dashboard?.soft_deleted_count || 0,
    drafts: dashboard?.drafts_count || 0,
    spam: dashboard?.spam_count || 0,
    other: dashboard?.other_count || 0,
    total: dashboard?.email_count || 0,
  };

  if (loading && !caseData) return <div className="app"><div className="empty">Loading case...</div></div>;

  return (
    <div className="app">
      <WorkspaceHeader
        caseData={caseData}
        evidence={evidence}
        dashboard={dashboard}
        activeEvidenceId={activeEvidenceId}
        setActiveEvidenceId={setActiveEvidenceId}
        showEvidenceDropdown={showEvidenceDropdown}
        setShowEvidenceDropdown={setShowEvidenceDropdown}
        view={view}
        setView={setView}
        setFolderFilter={setFolderFilter}
        onBack={onBack}
        hasDone={hasDone}
        caseId={caseId}
      />

      <div className="body">
        <WorkspaceSidebar
          sidebarCollapsed={sidebarCollapsed}
          setSidebarCollapsed={setSidebarCollapsed}
          overviewOpen={overviewOpen}
          setOverviewOpen={setOverviewOpen}
          intelligenceOpen={intelligenceOpen}
          setIntelligenceOpen={setIntelligenceOpen}
          emailFolderOpen={emailFolderOpen}
          setEmailFolderOpen={setEmailFolderOpen}
          investigationFolderOpen={investigationFolderOpen}
          setInvestigationFolderOpen={setInvestigationFolderOpen}
          caseManagementOpen={caseManagementOpen}
          setCaseManagementOpen={setCaseManagementOpen}
          helpFolderOpen={helpFolderOpen}
          setHelpFolderOpen={setHelpFolderOpen}
          view={view}
          setView={setView}
          folderFilter={folderFilter}
          setFolderFilter={setFolderFilter}
          setActiveEvidenceId={setActiveEvidenceId}
          evidence={evidence}
          dashboard={dashboard}
          emailCounts={emailCounts}
          notesCount={notesCount}
          hasDone={hasDone}
          onOpenExport={() => setShowExportModal(true)}
          onOpenDelete={() => setShowDeleteCase(true)}
        />

        <main className="content">
          {exportToast && (
            <div style={{ position: "fixed", bottom: 30, right: 30, background: "#1e293b", border: "1px solid #38bdf8", color: "#f8fafc", padding: "10px 18px", borderRadius: "var(--r-sm)", fontSize: 12, zIndex: 10001, boxShadow: "0 10px 25px rgba(0,0,0,0.5)" }}>
              {exportToast}
            </div>
          )}

          {view === "dashboard" && dashboard && (
            <DashboardView data={dashboard} evidence={evidence} caseData={caseData} caseId={caseId} onNavigate={setView} onRefresh={loadAll} />
          )}
          {view === "evidence" && (
            <EvidenceView evidence={evidence} caseId={caseId} onRefresh={loadAll} onViewEmails={(evId) => { setActiveEvidenceId(evId); setFolderFilter("all"); setView("emails"); }} />
          )}
          {view === "emails" && <EmailListView caseId={caseId} filter={folderFilter} evidenceFilter={activeEvidenceId} onEvidenceFilterChange={setActiveEvidenceId} onViewEntity={() => setView("entities")} />}
          {view === "sent" && <EmailListView caseId={caseId} filter="sent" evidenceFilter={activeEvidenceId} onEvidenceFilterChange={setActiveEvidenceId} onViewEntity={() => setView("entities")} />}
          {view === "inbox" && <EmailListView caseId={caseId} filter="inbox" evidenceFilter={activeEvidenceId} onEvidenceFilterChange={setActiveEvidenceId} onViewEntity={() => setView("entities")} />}
          {view === "drafts" && <EmailListView caseId={caseId} filter="drafts" evidenceFilter={activeEvidenceId} onEvidenceFilterChange={setActiveEvidenceId} onViewEntity={() => setView("entities")} />}
          {view === "soft_deleted" && <EmailListView caseId={caseId} filter="soft_deleted" evidenceFilter={activeEvidenceId} onEvidenceFilterChange={setActiveEvidenceId} onViewEntity={() => setView("entities")} />}
          {view === "spam" && <EmailListView caseId={caseId} filter="spam" evidenceFilter={activeEvidenceId} onEvidenceFilterChange={setActiveEvidenceId} onViewEntity={() => setView("entities")} />}
          {view === "other" && <EmailListView caseId={caseId} filter="other" evidenceFilter={activeEvidenceId} onEvidenceFilterChange={setActiveEvidenceId} onViewEntity={() => setView("entities")} />}
          {view === "search" && <SearchView caseId={caseId} evidenceFilter={activeEvidenceId} onViewEntity={() => setView("entities")} />}
          {view === "entities" && <EntityDiveView caseId={caseId} evidenceFilter={activeEvidenceId} />}
          {view === "timeline" && <TimelineView caseId={caseId} evidenceFilter={activeEvidenceId} />}
          {view === "graph" && <GraphView caseId={caseId} evidenceFilter={activeEvidenceId} />}
          {view === "findings" && <FindingsView caseId={caseId} evidenceFilter={activeEvidenceId} onGoToEvidence={() => setView("evidence")} />}
          {view === "artifacts" && <ArtifactsView caseId={caseId} evidenceFilter={activeEvidenceId} onSelectEmail={() => setView("emails")} />}
          {view === "attachments" && <AttachmentsView caseId={caseId} evidenceFilter={activeEvidenceId} onSelectEmail={() => setView("emails")} />}
          {view === "target" && <TargetProfileView caseId={caseId} caseData={caseData} evidenceFilter={activeEvidenceId} />}
          {view === "custody" && <CustodyView evidence={evidence} caseId={caseId} />}
          {view === "notes" && <NotesView caseId={caseId} onNotesCountChange={setNotesCount} />}
          {view === "case_manage" && <CaseManageView caseData={caseData} caseId={caseId} onUpdate={loadAll} onBack={() => setView("dashboard")} />}
          {view === "report" && <ReportView caseId={caseId} caseData={caseData} />}
          {view === "ai_setup" && <AISetupPage caseId={caseId} onAIEnabled={() => {}} onAIConfigured={() => {}} />}
          {view === "integrity" && <IntegrityView caseId={caseId} />}
          {view === "locker" && <EvidenceLockerView caseId={caseId} evidenceFilter={activeEvidenceId} onNavigate={(v) => setView(v as View)} />}
          {view === "docs" && <DocumentationView />}
        </main>
      </div>

      <ExportCaseModal
        show={showExportModal}
        onClose={() => setShowExportModal(false)}
        caseId={caseId}
        caseData={caseData}
        onSetToast={(msg) => {
          setExportToast(msg);
          setTimeout(() => setExportToast(null), 4000);
        }}
        onNavigate={setView}
      />

      <DeleteCaseModal
        show={showDeleteCase}
        onClose={() => {
          setShowDeleteCase(false);
          setDeleteConfirmText("");
        }}
        caseData={caseData}
        evidence={evidence}
        dashboard={dashboard}
        deleteConfirmText={deleteConfirmText}
        setDeleteConfirmText={setDeleteConfirmText}
        deletingCase={deletingCase}
        onDeleteCase={handleDeleteCase}
      />
    </div>
  );
}