import { Finding, FindingEmailItem, severityColor, statusBadge } from "./types";
import { BookmarkButton } from "../../components/BookmarkButton";

interface Props {
  caseId: string;
  selectedFinding: Finding;
  relatedEmails: FindingEmailItem[];
  loadingEmails: boolean;
  inspectingEmail: FindingEmailItem | null;
  setInspectingEmail: (em: FindingEmailItem | null) => void;
  noteText: string;
  setNoteText: (s: string) => void;
  authorName: string;
  setAuthorName: (s: string) => void;
  savingNote: boolean;
  onUpdateStatus: (id: string, st: string) => void;
  onAddNote: (e?: React.FormEvent) => void;
  onClose: () => void;
}

export function FindingDetailPanel({
  caseId,
  selectedFinding,
  relatedEmails,
  loadingEmails,
  inspectingEmail,
  setInspectingEmail,
  noteText,
  setNoteText,
  authorName,
  setAuthorName,
  savingNote,
  onUpdateStatus,
  onAddNote,
  onClose,
}: Props) {
  const parsedNotes = (selectedFinding.notes || "").split("\n---\n").filter(Boolean);

  return (
    <div className="card" style={{ border: "1px solid var(--accent)", boxShadow: "0 8px 30px rgba(0,0,0,0.25)" }}>
      <div className="row between mb-4" style={{ borderBottom: "1px solid var(--border)", paddingBottom: 12 }}>
        <div className="row gap-2" style={{ alignItems: "center", flexWrap: "wrap" }}>
          <span className="badge" style={{ background: `${severityColor(selectedFinding.severity)}22`, color: severityColor(selectedFinding.severity), border: `1px solid ${severityColor(selectedFinding.severity)}44`, fontWeight: 700 }}>
            {selectedFinding.severity.toUpperCase()}
          </span>
          <span className="badge badge-gray" style={{ fontWeight: 700 }}>{selectedFinding.type_}</span>
          <span className={`badge ${statusBadge(selectedFinding.status)}`}>{selectedFinding.status.toUpperCase()}</span>
          <h3 style={{ fontSize: 16, fontWeight: 700, color: "var(--text-0)", margin: 0 }}>
            {selectedFinding.title}
          </h3>
        </div>
        <div className="row gap-2" style={{ alignItems: "center" }}>
          <BookmarkButton
            caseId={caseId}
            itemId={selectedFinding.id}
            itemType="finding"
            compact={true}
          />
          <button className="btn btn-ghost btn-sm" onClick={onClose}>✕ Close Panel</button>
        </div>
      </div>

      {/* Top Quick Actions Bar */}
      <div className="row between mb-4" style={{ background: "var(--bg-0)", padding: "10px 14px", borderRadius: "var(--r-md)", border: "1px solid var(--border)", flexWrap: "wrap", gap: 10 }}>
        <div className="row gap-2" style={{ alignItems: "center", fontSize: 12, color: "var(--text-2)", flexWrap: "wrap" }}>
          <span>Investigator Decision:</span>
          <button
            className={`btn btn-sm ${selectedFinding.status === "confirmed" ? "btn-primary" : "btn-ghost"}`}
            style={{ background: selectedFinding.status === "confirmed" ? "var(--success)" : undefined, color: selectedFinding.status === "confirmed" ? "#fff" : "var(--success)", borderColor: "var(--success)" }}
            onClick={() => onUpdateStatus(selectedFinding.id, "confirmed")}
          >
            ✓ Confirm Threat
          </button>
          <button
            className={`btn btn-sm ${selectedFinding.status === "rejected" ? "btn-primary" : "btn-ghost"}`}
            style={{ background: selectedFinding.status === "rejected" ? "var(--danger)" : undefined, color: selectedFinding.status === "rejected" ? "#fff" : "var(--danger)", borderColor: "var(--danger)" }}
            onClick={() => onUpdateStatus(selectedFinding.id, "rejected")}
          >
            ✗ Reject (False Positive)
          </button>
          <button
            className={`btn btn-sm ${selectedFinding.status === "reviewed" ? "btn-primary" : "btn-ghost"}`}
            style={{ background: selectedFinding.status === "reviewed" ? "var(--warning)" : undefined, color: selectedFinding.status === "reviewed" ? "#000" : "var(--warning)", borderColor: "var(--warning)" }}
            onClick={() => onUpdateStatus(selectedFinding.id, "reviewed")}
          >
            👁 Mark Reviewed
          </button>
        </div>
        <div style={{ fontSize: 11, color: "var(--text-3)" }}>
          Recorded: {new Date(selectedFinding.created_at).toLocaleString()}
          {selectedFinding.reviewed_by && ` · Reviewed by: ${selectedFinding.reviewed_by}`}
        </div>
      </div>

      {/* Analysis Rationale Box */}
      <div className="mb-4" style={{ padding: 14, background: "rgba(239, 68, 68, 0.05)", border: "1px solid rgba(239, 68, 68, 0.2)", borderRadius: "var(--r-md)" }}>
        <h4 style={{ fontSize: 13, fontWeight: 700, color: "var(--danger)", marginBottom: 6 }}>
          🛡️ Automated Forensic Detection Rationale
        </h4>
        <p style={{ fontSize: 13, color: "var(--text-1)", lineHeight: 1.6, margin: 0 }}>
          {selectedFinding.description || "No automated rationale specified."}
        </p>
      </div>

      {/* Related Emails Section */}
      <div className="mb-4">
        <h4 style={{ fontSize: 14, fontWeight: 600, color: "var(--text-0)", marginBottom: 8 }}>
          Associated Evidentiary Email Messages ({relatedEmails.length})
        </h4>
        <p className="muted mb-3" style={{ fontSize: 12 }}>
          Inspect the exact email body, headers, and sender details that triggered this finding:
        </p>

        {loadingEmails ? (
          <div className="empty">Loading associated emails...</div>
        ) : relatedEmails.length === 0 ? (
          <div className="empty">No associated email messages found in database.</div>
        ) : (
          <div style={{ display: "flex", flexDirection: "column", gap: 12 }}>
            {relatedEmails.length > 1 && (
              <div className="row gap-2 mb-2" style={{ flexWrap: "wrap" }}>
                {relatedEmails.map((em, idx) => (
                  <button
                    key={em.id}
                    className={`btn btn-sm ${inspectingEmail?.id === em.id ? "btn-primary" : "btn-ghost"}`}
                    style={{ fontSize: 12 }}
                    onClick={() => setInspectingEmail(em)}
                  >
                    Email #{idx + 1}: {em.subject || "(no subject)"}
                  </button>
                ))}
              </div>
            )}

            {inspectingEmail && (
              <div style={{ background: "var(--bg-0)", borderRadius: "var(--r-md)", border: "1px solid var(--border)", overflow: "hidden" }}>
                <div style={{ padding: "12px 16px", background: "var(--bg-3)", borderBottom: "1px solid var(--border)" }}>
                  <div className="row between">
                    <div>
                      <strong style={{ fontSize: 14, color: "var(--text-0)" }}>{inspectingEmail.subject || "(no subject)"}</strong>
                      <div className="muted" style={{ fontSize: 12, marginTop: 2 }}>
                        From: <strong>{inspectingEmail.from_display || inspectingEmail.from_addr}</strong> ({inspectingEmail.from_addr})
                      </div>
                      <div className="muted" style={{ fontSize: 11, marginTop: 2 }}>
                        Date: {inspectingEmail.date_sent ? new Date(inspectingEmail.date_sent).toLocaleString() : "—"} · Folder: <span className="badge badge-gray">{inspectingEmail.folder_category}</span>
                      </div>
                    </div>
                    <div style={{ textAlign: "right" }}>
                      <span className="badge badge-red" style={{ fontSize: 11 }}>Risk Score: {inspectingEmail.risk_score}/100</span>
                    </div>
                  </div>
                </div>

                <div style={{ padding: 16 }}>
                  <div className="muted text-sm mb-2" style={{ fontWeight: 600 }}>Message Body Preview:</div>
                  <pre style={{
                    background: "var(--bg-1)",
                    border: "1px solid var(--border)",
                    borderRadius: "var(--r-sm)",
                    padding: 14,
                    fontSize: 12,
                    color: "var(--text-1)",
                    maxHeight: 240,
                    overflowY: "auto",
                    whiteSpace: "pre-wrap",
                    lineHeight: 1.5,
                    margin: 0,
                  }}>
                    {inspectingEmail.body_text || inspectingEmail.body_html || "No body content available in message."}
                  </pre>

                  {inspectingEmail.headers_raw && (
                    <details style={{ marginTop: 12 }}>
                      <summary style={{ fontSize: 12, color: "var(--accent)", cursor: "pointer", fontWeight: 500 }}>
                        View Raw Transport Headers ({inspectingEmail.headers_raw.split('\n').length} lines)
                      </summary>
                      <pre className="mono" style={{
                        fontSize: 10,
                        background: "var(--bg-1)",
                        padding: 12,
                        borderRadius: "var(--r-sm)",
                        border: "1px solid var(--border)",
                        maxHeight: 180,
                        overflowY: "auto",
                        marginTop: 8,
                        color: "var(--text-2)",
                      }}>
                        {inspectingEmail.headers_raw}
                      </pre>
                    </details>
                  )}
                </div>
              </div>
            )}
          </div>
        )}
      </div>

      <hr style={{ borderColor: "var(--border)", margin: "20px 0" }} />

      {/* Investigator Notes on Finding */}
      <div>
        <h4 style={{ fontSize: 14, fontWeight: 600, color: "var(--text-0)", marginBottom: 8 }}>
          Investigator Review Notes &amp; Justification
        </h4>
        <p className="muted mb-3" style={{ fontSize: 12 }}>
          Document why this finding was confirmed, rejected, or flagged for inclusion in the final court report:
        </p>

        {/* Note Composer */}
        <form onSubmit={onAddNote} className="mb-4">
          <div className="row gap-2 mb-2">
            <input
              className="input"
              style={{ maxWidth: 200, padding: "6px 10px", fontSize: 12 }}
              placeholder="Investigator name"
              value={authorName}
              onChange={e => setAuthorName(e.target.value)}
            />
            <input
              className="input"
              style={{ flex: 1, padding: "6px 12px", fontSize: 12 }}
              placeholder="Record your observation or justification..."
              value={noteText}
              onChange={e => setNoteText(e.target.value)}
            />
            <button type="submit" className="btn btn-primary btn-sm" disabled={savingNote || !noteText.trim()}>
              {savingNote ? "Saving..." : "+ Add Note"}
            </button>
          </div>
        </form>

        {/* Existing Notes List */}
        {parsedNotes.length === 0 ? (
          <div style={{ padding: 12, background: "var(--bg-0)", borderRadius: "var(--r-md)", border: "1px solid var(--border)", color: "var(--text-3)", fontSize: 12, textAlign: "center" }}>
            No investigator review notes recorded on this finding yet.
          </div>
        ) : (
          <div style={{ display: "flex", flexDirection: "column", gap: 8 }}>
            {parsedNotes.map((nt, idx) => (
              <div key={idx} style={{ padding: "10px 14px", background: "var(--bg-0)", borderRadius: "var(--r-md)", border: "1px solid var(--border)", fontSize: 12 }}>
                <span style={{ color: "var(--accent)", fontWeight: 600 }}>📝 Note #{idx + 1}</span>
                <div style={{ color: "var(--text-1)", marginTop: 4, whiteSpace: "pre-wrap" }}>{nt}</div>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
