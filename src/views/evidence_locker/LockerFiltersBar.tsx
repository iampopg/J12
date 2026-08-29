interface Props {
  search: string;
  setSearch: (s: string) => void;
  sortBy: "newest" | "oldest" | "label" | "title";
  setSortBy: (s: "newest" | "oldest" | "label" | "title") => void;
  viewMode: "grid" | "table";
  setViewMode: (v: "grid" | "table") => void;
  typeFilter: string;
  setTypeFilter: (t: string) => void;
  tagFilter: string;
  setTagFilter: (t: string) => void;
  stats: {
    total: number;
    emails: number;
    attachments: number;
    artifacts: number;
    findings: number;
    tags: Array<{ label: string; color: string; count: number }>;
  };
}

export function LockerFiltersBar({
  search,
  setSearch,
  sortBy,
  setSortBy,
  viewMode,
  setViewMode,
  typeFilter,
  setTypeFilter,
  tagFilter,
  setTagFilter,
  stats,
}: Props) {
  return (
    <div
      className="card"
      style={{
        padding: 14,
        display: "flex",
        flexDirection: "column",
        gap: 12,
      }}
    >
      <div style={{ display: "flex", gap: 12, flexWrap: "wrap", alignItems: "center", justifyContent: "space-between" }}>
        {/* Search box */}
        <div style={{ flex: 1, minWidth: 260, position: "relative" }}>
          <input
            className="input"
            style={{ width: "100%", paddingLeft: 34, fontSize: 13 }}
            placeholder="Search by label, investigator note, subject, sender, filename..."
            value={search}
            onChange={(e) => setSearch(e.target.value)}
          />
          <span style={{ position: "absolute", left: 12, top: "50%", transform: "translateY(-50%)", fontSize: 14, color: "var(--text-2)" }}>
            🔍
          </span>
          {search && (
            <button
              style={{
                position: "absolute",
                right: 10,
                top: "50%",
                transform: "translateY(-50%)",
                background: "transparent",
                border: "none",
                color: "var(--text-2)",
                cursor: "pointer",
                fontSize: 12,
              }}
              onClick={() => setSearch("")}
            >
              ✕
            </button>
          )}
        </div>

        {/* Sort & View Mode */}
        <div style={{ display: "flex", gap: 10, alignItems: "center" }}>
          <div className="row gap-1" style={{ alignItems: "center" }}>
            <span style={{ fontSize: 12, color: "var(--text-2)" }}>Sort:</span>
            <select
              className="input input-sm"
              value={sortBy}
              onChange={(e) => setSortBy(e.target.value as any)}
              style={{ fontSize: 12, padding: "4px 8px" }}
            >
              <option value="newest">Newest First</option>
              <option value="oldest">Oldest First</option>
              <option value="label">Tag Label</option>
              <option value="title">Item Title</option>
            </select>
          </div>

          <div className="row gap-1" style={{ background: "var(--bg-2)", padding: 2, borderRadius: "var(--r-sm)", border: "1px solid var(--border)" }}>
            <button
              className={`btn btn-sm ${viewMode === "grid" ? "btn-primary" : "btn-ghost"}`}
              style={{ padding: "3px 8px", fontSize: 12 }}
              onClick={() => setViewMode("grid")}
              title="Grid / Card View"
            >
              🔲 Grid
            </button>
            <button
              className={`btn btn-sm ${viewMode === "table" ? "btn-primary" : "btn-ghost"}`}
              style={{ padding: "3px 8px", fontSize: 12 }}
              onClick={() => setViewMode("table")}
              title="Table View"
            >
              📋 Table
            </button>
          </div>
        </div>
      </div>

      {/* Type & Tag Filter Pills */}
      <div style={{ display: "flex", flexDirection: "column", gap: 8, paddingTop: 6, borderTop: "1px solid var(--border)" }}>
        {/* Item Type Filters */}
        <div style={{ display: "flex", gap: 6, flexWrap: "wrap", alignItems: "center" }}>
          <span style={{ fontSize: 11, fontWeight: 700, color: "var(--text-2)", textTransform: "uppercase", marginRight: 4 }}>
            Type:
          </span>
          {[
            { id: "all", label: "All Items", count: stats.total },
            { id: "email", label: "✉️ Emails", count: stats.emails },
            { id: "attachment", label: "📎 Attachments", count: stats.attachments },
            { id: "artifact", label: "🧩 Artifacts", count: stats.artifacts },
            { id: "finding", label: "🎯 Findings", count: stats.findings },
          ].map((t) => (
            <button
              key={t.id}
              className={`btn btn-sm ${typeFilter === t.id ? "btn-primary" : "btn-ghost"}`}
              style={{ fontSize: 11, padding: "2px 8px", borderRadius: 999 }}
              onClick={() => setTypeFilter(t.id)}
            >
              {t.label} <span style={{ opacity: 0.7, marginLeft: 4 }}>({t.count})</span>
            </button>
          ))}
        </div>

        {/* Tag / Label Filters */}
        {stats.tags.length > 0 && (
          <div style={{ display: "flex", gap: 6, flexWrap: "wrap", alignItems: "center" }}>
            <span style={{ fontSize: 11, fontWeight: 700, color: "var(--text-2)", textTransform: "uppercase", marginRight: 4 }}>
              Tag:
            </span>
            <button
              className={`btn btn-sm ${tagFilter === "all" ? "btn-primary" : "btn-ghost"}`}
              style={{ fontSize: 11, padding: "2px 8px", borderRadius: 999 }}
              onClick={() => setTagFilter("all")}
            >
              All Tags
            </button>
            {stats.tags.map((t) => (
              <button
                key={t.label}
                onClick={() => setTagFilter(tagFilter === t.label ? "all" : t.label)}
                style={{
                  fontSize: 11,
                  padding: "2px 10px",
                  borderRadius: 999,
                  background: tagFilter === t.label ? t.color : `${t.color}22`,
                  color: tagFilter === t.label ? "#ffffff" : t.color,
                  border: `1px solid ${t.color}`,
                  cursor: "pointer",
                  fontWeight: 600,
                  display: "flex",
                  alignItems: "center",
                  gap: 5,
                }}
              >
                <span style={{ width: 6, height: 6, borderRadius: "50%", background: tagFilter === t.label ? "#ffffff" : t.color }} />
                {t.label} ({t.count})
              </button>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
