import { EntityEmail, TabType, cleanDisplayName } from "./types";
import { BookmarkButton } from "../../components/BookmarkButton";
import { RichEmailBodyViewer } from "../../components/RichEmailBodyViewer";

interface Props {
  caseId: string;
  activeTab: TabType;
  emails: EntityEmail[];
  emailsLoading: boolean;
  partnerFilter: string;
  onClearPartner: () => void;
  hasAttachment: boolean;
  onToggleAttachment: (v: boolean) => void;
  emailSearch: string;
  onSearchChange: (s: string) => void;
  dateFrom: string;
  onDateFromChange: (d: string) => void;
  dateTo: string;
  onDateToChange: (d: string) => void;
  selectedEmail: EntityEmail | null;
  onSelectEmail: (em: EntityEmail) => void;
  onOpenModal: (em: EntityEmail) => void;
  onClosePreview: () => void;
}

export function EntityMessagesExplorer({
  caseId,
  activeTab,
  emails,
  emailsLoading,
  partnerFilter,
  onClearPartner,
  hasAttachment,
  onToggleAttachment,
  emailSearch,
  onSearchChange,
  dateFrom,
  onDateFromChange,
  dateTo,
  onDateToChange,
  selectedEmail,
  onSelectEmail,
  onOpenModal,
  onClosePreview,
}: Props) {
  return (
    <div className="card mb-0" style={{ padding: 16 }}>
      <div className="row between mb-3">
        <div className="row gap-2">
          <strong style={{ fontSize: 13, color: "var(--text-0)" }}>
            📧 Messages (
            {activeTab === "sent"
              ? "Sent"
              : activeTab === "received"
              ? "Received"
              : activeTab === "deleted"
              ? "Deleted"
              : activeTab === "flagged"
              ? "Flagged"
              : "All"}
            : {emails.length})
          </strong>
          {partnerFilter && (
            <span
              className="badge badge-blue"
              style={{ cursor: "pointer" }}
              onClick={onClearPartner}
              title="Click to clear partner filter"
            >
              Thread with {partnerFilter} ✕
            </span>
          )}
        </div>

        <div className="row gap-2">
          <label className="row gap-1" style={{ fontSize: 11, color: "var(--text-2)", cursor: "pointer" }}>
            <input
              type="checkbox"
              checked={hasAttachment}
              onChange={(e) => onToggleAttachment(e.target.checked)}
            />
            Has Attachments
          </label>
        </div>
      </div>

      {/* Filter Controls Bar */}
      <div className="row gap-2 mb-3" style={{ flexWrap: "wrap" }}>
        <input
          className="input"
          style={{ flex: 1, minWidth: 200, fontSize: 12, padding: "6px 10px" }}
          placeholder="Search subject or body text..."
          value={emailSearch}
          onChange={(e) => onSearchChange(e.target.value)}
        />
        <div className="row gap-1">
          <input
            type="date"
            className="input"
            style={{ width: 140, fontSize: 11, padding: "5px 8px" }}
            value={dateFrom}
            onChange={(e) => onDateFromChange(e.target.value)}
            title="Date from"
          />
          <input
            type="date"
            className="input"
            style={{ width: 140, fontSize: 11, padding: "5px 8px" }}
            value={dateTo}
            onChange={(e) => onDateToChange(e.target.value)}
            title="Date to"
          />
        </div>
      </div>

      {/* Messages List Table */}
      {emailsLoading ? (
        <div className="empty">Loading emails...</div>
      ) : emails.length === 0 ? (
        <div className="empty">No emails match the selected filters</div>
      ) : (
        <div style={{ maxHeight: 380, overflowY: "auto", border: "1px solid var(--border)", borderRadius: "var(--r-md)" }}>
          <div
            style={{
              display: "grid",
              gridTemplateColumns: "150px 1fr 90px 55px 65px",
              padding: "8px 12px",
              background: "var(--bg-1)",
              borderBottom: "1px solid var(--border)",
              fontSize: 10,
              fontWeight: 700,
              textTransform: "uppercase",
              letterSpacing: "0.06em",
              color: "var(--text-3)",
            }}
          >
            <div>From</div>
            <div>Subject</div>
            <div style={{ textAlign: "right" }}>Date</div>
            <div style={{ textAlign: "center" }}>Risk</div>
            <div style={{ textAlign: "center" }}>Tag</div>
          </div>

          {emails.map((em) => {
            const isEmailActive = selectedEmail?.id === em.id;
            return (
              <div
                key={em.id}
                className="tr-click"
                style={{
                  display: "grid",
                  gridTemplateColumns: "150px 1fr 90px 55px 65px",
                  alignItems: "center",
                  padding: "8px 12px",
                  borderBottom: "1px solid var(--border)",
                  background: isEmailActive ? "var(--accent-subtle)" : "transparent",
                  fontSize: 12,
                }}
                onClick={() => onSelectEmail(em)}
              >
                <div
                  style={{
                    overflow: "hidden",
                    textOverflow: "ellipsis",
                    whiteSpace: "nowrap",
                    color: "var(--text-1)",
                  }}
                  title={em.from_addr}
                >
                  {cleanDisplayName(em.from_display) || em.from_addr}
                </div>
                <div
                  style={{
                    overflow: "hidden",
                    textOverflow: "ellipsis",
                    whiteSpace: "nowrap",
                    color: "var(--text-0)",
                    fontWeight: 500,
                  }}
                >
                  {em.subject || <span className="muted">(no subject)</span>}
                  {em.deleted_recovered && (
                    <span
                      className="badge badge-red"
                      style={{ fontSize: 9, marginLeft: 6 }}
                    >
                      DELETED
                    </span>
                  )}
                </div>
                <div style={{ textAlign: "right", fontSize: 11, color: "var(--text-3)" }}>
                  {em.date_sent_utc
                    ? new Date(em.date_sent_utc).toLocaleDateString()
                    : "—"}
                </div>
                <div style={{ textAlign: "center" }}>
                  <span
                    className={`badge ${
                      em.risk_score >= 50
                        ? "badge-red"
                        : em.risk_score >= 25
                        ? "badge-orange"
                        : "badge-green"
                    }`}
                    style={{ fontSize: 9 }}
                  >
                    {em.risk_score}
                  </span>
                </div>
                <div style={{ textAlign: "center" }} onClick={(e) => e.stopPropagation()}>
                  <BookmarkButton
                    caseId={caseId}
                    itemId={em.id}
                    itemType="email"
                    compact={true}
                  />
                </div>
              </div>
            );
          })}
        </div>
      )}

      {/* Inline Message Preview if Selected */}
      {selectedEmail && (
        <div
          style={{
            marginTop: 16,
            padding: 16,
            background: "var(--bg-1)",
            border: "1px solid var(--border)",
            borderRadius: "var(--r-md)",
            boxShadow: "0 4px 12px rgba(0,0,0,0.25)",
          }}
        >
          <div className="row between mb-2" style={{ alignItems: "flex-start", gap: 12 }}>
            <div style={{ flex: 1, minWidth: 0 }}>
              <h4 style={{ fontSize: 15, fontWeight: 700, color: "var(--text-0)", margin: 0, overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
                {selectedEmail.subject || "(no subject)"}
              </h4>
              <span className="muted" style={{ fontSize: 11 }}>Message ID: {selectedEmail.id}</span>
            </div>
            <div className="row gap-1" style={{ flexShrink: 0, alignItems: "center" }}>
              <BookmarkButton
                caseId={caseId}
                itemId={selectedEmail.id}
                itemType="email"
                compact={true}
              />
              <button
                className="btn btn-primary btn-sm"
                style={{ padding: "4px 10px", fontSize: 11 }}
                onClick={() => onOpenModal(selectedEmail)}
              >
                ✉️ Open Full Window
              </button>
              <button
                className="btn btn-ghost btn-sm"
                style={{ padding: "4px 8px", fontSize: 11 }}
                onClick={onClosePreview}
              >
                ✕ Close
              </button>
            </div>
          </div>

          <div style={{ background: "var(--bg-2)", padding: 10, borderRadius: "var(--r-sm)", marginBottom: 12, fontSize: 12, border: "1px solid var(--border)" }}>
            <div className="row mb-1">
              <span className="muted" style={{ width: 60, fontWeight: 600 }}>From:</span>
              <strong style={{ color: "#38bdf8" }}>{selectedEmail.from_display ? `${selectedEmail.from_display} <${selectedEmail.from_addr}>` : selectedEmail.from_addr}</strong>
            </div>
            <div className="row mb-1">
              <span className="muted" style={{ width: 60, fontWeight: 600 }}>To:</span>
              <span style={{ color: "var(--text-1)", wordBreak: "break-all" }}>{selectedEmail.to_addrs}</span>
            </div>
            {selectedEmail.cc_addrs && (
              <div className="row mb-1">
                <span className="muted" style={{ width: 60, fontWeight: 600 }}>Cc:</span>
                <span style={{ color: "var(--text-2)" }}>{selectedEmail.cc_addrs}</span>
              </div>
            )}
            <div className="row">
              <span className="muted" style={{ width: 60, fontWeight: 600 }}>Date:</span>
              <span style={{ color: "var(--text-2)", fontFamily: "var(--mono)" }}>
                {selectedEmail.date_sent_utc ? new Date(selectedEmail.date_sent_utc).toLocaleString() : "—"}
              </span>
            </div>
          </div>

          <RichEmailBodyViewer
            bodyText={selectedEmail.body_text}
            bodyHtml={selectedEmail.body_html}
            emailId={selectedEmail.id}
            defaultMode="rendered"
          />
        </div>
      )}
    </div>
  );
}
