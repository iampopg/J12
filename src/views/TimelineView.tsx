import { useState, useEffect, useMemo } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  DailyRecord,
  MonthlyRecord,
  TimelineEmail,
  FilterCategory,
  TimelineProps,
} from "./timeline/types";
import { TimelineAnalyticsCards } from "./timeline/TimelineAnalyticsCards";
import { TimelineHistogram } from "./timeline/TimelineHistogram";
import { TimelineStreamTable } from "./timeline/TimelineStreamTable";
import { TimelinePreviewPanel } from "./timeline/TimelinePreviewPanel";

export function TimelineView({ caseId, evidenceFilter }: TimelineProps) {
  const [dailyData, setDailyData] = useState<DailyRecord[]>([]);
  const [loading, setLoading] = useState(true);
  const [dateRange, setDateRange] = useState<{ min: string; max: string }>({ min: "", max: "" });

  const [granularity, setGranularity] = useState<"month" | "day">("month");
  const [selectedPeriod, setSelectedPeriod] = useState<string | null>(null);
  const [pageOffset, setPageOffset] = useState(0);
  const itemsPerPage = 14;

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
      const res = await invoke<any>("timeline_data", { input: { case_id: caseId, evidence_id: evidenceFilter || undefined } });
      const daily: DailyRecord[] = Array.isArray(res) ? res : (res?.daily || []);
      setDailyData(daily);
      if (res?.date_range?.min && res?.date_range?.max) {
        setDateRange({ min: res.date_range.min, max: res.date_range.max });
      } else if (daily.length > 0) {
        setDateRange({ min: daily[0].date, max: daily[daily.length - 1].date });
      }
      loadEmailStream("");
    } catch (e) {
      console.error("Failed to load timeline data:", e);
    } finally {
      setLoading(false);
    }
  };

  const monthlyData: MonthlyRecord[] = useMemo(() => {
    const map = new Map<string, { total: number; sent: number; received: number }>();
    dailyData.forEach((d) => {
      const m = d.date.slice(0, 7);
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

  const loadEmailStream = async (period: string) => {
    setLoadingEmails(true);
    setSelectedEmail(null);
    try {
      let res: TimelineEmail[] = [];
      if (period.length === 10) {
        res = await invoke<TimelineEmail[]>("emails_by_date", {
          input: { case_id: caseId, evidence_id: evidenceFilter || undefined, date: period },
        });
      } else if (period.length === 7) {
        res = await invoke<TimelineEmail[]>("advanced_search", {
          input: { case_id: caseId, evidence_id: evidenceFilter || undefined, query: `after:${period}-01 before:${period}-31`, limit: 500 },
        });
      } else {
        res = await invoke<TimelineEmail[]>("advanced_search", {
          input: { case_id: caseId, evidence_id: evidenceFilter || undefined, query: "", limit: 500 },
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

  const stats = useMemo(() => {
    const total = dailyData.reduce((acc, d) => acc + d.total, 0);
    const peak = dailyData.reduce((max, d) => (d.total > max.total ? d : max), dailyData[0] || { date: "—", total: 0 });
    return { total, peak };
  }, [dailyData]);

  const currentChartItems = useMemo(() => {
    const list = granularity === "month" ? monthlyData : dailyData;
    const start = pageOffset;
    return list.slice(start, start + itemsPerPage);
  }, [granularity, monthlyData, dailyData, pageOffset]);

  const maxChartValue = useMemo(() => {
    const list = granularity === "month" ? monthlyData : dailyData;
    return Math.max(...list.map((d) => d.total), 1);
  }, [granularity, monthlyData, dailyData]);

  const filteredEmails = useMemo(() => {
    let result = streamEmails.filter((em) => {
      if (filterType === "sent" && em.folder_category !== "sent") return false;
      if (filterType === "received" && em.folder_category === "sent") return false;
      if (filterType === "deleted" && !em.is_deleted && !em.deleted_recovered) return false;
      if (filterType === "flagged" && em.risk_score < 25) return false;
      if (filterType === "after_hours") {
        if (em.date_sent_utc) {
          const hour = new Date(em.date_sent_utc).getUTCHours();
          if (hour >= 6 && hour < 21) return false;
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
      <TimelineAnalyticsCards
        dateRange={dateRange}
        stats={stats}
        selectedPeriod={selectedPeriod}
      />

      {/* Interactive Activity Histogram Chart */}
      <TimelineHistogram
        granularity={granularity}
        setGranularity={setGranularity}
        pageOffset={pageOffset}
        setPageOffset={setPageOffset}
        itemsPerPage={itemsPerPage}
        monthlyData={monthlyData}
        dailyData={dailyData}
        currentChartItems={currentChartItems}
        maxChartValue={maxChartValue}
        selectedPeriod={selectedPeriod}
        onSelectPeriod={handleSelectPeriod}
      />

      {/* Chronological Event Stream & Split-Pane Inspector */}
      <div className="card mb-0" style={{ padding: 18 }}>
        <div
          style={{
            display: "grid",
            gridTemplateColumns: selectedEmail ? "1fr 420px" : "1fr",
            gap: 16,
            alignItems: "start",
          }}
        >
          <TimelineStreamTable
            caseId={caseId}
            loadingEmails={loadingEmails}
            filteredEmails={filteredEmails}
            selectedEmail={selectedEmail}
            filterType={filterType}
            setFilterType={setFilterType}
            searchQuery={searchQuery}
            setSearchQuery={setSearchQuery}
            sortOrder={sortOrder}
            setSortOrder={setSortOrder}
            onSelectEmail={setSelectedEmail}
          />

          {selectedEmail && (
            <TimelinePreviewPanel
              caseId={caseId}
              selectedEmail={selectedEmail}
              onClose={() => setSelectedEmail(null)}
            />
          )}
        </div>
      </div>
    </div>
  );
}
