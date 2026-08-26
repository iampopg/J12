import { useState, useEffect, useCallback, useMemo } from "react";
import { invoke } from "@tauri-apps/api/core";
import { EmailDetailModal, EmailModalData } from "../components/EmailDetailModal";
import { BookmarkButton } from "../components/BookmarkButton";

export interface ItemBookmark {
  id: string;
  case_id: string;
  item_id: string;
  item_type: "email" | "attachment" | "finding" | "artifact";
  label: string;
  color: string;
  note: string;
  created_at: string;
  item_title?: string | null;
  item_from?: string | null;
  item_date?: string | null;
}

interface Props {
  caseId: string;
  onNavigate?: (view: string, filter?: string) => void;
}

export function EvidenceLockerView({ caseId, onNavigate }: Props) {
  const [bookmarks, setBookmarks] = useState<ItemBookmark[]>([]);
  const [loading, setLoading] = useState(true);
  const [search, setSearch] = useState("");
  const [typeFilter, setTypeFilter] = useState<string>("all");
  const [tagFilter, setTagFilter] = useState<string>("all");
  const [viewMode, setViewMode] = useState<"grid" | "table">("grid");
  const [sortBy, setSortBy] = useState<"newest" | "oldest" | "label" | "title">("newest");

  // Detail Modal for viewing email
  const [activeEmail, setActiveEmail] = useState<EmailModalData | null>(null);
  const [loadingEmail, setLoadingEmail] = useState(false);
  const [statusMsg, setStatusMsg] = useState<string | null>(null);

  const loadBookmarks = useCallback(async () => {
    if (!caseId) return;
    setLoading(true);
    try {
      const data = await invoke<ItemBookmark[]>("bookmarks_list", { input: { case_id: caseId } });
      setBookmarks(data || []);
    } catch (e) {
      console.error("Failed to load bookmarks:", e);
    } finally {
      setLoading(false);
    }
  }, [caseId]);

  useEffect(() => {
    loadBookmarks();
  }, [loadBookmarks]);

  // Flash status message
  const showToast = (msg: string) => {
    setStatusMsg(msg);
    setTimeout(() => setStatusMsg(null), 3000);
  };

  // Open item action
  const handleOpenItem = async (b: ItemBookmark) => {
    if (b.item_type === "email") {
      setLoadingEmail(true);
      try {
        const fullEmail = await invoke<EmailModalData | null>("email_get", { input: { id: b.item_id } });
        if (fullEmail) {
          setActiveEmail(fullEmail);
        } else {
          showToast("⚠️ Email data not found in case database.");
        }
      } catch (e) {
        console.error("Failed to load email:", e);
        showToast("❌ Error loading email details.");
      } finally {
        setLoadingEmail(false);
      }
    } else if (b.item_type === "attachment") {
      try {
        const res = await invoke<string>("open_attachment_in_system", { input: { id: b.item_id } });
        showToast(`📂 ${res || "Attachment opened in system viewer"}`);
      } catch (e) {
        showToast(`❌ Failed to open attachment: ${e}`);
      }
    } else if (b.item_type === "artifact") {
      if (onNavigate) {
        onNavigate("artifacts");
      } else {
        showToast(`🧩 Artifact ID: ${b.item_id}`);
      }
    } else if (b.item_type === "finding") {
      if (onNavigate) {
        onNavigate("findings");
      } else {
        showToast(`🎯 Finding ID: ${b.item_id}`);
      }
    }
  };

  const handleRevealAttachment = async (attachmentId: string) => {
    try {
      const res = await invoke<string>("reveal_in_finder", { input: { id: attachmentId } });
      showToast(`📁 ${res || "Revealed in file manager"}`);
    } catch (e) {
      showToast(`❌ ${e}`);
    }
  };

  const handleExportJson = () => {
    const dataStr = "data:text/json;charset=utf-8," + encodeURIComponent(JSON.stringify(bookmarks, null, 2));
    const downloadAnchor = document.createElement("a");
    downloadAnchor.setAttribute("href", dataStr);
    downloadAnchor.setAttribute("download", `case_${caseId}_evidence_locker.json`);
    document.body.appendChild(downloadAnchor);
    downloadAnchor.click();
    downloadAnchor.remove();
    showToast("📁 Evidence Locker exported as JSON");
  };

  // Counts & tag stats
  const stats = useMemo(() => {
    const total = bookmarks.length;
    const emails = bookmarks.filter((b) => b.item_type === "email").length;
    const attachments = bookmarks.filter((b) => b.item_type === "attachment").length;
    const artifacts = bookmarks.filter((b) => b.item_type === "artifact").length;
    const findings = bookmarks.filter((b) => b.item_type === "finding").length;
    const withNotes = bookmarks.filter((b) => b.note && b.note.trim().length > 0).length;

    // Unique tags
    const tagMap = new Map<string, { label: string; color: string; count: number }>();
    for (const b of bookmarks) {
      const key = b.label.toLowerCase();
      if (!tagMap.has(key)) {
        tagMap.set(key, { label: b.label, color: b.color, count: 1 });
      } else {
        tagMap.get(key)!.count += 1;
      }
    }
    const tags = Array.from(tagMap.values()).sort((a, b) => b.count - a.count);

    return { total, emails, attachments, artifacts, findings, withNotes, tags };
  }, [bookmarks]);

  // Filtered & sorted bookmarks
  const filteredBookmarks = useMemo(() => {
    let result = [...bookmarks];

    // Type filter
    if (typeFilter !== "all") {
      result = result.filter((b) => b.item_type === typeFilter);
    }

    // Tag filter
    if (tagFilter !== "all") {
      result = result.filter((b) => b.label.toLowerCase() === tagFilter.toLowerCase());
    }

    // Search query
    if (search.trim()) {
      const q = search.toLowerCase();
      result = result.filter(
        (b) =>
          b.label.toLowerCase().includes(q) ||
          (b.note && b.note.toLowerCase().includes(q)) ||
          (b.item_title && b.item_title.toLowerCase().includes(q)) ||
          (b.item_from && b.item_from.toLowerCase().includes(q)) ||
          b.item_type.toLowerCase().includes(q)
      );
    }

    // Sort
    result.sort((a, b) => {
      if (sortBy === "newest") return new Date(b.created_at).getTime() - new Date(a.created_at).getTime();
      if (sortBy === "oldest") return new Date(a.created_at).getTime() - new Date(b.created_at).getTime();
      if (sortBy === "label") return a.label.localeCompare(b.label);
      if (sortBy === "title") return (a.item_title || "").localeCompare(b.item_title || "");
      return 0;
    });

    return result;
  }, [bookmarks, typeFilter, tagFilter, search, sortBy]);

  const getItemTypeBadge = (type: string) => {
    switch (type) {
      case "email":
        return { label: "EMAIL", icon: "✉️", color: "var(--accent-blue)" };
      case "attachment":
        return { label: "ATTACHMENT", icon: "📎", color: "var(--accent-green)" };
      case "artifact":
        return { label: "ARTIFACT", icon: "🧩", color: "#8b5cf6" };
      case "finding":
        return { label: "FINDING", icon: "🎯", color: "var(--accent-amber)" };
      default:
        return { label: type.toUpperCase(), icon: "📄", color: "var(--text-2)" };
    }
  };

  return (
    <div className="view-content" style={{ display: "flex", flexDirection: "column", height: "100%", gap: 16 }}>
      {/* Toast message */}
      {statusMsg && (
        <div
          style={{
            position: "fixed",
            bottom: 24,
            right: 24,
            background: "var(--bg-2)",
            color: "var(--text-0)",
            padding: "10px 18px",
            borderRadius: "var(--r-md)",
            border: "1px solid var(--border)",
            boxShadow: "0 10px 30px rgba(0,0,0,0.5)",
            zIndex: 99999,
            fontSize: 13,
            fontWeight: 600,
            animation: "fadeIn 0.2s ease-out",
          }}
        >
          {statusMsg}
        </div>
      )}

      {/* Header */}
      <div className="row between" style={{ flexWrap: "wrap", gap: 12 }}>
        <div>
          <h2 style={{ fontSize: 22, fontWeight: 700, color: "var(--text-0)" }}>
            🔖 Evidence Locker &amp; Investigator Notes
          </h2>
          <p className="muted" style={{ fontSize: 12 }}>
            Central repository of all tagged evidence items, critical artifacts, findings, key communications, and forensic notes.
          </p>
        </div>

        {/* Action Buttons */}
        <div className="row gap-2">
          <button className="btn btn-secondary btn-sm" onClick={handleExportJson} disabled={bookmarks.length === 0}>
            📤 Export Locker (JSON)
          </button>
          <button className="btn btn-ghost btn-sm" onClick={loadBookmarks}>
            ↻ Refresh
          </button>
        </div>
      </div>

      {/* Stats Summary Cards */}
      <div
        style={{
          display: "grid",
          gridTemplateColumns: "repeat(auto-fit, minmax(160px, 1fr))",
          gap: 12,
        }}
      >
        <div className="card" style={{ padding: "14px 18px", borderLeft: "4px solid var(--accent-blue)" }}>
          <div style={{ fontSize: 11, fontWeight: 700, color: "var(--text-2)", textTransform: "uppercase" }}>
            Total Evidence Items
          </div>
          <div style={{ fontSize: 24, fontWeight: 800, color: "var(--text-0)", marginTop: 4 }}>
            {stats.total.toLocaleString()}
          </div>
          <div style={{ fontSize: 11, color: "var(--text-2)", marginTop: 2 }}>All tagged records</div>
        </div>

        <div className="card" style={{ padding: "14px 18px", borderLeft: "4px solid #3b82f6" }}>
          <div style={{ fontSize: 11, fontWeight: 700, color: "var(--text-2)", textTransform: "uppercase" }}>
            ✉️ Tagged Emails
          </div>
          <div style={{ fontSize: 24, fontWeight: 800, color: "var(--text-0)", marginTop: 4 }}>
            {stats.emails.toLocaleString()}
          </div>
          <div style={{ fontSize: 11, color: "var(--text-2)", marginTop: 2 }}>Key correspondence</div>
        </div>

        <div className="card" style={{ padding: "14px 18px", borderLeft: "4px solid #10b981" }}>
          <div style={{ fontSize: 11, fontWeight: 700, color: "var(--text-2)", textTransform: "uppercase" }}>
            📎 Tagged Attachments
          </div>
          <div style={{ fontSize: 24, fontWeight: 800, color: "var(--text-0)", marginTop: 4 }}>
            {stats.attachments.toLocaleString()}
          </div>
          <div style={{ fontSize: 11, color: "var(--text-2)", marginTop: 2 }}>Files &amp; images</div>
        </div>

        <div className="card" style={{ padding: "14px 18px", borderLeft: "4px solid #8b5cf6" }}>
          <div style={{ fontSize: 11, fontWeight: 700, color: "var(--text-2)", textTransform: "uppercase" }}>
            🧩 Tagged Artifacts
          </div>
          <div style={{ fontSize: 24, fontWeight: 800, color: "var(--text-0)", marginTop: 4 }}>
            {stats.artifacts.toLocaleString()}
          </div>
          <div style={{ fontSize: 11, color: "var(--text-2)", marginTop: 2 }}>Credentials &amp; forensics</div>
        </div>

        <div className="card" style={{ padding: "14px 18px", borderLeft: "4px solid #f59e0b" }}>
          <div style={{ fontSize: 11, fontWeight: 700, color: "var(--text-2)", textTransform: "uppercase" }}>
            🎯 Tagged Findings
          </div>
          <div style={{ fontSize: 24, fontWeight: 800, color: "var(--text-0)", marginTop: 4 }}>
            {stats.findings.toLocaleString()}
          </div>
          <div style={{ fontSize: 11, color: "var(--text-2)", marginTop: 2 }}>Forensic observations</div>
        </div>

        <div className="card" style={{ padding: "14px 18px", borderLeft: "4px solid #ec4899" }}>
          <div style={{ fontSize: 11, fontWeight: 700, color: "var(--text-2)", textTransform: "uppercase" }}>
            📝 With Notes
          </div>
          <div style={{ fontSize: 24, fontWeight: 800, color: "var(--text-0)", marginTop: 4 }}>
            {stats.withNotes.toLocaleString()}
          </div>
          <div style={{ fontSize: 11, color: "var(--text-2)", marginTop: 2 }}>Annotated items</div>
        </div>
      </div>

      {/* Filter & Search Bar */}
      <div
        className="card"
        style={{
          padding: 14,
          display: "flex",
          flexDirection: "column",
          gap: 12,
        }}
      >
        <div style={{ display: "flex", gap: 12, flexWrap: "wrap", alignItems: "center", justifyContent: "space-between" }}>
          {/* Search box */}
          <div style={{ flex: 1, minWidth: 260, position: "relative" }}>
            <input
              className="input"
              style={{ width: "100%", paddingLeft: 34, fontSize: 13 }}
              placeholder="Search by label, investigator note, subject, sender, filename..."
              value={search}
              onChange={(e) => setSearch(e.target.value)}
            />
            <span style={{ position: "absolute", left: 12, top: "50%", transform: "translateY(-50%)", fontSize: 14, color: "var(--text-2)" }}>
              🔍
            </span>
            {search && (
              <button
                style={{
                  position: "absolute",
                  right: 10,
                  top: "50%",
                  transform: "translateY(-50%)",
                  background: "transparent",
                  border: "none",
                  color: "var(--text-2)",
                  cursor: "pointer",
                  fontSize: 12,
                }}
                onClick={() => setSearch("")}
              >
                ✕
              </button>
            )}
          </div>

          {/* Sort & View Mode */}
          <div style={{ display: "flex", gap: 10, alignItems: "center" }}>
            <div className="row gap-1" style={{ alignItems: "center" }}>
              <span style={{ fontSize: 12, color: "var(--text-2)" }}>Sort:</span>
              <select
                className="input input-sm"
                value={sortBy}
                onChange={(e) => setSortBy(e.target.value as any)}
                style={{ fontSize: 12, padding: "4px 8px" }}
              >
                <option value="newest">Newest First</option>
                <option value="oldest">Oldest First</option>
                <option value="label">Tag Label</option>
                <option value="title">Item Title</option>
              </select>
            </div>

            <div className="row gap-1" style={{ background: "var(--bg-2)", padding: 2, borderRadius: "var(--r-sm)", border: "1px solid var(--border)" }}>
              <button
                className={`btn btn-sm ${viewMode === "grid" ? "btn-primary" : "btn-ghost"}`}
                style={{ padding: "3px 8px", fontSize: 12 }}
                onClick={() => setViewMode("grid")}
                title="Grid / Card View"
              >
                🔲 Grid
              </button>
              <button
                className={`btn btn-sm ${viewMode === "table" ? "btn-primary" : "btn-ghost"}`}
                style={{ padding: "3px 8px", fontSize: 12 }}
                onClick={() => setViewMode("table")}
                title="Table View"
              >
                📋 Table
              </button>
            </div>
          </div>
        </div>

        {/* Type & Tag Filter Pills */}
        <div style={{ display: "flex", flexDirection: "column", gap: 8, paddingTop: 6, borderTop: "1px solid var(--border)" }}>
          {/* Item Type Filters */}
          <div style={{ display: "flex", gap: 6, flexWrap: "wrap", alignItems: "center" }}>
            <span style={{ fontSize: 11, fontWeight: 700, color: "var(--text-2)", textTransform: "uppercase", marginRight: 4 }}>
              Type:
            </span>
            {[
              { id: "all", label: "All Items", count: stats.total },
              { id: "email", label: "✉️ Emails", count: stats.emails },
              { id: "attachment", label: "📎 Attachments", count: stats.attachments },
              { id: "artifact", label: "🧩 Artifacts", count: stats.artifacts },
              { id: "finding", label: "🎯 Findings", count: stats.findings },
            ].map((t) => (
              <button
                key={t.id}
                className={`btn btn-sm ${typeFilter === t.id ? "btn-primary" : "btn-ghost"}`}
                style={{ fontSize: 11, padding: "2px 8px", borderRadius: 999 }}
                onClick={() => setTypeFilter(t.id)}
              >
                {t.label} <span style={{ opacity: 0.7, marginLeft: 4 }}>({t.count})</span>
              </button>
            ))}
          </div>

          {/* Tag / Label Filters */}
          {stats.tags.length > 0 && (
            <div style={{ display: "flex", gap: 6, flexWrap: "wrap", alignItems: "center" }}>
              <span style={{ fontSize: 11, fontWeight: 700, color: "var(--text-2)", textTransform: "uppercase", marginRight: 4 }}>
                Tag:
              </span>
              <button
                className={`btn btn-sm ${tagFilter === "all" ? "btn-primary" : "btn-ghost"}`}
                style={{ fontSize: 11, padding: "2px 8px", borderRadius: 999 }}
                onClick={() => setTagFilter("all")}
              >
                All Tags
              </button>
              {stats.tags.map((t) => (
                <button
                  key={t.label}
                  onClick={() => setTagFilter(tagFilter === t.label ? "all" : t.label)}
                  style={{
                    fontSize: 11,
                    padding: "2px 10px",
                    borderRadius: 999,
                    background: tagFilter === t.label ? t.color : `${t.color}22`,
                    color: tagFilter === t.label ? "#ffffff" : t.color,
                    border: `1px solid ${t.color}`,
                    cursor: "pointer",
                    fontWeight: 600,
                    display: "flex",
                    alignItems: "center",
                    gap: 5,
                  }}
                >
                  <span style={{ width: 6, height: 6, borderRadius: "50%", background: tagFilter === t.label ? "#ffffff" : t.color }} />
                  {t.label} ({t.count})
                </button>
              ))}
            </div>
          )}
        </div>
      </div>

      {/* Main Content Area */}
      {loading ? (
        <div className="card" style={{ padding: 48, textAlign: "center", color: "var(--text-2)" }}>
          <span style={{ fontSize: 24 }}>⏳</span>
          <div style={{ marginTop: 12, fontSize: 14 }}>Loading evidence locker items...</div>
        </div>
      ) : filteredBookmarks.length === 0 ? (
        <div
          className="card"
          style={{
            padding: 48,
            textAlign: "center",
            background: "var(--bg-1)",
            border: "1px dashed var(--border)",
            borderRadius: "var(--r-lg)",
          }}
        >
          <span style={{ fontSize: 36 }}>🔖</span>
          <h3 style={{ fontSize: 16, fontWeight: 700, color: "var(--text-0)", marginTop: 12 }}>
            {bookmarks.length === 0 ? "No Items in Evidence Locker Yet" : "No Matching Tagged Evidence Found"}
          </h3>
          <p style={{ fontSize: 13, color: "var(--text-2)", maxWidth: 500, margin: "8px auto 20px auto", lineHeight: 1.5 }}>
            {bookmarks.length === 0
              ? "You can tag and bookmark any email, attachment, or finding across the case by clicking the '🔖 Tag' button in each view. Add custom labels and notes to build your case evidence binder."
              : "Try adjusting your search keywords, item type filter, or tag selection."}
          </p>
          {bookmarks.length > 0 && (
            <button
              className="btn btn-secondary btn-sm"
              onClick={() => {
                setSearch("");
                setTypeFilter("all");
                setTagFilter("all");
              }}
            >
              Reset Filters
            </button>
          )}
        </div>
      ) : viewMode === "grid" ? (
        /* GRID / CARD VIEW */
        <div
          style={{
            display: "grid",
            gridTemplateColumns: "repeat(auto-fill, minmax(340px, 1fr))",
            gap: 16,
            overflowY: "auto",
            paddingRight: 4,
          }}
        >
          {filteredBookmarks.map((b) => {
            const badge = getItemTypeBadge(b.item_type);
            return (
              <div
                key={b.id}
                className="card"
                style={{
                  padding: 16,
                  display: "flex",
                  flexDirection: "column",
                  justifyContent: "space-between",
                  gap: 12,
                  border: `1px solid var(--border)`,
                  borderTop: `3px solid ${b.color || "var(--accent-blue)"}`,
                  position: "relative",
                  transition: "transform 0.15s ease, box-shadow 0.15s ease",
                }}
              >
                <div>
                  {/* Top Bar: Item Type & Tag Pill */}
                  <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: 10 }}>
                    <span
                      style={{
                        fontSize: 10,
                        fontWeight: 800,
                        color: badge.color,
                        background: `${badge.color}15`,
                        padding: "2px 8px",
                        borderRadius: 4,
                        border: `1px solid ${badge.color}33`,
                        display: "flex",
                        alignItems: "center",
                        gap: 4,
                      }}
                    >
                      <span>{badge.icon}</span> {badge.label}
                    </span>

                    {/* Tag pill */}
                    <span
                      style={{
                        fontSize: 11,
                        fontWeight: 700,
                        color: b.color,
                        background: `${b.color}20`,
                        border: `1px solid ${b.color}50`,
                        padding: "2px 8px",
                        borderRadius: 999,
                        display: "flex",
                        alignItems: "center",
                        gap: 4,
                      }}
                    >
                      🔖 {b.label}
                    </span>
                  </div>

                  {/* Title / Subject */}
                  <div
                    style={{
                      fontSize: 14,
                      fontWeight: 700,
                      color: "var(--text-0)",
                      marginBottom: 6,
                      wordBreak: "break-word",
                      lineHeight: 1.4,
                    }}
                  >
                    {b.item_title || (b.item_type === "email" ? "(No Subject)" : `Item ${b.item_id.slice(0, 8)}...`)}
                  </div>

                  {/* From & Date meta */}
                  {(b.item_from || b.item_date) && (
                    <div style={{ fontSize: 11, color: "var(--text-2)", marginBottom: 10 }}>
                      {b.item_from && <div>From: <strong style={{ color: "var(--text-1)" }}>{b.item_from}</strong></div>}
                      {b.item_date && <div>Date: {new Date(b.item_date).toLocaleString()}</div>}
                    </div>
                  )}

                  {/* Investigator Note (if present) */}
                  {b.note && b.note.trim().length > 0 && (
                    <div
                      style={{
                        background: "var(--bg-2)",
                        borderLeft: `3px solid ${b.color || "var(--accent-blue)"}`,
                        borderRadius: "0 var(--r-sm) var(--r-sm) 0",
                        padding: "8px 12px",
                        fontSize: 12,
                        color: "var(--text-1)",
                        marginBottom: 10,
                        whiteSpace: "pre-wrap",
                        lineHeight: 1.4,
                      }}
                    >
                      <div style={{ fontSize: 10, fontWeight: 700, color: "var(--text-2)", textTransform: "uppercase", marginBottom: 3 }}>
                        📝 Investigator Note:
                      </div>
                      {b.note}
                    </div>
                  )}
                </div>

                {/* Footer Actions */}
                <div
                  style={{
                    display: "flex",
                    justifyContent: "space-between",
                    alignItems: "center",
                    paddingTop: 10,
                    borderTop: "1px solid var(--border)",
                    gap: 8,
                  }}
                >
                  <span style={{ fontSize: 10, color: "var(--text-2)" }}>
                    Tagged {new Date(b.created_at).toLocaleDateString()}
                  </span>

                  <div style={{ display: "flex", gap: 6, alignItems: "center" }}>
                    <BookmarkButton
                      caseId={caseId}
                      itemId={b.item_id}
                      itemType={b.item_type}
                      compact={true}
                      onChanged={() => loadBookmarks()}
                    />

                    {b.item_type === "attachment" && (
                      <button
                        className="btn btn-ghost btn-sm"
                        style={{ fontSize: 11, padding: "3px 6px" }}
                        onClick={() => handleRevealAttachment(b.item_id)}
                        title="Reveal in Finder / Explorer"
                      >
                        📁
                      </button>
                    )}

                    <button
                      className="btn btn-secondary btn-sm"
                      style={{ fontSize: 11, padding: "3px 10px", fontWeight: 600 }}
                      onClick={() => handleOpenItem(b)}
                    >
                      👁️ View {badge.label}
                    </button>
                  </div>
                </div>
              </div>
            );
          })}
        </div>
      ) : (
        /* TABLE VIEW */
        <div className="card" style={{ padding: 0, overflow: "hidden", flex: 1, display: "flex", flexDirection: "column" }}>
          <div style={{ overflowX: "auto", flex: 1 }}>
            <table className="table" style={{ width: "100%", fontSize: 12 }}>
              <thead>
                <tr>
                  <th style={{ width: 100 }}>Type</th>
                  <th style={{ width: 140 }}>Tag Label</th>
                  <th>Title / Subject</th>
                  <th>Sender / Source</th>
                  <th>Investigator Note</th>
                  <th style={{ width: 130 }}>Date Added</th>
                  <th style={{ width: 130, textAlign: "right" }}>Actions</th>
                </tr>
              </thead>
              <tbody>
                {filteredBookmarks.map((b) => {
                  const badge = getItemTypeBadge(b.item_type);
                  return (
                    <tr key={b.id}>
                      <td>
                        <span
                          style={{
                            fontSize: 10,
                            fontWeight: 800,
                            color: badge.color,
                            background: `${badge.color}15`,
                            padding: "2px 6px",
                            borderRadius: 4,
                            border: `1px solid ${badge.color}33`,
                            display: "inline-flex",
                            alignItems: "center",
                            gap: 3,
                          }}
                        >
                          {badge.icon} {badge.label}
                        </span>
                      </td>
                      <td>
                        <span
                          style={{
                            fontSize: 11,
                            fontWeight: 700,
                            color: b.color,
                            background: `${b.color}20`,
                            border: `1px solid ${b.color}50`,
                            padding: "2px 8px",
                            borderRadius: 999,
                            display: "inline-flex",
                            alignItems: "center",
                            gap: 4,
                          }}
                        >
                          🔖 {b.label}
                        </span>
                      </td>
                      <td>
                        <strong style={{ color: "var(--text-0)" }}>
                          {b.item_title || (b.item_type === "email" ? "(No Subject)" : b.item_id.slice(0, 12))}
                        </strong>
                      </td>
                      <td style={{ color: "var(--text-1)" }}>
                        {b.item_from || "-"}
                      </td>
                      <td>
                        {b.note ? (
                          <span style={{ color: "var(--text-1)", fontStyle: "italic" }}>
                            "{b.note.length > 50 ? b.note.slice(0, 50) + "..." : b.note}"
                          </span>
                        ) : (
                          <span style={{ color: "var(--text-2)" }}>-</span>
                        )}
                      </td>
                      <td style={{ color: "var(--text-2)", fontSize: 11 }}>
                        {new Date(b.created_at).toLocaleString()}
                      </td>
                      <td style={{ textAlign: "right" }}>
                        <div style={{ display: "inline-flex", gap: 4 }}>
                          <BookmarkButton
                            caseId={caseId}
                            itemId={b.item_id}
                            itemType={b.item_type}
                            compact={true}
                            onChanged={() => loadBookmarks()}
                          />
                          <button
                            className="btn btn-secondary btn-sm"
                            style={{ fontSize: 11, padding: "2px 8px" }}
                            onClick={() => handleOpenItem(b)}
                          >
                            Open
                          </button>
                        </div>
                      </td>
                    </tr>
                  );
                })}
              </tbody>
            </table>
          </div>
        </div>
      )}

      {/* Email Detail Modal */}
      {activeEmail && (
        <EmailDetailModal
          email={activeEmail}
          onClose={() => setActiveEmail(null)}
          titleSuffix="Back to Evidence Locker"
        />
      )}
    </div>
  );
}
