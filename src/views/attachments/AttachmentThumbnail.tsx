import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { isImagePreviewable } from "./types";

interface Props {
  attachmentId: string;
  storedPath?: string | null;
  filename: string;
  category: string;
  onZoom?: (src: string) => void;
}

export function AttachmentThumbnail({
  attachmentId,
  storedPath,
  filename,
  category,
  onZoom,
}: Props) {
  const isPdf = filename.toLowerCase().endsWith(".pdf");
  const isImg = isImagePreviewable(category, filename);
  const [src, setSrc] = useState<string | null>(null);
  const [loading, setLoading] = useState(isImg || isPdf);

  useEffect(() => {
    if (isImg || isPdf) {
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
    } else {
      setLoading(false);
    }
  }, [attachmentId, storedPath, isImg, isPdf]);

  const handleOpenPdf = async (e: React.MouseEvent) => {
    e.stopPropagation();
    try {
      await invoke("open_attachment_in_system", { input: { id: attachmentId } });
    } catch (err) {
      console.error(err);
    }
  };

  if (loading) {
    return (
      <div style={{ width: "100%", height: "100%", background: "var(--bg-0)", borderRadius: 6, display: "flex", alignItems: "center", justifyContent: "center", color: "var(--text-3)" }}>
        <span style={{ fontSize: 11 }}>Rendering preview...</span>
      </div>
    );
  }

  // Visual thumbnail rendered (for images OR visual PDF cover thumbnails)
  if (src) {
    return (
      <div 
        style={{ width: "100%", height: "100%", position: "relative", overflow: "hidden", borderRadius: 6, cursor: isPdf ? "pointer" : "zoom-in", border: "1px solid var(--border)" }}
        onClick={(e) => {
          if (isPdf) {
            handleOpenPdf(e);
          } else {
            e.stopPropagation();
            onZoom?.(src);
          }
        }}
        title={isPdf ? "Click to open PDF document" : "Click to zoom image"}
      >
        <img 
          src={src} 
          alt={filename} 
          style={{ width: "100%", height: "100%", objectFit: "cover", background: "#fff" }} 
        />
        {isPdf && (
          <span style={{ position: "absolute", top: 4, left: 4, background: "rgba(239, 68, 68, 0.9)", color: "#fff", padding: "1px 5px", borderRadius: 3, fontSize: 8.5, fontWeight: 700 }}>
            PDF
          </span>
        )}
        <span style={{ position: "absolute", bottom: 4, right: 4, background: "rgba(0,0,0,0.7)", color: "#fff", padding: "2px 6px", borderRadius: 4, fontSize: 9 }}>
          {isPdf ? "📄 Open" : "🔍 Zoom"}
        </span>
      </div>
    );
  }

  if (isPdf) {
    return (
      <div 
        style={{
          width: "100%",
          height: "100%",
          background: "linear-gradient(135deg, rgba(239, 68, 68, 0.12) 0%, var(--bg-2) 100%)",
          border: "1px solid rgba(239, 68, 68, 0.25)",
          borderRadius: 6,
          display: "flex",
          flexDirection: "column",
          alignItems: "center",
          justifyContent: "center",
          gap: 6,
          padding: 8,
          cursor: "pointer",
          position: "relative",
        }}
        onClick={handleOpenPdf}
        title="Click to open PDF document"
      >
        <span style={{ fontSize: 32 }}>📕</span>
        <div style={{ display: "flex", alignItems: "center", gap: 4 }}>
          <span style={{ fontSize: 10, fontWeight: 700, background: "rgba(239, 68, 68, 0.2)", color: "#f87171", padding: "1px 5px", borderRadius: 3 }}>
            PDF
          </span>
          <span style={{ fontSize: 9.5, color: "var(--text-2)", maxWidth: 100, overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
            Document
          </span>
        </div>
      </div>
    );
  }

  return (
    <div style={{ width: "100%", height: "100%", background: "var(--bg-2)", borderRadius: 6, display: "flex", flexDirection: "column", alignItems: "center", justifyContent: "center", gap: 6 }}>
      <span style={{ fontSize: 36 }}>
        {category === "dangerous" ? "🚨" : category === "documents" ? "📄" : category === "archives" ? "📦" : "📎"}
      </span>
      <span style={{ fontSize: 10, color: "var(--text-3)", textTransform: "uppercase" }}>{category}</span>
    </div>
  );
}
