import { GraphNode, GraphEdge, ExchangedEmail, cleanDisplayName } from "./types";

interface Props {
  selectedNode: GraphNode | null;
  selectedEdge: GraphEdge | null;
  connectedPartners: Array<{ id: string; name: string; count: number }>;
  inspectorEmails: ExchangedEmail[];
  loadingEmails: boolean;
  selectedEmail: ExchangedEmail | null;
  setSelectedEmail: (em: ExchangedEmail | null) => void;
  onPartnerClick: (partnerId: string) => void;
  onClearLinkFilter: () => void;
}

export function GraphInspector({
  selectedNode,
  selectedEdge,
  connectedPartners,
  inspectorEmails,
  loadingEmails,
  selectedEmail,
  setSelectedEmail,
  onPartnerClick,
  onClearLinkFilter,
}: Props) {
  if (!selectedNode) {
    return <div className="card empty">Click any entity on the canvas to inspect its relationships</div>;
  }

  return (
    <div
      className="card mb-0"
      style={{
        padding: 16,
        height: "72vh",
        overflowY: "auto",
        display: "flex",
        flexDirection: "column",
        gap: 14,
      }}
    >
      {/* Selected Node Summary */}
      <div
        style={{
          padding: 14,
          background: "var(--bg-3)",
          borderRadius: "var(--r-md)",
          borderLeft: selectedNode.is_target
            ? "4px solid #f59e0b"
            : "4px solid var(--accent)",
        }}
      >
        <div className="row between mb-2">
          <strong style={{ fontSize: 14, color: "var(--text-0)" }}>
            {cleanDisplayName(selectedNode.name) || selectedNode.id}
          </strong>
          {selectedNode.is_target && (
            <span className="badge badge-orange" style={{ fontSize: 10 }}>
              TARGET
            </span>
          )}
        </div>
        <div
          style={{
            fontSize: 11,
            color: "var(--accent)",
            fontFamily: "var(--mono)",
            marginBottom: 10,
          }}
        >
          {selectedNode.id}
        </div>

        <div className="grid-3" style={{ gap: 6, textAlign: "center" }}>
          <div style={{ background: "var(--bg-2)", padding: 6, borderRadius: "var(--r-xs)" }}>
            <div style={{ fontSize: 14, fontWeight: 700, color: "#3b82f6" }}>
              {selectedNode.sent}
            </div>
            <div style={{ fontSize: 9, color: "var(--text-3)" }}>SENT</div>
          </div>
          <div style={{ background: "var(--bg-2)", padding: 6, borderRadius: "var(--r-xs)" }}>
            <div style={{ fontSize: 14, fontWeight: 700, color: "#22c55e" }}>
              {selectedNode.received}
            </div>
            <div style={{ fontSize: 9, color: "var(--text-3)" }}>RECEIVED</div>
          </div>
          <div style={{ background: "var(--bg-2)", padding: 6, borderRadius: "var(--r-xs)" }}>
            <div style={{ fontSize: 14, fontWeight: 700, color: "var(--text-0)" }}>
              {selectedNode.total}
            </div>
            <div style={{ fontSize: 9, color: "var(--text-3)" }}>TOTAL</div>
          </div>
        </div>
      </div>

      {/* Connected Communication Partners */}
      <div>
        <div className="row between mb-2">
          <strong style={{ fontSize: 12, color: "var(--text-0)" }}>
            🔗 Direct Partners in Network ({connectedPartners.length})
          </strong>
        </div>
        {connectedPartners.length > 0 ? (
          <div style={{ display: "flex", flexDirection: "column", gap: 3, maxHeight: 130, overflowY: "auto" }}>
            {connectedPartners.map((p) => {
              const isPartnerSelected =
                selectedEdge &&
                ((selectedEdge.source === p.id && selectedEdge.target === selectedNode.id) ||
                  (selectedEdge.target === p.id && selectedEdge.source === selectedNode.id));

              return (
                <div
                  key={p.id}
                  className="row between tr-click"
                  style={{
                    padding: "5px 8px",
                    borderRadius: "var(--r-xs)",
                    background: isPartnerSelected
                      ? "var(--accent-subtle)"
                      : "var(--bg-3)",
                    border: isPartnerSelected
                      ? "1px solid var(--accent)"
                      : "1px solid transparent",
                  }}
                  onClick={() => onPartnerClick(p.id)}
                >
                  <span
                    style={{
                      fontSize: 11,
                      color: isPartnerSelected ? "var(--accent)" : "var(--text-1)",
                      overflow: "hidden",
                      textOverflow: "ellipsis",
                      whiteSpace: "nowrap",
                    }}
                  >
                    {p.name}
                  </span>
                  <span className="badge badge-blue" style={{ fontSize: 10 }}>
                    {p.count} emails
                  </span>
                </div>
              );
            })}
          </div>
        ) : (
          <div className="muted text-sm">No connected links within current threshold</div>
        )}
      </div>

      {/* Exchanged Messages Feed */}
      <div style={{ flex: 1, display: "flex", flexDirection: "column", minHeight: 0 }}>
        <div className="row between mb-2">
          <strong style={{ fontSize: 12, color: "var(--text-0)" }}>
            📧 {selectedEdge ? "Thread Between Partners" : "Recent Communications"} (
            {inspectorEmails.length})
          </strong>
          {selectedEdge && (
            <button
              className="btn btn-ghost btn-sm"
              style={{ fontSize: 10, padding: "1px 6px" }}
              onClick={onClearLinkFilter}
            >
              Clear Link Filter
            </button>
          )}
        </div>

        {loadingEmails ? (
          <div className="empty" style={{ padding: 16 }}>Loading messages...</div>
        ) : inspectorEmails.length === 0 ? (
          <div className="empty" style={{ padding: 16 }}>No exchanged emails found</div>
        ) : (
          <div
            style={{
              flex: 1,
              overflowY: "auto",
              border: "1px solid var(--border)",
              borderRadius: "var(--r-sm)",
            }}
          >
            {inspectorEmails.map((em) => {
              const isEmailActive = selectedEmail?.id === em.id;
              return (
                <div
                  key={em.id}
                  className="tr-click"
                  style={{
                    padding: "7px 10px",
                    borderBottom: "1px solid var(--border)",
                    background: isEmailActive ? "var(--accent-subtle)" : "transparent",
                    fontSize: 11,
                  }}
                  onClick={() => setSelectedEmail(isEmailActive ? null : em)}
                >
                  <div
                    style={{
                      fontWeight: 600,
                      color: "var(--text-0)",
                      overflow: "hidden",
                      textOverflow: "ellipsis",
                      whiteSpace: "nowrap",
                    }}
                  >
                    {em.subject || "(no subject)"}
                  </div>
                  <div className="row between mt-1" style={{ fontSize: 10, color: "var(--text-3)" }}>
                    <span style={{ overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap", maxWidth: 180 }}>
                      {cleanDisplayName(em.from_display) || em.from_addr}
                    </span>
                    <span>{em.date_sent_utc ? new Date(em.date_sent_utc).toLocaleDateString() : ""}</span>
                  </div>
                </div>
              );
            })}
          </div>
        )}

        {/* Inline Message Preview */}
        {selectedEmail && (
          <div
            style={{
              marginTop: 10,
              padding: 10,
              background: "var(--bg-1)",
              border: "1px solid var(--border)",
              borderRadius: "var(--r-sm)",
            }}
          >
            <div className="row between mb-1">
              <strong style={{ fontSize: 11, color: "var(--text-0)" }}>
                {selectedEmail.subject || "(no subject)"}
              </strong>
              <button
                className="btn btn-ghost btn-sm"
                style={{ fontSize: 9, padding: "0 4px" }}
                onClick={() => setSelectedEmail(null)}
              >
                ✕
              </button>
            </div>
            {selectedEmail.body_text && (
              <pre
                style={{
                  background: "var(--bg-0)",
                  padding: 8,
                  borderRadius: "var(--r-xs)",
                  fontSize: 10,
                  maxHeight: 90,
                  overflow: "auto",
                  whiteSpace: "pre-wrap",
                  marginTop: 4,
                }}
              >
                {selectedEmail.body_text}
              </pre>
            )}
          </div>
        )}
      </div>
    </div>
  );
}
