import { useState, useMemo, useEffect, useRef, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { RichEmailBodyViewer } from "../components/RichEmailBodyViewer";
import { BookmarkButton } from "../components/BookmarkButton";

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
  flags?: string | null;
  is_deleted?: boolean;
  recovery_status: string;
  deleted_recovered: boolean;
  risk_score: number;
  attachment_count?: number;
  image_count?: number;
}

export interface ColumnSettings {
  name: boolean;
  from: boolean;
  to: boolean;
  subject: boolean;
  attachments: boolean;
  date: boolean;
  folder: boolean;
  risk: boolean;
  tag: boolean;
}

export const DEFAULT_COLUMNS: ColumnSettings = {
  name: true,
  from: true,
  to: false,
  subject: true,
  attachments: true,
  date: true,
  folder: true,
  risk: true,
  tag: true,
};

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
  evidenceFilter,
  onEvidenceFilterChange,
  onViewEntity,
}: {
  caseId: string;
  filter?: string;
  evidenceFilter?: string | null;
  onEvidenceFilterChange?: (evidenceId: string | null) => void;
  onViewEntity?: (email: string) => void;
}) {
  const [emails, setEmails] = useState<Email[]>([]);
  const [evidence, setEvidence] = useState<Evidence[]>([]);
  const [tags, setTags] = useState<EmailTag[]>([]);
  const [loading, setLoading] = useState(true);
  
  // Search & Filter state
  const [selectedEvidenceId, setSelectedEvidenceId] = useState<string | null>(evidenceFilter || null);

  useEffect(() => {
    if (evidenceFilter !== undefined) {
      setSelectedEvidenceId(evidenceFilter);
    }
  }, [evidenceFilter]);

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

  // Column customization state
  const [columns, setColumns] = useState<ColumnSettings>(() => {
    try {
      const saved = localStorage.getItem("j12_email_columns");
      if (saved) return { ...DEFAULT_COLUMNS, ...JSON.parse(saved) };
    } catch {}
    return DEFAULT_COLUMNS;
  });
  const [showColumnPicker, setShowColumnPicker] = useState(false);
  const columnPickerRef = useRef<HTMLDivElement>(null);

  const toggleColumn = (key: keyof ColumnSettings) => {
    setColumns((prev) => {
      const updated = { ...prev, [key]: !prev[key] };
      try {
        localStorage.setItem("j12_email_columns", JSON.stringify(updated));
      } catch {}
      return updated;
    });
  };

  const resetColumns = () => {
    setColumns(DEFAULT_COLUMNS);
    try {
      localStorage.setItem("j12_email_columns", JSON.stringify(DEFAULT_COLUMNS));
    } catch {}
  };

  useEffect(() => {
    if (!showColumnPicker) return;
    const handler = (e: MouseEvent) => {
      if (columnPickerRef.current && !columnPickerRef.current.contains(e.target as Node)) {
        setShowColumnPicker(false);
      }
    };
    document.addEventListener("mousedown", handler);
    return () => document.removeEventListener("mousedown", handler);
  }, [showColumnPicker]);

  const load = async () => {
    if (!caseId) return;
    setLoading(true);
    try {
      const [em, ev, tg] = await Promise.all([
        invoke<Email[]>("email_list", { input: { case_id: caseId, limit: 10000 } }).catch((err) => {
          console.error("Failed to load email_list:", err);
          return [] as Email[];
        }),
        invoke<Evidence[]>("evidence_list", { input: { case_id: caseId } }).catch((err) => {
          console.error("Failed to load evidence_list:", err);
          return [] as Evidence[];
        }),
        invoke<EmailTag[]>("email_tags_list", { input: { case_id: caseId } }).catch((err) => {
          console.error("Failed to load email_tags_list:", err);
          return [] as EmailTag[];
        }),
      ]);
      setEmails(em);
      setEvidence(ev);
      setTags(tg);
    } catch (e) {
      console.error("Error loading email data:", e);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    load();
  }, [caseId, filter]);

  const loadTags = async () => {
    if (!caseId) return;
    try {
      const tg = await invoke<EmailTag[]>("email_tags_list", { input: { case_id: caseId } });
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

  const evidenceCounts = useMemo(() => {
    const counts: Record<string, number> = {};
    emails.forEach((e) => {
      counts[e.evidence_id] = (counts[e.evidence_id] || 0) + 1;
    });
    return counts;
  }, [emails]);

  const filteredBySource = useMemo(() => {
    if (!selectedEvidenceId) return uniqueEmails;
    return uniqueEmails.filter((e) => e.evidence_id === selectedEvidenceId);
  }, [uniqueEmails, selectedEvidenceId]);

  // Folder Category Filter
  const filteredByFolder = useMemo(() => {
    if (!filter || filter === "all") return filteredBySource;
    if (filter === "sent") {
      return filteredBySource.filter((e) => e.folder_category === "sent");
    }
    if (filter === "inbox") {
      return filteredBySource.filter((e) => e.folder_category === "inbox");
    }
    if (filter === "important") {
      return filteredBySource.filter(
        (e) =>
          e.folder_category === "important" ||
          (e.folder_name && e.folder_name.toLowerCase().includes("important")) ||
          (e.flags && e.flags.toLowerCase().includes("important"))
      );
    }
    if (filter === "soft_deleted" || filter === "trash" || filter === "deleted") {
      return filteredBySource.filter(
        (e) =>
          e.folder_category === "soft_deleted" ||
          e.folder_category === "trash" ||
          e.folder_category === "deleted" ||
          e.is_deleted ||
          e.deleted_recovered ||
          e.recovery_status === "soft_deleted" ||
          e.recovery_status === "recoverable"
      );
    }
    if (filter === "hard_deleted") {
      return filteredBySource.filter(
        (e) => e.recovery_status === "hard_deleted" || e.recovery_status === "purged"
      );
    }
    if (filter === "recoverable") {
      return filteredBySource.filter(
        (e) => e.recovery_status === "recoverable" || e.deleted_recovered
      );
    }
    if (filter === "drafts") {
      return filteredBySource.filter((e) => e.folder_category === "drafts");
    }
    if (filter === "spam") {
      return filteredBySource.filter((e) => e.folder_category === "spam");
    }
    if (filter === "other") {
      return filteredBySource.filter(
        (e) =>
          e.folder_category === "other" ||
          (!["inbox", "important", "sent", "drafts", "spam", "trash", "soft_deleted"].includes(e.folder_category) &&
            !e.is_deleted)
      );
    }
    return filteredBySource;
  }, [filteredBySource, filter]);

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

  if (loading) {
    return (
      <div className="card" style={{ padding: "60px 20px", textAlign: "center", minHeight: 320, display: "flex", flexDirection: "column", alignItems: "center", justifyContent: "center" }}>
        <div className="spinner mb-3" style={{ width: 28, height: 28, border: "3px solid var(--border)", borderTopColor: "var(--accent)", borderRadius: "50%", animation: "spin 0.8s linear infinite" }} />
        <div style={{ fontSize: 16, fontWeight: 600, color: "var(--text-0)" }}>
          Loading emails...
        </div>
        <div className="muted text-sm mt-1">Retrieving forensic email records and metadata</div>
      </div>
    );
  }

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

              {/* Column Settings Picker */}
              <div style={{ position: "relative" }}>
                <button
                  className={`btn btn-sm ${showColumnPicker ? "btn-primary" : "btn-ghost"}`}
                  onClick={() => setShowColumnPicker(!showColumnPicker)}
                  title="Customize table columns"
                >
                  ⚙️ Columns
                </button>

                {showColumnPicker && (
                  <div
                    ref={columnPickerRef}
                    style={{
                      position: "absolute",
                      right: 0,
                      top: "calc(100% + 6px)",
                      zIndex: 9999,
                      background: "var(--bg-1)",
                      border: "1px solid var(--border)",
                      borderRadius: "var(--r-md)",
                      boxShadow: "0 10px 25px rgba(0,0,0,0.5)",
                      padding: 14,
                      width: 220,
                    }}
                  >
                    <div className="row between mb-2" style={{ alignItems: "center" }}>
                      <strong style={{ fontSize: 12, color: "var(--text-0)" }}>Visible Columns</strong>
                      <button
                        className="btn btn-ghost btn-sm"
                        style={{ fontSize: 10, padding: "2px 6px" }}
                        onClick={resetColumns}
                      >
                        Reset
                      </button>
                    </div>
                    <div style={{ display: "flex", flexDirection: "column", gap: 6, fontSize: 12 }}>
                      <label className="row gap-2" style={{ cursor: "pointer" }}>
                        <input type="checkbox" checked={columns.name} onChange={() => toggleColumn("name")} />
                        <span>Sender Name</span>
                      </label>
                      <label className="row gap-2" style={{ cursor: "pointer" }}>
                        <input type="checkbox" checked={columns.from} onChange={() => toggleColumn("from")} />
                        <span>From Email</span>
                      </label>
                      <label className="row gap-2" style={{ cursor: "pointer" }}>
                        <input type="checkbox" checked={columns.to} onChange={() => toggleColumn("to")} />
                        <span>To Recipient</span>
                      </label>
                      <label className="row gap-2" style={{ cursor: "pointer" }}>
                        <input type="checkbox" checked={columns.subject} onChange={() => toggleColumn("subject")} />
                        <span>Subject &amp; Tags</span>
                      </label>
                      <label className="row gap-2" style={{ cursor: "pointer" }}>
                        <input type="checkbox" checked={columns.attachments} onChange={() => toggleColumn("attachments")} />
                        <span>Attachments (📎/🖼️)</span>
                      </label>
                      <label className="row gap-2" style={{ cursor: "pointer" }}>
                        <input type="checkbox" checked={columns.date} onChange={() => toggleColumn("date")} />
                        <span>Date Sent</span>
                      </label>
                      <label className="row gap-2" style={{ cursor: "pointer" }}>
                        <input type="checkbox" checked={columns.folder} onChange={() => toggleColumn("folder")} />
                        <span>Folder Category</span>
                      </label>
                      <label className="row gap-2" style={{ cursor: "pointer" }}>
                        <input type="checkbox" checked={columns.risk} onChange={() => toggleColumn("risk")} />
                        <span>Risk Score</span>
                      </label>
                      <label className="row gap-2" style={{ cursor: "pointer" }}>
                        <input type="checkbox" checked={columns.tag} onChange={() => toggleColumn("tag")} />
                        <span>Tag / Locker (🔖)</span>
                      </label>
                    </div>
                  </div>
                )}
              </div>

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
                onClick={() => {
                  setSelectedEvidenceId(null);
                  onEvidenceFilterChange?.(null);
                }}
              >
                🌐 All Sources ({emails.length.toLocaleString()})
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
                    onClick={() => {
                      const next = isSelected ? null : ev.id;
                      setSelectedEvidenceId(next);
                      onEvidenceFilterChange?.(next);
                    }}
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
                  onClick={() => {
                    setSelectedEvidenceId(null);
                    onEvidenceFilterChange?.(null);
                  }}
                >
                  ✕ Show All ({emails.length.toLocaleString()})
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
                  onClick={handleResetFilters}
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
            columns={columns}
            caseId={caseId}
          />
        </>
      )}
    </div>
  );
}

interface ColumnWidths {
  name: number;
  from: number;
  to: number;
  subject: number;
  attachments: number;
  date: number;
  folder: number;
  risk: number;
  tag: number;
}

const DEFAULT_COL_WIDTHS: ColumnWidths = {
  name: 150,
  from: 180,
  to: 160,
  subject: 320,
  attachments: 85,
  date: 105,
  folder: 85,
  risk: 65,
  tag: 65,
};

function VirtualEmailList({
  emails,
  tagsByEmail,
  sortField,
  sortDir,
  onToggleSort,
  onSelect,
  onViewEntity,
  columns,
  caseId,
}: {
  emails: Email[];
  tagsByEmail: Map<string, EmailTag[]>;
  sortField: SortField;
  sortDir: SortDir;
  onToggleSort: (field: SortField) => void;
  onSelect: (e: Email) => void;
  onViewEntity?: (email: string) => void;
  columns: ColumnSettings;
  caseId: string;
}) {
  const [scrollOffset, setScrollOffset] = useState(0);
  const containerRef = useRef<HTMLDivElement>(null);
  const rowHeight = 44;
  const visibleCount = 40;

  // Resizable column widths state
  const [colWidths, setColWidths] = useState<ColumnWidths>(() => {
    try {
      const saved = localStorage.getItem("j12_email_col_widths");
      if (saved) return { ...DEFAULT_COL_WIDTHS, ...JSON.parse(saved) };
    } catch {}
    return DEFAULT_COL_WIDTHS;
  });

  const resizingRef = useRef<{ col: keyof ColumnWidths; startX: number; startW: number } | null>(null);

  const startResize = (col: keyof ColumnWidths, e: React.MouseEvent) => {
    e.preventDefault();
    e.stopPropagation();
    resizingRef.current = { col, startX: e.clientX, startW: colWidths[col] };

    const onMouseMove = (ev: MouseEvent) => {
      if (!resizingRef.current) return;
      const delta = ev.clientX - resizingRef.current.startX;
      const newWidth = Math.max(45, resizingRef.current.startW + delta);
      setColWidths((prev) => {
        const next = { ...prev, [resizingRef.current!.col]: newWidth };
        try {
          localStorage.setItem("j12_email_col_widths", JSON.stringify(next));
        } catch {}
        return next;
      });
    };

    const onMouseUp = () => {
      resizingRef.current = null;
      window.removeEventListener("mousemove", onMouseMove);
      window.removeEventListener("mouseup", onMouseUp);
    };

    window.addEventListener("mousemove", onMouseMove);
    window.addEventListener("mouseup", onMouseUp);
  };

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

  // Compute dynamic grid template with resizable widths
  const gridTemplate = useMemo(() => {
    const parts: string[] = [];
    if (columns.name) parts.push(`${colWidths.name}px`);
    if (columns.from) parts.push(`${colWidths.from}px`);
    if (columns.to) parts.push(`${colWidths.to}px`);
    if (columns.subject) parts.push(`${colWidths.subject}px`);
    if (columns.attachments) parts.push(`${colWidths.attachments}px`);
    if (columns.date) parts.push(`${colWidths.date}px`);
    if (columns.folder) parts.push(`${colWidths.folder}px`);
    if (columns.risk) parts.push(`${colWidths.risk}px`);
    if (columns.tag) parts.push(`${colWidths.tag}px`);
    return parts.length > 0 ? parts.join(" ") : "1fr";
  }, [columns, colWidths]);

  const totalHeight = emails.length * rowHeight;
  const startIdx = Math.max(0, Math.floor(scrollOffset / rowHeight));
  const endIdx = Math.min(emails.length, startIdx + visibleCount);
  const visibleEmails = emails.slice(startIdx, endIdx);

  const SortIcon = ({ field }: { field: SortField }) => (
    <span style={{ opacity: sortField === field ? 1 : 0.35, marginLeft: 4, fontSize: 11 }}>
      {sortField === field ? (sortDir === "asc" ? "▲" : "▼") : "⇅"}
    </span>
  );

  const Resizer = ({ col }: { col: keyof ColumnWidths }) => (
    <div
      style={{
        position: "absolute",
        right: 0,
        top: 0,
        bottom: 0,
        width: 8,
        cursor: "col-resize",
        zIndex: 5,
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
      }}
      onMouseDown={(e) => startResize(col, e)}
      onClick={(e) => e.stopPropagation()}
      title="Drag to resize column width"
    >
      <div style={{ width: 2, height: "60%", background: "var(--border)", borderRadius: 1 }} />
    </div>
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
    <div className="card" style={{ padding: 0, overflowX: "auto", overflowY: "hidden" }}>
      <div style={{ minWidth: "100%", width: "max-content" }}>
        {/* Interactive Sortable & Resizable Header */}
        <div
          style={{
            display: "grid",
            gridTemplateColumns: gridTemplate,
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
            gap: 8,
          }}
        >
          {columns.name && (
            <div
              className="sort-header"
              style={{ position: "relative", paddingRight: 10 }}
              onClick={() => onToggleSort("name")}
              title="Click to sort by Name (A-Z / Z-A). Drag right edge to resize."
            >
              Name <SortIcon field="name" />
              <Resizer col="name" />
            </div>
          )}
          {columns.from && (
            <div
              className="sort-header"
              style={{ position: "relative", paddingRight: 10 }}
              onClick={() => onToggleSort("from")}
              title="Click to sort by Sender Email (A-Z / Z-A). Drag right edge to resize."
            >
              From <SortIcon field="from" />
              <Resizer col="from" />
            </div>
          )}
          {columns.to && (
            <div style={{ position: "relative", paddingRight: 10 }} title="Recipient Email. Drag right edge to resize.">
              To
              <Resizer col="to" />
            </div>
          )}
          {columns.subject && (
            <div
              className="sort-header"
              style={{ position: "relative", paddingRight: 10 }}
              onClick={() => onToggleSort("subject")}
              title="Click to sort by Subject (A-Z / Z-A). Drag right edge to resize."
            >
              Subject &amp; Tags <SortIcon field="subject" />
              <Resizer col="subject" />
            </div>
          )}
          {columns.attachments && (
            <div style={{ position: "relative", textAlign: "center", paddingRight: 10 }} title="Attachments &amp; Photos. Drag right edge to resize.">
              📎 Files
              <Resizer col="attachments" />
            </div>
          )}
          {columns.date && (
            <div
              className="sort-header"
              style={{ position: "relative", textAlign: "right", paddingRight: 10 }}
              onClick={() => onToggleSort("date")}
              title="Click to sort by Date (Newest / Oldest). Drag right edge to resize."
            >
              Date <SortIcon field="date" />
              <Resizer col="date" />
            </div>
          )}
          {columns.folder && (
            <div
              className="sort-header"
              style={{ position: "relative", textAlign: "center", paddingRight: 10 }}
              onClick={() => onToggleSort("folder")}
              title="Click to sort by Folder. Drag right edge to resize."
            >
              Folder <SortIcon field="folder" />
              <Resizer col="folder" />
            </div>
          )}
          {columns.risk && (
            <div
              className="sort-header"
              style={{ position: "relative", textAlign: "center", paddingRight: 10 }}
              onClick={() => onToggleSort("risk")}
              title="Click to sort by Risk Score. Drag right edge to resize."
            >
              Risk <SortIcon field="risk" />
              <Resizer col="risk" />
            </div>
          )}
          {columns.tag && (
            <div style={{ position: "relative", textAlign: "center", paddingRight: 10 }} title="Evidence Locker Bookmark. Drag right edge to resize.">
              Locker
              <Resizer col="tag" />
            </div>
          )}
        </div>

        {/* Virtual Scroll Area */}
        <div
          ref={containerRef}
          style={{ height: "60vh", overflowY: "auto", position: "relative" }}
        >
        <div style={{ height: totalHeight, position: "relative" }}>
          {visibleEmails.map((e, i) => {
            const emailTags = tagsByEmail.get(e.id) || [];
            const attCount = e.attachment_count || 0;
            const imgCount = e.image_count || 0;

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
                  gridTemplateColumns: gridTemplate,
                  alignItems: "center",
                  padding: "0 16px",
                  borderBottom: "1px solid var(--border)",
                  fontSize: 13,
                  transition: "background 0.1s",
                  gap: 8,
                }}
                onClick={() => onSelect(e)}
              >
                {/* Name */}
                {columns.name && (
                  <div
                    style={{
                      overflow: "hidden",
                      textOverflow: "ellipsis",
                      whiteSpace: "nowrap",
                      color: "var(--text-1)",
                    }}
                    title={e.from_display || undefined}
                  >
                    {cleanDisplayName(e.from_display) || "—"}
                  </div>
                )}

                {/* From Email */}
                {columns.from && (
                  <div
                    style={{
                      overflow: "hidden",
                      textOverflow: "ellipsis",
                      whiteSpace: "nowrap",
                      fontFamily: "var(--mono)",
                      fontSize: 11,
                      color: "var(--accent)",
                    }}
                    title={e.from_addr}
                  >
                    {e.from_addr}
                  </div>
                )}

                {/* To Recipient */}
                {columns.to && (
                  <div
                    style={{
                      overflow: "hidden",
                      textOverflow: "ellipsis",
                      whiteSpace: "nowrap",
                      fontFamily: "var(--mono)",
                      fontSize: 11,
                      color: "var(--text-2)",
                    }}
                    title={e.to_addrs}
                  >
                    {e.to_addrs || "—"}
                  </div>
                )}

                {/* Subject & Tags */}
                {columns.subject && (
                  <div
                    style={{
                      display: "flex",
                      alignItems: "center",
                      gap: 6,
                      overflow: "hidden",
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
                )}

                {/* Attachments / Photos Badge Indicator */}
                {columns.attachments && (
                  <div style={{ display: "flex", justifyContent: "center", gap: 4, alignItems: "center" }}>
                    {attCount > 0 ? (
                      <span
                        className="badge badge-blue"
                        style={{
                          fontSize: 10,
                          padding: "1px 6px",
                          display: "inline-flex",
                          alignItems: "center",
                          gap: 2,
                          fontWeight: 700,
                        }}
                        title={`${attCount} total attachment(s)`}
                      >
                        📎 {attCount}
                      </span>
                    ) : null}

                    {imgCount > 0 ? (
                      <span
                        className="badge badge-green"
                        style={{
                          fontSize: 10,
                          padding: "1px 6px",
                          display: "inline-flex",
                          alignItems: "center",
                          gap: 2,
                          fontWeight: 700,
                        }}
                        title={`${imgCount} image attachment(s)`}
                      >
                        🖼️ {imgCount}
                      </span>
                    ) : null}

                    {attCount === 0 && imgCount === 0 && (
                      <span className="muted" style={{ opacity: 0.25, fontSize: 11 }}>—</span>
                    )}
                  </div>
                )}

                {/* Date */}
                {columns.date && (
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
                )}

                {/* Folder */}
                {columns.folder && (
                  <div style={{ textAlign: "center" }}>
                    <span className="badge badge-gray" style={{ fontSize: 9 }}>
                      {e.folder_category}
                    </span>
                  </div>
                )}

                {/* Risk Score */}
                {columns.risk && (
                  <div style={{ textAlign: "center" }}>
                    <span
                      className={`badge ${
                        e.risk_score >= 50
                          ? "badge-red"
                          : e.risk_score >= 25
                          ? "badge-orange"
                          : "badge-gray"
                      }`}
                      style={{ fontSize: 10, fontWeight: 700, minWidth: 26, textAlign: "center" }}
                    >
                      {e.risk_score}
                    </span>
                  </div>
                )}

                {/* Locker Bookmark Button */}
                {columns.tag && (
                  <div
                    style={{ display: "flex", justifyContent: "center" }}
                    onClick={(ev) => ev.stopPropagation()}
                  >
                    <BookmarkButton
                      caseId={caseId}
                      itemId={e.id}
                      itemType="email"
                      compact={true}
                    />
                  </div>
                )}
              </div>
            );
          })}
        </div>
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
  email: initialEmail,
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
  const [email, setEmail] = useState<Email>(initialEmail);

  useEffect(() => {
    setEmail(initialEmail);
    if (!initialEmail.body_text && !initialEmail.body_html && !initialEmail.headers_raw) {
      invoke<Email | null>("email_get", { input: { id: initialEmail.id } }).then((full) => {
        if (full) setEmail(full);
      }).catch(console.error);
    }
  }, [initialEmail.id]);

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
            <div className="mb-4">
              <span className="muted" style={{ fontWeight: 600 }}>Message Content</span>
              <RichEmailBodyViewer
                bodyText={email.body_text}
                bodyHtml={email.body_html}
                emailId={email.id}
                defaultMode="rendered"
              />
            </div>
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
  const [zoomImage, setZoomImage] = useState<{ src: string; filename: string } | null>(null);

  useEffect(() => {
    invoke<any[]>("email_attachments", { input: { email_id: emailId } })
      .then((data) => setAttachments(data))
      .catch(() => setAttachments([]))
      .finally(() => setLoading(false));
  }, [emailId]);

  const exportSingle = async (attId: string) => {
    try {
      const path = await invoke<string>("export_attachment", { input: { attachment_id: attId } });
      alert(`Exported to: ${path}`);
    } catch (e) {
      alert(`Export failed: ${e}`);
    }
  };

  if (loading) return <div className="empty">Loading attachments...</div>;

  if (attachments.length === 0) {
    return <div className="empty">No attachments in this email</div>;
  }

  const isImage = (mime: string, name: string) => {
    const lower = (name || "").toLowerCase();
    return (mime || "").startsWith("image/") || lower.endsWith(".jpg") || lower.endsWith(".jpeg") || lower.endsWith(".png") || lower.endsWith(".gif") || lower.endsWith(".webp") || lower.endsWith(".svg");
  };

  const imageAttachments = attachments.filter((a) => isImage(a.mime_type, a.filename));

  return (
    <div>
      {/* Zoom Modal */}
      {zoomImage && (
        <div
          style={{
            position: "fixed",
            inset: 0,
            background: "rgba(0,0,0,0.85)",
            backdropFilter: "blur(6px)",
            display: "flex",
            flexDirection: "column",
            alignItems: "center",
            justifyContent: "center",
            zIndex: 10000,
            padding: 24,
          }}
          onClick={() => setZoomImage(null)}
        >
          <div
            style={{
              maxWidth: "90vw",
              maxHeight: "85vh",
              display: "flex",
              flexDirection: "column",
              alignItems: "center",
              background: "var(--bg-1)",
              borderRadius: "var(--r-md)",
              padding: 16,
              border: "1px solid var(--border)",
            }}
            onClick={(e) => e.stopPropagation()}
          >
            <div className="row between" style={{ width: "100%", marginBottom: 12 }}>
              <span style={{ fontSize: 14, fontWeight: 700, color: "var(--text-0)" }}>
                🖼️ {zoomImage.filename}
              </span>
              <button className="btn btn-ghost btn-sm" onClick={() => setZoomImage(null)}>✕ Close</button>
            </div>
            <img
              src={zoomImage.src}
              alt={zoomImage.filename}
              style={{ maxWidth: "100%", maxHeight: "75vh", objectFit: "contain", borderRadius: 4 }}
            />
          </div>
        </div>
      )}

      {/* Image Gallery Strip if photos attached */}
      {imageAttachments.length > 0 && (
        <div style={{ marginBottom: 16 }}>
          <div style={{ fontSize: 12, fontWeight: 700, color: "var(--text-2)", textTransform: "uppercase", marginBottom: 8 }}>
            🖼️ Image &amp; Scan Previews ({imageAttachments.length})
          </div>
          <div style={{ display: "grid", gridTemplateColumns: "repeat(auto-fill, minmax(130px, 1fr))", gap: 10 }}>
            {imageAttachments.map((att) => (
              <EmailImageCard 
                key={att.id} 
                attachment={att} 
                onZoom={(src) => setZoomImage({ src, filename: att.filename })}
                onExport={() => exportSingle(att.id)}
              />
            ))}
          </div>
        </div>
      )}

      <table style={{ width: "100%", borderCollapse: "collapse", textAlign: "left" }}>
        <thead>
          <tr>
            <th className="th">Filename</th>
            <th className="th">Type</th>
            <th className="th" style={{ width: 80 }}>Size</th>
            <th className="th" style={{ width: 140 }}>SHA-256</th>
            <th className="th" style={{ width: 80, textAlign: "right" }}>Action</th>
          </tr>
        </thead>
        <tbody>
          {attachments.map((att) => (
            <tr key={att.id}>
              <td className="td">
                <div style={{ fontWeight: 600, color: "var(--text-0)" }}>
                  {att.filename || <span className="muted">unnamed</span>}
                </div>
              </td>
              <td className="td">
                <span className="badge badge-blue">{att.mime_type || "application/octet-stream"}</span>
              </td>
              <td className="td mono" style={{ fontSize: 11 }}>
                {formatBytes(att.size_bytes)}
              </td>
              <td className="td mono" style={{ fontSize: 10, color: "var(--accent)" }}>
                {att.sha256?.slice(0, 14)}…
              </td>
              <td className="td" style={{ textAlign: "right" }}>
                <button
                  className="btn btn-ghost btn-sm"
                  style={{ padding: "2px 8px", fontSize: 11 }}
                  onClick={() => exportSingle(att.id)}
                >
                  📥 Export
                </button>
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}

function EmailImageCard({ 
  attachment, 
  onZoom, 
  onExport 
}: { 
  attachment: any; 
  onZoom: (src: string) => void; 
  onExport: () => void; 
}) {
  const [src, setSrc] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    invoke<string | null>("get_attachment_preview", { input: { attachment_id: attachment.id } })
      .then((data) => {
        if (data) setSrc(data);
      })
      .catch(console.error)
      .finally(() => setLoading(false));
  }, [attachment.id]);

  return (
    <div
      style={{
        background: "var(--bg-2)",
        border: "1px solid var(--border)",
        borderRadius: "var(--r-sm)",
        padding: 6,
        display: "flex",
        flexDirection: "column",
      }}
    >
      <div
        style={{
          width: "100%",
          height: 90,
          background: "var(--bg-0)",
          borderRadius: 4,
          overflow: "hidden",
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
          cursor: src ? "zoom-in" : "default",
        }}
        onClick={() => { if (src) onZoom(src); }}
      >
        {loading ? (
          <span style={{ fontSize: 10, color: "var(--text-3)" }}>Loading...</span>
        ) : src ? (
          <img src={src} alt={attachment.filename} style={{ width: "100%", height: "100%", objectFit: "cover" }} />
        ) : (
          <span style={{ fontSize: 24 }}>🖼️</span>
        )}
      </div>
      <div
        style={{
          fontSize: 11,
          fontWeight: 600,
          color: "var(--text-0)",
          marginTop: 4,
          overflow: "hidden",
          textOverflow: "ellipsis",
          whiteSpace: "nowrap",
        }}
        title={attachment.filename}
      >
        {attachment.filename}
      </div>
      <div className="row between mt-1">
        <span style={{ fontSize: 10, color: "var(--text-3)" }}>{formatBytes(attachment.size_bytes)}</span>
        <button
          className="btn btn-ghost btn-sm"
          style={{ padding: "1px 4px", fontSize: 9 }}
          onClick={(e) => { e.stopPropagation(); onExport(); }}
        >
          📥
        </button>
      </div>
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