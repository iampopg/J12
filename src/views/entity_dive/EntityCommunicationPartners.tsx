import { EntityDetail } from "./types";

interface Props {
  selectedEntity: EntityDetail;
  partnerFilter: string;
  onPartnerSelect: (partnerEmail: string) => void;
}

export function EntityCommunicationPartners({
  selectedEntity,
  partnerFilter,
  onPartnerSelect,
}: Props) {
  return (
    <div className="grid-2 mb-0" style={{ gap: 16 }}>
      {/* Top Sent To */}
      <div className="card mb-0" style={{ padding: 16 }}>
        <div className="row between mb-2">
          <strong style={{ fontSize: 12, color: "var(--text-0)" }}>
            📤 Communicated / Sent To (Click to Filter)
          </strong>
        </div>
        {selectedEntity.sent_to && selectedEntity.sent_to.length > 0 ? (
          <div style={{ display: "flex", flexDirection: "column", gap: 4 }}>
            {selectedEntity.sent_to.map(([email, count]) => {
              const isPartnerSelected = partnerFilter === email;
              return (
                <div
                  key={email}
                  className="row between tr-click"
                  style={{
                    padding: "6px 8px",
                    borderRadius: "var(--r-sm)",
                    background: isPartnerSelected
                      ? "var(--accent-subtle)"
                      : "var(--bg-3)",
                    border: isPartnerSelected
                      ? "1px solid var(--accent)"
                      : "1px solid transparent",
                  }}
                  onClick={() => onPartnerSelect(email)}
                >
                  <span
                    style={{
                      fontSize: 11,
                      fontFamily: "var(--mono)",
                      color: isPartnerSelected ? "var(--accent)" : "var(--text-1)",
                      overflow: "hidden",
                      textOverflow: "ellipsis",
                      whiteSpace: "nowrap",
                    }}
                  >
                    {email}
                  </span>
                  <span className="badge badge-blue">{count}</span>
                </div>
              );
            })}
          </div>
        ) : (
          <div className="muted text-sm">No sent communications</div>
        )}
      </div>

      {/* Top Received From */}
      <div className="card mb-0" style={{ padding: 16 }}>
        <div className="row between mb-2">
          <strong style={{ fontSize: 12, color: "var(--text-0)" }}>
            📥 Received From (Click to Filter)
          </strong>
        </div>
        {selectedEntity.received_from && selectedEntity.received_from.length > 0 ? (
          <div style={{ display: "flex", flexDirection: "column", gap: 4 }}>
            {selectedEntity.received_from.map(([email, count]) => {
              const isPartnerSelected = partnerFilter === email;
              return (
                <div
                  key={email}
                  className="row between tr-click"
                  style={{
                    padding: "6px 8px",
                    borderRadius: "var(--r-sm)",
                    background: isPartnerSelected
                      ? "var(--accent-subtle)"
                      : "var(--bg-3)",
                    border: isPartnerSelected
                      ? "1px solid var(--accent)"
                      : "1px solid transparent",
                  }}
                  onClick={() => onPartnerSelect(email)}
                >
                  <span
                    style={{
                      fontSize: 11,
                      fontFamily: "var(--mono)",
                      color: isPartnerSelected ? "var(--accent)" : "var(--text-1)",
                      overflow: "hidden",
                      textOverflow: "ellipsis",
                      whiteSpace: "nowrap",
                    }}
                  >
                    {email}
                  </span>
                  <span className="badge badge-gray">{count}</span>
                </div>
              );
            })}
          </div>
        ) : (
          <div className="muted text-sm">No received communications</div>
        )}
      </div>
    </div>
  );
}
