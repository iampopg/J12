import { useState, useEffect, useMemo } from "react";
import { invoke } from "@tauri-apps/api/core";

function cleanDisplayName(name: string | null): string {
  if (!name) return "";
  let cleaned = name
    .replace(/@ENRON.*$/g, "")
    .replace(/IMCEANOTES-[^<]*/g, "")
    .replace(/<[^>]*>/g, "")
    .replace(/"/g, "")
    .replace(/\s+/g, " ")
    .trim();
  if (cleaned.includes("@")) {
    return cleaned.split("@")[0].trim() || cleaned;
  }
  return cleaned;
}

interface DailyRecord {
  date: string;
  total: number;
  sent: number;
  received: number;
}

interface MonthlyRecord {
  month: string;
  total: number;
  sent: number;
  received: number;
}

interface TimelineEmail {
  id: string;
  evidence_id: string;
  message_id: string | null;
  from_addr: string;
  from_display: string | null;
  to_addrs: string;
  cc_addrs: string;
  subject: string | null;
  date_sent: string | null;
  date_sent_utc: string;
  folder_name: string | null;
  folder_category: string;
  is_deleted: boolean;
  deleted_recovered: boolean;
  risk_score: number;
  flags: string | null;
  body_text: string | null;
  headers_raw: string | null;
}

type FilterCategory = "all" | "sent" | "received" | "deleted" | "flagged" | "after_hours";

interface Props {
  caseId: string;
  evidenceFilter?: string | null;
  onSelectEmail?: (id: string) => void;
}

export function TimelineView({ caseId, evidenceFilter }: Props) {
  const [dailyData, setDailyData] = useState<DailyRecord[]>([]);
  const [loading, setLoading] = useState(true);
  const [dateRange, setDateRange] = useState<{ min: string; max: string }>({ min: "", max: "" });

  // View modes & selections
  const [granularity, setGranularity] = useState<"month" | "day">("month");
  const [selectedPeriod, setSelectedPeriod] = useState<string | null>(null);
  const [pageOffset, setPageOffset] = useState(0);
  const itemsPerPage = 14;

  // Stream & filters
  const [streamEmails, setStreamEmails] = useState<TimelineEmail[]>([]);
  const [loadingEmails, setLoadingEmails] = useState(false);
  const [filterType, setFilterType] = useState<FilterCategory>("all");
  const [searchQuery, setSearchQuery] = useState("");
  const [sortOrder, setSortOrder] = useState<"desc" | "asc">("desc");
  const [selectedEmail, setSelectedEmail] = useState<TimelineEmail | null>(null);

  useEffect(() => {
    loadData();
  }, [caseId, evidenceFilter]);

  const loadData = async () => {
    setLoading(true);
    try {
      const res = await invoke<any>("timeline_data", { 
        input: { 
          case_id: caseId,
          evidence_id: evidenceFilter || undefined
        } 
      });
      const daily: DailyRecord[] = res.daily || [];
      setDailyData(daily);
      if (res.date_range) {
        setDateRange({ min: res.date_range.min || "", max: res.date_range.max || "" });
      }

      // Load all case emails into the stream initially
      loadEmailStream("");
    } catch (e) {
      console.error("Failed to load timeline data:", e);
    } finally {
      setLoading(false);
    }
  };

  // Group daily data into monthly buckets
  const monthlyData: MonthlyRecord[] = useMemo(() => {
    const map = new Map<string, { total: number; sent: number; received: number }>();
    dailyData.forEach((d) => {
      const m = d.date.slice(0, 7); // "YYYY-MM"
      const existing = map.get(m) || { total: 0, sent: 0, received: 0 };
      existing.total += d.total;
      existing.sent += d.sent;
      existing.received += d.received;
      map.set(m, existing);
    });

    const result: MonthlyRecord[] = [];
    map.forEach((val, key) => {
      result.push({ month: key, total: val.total, sent: val.sent, received: val.received });
    });
    result.sort((a, b) => a.month.localeCompare(b.month));
    return result;
  }, [dailyData]);

  // Load emails for selected date or entire case
  const loadEmailStream = async (period: string) => {
    setLoadingEmails(true);
    setSelectedEmail(null);
    try {
      let res: TimelineEmail[] = [];
      if (period.length === 10) {
        // Daily date
        res = await invoke<TimelineEmail[]>("emails_by_date", {
          input: { 
            case_id: caseId, 
            date: period,
            evidence_id: evidenceFilter || undefined
          },
        });
      } else if (period.length === 7) {
        // Monthly prefix (search with after & before)
        res = await invoke<TimelineEmail[]>("advanced_search", {
          input: { 
            case_id: caseId, 
            query: `after:${period}-01 before:${period}-31`, 
            limit: 500,
            evidence_id: evidenceFilter || undefined
          },
        });
      } else {
        // All emails
        res = await invoke<TimelineEmail[]>("advanced_search", {
          input: { 
            case_id: caseId, 
            query: "", 
            limit: 500,
            evidence_id: evidenceFilter || undefined
          },
        });
      }
      setStreamEmails(res || []);
    } catch (e) {
      console.error(e);
      setStreamEmails([]);
    } finally {
      setLoadingEmails(false);
    }
  };

  const handleSelectPeriod = (period: string) => {
    if (selectedPeriod === period) {
      setSelectedPeriod(null);
      loadEmailStream("");
    } else {
      setSelectedPeriod(period);
      loadEmailStream(period);
    }
  };

  // Metrics
  const stats = useMemo(() => {
    const total = dailyData.reduce((acc, d) => acc + d.total, 0);
    const peak = dailyData.reduce((max, d) => (d.total > max.total ? d : max), dailyData[0] || { date: "—", total: 0 });
    return { total, peak };
  }, [dailyData]);

  // Active chart items with pagination
  const currentChartItems = useMemo(() => {
    const list = granularity === "month" ? monthlyData : dailyData;
    const start = pageOffset;
    return list.slice(start, start + itemsPerPage);
  }, [granularity, monthlyData, dailyData, pageOffset]);

  const maxChartValue = useMemo(() => {
    const list = granularity === "month" ? monthlyData : dailyData;
    return Math.max(...list.map((d) => d.total), 1);
  }, [granularity, monthlyData, dailyData]);

  // Filter and sort email stream
  const filteredEmails = useMemo(() => {
    let result = streamEmails.filter((em) => {
      if (filterType === "sent" && em.folder_category !== "sent") return false;
      if (filterType === "received" && em.folder_category === "sent") return false;
      if (filterType === "deleted" && !em.is_deleted && !em.deleted_recovered) return false;
      if (filterType === "flagged" && em.risk_score < 25) return false;
      if (filterType === "after_hours") {
        if (em.date_sent_utc) {
          const hour = new Date(em.date_sent_utc).getUTCHours();
          if (hour >= 6 && hour < 21) return false; // 9 PM - 6 AM is after hours
        }
      }

      if (searchQuery.trim()) {
        const q = searchQuery.toLowerCase();
        const mSub = (em.subject || "").toLowerCase().includes(q);
        const mFrom = em.from_addr.toLowerCase().includes(q);
        const mDisp = (em.from_display || "").toLowerCase().includes(q);
        return mSub || mFrom || mDisp;
      }
      return true;
    });

    result.sort((a, b) => {
      const dA = a.date_sent_utc || "";
      const dB = b.date_sent_utc || "";
      return sortOrder === "desc" ? dB.localeCompare(dA) : dA.localeCompare(dB);
    });

    return result;
  }, [streamEmails, filterType, searchQuery, sortOrder]);

  if (loading) return <div className="card empty">Loading forensic timeline data...</div>;

  return (
    <div>
      {/* Top Header */}
      <div className="row between mb-4">
        <div>
          <h2 style={{ fontSize: 22, fontWeight: 700, color: "var(--text-0)" }}>
            Forensic Activity Chronology
          </h2>
          <p className="muted" style={{ fontSize: 12 }}>
            Industry-standard linear timeline audit log and volume histogram. Select any month or day to drill down.
          </p>
        </div>
        <div className="row gap-2">
          {selectedPeriod && (
            <button
              className="btn btn-primary btn-sm"
              onClick={() => {
                setSelectedPeriod(null);
                loadEmailStream("");
              }}
            >
              Showing: {selectedPeriod} ✕ (Reset to All)
            </button>
          )}
          <button className="btn btn-ghost btn-sm" onClick={loadData}>
            ↻ Refresh
          </button>
        </div>
      </div>

      {/* Summary Analytics Cards */}
      <div
        style={{
          display: "grid",
          gridTemplateColumns: "repeat(4, 1fr)",
          gap: 12,
          marginBottom: 16,
        }}
      >
        <div className="card mb-0" style={{ padding: 14 }}>
          <div style={{ fontSize: 11, color: "var(--text-3)", fontWeight: 600 }}>TIMELINE SPAN</div>
          <div style={{ fontSize: 13, fontWeight: 700, color: "var(--text-0)", marginTop: 4 }}>
            {dateRange.min ? dateRange.min.slice(0, 10) : "—"} → {dateRange.max ? dateRange.max.slice(0, 10) : "—"}
          </div>
        </div>

        <div className="card mb-0" style={{ padding: 14 }}>
          <div style={{ fontSize: 11, color: "var(--text-3)", fontWeight: 600 }}>TOTAL EMAILS LOGGED</div>
          <div style={{ fontSize: 18, fontWeight: 700, color: "var(--accent)", marginTop: 2 }}>
            {stats.total.toLocaleString()}
          </div>
        </div>

        <div className="card mb-0" style={{ padding: 14 }}>
          <div style={{ fontSize: 11, color: "var(--text-3)", fontWeight: 600 }}>PEAK ACTIVITY SPIKE</div>
          <div style={{ fontSize: 14, fontWeight: 700, color: "#f59e0b", marginTop: 2 }}>
            {stats.peak?.date || "—"} ({stats.peak?.total} emails)
          </div>
        </div>

        <div className="card mb-0" style={{ padding: 14 }}>
          <div style={{ fontSize: 11, color: "var(--text-3)", fontWeight: 600 }}>CURRENT SELECTION</div>
          <div style={{ fontSize: 14, fontWeight: 700, color: "#22c55e", marginTop: 2 }}>
            {selectedPeriod ? selectedPeriod : "All Dates (Full Archive)"}
          </div>
        </div>
      </div>

      {/* Interactive Activity Histogram Chart */}
      <div className="card mb-4" style={{ padding: 16 }}>
        {/* Histogram Controls */}
        <div className="row between mb-3" style={{ flexWrap: "wrap", gap: 8 }}>
          <div className="row gap-2">
            <span style={{ fontSize: 12, fontWeight: 600, color: "var(--text-2)" }}>Granularity:</span>
            <button
              className={`btn btn-sm ${granularity === "month" ? "btn-primary" : "btn-ghost"}`}
              style={{ fontSize: 11, padding: "3px 10px" }}
              onClick={() => {
                setGranularity("month");
                setPageOffset(0);
              }}
            >
              🗓️ By Month ({monthlyData.length})
            </button>
            <button
              className={`btn btn-sm ${granularity === "day" ? "btn-primary" : "btn-ghost"}`}
              style={{ fontSize: 11, padding: "3px 10px" }}
              onClick={() => {
                setGranularity("day");
                setPageOffset(0);
              }}
            >
              📆 By Day ({dailyData.length})
            </button>
          </div>

          <div className="row gap-2">
            <span style={{ fontSize: 11, color: "var(--text-3)" }}>Legend:</span>
            <span style={{ fontSize: 11, color: "#3b82f6" }}>■ Sent</span>
            <span style={{ fontSize: 11, color: "#22c55e" }}>■ Received</span>
            <span style={{ fontSize: 11, color: "#fbbf24" }}>● Selected</span>

            {/* Pagination / Window Navigation */}
            <button
              className="btn btn-ghost btn-sm"
              style={{ fontSize: 11, padding: "2px 8px" }}
              disabled={pageOffset <= 0}
              onClick={() => setPageOffset((prev) => Math.max(0, prev - itemsPerPage))}
            >
              ◀ Earlier
            </button>
            <button
              className="btn btn-ghost btn-sm"
              style={{ fontSize: 11, padding: "2px 8px" }}
              disabled={
                pageOffset + itemsPerPage >=
                (granularity === "month" ? monthlyData.length : dailyData.length)
              }
              onClick={() => setPageOffset((prev) => prev + itemsPerPage)}
            >
              Later ▶
            </button>
          </div>
        </div>

        {/* Visual Bar Chart */}
        <div
          style={{
            display: "grid",
            gridTemplateColumns: `repeat(${currentChartItems.length}, 1fr)`,
            gap: 8,
            alignItems: "end",
            height: 140,
            padding: "10px 0 6px",
            borderBottom: "1px solid var(--border)",
          }}
        >
          {currentChartItems.map((item: any) => {
            const key = item.month || item.date;
            const isSelected = selectedPeriod === key;
            const heightPercent = Math.max(8, (item.total / maxChartValue) * 100);
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
                }}
                onClick={() => handleSelectPeriod(key)}
                title={`${key}: ${item.total} emails (${item.sent} sent · ${item.received} received)`}
              >
                {/* Number on top of bar */}
                <div
                  style={{
                    fontSize: 9,
                    color: isSelected ? "#fbbf24" : "var(--text-3)",
                    fontWeight: isSelected ? 700 : 500,
                    marginBottom: 2,
                  }}
                >
                  {item.total}
                </div>

                {/* Stacked Bar */}
                <div
                  style={{
                    width: "100%",
                    maxWidth: 32,
                    height: `${heightPercent}%`,
                    borderRadius: "4px 4px 0 0",
                    overflow: "hidden",
                    display: "flex",
                    flexDirection: "column-reverse",
                    border: isSelected ? "2px solid #fbbf24" : "1px solid transparent",
                    boxShadow: isSelected ? "0 0 10px rgba(251, 191, 36, 0.5)" : "none",
                  }}
                >
                  {/* Received segment */}
                  <div
                    style={{
                      height: `${recRatio * 100}%`,
                      background: isSelected ? "#22c55e" : "rgba(34, 197, 94, 0.75)",
                    }}
                  />
                  {/* Sent segment */}
                  <div
                    style={{
                      height: `${sentRatio * 100}%`,
                      background: isSelected ? "#3b82f6" : "rgba(59, 130, 246, 0.75)",
                    }}
                  />
                </div>

                {/* Date / Month Label below bar */}
                <div
                  style={{
                    fontSize: 10,
                    color: isSelected ? "#fbbf24" : "var(--text-2)",
                    fontWeight: isSelected ? 700 : 400,
                    marginTop: 6,
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
      </div>

      {/* Chronological Event Stream & Split-Pane Inspector */}
      <div className="card mb-0" style={{ padding: 18 }}>
        {/* Stream Filter Toolbar */}
        <div className="row between mb-3" style={{ flexWrap: "wrap", gap: 10 }}>
          <div className="row gap-1">
            {(
              [
                ["all", `All (${filteredEmails.length})`],
                ["sent", "Sent Only"],
                ["received", "Received Only"],
                ["deleted", "🗑️ Deleted"],
                ["flagged", "🚨 High Risk"],
                ["after_hours", "🌙 After-Hours"],
              ] as const
            ).map(([key, label]) => (
              <button
                key={key}
                className={`btn btn-sm ${filterType === key ? "btn-primary" : "btn-ghost"}`}
                style={{ fontSize: 11, padding: "3px 8px" }}
                onClick={() => setFilterType(key as FilterCategory)}
              >
                {label}
              </button>
            ))}
          </div>

          <div className="row gap-2">
            <button
              className="btn btn-ghost btn-sm"
              style={{ fontSize: 11, padding: "3px 8px" }}
              onClick={() => setSortOrder((s) => (s === "desc" ? "asc" : "desc"))}
            >
              {sortOrder === "desc" ? "⬇ Newest First" : "⬆ Oldest First"}
            </button>
          </div>
        </div>

        {/* Search inside stream */}
        <div className="mb-3">
          <input
            className="input"
            style={{ fontSize: 12, padding: "7px 12px", width: "100%" }}
            placeholder="Filter emails in timeline by subject, sender, or recipient..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
          />
        </div>

        {/* Stream Table & Side Inspector Grid */}
        <div
          style={{
            display: "grid",
            gridTemplateColumns: selectedEmail ? "1fr 420px" : "1fr",
            gap: 16,
            alignItems: "start",
          }}
        >
          {/* Linear Stream List */}
          {loadingEmails ? (
            <div className="empty">Loading chronological stream...</div>
          ) : filteredEmails.length === 0 ? (
            <div className="empty">No emails match the selected timeline filters.</div>
          ) : (
            <div
              style={{
                maxHeight: "56vh",
                overflowY: "auto",
                border: "1px solid var(--border)",
                borderRadius: "var(--r-md)",
              }}
            >
              {/* Header */}
              <div
                style={{
                  display: "grid",
                  gridTemplateColumns: "110px 160px 1fr 60px",
                  padding: "8px 12px",
                  background: "var(--bg-1)",
                  borderBottom: "1px solid var(--border)",
                  fontSize: 10,
                  fontWeight: 700,
                  textTransform: "uppercase",
                  color: "var(--text-3)",
                }}
              >
                <div>Date & Time</div>
                <div>Sender</div>
                <div>Subject & Indicators</div>
                <div style={{ textAlign: "center" }}>Risk</div>
              </div>

              {/* Rows */}
              {filteredEmails.map((em) => {
                const isSelected = selectedEmail?.id === em.id;
                const isSent = em.folder_category === "sent";

                return (
                  <div
                    key={em.id}
                    className="tr-click"
                    style={{
                      display: "grid",
                      gridTemplateColumns: "110px 160px 1fr 60px",
                      alignItems: "center",
                      padding: "8px 12px",
                      borderBottom: "1px solid var(--border)",
                      background: isSelected ? "var(--accent-subtle)" : "transparent",
                      fontSize: 12,
                    }}
                    onClick={() => setSelectedEmail(isSelected ? null : em)}
                  >
                    {/* Timestamp */}
                    <div style={{ fontSize: 11, color: "var(--text-3)" }}>
                      <div>{em.date_sent_utc ? em.date_sent_utc.slice(0, 10) : "—"}</div>
                      <div style={{ fontSize: 10 }}>
                        {em.date_sent_utc ? new Date(em.date_sent_utc).toLocaleTimeString() : ""}
                      </div>
                    </div>

                    {/* Sender */}
                    <div
                      style={{
                        overflow: "hidden",
                        textOverflow: "ellipsis",
                        whiteSpace: "nowrap",
                        color: "var(--text-1)",
                      }}
                      title={em.from_addr}
                    >
                      <span className={`badge ${isSent ? "badge-blue" : "badge-green"}`} style={{ fontSize: 8, marginRight: 4 }}>
                        {isSent ? "SENT" : "IN"}
                      </span>
                      {cleanDisplayName(em.from_display) || em.from_addr}
                    </div>

                    {/* Subject */}
                    <div
                      style={{
                        overflow: "hidden",
                        textOverflow: "ellipsis",
                        whiteSpace: "nowrap",
                        color: "var(--text-0)",
                        fontWeight: 500,
                      }}
                    >
                      {em.subject || <span className="muted">(no subject)</span>}
                      {em.deleted_recovered && (
                        <span className="badge badge-red" style={{ fontSize: 9, marginLeft: 6 }}>
                          DELETED
                        </span>
                      )}
                    </div>

                    {/* Risk Badge */}
                    <div style={{ textAlign: "center" }}>
                      <span
                        className={`badge ${
                          em.risk_score >= 50
                            ? "badge-red"
                            : em.risk_score >= 25
                            ? "badge-orange"
                            : "badge-green"
                        }`}
                        style={{ fontSize: 9 }}
                      >
                        {em.risk_score}
                      </span>
                    </div>
                  </div>
                );
              })}
            </div>
          )}

          {/* Inline Email Preview Panel */}
          {selectedEmail && (
            <div
              className="card mb-0"
              style={{
                padding: 16,
                maxHeight: "56vh",
                overflowY: "auto",
                background: "var(--bg-1)",
                border: "1px solid var(--border)",
                borderLeft: "4px solid var(--accent)",
              }}
            >
              <div className="row between mb-2">
                <strong style={{ fontSize: 14, color: "var(--text-0)" }}>
                  {selectedEmail.subject || "(no subject)"}
                </strong>
                <button
                  className="btn btn-ghost btn-sm"
                  style={{ padding: "1px 5px", fontSize: 10 }}
                  onClick={() => setSelectedEmail(null)}
                >
                  ✕ Close
                </button>
              </div>

              <div
                style={{
                  background: "var(--bg-3)",
                  padding: 10,
                  borderRadius: "var(--r-sm)",
                  fontSize: 11,
                  display: "flex",
                  flexDirection: "column",
                  gap: 4,
                  marginBottom: 10,
                }}
              >
                <div>
                  <span className="muted">From: </span>
                  <strong>
                    {selectedEmail.from_display
                      ? `${selectedEmail.from_display} <${selectedEmail.from_addr}>`
                      : selectedEmail.from_addr}
                  </strong>
                </div>
                <div>
                  <span className="muted">To: </span>
                  <span style={{ fontFamily: "var(--mono)" }}>{selectedEmail.to_addrs}</span>
                </div>
                <div>
                  <span className="muted">Timestamp: </span>
                  {selectedEmail.date_sent_utc
                    ? new Date(selectedEmail.date_sent_utc).toUTCString()
                    : "—"}
                </div>
                <div>
                  <span className="muted">Risk Score: </span>
                  <span
                    className={`badge ${
                      selectedEmail.risk_score >= 50
                        ? "badge-red"
                        : selectedEmail.risk_score >= 25
                        ? "badge-orange"
                        : "badge-green"
                    }`}
                  >
                    {selectedEmail.risk_score}
                  </span>
                </div>
              </div>

              {/* Message Body */}
              <pre
                style={{
                  background: "var(--bg-0)",
                  border: "1px solid var(--border)",
                  borderRadius: "var(--r-xs)",
                  padding: 10,
                  fontSize: 11,
                  maxHeight: 180,
                  overflow: "auto",
                  whiteSpace: "pre-wrap",
                  color: "var(--text-1)",
                }}
              >
                {selectedEmail.body_text || "(No text content)"}
              </pre>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
