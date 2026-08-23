import { useState, useEffect, useRef, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";

interface TimelineDay {
  date: string;
  total: number;
  sent: number;
  received: number;
}

interface EmailBrief {
  id: string;
  from_addr: string;
  from_display: string | null;
  subject: string | null;
  date: string;
  folder: string;
  risk: number;
}

interface Props {
  caseId: string;
  onSelectEmail?: (id: string) => void;
}

export function TimelineView({ caseId, onSelectEmail }: Props) {
  const [data, setData] = useState<TimelineDay[]>([]);
  const [loading, setLoading] = useState(true);
  const [dateRange, setDateRange] = useState<{ min: string; max: string }>({ min: "", max: "" });
  const [viewStart, setViewStart] = useState(0);
  const [viewEnd, setViewEnd] = useState(100);
  const [selectedDay, setSelectedDay] = useState<TimelineDay | null>(null);
  const [dayEmails, setDayEmails] = useState<EmailBrief[]>([]);
  const [loadingEmails, setLoadingEmails] = useState(false);
  const [filterType, setFilterType] = useState<"all" | "sent" | "received">("all");
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const containerRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    loadData();
  }, [caseId]);

  useEffect(() => {
    drawTimeline();
  }, [data, viewStart, viewEnd]);

  const loadData = async () => {
    setLoading(true);
    try {
      const res = await invoke<any>("timeline_data", { input: { case_id: caseId } });
      setData(res.daily || []);
      if (res.date_range) {
        setDateRange({ min: res.date_range.min || "", max: res.date_range.max || "" });
      }
      setViewStart(0);
      setViewEnd((res.daily || []).length);
    } catch (e) {
      console.error("Failed to load timeline:", e);
    }
    setLoading(false);
  };

  const drawTimeline = useCallback(() => {
    const canvas = canvasRef.current;
    if (!canvas || data.length === 0) return;

    const ctx = canvas.getContext("2d");
    if (!ctx) return;

    const dpr = window.devicePixelRatio || 1;
    const rect = canvas.getBoundingClientRect();
    canvas.width = rect.width * dpr;
    canvas.height = rect.height * dpr;
    ctx.scale(dpr, dpr);

    const width = rect.width;
    const height = rect.height;
    const padding = { top: 30, right: 20, bottom: 40, left: 20 };
    const chartWidth = width - padding.left - padding.right;
    const chartHeight = height - padding.top - padding.bottom;

    // Clear
    ctx.fillStyle = "#151a23";
    ctx.fillRect(0, 0, width, height);

    // Get visible data slice
    const start = Math.max(0, Math.floor(viewStart));
    const end = Math.min(data.length, Math.ceil(viewEnd));
    const visible = data.slice(start, end);

    if (visible.length === 0) return;

    const maxValue = Math.max(...visible.map(d => d.total), 1);
    const barWidth = Math.max(2, chartWidth / visible.length - 1);

    // Draw grid lines
    ctx.strokeStyle = "#2a3344";
    ctx.lineWidth = 1;
    for (let i = 0; i <= 4; i++) {
      const y = padding.top + (chartHeight / 4) * i;
      ctx.beginPath();
      ctx.moveTo(padding.left, y);
      ctx.lineTo(width - padding.right, y);
      ctx.stroke();

      ctx.fillStyle = "#64748b";
      ctx.font = "10px system-ui";
      ctx.textAlign = "right";
      ctx.fillText(String(Math.round(maxValue * (4 - i) / 4)), padding.left - 5, y + 4);
    }

    // Draw bars
    visible.forEach((day, i) => {
      const x = padding.left + (i / visible.length) * chartWidth;
      const barHeight = (day.total / maxValue) * chartHeight;
      const y = padding.top + chartHeight - barHeight;

      // Stacked bars for sent/received
      const receivedHeight = (day.received / maxValue) * chartHeight;
      const sentHeight = (day.sent / maxValue) * chartHeight;

      // Received (green)
      ctx.fillStyle = "#22c55e";
      ctx.fillRect(x, padding.top + chartHeight - receivedHeight, barWidth, receivedHeight);

      // Sent (blue) - stacked on top of received
      ctx.fillStyle = "#3b82f6";
      ctx.fillRect(x, padding.top + chartHeight - receivedHeight - sentHeight, barWidth, sentHeight);

      // Selection highlight
      if (selectedDay && selectedDay.date === day.date) {
        ctx.strokeStyle = "#fbbf24";
        ctx.lineWidth = 2;
        ctx.strokeRect(x - 1, y - 1, barWidth + 2, barHeight + 2);
      }
    });

    // X-axis labels (show every Nth date)
    const labelInterval = Math.max(1, Math.floor(visible.length / 8));
    ctx.fillStyle = "#64748b";
    ctx.font = "10px system-ui";
    ctx.textAlign = "center";
    visible.forEach((day, i) => {
      if (i % labelInterval === 0 || i === visible.length - 1) {
        const x = padding.left + (i / visible.length) * chartWidth + barWidth / 2;
        const date = day.date.slice(5); // MM-DD
        ctx.fillText(date, x, height - padding.bottom + 20);
      }
    });

    // Title
    ctx.fillStyle = "#94a3b8";
    ctx.font = "11px system-ui";
    ctx.textAlign = "left";
    ctx.fillText("Email Activity Timeline (Green=Received, Blue=Sent)", padding.left, 15);

  }, [data, viewStart, viewEnd, selectedDay]);

  const handleCanvasClick = (e: React.MouseEvent<HTMLCanvasElement>) => {
    const canvas = canvasRef.current;
    if (!canvas || data.length === 0) return;

    const rect = canvas.getBoundingClientRect();
    const x = e.clientX - rect.left;
    const padding = { left: 20, right: 20 };
    const chartWidth = rect.width - padding.left - padding.right;

    const start = Math.max(0, Math.floor(viewStart));
    const end = Math.min(data.length, Math.ceil(viewEnd));
    const visible = data.slice(start, end);

    const idx = Math.floor(((x - padding.left) / chartWidth) * visible.length);
    if (idx >= 0 && idx < visible.length) {
      setSelectedDay(visible[idx]);
    }
  };

  const handleWheel = (e: React.WheelEvent) => {
    e.preventDefault();
    const delta = e.deltaY > 0 ? 1.1 : 0.9;
    const center = (viewStart + viewEnd) / 2;
    const range = (viewEnd - viewStart) * delta;
    const newStart = Math.max(0, center - range / 2);
    const newEnd = Math.min(data.length, center + range / 2);
    setViewStart(newStart);
    setViewEnd(newEnd);
  };

  if (loading) return <div className="empty">Loading timeline...</div>;

  if (data.length === 0) {
    return (
      <div>
        <h2 style={{ fontSize: 22, fontWeight: 700, color: "var(--text-0)", marginBottom: 16 }}>Timeline</h2>
        <div className="card empty">No timeline data available</div>
      </div>
    );
  }

  return (
    <div>
      <div className="row between mb-4">
        <div>
          <h2 style={{ fontSize: 22, fontWeight: 700, color: "var(--text-0)" }}>Timeline</h2>
          <p className="muted">{data.length} days · Scroll to zoom, click bars for details</p>
        </div>
        <div className="row gap-2">
          <button className="btn btn-ghost btn-sm" onClick={() => { setViewStart(0); setViewEnd(data.length); }}>
            Reset Zoom
          </button>
          <button className="btn btn-ghost btn-sm" onClick={loadData}>↻ Refresh</button>
        </div>
      </div>

      {/* Timeline Canvas */}
      <div className="card mb-4" ref={containerRef} style={{ padding: 0, overflow: "hidden" }}>
        <canvas
          ref={canvasRef}
          style={{ width: "100%", height: 250, cursor: "crosshair" }}
          onClick={handleCanvasClick}
          onWheel={handleWheel}
        />
      </div>

       {/* Selected Day Details */}
       {selectedDay && (
         <div className="card" style={{ borderLeft: "4px solid var(--accent)" }}>
           <div className="row between mb-4">
             <h3 style={{ fontSize: 15, fontWeight: 600 }}>
               {selectedDay.date}
             </h3>
             <div className="row gap-2">
               {(["all", "sent", "received"] as const).map(t => (
                 <button
                   key={t}
                   className={`btn btn-sm ${filterType === t ? "btn-primary" : "btn-ghost"}`}
                   onClick={() => setFilterType(t)}
                 >
                   {t === "all" ? `All (${selectedDay.total})` : t === "sent" ? `Sent (${selectedDay.sent})` : `Received (${selectedDay.received})`}
                 </button>
               ))}
             </div>
           </div>
           <div className="row gap-4 mb-4">
             <div style={{ flex: 1, padding: 12, background: "var(--bg-3)", borderRadius: "var(--r-sm)", textAlign: "center" }}>
               <div style={{ fontSize: 24, fontWeight: 700, color: "var(--text-0)" }}>{selectedDay.total}</div>
               <div style={{ fontSize: 10, color: "var(--text-3)" }}>Total</div>
             </div>
             <div style={{ flex: 1, padding: 12, background: "var(--bg-3)", borderRadius: "var(--r-sm)", textAlign: "center" }}>
               <div style={{ fontSize: 24, fontWeight: 700, color: "#22c55e" }}>{selectedDay.received}</div>
               <div style={{ fontSize: 10, color: "var(--text-3)" }}>Received</div>
             </div>
             <div style={{ flex: 1, padding: 12, background: "var(--bg-3)", borderRadius: "var(--r-sm)", textAlign: "center" }}>
               <div style={{ fontSize: 24, fontWeight: 700, color: "#3b82f6" }}>{selectedDay.sent}</div>
               <div style={{ fontSize: 10, color: "var(--text-3)" }}>Sent</div>
             </div>
           </div>
         </div>
       )}

       {/* Day Emails List */}
       {selectedDay && (
         <div className="card">
           <h4 style={{ fontSize: 13, fontWeight: 600, marginBottom: 12 }}>
             Emails on {selectedDay.date}
           </h4>
           <TimelineDayEmails caseId={caseId} date={selectedDay.date} filter={filterType} onSelect={(id) => onSelectEmail?.(id)} />
         </div>
       )}

      {/* Summary Stats */}
      <div className="card">
        <h3 style={{ fontSize: 14, fontWeight: 600, marginBottom: 12 }}>Summary</h3>
        <div className="grid-2">
          <div>
            <span className="muted">Date Range:</span>
            <p style={{ fontSize: 13, color: "var(--text-1)", marginTop: 4 }}>
              {dateRange.min ? dateRange.min.slice(0, 10) : "—"} to {dateRange.max ? dateRange.max.slice(0, 10) : "—"}
            </p>
          </div>
          <div>
            <span className="muted">Total Emails:</span>
            <p style={{ fontSize: 13, color: "var(--text-1)", marginTop: 4 }}>
              {data.reduce((sum, d) => sum + d.total, 0).toLocaleString()}
            </p>
          </div>
          <div>
            <span className="muted">Peak Day:</span>
            <p style={{ fontSize: 13, color: "var(--text-1)", marginTop: 4 }}>
              {data.length > 0 ? data.reduce((max, d) => d.total > max.total ? d : max).date : "—"} ({Math.max(...data.map(d => d.total))} emails)
            </p>
          </div>
          <div>
            <span className="muted">Active Days:</span>
            <p style={{ fontSize: 13, color: "var(--text-1)", marginTop: 4 }}>
              {data.filter(d => d.total > 0).length} of {data.length} days
            </p>
          </div>
        </div>
      </div>
    </div>
  );
}

function TimelineDayEmails({ caseId, date, filter, onSelect }: { caseId: string; date: string; filter: string; onSelect?: (id: string) => void }) {
  const [emails, setEmails] = useState<any[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    setLoading(true);
    invoke<any>("emails_by_date", { input: { case_id: caseId, date } })
      .then(data => {
        let filtered = data;
        if (filter === "sent") filtered = data.filter((e: any) => e.folder_category === "sent");
        if (filter === "received") filtered = data.filter((e: any) => e.folder_category !== "sent");
        setEmails(filtered);
      })
      .catch(() => setEmails([]))
      .finally(() => setLoading(false));
  }, [caseId, date, filter]);

  if (loading) return <div className="empty">Loading emails...</div>;
  if (emails.length === 0) return <div className="empty">No emails match filter</div>;

  return (
    <div style={{ maxHeight: 300, overflowY: "auto" }}>
      <table style={{ marginTop: 0 }}>
        <thead>
          <tr>
            <th className="th" style={{ width: 150 }}>From</th>
            <th className="th">Subject</th>
            <th className="th" style={{ width: 80 }}>Time</th>
            <th className="th" style={{ width: 50 }}>Risk</th>
          </tr>
        </thead>
        <tbody>
          {emails.map((e) => (
            <tr key={e.id} className="tr-click" onClick={() => onSelect?.(e.id)}>
              <td className="td" style={{ fontSize: 12 }}>
                {e.from_display || e.from_addr}
              </td>
              <td className="td" style={{ fontSize: 12 }}>
                {e.subject || <span className="muted">(no subject)</span>}
              </td>
              <td className="td muted" style={{ fontSize: 11 }}>
                {e.date_sent_utc ? new Date(e.date_sent_utc).toLocaleTimeString() : "—"}
              </td>
              <td className="td">
                <span className={`badge ${e.risk_score >= 50 ? "badge-red" : e.risk_score >= 25 ? "badge-orange" : "badge-green"}`}>
                  {e.risk_score}
                </span>
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}
