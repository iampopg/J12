import {
  ForensicTaxonomyArtifact,
  getSeverityBadge,
  getTypeBadge,
  getConfidenceBadge,
} from "./types";
import { BookmarkButton } from "../../components/BookmarkButton";

interface Props {
  caseId: string;
  displayedArtifacts: ForensicTaxonomyArtifact[];
  selectedArtifact: ForensicTaxonomyArtifact | null;
  loading: boolean;
  onSelectArtifact: (a: ForensicTaxonomyArtifact) => void;
  onCopyToClipboard: (text: string) => void;
  onOpenEmailModal: (emailId: string) => void;
}

export function ArtifactsFeed({
  caseId,
  displayedArtifacts,
  selectedArtifact,
  loading,
  onSelectArtifact,
  onCopyToClipboard,
  onOpenEmailModal,
}: Props) {
  return (
    <div style={{ display: "flex", flexDirection: "column", gap: 8, minWidth: 0 }}>
      {loading ? (
        <div className="empty" style={{ padding: 40 }}>Classifying and indexing forensic taxonomy artifacts...</div>
      ) : displayedArtifacts.length === 0 ? (
        <div className="card empty" style={{ padding: 40 }}>
          No artifacts found for the selected taxonomy domain or query.
        </div>
      ) : (
        displayedArtifacts.map((a) => {
          const isSelected = selectedArtifact?.id === a.id;
          return (
            <div 
              key={a.id}
              className="card"
              style={{
                padding: "10px 14px",
                margin: 0,
                cursor: "pointer",
                borderLeft: a.severity === "critical" ? "4px solid var(--danger)" : a.severity === "high" ? "4px solid var(--warning)" : a.severity === "medium" ? "4px solid var(--accent)" : "4px solid var(--border)",
                background: isSelected ? "var(--bg-3)" : "var(--bg-2)",
                transition: "all 0.15s ease",
                minWidth: 0,
                overflow: "hidden"
              }}
              onClick={() => onSelectArtifact(a)}
            >
              <div className="row between mb-2" style={{ flexWrap: "wrap", gap: 6 }}>
                <div className="row gap-2" style={{ alignItems: "center", flexWrap: "wrap", minWidth: 0 }}>
                  <span style={{ fontSize: 12.5, fontWeight: 700, color: "var(--text-0)" }}>{a.title}</span>
                  {getSeverityBadge(a.severity)}
                  {getTypeBadge(a.artifact_type)}
                  {getConfidenceBadge(a.confidence)}
                  {a.occurrenceCount && a.occurrenceCount > 1 && (
                    <span className="badge badge-blue" style={{ fontSize: 9.5 }}>
                      x{a.occurrenceCount}
                    </span>
                  )}
                </div>
                <span style={{ fontSize: 10.5, color: "var(--text-3)", fontFamily: "var(--mono)" }}>
                  {a.date_sent_utc ? new Date(a.date_sent_utc).toLocaleDateString() : ""}
                </span>
              </div>

              {/* Highlighted Extracted Value Box */}
              <div 
                style={{
                  background: "rgba(15, 23, 42, 0.9)",
                  border: "1px solid var(--border)",
                  borderRadius: "var(--r-sm)",
                  padding: "7px 10px",
                  fontFamily: "var(--mono)",
                  fontSize: 12,
                  color: a.domain_id === "credentials" ? "#f43f5e" : a.domain_id === "financial" ? "#22c55e" : a.domain_id === "crypto" ? "#eab308" : a.domain_id === "contraband" ? "#ef4444" : "#38bdf8",
                  marginBottom: 6,
                  display: "flex",
                  alignItems: "center",
                  justifyContent: "space-between",
                  gap: 8,
                  wordBreak: "break-all",
                  overflow: "hidden"
                }}
              >
                <span style={{ overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap", flex: 1 }}>
                  {a.primary_value}
                </span>
                <button 
                  className="btn btn-ghost btn-sm" 
                  style={{ padding: "1px 6px", fontSize: 10, height: "auto", flexShrink: 0 }}
                  onClick={(e) => {
                    e.stopPropagation();
                    onCopyToClipboard(a.primary_value);
                  }}
                  title="Copy extracted value"
                >
                  📋 Copy
                </button>
              </div>

              {/* Context & Source Row */}
              <div className="row between" style={{ fontSize: 11, color: "var(--text-3)", minWidth: 0 }}>
                <div style={{ overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap", flex: 1 }}>
                  Source: <strong style={{ color: "var(--text-2)" }}>{a.email_from}</strong>
                  {a.email_subject && ` · Subject: ${a.email_subject}`}
                </div>
                <div className="row gap-1" style={{ alignItems: "center", flexShrink: 0, marginLeft: 8 }} onClick={(e) => e.stopPropagation()}>
                  <BookmarkButton
                    caseId={caseId}
                    itemId={a.id}
                    itemType="artifact"
                    compact={true}
                  />
                  {a.email_id && (
                    <button 
                      className="btn btn-ghost btn-sm" 
                      style={{ padding: "1px 6px", fontSize: 10, height: "auto" }}
                      onClick={(e) => {
                        e.stopPropagation();
                        onOpenEmailModal(a.email_id);
                      }}
                    >
                      ✉️ View Email
                    </button>
                  )}
                </div>
              </div>
            </div>
          );
        })
      )}
    </div>
  );
}
