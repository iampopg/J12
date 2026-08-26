import { useState, useEffect, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";

interface ItemBookmark {
  id: string;
  case_id: string;
  item_id: string;
  item_type: string;
  label: string;
  color: string;
  note: string;
  created_at: string;
}

interface Props {
  caseId: string;
  itemId: string;
  itemType: "email" | "attachment" | "finding" | "artifact";
  compact?: boolean; // just icon, no text
  align?: "left" | "right";
  onChanged?: (bookmark: ItemBookmark | null) => void;
}

const PRESET_LABELS = [
  { label: "Key Evidence",   color: "#ef4444" },
  { label: "Person of Interest", color: "#f97316" },
  { label: "Financial",      color: "#eab308" },
  { label: "Follow Up",      color: "#3b82f6" },
  { label: "Suspicious",     color: "#8b5cf6" },
  { label: "Cleared",        color: "#22c55e" },
  { label: "Bookmarked",     color: "#64748b" },
];

export function BookmarkButton({ caseId, itemId, itemType, compact = false, align = "right", onChanged }: Props) {
  const [bookmark, setBookmark] = useState<ItemBookmark | null>(null);
  const [open, setOpen] = useState(false);
  const [label, setLabel] = useState("Bookmarked");
  const [color, setColor] = useState("#3b82f6");
  const [note, setNote] = useState("");
  const [saving, setSaving] = useState(false);
  const popoverRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!caseId || !itemId) return;
    invoke<ItemBookmark | null>("bookmark_check", { input: { case_id: caseId, item_id: itemId } })
      .then(b => setBookmark(b))
      .catch(() => {});
  }, [caseId, itemId]);

  // Prefill form from existing bookmark
  useEffect(() => {
    if (bookmark) {
      setLabel(bookmark.label);
      setColor(bookmark.color);
      setNote(bookmark.note || "");
    } else {
      setLabel("Bookmarked");
      setColor("#3b82f6");
      setNote("");
    }
  }, [bookmark]);

  // Close on outside click
  useEffect(() => {
    if (!open) return;
    const handler = (e: MouseEvent) => {
      if (popoverRef.current && !popoverRef.current.contains(e.target as Node)) {
        setOpen(false);
      }
    };
    document.addEventListener("mousedown", handler);
    return () => document.removeEventListener("mousedown", handler);
  }, [open]);

  const save = async () => {
    setSaving(true);
    try {
      const b = await invoke<ItemBookmark>("bookmark_add", {
        input: { case_id: caseId, item_id: itemId, item_type: itemType, label, color, note },
      });
      setBookmark(b);
      onChanged?.(b);
      setOpen(false);
    } catch (e) {
      console.error(e);
    } finally {
      setSaving(false);
    }
  };

  const remove = async () => {
    setSaving(true);
    try {
      await invoke("bookmark_remove", { input: { case_id: caseId, item_id: itemId } });
      setBookmark(null);
      onChanged?.(null);
      setOpen(false);
    } catch (e) {
      console.error(e);
    } finally {
      setSaving(false);
    }
  };

  const isBookmarked = Boolean(bookmark);

  return (
    <div style={{ position: "relative", display: "inline-block", zIndex: open ? 1000 : undefined }}>
      <button
        className={`btn btn-sm ${isBookmarked ? "" : "btn-ghost"}`}
        style={{
          padding: compact ? "2px 5px" : "3px 8px",
          fontSize: 12,
          background: isBookmarked ? bookmark!.color + "22" : undefined,
          border: isBookmarked ? `1px solid ${bookmark!.color}55` : undefined,
          color: isBookmarked ? bookmark!.color : undefined,
          borderRadius: "var(--r-sm)",
          gap: 4,
          display: "flex",
          alignItems: "center",
        }}
        onClick={(e) => { e.stopPropagation(); setOpen(o => !o); }}
        title={isBookmarked ? `Tagged: ${bookmark!.label}` : "Add to Evidence Locker"}
      >
        <span style={{ fontSize: 13 }}>{isBookmarked ? "🔖" : "🏷️"}</span>
        {!compact && (
          <span>{isBookmarked ? bookmark!.label : "Tag"}</span>
        )}
      </button>

      {open && (
        <div
          ref={popoverRef}
          style={{
            position: "absolute",
            top: "calc(100% + 6px)",
            right: align === "right" ? 0 : "auto",
            left: align === "left" ? 0 : "auto",
            zIndex: 99999,
            background: "var(--bg-1)",
            border: "1px solid var(--border)",
            borderRadius: "var(--r-md)",
            boxShadow: "0 12px 32px rgba(0,0,0,0.6)",
            width: 270,
            maxWidth: "calc(100vw - 40px)",
            padding: 14,
          }}
          onClick={e => e.stopPropagation()}
        >
          <div style={{ fontSize: 12, fontWeight: 700, color: "var(--text-0)", marginBottom: 10 }}>
            🔖 Evidence Locker Tag
          </div>

          {/* Preset labels */}
          <div style={{ display: "flex", flexWrap: "wrap", gap: 5, marginBottom: 10 }}>
            {PRESET_LABELS.map(p => (
              <button
                key={p.label}
                onClick={() => { setLabel(p.label); setColor(p.color); }}
                style={{
                  padding: "2px 8px",
                  borderRadius: 999,
                  fontSize: 11,
                  background: label === p.label ? p.color + "33" : "var(--bg-2)",
                  border: `1px solid ${label === p.label ? p.color : "var(--border)"}`,
                  color: label === p.label ? p.color : "var(--text-1)",
                  cursor: "pointer",
                  fontWeight: label === p.label ? 700 : 400,
                }}
              >
                {p.label}
              </button>
            ))}
          </div>

          {/* Custom label */}
          <input
            className="input input-sm"
            placeholder="Custom label…"
            value={label}
            onChange={e => setLabel(e.target.value)}
            style={{ width: "100%", marginBottom: 8, fontSize: 12 }}
          />

          {/* Color picker */}
          <div className="row gap-2 mb-2" style={{ alignItems: "center" }}>
            <span style={{ fontSize: 11, color: "var(--text-2)" }}>Color:</span>
            {["#ef4444","#f97316","#eab308","#22c55e","#3b82f6","#8b5cf6","#64748b"].map(c => (
              <div
                key={c}
                onClick={() => setColor(c)}
                style={{
                  width: 18, height: 18, borderRadius: "50%",
                  background: c, cursor: "pointer",
                  border: color === c ? "2px solid white" : "2px solid transparent",
                  boxShadow: color === c ? `0 0 0 2px ${c}` : "none",
                }}
              />
            ))}
          </div>

          {/* Note */}
          <textarea
            className="input"
            placeholder="Add a note (optional)…"
            value={note}
            onChange={e => setNote(e.target.value)}
            rows={2}
            style={{ width: "100%", fontSize: 12, resize: "vertical", marginBottom: 10 }}
          />

          <div className="row gap-2">
            <button
              className="btn btn-primary btn-sm"
              style={{ flex: 1 }}
              disabled={saving || !label.trim()}
              onClick={save}
            >
              {saving ? "Saving…" : isBookmarked ? "Update Tag" : "🔖 Tag It"}
            </button>
            {isBookmarked && (
              <button
                className="btn btn-ghost btn-sm"
                style={{ color: "var(--danger)" }}
                disabled={saving}
                onClick={remove}
              >
                Remove
              </button>
            )}
            <button className="btn btn-ghost btn-sm" onClick={() => setOpen(false)}>Cancel</button>
          </div>
        </div>
      )}
    </div>
  );
}
