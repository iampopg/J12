import { Evidence, SortField, SortDir } from "./types";

interface Props {
  evidence: Evidence[];
  selectedEvidenceId: string | null;
  onSelectEvidence: (id: string | null) => void;
  evidenceCounts: Record<string, number>;
  totalEmailsCount: number;
  q: string;
  setQ: (s: string) => void;
  sortField: SortField;
  sortDir: SortDir;
  onSortChange: (field: SortField, dir: SortDir) => void;
  hasActiveFilters: boolean;
  onResetFilters: () => void;
}

export function EmailFiltersBar({
  evidence,
  selectedEvidenceId,
  onSelectEvidence,
  evidenceCounts,
  totalEmailsCount,
  q,
  setQ,
  sortField,
  sortDir,
  onSortChange,
  hasActiveFilters,
  onResetFilters,
}: Props) {
  return (
    <>
      {/* Evidence Source Switcher Bar (Quick Filter) */}
      {evidence.length > 1 && (
        <div
          className="card mb-3"
          style={{
            padding: "8px 12px",
            display: "flex",
            alignItems: "center",
            gap: 8,
            flexWrap: "wrap",
            background: "var(--bg-2)",
            border: "1px solid var(--border)",
            borderRadius: "var(--r-md)",
          }}
        >
          <span style={{ fontSize: 11, fontWeight: 700, color: "var(--text-2)", display: "flex", alignItems: "center", gap: 4 }}>
            <span>📁 Source Filter:</span>
          </span>

          {/* All Sources Pill */}
          <button
            className={`btn btn-sm ${!selectedEvidenceId ? "btn-primary" : "btn-ghost"}`}
            style={{ fontSize: 11, padding: "3px 10px", borderRadius: "var(--r-sm)", fontWeight: !selectedEvidenceId ? 700 : 500 }}
            onClick={() => onSelectEvidence(null)}
          >
            🌐 All Sources ({totalEmailsCount.toLocaleString()})
          </button>

          {/* Individual Evidence Source Pills */}
          {evidence.map((ev) => {
            const isSelected = selectedEvidenceId === ev.id;
            const count = evidenceCounts[ev.id] || 0;
            const icon = ev.filename.includes("gmail") || ev.filename.includes("imap") ? "☁️" : ev.filename.endsWith(".mbox") ? "📦" : ev.filename.endsWith(".eml") ? "📧" : "📄";

            return (
              <button
                key={ev.id}
                className={`btn btn-sm ${isSelected ? "btn-primary" : "btn-ghost"}`}
                style={{
                  fontSize: 11,
                  padding: "3px 10px",
                  borderRadius: "var(--r-sm)",
                  display: "flex",
                  alignItems: "center",
                  gap: 6,
                  border: isSelected ? "1px solid var(--accent)" : "1px solid var(--border)",
                  background: isSelected ? "var(--accent)" : "var(--bg-3)",
                  color: isSelected ? "#fff" : "var(--text-1)",
                  fontWeight: isSelected ? 700 : 500,
                }}
                onClick={() => onSelectEvidence(isSelected ? null : ev.id)}
                title={`Switch view to only ${ev.filename}`}
              >
                <span>{icon}</span>
                <span style={{ maxWidth: 200, overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
                  {ev.filename}
                </span>
                <span
                  style={{
                    fontSize: 10,
                    padding: "1px 6px",
                    borderRadius: 10,
                    background: isSelected ? "rgba(0,0,0,0.25)" : "var(--bg-4)",
                    color: isSelected ? "#fff" : "var(--text-2)",
                    fontWeight: 600,
                  }}
                >
                  {count.toLocaleString()}
                </span>
              </button>
            );
          })}

          {selectedEvidenceId && (
            <button
              className="btn btn-ghost btn-sm"
              style={{ fontSize: 11, padding: "2px 8px", color: "var(--accent)", marginLeft: "auto" }}
              onClick={() => onSelectEvidence(null)}
            >
              ✕ Show All ({totalEmailsCount.toLocaleString()})
            </button>
          )}
        </div>
      )}

      {/* Quick Search and Active Filter Pills */}
      <div className="row gap-2 mb-3" style={{ flexWrap: "wrap" }}>
        <input
          className="input"
          style={{ flex: 1, minWidth: 260 }}
          placeholder="Search subject, sender email, display name, body..."
          value={q}
          onChange={(e) => setQ(e.target.value)}
        />

        {/* Quick Sort Selector */}
        <div className="row gap-1">
          <select
            className="select input"
            style={{ minWidth: 170, fontSize: 12, padding: "6px 10px" }}
            value={`${sortField}-${sortDir}`}
            onChange={(e) => {
              const [field, dir] = e.target.value.split("-") as [SortField, SortDir];
              onSortChange(field, dir);
            }}
          >
            <option value="date-desc">📅 Date (Newest First)</option>
            <option value="date-asc">📅 Date (Oldest First)</option>
            <option value="subject-asc">🔤 Subject (A → Z)</option>
            <option value="subject-desc">🔤 Subject (Z → A)</option>
            <option value="name-asc">👤 Name (A → Z)</option>
            <option value="name-desc">👤 Name (Z → A)</option>
            <option value="from-asc">✉️ Email (A → Z)</option>
            <option value="from-desc">✉️ Email (Z → A)</option>
            <option value="risk-desc">⚠️ Risk Score (Highest First)</option>
            <option value="folder-asc">📁 Folder Category</option>
          </select>
        </div>

        {hasActiveFilters && (
          <button
            className="btn btn-ghost btn-sm"
            style={{ color: "var(--danger)", fontSize: 12 }}
            onClick={onResetFilters}
          >
            ✕ Clear Filters
          </button>
        )}
      </div>
    </>
  );
}
