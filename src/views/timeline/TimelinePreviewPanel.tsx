import { useState } from "react";
import { TimelineEmail } from "./types";
import { BookmarkButton } from "../../components/BookmarkButton";
import { EmailDetailModal, EmailModalData } from "../../components/EmailDetailModal";
import { decodeQuotedPrintable } from "../../components/RichEmailBodyViewer";

interface Props {
  caseId: string;
  selectedEmail: TimelineEmail;
  onClose: () => void;
}

export function TimelinePreviewPanel({ caseId, selectedEmail, onClose }: Props) {
  const [modalOpen, setModalOpen] = useState(false);

  let toAddresses = selectedEmail.to_addrs;
  try {
    const p = JSON.parse(selectedEmail.to_addrs);
    if (Array.isArray(p)) toAddresses = p.join(", ");
  } catch {}

  const cleanBody = decodeQuotedPrintable(selectedEmail.body_text || "");

  const modalData: EmailModalData = {
    id: selectedEmail.id,
    evidence_id: selectedEmail.evidence_id,
    case_id: caseId,
    message_id: selectedEmail.message_id,
    from_addr: selectedEmail.from_addr,
    from_display: selectedEmail.from_display,
    to_addrs: selectedEmail.to_addrs,
    cc_addrs: selectedEmail.cc_addrs,
    subject: selectedEmail.subject,
    date_sent: selectedEmail.date_sent,
    date_sent_utc: selectedEmail.date_sent_utc,
    headers_raw: selectedEmail.headers_raw,
    body_text: selectedEmail.body_text,
    folder_name: selectedEmail.folder_name,
    folder_category: selectedEmail.folder_category,
  };

  return (
    <>
      {modalOpen && (
        <EmailDetailModal
          email={modalData}
          onClose={() => setModalOpen(false)}
        />
      )}

      <div
        className="card mb-0"
        style={{
          padding: 16,
          maxHeight: "58vh",
          overflowY: "auto",
          background: "var(--bg-1)",
          border: "1px solid var(--border)",
          borderLeft: "4px solid var(--accent)",
          boxShadow: "0 4px 14px rgba(0,0,0,0.2)",
        }}
      >
        <div className="row between mb-2" style={{ alignItems: "center" }}>
          <strong style={{ fontSize: 13, color: "var(--text-0)", overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap", maxWidth: 260 }}>
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
              ✕
            </button>
          </div>
        </div>

        <div
          style={{
            background: "var(--bg-2)",
            padding: 12,
            borderRadius: "var(--r-sm)",
            fontSize: 11.5,
            display: "flex",
            flexDirection: "column",
            gap: 6,
            marginBottom: 12,
            border: "1px solid var(--border)",
          }}
        >
          <div>
            <span className="muted" style={{ fontWeight: 600 }}>From: </span>
            <strong style={{ color: "#38bdf8" }}>
              {selectedEmail.from_display
                ? `${selectedEmail.from_display} <${selectedEmail.from_addr}>`
                : selectedEmail.from_addr}
            </strong>
          </div>
          <div>
            <span className="muted" style={{ fontWeight: 600 }}>To: </span>
            <span style={{ fontFamily: "var(--mono)", color: "var(--text-1)" }}>{toAddresses}</span>
          </div>
          <div>
            <span className="muted" style={{ fontWeight: 600 }}>Timestamp: </span>
            <span style={{ fontFamily: "var(--mono)" }}>
              {selectedEmail.date_sent_utc ? new Date(selectedEmail.date_sent_utc).toUTCString() : "—"}
            </span>
          </div>
          <div className="row between" style={{ alignItems: "center", marginTop: 2 }}>
            <div>
              <span className="muted" style={{ fontWeight: 600 }}>Risk: </span>
              <span
                className={`badge ${
                  selectedEmail.risk_score >= 50
                    ? "badge-red"
                    : selectedEmail.risk_score >= 25
                    ? "badge-orange"
                    : "badge-green"
                }`}
                style={{ fontSize: 10, padding: "1px 6px", fontWeight: 700 }}
              >
                {selectedEmail.risk_score}
              </span>
            </div>
            <button
              type="button"
              className="btn btn-primary btn-sm"
              style={{ fontSize: 11, padding: "3px 10px", fontWeight: 600 }}
              onClick={() => setModalOpen(true)}
            >
              ✉️ Open Full View
            </button>
          </div>
        </div>

        {/* Message Body */}
        <pre
          style={{
            background: "var(--bg-0)",
            border: "1px solid var(--border)",
            borderRadius: "var(--r-sm)",
            padding: 12,
            fontSize: 12,
            maxHeight: 220,
            overflow: "auto",
            whiteSpace: "pre-wrap",
            wordBreak: "break-word",
            fontFamily: "var(--font-sans)",
            color: "var(--text-1)",
            lineHeight: 1.5,
          }}
        >
          {cleanBody || "(No text content preview available. Click Open Full View to render full email)"}
        </pre>
      </div>
    </>
  );
}
