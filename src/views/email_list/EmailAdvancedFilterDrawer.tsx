interface Props {
  show: boolean;
  dateFilterMode: "all" | "single" | "range";
  setDateFilterMode: (m: "all" | "single" | "range") => void;
  singleDate: string;
  setSingleDate: (d: string) => void;
  startDate: string;
  setStartDate: (d: string) => void;
  endDate: string;
  setEndDate: (d: string) => void;
  tagFilter: string;
  setTagFilter: (t: string) => void;
  uniqueTags: string[];
  onResetFilters: () => void;
}

export function EmailAdvancedFilterDrawer({
  show,
  dateFilterMode,
  setDateFilterMode,
  singleDate,
  setSingleDate,
  startDate,
  setStartDate,
  endDate,
  setEndDate,
  tagFilter,
  setTagFilter,
  uniqueTags,
  onResetFilters,
}: Props) {
  if (!show) return null;

  return (
    <div
      className="card mb-3"
      style={{
        padding: "16px 20px",
        background: "var(--bg-2)",
        border: "1px solid var(--border)",
        display: "flex",
        flexDirection: "column",
        gap: 16,
      }}
    >
      <div className="row between" style={{ alignItems: "center" }}>
        <span style={{ fontWeight: 600, fontSize: 13, color: "var(--text-0)" }}>
          📅 Advanced Temporal &amp; Classification Filters
        </span>
        <button
          className="btn btn-ghost btn-sm"
          style={{ fontSize: 11 }}
          onClick={onResetFilters}
        >
          Reset All Filters
        </button>
      </div>

      {/* Mode Selection */}
      <div className="row gap-2">
        <button
          className={`btn btn-sm ${dateFilterMode === "all" ? "btn-primary" : "btn-ghost"}`}
          onClick={() => setDateFilterMode("all")}
        >
          All Dates
        </button>
        <button
          className={`btn btn-sm ${dateFilterMode === "single" ? "btn-primary" : "btn-ghost"}`}
          onClick={() => setDateFilterMode("single")}
        >
          Single Date
        </button>
        <button
          className={`btn btn-sm ${dateFilterMode === "range" ? "btn-primary" : "btn-ghost"}`}
          onClick={() => setDateFilterMode("range")}
        >
          Date Range
        </button>
      </div>

      {/* Single Date Picker */}
      {dateFilterMode === "single" && (
        <div className="row gap-2" style={{ alignItems: "center" }}>
          <span className="muted" style={{ fontSize: 12 }}>
            Pick Day:
          </span>
          <input
            type="date"
            className="input input-sm"
            value={singleDate}
            onChange={(e) => setSingleDate(e.target.value)}
          />
          {singleDate && (
            <button
              className="btn btn-ghost btn-sm"
              onClick={() => setSingleDate("")}
              style={{ fontSize: 11 }}
            >
              Clear
            </button>
          )}
        </div>
      )}

      {/* Date Range Picker */}
      {dateFilterMode === "range" && (
        <div className="row gap-2" style={{ alignItems: "center", flexWrap: "wrap" }}>
          <div className="row gap-1" style={{ alignItems: "center" }}>
            <span className="muted" style={{ fontSize: 12 }}>
              From:
            </span>
            <input
              type="date"
              className="input input-sm"
              value={startDate}
              onChange={(e) => setStartDate(e.target.value)}
            />
          </div>
          <div className="row gap-1" style={{ alignItems: "center" }}>
            <span className="muted" style={{ fontSize: 12 }}>
              To:
            </span>
            <input
              type="date"
              className="input input-sm"
              value={endDate}
              onChange={(e) => setEndDate(e.target.value)}
            />
          </div>
          {(startDate || endDate) && (
            <button
              className="btn btn-ghost btn-sm"
              onClick={() => {
                setStartDate("");
                setEndDate("");
              }}
              style={{ fontSize: 11 }}
            >
              Clear Range
            </button>
          )}
        </div>
      )}

      {/* Tag Filters */}
      {uniqueTags.length > 0 && (
        <div>
          <span className="muted" style={{ fontSize: 12, display: "block", marginBottom: 6 }}>
            Filter by Tag:
          </span>
          <div className="row gap-2" style={{ flexWrap: "wrap" }}>
            <button
              className={`btn btn-sm ${tagFilter === "all" ? "btn-primary" : "btn-ghost"}`}
              style={{ fontSize: 11 }}
              onClick={() => setTagFilter("all")}
            >
              All Tags
            </button>
            {uniqueTags.map((tagName) => (
              <button
                key={tagName}
                className={`btn btn-sm ${
                  tagFilter === tagName ? "btn-primary" : "btn-ghost"
                }`}
                style={{ fontSize: 11 }}
                onClick={() => setTagFilter(tagFilter === tagName ? "all" : tagName)}
              >
                🏷️ {tagName}
              </button>
            ))}
          </div>
        </div>
      )}
    </div>
  );
}
