import { quickPresets, operatorChips } from "./types";

interface Props {
  query: string;
  setQuery: (q: string | ((prev: string) => string)) => void;
  loading: boolean;
  inputRef: React.RefObject<HTMLInputElement>;
  onSearch: (q?: string) => void;
  searchMetrics?: { hits: number; ms: number } | null;
}

export function SearchBar({
  query,
  setQuery,
  loading,
  inputRef,
  onSearch,
  searchMetrics,
}: Props) {
  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Enter") onSearch();
  };

  const handleQuickSearch = (q: string) => {
    setQuery(q);
    onSearch(q);
  };

  const handleAddOperator = (op: string) => {
    setQuery((prev) => {
      const trimmed = prev.trim();
      return trimmed ? `${trimmed} ${op}` : op;
    });
    inputRef.current?.focus();
  };

  return (
    <div className="card mb-3" style={{ padding: 16, background: "var(--bg-1)", border: "1px solid var(--border)" }}>
      <div className="row gap-2 mb-3">
        <input
          ref={inputRef}
          className="input"
          style={{ flex: 1, fontSize: 13.5, padding: "10px 14px", fontFamily: "var(--font-sans)" }}
          placeholder='FTS5 Full-Text Query (e.g. fraud AND wire, "offshore account", NEAR("bribe" "payment", 5), crypt*)...'
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          onKeyDown={handleKeyDown}
        />
        <button
          className="btn btn-primary"
          style={{ padding: "0 22px", fontWeight: 700 }}
          onClick={() => onSearch()}
          disabled={loading}
        >
          {loading ? "Searching..." : "⚡ FTS5 Search"}
        </button>
        {query && (
          <button
            className="btn btn-ghost"
            onClick={() => {
              setQuery("");
              onSearch("");
            }}
          >
            Clear
          </button>
        )}
      </div>

      {/* Metrics Banner */}
      {searchMetrics && (
        <div className="row between mb-3" style={{ fontSize: 11.5, color: "var(--text-2)", background: "var(--bg-2)", padding: "6px 12px", borderRadius: "var(--r-sm)", border: "1px solid var(--border)" }}>
          <span>
            Found <strong style={{ color: "var(--accent)" }}>{searchMetrics.hits.toLocaleString()}</strong> results in <strong style={{ color: "#22c55e" }}>{searchMetrics.ms.toFixed(1)}ms</strong>
          </span>
          <span style={{ fontSize: 10.5, color: "var(--text-3)", fontFamily: "var(--mono)" }}>
            ⚡ SQLite FTS5 Engine · Porter Stemmed
          </span>
        </div>
      )}

      {/* Quick Search Preset Badges */}
      <div className="row gap-2 mb-3" style={{ flexWrap: "wrap", alignItems: "center" }}>
        <span style={{ fontSize: 10.5, fontWeight: 700, color: "var(--text-3)", letterSpacing: "0.5px" }}>
          PRESETS:
        </span>
        {quickPresets.map((preset) => (
          <button
            key={preset.label}
            className="btn btn-ghost btn-sm"
            style={{
              fontSize: 11,
              padding: "3px 9px",
              background: query === preset.query ? "var(--accent-subtle)" : "var(--bg-3)",
              border: query === preset.query ? "1px solid var(--accent)" : "1px solid var(--border)",
            }}
            onClick={() => handleQuickSearch(preset.query)}
          >
            {preset.label}
          </button>
        ))}
      </div>

      {/* Search Operator Helper Chips */}
      <div>
        <details style={{ fontSize: 11.5, color: "var(--text-3)" }}>
          <summary style={{ cursor: "pointer", fontWeight: 700, userSelect: "none", color: "var(--text-2)" }}>
            💡 Boolean &amp; Proximity Operators Guide (Click to insert)
          </summary>
          <div
            style={{
              display: "grid",
              gridTemplateColumns: "repeat(auto-fill, minmax(210px, 1fr))",
              gap: 8,
              marginTop: 10,
              padding: 10,
              background: "var(--bg-0)",
              borderRadius: "var(--r-sm)",
              border: "1px solid var(--border)",
            }}
          >
            {operatorChips.map((chip) => (
              <div
                key={chip.op}
                className="tr-click"
                style={{
                  padding: "6px 8px",
                  borderRadius: "var(--r-xs)",
                  border: "1px solid var(--border)",
                  background: "var(--bg-2)",
                  cursor: "pointer",
                }}
                onClick={() => handleAddOperator(chip.op)}
                title={`Click to insert ${chip.op}`}
              >
                <code style={{ fontSize: 11, color: "#38bdf8", fontWeight: 700 }}>
                  {chip.op}
                </code>
                <div style={{ fontSize: 10.5, color: "var(--text-3)", marginTop: 2 }}>
                  {chip.desc}
                </div>
              </div>
            ))}
          </div>
        </details>
      </div>
    </div>
  );
}
