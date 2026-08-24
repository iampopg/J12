import { RichEmailBodyViewer } from "./RichEmailBodyViewer";

export interface EmailModalData {
  id: string;
  evidence_id?: string;
  case_id?: string;
  message_id: string | null;
  from_addr: string;
  from_display: string | null;
  to_addrs: string;
  cc_addrs?: string;
  subject: string | null;
  date_sent?: string | null;
  date_sent_utc?: string | null;
  headers_raw?: string | null;
  body_text?: string | null;
  body_html?: string | null;
  folder_name?: string | null;
  folder_category?: string;
}

interface Props {
  email: EmailModalData | null;
  onClose: () => void;
  titleSuffix?: string;
}

export function EmailDetailModal({ email, onClose, titleSuffix = "Return" }: Props) {
  if (!email) return null;

  return (
    <div
      style={{
        position: "fixed",
        inset: 0,
        background: "rgba(0, 0, 0, 0.82)",
        backdropFilter: "blur(6px)",
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
        zIndex: 10000,
        padding: 24,
      }}
      onClick={onClose}
    >
      <div
        style={{
          background: "#0f172a",
          border: "1px solid #334155",
          borderRadius: "var(--r-md)",
          width: "100%",
          maxWidth: 820,
          maxHeight: "90vh",
          overflowY: "auto",
          padding: 24,
          boxShadow: "0 25px 60px rgba(0,0,0,0.8)",
        }}
        onClick={(e) => e.stopPropagation()}
      >
        <div className="row between mb-3" style={{ borderBottom: "1px solid #1e293b", paddingBottom: 12 }}>
          <div className="row gap-2" style={{ alignItems: "center", overflow: "hidden", minWidth: 0, flex: 1 }}>
            <span style={{ fontSize: 18 }}>✉️</span>
            <div style={{ overflow: "hidden", minWidth: 0 }}>
              <h3 style={{ fontSize: 16, fontWeight: 700, margin: 0, color: "#f8fafc", overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
                {email.subject || "(No Subject)"}
              </h3>
              <span className="muted" style={{ fontSize: 11 }}>Message ID: {email.message_id || email.id}</span>
            </div>
          </div>
          <button className="btn btn-ghost btn-sm" style={{ flexShrink: 0, marginLeft: 12 }} onClick={onClose}>
            ✕ Close &amp; {titleSuffix}
          </button>
        </div>

        {/* Email Metadata Header Table */}
        <div style={{ background: "#1e293b", padding: 14, borderRadius: "var(--r-sm)", marginBottom: 16, fontSize: 12.5 }}>
          <div className="row mb-1">
            <span className="muted" style={{ width: 80, fontWeight: 600 }}>From:</span>
            <span style={{ color: "#38bdf8", fontWeight: 600 }}>{email.from_display ? `${email.from_display} <${email.from_addr}>` : email.from_addr}</span>
          </div>
          <div className="row mb-1">
            <span className="muted" style={{ width: 80, fontWeight: 600 }}>To:</span>
            <span style={{ color: "#e2e8f0" }}>{email.to_addrs}</span>
          </div>
          {email.cc_addrs && (
            <div className="row mb-1">
              <span className="muted" style={{ width: 80, fontWeight: 600 }}>Cc:</span>
              <span style={{ color: "#94a3b8" }}>{email.cc_addrs}</span>
            </div>
          )}
          <div className="row mb-1">
            <span className="muted" style={{ width: 80, fontWeight: 600 }}>Date UTC:</span>
            <span style={{ color: "#94a3b8", fontFamily: "var(--mono)" }}>{email.date_sent_utc || email.date_sent || "Unknown"}</span>
          </div>
          <div className="row">
            <span className="muted" style={{ width: 80, fontWeight: 600 }}>Folder:</span>
            <span className="badge badge-gray">{email.folder_name || email.folder_category || "inbox"}</span>
          </div>
        </div>

        {/* Email Body View */}
        <div style={{ marginBottom: 16 }}>
          <div className="label">EMAIL MESSAGE CONTENT</div>
          <RichEmailBodyViewer
            bodyText={email.body_text}
            bodyHtml={email.body_html}
            emailId={email.id}
            defaultMode="rendered"
          />
        </div>

        {/* Raw Headers Toggle Section */}
        {email.headers_raw && (
          <details style={{ marginTop: 12 }}>
            <summary style={{ cursor: "pointer", fontSize: 12, fontWeight: 600, color: "#94a3b8" }}>
              🧬 View Raw Transport Headers ({email.headers_raw.length} bytes)
            </summary>
            <pre 
              style={{ 
                marginTop: 8, 
                padding: 12, 
                background: "#020617", 
                border: "1px solid #1e293b", 
                borderRadius: "var(--r-sm)", 
                fontSize: 11, 
                color: "#64748b", 
                maxHeight: 200, 
                overflowY: "auto", 
                whiteSpace: "pre-wrap" 
              }}
            >
              {email.headers_raw}
            </pre>
          </details>
        )}
      </div>
    </div>
  );
}
