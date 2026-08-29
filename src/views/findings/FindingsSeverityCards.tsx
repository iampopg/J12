interface Props {
  findingsLength: number;
  severityCounts: { critical: number; high: number; medium: number; low: number };
  typeCounts: Record<string, number>;
  filterSeverity: string;
  setFilterSeverity: (s: string) => void;
  filterType: string;
  setFilterType: (t: string) => void;
}

export function FindingsSeverityCards({
  findingsLength,
  severityCounts,
  typeCounts,
  filterSeverity,
  setFilterSeverity,
  filterType,
  setFilterType,
}: Props) {
  return (
    <>
      {/* Severity Summary Cards */}
      <div className="row gap-4 mb-4" style={{ flexWrap: "wrap" }}>
        <div 
          style={{ 
            flex: 1, 
            minWidth: 140, 
            padding: 16, 
            background: "var(--bg-3)", 
            borderRadius: "var(--r-md)", 
            textAlign: "center", 
            borderLeft: "4px solid var(--danger)",
            cursor: "pointer"
          }}
          onClick={() => setFilterSeverity(filterSeverity === "critical" ? "all" : "critical")}
        >
          <div style={{ fontSize: 26, fontWeight: 800, color: "var(--danger)" }}>{severityCounts.critical}</div>
          <div className="muted text-sm" style={{ fontWeight: 600 }}>Critical Severity</div>
        </div>

        <div 
          style={{ 
            flex: 1, 
            minWidth: 140, 
            padding: 16, 
            background: "var(--bg-3)", 
            borderRadius: "var(--r-md)", 
            textAlign: "center", 
            borderLeft: "4px solid #f97316",
            cursor: "pointer"
          }}
          onClick={() => setFilterSeverity(filterSeverity === "high" ? "all" : "high")}
        >
          <div style={{ fontSize: 26, fontWeight: 800, color: "#f97316" }}>{severityCounts.high}</div>
          <div className="muted text-sm" style={{ fontWeight: 600 }}>High Threats</div>
        </div>

        <div 
          style={{ 
            flex: 1, 
            minWidth: 140, 
            padding: 16, 
            background: "var(--bg-3)", 
            borderRadius: "var(--r-md)", 
            textAlign: "center", 
            borderLeft: "4px solid #eab308",
            cursor: "pointer"
          }}
          onClick={() => setFilterSeverity(filterSeverity === "medium" ? "all" : "medium")}
        >
          <div style={{ fontSize: 26, fontWeight: 800, color: "#eab308" }}>{severityCounts.medium}</div>
          <div className="muted text-sm" style={{ fontWeight: 600 }}>Medium Risks</div>
        </div>

        <div 
          style={{ 
            flex: 1, 
            minWidth: 140, 
            padding: 16, 
            background: "var(--bg-3)", 
            borderRadius: "var(--r-md)", 
            textAlign: "center", 
            borderLeft: "4px solid #3b82f6",
            cursor: "pointer"
          }}
          onClick={() => setFilterSeverity(filterSeverity === "low" ? "all" : "low")}
        >
          <div style={{ fontSize: 26, fontWeight: 800, color: "#3b82f6" }}>{severityCounts.low}</div>
          <div className="muted text-sm" style={{ fontWeight: 600 }}>Low / Info</div>
        </div>
      </div>

      {/* Category Breakdown Pills */}
      <div className="row gap-2 mb-4" style={{ flexWrap: "wrap" }}>
        <button
          className={`btn btn-sm ${filterType === "all" ? "btn-primary" : "btn-ghost"}`}
          onClick={() => setFilterType("all")}
        >
          All Categories ({findingsLength})
        </button>
        {Object.entries(typeCounts).map(([type, count]) => (
          <button
            key={type}
            className={`btn btn-sm ${filterType === type ? "btn-primary" : "btn-ghost"}`}
            onClick={() => setFilterType(filterType === type ? "all" : type)}
            style={{ opacity: count === 0 ? 0.5 : 1 }}
          >
            {type}: {count}
          </button>
        ))}
      </div>
    </>
  );
}
