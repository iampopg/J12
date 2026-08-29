interface Props {
  dateRange: { min: string; max: string };
  stats: { total: number; peak: { date: string; total: number } };
  selectedPeriod: string | null;
}

export function TimelineAnalyticsCards({ dateRange, stats, selectedPeriod }: Props) {
  return (
    <div
      style={{
        display: "grid",
        gridTemplateColumns: "repeat(auto-fit, minmax(220px, 1fr))",
        gap: 12,
        marginBottom: 16,
      }}
    >
      <div className="card mb-0" style={{ padding: 14, background: "var(--bg-1)", border: "1px solid var(--border)", position: "relative", overflow: "hidden" }}>
        <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", marginBottom: 6 }}>
          <span style={{ fontSize: 11, color: "var(--text-3)", fontWeight: 700, textTransform: "uppercase", letterSpacing: "0.5px" }}>
            📅 Chronology Span
          </span>
          <span style={{ fontSize: 14 }}>⏱️</span>
        </div>
        <div style={{ fontSize: 13, fontWeight: 700, color: "var(--text-0)", fontFamily: "var(--mono)" }}>
          {dateRange.min ? dateRange.min.slice(0, 10) : "—"} <span style={{ color: "var(--accent)" }}>→</span> {dateRange.max ? dateRange.max.slice(0, 10) : "—"}
        </div>
      </div>

      <div className="card mb-0" style={{ padding: 14, background: "var(--bg-1)", border: "1px solid var(--border)", position: "relative", overflow: "hidden" }}>
        <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", marginBottom: 6 }}>
          <span style={{ fontSize: 11, color: "var(--text-3)", fontWeight: 700, textTransform: "uppercase", letterSpacing: "0.5px" }}>
            📊 Total Communications
          </span>
          <span style={{ fontSize: 14 }}>✉️</span>
        </div>
        <div style={{ fontSize: 20, fontWeight: 800, color: "var(--accent)" }}>
          {stats.total.toLocaleString()} <span style={{ fontSize: 11, fontWeight: 500, color: "var(--text-3)" }}>messages</span>
        </div>
      </div>

      <div className="card mb-0" style={{ padding: 14, background: "var(--bg-1)", border: "1px solid var(--border)", position: "relative", overflow: "hidden" }}>
        <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", marginBottom: 6 }}>
          <span style={{ fontSize: 11, color: "var(--text-3)", fontWeight: 700, textTransform: "uppercase", letterSpacing: "0.5px" }}>
            ⚡ Peak Activity Spike
          </span>
          <span style={{ fontSize: 14 }}>📈</span>
        </div>
        <div style={{ fontSize: 13, fontWeight: 700, color: "#f59e0b" }}>
          {stats.peak?.date || "—"} <span style={{ fontSize: 12, fontWeight: 500, color: "var(--text-2)" }}>({stats.peak?.total || 0} emails)</span>
        </div>
      </div>

      <div className="card mb-0" style={{ padding: 14, background: selectedPeriod ? "rgba(34, 197, 94, 0.08)" : "var(--bg-1)", border: selectedPeriod ? "1px solid #22c55e44" : "1px solid var(--border)", position: "relative", overflow: "hidden" }}>
        <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", marginBottom: 6 }}>
          <span style={{ fontSize: 11, color: "var(--text-3)", fontWeight: 700, textTransform: "uppercase", letterSpacing: "0.5px" }}>
            🔍 Active Period Filter
          </span>
          <span style={{ fontSize: 14 }}>🎯</span>
        </div>
        <div style={{ fontSize: 13, fontWeight: 700, color: selectedPeriod ? "#22c55e" : "var(--text-1)" }}>
          {selectedPeriod ? `Filtered: ${selectedPeriod}` : "All Dates (Full Archive)"}
        </div>
      </div>
    </div>
  );
}
