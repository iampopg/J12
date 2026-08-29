import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";

interface Props {
  attachmentId: string;
  storedPath?: string | null;
  filename: string;
  onZoom?: (src: string) => void;
}

export function InspectorPhotoViewer({
  attachmentId,
  storedPath,
  filename,
  onZoom,
}: Props) {
  const isPdf = filename.toLowerCase().endsWith(".pdf");
  const [src, setSrc] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    invoke<string | null>("get_attachment_preview", {
      input: {
        attachment_id: attachmentId,
        stored_path: storedPath,
      },
    })
      .then((data) => {
        if (data) setSrc(data);
      })
      .catch(console.error)
      .finally(() => setLoading(false));
  }, [attachmentId, storedPath]);

  const handleOpenPdf = async () => {
    try {
      await invoke("open_attachment_in_system", { input: { id: attachmentId } });
    } catch (err) {
      console.error(err);
    }
  };

  if (loading) {
    return <div style={{ height: 160, display: "flex", alignItems: "center", justifyContent: "center", color: "var(--text-3)", fontSize: 12 }}>Loading preview...</div>;
  }

  if (isPdf) {
    return (
      <div 
        style={{
          padding: 16,
          background: "linear-gradient(135deg, rgba(239, 68, 68, 0.1) 0%, var(--bg-1) 100%)",
          border: "1px solid rgba(239, 68, 68, 0.3)",
          borderRadius: "var(--r-md)",
          textAlign: "center",
          display: "flex",
          flexDirection: "column",
          alignItems: "center",
          gap: 10,
        }}
      >
        <span style={{ fontSize: 44 }}>📕</span>
        <div>
          <div style={{ fontSize: 13, fontWeight: 700, color: "var(--text-0)" }}>{filename}</div>
          <div style={{ fontSize: 11, color: "#f87171", marginTop: 2 }}>Portable Document Format (PDF)</div>
        </div>
        <button
          className="btn btn-primary btn-sm"
          style={{ background: "#ef4444", borderColor: "#ef4444", marginTop: 4, display: "flex", alignItems: "center", gap: 6 }}
          onClick={handleOpenPdf}
        >
          <span>📄</span> Open in PDF Reader
        </button>
      </div>
    );
  }

  if (src) {
    return (
      <div 
        style={{ position: "relative", cursor: "zoom-in", textAlign: "center" }}
        onClick={() => onZoom?.(src)}
        title="Click to zoom image"
      >
        <img 
          src={src} 
          alt={filename} 
          style={{ maxWidth: "100%", maxHeight: 200, objectFit: "contain", display: "block", margin: "0 auto" }} 
        />
        <div style={{ padding: "4px 8px", background: "rgba(0,0,0,0.7)", color: "#fff", fontSize: 10 }}>
          🔍 Click image to expand full resolution
        </div>
      </div>
    );
  }

  return <div style={{ height: 80, display: "flex", alignItems: "center", justifyContent: "center", color: "var(--text-3)", fontSize: 12 }}>No visual preview available</div>;
}
