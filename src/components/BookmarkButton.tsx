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
  const [popoverPos, setPopoverPos] = useState<{ top: number; left: number }>({ top: 0, left: 0 });
  const buttonRef = useRef<HTMLButtonElement>(null);
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

  // Calculate fixed popup position when opened
  const toggleOpen = (e: React.MouseEvent) => {
    e.stopPropagation();
    if (!open && buttonRef.current) {
      const rect = buttonRef.current.getBoundingClientRect();
      const popoverWidth = 280;
      const popoverHeight = 310;
      
      let left = align === "left" ? rect.left : rect.right - popoverWidth;
      // Keep within viewport horizontal bounds
      if (left < 10) left = 10;
      if (left + popoverWidth > window.innerWidth - 10) {
        left = window.innerWidth - popoverWidth - 10;
      }

      let top = rect.bottom + 6;
      // If overflowing bottom of screen, flip upwards
      if (top + popoverHeight > window.innerHeight - 10) {
        top = Math.max(10, rect.top - popoverHeight - 6);
      }

      setPopoverPos({ top, left });
    }
    setOpen(o => !o);
  };

  // Close on outside click or escape
  useEffect(() => {
    if (!open) return;
    const handler = (e: MouseEvent) => {
      if (
        popoverRef.current && !popoverRef.current.contains(e.target as Node) &&
        buttonRef.current && !buttonRef.current.contains(e.target as Node)
      ) {
        setOpen(false);
      }
    };
    const keyHandler = (e: KeyboardEvent) => {
      if (e.key === "Escape") setOpen(false);
    };
    document.addEventListener("mousedown", handler);
    document.addEventListener("keydown", keyHandler);
    return () => {
      document.removeEventListener("mousedown", handler);
      document.removeEventListener("keydown", keyHandler);
    };
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
    <div style={{ display: "inline-flex", alignItems: "center" }}>
      <button
        ref={buttonRef}
        type="button"
        className={`btn btn-sm ${isBookmarked ? "" : "btn-ghost"}`}
        style={{
          padding: compact ? "2px 6px" : "3px 8px",
          fontSize: 11.5,
          fontWeight: 600,
          background: isBookmarked ? bookmark!.color + "25" : undefined,
          border: isBookmarked ? `1px solid ${bookmark!.color}66` : undefined,
          color: isBookmarked ? bookmark!.color : undefined,
          borderRadius: "var(--r-sm)",
          gap: 4,
          display: "flex",
          alignItems: "center",
          flexShrink: 0,
        }}
        onClick={toggleOpen}
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
            position: "fixed",
            top: popoverPos.top,
            left: popoverPos.left,
            zIndex: 999999,
            background: "var(--bg-1)",
            border: "1px solid var(--border)",
            borderRadius: "var(--r-md)",
            boxShadow: "0 16px 40px rgba(0,0,0,0.75)",
            width: 280,
            maxWidth: "calc(100vw - 20px)",
            padding: 14,
            animation: "fadeIn 0.12s ease-out",
          }}
          onClick={e => e.stopPropagation()}
        >
          <div className="row between mb-2" style={{ alignItems: "center" }}>
            <span style={{ fontSize: 12.5, fontWeight: 700, color: "var(--text-0)" }}>
              🔖 Evidence Locker Tag
            </span>
            <button
              className="btn btn-ghost btn-sm"
              style={{ padding: "1px 6px", fontSize: 10 }}
              onClick={() => setOpen(false)}
            >
              ✕
            </button>
          </div>

          {/* Preset labels */}
          <div style={{ display: "flex", flexWrap: "wrap", gap: 5, marginBottom: 10 }}>
            {PRESET_LABELS.map(p => (
              <button
                key={p.label}
                type="button"
                onClick={() => { setLabel(p.label); setColor(p.color); }}
                style={{
                  padding: "3px 8px",
                  borderRadius: 999,
                  fontSize: 11,
                  background: label === p.label ? p.color + "33" : "var(--bg-2)",
                  border: `1px solid ${label === p.label ? p.color : "var(--border)"}`,
                  color: label === p.label ? p.color : "var(--text-1)",
                  cursor: "pointer",
                  fontWeight: label === p.label ? 700 : 400,
                  transition: "all 0.15s ease",
                }}
              >
                {p.label}
              </button>
            ))}
          </div>

          {/* Custom label */}
          <div style={{ marginBottom: 8 }}>
            <div className="label" style={{ fontSize: 9.5, marginBottom: 3 }}>TAG LABEL</div>
            <input
              className="input input-sm"
              placeholder="Tag label (e.g. Hot Evidence)..."
              value={label}
              onChange={e => setLabel(e.target.value)}
              style={{ width: "100%", fontSize: 12, padding: "5px 8px" }}
            />
          </div>

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
                  transition: "transform 0.1s ease",
                }}
              />
            ))}
          </div>

          {/* Note */}
          <div style={{ marginBottom: 10 }}>
            <div className="label" style={{ fontSize: 9.5, marginBottom: 3 }}>EVIDENCE NOTE (OPTIONAL)</div>
            <textarea
              className="input"
              placeholder="Add investigator notes on why this is tagged..."
              value={note}
              onChange={e => setNote(e.target.value)}
              rows={2}
              style={{ width: "100%", fontSize: 11.5, resize: "vertical" }}
            />
          </div>

          <div className="row gap-2">
            <button
              type="button"
              className="btn btn-primary btn-sm"
              style={{ flex: 1, fontSize: 11.5 }}
              disabled={saving || !label.trim()}
              onClick={save}
            >
              {saving ? "Saving…" : isBookmarked ? "Update Tag" : "🔖 Tag Evidence"}
            </button>
            {isBookmarked && (
              <button
                type="button"
                className="btn btn-ghost btn-sm"
                style={{ color: "var(--danger)", fontSize: 11.5 }}
                disabled={saving}
                onClick={remove}
              >
                Untag
              </button>
            )}
            <button type="button" className="btn btn-ghost btn-sm" style={{ fontSize: 11.5 }} onClick={() => setOpen(false)}>
              Cancel
            </button>
          </div>
        </div>
      )}
    </div>
  );
}
