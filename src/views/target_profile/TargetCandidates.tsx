import { useState } from "react";
import { DetectedTarget, formatName } from "./types";

interface Props {
  detected: DetectedTarget[];
  currentEmail: string;
  onSelectCandidate: (email: string) => void;
}

export function TargetCandidates({ detected, currentEmail, onSelectCandidate }: Props) {
  const [showAllCandidates, setShowAllCandidates] = useState(false);

  if (detected.length <= 1) return null;

  return (
    <div className="card mb-4">
      <div className="row between mb-3">
        <div>
          <h4 style={{ fontSize: 13, fontWeight: 700, color: "var(--text-0)" }}>
            Detected Candidate Subjects ({detected.length})
          </h4>
          <p className="muted" style={{ fontSize: 11, margin: 0 }}>
            Select any subject to pivot and inspect their individual communication network:
          </p>
        </div>
        <button className="btn btn-ghost btn-sm" onClick={() => setShowAllCandidates(!showAllCandidates)}>
          {showAllCandidates ? "▲ Show Fewer" : `▼ View All (${detected.length})`}
        </button>
      </div>
      <div style={{ display: "grid", gridTemplateColumns: "repeat(auto-fill, minmax(260px, 1fr))", gap: 8 }}>
        {(showAllCandidates ? detected : detected.slice(0, 6)).map((t, i) => {
          const name = formatName(t.display_name, t.email);
          const isSelected = t.email === currentEmail;
          return (
            <div
              key={i}
              className="row between tr-click"
              style={{
                padding: "8px 12px",
                background: isSelected ? "var(--accent-subtle)" : "var(--bg-3)",
                border: isSelected ? "1px solid var(--accent)" : "1px solid var(--border)",
                borderRadius: "var(--r-sm)",
                cursor: "pointer"
              }}
              onClick={() => onSelectCandidate(t.email)}
            >
              <div style={{ overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap", paddingRight: 8 }}>
                <div style={{ fontSize: 12, fontWeight: 600, color: isSelected ? "var(--accent)" : "var(--text-0)", display: "flex", alignItems: "center", gap: 6 }}>
                  <span>{t.is_primary_target || t.is_custodian ? "🎯" : t.is_automated ? "🤖" : "👤"}</span>
                  <span style={{ overflow: "hidden", textOverflow: "ellipsis" }}>{name}</span>
                </div>
                <div style={{ fontSize: 10, color: "var(--text-3)", fontFamily: "var(--mono)" }}>
                  {t.email}
                </div>
              </div>
              <div style={{ display: "flex", flexDirection: "column", alignItems: "flex-end", gap: 2 }}>
                <span className={`badge ${isSelected ? "badge-blue" : t.is_automated ? "badge-gray" : "badge-outline"}`} style={{ fontSize: 10 }}>
                  {t.total_emails} msgs
                </span>
                {(t.is_primary_target || t.is_custodian) && (
                  <span style={{ fontSize: 8, color: "var(--accent)", fontWeight: 700, letterSpacing: "0.04em" }}>CUSTODIAN</span>
                )}
              </div>
            </div>
          );
        })}
      </div>
    </div>
  );
}
