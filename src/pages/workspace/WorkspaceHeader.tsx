import { useRef, useEffect } from "react";
import { Case, Evidence, Dashboard, View } from "./types";
import { J12Logo } from "../../components/J12Logo";
import { ExaminerProfileButton } from "../../components/ExaminerProfileButton";
import { useAcquisition } from "../../context/AcquisitionContext";

interface Props {
  caseData: Case | null;
  evidence: Evidence[];
  dashboard: Dashboard | null;
  activeEvidenceId: string | null;
  setActiveEvidenceId: (id: string | null) => void;
  showEvidenceDropdown: boolean;
  setShowEvidenceDropdown: (b: boolean) => void;
  view: View;
  setView: (v: View) => void;
  setFolderFilter: (f: any) => void;
  onBack: () => void;
  hasDone: boolean;
  caseId: string;
}

export function WorkspaceHeader({
  caseData,
  evidence,
  dashboard,
  activeEvidenceId,
  setActiveEvidenceId,
  showEvidenceDropdown,
  setShowEvidenceDropdown,
  view,
  setView,
  setFolderFilter,
  onBack,
  hasDone,
  caseId,
}: Props) {
  const {
    isAcquiring,
    pipelineStep,
    activeCaseId: acquiringCaseId,
    account: acquiringAccount,
    protocol: acquiringProtocol,
    progress: acquiringProgress,
    percent: acquiringPercent,
    stopAcquisition,
  } = useAcquisition();

  const evidenceDropdownRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!showEvidenceDropdown) return;
    const handler = (e: MouseEvent) => {
      if (evidenceDropdownRef.current && !evidenceDropdownRef.current.contains(e.target as Node)) {
        setShowEvidenceDropdown(false);
      }
    };
    document.addEventListener("mousedown", handler);
    return () => document.removeEventListener("mousedown", handler);
  }, [showEvidenceDropdown, setShowEvidenceDropdown]);

  const activeEvidenceObj = activeEvidenceId ? evidence.find((e) => e.id === activeEvidenceId) : null;

  return (
    <>
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
        <div className="row gap-3" style={{ alignItems: "center" }}>
          {caseData?.target_email && !activeEvidenceId && (
            <div style={{ textAlign: "right" }}>
              <div style={{ fontSize: 10, color: "var(--text-3)", letterSpacing: "0.05em" }}>CASE TARGET</div>
              <div style={{ fontSize: 12, color: "var(--accent)", fontFamily: "var(--mono)" }}>{caseData.target_email}</div>
            </div>
          )}
          {activeEvidenceObj && (
            <div style={{ textAlign: "right" }}>
              <div style={{ fontSize: 10, color: "var(--text-3)", letterSpacing: "0.05em" }}>EVIDENCE SCOPE</div>
              <div style={{ fontSize: 12, color: "var(--accent)", fontFamily: "var(--mono)", maxWidth: 190, overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
                {activeEvidenceObj.filename}
              </div>
            </div>
          )}

          {/* Evidence Source Switcher Dropdown */}
          {evidence.length > 0 && (
            <div ref={evidenceDropdownRef} style={{ position: "relative" }}>
              <button
                className={`btn btn-sm ${activeEvidenceId ? "btn-primary" : "btn-ghost"}`}
                style={{
                  display: "flex",
                  alignItems: "center",
                  gap: 6,
                  fontSize: 12,
                  padding: "5px 12px",
                  borderRadius: "var(--r-sm)",
                  border: activeEvidenceId ? "1px solid var(--accent)" : "1px solid var(--border)",
                  background: activeEvidenceId ? "rgba(56, 189, 248, 0.15)" : "var(--bg-2)",
                  color: activeEvidenceId ? "var(--accent)" : "var(--text-1)",
                }}
                onClick={() => setShowEvidenceDropdown(!showEvidenceDropdown)}
                title="Switch between evidence sources or view combined case"
              >
                <span>{activeEvidenceObj ? (activeEvidenceObj.format === "imap" ? "☁️" : activeEvidenceObj.format === "mbox" ? "📦" : "📧") : "🌐"}</span>
                <span style={{ maxWidth: 190, overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap", fontWeight: 600 }}>
                  {activeEvidenceObj ? activeEvidenceObj.filename : `All Sources (${evidence.length})`}
                </span>
                <span style={{ fontSize: 9, opacity: 0.7 }}>▼</span>
              </button>

              {showEvidenceDropdown && (
                <div
                  style={{
                    position: "absolute",
                    top: "calc(100% + 6px)",
                    right: 0,
                    zIndex: 9999,
                    background: "var(--bg-1)",
                    border: "1px solid var(--border)",
                    borderRadius: "var(--r-md)",
                    boxShadow: "0 12px 32px rgba(0,0,0,0.6)",
                    padding: 8,
                    width: 320,
                    display: "flex",
                    flexDirection: "column",
                    gap: 4,
                  }}
                >
                  <div className="row between mb-1" style={{ padding: "4px 8px" }}>
                    <span style={{ fontSize: 10, fontWeight: 700, color: "var(--text-3)", textTransform: "uppercase", letterSpacing: "0.06em" }}>
                      Switch Evidence Source
                    </span>
                    <button
                      className="btn btn-ghost btn-sm"
                      style={{ fontSize: 10, padding: "1px 6px" }}
                      onClick={() => {
                        setView("evidence");
                        setShowEvidenceDropdown(false);
                      }}
                    >
                      + Ingest New
                    </button>
                  </div>

                  <div
                    className="tr-click"
                    style={{
                      padding: "8px 10px",
                      borderRadius: "var(--r-sm)",
                      background: !activeEvidenceId ? "var(--accent-subtle)" : "transparent",
                      border: !activeEvidenceId ? "1px solid var(--accent)" : "1px solid transparent",
                      display: "flex",
                      alignItems: "center",
                      justifyContent: "space-between",
                    }}
                    onClick={() => {
                      setActiveEvidenceId(null);
                      setShowEvidenceDropdown(false);
                    }}
                  >
                    <div className="row gap-2" style={{ alignItems: "center" }}>
                      <span style={{ fontSize: 16 }}>🌐</span>
                      <div>
                        <div style={{ fontSize: 12, fontWeight: 600, color: "var(--text-0)" }}>
                          All Evidence Sources
                        </div>
                        <div style={{ fontSize: 10, color: "var(--text-3)" }}>
                          Combined case records ({evidence.length} sources)
                        </div>
                      </div>
                    </div>
                    <span className="badge" style={{ fontSize: 10 }}>
                      {dashboard?.email_count?.toLocaleString() || 0}
                    </span>
                  </div>

                  <div style={{ height: 1, background: "var(--border)", margin: "4px 0" }} />

                  {evidence.map((ev) => {
                    const isSelected = activeEvidenceId === ev.id;
                    const icon = ev.format === "imap" ? "☁️" : ev.format === "mbox" ? "📦" : ev.format === "eml" ? "📧" : "📄";
                    return (
                      <div
                        key={ev.id}
                        className="tr-click"
                        style={{
                          padding: "8px 10px",
                          borderRadius: "var(--r-sm)",
                          background: isSelected ? "var(--accent-subtle)" : "transparent",
                          border: isSelected ? "1px solid var(--accent)" : "1px solid transparent",
                          display: "flex",
                          alignItems: "center",
                          justifyContent: "space-between",
                          gap: 8,
                        }}
                        onClick={() => {
                          setActiveEvidenceId(ev.id);
                          setView("emails");
                          setFolderFilter("all");
                          setShowEvidenceDropdown(false);
                        }}
                      >
                        <div className="row gap-2" style={{ alignItems: "center", minWidth: 0, flex: 1 }}>
                          <span style={{ fontSize: 16 }}>{icon}</span>
                          <div style={{ minWidth: 0 }}>
                            <div
                              style={{
                                fontSize: 12,
                                fontWeight: 600,
                                color: isSelected ? "var(--accent)" : "var(--text-0)",
                                overflow: "hidden",
                                textOverflow: "ellipsis",
                                whiteSpace: "nowrap",
                              }}
                            >
                              {ev.filename}
                            </div>
                            <div style={{ fontSize: 10, color: "var(--text-3)" }}>
                              {ev.format.toUpperCase()} · {(ev.size_bytes / 1024).toFixed(0)} KB
                            </div>
                          </div>
                        </div>
                        <span className="badge badge-blue" style={{ fontSize: 10, flexShrink: 0 }}>
                          {ev.message_count.toLocaleString()} msgs
                        </span>
                      </div>
                    );
                  })}
                </div>
              )}
            </div>
          )}

          {hasDone && (
            <span className="badge badge-green" style={{ fontSize: 11 }}>
              ● {dashboard?.email_count?.toLocaleString() || 0} emails
            </span>
          )}

          <button
            className={`btn ${view === "evidence" ? "btn-primary" : "btn-ghost"} btn-sm`}
            onClick={() => setView("evidence")}
            style={{ fontSize: 12 }}
          >
            📥 Ingest Hub
          </button>

          <ExaminerProfileButton />
        </div>
      </header>

      {/* Global Live Acquisition HUD */}
      {(isAcquiring || pipelineStep === "artifacts" || pipelineStep === "analysis" || (pipelineStep === "complete" && acquiringCaseId === caseId)) && (
        <div 
          style={{
            background: pipelineStep === "complete" ? "linear-gradient(90deg, #064e3b, #0f172a)" : "linear-gradient(90deg, rgba(30, 58, 138, 0.95), rgba(15, 23, 42, 0.95))",
            borderBottom: pipelineStep === "complete" ? "1px solid #10b981" : "1px solid #3b82f6",
            padding: "8px 16px",
            display: "flex",
            alignItems: "center",
            justifyContent: "space-between",
            gap: 16,
            boxShadow: "0 4px 20px rgba(0, 0, 0, 0.5)",
            zIndex: 100,
            flexWrap: "wrap",
          }}
        >
          <div style={{ display: "flex", alignItems: "center", gap: 12, minWidth: 0, flex: 1 }}>
            <div style={{ display: "flex", alignItems: "center", gap: 6, flexShrink: 0 }}>
              <span style={{ display: "inline-block", width: 8, height: 8, borderRadius: "50%", background: pipelineStep === "complete" ? "#10b981" : "#38bdf8", boxShadow: `0 0 10px ${pipelineStep === "complete" ? "#10b981" : "#38bdf8"}` }} />
              <span style={{ fontSize: 11, fontWeight: 700, letterSpacing: "0.06em", color: pipelineStep === "complete" ? "#4ade80" : "#60a5fa", textTransform: "uppercase" }}>
                {pipelineStep === "ingesting" ? `LIVE ${acquiringProtocol?.toUpperCase()} ACQUISITION ACTIVE` : pipelineStep === "artifacts" ? "AUTOMATED PIPELINE: EXTRACTING ARTIFACTS" : pipelineStep === "analysis" ? "AUTOMATED PIPELINE: ANALYZING THREAT INTEL" : "FORENSIC INGESTION & PIPELINE READY"}
              </span>
            </div>

            <div style={{ fontSize: 12, color: "var(--text-1)", overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
              {pipelineStep === "ingesting" ? (
                <>
                  <strong>{acquiringAccount}</strong> · {acquiringProgress?.folder ? `[${acquiringProgress.folder}]` : "Scanning..."} · Ingested <strong style={{ color: "#4ade80" }}>{acquiringProgress?.ingested || 0}</strong> {acquiringProgress?.duplicatesSkipped ? `(${acquiringProgress.duplicatesSkipped} skipped)` : ""}
                  {acquiringProgress?.subject && <span style={{ opacity: 0.8, marginLeft: 8 }}>· Current: "{acquiringProgress.subject.slice(0, 45)}..."</span>}
                </>
              ) : pipelineStep === "artifacts" ? (
                "Step 1/3: Extracting Cryptos, IPs, Phone Numbers, Identifiers & Taxonomy..."
              ) : pipelineStep === "analysis" ? (
                "Step 2/3: Computing threat risk scores, SPF/DKIM/DMARC auth & spoofing detection..."
              ) : (
                "✓ All emails, artifacts, and security findings are synchronized and indexed."
              )}
            </div>
          </div>

          <div style={{ display: "flex", alignItems: "center", gap: 12, flexShrink: 0 }}>
            {pipelineStep === "ingesting" && (
              <div style={{ display: "flex", alignItems: "center", gap: 8, minWidth: 160 }}>
                <div style={{ width: 100, height: 6, background: "rgba(255,255,255,0.15)", borderRadius: 3, overflow: "hidden" }}>
                  <div style={{ width: `${acquiringPercent}%`, height: "100%", background: "#38bdf8", transition: "width 0.2s" }} />
                </div>
                <span style={{ fontSize: 11, fontWeight: 700, color: "#38bdf8", minWidth: 32 }}>{acquiringPercent}%</span>
              </div>
            )}

            {view !== "evidence" && (
              <button
                className="btn btn-sm btn-ghost"
                style={{ fontSize: 11, padding: "3px 8px", background: "rgba(255,255,255,0.1)" }}
                onClick={() => setView("evidence")}
              >
                📥 Open Ingest Hub
              </button>
            )}

            {isAcquiring && (
              <button
                className="btn btn-sm btn-danger"
                style={{ fontSize: 11, padding: "3px 8px" }}
                onClick={stopAcquisition}
              >
                ⏹ Stop
              </button>
            )}
          </div>
        </div>
      )}
    </>
  );
}
