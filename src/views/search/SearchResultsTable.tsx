import { SearchEmail, SortField, cleanDisplayName } from "./types";
import { BookmarkButton } from "../../components/BookmarkButton";

interface Props {
  caseId: string;
  results: SearchEmail[];
  selectedEmail: SearchEmail | null;
  sortField: SortField;
  sortDir: "asc" | "desc";
  onSort: (field: SortField) => void;
  onSelectEmail: (email: SearchEmail | null) => void;
  onViewEntity?: (email: string) => void;
}

export function SearchResultsTable({
  caseId,
  results,
  selectedEmail,
  sortField,
  sortDir,
  onSort,
  onSelectEmail,
  onViewEntity,
}: Props) {
  return (
    <div>
      {/* Results Header */}
      <div className="row between mb-2">
        <span className="muted" style={{ fontSize: 12 }}>
          Found <strong>{results.length.toLocaleString()}</strong> matching message{results.length !== 1 ? "s" : ""}
        </span>

        {/* Sort Controls */}
        <div className="row gap-2">
          <span className="muted" style={{ fontSize: 11 }}>Sort:</span>
          {(
            [
              ["date", "Date"],
              ["from", "Sender"],
              ["subject", "Subject"],
              ["risk", "Risk Score"],
              ["rank", "FTS5 Rank"],
            ] as const
          ).map(([field, label]) => (
            <button
              key={field}
              type="button"
              className={`btn btn-sm ${sortField === field ? "btn-primary" : "btn-ghost"}`}
              style={{ fontSize: 11, padding: "2px 8px" }}
              onClick={() => onSort(field as SortField)}
            >
              {label} {sortField === field ? (sortDir === "asc" ? "▲" : "▼") : ""}
            </button>
          ))}
        </div>
      </div>

      {/* Table Card */}
      <div
        className="card mb-0"
        style={{
          padding: 0,
          overflow: "hidden",
          borderRadius: "var(--r-md)",
          border: "1px solid var(--border)",
          background: "var(--bg-0)",
        }}
      >
        <div
          style={{
            display: "grid",
            gridTemplateColumns: "160px 180px 1fr 100px 55px 70px",
            padding: "9px 14px",
            background: "var(--bg-1)",
            borderBottom: "1px solid var(--border)",
            fontSize: 10.5,
            fontWeight: 700,
            textTransform: "uppercase",
            letterSpacing: "0.06em",
            color: "var(--text-3)",
            gap: 8,
            alignItems: "center",
          }}
        >
          <div>Sender / Origin</div>
          <div>Recipient(s)</div>
          <div>Subject &amp; FTS Match Snippet</div>
          <div style={{ textAlign: "right" }}>Date</div>
          <div style={{ textAlign: "center" }}>Risk</div>
          <div style={{ textAlign: "center" }}>Locker</div>
        </div>

        {/* Table Rows */}
        <div style={{ maxHeight: "68vh", overflowY: "auto" }}>
          {results.map((em) => {
            const isSelected = selectedEmail?.id === em.id;
            let toList: string[] = [];
            try {
              toList = em.to_addrs.startsWith("[")
                ? JSON.parse(em.to_addrs)
                : [em.to_addrs];
            } catch {
              toList = [em.to_addrs];
            }

            const senderClean = cleanDisplayName(em.from_display) || em.from_addr;

            return (
              <div
                key={em.id}
                className="tr-click"
                style={{
                  display: "grid",
                  gridTemplateColumns: "160px 180px 1fr 100px 55px 70px",
                  alignItems: "center",
                  padding: "9px 14px",
                  borderBottom: "1px solid var(--border)",
                  background: isSelected ? "var(--accent-subtle)" : "transparent",
                  fontSize: 12,
                  gap: 8,
                }}
                onClick={() => onSelectEmail(isSelected ? null : em)}
              >
                {/* Sender */}
                <div
                  style={{
                    overflow: "hidden",
                    textOverflow: "ellipsis",
                    whiteSpace: "nowrap",
                    color: "var(--text-0)",
                    fontWeight: 600,
                  }}
                  title={em.from_addr}
                >
                  {onViewEntity ? (
                    <span
                      style={{ cursor: "pointer", color: "var(--accent)" }}
                      onClick={(e) => {
                        e.stopPropagation();
                        onViewEntity(em.from_addr);
                      }}
                    >
                      {senderClean}
                    </span>
                  ) : (
                    senderClean
                  )}
                </div>

                {/* Recipients */}
                <div
                  style={{
                    overflow: "hidden",
                    textOverflow: "ellipsis",
                    whiteSpace: "nowrap",
                    color: "var(--text-2)",
                    fontSize: 11.5,
                  }}
                  title={toList.join(", ")}
                >
                  {toList.slice(0, 2).map(cleanDisplayName).join(", ") || "—"}
                  {toList.length > 2 && (
                    <span className="muted" style={{ marginLeft: 4 }}>
                      +{toList.length - 2}
                    </span>
                  )}
                </div>

                {/* Subject & Snippet */}
                <div style={{ overflow: "hidden", display: "flex", flexDirection: "column", gap: 2 }}>
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
                      <span className="badge badge-red" style={{ fontSize: 8.5, marginLeft: 6 }}>
                        DELETED
                      </span>
                    )}
                  </div>
                  {em.snippet && (
                    <div
                      style={{
                        fontSize: 11,
                        color: "var(--text-2)",
                        overflow: "hidden",
                        textOverflow: "ellipsis",
                        whiteSpace: "nowrap",
                        fontStyle: "italic",
                      }}
                      dangerouslySetInnerHTML={{ __html: em.snippet }}
                    />
                  )}
                </div>

                {/* Date */}
                <div
                  style={{
                    textAlign: "right",
                    fontSize: 11,
                    fontFamily: "var(--mono)",
                    color: "var(--text-3)",
                  }}
                >
                  {em.date_sent_utc ? em.date_sent_utc.slice(0, 10) : "—"}
                </div>

                {/* Risk */}
                <div style={{ textAlign: "center" }}>
                  <span
                    className={`badge ${
                      em.risk_score >= 50
                        ? "badge-red"
                        : em.risk_score >= 25
                        ? "badge-orange"
                        : "badge-green"
                    }`}
                    style={{ fontSize: 9.5, padding: "1px 5px" }}
                  >
                    {em.risk_score}
                  </span>
                </div>

                {/* Locker Bookmark */}
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
      </div>
    </div>
  );
}
