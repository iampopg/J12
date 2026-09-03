import { Case, View } from "../types";

interface Props {
  caseData: Case | null;
  analyzing: boolean;
  onRunAnalysis: () => void;
  onRefresh: () => void;
  onNavigate: (view: View) => void;
}

export function DashboardHeader({
  caseData,
  analyzing,
  onRunAnalysis,
  onRefresh,
  onNavigate,
}: Props) {
  return (
    <div className="mb-4">
      {/* Top Command Banner */}
      <div
        className="card mb-3"
        style={{
          padding: "16px 20px",
          background: "linear-gradient(135deg, rgba(30, 41, 59, 0.7) 0%, rgba(15, 23, 42, 0.85) 100%)",
          border: "1px solid rgba(255, 255, 255, 0.08)",
          backdropFilter: "blur(12px)",
          boxShadow: "0 8px 32px 0 rgba(0, 0, 0, 0.37)",
        }}
      >
        <div className="row between" style={{ flexWrap: "wrap", gap: 14 }}>
          <div>
            <div className="row gap-2 mb-1" style={{ alignItems: "center" }}>
              <span
                style={{
                  display: "inline-block",
                  width: 8,
                  height: 8,
                  borderRadius: "50%",
                  backgroundColor: "#10b981",
                  boxShadow: "0 0 10px #10b981",
                }}
              />
              <span style={{ fontSize: 11, fontWeight: 700, color: "#10b981", letterSpacing: "0.08em" }}>
                ACTIVE INVESTIGATION
              </span>
              <span className="badge badge-blue" style={{ fontSize: 10 }}>
                CASE #{caseData?.case_number || "J12-001"}
              </span>
            </div>
            <h2 style={{ fontSize: 22, fontWeight: 800, color: "var(--text-0)", margin: 0 }}>
              {caseData?.title || "Case Investigation Command Center"}
            </h2>
            <p className="muted" style={{ fontSize: 12, marginTop: 4, marginBottom: 0 }}>
              Central intelligence hub for evidence triage, threat detection, entity profiling, and case reporting.
            </p>
          </div>

          <div className="row gap-2" style={{ alignItems: "center" }}>
            <button
              className="btn btn-ghost btn-sm"
              onClick={onRunAnalysis}
              disabled={analyzing}
              title="Run forensic rules, IOC detection & brand impersonation checks"
              style={{ display: "flex", alignItems: "center", gap: 6, fontWeight: 600 }}
            >
              <span>{analyzing ? "⏳" : "⚡"}</span>
              {analyzing ? "Analyzing Evidence..." : "Run Forensic Scan"}
            </button>
            <button
              className="btn btn-ghost btn-sm"
              onClick={onRefresh}
              title="Refresh database tallies and findings"
            >
              ↻ Sync
            </button>
            <button
              className="btn btn-primary btn-sm"
              onClick={() => onNavigate("report")}
              style={{ fontWeight: 600 }}
            >
              📄 Generate Report
            </button>
          </div>
        </div>
      </div>

      {/* Forensic Quick Navigation Bar */}
      <div
        className="card"
        style={{
          padding: "8px 14px",
          display: "flex",
          alignItems: "center",
          gap: 6,
          flexWrap: "wrap",
          background: "var(--bg-2)",
          border: "1px solid var(--border)",
        }}
      >
        <span style={{ fontSize: 11, fontWeight: 700, color: "var(--text-3)", marginRight: 6 }}>
          INVESTIGATION TOOLS:
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
          onClick={() => onNavigate("artifacts")}
        >
          🧬 Extracted Artifacts
        </button>
        <button
          className="btn btn-ghost btn-sm"
          style={{ fontSize: 11, padding: "4px 10px" }}
          onClick={() => onNavigate("findings")}
        >
          🛡️ Findings Matrix
        </button>
        <button
          className="btn btn-ghost btn-sm"
          style={{ fontSize: 11, padding: "4px 10px" }}
          onClick={() => onNavigate("custody")}
        >
          ⚖️ Chain of Custody
        </button>
      </div>
    </div>
  );
}
