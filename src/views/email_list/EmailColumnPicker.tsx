import { useRef, useEffect } from "react";
import { ColumnSettings } from "./types";

interface Props {
  show: boolean;
  onClose: () => void;
  columns: ColumnSettings;
  onToggleColumn: (key: keyof ColumnSettings) => void;
  onResetColumns: () => void;
}

export function EmailColumnPicker({
  show,
  onClose,
  columns,
  onToggleColumn,
  onResetColumns,
}: Props) {
  const columnPickerRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!show) return;
    const handler = (e: MouseEvent) => {
      if (columnPickerRef.current && !columnPickerRef.current.contains(e.target as Node)) {
        onClose();
      }
    };
    document.addEventListener("mousedown", handler);
    return () => document.removeEventListener("mousedown", handler);
  }, [show, onClose]);

  if (!show) return null;

  return (
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
          onClick={onResetColumns}
        >
          Reset
        </button>
      </div>
      <div style={{ display: "flex", flexDirection: "column", gap: 6, fontSize: 12 }}>
        <label className="row gap-2" style={{ cursor: "pointer" }}>
          <input type="checkbox" checked={columns.name} onChange={() => onToggleColumn("name")} />
          <span>Sender Name</span>
        </label>
        <label className="row gap-2" style={{ cursor: "pointer" }}>
          <input type="checkbox" checked={columns.from} onChange={() => onToggleColumn("from")} />
          <span>From Email</span>
        </label>
        <label className="row gap-2" style={{ cursor: "pointer" }}>
          <input type="checkbox" checked={columns.to} onChange={() => onToggleColumn("to")} />
          <span>To Recipient</span>
        </label>
        <label className="row gap-2" style={{ cursor: "pointer" }}>
          <input type="checkbox" checked={columns.subject} onChange={() => onToggleColumn("subject")} />
          <span>Subject &amp; Tags</span>
        </label>
        <label className="row gap-2" style={{ cursor: "pointer" }}>
          <input type="checkbox" checked={columns.attachments} onChange={() => onToggleColumn("attachments")} />
          <span>Attachments (📎/🖼️)</span>
        </label>
        <label className="row gap-2" style={{ cursor: "pointer" }}>
          <input type="checkbox" checked={columns.date} onChange={() => onToggleColumn("date")} />
          <span>Date Sent</span>
        </label>
        <label className="row gap-2" style={{ cursor: "pointer" }}>
          <input type="checkbox" checked={columns.folder} onChange={() => onToggleColumn("folder")} />
          <span>Folder Category</span>
        </label>
        <label className="row gap-2" style={{ cursor: "pointer" }}>
          <input type="checkbox" checked={columns.risk} onChange={() => onToggleColumn("risk")} />
          <span>Risk Score</span>
        </label>
        <label className="row gap-2" style={{ cursor: "pointer" }}>
          <input type="checkbox" checked={columns.tag} onChange={() => onToggleColumn("tag")} />
          <span>Tag / Locker (🔖)</span>
        </label>
      </div>
    </div>
  );
}
