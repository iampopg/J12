import { DailyRecord, MonthlyRecord } from "./types";

interface Props {
  granularity: "month" | "day";
  setGranularity: (g: "month" | "day") => void;
  pageOffset: number;
  setPageOffset: React.Dispatch<React.SetStateAction<number>>;
  itemsPerPage: number;
  monthlyData: MonthlyRecord[];
  dailyData: DailyRecord[];
  currentChartItems: (DailyRecord | MonthlyRecord)[];
  maxChartValue: number;
  selectedPeriod: string | null;
  onSelectPeriod: (period: string) => void;
}

export function TimelineHistogram({
  granularity,
  setGranularity,
  pageOffset,
  setPageOffset,
  itemsPerPage,
  monthlyData,
  dailyData,
  currentChartItems,
  maxChartValue,
  selectedPeriod,
  onSelectPeriod,
}: Props) {
  const totalCount = granularity === "month" ? monthlyData.length : dailyData.length;

  return (
    <div className="card mb-4" style={{ padding: "16px 20px", background: "var(--bg-1)", border: "1px solid var(--border)" }}>
      {/* Histogram Toolbar Controls */}
      <div className="row between mb-3" style={{ flexWrap: "wrap", gap: 10, alignItems: "center" }}>
        <div className="row gap-2" style={{ alignItems: "center" }}>
          <span style={{ fontSize: 11, fontWeight: 700, color: "var(--text-3)", textTransform: "uppercase", letterSpacing: "0.5px" }}>
            Granularity:
          </span>
          <div className="row gap-1" style={{ background: "var(--bg-2)", padding: 2, borderRadius: "var(--r-sm)", border: "1px solid var(--border)" }}>
            <button
              type="button"
              className={`btn btn-sm ${granularity === "month" ? "btn-primary" : "btn-ghost"}`}
              style={{ fontSize: 11, padding: "3px 12px", fontWeight: 600 }}
              onClick={() => {
                setGranularity("month");
                setPageOffset(0);
              }}
            >
              🗓️ By Month ({monthlyData.length})
            </button>
            <button
              type="button"
              className={`btn btn-sm ${granularity === "day" ? "btn-primary" : "btn-ghost"}`}
              style={{ fontSize: 11, padding: "3px 12px", fontWeight: 600 }}
              onClick={() => {
                setGranularity("day");
                setPageOffset(0);
              }}
            >
              📆 By Day ({dailyData.length})
            </button>
          </div>
        </div>

        <div className="row gap-3" style={{ alignItems: "center" }}>
          {/* Legend */}
          <div className="row gap-2" style={{ fontSize: 11, fontWeight: 600 }}>
            <span style={{ display: "inline-flex", alignItems: "center", gap: 4, color: "#38bdf8" }}>
              <span style={{ width: 8, height: 8, borderRadius: 2, background: "#38bdf8" }} /> Sent
            </span>
            <span style={{ display: "inline-flex", alignItems: "center", gap: 4, color: "#4ade80" }}>
              <span style={{ width: 8, height: 8, borderRadius: 2, background: "#4ade80" }} /> Received
            </span>
            <span style={{ display: "inline-flex", alignItems: "center", gap: 4, color: "#fbbf24" }}>
              <span style={{ width: 8, height: 8, borderRadius: "50%", background: "#fbbf24" }} /> Active Filter
            </span>
          </div>

          {/* Pagination Navigation */}
          <div className="row gap-1" style={{ alignItems: "center" }}>
            <button
              type="button"
              className="btn btn-ghost btn-sm"
              style={{ fontSize: 11, padding: "2px 8px" }}
              disabled={pageOffset <= 0}
              onClick={() => setPageOffset((prev) => Math.max(0, prev - itemsPerPage))}
              title="View earlier chronological period"
            >
              ◀ Earlier
            </button>
            <span className="muted" style={{ fontSize: 10, fontFamily: "var(--mono)", padding: "0 4px" }}>
              {totalCount > 0 ? `${pageOffset + 1}–${Math.min(pageOffset + itemsPerPage, totalCount)} of ${totalCount}` : "0"}
            </span>
            <button
              type="button"
              className="btn btn-ghost btn-sm"
              style={{ fontSize: 11, padding: "2px 8px" }}
              disabled={pageOffset + itemsPerPage >= totalCount}
              onClick={() => setPageOffset((prev) => prev + itemsPerPage)}
              title="View later chronological period"
            >
              Later ▶
            </button>
          </div>
        </div>
      </div>

      {/* Visual Bar Chart */}
      {currentChartItems.length === 0 ? (
        <div style={{ height: 120, display: "flex", alignItems: "center", justifyContent: "center", color: "var(--text-3)", fontSize: 12 }}>
          No communication events found for the active timeline filter.
        </div>
      ) : (
        <div
          style={{
            display: "grid",
            gridTemplateColumns: `repeat(${currentChartItems.length}, 1fr)`,
            gap: 10,
            alignItems: "end",
            height: 150,
            padding: "12px 6px 6px",
            borderBottom: "1px solid var(--border)",
            background: "rgba(0,0,0,0.15)",
            borderRadius: "var(--r-sm)",
          }}
        >
          {currentChartItems.map((item: any) => {
            const key = item.month || item.date;
            const isSelected = selectedPeriod === key;
            const heightPercent = Math.max(10, (item.total / maxChartValue) * 100);
            const sentRatio = item.total > 0 ? item.sent / item.total : 0;
            const recRatio = item.total > 0 ? item.received / item.total : 0;

            return (
              <div
                key={key}
                className="tr-click"
                style={{
                  display: "flex",
                  flexDirection: "column",
                  alignItems: "center",
                  height: "100%",
                  justifyContent: "flex-end",
                  padding: "0 2px",
                  cursor: "pointer",
                }}
                onClick={() => onSelectPeriod(key)}
                title={`📅 ${key}\nTotal: ${item.total.toLocaleString()} emails\n📤 Sent: ${item.sent.toLocaleString()}\n📥 Received: ${item.received.toLocaleString()}`}
              >
                {/* Count badge on top */}
                <div
                  style={{
                    fontSize: 9.5,
                    fontFamily: "var(--mono)",
                    color: isSelected ? "#fbbf24" : "var(--text-3)",
                    fontWeight: isSelected ? 800 : 600,
                    marginBottom: 4,
                  }}
                >
                  {item.total > 999 ? `${(item.total / 1000).toFixed(1)}k` : item.total}
                </div>

                {/* Stacked Activity Bar */}
                <div
                  style={{
                    width: "100%",
                    maxWidth: 36,
                    height: `${heightPercent}%`,
                    borderRadius: "4px 4px 0 0",
                    overflow: "hidden",
                    display: "flex",
                    flexDirection: "column-reverse",
                    border: isSelected ? "2px solid #fbbf24" : "1px solid rgba(255,255,255,0.08)",
                    boxShadow: isSelected ? "0 0 12px rgba(251, 191, 36, 0.6)" : "none",
                    transition: "all 0.15s ease",
                  }}
                >
                  {/* Received segment (Green) */}
                  <div
                    style={{
                      height: `${recRatio * 100}%`,
                      background: isSelected ? "#22c55e" : "rgba(34, 197, 94, 0.8)",
                    }}
                  />
                  {/* Sent segment (Blue) */}
                  <div
                    style={{
                      height: `${sentRatio * 100}%`,
                      background: isSelected ? "#0284c7" : "rgba(56, 189, 248, 0.8)",
                    }}
                  />
                </div>

                {/* Date / Month Label */}
                <div
                  style={{
                    fontSize: 10,
                    fontFamily: "var(--mono)",
                    color: isSelected ? "#fbbf24" : "var(--text-2)",
                    fontWeight: isSelected ? 800 : 500,
                    marginTop: 8,
                    whiteSpace: "nowrap",
                    overflow: "hidden",
                    textOverflow: "ellipsis",
                  }}
                >
                  {key.length === 10 ? key.slice(5) : key}
                </div>
              </div>
            );
          })}
        </div>
      )}
    </div>
  );
}
