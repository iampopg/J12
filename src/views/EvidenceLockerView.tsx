import { useState, useEffect, useCallback, useMemo } from "react";
import { invoke } from "@tauri-apps/api/core";
import { EmailDetailModal, EmailModalData } from "../components/EmailDetailModal";
import {
  ItemBookmark,
  EvidenceLockerProps,
} from "./evidence_locker/types";
import { LockerStatsCards } from "./evidence_locker/LockerStatsCards";
import { LockerFiltersBar } from "./evidence_locker/LockerFiltersBar";
import { LockerGridView } from "./evidence_locker/LockerGridView";
import { LockerTableView } from "./evidence_locker/LockerTableView";

export function EvidenceLockerView({ caseId, evidenceFilter, onNavigate }: EvidenceLockerProps) {
  const [bookmarks, setBookmarks] = useState<ItemBookmark[]>([]);
  const [loading, setLoading] = useState(true);
  const [search, setSearch] = useState("");
  const [typeFilter, setTypeFilter] = useState<string>("all");
  const [tagFilter, setTagFilter] = useState<string>("all");
  const [viewMode, setViewMode] = useState<"grid" | "table">("grid");
  const [sortBy, setSortBy] = useState<"newest" | "oldest" | "label" | "title">("newest");

  const [activeEmail, setActiveEmail] = useState<EmailModalData | null>(null);
  const [statusMsg, setStatusMsg] = useState<string | null>(null);

  const loadBookmarks = useCallback(async () => {
    if (!caseId) return;
    setLoading(true);
    try {
      const data = await invoke<ItemBookmark[]>("bookmarks_list", {
        input: { case_id: caseId, evidence_id: evidenceFilter || undefined },
      });
      setBookmarks(data || []);
    } catch (e) {
      console.error("Failed to load bookmarks:", e);
    } finally {
      setLoading(false);
    }
  }, [caseId, evidenceFilter]);

  useEffect(() => {
    loadBookmarks();
  }, [loadBookmarks]);

  const showToast = (msg: string) => {
    setStatusMsg(msg);
    setTimeout(() => setStatusMsg(null), 3000);
  };

  const handleOpenItem = async (b: ItemBookmark) => {
    if (b.item_type === "email") {
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

  const stats = useMemo(() => {
    const total = bookmarks.length;
    const emails = bookmarks.filter((b) => b.item_type === "email").length;
    const attachments = bookmarks.filter((b) => b.item_type === "attachment").length;
    const artifacts = bookmarks.filter((b) => b.item_type === "artifact").length;
    const findings = bookmarks.filter((b) => b.item_type === "finding").length;
    const withNotes = bookmarks.filter((b) => b.note && b.note.trim().length > 0).length;

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

  const filteredBookmarks = useMemo(() => {
    let result = [...bookmarks];

    if (typeFilter !== "all") {
      result = result.filter((b) => b.item_type === typeFilter);
    }

    if (tagFilter !== "all") {
      result = result.filter((b) => b.label.toLowerCase() === tagFilter.toLowerCase());
    }

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

    result.sort((a, b) => {
      if (sortBy === "newest") return new Date(b.created_at).getTime() - new Date(a.created_at).getTime();
      if (sortBy === "oldest") return new Date(a.created_at).getTime() - new Date(b.created_at).getTime();
      if (sortBy === "label") return a.label.localeCompare(b.label);
      if (sortBy === "title") return (a.item_title || "").localeCompare(b.item_title || "");
      return 0;
    });

    return result;
  }, [bookmarks, typeFilter, tagFilter, search, sortBy]);

  return (
    <div className="view-content" style={{ display: "flex", flexDirection: "column", height: "100%", gap: 16 }}>
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
      <LockerStatsCards stats={stats} />

      {/* Filter & Search Bar */}
      <LockerFiltersBar
        search={search}
        setSearch={setSearch}
        sortBy={sortBy}
        setSortBy={setSortBy}
        viewMode={viewMode}
        setViewMode={setViewMode}
        typeFilter={typeFilter}
        setTypeFilter={setTypeFilter}
        tagFilter={tagFilter}
        setTagFilter={setTagFilter}
        stats={stats}
      />

      {/* Main Content View */}
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
              ? "You can tag and bookmark any email, attachment, or finding across the case by clicking the '🔖 Tag' button in each view."
              : "Try adjusting your search keywords, item type filter, or tag selection."}
          </p>
        </div>
      ) : viewMode === "grid" ? (
        <LockerGridView
          caseId={caseId}
          bookmarks={filteredBookmarks}
          onOpenItem={handleOpenItem}
          onRevealAttachment={handleRevealAttachment}
          onRefresh={loadBookmarks}
        />
      ) : (
        <LockerTableView
          caseId={caseId}
          bookmarks={filteredBookmarks}
          onOpenItem={handleOpenItem}
          onRefresh={loadBookmarks}
        />
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
