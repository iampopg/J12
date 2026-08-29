import { ForensicTaxonomyArtifact } from "./types";
import { BookmarkButton } from "../../components/BookmarkButton";

interface Props {
  caseId: string;
  selectedArtifact: ForensicTaxonomyArtifact;
  onClose: () => void;
  onCopyToClipboard: (text: string) => void;
  onOpenEmailModal: (emailId: string) => void;
}

export function ArtifactInspectorDrawer({
  caseId,
  selectedArtifact,
  onClose,
  onCopyToClipboard,
  onOpenEmailModal,
}: Props) {
  return (
    <div className="card" style={{ position: "sticky", top: 16, height: "fit-content", padding: 16, minWidth: 0, overflow: "hidden" }}>
      <div className="row between mb-3">
        <h3 style={{ fontSize: 14, fontWeight: 700, color: "var(--text-0)", margin: 0 }}>
          Artifact Forensic Dossier
        </h3>
        <div className="row gap-1">
          <BookmarkButton
            caseId={caseId}
            itemId={selectedArtifact.id}
            itemType="artifact"
            compact={true}
          />
          <button className="btn btn-ghost btn-sm" onClick={onClose}>✕</button>
        </div>
      </div>

      <div style={{ marginBottom: 12 }}>
        <div className="label">TAXONOMY CLASSIFICATION</div>
        <div style={{ fontSize: 13, fontWeight: 700, color: "var(--text-0)" }}>
          {selectedArtifact.title}
        </div>
        <div style={{ fontSize: 11, fontFamily: "var(--mono)", color: "var(--text-3)", marginTop: 2 }}>
          Domain: <strong>{selectedArtifact.domain_id}</strong> / {selectedArtifact.subcategory_id}
        </div>
      </div>

      <div style={{ marginBottom: 12 }}>
        <div className="label">EXTRACTED PRIMARY VALUE</div>
        <div 
          style={{
            background: "var(--bg-3)",
            border: "1px solid var(--accent)",
            borderRadius: "var(--r-sm)",
            padding: "10px 12px",
            fontFamily: "var(--mono)",
            fontSize: 12.5,
            color: "#4ade80",
            fontWeight: 700,
            wordBreak: "break-all"
          }}
        >
          {selectedArtifact.primary_value}
        </div>
        <button 
          className="btn btn-ghost btn-sm" 
          style={{ width: "100%", marginTop: 6, fontSize: 11 }}
          onClick={() => onCopyToClipboard(selectedArtifact.primary_value)}
        >
          📋 Copy Value
        </button>
      </div>

      {selectedArtifact.secondary_value && (
        <div style={{ marginBottom: 12 }}>
          <div className="label">SECONDARY METADATA / PROVENANCE</div>
          <div style={{ fontSize: 11, fontFamily: "var(--mono)", color: "var(--text-1)", wordBreak: "break-all" }}>
            {selectedArtifact.secondary_value}
          </div>
        </div>
      )}

      <div style={{ marginBottom: 12 }}>
        <div className="label">EVIDENCE CONTEXT PREVIEW</div>
        <div 
          style={{
            background: "var(--bg-1)",
            border: "1px solid var(--border)",
            borderRadius: "var(--r-sm)",
            padding: "10px 12px",
            fontSize: 11.5,
            color: "var(--text-1)",
            maxHeight: 160,
            overflowY: "auto",
            lineHeight: 1.4
          }}
        >
          {selectedArtifact.details || "No preview snippet available."}
        </div>
      </div>

      {selectedArtifact.email_id ? (
        <>
          <div style={{ marginBottom: 16 }}>
            <div className="label">ORIGINATING EMAIL</div>
            <div style={{ fontSize: 12, color: "var(--text-1)", marginBottom: 4 }}>
              <strong>Subject:</strong> {selectedArtifact.email_subject || "(No Subject)"}
            </div>
            <div style={{ fontSize: 11, color: "var(--text-3)", fontFamily: "var(--mono)", marginBottom: 4 }}>
              <strong>From:</strong> {selectedArtifact.email_from}
            </div>
            <div style={{ fontSize: 11, color: "var(--text-3)" }}>
              <strong>Date:</strong> {selectedArtifact.date_sent_utc || "Unknown"}
            </div>
          </div>

          <button 
            className="btn btn-primary btn-sm" 
            style={{ width: "100%" }}
            onClick={() => onOpenEmailModal(selectedArtifact.email_id)}
          >
            ✉️ Open Email in Forensic Viewer
          </button>
        </>
      ) : null}
    </div>
  );
}
