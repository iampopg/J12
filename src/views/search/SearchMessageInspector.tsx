import { SearchEmail } from "./types";
import { BookmarkButton } from "../../components/BookmarkButton";

interface Props {
  caseId: string;
  selectedEmail: SearchEmail;
  onClose: () => void;
}

export function SearchMessageInspector({ caseId, selectedEmail, onClose }: Props) {
  return (
    <div
      className="card mb-0"
      style={{
        padding: 16,
        maxHeight: "72vh",
        overflowY: "auto",
        borderLeft: "4px solid var(--accent)",
      }}
    >
      <div className="row between mb-3" style={{ alignItems: "center" }}>
        <strong style={{ fontSize: 15, color: "var(--text-0)" }}>
          {selectedEmail.subject || "(no subject)"}
        </strong>
        <div className="row gap-2" style={{ alignItems: "center" }}>
          <BookmarkButton
            caseId={caseId}
            itemId={selectedEmail.id}
            itemType="email"
            compact={true}
          />
          <button
            className="btn btn-ghost btn-sm"
            style={{ padding: "2px 6px", fontSize: 11 }}
            onClick={onClose}
          >
            ✕ Close
          </button>
        </div>
      </div>

      {/* Metadata Grid */}
      <div
        style={{
          background: "var(--bg-3)",
          padding: 12,
          borderRadius: "var(--r-md)",
          display: "flex",
          flexDirection: "column",
          gap: 6,
          fontSize: 12,
          marginBottom: 12,
        }}
      >
        <div>
          <span className="muted">From: </span>
          <strong style={{ color: "var(--accent)" }}>
            {selectedEmail.from_display
              ? `${selectedEmail.from_display} <${selectedEmail.from_addr}>`
              : selectedEmail.from_addr}
          </strong>
        </div>

        <div>
          <span className="muted">To: </span>
          <span style={{ fontFamily: "var(--mono)", fontSize: 11 }}>
            {selectedEmail.to_addrs}
          </span>
        </div>

        {selectedEmail.cc_addrs && selectedEmail.cc_addrs !== "[]" && (
          <div>
            <span className="muted">CC: </span>
            <span style={{ fontFamily: "var(--mono)", fontSize: 11 }}>
              {selectedEmail.cc_addrs}
            </span>
          </div>
        )}

        <div className="row between" style={{ marginTop: 4 }}>
          <div>
            <span className="muted">Date: </span>
            {selectedEmail.date_sent_utc
              ? new Date(selectedEmail.date_sent_utc).toLocaleString()
              : "—"}
          </div>
          <div>
            <span className="muted">Risk: </span>
            <span
              className={`badge ${
                selectedEmail.risk_score >= 50
                  ? "badge-red"
                  : selectedEmail.risk_score >= 25
                  ? "badge-orange"
                  : "badge-green"
              }`}
            >
              {selectedEmail.risk_score}
            </span>
          </div>
        </div>
      </div>

      {/* Email Body Content */}
      <div>
        <span className="muted" style={{ fontSize: 11, fontWeight: 600 }}>
          MESSAGE BODY:
        </span>
        <pre
          style={{
            background: "var(--bg-1)",
            border: "1px solid var(--border)",
            borderRadius: "var(--r-md)",
            padding: 12,
            fontSize: 12,
            maxHeight: 250,
            overflow: "auto",
            whiteSpace: "pre-wrap",
            marginTop: 6,
            color: "var(--text-1)",
          }}
        >
          {selectedEmail.body_text || "(No message body)"}
        </pre>
      </div>

      {/* Collapsible Transport Headers */}
      {selectedEmail.headers_raw && (
        <details style={{ marginTop: 12 }}>
          <summary
            style={{
              cursor: "pointer",
              fontSize: 11,
              fontWeight: 600,
              color: "var(--text-3)",
            }}
          >
            View Raw Transport Headers
          </summary>
          <pre
            style={{
              background: "var(--bg-1)",
              border: "1px solid var(--border)",
              borderRadius: "var(--r-sm)",
              padding: 10,
              fontSize: 10,
              fontFamily: "var(--mono)",
              maxHeight: 180,
              overflow: "auto",
              whiteSpace: "pre-wrap",
              marginTop: 6,
              color: "var(--text-2)",
            }}
          >
            {selectedEmail.headers_raw.slice(0, 3000)}
          </pre>
        </details>
      )}
    </div>
  );
}
