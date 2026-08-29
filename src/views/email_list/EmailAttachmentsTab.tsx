import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { formatBytes } from "./types";

interface Props {
  emailId: string;
}

export function EmailAttachmentsTab({ emailId }: Props) {
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
