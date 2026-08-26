import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { RichEmailBodyViewer } from "./RichEmailBodyViewer";

export interface EmailModalData {
  id: string;
  evidence_id?: string;
  case_id?: string;
  message_id: string | null;
  from_addr: string;
  from_display: string | null;
  to_addrs: string;
  cc_addrs?: string;
  subject: string | null;
  date_sent?: string | null;
  date_sent_utc?: string | null;
  headers_raw?: string | null;
  body_text?: string | null;
  body_html?: string | null;
  folder_name?: string | null;
  folder_category?: string;
}

interface AttachmentItem {
  id: string;
  email_id: string;
  filename: string | null;
  mime_type: string;
  size_bytes: number;
  sha256_hash?: string | null;
  is_dangerous?: boolean;
}

interface Props {
  email: EmailModalData | null;
  onClose: () => void;
  titleSuffix?: string;
}

export function EmailDetailModal({ email, onClose, titleSuffix = "Return" }: Props) {
  if (!email) return null;

  const [attachments, setAttachments] = useState<AttachmentItem[]>([]);
  const [loadingAtts, setLoadingAtts] = useState(false);
  const [previews, setPreviews] = useState<Record<string, string>>({});
  const [zoomImage, setZoomImage] = useState<{ src: string; filename: string } | null>(null);
  const [exportingId, setExportingId] = useState<string | null>(null);
  const [toastMsg, setToastMsg] = useState<string | null>(null);

  useEffect(() => {
    if (!email.id) return;
    setLoadingAtts(true);
    invoke<AttachmentItem[]>("email_attachments", { input: { email_id: email.id } })
      .then(async (res) => {
        const atts = res || [];
        setAttachments(atts);
        
        // Fetch previews for image and document attachments
        const previewMap: Record<string, string> = {};
        for (const a of atts) {
          if (isImage(a.mime_type, a.filename) || (a.filename && a.filename.toLowerCase().endsWith(".pdf"))) {
            try {
              const dataUrl = await invoke<string | null>("get_attachment_preview", { input: { attachment_id: a.id } });
              if (dataUrl) {
                previewMap[a.id] = dataUrl;
              }
            } catch (err) {
              console.warn("Failed preview for", a.filename, err);
            }
          }
        }
        setPreviews(previewMap);
      })
      .catch((err) => {
        console.error("Failed to load attachments:", err);
        setAttachments([]);
      })
      .finally(() => setLoadingAtts(false));
  }, [email.id]);

  const showToast = (msg: string) => {
    setToastMsg(msg);
    setTimeout(() => setToastMsg(null), 3000);
  };

  const handleExport = async (attId: string, filename: string) => {
    setExportingId(attId);
    try {
      const path = await invoke<string>("export_attachment", { input: { attachment_id: attId } });
      showToast(`✓ Exported ${filename} to: ${path}`);
    } catch (e) {
      console.error("Export failed:", e);
      showToast(`❌ Export failed: ${e}`);
    } finally {
      setExportingId(null);
    }
  };

  const handleOpenSystem = async (attId: string, filename: string) => {
    try {
      await invoke<string>("open_attachment_in_system", { input: { attachment_id: attId } });
      showToast(`✓ Opened ${filename} in system viewer`);
    } catch (e) {
      console.error("Failed to open attachment in system:", e);
      showToast(`❌ Could not open: ${e}`);
    }
  };

  const formatSize = (bytes: number) => {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(2)} MB`;
  };

  const isImage = (mime: string, name: string | null) => {
    const lower = (name || "").toLowerCase();
    return (mime || "").startsWith("image/") || lower.endsWith(".jpg") || lower.endsWith(".jpeg") || lower.endsWith(".png") || lower.endsWith(".gif") || lower.endsWith(".webp") || lower.endsWith(".svg");
  };

  return (
    <div
      style={{
        position: "fixed",
        inset: 0,
        background: "rgba(0, 0, 0, 0.82)",
        backdropFilter: "blur(6px)",
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
        zIndex: 10000,
        padding: 24,
      }}
      onClick={onClose}
    >
      <div
        style={{
          background: "#0f172a",
          border: "1px solid #334155",
          borderRadius: "var(--r-md)",
          width: "100%",
          maxWidth: 880,
          maxHeight: "92vh",
          overflowY: "auto",
          padding: 24,
          boxShadow: "0 25px 60px rgba(0,0,0,0.8)",
          position: "relative",
        }}
        onClick={(e) => e.stopPropagation()}
      >
        {/* Toast Notification */}
        {toastMsg && (
          <div
            style={{
              position: "fixed",
              bottom: 30,
              right: 30,
              background: "#1e293b",
              border: "1px solid #38bdf8",
              color: "#f8fafc",
              padding: "10px 18px",
              borderRadius: "var(--r-sm)",
              fontSize: 12,
              zIndex: 10001,
              boxShadow: "0 10px 25px rgba(0,0,0,0.5)",
            }}
          >
            {toastMsg}
          </div>
        )}

        {/* Zoom Modal */}
        {zoomImage && (
          <div
            style={{
              position: "fixed",
              inset: 0,
              background: "rgba(0,0,0,0.88)",
              backdropFilter: "blur(8px)",
              display: "flex",
              flexDirection: "column",
              alignItems: "center",
              justifyContent: "center",
              zIndex: 10002,
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
                background: "#0f172a",
                borderRadius: "var(--r-md)",
                padding: 16,
                border: "1px solid #334155",
              }}
              onClick={(e) => e.stopPropagation()}
            >
              <div className="row between" style={{ width: "100%", marginBottom: 12 }}>
                <span style={{ fontSize: 14, fontWeight: 700, color: "#f8fafc" }}>🖼️ {zoomImage.filename}</span>
                <button className="btn btn-ghost btn-sm" onClick={() => setZoomImage(null)}>✕ Close</button>
              </div>
              <img src={zoomImage.src} alt={zoomImage.filename} style={{ maxWidth: "100%", maxHeight: "75vh", objectFit: "contain", borderRadius: 4 }} />
            </div>
          </div>
        )}

        {/* Header */}
        <div className="row between mb-3" style={{ borderBottom: "1px solid #1e293b", paddingBottom: 12 }}>
          <div className="row gap-2" style={{ alignItems: "center", overflow: "hidden", minWidth: 0, flex: 1 }}>
            <span style={{ fontSize: 18 }}>✉️</span>
            <div style={{ overflow: "hidden", minWidth: 0 }}>
              <h3 style={{ fontSize: 16, fontWeight: 700, margin: 0, color: "#f8fafc", overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
                {email.subject || "(No Subject)"}
              </h3>
              <span className="muted" style={{ fontSize: 11 }}>Message ID: {email.message_id || email.id}</span>
            </div>
          </div>
          <button className="btn btn-ghost btn-sm" style={{ flexShrink: 0, marginLeft: 12 }} onClick={onClose}>
            ✕ Close &amp; {titleSuffix}
          </button>
        </div>

        {/* Email Metadata Header Table */}
        <div style={{ background: "#1e293b", padding: 14, borderRadius: "var(--r-sm)", marginBottom: 16, fontSize: 12.5 }}>
          <div className="row mb-1">
            <span className="muted" style={{ width: 80, fontWeight: 600 }}>From:</span>
            <span style={{ color: "#38bdf8", fontWeight: 600 }}>{email.from_display ? `${email.from_display} <${email.from_addr}>` : email.from_addr}</span>
          </div>
          <div className="row mb-1">
            <span className="muted" style={{ width: 80, fontWeight: 600 }}>To:</span>
            <span style={{ color: "#e2e8f0" }}>{email.to_addrs}</span>
          </div>
          {email.cc_addrs && (
            <div className="row mb-1">
              <span className="muted" style={{ width: 80, fontWeight: 600 }}>Cc:</span>
              <span style={{ color: "#94a3b8" }}>{email.cc_addrs}</span>
            </div>
          )}
          <div className="row mb-1">
            <span className="muted" style={{ width: 80, fontWeight: 600 }}>Date UTC:</span>
            <span style={{ color: "#94a3b8", fontFamily: "var(--mono)" }}>{email.date_sent_utc || email.date_sent || "Unknown"}</span>
          </div>
          <div className="row between">
            <div className="row">
              <span className="muted" style={{ width: 80, fontWeight: 600 }}>Folder:</span>
              <span className="badge badge-gray">{email.folder_name || email.folder_category || "inbox"}</span>
            </div>
            {attachments.length > 0 && (
              <span className="badge badge-blue">
                📎 {attachments.length} Attachment{attachments.length !== 1 ? "s" : ""}
              </span>
            )}
          </div>
        </div>

        {/* Visual Media Gallery for Image Attachments */}
        {Object.keys(previews).length > 0 && (
          <div style={{ marginBottom: 16, background: "rgba(15, 23, 42, 0.95)", border: "1px solid #38bdf8", borderRadius: "var(--r-md)", padding: 14 }}>
            <div className="row between mb-2" style={{ alignItems: "center" }}>
              <span style={{ fontSize: 13, fontWeight: 700, color: "#38bdf8" }}>
                🖼️ Forensic Visual Attachments ({Object.keys(previews).length})
              </span>
              <span className="muted text-xs">Click image to expand / inspect full resolution</span>
            </div>
            <div style={{ display: "grid", gridTemplateColumns: "repeat(auto-fill, minmax(200px, 1fr))", gap: 12 }}>
              {attachments.filter(a => previews[a.id]).map(a => (
                <div 
                  key={a.id}
                  style={{ background: "#020617", border: "1px solid #1e293b", borderRadius: 6, padding: 8, cursor: "zoom-in", transition: "all 0.15s ease" }}
                  onClick={() => setZoomImage({ src: previews[a.id], filename: a.filename || "image" })}
                >
                  <img 
                    src={previews[a.id]} 
                    alt={a.filename || "image"} 
                    style={{ width: "100%", height: 140, objectFit: "contain", background: "#090d16", borderRadius: 4 }} 
                  />
                  <div style={{ fontSize: 11, fontWeight: 600, color: "#f8fafc", marginTop: 6, overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
                    {a.filename}
                  </div>
                  <div className="row between mt-1" style={{ fontSize: 10, color: "#64748b" }}>
                    <span>{formatSize(a.size_bytes)}</span>
                    <span style={{ color: "#38bdf8" }}>🔍 Click to zoom</span>
                  </div>
                </div>
              ))}
            </div>
          </div>
        )}

        {/* Attachments Section */}
        {attachments.length > 0 && (
          <div style={{ marginBottom: 16, background: "#131f37", border: "1px solid #1e3a5f", borderRadius: "var(--r-sm)", padding: 14 }}>
            <div className="row between mb-2" style={{ alignItems: "center" }}>
              <div style={{ fontSize: 12, fontWeight: 700, color: "#38bdf8", textTransform: "uppercase" }}>
                📎 Forensic Attachments ({attachments.length})
              </div>
            </div>

            <div style={{ display: "flex", flexDirection: "column", gap: 8 }}>
              {attachments.map((att) => {
                const fname = att.filename || "unnamed_attachment";
                const isImg = isImage(att.mime_type, fname);
                const prev = previews[att.id];

                return (
                  <div
                    key={att.id}
                    className="row between"
                    style={{
                      background: "#0a1122",
                      border: "1px solid #1e293b",
                      borderRadius: 6,
                      padding: "8px 12px",
                      alignItems: "center",
                      fontSize: 12,
                    }}
                  >
                    <div className="row gap-2" style={{ alignItems: "center", minWidth: 0, flex: 1 }}>
                      {prev ? (
                        <img 
                          src={prev} 
                          alt={fname} 
                          style={{ width: 44, height: 44, objectFit: "cover", borderRadius: 4, cursor: "zoom-in", border: "1px solid #38bdf8", flexShrink: 0 }} 
                          onClick={() => setZoomImage({ src: prev, filename: fname })}
                          title="Click to zoom image"
                        />
                      ) : (
                        <span style={{ fontSize: 20 }}>{isImg ? "🖼️" : att.mime_type.includes("pdf") ? "📕" : att.mime_type.includes("zip") ? "📦" : "📎"}</span>
                      )}
                      <div style={{ minWidth: 0, overflow: "hidden" }}>
                        <div style={{ fontWeight: 600, color: "#f8fafc", overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
                          {fname}
                        </div>
                        <div className="row gap-2" style={{ fontSize: 11, color: "#64748b" }}>
                          <span>{formatSize(att.size_bytes)}</span>
                          <span>•</span>
                          <span className="badge badge-gray" style={{ fontSize: 10, padding: "0 4px" }}>{att.mime_type}</span>
                          {att.sha256_hash && (
                            <>
                              <span>•</span>
                              <span style={{ fontFamily: "var(--mono)" }} title={att.sha256_hash}>
                                SHA256: {att.sha256_hash.slice(0, 10)}...
                              </span>
                            </>
                          )}
                        </div>
                      </div>
                    </div>

                    <div className="row gap-2" style={{ flexShrink: 0, marginLeft: 12 }}>
                      {prev && (
                        <button
                          className="btn btn-ghost btn-sm"
                          style={{ padding: "3px 8px", fontSize: 11, color: "#38bdf8", border: "1px solid rgba(56, 189, 248, 0.3)" }}
                          onClick={() => setZoomImage({ src: prev, filename: fname })}
                        >
                          👁️ Zoom
                        </button>
                      )}
                      <button
                        className="btn btn-ghost btn-sm"
                        style={{ padding: "3px 8px", fontSize: 11 }}
                        onClick={() => handleOpenSystem(att.id, fname)}
                        title="Open in system viewer"
                      >
                        📂 Open File
                      </button>
                      <button
                        className="btn btn-primary btn-sm"
                        style={{ padding: "3px 10px", fontSize: 11 }}
                        disabled={exportingId === att.id}
                        onClick={() => handleExport(att.id, fname)}
                      >
                        {exportingId === att.id ? "Exporting..." : "💾 Export"}
                      </button>
                    </div>
                  </div>
                );
              })}
            </div>
          </div>
        )}

        {/* Email Body View */}
        <div style={{ marginBottom: 16 }}>
          <div className="label">EMAIL MESSAGE CONTENT</div>
          <RichEmailBodyViewer
            bodyText={email.body_text}
            bodyHtml={email.body_html}
            emailId={email.id}
            defaultMode="rendered"
          />
        </div>

        {/* Raw Headers Toggle Section */}
        {email.headers_raw && (
          <details style={{ marginTop: 12 }}>
            <summary style={{ cursor: "pointer", fontSize: 12, fontWeight: 600, color: "#94a3b8" }}>
              🧬 View Raw Transport Headers ({email.headers_raw.length} bytes)
            </summary>
            <pre 
              style={{ 
                marginTop: 8, 
                padding: 12, 
                background: "#020617", 
                border: "1px solid #1e293b", 
                borderRadius: "var(--r-sm)", 
                fontSize: 11, 
                color: "#64748b", 
                maxHeight: 200, 
                overflowY: "auto", 
                whiteSpace: "pre-wrap" 
              }}
            >
              {email.headers_raw}
            </pre>
          </details>
        )}
      </div>
    </div>
  );
}
