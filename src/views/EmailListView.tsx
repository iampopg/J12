import { useState, useMemo, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  Email,
  ColumnSettings,
  DEFAULT_COLUMNS,
  EmailTag,
  Evidence,
  SortField,
  SortDir,
  cleanDisplayName,
} from "./email_list/types";
import { EmailColumnPicker } from "./email_list/EmailColumnPicker";
import { EmailFiltersBar } from "./email_list/EmailFiltersBar";
import { EmailAdvancedFilterDrawer } from "./email_list/EmailAdvancedFilterDrawer";
import { VirtualEmailList } from "./email_list/VirtualEmailList";
import { EmailDetail } from "./email_list/EmailDetail";

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

  const [dateFilterMode, setDateFilterMode] = useState<"all" | "single" | "range">("all");
  const [singleDate, setSingleDate] = useState<string>("");
  const [startDate, setStartDate] = useState<string>("");
  const [endDate, setEndDate] = useState<string>("");
  const [showFilterDrawer, setShowFilterDrawer] = useState(false);

  const [columns, setColumns] = useState<ColumnSettings>(() => {
    try {
      const saved = localStorage.getItem("j12_email_columns");
      if (saved) return { ...DEFAULT_COLUMNS, ...JSON.parse(saved) };
    } catch {}
    return DEFAULT_COLUMNS;
  });
  const [showColumnPicker, setShowColumnPicker] = useState(false);

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

  const uniqueEmails = useMemo(() => {
    if (!showUnique) return emails;
    const seen = new Set<string>();
    return emails.filter((e) => {
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

  const filtered = useMemo(() => {
    let result = filteredByFolder;

    if (tagFilter !== "all") {
      result = result.filter((e) => {
        const emailTags = tagsByEmail.get(e.id) || [];
        return emailTags.some((t) => t.tag.toLowerCase() === tagFilter.toLowerCase());
      });
    }

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

  const hasActiveFilters = Boolean(
    q ||
    tagFilter !== "all" ||
    dateFilterMode !== "all" ||
    singleDate ||
    startDate ||
    endDate ||
    showUnique
  );

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

              <div style={{ position: "relative" }}>
                <button
                  className={`btn btn-sm ${showColumnPicker ? "btn-primary" : "btn-ghost"}`}
                  onClick={() => setShowColumnPicker(!showColumnPicker)}
                  title="Customize table columns"
                >
                  ⚙️ Columns
                </button>

                <EmailColumnPicker
                  show={showColumnPicker}
                  onClose={() => setShowColumnPicker(false)}
                  columns={columns}
                  onToggleColumn={toggleColumn}
                  onResetColumns={resetColumns}
                />
              </div>

              <button
                className={`btn btn-sm ${showFilterDrawer ? "btn-primary" : "btn-ghost"}`}
                onClick={() => setShowFilterDrawer(!showFilterDrawer)}
              >
                📅 Date &amp; Sort Filters {hasActiveFilters && "●"}
              </button>
              <button className="btn btn-ghost btn-sm" onClick={load}>
                ↻ Refresh
              </button>
            </div>
          </div>

          <EmailFiltersBar
            evidence={evidence}
            selectedEvidenceId={selectedEvidenceId}
            onSelectEvidence={(id) => {
              setSelectedEvidenceId(id);
              onEvidenceFilterChange?.(id);
            }}
            evidenceCounts={evidenceCounts}
            totalEmailsCount={emails.length}
            q={q}
            setQ={setQ}
            sortField={sortField}
            sortDir={sortDir}
            onSortChange={(field, dir) => {
              setSortField(field);
              setSortDir(dir);
            }}
            hasActiveFilters={hasActiveFilters}
            onResetFilters={handleResetFilters}
          />

          <EmailAdvancedFilterDrawer
            show={showFilterDrawer}
            dateFilterMode={dateFilterMode}
            setDateFilterMode={setDateFilterMode}
            singleDate={singleDate}
            setSingleDate={setSingleDate}
            startDate={startDate}
            setStartDate={setStartDate}
            endDate={endDate}
            setEndDate={setEndDate}
            tagFilter={tagFilter}
            setTagFilter={setTagFilter}
            uniqueTags={uniqueTags}
            onResetFilters={handleResetFilters}
          />

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