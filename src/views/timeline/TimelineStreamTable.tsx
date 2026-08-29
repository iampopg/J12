import { TimelineEmail, FilterCategory, cleanDisplayName } from "./types";
import { BookmarkButton } from "../../components/BookmarkButton";

interface Props {
  caseId: string;
  loadingEmails: boolean;
  filteredEmails: TimelineEmail[];
  selectedEmail: TimelineEmail | null;
  filterType: FilterCategory;
  setFilterType: (f: FilterCategory) => void;
  searchQuery: string;
  setSearchQuery: (q: string) => void;
  sortOrder: "desc" | "asc";
  setSortOrder: React.Dispatch<React.SetStateAction<"desc" | "asc">>;
  onSelectEmail: (em: TimelineEmail | null) => void;
}

export function TimelineStreamTable({
  caseId,
  loadingEmails,
  filteredEmails,
  selectedEmail,
  filterType,
  setFilterType,
  searchQuery,
  setSearchQuery,
  sortOrder,
  setSortOrder,
  onSelectEmail,
}: Props) {
  return (
    <div>
      {/* Stream Filter Toolbar */}
      <div className="row between mb-3" style={{ flexWrap: "wrap", gap: 10, alignItems: "center" }}>
        <div className="row gap-1" style={{ flexWrap: "wrap" }}>
          {(
            [
              ["all", `All (${filteredEmails.length})`],
              ["sent", "📤 Sent Only"],
              ["received", "📥 Received Only"],
              ["deleted", "🗑️ Deleted"],
              ["flagged", "🚨 High Risk"],
              ["after_hours", "🌙 After-Hours"],
            ] as const
          ).map(([key, label]) => (
            <button
              key={key}
              type="button"
              className={`btn btn-sm ${filterType === key ? "btn-primary" : "btn-ghost"}`}
              style={{ fontSize: 11, padding: "4px 10px", fontWeight: 600 }}
              onClick={() => setFilterType(key as FilterCategory)}
            >
              {label}
            </button>
          ))}
        </div>

        <div className="row gap-2" style={{ alignItems: "center" }}>
          <button
            type="button"
            className="btn btn-ghost btn-sm"
            style={{ fontSize: 11, padding: "4px 10px", fontWeight: 600 }}
            onClick={() => setSortOrder((s) => (s === "desc" ? "asc" : "desc"))}
          >
            {sortOrder === "desc" ? "⬇ Newest First" : "⬆ Oldest First"}
          </button>
        </div>
      </div>

      {/* Search inside stream */}
      <div className="mb-3">
        <input
          className="input"
          style={{ fontSize: 12, padding: "8px 14px", width: "100%", borderRadius: "var(--r-sm)" }}
          placeholder="Filter chronological stream by subject, sender, or recipient..."
          value={searchQuery}
          onChange={(e) => setSearchQuery(e.target.value)}
        />
      </div>

      {/* Stream Table */}
      {loadingEmails ? (
        <div className="empty" style={{ padding: 32 }}>Loading chronological email stream...</div>
      ) : filteredEmails.length === 0 ? (
        <div className="empty" style={{ padding: 32 }}>No emails match the selected timeline filters.</div>
      ) : (
        <div
          style={{
            maxHeight: "58vh",
            overflowY: "auto",
            border: "1px solid var(--border)",
            borderRadius: "var(--r-md)",
            background: "var(--bg-0)",
          }}
        >
          {/* Header */}
          <div
            style={{
              display: "grid",
              gridTemplateColumns: "140px 180px 1fr 65px 60px",
              padding: "10px 14px",
              background: "var(--bg-2)",
              borderBottom: "1px solid var(--border)",
              fontSize: 10.5,
              fontWeight: 700,
              textTransform: "uppercase",
              letterSpacing: "0.05em",
              color: "var(--text-3)",
              gap: 8,
              alignItems: "center",
              position: "sticky",
              top: 0,
              zIndex: 2,
            }}
          >
            <div>Date &amp; Time UTC</div>
            <div>Sender / Origin</div>
            <div>Subject &amp; Indicators</div>
            <div style={{ textAlign: "center" }}>Risk</div>
            <div style={{ textAlign: "center" }}>Locker</div>
          </div>

          {/* Rows */}
          {filteredEmails.map((em: TimelineEmail) => {
            const isSelected = selectedEmail?.id === em.id;
            const isSent = em.folder_category === "sent";

            return (
              <div
                key={em.id}
                className="tr-click"
                style={{
                  display: "grid",
                  gridTemplateColumns: "140px 180px 1fr 65px 60px",
                  alignItems: "center",
                  padding: "9px 14px",
                  borderBottom: "1px solid var(--border)",
                  background: isSelected ? "var(--accent-subtle)" : "transparent",
                  fontSize: 12.5,
                  gap: 8,
                  transition: "background 0.1s",
                }}
                onClick={() => onSelectEmail(isSelected ? null : em)}
              >
                {/* Timestamp */}
                <div style={{ display: "flex", flexDirection: "column", gap: 1, fontFamily: "var(--mono)", fontSize: 11 }}>
                  <span style={{ color: "var(--text-0)", fontWeight: 600 }}>
                    {em.date_sent_utc ? em.date_sent_utc.slice(0, 10) : "—"}
                  </span>
                  <span style={{ fontSize: 10, color: "var(--text-3)" }}>
                    {em.date_sent_utc ? new Date(em.date_sent_utc).toLocaleTimeString([], { hour: "2-digit", minute: "2-digit", second: "2-digit" }) : ""}
                  </span>
                </div>

                {/* Sender with Direction Pill Badge */}
                <div
                  style={{
                    display: "flex",
                    alignItems: "center",
                    gap: 6,
                    overflow: "hidden",
                    minWidth: 0,
                  }}
                  title={em.from_addr}
                >
                  <span
                    className={`badge ${isSent ? "badge-blue" : "badge-green"}`}
                    style={{
                      fontSize: 9,
                      fontWeight: 700,
                      padding: "2px 6px",
                      flexShrink: 0,
                      borderRadius: 4,
                    }}
                  >
                    {isSent ? "OUT" : "IN"}
                  </span>
                  <span
                    style={{
                      overflow: "hidden",
                      textOverflow: "ellipsis",
                      whiteSpace: "nowrap",
                      color: "var(--text-1)",
                      fontWeight: 500,
                    }}
                  >
                    {cleanDisplayName(em.from_display) || em.from_addr}
                  </span>
                </div>

                {/* Subject */}
                <div
                  style={{
                    display: "flex",
                    alignItems: "center",
                    gap: 6,
                    overflow: "hidden",
                    minWidth: 0,
                  }}
                >
                  <span
                    style={{
                      overflow: "hidden",
                      textOverflow: "ellipsis",
                      whiteSpace: "nowrap",
                      color: "var(--text-0)",
                      fontWeight: 500,
                    }}
                    title={em.subject || undefined}
                  >
                    {em.subject || <span className="muted">(no subject)</span>}
                  </span>
                  {em.deleted_recovered && (
                    <span className="badge badge-red" style={{ fontSize: 8.5, padding: "1px 5px", flexShrink: 0 }}>
                      DELETED
                    </span>
                  )}
                </div>

                {/* Risk Score */}
                <div style={{ textAlign: "center" }}>
                  <span
                    className={`badge ${
                      em.risk_score >= 50
                        ? "badge-red"
                        : em.risk_score >= 25
                        ? "badge-orange"
                        : "badge-green"
                    }`}
                    style={{ fontSize: 9.5, padding: "1px 6px", fontWeight: 700 }}
                  >
                    {em.risk_score}
                  </span>
                </div>

                {/* Tag / Locker Bookmark Button */}
                <div style={{ textAlign: "center", display: "flex", justifyContent: "center" }} onClick={(e) => e.stopPropagation()}>
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
    </div>
  );
}
