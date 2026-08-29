import { useState, useEffect, useRef, useCallback, useMemo } from "react";
import {
  Email,
  EmailTag,
  SortField,
  SortDir,
  ColumnSettings,
  ColumnWidths,
  DEFAULT_COL_WIDTHS,
  cleanDisplayName,
} from "./types";
import { BookmarkButton } from "../../components/BookmarkButton";

interface Props {
  emails: Email[];
  tagsByEmail: Map<string, EmailTag[]>;
  sortField: SortField;
  sortDir: SortDir;
  onToggleSort: (field: SortField) => void;
  onSelect: (e: Email) => void;
  onViewEntity?: (email: string) => void;
  columns: ColumnSettings;
  caseId: string;
}

export function VirtualEmailList({
  emails,
  tagsByEmail,
  sortField,
  sortDir,
  onToggleSort,
  onSelect,
  columns,
  caseId,
}: Props) {
  const [scrollOffset, setScrollOffset] = useState(0);
  const containerRef = useRef<HTMLDivElement>(null);
  const rowHeight = 44;
  const visibleCount = 40;

  const [colWidths, setColWidths] = useState<ColumnWidths>(() => {
    try {
      const saved = localStorage.getItem("j12_email_col_widths");
      if (saved) return { ...DEFAULT_COL_WIDTHS, ...JSON.parse(saved) };
    } catch {}
    return DEFAULT_COL_WIDTHS;
  });

  const resizingRef = useRef<{ col: keyof ColumnWidths; startX: number; startW: number } | null>(null);

  const startResize = (col: keyof ColumnWidths, e: React.MouseEvent) => {
    e.preventDefault();
    e.stopPropagation();
    resizingRef.current = { col, startX: e.clientX, startW: colWidths[col] };

    const onMouseMove = (ev: MouseEvent) => {
      if (!resizingRef.current) return;
      const delta = ev.clientX - resizingRef.current.startX;
      const newWidth = Math.max(45, resizingRef.current.startW + delta);
      setColWidths((prev) => {
        const next = { ...prev, [resizingRef.current!.col]: newWidth };
        try {
          localStorage.setItem("j12_email_col_widths", JSON.stringify(next));
        } catch {}
        return next;
      });
    };

    const onMouseUp = () => {
      resizingRef.current = null;
      window.removeEventListener("mousemove", onMouseMove);
      window.removeEventListener("mouseup", onMouseUp);
    };

    window.addEventListener("mousemove", onMouseMove);
    window.addEventListener("mouseup", onMouseUp);
  };

  const handleScroll = useCallback(() => {
    if (containerRef.current) {
      setScrollOffset(containerRef.current.scrollTop);
    }
  }, []);

  useEffect(() => {
    const el = containerRef.current;
    if (el) {
      el.addEventListener("scroll", handleScroll);
      return () => el.removeEventListener("scroll", handleScroll);
    }
  }, [handleScroll]);

  const gridTemplate = useMemo(() => {
    const parts: string[] = [];
    if (columns.name) parts.push(`${colWidths.name}px`);
    if (columns.from) parts.push(`${colWidths.from}px`);
    if (columns.to) parts.push(`${colWidths.to}px`);
    if (columns.subject) parts.push(`${colWidths.subject}px`);
    if (columns.attachments) parts.push(`${colWidths.attachments}px`);
    if (columns.date) parts.push(`${colWidths.date}px`);
    if (columns.folder) parts.push(`${colWidths.folder}px`);
    if (columns.risk) parts.push(`${colWidths.risk}px`);
    if (columns.tag) parts.push(`${colWidths.tag}px`);
    return parts.length > 0 ? parts.join(" ") : "1fr";
  }, [columns, colWidths]);

  const totalHeight = emails.length * rowHeight;
  const startIdx = Math.max(0, Math.floor(scrollOffset / rowHeight));
  const endIdx = Math.min(emails.length, startIdx + visibleCount);
  const visibleEmails = emails.slice(startIdx, endIdx);

  const SortIcon = ({ field }: { field: SortField }) => (
    <span style={{ opacity: sortField === field ? 1 : 0.35, marginLeft: 4, fontSize: 11 }}>
      {sortField === field ? (sortDir === "asc" ? "▲" : "▼") : "⇅"}
    </span>
  );

  const Resizer = ({ col }: { col: keyof ColumnWidths }) => (
    <div
      style={{
        position: "absolute",
        right: 0,
        top: 0,
        bottom: 0,
        width: 8,
        cursor: "col-resize",
        zIndex: 5,
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
      }}
      onMouseDown={(e) => startResize(col, e)}
      onClick={(e) => e.stopPropagation()}
      title="Drag to resize column width"
    >
      <div style={{ width: 2, height: "60%", background: "var(--border)", borderRadius: 1 }} />
    </div>
  );

  if (emails.length === 0) {
    return (
      <div className="card" style={{ padding: 40, textAlign: "center" }}>
        <div style={{ fontSize: 32, marginBottom: 8 }}>🔍</div>
        <div style={{ fontSize: 15, fontWeight: 600, color: "var(--text-0)" }}>
          No emails match your criteria
        </div>
        <div className="muted text-sm mt-1">Try clearing your date or search filters.</div>
      </div>
    );
  }

  return (
    <div className="card" style={{ padding: 0, overflowX: "auto", overflowY: "hidden" }}>
      <div style={{ minWidth: "100%", width: "max-content" }}>
        {/* Interactive Sortable & Resizable Header */}
        <div
          style={{
            display: "grid",
            gridTemplateColumns: gridTemplate,
            alignItems: "center",
            padding: "10px 16px",
            background: "var(--bg-1)",
            borderBottom: "1px solid var(--border)",
            fontSize: 11,
            fontWeight: 600,
            textTransform: "uppercase",
            letterSpacing: "0.06em",
            color: "var(--text-3)",
            userSelect: "none",
            gap: 8,
          }}
        >
          {columns.name && (
            <div
              className="sort-header"
              style={{ position: "relative", paddingRight: 10 }}
              onClick={() => onToggleSort("name")}
              title="Click to sort by Name (A-Z / Z-A). Drag right edge to resize."
            >
              Name <SortIcon field="name" />
              <Resizer col="name" />
            </div>
          )}
          {columns.from && (
            <div
              className="sort-header"
              style={{ position: "relative", paddingRight: 10 }}
              onClick={() => onToggleSort("from")}
              title="Click to sort by Sender Email (A-Z / Z-A). Drag right edge to resize."
            >
              From <SortIcon field="from" />
              <Resizer col="from" />
            </div>
          )}
          {columns.to && (
            <div style={{ position: "relative", paddingRight: 10 }} title="Recipient Email. Drag right edge to resize.">
              To
              <Resizer col="to" />
            </div>
          )}
          {columns.subject && (
            <div
              className="sort-header"
              style={{ position: "relative", paddingRight: 10 }}
              onClick={() => onToggleSort("subject")}
              title="Click to sort by Subject (A-Z / Z-A). Drag right edge to resize."
            >
              Subject &amp; Tags <SortIcon field="subject" />
              <Resizer col="subject" />
            </div>
          )}
          {columns.attachments && (
            <div style={{ position: "relative", textAlign: "center", paddingRight: 10 }} title="Attachments &amp; Photos. Drag right edge to resize.">
              📎 Files
              <Resizer col="attachments" />
            </div>
          )}
          {columns.date && (
            <div
              className="sort-header"
              style={{ position: "relative", textAlign: "right", paddingRight: 10 }}
              onClick={() => onToggleSort("date")}
              title="Click to sort by Date (Newest / Oldest). Drag right edge to resize."
            >
              Date <SortIcon field="date" />
              <Resizer col="date" />
            </div>
          )}
          {columns.folder && (
            <div
              className="sort-header"
              style={{ position: "relative", textAlign: "center", paddingRight: 10 }}
              onClick={() => onToggleSort("folder")}
              title="Click to sort by Folder. Drag right edge to resize."
            >
              Folder <SortIcon field="folder" />
              <Resizer col="folder" />
            </div>
          )}
          {columns.risk && (
            <div
              className="sort-header"
              style={{ position: "relative", textAlign: "center", paddingRight: 10 }}
              onClick={() => onToggleSort("risk")}
              title="Click to sort by Risk Score. Drag right edge to resize."
            >
              Risk <SortIcon field="risk" />
              <Resizer col="risk" />
            </div>
          )}
          {columns.tag && (
            <div style={{ position: "relative", textAlign: "center", paddingRight: 10 }} title="Evidence Locker Bookmark. Drag right edge to resize.">
              Locker
              <Resizer col="tag" />
            </div>
          )}
        </div>

        {/* Virtual Scroll Area */}
        <div
          ref={containerRef}
          style={{ height: "60vh", overflowY: "auto", position: "relative" }}
        >
          <div style={{ height: totalHeight, position: "relative" }}>
            {visibleEmails.map((e, i) => {
              const emailTags = tagsByEmail.get(e.id) || [];
              const attCount = e.attachment_count || 0;
              const imgCount = e.image_count || 0;

              return (
                <div
                  key={e.id}
                  className="tr-click"
                  style={{
                    position: "absolute",
                    top: (startIdx + i) * rowHeight,
                    left: 0,
                    right: 0,
                    height: rowHeight,
                    display: "grid",
                    gridTemplateColumns: gridTemplate,
                    alignItems: "center",
                    padding: "0 16px",
                    borderBottom: "1px solid var(--border)",
                    fontSize: 13,
                    transition: "background 0.1s",
                    gap: 8,
                  }}
                  onClick={() => onSelect(e)}
                >
                  {/* Name */}
                  {columns.name && (
                    <div
                      style={{
                        overflow: "hidden",
                        textOverflow: "ellipsis",
                        whiteSpace: "nowrap",
                        color: "var(--text-1)",
                      }}
                      title={e.from_display || undefined}
                    >
                      {cleanDisplayName(e.from_display) || "—"}
                    </div>
                  )}

                  {/* From Email */}
                  {columns.from && (
                    <div
                      style={{
                        overflow: "hidden",
                        textOverflow: "ellipsis",
                        whiteSpace: "nowrap",
                        fontFamily: "var(--mono)",
                        fontSize: 11,
                        color: "var(--accent)",
                      }}
                      title={e.from_addr}
                    >
                      {e.from_addr}
                    </div>
                  )}

                  {/* To Recipient */}
                  {columns.to && (
                    <div
                      style={{
                        overflow: "hidden",
                        textOverflow: "ellipsis",
                        whiteSpace: "nowrap",
                        fontFamily: "var(--mono)",
                        fontSize: 11,
                        color: "var(--text-2)",
                      }}
                      title={e.to_addrs}
                    >
                      {e.to_addrs || "—"}
                    </div>
                  )}

                  {/* Subject & Tags */}
                  {columns.subject && (
                    <div
                      style={{
                        display: "flex",
                        alignItems: "center",
                        gap: 6,
                        overflow: "hidden",
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
                        title={e.subject || undefined}
                      >
                        {e.subject || <span className="muted">(no subject)</span>}
                      </span>
                      {emailTags.map((t) => (
                        <span
                          key={t.id}
                          className="badge"
                          style={{
                            background: `${t.color}22`,
                            color: t.color,
                            border: `1px solid ${t.color}44`,
                            fontSize: 9,
                            padding: "1px 5px",
                            whiteSpace: "nowrap",
                            flexShrink: 0,
                          }}
                        >
                          {t.tag}
                        </span>
                      ))}
                    </div>
                  )}

                  {/* Attachments / Photos Badge Indicator */}
                  {columns.attachments && (
                    <div style={{ display: "flex", justifyContent: "center", gap: 4, alignItems: "center" }}>
                      {attCount > 0 ? (
                        <span
                          className="badge badge-blue"
                          style={{
                            fontSize: 10,
                            padding: "1px 6px",
                            display: "inline-flex",
                            alignItems: "center",
                            gap: 2,
                            fontWeight: 700,
                          }}
                          title={`${attCount} total attachment(s)`}
                        >
                          📎 {attCount}
                        </span>
                      ) : null}

                      {imgCount > 0 ? (
                        <span
                          className="badge badge-green"
                          style={{
                            fontSize: 10,
                            padding: "1px 6px",
                            display: "inline-flex",
                            alignItems: "center",
                            gap: 2,
                            fontWeight: 700,
                          }}
                          title={`${imgCount} image attachment(s)`}
                        >
                          🖼️ {imgCount}
                        </span>
                      ) : null}

                      {attCount === 0 && imgCount === 0 && (
                        <span className="muted" style={{ opacity: 0.25, fontSize: 11 }}>—</span>
                      )}
                    </div>
                  )}

                  {/* Date */}
                  {columns.date && (
                    <div
                      style={{
                        textAlign: "right",
                        fontSize: 11,
                        color: "var(--text-3)",
                        whiteSpace: "nowrap",
                      }}
                    >
                      {e.date_sent ? new Date(e.date_sent).toLocaleDateString() : "—"}
                    </div>
                  )}

                  {/* Folder */}
                  {columns.folder && (
                    <div style={{ textAlign: "center" }}>
                      <span className="badge badge-gray" style={{ fontSize: 9 }}>
                        {e.folder_category}
                      </span>
                    </div>
                  )}

                  {/* Risk Score */}
                  {columns.risk && (
                    <div style={{ textAlign: "center" }}>
                      <span
                        className={`badge ${
                          e.risk_score >= 50
                            ? "badge-red"
                            : e.risk_score >= 25
                            ? "badge-orange"
                            : "badge-gray"
                        }`}
                        style={{ fontSize: 10, fontWeight: 700, minWidth: 26, textAlign: "center" }}
                      >
                        {e.risk_score}
                      </span>
                    </div>
                  )}

                  {/* Locker Bookmark Button */}
                  {columns.tag && (
                    <div
                      style={{ display: "flex", justifyContent: "center" }}
                      onClick={(ev) => ev.stopPropagation()}
                    >
                      <BookmarkButton caseId={caseId} itemId={e.id} itemType="email" compact={true} />
                    </div>
                  )}
                </div>
              );
            })}
          </div>
        </div>
      </div>
      <div style={{ padding: "8px 16px", background: "var(--bg-3)", fontSize: 11, color: "var(--text-3)", borderTop: "1px solid var(--border)" }}>
        Showing {emails.length.toLocaleString()} emails
      </div>
    </div>
  );
}
