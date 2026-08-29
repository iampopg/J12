import { ItemBookmark, getItemTypeBadge } from "./types";
import { BookmarkButton } from "../../components/BookmarkButton";

interface Props {
  caseId: string;
  bookmarks: ItemBookmark[];
  onOpenItem: (b: ItemBookmark) => void;
  onRefresh: () => void;
}

export function LockerTableView({
  caseId,
  bookmarks,
  onOpenItem,
  onRefresh,
}: Props) {
  return (
    <div className="card" style={{ padding: 0, overflow: "hidden", flex: 1, display: "flex", flexDirection: "column" }}>
      <div style={{ overflowX: "auto", flex: 1 }}>
        <table className="table" style={{ width: "100%", fontSize: 12 }}>
          <thead>
            <tr>
              <th style={{ width: 100 }}>Type</th>
              <th style={{ width: 140 }}>Tag Label</th>
              <th>Title / Subject</th>
              <th>Sender / Source</th>
              <th>Investigator Note</th>
              <th style={{ width: 130 }}>Date Added</th>
              <th style={{ width: 130, textAlign: "right" }}>Actions</th>
            </tr>
          </thead>
          <tbody>
            {bookmarks.map((b) => {
              const badge = getItemTypeBadge(b.item_type);
              return (
                <tr key={b.id}>
                  <td>
                    <span
                      style={{
                        fontSize: 10,
                        fontWeight: 800,
                        color: badge.color,
                        background: `${badge.color}15`,
                        padding: "2px 6px",
                        borderRadius: 4,
                        border: `1px solid ${badge.color}33`,
                        display: "inline-flex",
                        alignItems: "center",
                        gap: 3,
                      }}
                    >
                      {badge.icon} {badge.label}
                    </span>
                  </td>
                  <td>
                    <span
                      style={{
                        fontSize: 11,
                        fontWeight: 700,
                        color: b.color,
                        background: `${b.color}20`,
                        border: `1px solid ${b.color}50`,
                        padding: "2px 8px",
                        borderRadius: 999,
                        display: "inline-flex",
                        alignItems: "center",
                        gap: 4,
                      }}
                    >
                      🔖 {b.label}
                    </span>
                  </td>
                  <td>
                    <strong style={{ color: "var(--text-0)" }}>
                      {b.item_title || (b.item_type === "email" ? "(No Subject)" : b.item_id.slice(0, 12))}
                    </strong>
                  </td>
                  <td style={{ color: "var(--text-1)" }}>
                    {b.item_from || "-"}
                  </td>
                  <td>
                    {b.note ? (
                      <span style={{ color: "var(--text-1)", fontStyle: "italic" }}>
                        "{b.note.length > 50 ? b.note.slice(0, 50) + "..." : b.note}"
                      </span>
                    ) : (
                      <span style={{ color: "var(--text-2)" }}>-</span>
                    )}
                  </td>
                  <td style={{ color: "var(--text-2)", fontSize: 11 }}>
                    {new Date(b.created_at).toLocaleString()}
                  </td>
                  <td style={{ textAlign: "right" }}>
                    <div style={{ display: "inline-flex", gap: 4 }}>
                      <BookmarkButton
                        caseId={caseId}
                        itemId={b.item_id}
                        itemType={b.item_type}
                        compact={true}
                        onChanged={onRefresh}
                      />
                      <button
                        className="btn btn-secondary btn-sm"
                        style={{ fontSize: 11, padding: "2px 8px" }}
                        onClick={() => onOpenItem(b)}
                      >
                        Open
                      </button>
                    </div>
                  </td>
                </tr>
              );
            })}
          </tbody>
        </table>
      </div>
    </div>
  );
}
