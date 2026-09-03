import { Case, Evidence, View, cleanDisplayName } from "../types";

interface Props {
  caseData: Case | null;
  evidence?: Evidence[];
  targetPartners: any[];
  criticalFindings: any[];
  totalFindings: number;
  onNavigate: (view: View) => void;
}

export function DashboardThreatRadar({
  caseData,
  evidence,
  targetPartners,
  criticalFindings,
  totalFindings,
  onNavigate,
}: Props) {
  const maxPartnerCount = Math.max(...targetPartners.map((p) => p.count || 1), 1);

  const emailMatch = evidence?.find((e) => e.filename?.includes("@") || e.source_description?.includes("@"))
    ? (evidence?.find((e) => e.filename?.includes("@"))?.filename?.match(/([a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,})/)?.[1]
       || evidence?.find((e) => e.source_description?.includes("@"))?.source_description?.match(/([a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,})/)?.[1])
    : undefined;
  const targetEmail = caseData?.target_email || emailMatch || targetPartners[0]?.email;
  const targetName = caseData?.target_name || (targetEmail?.includes("@") ? targetEmail.split("@")[0] : "Target Subject");

  return (
    <div
      style={{
        display: "grid",
        gridTemplateColumns: "repeat(auto-fit, minmax(340px, 1fr))",
        gap: 16,
        marginBottom: 16,
      }}
    >
      {/* Left: Investigation Target Dossier */}
      <div
        className="card mb-0"
        style={{
          padding: 18,
          background: "linear-gradient(135deg, rgba(30, 41, 59, 0.4) 0%, rgba(15, 23, 42, 0.6) 100%)",
          border: "1px solid rgba(99, 102, 241, 0.25)",
          borderLeft: "4px solid var(--accent)",
          backdropFilter: "blur(8px)",
        }}
      >
        <div className="row between mb-3" style={{ alignItems: "center" }}>
          <span style={{ fontSize: 11, fontWeight: 700, color: "var(--accent)", letterSpacing: "0.08em" }}>
            🎯 CASE TARGET DOSSIER
          </span>
          <button
            className="btn btn-ghost btn-sm"
            style={{ fontSize: 10, padding: "2px 8px" }}
            onClick={() => onNavigate("target")}
          >
            Target Profile →
          </button>
        </div>

        <div style={{ fontSize: 18, fontWeight: 700, color: "var(--text-0)", marginBottom: 4 }}>
          {targetName}
        </div>
        <div
          style={{
            fontSize: 12,
            color: "var(--accent)",
            fontFamily: "var(--mono)",
            marginBottom: 6,
            display: "inline-block",
            background: "rgba(99, 102, 241, 0.1)",
            padding: "2px 8px",
            borderRadius: "var(--r-xs)",
          }}
        >
          {targetEmail || "No primary email assigned"}
        </div>
        <div style={{ fontSize: 12, color: "var(--text-2)", marginBottom: 14 }}>
          Organization: <strong style={{ color: "var(--text-1)" }}>{caseData?.target_organization || "Primary Mailbox"}</strong>
        </div>

        {targetPartners.length > 0 ? (
          <div>
            <div style={{ fontSize: 11, fontWeight: 700, color: "var(--text-3)", marginBottom: 8, letterSpacing: "0.04em" }}>
              TOP CORRESPONDENTS:
            </div>
            <div style={{ display: "flex", flexDirection: "column", gap: 6 }}>
              {targetPartners.map((p, i) => {
                const name = cleanDisplayName(p.display_name) || p.email;
                const percent = Math.min(100, Math.round((p.count / maxPartnerCount) * 100));

                return (
                  <div
                    key={i}
                    className="tr-click"
                    style={{
                      padding: "6px 10px",
                      background: "var(--bg-3)",
                      borderRadius: "var(--r-xs)",
                      cursor: "pointer",
                      border: "1px solid var(--border)",
                    }}
                    onClick={() => onNavigate("entities")}
                    title="Click to view entity communications"
                  >
                    <div className="row between mb-1" style={{ fontSize: 11 }}>
                      <span style={{ fontWeight: 600, color: "var(--text-1)" }}>{name}</span>
                      <span className="badge badge-blue" style={{ fontSize: 9 }}>
                        {p.count} msgs
                      </span>
                    </div>
                    <div style={{ height: 4, background: "rgba(255,255,255,0.06)", borderRadius: 2, overflow: "hidden" }}>
                      <div
                        style={{
                          width: `${percent}%`,
                          height: "100%",
                          background: "var(--accent)",
                          borderRadius: 2,
                        }}
                      />
                    </div>
                  </div>
                );
              })}
            </div>
          </div>
        ) : (
          <div className="empty" style={{ padding: 14, fontSize: 11 }}>
            No frequent correspondent records indexed yet.
          </div>
        )}
      </div>

      {/* Right: Active Threats & Security Findings */}
      <div
        className="card mb-0"
        style={{
          padding: 18,
          background: "linear-gradient(135deg, rgba(30, 41, 59, 0.4) 0%, rgba(15, 23, 42, 0.6) 100%)",
          border: "1px solid rgba(239, 68, 68, 0.25)",
          borderLeft: "4px solid #ef4444",
          backdropFilter: "blur(8px)",
        }}
      >
        <div className="row between mb-3" style={{ alignItems: "center" }}>
          <span style={{ fontSize: 11, fontWeight: 700, color: "#ef4444", letterSpacing: "0.08em" }}>
            🚨 CRITICAL THREAT RADAR ({totalFindings})
          </span>
          <button
            className="btn btn-ghost btn-sm"
            style={{ fontSize: 10, padding: "2px 8px" }}
            onClick={() => onNavigate("findings")}
          >
            Findings Matrix →
          </button>
        </div>

        {criticalFindings.length > 0 ? (
          <div style={{ display: "flex", flexDirection: "column", gap: 8 }}>
            {criticalFindings.map((f: any) => (
              <div
                key={f.id}
                className="tr-click"
                style={{
                  padding: "10px 12px",
                  background: "var(--bg-3)",
                  borderRadius: "var(--r-xs)",
                  borderLeft: "3px solid #ef4444",
                  cursor: "pointer",
                  border: "1px solid var(--border)",
                }}
                onClick={() => onNavigate("findings")}
                title="Click to view finding details"
              >
                <div className="row between mb-1" style={{ alignItems: "center" }}>
                  <strong style={{ fontSize: 12, color: "var(--text-0)" }}>{f.title}</strong>
                  <span
                    className={`badge badge-${
                      f.severity === "critical" ? "red" : f.severity === "high" ? "orange" : "blue"
                    }`}
                    style={{ fontSize: 9, textTransform: "uppercase" }}
                  >
                    {f.severity}
                  </span>
                </div>
                <div
                  style={{
                    fontSize: 11,
                    color: "var(--text-3)",
                    overflow: "hidden",
                    textOverflow: "ellipsis",
                    whiteSpace: "nowrap",
                  }}
                >
                  {f.description}
                </div>
              </div>
            ))}
          </div>
        ) : (
          <div
            className="empty"
            style={{
              padding: 24,
              fontSize: 12,
              display: "flex",
              flexDirection: "column",
              alignItems: "center",
              gap: 6,
            }}
          >
            <span style={{ fontSize: 24 }}>🛡️</span>
            <span>No critical threat violations flagged.</span>
            <span className="muted" style={{ fontSize: 11 }}>
              Run analysis to scan archive against IOCs & forensic heuristics.
            </span>
          </div>
        )}
      </div>
    </div>
  );
}
