import { useState, useMemo, useEffect, useRef, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";

// Helper to clean display names
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
    const parts = cleaned.split("@");
    return parts[0].trim() || cleaned;
  }
  return cleaned;
}

interface Email {
  id: string;
  evidence_id: string;
  message_id: string | null;
  from_addr: string;
  from_display: string | null;
  to_addrs: string;
  cc_addrs: string;
  subject: string | null;
  date_sent: string | null;
  body_text: string | null;
  body_html: string | null;
  headers_raw: string | null;
  folder_name: string | null;
  folder_category: string;
  recovery_status: string;
  deleted_recovered: boolean;
  risk_score: number;
}

export interface EmailTag {
  id: string;
  case_id: string;
  email_id: string;
  tag: string;
  color: string;
  created_by: string;
  created_at: string;
}

interface Evidence {
  id: string;
  filename: string;
}

export type SortField = "date" | "name" | "from" | "subject" | "risk" | "folder";
export type SortDir = "asc" | "desc";

export function EmailListView({
  caseId,
  filter,
  onViewEntity,
}: {
  caseId: string;
  filter?: string;
  onViewEntity?: (email: string) => void;
}) {
  const [emails, setEmails] = useState<Email[]>([]);
  const [evidence, setEvidence] = useState<Evidence[]>([]);
  const [tags, setTags] = useState<EmailTag[]>([]);
  const [loading, setLoading] = useState(true);
  
  // Search & Filter state
  const [q, setQ] = useState("");
  const [sortField, setSortField] = useState<SortField>("date");
  const [sortDir, setSortDir] = useState<SortDir>("desc");
  const [selected, setSelected] = useState<Email | null>(null);
  const [showUnique, setShowUnique] = useState(false);
  const [tagFilter, setTagFilter] = useState<string>("all");
  
  // Date Filtering state
  const [dateFilterMode, setDateFilterMode] = useState<"all" | "single" | "range">("all");
  const [singleDate, setSingleDate] = useState<string>("");
  const [startDate, setStartDate] = useState<string>("");
  const [endDate, setEndDate] = useState<string>("");
  const [showFilterDrawer, setShowFilterDrawer] = useState(false);

  const load = async () => {
    setLoading(true);
    try {
      const [em, ev, tg] = await Promise.all([
        invoke<Email[]>("email_list", { input: { case_id: caseId, limit: 10000 } }),
        invoke<Evidence[]>("evidence_list", { input: { case_id: caseId } }),
        invoke<EmailTag[]>("email_tags_list", { caseId }).catch(() => [] as EmailTag[]),
      ]);
      setEmails(em);
      setEvidence(ev);
      setTags(tg);
    } catch (e) {
      console.error(e);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    load();
  }, []);

  const loadTags = async () => {
    try {
      const tg = await invoke<EmailTag[]>("email_tags_list", { caseId });
      setTags(tg);
    } catch (e) {
      console.error(e);
    }
  };

  const evidenceMap = useMemo(() => {
    const m = new Map<string, Evidence>();
    evidence.forEach((e) => m.set(e.id, e));
    return m;
  }, [evidence]);

  const tagsByEmail = useMemo(() => {
    const m = new Map<string, EmailTag[]>();
    tags.forEach((t) => {
      const arr = m.get(t.email_id) || [];
      arr.push(t);
      m.set(t.email_id, arr);
    });
    return m;
  }, [tags]);

  const uniqueTags = useMemo(() => {
    return Array.from(new Set(tags.map((t) => t.tag)));
  }, [tags]);

  // Robust Deduplication
  const uniqueEmails = useMemo(() => {
    if (!showUnique) return emails;
    const seen = new Set<string>();
    return emails.filter((e) => {
      // Clean normalize subject and sender
      const cleanSub = (e.subject || "").toLowerCase().replace(/^(re|fwd|fw):\s*/gi, "").trim();
      const cleanFrom = e.from_addr.toLowerCase().trim();
      const cleanDate = e.date_sent ? e.date_sent.slice(0, 10) : "";
      const msgId = (e.message_id || "").trim();
      
      const key = msgId ? `msgid:${msgId}` : `composite:${cleanFrom}|${cleanSub}|${cleanDate}`;
      if (seen.has(key)) return false;
      seen.add(key);
      return true;
    });
  }, [emails, showUnique]);

  // Folder Category Filter
  const filteredByFolder = useMemo(() => {
    if (!filter || filter === "all") return uniqueEmails;
    if (filter === "sent") {
      return uniqueEmails.filter((e) => e.folder_category === "sent");
    }
    if (filter === "inbox") {
      return uniqueEmails.filter((e) => e.folder_category === "inbox");
    }
    if (filter === "soft_deleted") {
      return uniqueEmails.filter(
        (e) => e.folder_category === "soft_deleted" || e.recovery_status === "soft_deleted"
      );
    }
    if (filter === "hard_deleted") {
      return uniqueEmails.filter(
        (e) => e.recovery_status === "hard_deleted" || e.recovery_status === "purged"
      );
    }
    if (filter === "recoverable") {
      return uniqueEmails.filter((e) => e.recovery_status === "recoverable");
    }
    if (filter === "drafts") {
      return uniqueEmails.filter((e) => e.folder_category === "drafts");
    }
    if (filter === "spam") {
      return uniqueEmails.filter((e) => e.folder_category === "spam");
    }
    if (filter === "other") {
      return uniqueEmails.filter((e) => e.folder_category === "other");
    }
    return uniqueEmails;
  }, [uniqueEmails, filter]);

  // Master Filter & Sort
  const filtered = useMemo(() => {
    let result = filteredByFolder;

    // Tag Filter
    if (tagFilter !== "all") {
      result = result.filter((e) => {
        const emailTags = tagsByEmail.get(e.id) || [];
        return emailTags.some((t) => t.tag.toLowerCase() === tagFilter.toLowerCase());
      });
    }

    // Date Filters
    if (dateFilterMode === "single" && singleDate) {
      result = result.filter((e) => {
        if (!e.date_sent) return false;
        return e.date_sent.startsWith(singleDate);
      });
    } else if (dateFilterMode === "range") {
      if (startDate) {
        result = result.filter((e) => {
          if (!e.date_sent) return false;
          return e.date_sent.slice(0, 10) >= startDate;
        });
      }
      if (endDate) {
        result = result.filter((e) => {
          if (!e.date_sent) return false;
          return e.date_sent.slice(0, 10) <= endDate;
        });
      }
    }

    // Text Search
    if (q) {
      const qq = q.toLowerCase();
      result = result.filter(
        (e) =>
          (e.subject || "").toLowerCase().includes(qq) ||
          e.from_addr.toLowerCase().includes(qq) ||
          (cleanDisplayName(e.from_display) || "").toLowerCase().includes(qq) ||
          (e.body_text || "").toLowerCase().includes(qq)
      );
    }

    // Sorting
    result = [...result].sort((a, b) => {
      let cmp = 0;
      switch (sortField) {
        case "date":
          cmp = (a.date_sent || "").localeCompare(b.date_sent || "");
          break;
        case "name":
          cmp = (cleanDisplayName(a.from_display) || a.from_addr).localeCompare(
            cleanDisplayName(b.from_display) || b.from_addr
          );
          break;
        case "from":
          cmp = a.from_addr.localeCompare(b.from_addr);
          break;
        case "subject":
          cmp = (a.subject || "").localeCompare(b.subject || "");
          break;
        case "risk":
          cmp = a.risk_score - b.risk_score;
          break;
        case "folder":
          cmp = a.folder_category.localeCompare(b.folder_category);
          break;
      }
      return sortDir === "asc" ? cmp : -cmp;
    });

    return result;
  }, [
    filteredByFolder,
    tagFilter,
    tagsByEmail,
    dateFilterMode,
    singleDate,
    startDate,
    endDate,
    q,
    sortField,
    sortDir,
  ]);

  const toggleSort = (field: SortField) => {
    if (sortField === field) {
      setSortDir((d) => (d === "asc" ? "desc" : "asc"));
    } else {
      setSortField(field);
      setSortDir("asc");
    }
  };

  const handleResetFilters = () => {
    setQ("");
    setTagFilter("all");
    setDateFilterMode("all");
    setSingleDate("");
    setStartDate("");
    setEndDate("");
    setShowUnique(false);
    setSortField("date");
    setSortDir("desc");
  };

  const hasActiveFilters =
    q ||
    tagFilter !== "all" ||
    dateFilterMode !== "all" ||
    singleDate ||
    startDate ||
    endDate ||
    showUnique;

  if (loading) return <div className="empty">Loading emails...</div>;

  return (
    <div>
      {selected ? (
        <EmailDetail
          email={selected}
          caseId={caseId}
          evidenceName={evidenceMap.get(selected.evidence_id)?.filename}
          tags={tagsByEmail.get(selected.id) || []}
          onTagsChanged={loadTags}
          onClose={() => setSelected(null)}
        />
      ) : (
        <>
          {/* Header Bar */}
          <div className="row between mb-3">
            <div>
              <h2 style={{ fontSize: 22, fontWeight: 700, color: "var(--text-0)" }}>
                {filter === "sent"
                  ? "Sent Emails"
                  : filter === "inbox"
                  ? "Inbox"
                  : filter === "soft_deleted" || filter === "hard_deleted"
                  ? "Deleted Emails"
                  : "All Emails"}{" "}
                <span style={{ fontSize: 16, fontWeight: 500, color: "var(--text-2)" }}>
                  ({filtered.length.toLocaleString()} of {emails.length.toLocaleString()})
                </span>
              </h2>
              <p className="muted" style={{ fontSize: 12 }}>
                Click any column header to sort (A-Z, Z-A, Date). Click rows for deep forensic inspection.
              </p>
            </div>
            <div className="row gap-2">
              <label
                className="row gap-2"
                style={{
                  fontSize: 12,
                  color: "var(--text-1)",
                  cursor: "pointer",
                  background: showUnique ? "var(--bg-3)" : "transparent",
                  padding: "4px 10px",
                  borderRadius: "var(--r-md)",
                  border: "1px solid var(--border)",
                }}
              >
                <input
                  type="checkbox"
                  checked={showUnique}
                  onChange={(e) => setShowUnique(e.target.checked)}
                />
                <strong>Unique Only</strong>
              </label>
              <button
                className={`btn btn-sm ${showFilterDrawer ? "btn-primary" : "btn-ghost"}`}
                onClick={() => setShowFilterDrawer(!showFilterDrawer)}
              >
                📅 Date & Sort Filters {hasActiveFilters && "●"}
              </button>
              <button className="btn btn-ghost btn-sm" onClick={load}>
                ↻ Refresh
              </button>
            </div>
          </div>

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
                  setSortField(field);
                  setSortDir(dir);
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
                onClick={handleResetFilters}
              >
                ✕ Clear Filters
              </button>
            )}
          </div>

          {/* Collapsible Advanced Date & Tag Filter Drawer */}
          {showFilterDrawer && (
            <div
              className="card mb-3"
              style={{
                padding: "16px 20px",
                background: "var(--bg-1)",
                border: "1px solid var(--border)",
              }}
            >
              <div className="row between mb-3">
                <strong style={{ fontSize: 13, color: "var(--text-0)" }}>
                  📅 Advanced Date & Forensic Tag Filters
                </strong>
                <button
                  className="btn btn-ghost btn-sm"
                  style={{ padding: "2px 8px", fontSize: 11 }}
                  onClick={() => setShowFilterDrawer(false)}
                >
                  Hide
                </button>
              </div>

              {/* Date Filter Modes */}
              <div className="row gap-3 mb-3" style={{ flexWrap: "wrap", alignItems: "center" }}>
                <span className="muted" style={{ fontSize: 12 }}>
                  Date Filter Mode:
                </span>
                <button
                  className={`btn btn-sm ${dateFilterMode === "all" ? "btn-primary" : "btn-ghost"}`}
                  style={{ fontSize: 11 }}
                  onClick={() => {
                    setDateFilterMode("all");
                    setSingleDate("");
                    setStartDate("");
                    setEndDate("");
                  }}
                >
                  All Dates
                </button>
                <button
                  className={`btn btn-sm ${
                    dateFilterMode === "single" ? "btn-primary" : "btn-ghost"
                  }`}
                  style={{ fontSize: 11 }}
                  onClick={() => setDateFilterMode("single")}
                >
                  This Date Only
                </button>
                <button
                  className={`btn btn-sm ${
                    dateFilterMode === "range" ? "btn-primary" : "btn-ghost"
                  }`}
                  style={{ fontSize: 11 }}
                  onClick={() => setDateFilterMode("range")}
                >
                  Date Range (From - To)
                </button>
              </div>

              {/* Date Inputs */}
              {dateFilterMode === "single" && (
                <div className="row gap-2 mb-3" style={{ alignItems: "center" }}>
                  <label className="muted" style={{ fontSize: 12 }}>
                    Exact Date:
                  </label>
                  <input
                    type="date"
                    className="input"
                    style={{ width: 180, padding: "6px 12px", fontSize: 12 }}
                    value={singleDate}
                    onChange={(e) => setSingleDate(e.target.value)}
                  />
                  {singleDate && (
                    <span className="muted" style={{ fontSize: 11 }}>
                      Showing only emails sent on {new Date(singleDate).toLocaleDateString()}
                    </span>
                  )}
                </div>
              )}

              {dateFilterMode === "range" && (
                <div className="row gap-3 mb-3" style={{ flexWrap: "wrap", alignItems: "center" }}>
                  <div className="row gap-2">
                    <label className="muted" style={{ fontSize: 12 }}>
                      From:
                    </label>
                    <input
                      type="date"
                      className="input"
                      style={{ width: 160, padding: "6px 10px", fontSize: 12 }}
                      value={startDate}
                      onChange={(e) => setStartDate(e.target.value)}
                    />
                  </div>
                  <div className="row gap-2">
                    <label className="muted" style={{ fontSize: 12 }}>
                      To:
                    </label>
                    <input
                      type="date"
                      className="input"
                      style={{ width: 160, padding: "6px 10px", fontSize: 12 }}
                      value={endDate}
                      onChange={(e) => setEndDate(e.target.value)}
                    />
                  </div>
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
          )}

          {/* Virtual Email List Table */}
          <VirtualEmailList
            emails={filtered}
            tagsByEmail={tagsByEmail}
            sortField={sortField}
            sortDir={sortDir}
            onToggleSort={toggleSort}
            onSelect={setSelected}
            onViewEntity={onViewEntity}
          />
        </>
      )}
    </div>
  );
}

function VirtualEmailList({
  emails,
  tagsByEmail,
  sortField,
  sortDir,
  onToggleSort,
  onSelect,
  onViewEntity,
}: {
  emails: Email[];
  tagsByEmail: Map<string, EmailTag[]>;
  sortField: SortField;
  sortDir: SortDir;
  onToggleSort: (field: SortField) => void;
  onSelect: (e: Email) => void;
  onViewEntity?: (email: string) => void;
}) {
  const [scrollOffset, setScrollOffset] = useState(0);
  const containerRef = useRef<HTMLDivElement>(null);
  const rowHeight = 41;
  const visibleCount = 40;

  const handleScroll = useCallback(() => {
    if (containerRef.current) {
      setScrollOffset(containerRef.current.scrollTop);
    }
  }, []);

  useEffect(() => {
    const el = containerRef.current;
    if (el) {
      el.addEventListener("scroll", handleScroll);
      return () => el.removeEventListener("scroll", handleScroll);
    }
  }, [handleScroll]);

  const totalHeight = emails.length * rowHeight;
  const startIdx = Math.max(0, Math.floor(scrollOffset / rowHeight));
  const endIdx = Math.min(emails.length, startIdx + visibleCount);
  const visibleEmails = emails.slice(startIdx, endIdx);

  const SortIcon = ({ field }: { field: SortField }) => (
    <span style={{ opacity: sortField === field ? 1 : 0.35, marginLeft: 4, fontSize: 11 }}>
      {sortField === field ? (sortDir === "asc" ? "▲" : "▼") : "⇅"}
    </span>
  );

  if (emails.length === 0) {
    return (
      <div className="card" style={{ padding: 40, textAlign: "center" }}>
        <div style={{ fontSize: 32, marginBottom: 8 }}>🔍</div>
        <div style={{ fontSize: 15, fontWeight: 600, color: "var(--text-0)" }}>
          No emails match your criteria
        </div>
        <div className="muted text-sm mt-1">Try clearing your date or search filters.</div>
      </div>
    );
  }

  return (
    <div className="card" style={{ padding: 0, overflow: "hidden" }}>
      {/* Interactive Sortable Header */}
      <div
        style={{
          display: "grid",
          gridTemplateColumns: "170px 210px 1fr 110px 90px",
          alignItems: "center",
          padding: "10px 16px",
          background: "var(--bg-1)",
          borderBottom: "1px solid var(--border)",
          fontSize: 11,
          fontWeight: 600,
          textTransform: "uppercase",
          letterSpacing: "0.06em",
          color: "var(--text-3)",
          userSelect: "none",
        }}
      >
        <div
          className="sort-header"
          onClick={() => onToggleSort("name")}
          title="Click to sort by Name (A-Z / Z-A)"
        >
          Name <SortIcon field="name" />
        </div>
        <div
          className="sort-header"
          onClick={() => onToggleSort("from")}
          title="Click to sort by Sender Email (A-Z / Z-A)"
        >
          Email <SortIcon field="from" />
        </div>
        <div
          className="sort-header"
          onClick={() => onToggleSort("subject")}
          title="Click to sort by Subject (A-Z / Z-A)"
        >
          Subject & Tags <SortIcon field="subject" />
        </div>
        <div
          className="sort-header"
          style={{ textAlign: "right" }}
          onClick={() => onToggleSort("date")}
          title="Click to sort by Date (Newest / Oldest)"
        >
          Date <SortIcon field="date" />
        </div>
        <div
          className="sort-header"
          style={{ textAlign: "center" }}
          onClick={() => onToggleSort("folder")}
          title="Click to sort by Folder"
        >
          Folder <SortIcon field="folder" />
        </div>
      </div>

      {/* Virtual Scroll Area */}
      <div
        ref={containerRef}
        style={{ height: "60vh", overflowY: "auto", position: "relative" }}
      >
        <div style={{ height: totalHeight, position: "relative" }}>
          {visibleEmails.map((e, i) => {
            const emailTags = tagsByEmail.get(e.id) || [];
            return (
              <div
                key={e.id}
                className="tr-click"
                style={{
                  position: "absolute",
                  top: (startIdx + i) * rowHeight,
                  left: 0,
                  right: 0,
                  height: rowHeight,
                  display: "grid",
                  gridTemplateColumns: "170px 210px 1fr 110px 90px",
                  alignItems: "center",
                  padding: "0 16px",
                  borderBottom: "1px solid var(--border)",
                  fontSize: 13,
                  transition: "background 0.1s",
                }}
                onClick={() => onSelect(e)}
              >
                {/* Name */}
                <div
                  style={{
                    overflow: "hidden",
                    textOverflow: "ellipsis",
                    whiteSpace: "nowrap",
                    color: "var(--text-1)",
                    paddingRight: 10,
                  }}
                  title={e.from_display || undefined}
                >
                  {cleanDisplayName(e.from_display) || "—"}
                </div>

                {/* Email */}
                <div
                  style={{
                    overflow: "hidden",
                    textOverflow: "ellipsis",
                    whiteSpace: "nowrap",
                    fontFamily: "var(--mono)",
                    fontSize: 11,
                    color: "var(--accent)",
                    paddingRight: 10,
                  }}
                  title={e.from_addr}
                >
                  {e.from_addr}
                </div>

                {/* Subject & Tags */}
                <div
                  style={{
                    display: "flex",
                    alignItems: "center",
                    gap: 6,
                    overflow: "hidden",
                    paddingRight: 12,
                  }}
                >
                  <span
                    style={{
                      overflow: "hidden",
                      textOverflow: "ellipsis",
                      whiteSpace: "nowrap",
                      color: "var(--text-0)",
                      fontWeight: 500,
                    }}
                  >
                    {e.subject || <span className="muted">(no subject)</span>}
                  </span>
                  {emailTags.map((t) => (
                    <span
                      key={t.id}
                      className="badge"
                      style={{
                        background: `${t.color}22`,
                        color: t.color,
                        border: `1px solid ${t.color}44`,
                        fontSize: 9,
                        padding: "1px 5px",
                        whiteSpace: "nowrap",
                        flexShrink: 0,
                      }}
                    >
                      {t.tag}
                    </span>
                  ))}
                </div>

                {/* Date */}
                <div
                  style={{
                    textAlign: "right",
                    fontSize: 11,
                    color: "var(--text-3)",
                    whiteSpace: "nowrap",
                  }}
                >
                  {e.date_sent ? new Date(e.date_sent).toLocaleDateString() : "—"}
                </div>

                {/* Folder */}
                <div style={{ textAlign: "center" }}>
                  <span className="badge badge-gray" style={{ fontSize: 9 }}>
                    {e.folder_category}
                  </span>
                </div>
              </div>
            );
          })}
        </div>
      </div>
      <div
        style={{
          padding: "8px 16px",
          background: "var(--bg-3)",
          fontSize: 11,
          color: "var(--text-3)",
          borderTop: "1px solid var(--border)",
        }}
      >
        Showing {emails.length.toLocaleString()} emails
      </div>
    </div>
  );
}

function EmailDetail({
  email,
  caseId,
  evidenceName,
  tags,
  onTagsChanged,
  onClose,
}: {
  email: Email;
  caseId: string;
  evidenceName?: string;
  tags: EmailTag[];
  onTagsChanged: () => void;
  onClose: () => void;
}) {
  const [tab, setTab] = useState<
    "overview" | "notes" | "headers" | "auth" | "mime" | "raw" | "attachments"
  >("overview");
  const [analysisData, setAnalysisData] = useState<any>(null);
  const [analysisLoading, setAnalysisLoading] = useState(false);

  // Load analysis data when auth or headers tab is selected
  useEffect(() => {
    if ((tab === "auth" || tab === "headers") && !analysisData && !analysisLoading) {
      setAnalysisLoading(true);
      invoke<any>("email_headers", { emailId: email.id })
        .then((data) => setAnalysisData(data))
        .catch(console.error)
        .finally(() => setAnalysisLoading(false));
    }
  }, [tab, email.id, analysisData, analysisLoading]);

  let toList: string[] = [];
  let ccList: string[] = [];
  try {
    toList = JSON.parse(email.to_addrs || "[]");
  } catch {}
  try {
    ccList = JSON.parse(email.cc_addrs || "[]");
  } catch {}

  // Risk score color
  const riskColor =
    email.risk_score >= 50
      ? "var(--danger)"
      : email.risk_score >= 25
      ? "var(--warning)"
      : "var(--success)";
  const riskLabel =
    email.risk_score >= 50 ? "HIGH" : email.risk_score >= 25 ? "MEDIUM" : "LOW";

  const tabs = [
    { key: "overview", label: "Overview" },
    { key: "notes", label: `Notes & Tags ${tags.length > 0 ? `(${tags.length})` : ""}` },
    { key: "headers", label: "Headers" },
    { key: "auth", label: "Authentication" },
    { key: "mime", label: "MIME" },
    { key: "raw", label: "Raw" },
    { key: "attachments", label: "Attachments" },
  ];

  return (
    <div>
      <div className="row between mb-4">
        <div style={{ flex: 1, minWidth: 0 }}>
          <div className="row gap-2 mb-1" style={{ flexWrap: "wrap" }}>
            <h2 style={{ fontSize: 16, fontWeight: 600, color: "var(--text-0)" }}>
              {email.subject || "(no subject)"}
            </h2>
            {tags.map((t) => (
              <span
                key={t.id}
                className="badge"
                style={{
                  background: `${t.color}22`,
                  color: t.color,
                  border: `1px solid ${t.color}44`,
                  fontSize: 10,
                }}
              >
                🏷️ {t.tag}
              </span>
            ))}
          </div>
          <p className="muted" style={{ fontSize: 12 }}>
            From: {email.from_addr} ·{" "}
            {email.date_sent ? new Date(email.date_sent).toLocaleString() : "—"}
          </p>
          {evidenceName && (
            <p className="muted" style={{ fontSize: 11 }}>
              Source: {evidenceName}
            </p>
          )}
        </div>
        <button className="btn btn-ghost btn-sm" onClick={onClose}>
          ← Back to Emails
        </button>
      </div>

      <div
        className="row gap-2 mb-4"
        style={{ borderBottom: "1px solid var(--border)", paddingBottom: 0 }}
      >
        {tabs.map((t) => (
          <button
            key={t.key}
            className={`btn btn-sm ${tab === t.key ? "btn-primary" : "btn-ghost"}`}
            style={{ borderRadius: "6px 6px 0 0" }}
            onClick={() => setTab(t.key as any)}
          >
            {t.label}
          </button>
        ))}
      </div>

      <div className="card" style={{ marginTop: 0 }}>
        {tab === "overview" && (
          <div>
            <div className="grid-2 mb-4">
              <div>
                <span className="muted">From</span>
                <p style={{ fontWeight: 500 }}>{email.from_display || email.from_addr}</p>
              </div>
              <div>
                <span className="muted">Date</span>
                <p>{email.date_sent ? new Date(email.date_sent).toLocaleString() : "—"}</p>
              </div>
            </div>
            <div className="mb-4">
              <span className="muted">To</span>
              <p className="mono">{toList.join(", ")}</p>
            </div>
            {ccList.length > 0 && (
              <div className="mb-4">
                <span className="muted">CC</span>
                <p className="mono">{ccList.join(", ")}</p>
              </div>
            )}
            <div className="mb-4">
              <span className="muted">Message-ID</span>
              <p className="mono text-sm">{email.message_id || "—"}</p>
            </div>
            <div className="mb-4">
              <span className="muted">Forensic Tags</span>
              <div className="row gap-2 mt-1" style={{ flexWrap: "wrap" }}>
                {tags.length === 0 ? (
                  <span className="muted" style={{ fontSize: 12 }}>
                    No tags assigned yet.
                  </span>
                ) : (
                  tags.map((t) => (
                    <span
                      key={t.id}
                      className="badge"
                      style={{
                        background: `${t.color}22`,
                        color: t.color,
                        border: `1px solid ${t.color}44`,
                      }}
                    >
                      {t.tag}
                    </span>
                  ))
                )}
                <button
                  className="btn btn-ghost btn-sm"
                  style={{ padding: "2px 8px", fontSize: 11 }}
                  onClick={() => setTab("notes")}
                >
                  + Manage Tags & Notes
                </button>
              </div>
            </div>
            <div className="mb-4">
              <span className="muted">Risk Score</span>
              <p style={{ fontWeight: 600, color: riskColor }}>
                {email.risk_score}/100 ({riskLabel})
              </p>
            </div>
            {email.deleted_recovered && (
              <div className="mb-4">
                <span className="badge badge-red">DELETED / RECOVERED</span>
              </div>
            )}
            {email.body_text && (
              <div>
                <span className="muted">Body Text</span>
                <pre
                  style={{
                    background: "var(--bg-0)",
                    border: "1px solid var(--border)",
                    borderRadius: "var(--r-md)",
                    padding: 16,
                    fontSize: 13,
                    marginTop: 8,
                    maxHeight: 300,
                    overflow: "auto",
                    whiteSpace: "pre-wrap",
                  }}
                >
                  {email.body_text.slice(0, 5000)}
                </pre>
              </div>
            )}
          </div>
        )}

        {tab === "notes" && (
          <EmailNotesAndTagsTab
            emailId={email.id}
            caseId={caseId}
            tags={tags}
            onTagsChanged={onTagsChanged}
          />
        )}

        {tab === "headers" && (
          <div>
            {analysisLoading && <div className="empty">Analyzing headers...</div>}
            {analysisData?.header_analysis && (
              <div className="mb-4">
                <h4 style={{ fontSize: 13, fontWeight: 600, marginBottom: 12 }}>
                  Header Analysis Summary
                </h4>
                <div className="analysis-summary">
                  <div className="analysis-stat">
                    <div className="analysis-stat-val">
                      {analysisData.header_analysis.received_chain?.length || 0}
                    </div>
                    <div className="analysis-stat-label">Received Hops</div>
                  </div>
                  <div className="analysis-stat">
                    <div className="analysis-stat-val" style={{ fontSize: 14 }}>
                      {analysisData.header_analysis.originating_ip || "Unknown"}
                    </div>
                    <div className="analysis-stat-label">Originating IP</div>
                  </div>
                  <div className="analysis-stat">
                    <div
                      className="analysis-stat-val"
                      style={{
                        color:
                          analysisData.header_analysis.routing_anomalies?.length > 0
                            ? "var(--danger)"
                            : "var(--text-0)",
                      }}
                    >
                      {analysisData.header_analysis.routing_anomalies?.length || 0}
                    </div>
                    <div className="analysis-stat-label">Routing Anomalies</div>
                  </div>
                  <div className="analysis-stat">
                    <div
                      className="analysis-stat-val"
                      style={{
                        color:
                          analysisData.header_analysis.clock_skew?.length > 0
                            ? "var(--warning)"
                            : "var(--text-0)",
                      }}
                    >
                      {analysisData.header_analysis.clock_skew?.length || 0}
                    </div>
                    <div className="analysis-stat-label">Clock Skew Events</div>
                  </div>
                </div>
              </div>
            )}
            <pre
              className="mono"
              style={{
                fontSize: 11,
                background: "var(--bg-0)",
                padding: 16,
                borderRadius: "var(--r-md)",
                border: "1px solid var(--border)",
                maxHeight: 500,
                overflow: "auto",
              }}
            >
              {email.headers_raw}
            </pre>
          </div>
        )}

        {tab === "auth" && (
          <div>
            <h4 style={{ fontSize: 13, fontWeight: 600, marginBottom: 12 }}>
              Authentication Results (SPF / DKIM / DMARC)
            </h4>
            <pre
              className="mono"
              style={{
                fontSize: 12,
                background: "var(--bg-0)",
                padding: 16,
                borderRadius: "var(--r-md)",
                border: "1px solid var(--border)",
              }}
            >
              {analysisData
                ? JSON.stringify(analysisData.authentication || {}, null, 2)
                : "Loading authentication verification..."}
            </pre>
          </div>
        )}

        {tab === "mime" && (
          <div>
            <h4 style={{ fontSize: 13, fontWeight: 600, marginBottom: 12 }}>MIME Tree Structure</h4>
            <div
              style={{
                padding: 16,
                background: "var(--bg-0)",
                borderRadius: "var(--r-md)",
                border: "1px solid var(--border)",
                fontSize: 13,
              }}
            >
              <div>
                📦 <strong>multipart/alternative</strong>
              </div>
              <div style={{ marginLeft: 20 }}>
                ├── 📄 text/plain ({email.body_text?.length || 0} chars)
              </div>
              {email.body_html && (
                <div style={{ marginLeft: 20 }}>
                  └── 🌐 text/html ({email.body_html.length} chars)
                </div>
              )}
            </div>
          </div>
        )}

        {tab === "raw" && (
          <div>
            <h4 style={{ fontSize: 13, fontWeight: 600, marginBottom: 12 }}>Full Raw Message</h4>
            <pre
              className="mono"
              style={{
                fontSize: 11,
                color: "var(--text-2)",
                whiteSpace: "pre-wrap",
                wordBreak: "break-all",
                maxHeight: 600,
                overflow: "auto",
                background: "var(--bg-0)",
                padding: 16,
                borderRadius: "var(--r-md)",
                border: "1px solid var(--border)",
              }}
            >
              {email.headers_raw || "No raw headers available"}
              {"\n\n--- BODY ---\n\n"}
              {email.body_text || email.body_html || "No body available"}
            </pre>
          </div>
        )}

        {tab === "attachments" && (
          <div>
            <h4 style={{ fontSize: 13, fontWeight: 600, marginBottom: 12 }}>Attachments</h4>
            <EmailAttachments emailId={email.id} />
          </div>
        )}
      </div>
    </div>
  );
}

function EmailNotesAndTagsTab({
  emailId,
  caseId,
  tags,
  onTagsChanged,
}: {
  emailId: string;
  caseId: string;
  tags: EmailTag[];
  onTagsChanged: () => void;
}) {
  const [notes, setNotes] = useState<any[]>([]);
  const [newNote, setNewNote] = useState("");
  const [customTag, setCustomTag] = useState("");
  const [loading, setLoading] = useState(true);
  const [savingNote, setSavingNote] = useState(false);

  const PRESET_TAGS = [
    { name: "Key Evidence", color: "#ef4444" },
    { name: "Privileged", color: "#8b5cf6" },
    { name: "Hot", color: "#f97316" },
    { name: "Responsive", color: "#22c55e" },
    { name: "Suspicious", color: "#eab308" },
    { name: "Reviewed", color: "#3b82f6" },
  ];

  const loadNotes = async () => {
    setLoading(true);
    try {
      const emailNotes = await invoke<any[]>("email_notes_list", { emailId });
      setNotes(emailNotes);
    } catch (e) {
      console.error("Failed to load email notes:", e);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    loadNotes();
  }, [emailId]);

  const handleToggleTag = async (tagName: string, color?: string) => {
    const existing = tags.find((t) => t.tag.toLowerCase() === tagName.toLowerCase());
    try {
      if (existing) {
        await invoke("email_tag_remove", {
          input: { case_id: caseId, email_id: emailId, tag: existing.tag },
        });
      } else {
        await invoke("email_tag_add", {
          input: {
            case_id: caseId,
            email_id: emailId,
            tag: tagName,
            color: color || "#3b82f6",
            created_by: "Investigator",
          },
        });
      }
      onTagsChanged();
    } catch (err: any) {
      alert(`Error updating tag: ${err}`);
    }
  };

  const handleAddCustomTag = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!customTag.trim()) return;
    await handleToggleTag(customTag.trim(), "#3b82f6");
    setCustomTag("");
  };

  const handleAddNote = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!newNote.trim()) return;
    setSavingNote(true);
    try {
      await invoke("email_note_add", {
        input: {
          case_id: caseId,
          email_id: emailId,
          content: newNote.trim(),
          author: "Investigator",
        },
      });
      setNewNote("");
      loadNotes();
    } catch (err: any) {
      alert(`Error saving email note: ${err}`);
    } finally {
      setSavingNote(false);
    }
  };

  const handleDeleteNote = async (noteId: string) => {
    if (!confirm("Delete this email note?")) return;
    try {
      await invoke("email_note_delete", { noteId });
      loadNotes();
    } catch (err: any) {
      alert(`Error deleting note: ${err}`);
    }
  };

  return (
    <div>
      {/* Tagging Section */}
      <div className="mb-4">
        <h4 style={{ fontSize: 14, fontWeight: 600, color: "var(--text-0)", marginBottom: 8 }}>
          Forensic Email Tags
        </h4>
        <p className="muted mb-4" style={{ fontSize: 12 }}>
          Click a preset tag to toggle it on/off for this email message.
        </p>

        <div className="row gap-2 mb-4" style={{ flexWrap: "wrap" }}>
          {PRESET_TAGS.map((pt) => {
            const active = tags.some((t) => t.tag.toLowerCase() === pt.name.toLowerCase());
            return (
              <button
                key={pt.name}
                className="btn btn-sm"
                style={{
                  background: active ? pt.color : "var(--bg-3)",
                  color: active ? "#fff" : "var(--text-1)",
                  border: `1px solid ${active ? pt.color : "var(--border)"}`,
                  fontWeight: active ? 600 : 400,
                  fontSize: 12,
                  padding: "5px 12px",
                }}
                onClick={() => handleToggleTag(pt.name, pt.color)}
              >
                {active ? "✓ " : "+ "}
                {pt.name}
              </button>
            );
          })}
        </div>

        {/* Custom Tag Input */}
        <form onSubmit={handleAddCustomTag} className="row gap-2" style={{ maxWidth: 360 }}>
          <input
            className="input"
            style={{ padding: "6px 12px", fontSize: 12 }}
            placeholder="Add custom tag (e.g. 'Accounting Lead')..."
            value={customTag}
            onChange={(e) => setCustomTag(e.target.value)}
          />
          <button type="submit" className="btn btn-ghost btn-sm" disabled={!customTag.trim()}>
            + Add Tag
          </button>
        </form>
      </div>

      <hr style={{ borderColor: "var(--border)", margin: "24px 0" }} />

      {/* Email Specific Notes */}
      <div>
        <h4 style={{ fontSize: 14, fontWeight: 600, color: "var(--text-0)", marginBottom: 8 }}>
          Investigator Observations for this Email
        </h4>
        <p className="muted mb-4" style={{ fontSize: 12 }}>
          Recorded notes are timestamped and tied directly to this email message.
        </p>

        {/* Add Note Form */}
        <form onSubmit={handleAddNote} className="mb-4">
          <textarea
            className="textarea"
            placeholder="Write an observation regarding this email's contents, headers, or relevance..."
            value={newNote}
            onChange={(e) => setNewNote(e.target.value)}
            style={{ minHeight: 80, marginBottom: 8 }}
          />
          <button
            type="submit"
            className="btn btn-primary btn-sm"
            disabled={savingNote || !newNote.trim()}
          >
            {savingNote ? "Saving..." : "+ Record Email Note"}
          </button>
        </form>

        {/* Notes List */}
        {loading ? (
          <div className="empty">Loading email notes...</div>
        ) : notes.length === 0 ? (
          <div
            style={{
              padding: 16,
              background: "var(--bg-0)",
              borderRadius: "var(--r-md)",
              border: "1px solid var(--border)",
              color: "var(--text-3)",
              fontSize: 12,
              textAlign: "center",
            }}
          >
            No specific notes recorded for this email yet.
          </div>
        ) : (
          <div style={{ display: "flex", flexDirection: "column", gap: 10 }}>
            {notes.map((n) => (
              <div
                key={n.id}
                style={{
                  padding: 14,
                  background: "var(--bg-0)",
                  borderRadius: "var(--r-md)",
                  border: "1px solid var(--border)",
                }}
              >
                <div className="row between mb-2">
                  <div style={{ fontSize: 11, color: "var(--text-3)" }}>
                    <strong style={{ color: "var(--text-1)" }}>{n.author}</strong> ·{" "}
                    {new Date(n.created_at).toLocaleString()}
                  </div>
                  <button
                    className="btn btn-ghost btn-sm"
                    style={{ padding: "2px 6px", fontSize: 10, color: "var(--danger)" }}
                    onClick={() => handleDeleteNote(n.id)}
                  >
                    Delete
                  </button>
                </div>
                <div
                  style={{
                    fontSize: 13,
                    color: "var(--text-1)",
                    whiteSpace: "pre-wrap",
                    lineHeight: 1.5,
                  }}
                >
                  {n.content}
                </div>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}

function EmailAttachments({ emailId }: { emailId: string }) {
  const [attachments, setAttachments] = useState<any[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    invoke<any[]>("email_attachments", { emailId })
      .then((data) => setAttachments(data))
      .catch(() => setAttachments([]))
      .finally(() => setLoading(false));
  }, [emailId]);

  if (loading) return <div className="empty">Loading attachments...</div>;

  if (attachments.length === 0) {
    return <div className="empty">No attachments in this email</div>;
  }

  return (
    <div>
      <table>
        <thead>
          <tr>
            <th className="th">Filename</th>
            <th className="th">Type</th>
            <th className="th" style={{ width: 80 }}>
              Size
            </th>
            <th className="th" style={{ width: 120 }}>
              SHA-256
            </th>
          </tr>
        </thead>
        <tbody>
          {attachments.map((att) => (
            <tr key={att.id}>
              <td className="td">{att.filename || <span className="muted">unnamed</span>}</td>
              <td className="td">
                <span className="badge badge-blue">{att.mime_type}</span>
              </td>
              <td className="td">{formatBytes(att.size_bytes)}</td>
              <td className="td mono" style={{ fontSize: 10 }}>
                {att.sha256?.slice(0, 10)}…
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}

function formatBytes(bytes: number): string {
  if (bytes === 0) return "0 B";
  const k = 1024;
  const sizes = ["B", "KB", "MB", "GB"];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + " " + sizes[i];
}