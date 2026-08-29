import { ItemBookmark, getItemTypeBadge } from "./types";
import { BookmarkButton } from "../../components/BookmarkButton";

interface Props {
  caseId: string;
  bookmarks: ItemBookmark[];
  onOpenItem: (b: ItemBookmark) => void;
  onRevealAttachment: (id: string) => void;
  onRefresh: () => void;
}

export function LockerGridView({
  caseId,
  bookmarks,
  onOpenItem,
  onRevealAttachment,
  onRefresh,
}: Props) {
  return (
    <div
      style={{
        display: "grid",
        gridTemplateColumns: "repeat(auto-fill, minmax(340px, 1fr))",
        gap: 16,
        overflowY: "auto",
        paddingRight: 4,
      }}
    >
      {bookmarks.map((b) => {
        const badge = getItemTypeBadge(b.item_type);
        return (
          <div
            key={b.id}
            className="card"
            style={{
              padding: 16,
              display: "flex",
              flexDirection: "column",
              justifyContent: "space-between",
              gap: 12,
              border: `1px solid var(--border)`,
              borderTop: `3px solid ${b.color || "var(--accent-blue)"}`,
              position: "relative",
              transition: "transform 0.15s ease, box-shadow 0.15s ease",
            }}
          >
            <div>
              {/* Top Bar: Item Type & Tag Pill */}
              <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: 10 }}>
                <span
                  style={{
                    fontSize: 10,
                    fontWeight: 800,
                    color: badge.color,
                    background: `${badge.color}15`,
                    padding: "2px 8px",
                    borderRadius: 4,
                    border: `1px solid ${badge.color}33`,
                    display: "flex",
                    alignItems: "center",
                    gap: 4,
                  }}
                >
                  <span>{badge.icon}</span> {badge.label}
                </span>

                {/* Tag pill */}
                <span
                  style={{
                    fontSize: 11,
                    fontWeight: 700,
                    color: b.color,
                    background: `${b.color}20`,
                    border: `1px solid ${b.color}50`,
                    padding: "2px 8px",
                    borderRadius: 999,
                    display: "flex",
                    alignItems: "center",
                    gap: 4,
                  }}
                >
                  🔖 {b.label}
                </span>
              </div>

              {/* Title / Subject */}
              <div
                style={{
                  fontSize: 14,
                  fontWeight: 700,
                  color: "var(--text-0)",
                  marginBottom: 6,
                  wordBreak: "break-word",
                  lineHeight: 1.4,
                }}
              >
                {b.item_title || (b.item_type === "email" ? "(No Subject)" : `Item ${b.item_id.slice(0, 8)}...`)}
              </div>

              {/* From & Date meta */}
              {(b.item_from || b.item_date) && (
                <div style={{ fontSize: 11, color: "var(--text-2)", marginBottom: 10 }}>
                  {b.item_from && <div>From: <strong style={{ color: "var(--text-1)" }}>{b.item_from}</strong></div>}
                  {b.item_date && <div>Date: {new Date(b.item_date).toLocaleString()}</div>}
                </div>
              )}

              {/* Investigator Note */}
              {b.note && b.note.trim().length > 0 && (
                <div
                  style={{
                    background: "var(--bg-2)",
                    borderLeft: `3px solid ${b.color || "var(--accent-blue)"}`,
                    borderRadius: "0 var(--r-sm) var(--r-sm) 0",
                    padding: "8px 12px",
                    fontSize: 12,
                    color: "var(--text-1)",
                    marginBottom: 10,
                    whiteSpace: "pre-wrap",
                    lineHeight: 1.4,
                  }}
                >
                  <div style={{ fontSize: 10, fontWeight: 700, color: "var(--text-2)", textTransform: "uppercase", marginBottom: 3 }}>
                    📝 Investigator Note:
                  </div>
                  {b.note}
                </div>
              )}
            </div>

            {/* Footer Actions */}
            <div
              style={{
                display: "flex",
                justifyContent: "space-between",
                alignItems: "center",
                paddingTop: 10,
                borderTop: "1px solid var(--border)",
                gap: 8,
              }}
            >
              <span style={{ fontSize: 10, color: "var(--text-2)" }}>
                Tagged {new Date(b.created_at).toLocaleDateString()}
              </span>

              <div style={{ display: "flex", gap: 6, alignItems: "center" }}>
                <BookmarkButton
                  caseId={caseId}
                  itemId={b.item_id}
                  itemType={b.item_type}
                  compact={true}
                  onChanged={onRefresh}
                />

                {b.item_type === "attachment" && (
                  <button
                    className="btn btn-ghost btn-sm"
                    style={{ fontSize: 11, padding: "3px 6px" }}
                    onClick={() => onRevealAttachment(b.item_id)}
                    title="Reveal in Finder / Explorer"
                  >
                    📁
                  </button>
                )}

                <button
                  className="btn btn-secondary btn-sm"
                  style={{ fontSize: 11, padding: "3px 10px", fontWeight: 600 }}
                  onClick={() => onOpenItem(b)}
                >
                  👁️ View {badge.label}
                </button>
              </div>
            </div>
          </div>
        );
      })}
    </div>
  );
}
