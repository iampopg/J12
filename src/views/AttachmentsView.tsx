import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";

export interface CaseAttachmentItem {
  id: string;
  email_id: string;
  filename: string;
  sha256: string;
  mime_type: string;
  size_bytes: number;
  stored_path: string | null;
  entropy: number | null;
  risk_flags: string | null;
  email_subject: string | null;
  email_from: string;
  email_date: string | null;
  email_risk_score: number;
  category: "dangerous" | "documents" | "images" | "archives" | "media" | "other";
}

interface Props {
  caseId: string;
  onSelectEmail?: (emailId: string) => void;
}

export function AttachmentsView({ caseId, onSelectEmail }: Props) {
  const [attachments, setAttachments] = useState<CaseAttachmentItem[]>([]);
  const [category, setCategory] = useState<string>("all");
  const [search, setSearch] = useState<string>("");
  const [loading, setLoading] = useState<boolean>(true);
  const [viewMode, setViewMode] = useState<"table" | "grid">("table");
  const [selectedAtt, setSelectedAtt] = useState<CaseAttachmentItem | null>(null);
  const [zoomImage, setZoomImage] = useState<{ src: string; filename: string } | null>(null);
  const [toastMessage, setToastMessage] = useState<string | null>(null);

  const showToast = (msg: string) => {
    setToastMessage(msg);
    setTimeout(() => setToastMessage(null), 3000);
  };

  useEffect(() => {
    loadData();
  }, [caseId, category]);

  const loadData = async () => {
    setLoading(true);
    try {
      const list = await invoke<CaseAttachmentItem[]>("case_attachments_list", {
        input: { case_id: caseId, category, search }
      });
      setAttachments(list);
    } catch (e) {
      console.error("Failed to load attachments:", e);
    } finally {
      setLoading(false);
    }
  };

  const handleSearchSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    loadData();
  };

  const copyToClipboard = (text: string) => {
    navigator.clipboard.writeText(text);
    showToast("✓ Copied SHA-256 hash to clipboard");
  };

  const handleOpenSystem = async (attId: string, filename: string) => {
    try {
      await invoke("open_attachment_in_system", { input: { attachment_id: attId } });
      showToast(`✓ Opened ${filename}`);
    } catch (e: any) {
      console.error(e);
      showToast(`❌ Could not open file: ${e}`);
    }
  };

  const handleRevealFinder = async (attId: string) => {
    try {
      await invoke("reveal_in_finder", { input: { attachment_id: attId } });
      showToast("✓ Revealed in Finder");
    } catch (e: any) {
      console.error(e);
      showToast(`❌ Could not reveal: ${e}`);
    }
  };

  const exportSingleAttachment = async (att: CaseAttachmentItem) => {
    try {
      const savedPath = await invoke<string>("export_attachment", {
        input: { attachment_id: att.id }
      });
      showToast(`📥 Exported to: ${savedPath}`);
    } catch (e) {
      showToast(`Export error: ${e}`);
    }
  };

  const formatSize = (bytes: number) => {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(2)} MB`;
  };

  const getFileIcon = (cat: string, filename: string) => {
    const lower = filename.toLowerCase();
    if (cat === "dangerous" || lower.endsWith(".exe") || lower.endsWith(".scr") || lower.endsWith(".vbs")) return "🚨";
    if (lower.endsWith(".pdf")) return "📕";
    if (lower.endsWith(".doc") || lower.endsWith(".docx")) return "📘";
    if (lower.endsWith(".xls") || lower.endsWith(".xlsx") || lower.endsWith(".csv")) return "📗";
    if (lower.endsWith(".ppt") || lower.endsWith(".pptx")) return "📙";
    if (cat === "images") return "🖼️";
    if (cat === "archives") return "📦";
    if (cat === "media") return "🎵";
    return "📎";
  };

  // Metrics for categories
  const counts = {
    all: attachments.length,
    dangerous: attachments.filter(a => a.category === "dangerous").length,
    documents: attachments.filter(a => a.category === "documents").length,
    images: attachments.filter(a => a.category === "images").length,
    archives: attachments.filter(a => a.category === "archives").length,
    media: attachments.filter(a => a.category === "media").length,
  };

  return (
    <div>
      {/* Image Zoom Modal */}
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
              boxShadow: "0 25px 50px rgba(0,0,0,0.7)",
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
              style={{
                maxWidth: "100%",
                maxHeight: "75vh",
                objectFit: "contain",
                borderRadius: 4,
                border: "1px solid var(--border)",
              }}
            />
          </div>
        </div>
      )}

      {/* Toast Notification */}
      {toastMessage && (
        <div 
          className="card"
          style={{
            position: "fixed",
            bottom: 24,
            right: 24,
            zIndex: 9999,
            background: "#1e293b",
            border: "1px solid #22c55e",
            color: "#4ade80",
            padding: "10px 18px",
            fontWeight: 600,
            fontSize: 13,
            boxShadow: "0 10px 25px rgba(0,0,0,0.5)",
          }}
        >
          {toastMessage}
        </div>
      )}

      {/* Header */}
      <div className="row between mb-4">
        <div>
          <h2 style={{ fontSize: 22, fontWeight: 700, color: "var(--text-0)" }}>
            Evidence Attachments &amp; Payloads Gallery
          </h2>
          <p className="muted" style={{ margin: 0 }}>
            Forensic catalog of all extracted files, photographic evidence, cryptographic hashes, and dangerous payloads.
          </p>
        </div>
        <div className="row gap-2">
          <div className="row" style={{ background: "var(--bg-2)", borderRadius: "var(--r-sm)", padding: 2, border: "1px solid var(--border)" }}>
            <button
              className={`btn btn-sm ${viewMode === "table" ? "btn-primary" : "btn-ghost"}`}
              style={{ padding: "4px 10px", fontSize: 12 }}
              onClick={() => setViewMode("table")}
            >
              📄 List
            </button>
            <button
              className={`btn btn-sm ${viewMode === "grid" ? "btn-primary" : "btn-ghost"}`}
              style={{ padding: "4px 10px", fontSize: 12 }}
              onClick={() => setViewMode("grid")}
            >
              🖼️ Photo &amp; Scan Gallery
            </button>
          </div>
          <button className="btn btn-ghost btn-sm" onClick={loadData}>
            ↻ Refresh
          </button>
        </div>
      </div>

      {/* Category Filter Tabs */}
      <div className="row gap-2 mb-4" style={{ flexWrap: "wrap" }}>
        <button
          className={`btn ${category === "all" ? "btn-primary" : "btn-ghost"} btn-sm`}
          onClick={() => setCategory("all")}
        >
          📎 All Files ({counts.all})
        </button>
        <button
          className={`btn ${category === "dangerous" ? "btn-danger" : "btn-ghost"} btn-sm`}
          onClick={() => setCategory("dangerous")}
          style={{ color: category === "dangerous" ? "#fff" : "var(--danger)", border: "1px solid var(--danger)" }}
        >
          🚨 Dangerous / Executables ({counts.dangerous})
        </button>
        <button
          className={`btn ${category === "images" ? "btn-primary" : "btn-ghost"} btn-sm`}
          onClick={() => { setCategory("images"); setViewMode("grid"); }}
        >
          🖼️ Images &amp; Scans ({counts.images})
        </button>
        <button
          className={`btn ${category === "documents" ? "btn-primary" : "btn-ghost"} btn-sm`}
          onClick={() => setCategory("documents")}
        >
          📄 Documents &amp; PDFs ({counts.documents})
        </button>
        <button
          className={`btn ${category === "archives" ? "btn-primary" : "btn-ghost"} btn-sm`}
          onClick={() => setCategory("archives")}
        >
          📦 Archives ({counts.archives})
        </button>
        <button
          className={`btn ${category === "media" ? "btn-primary" : "btn-ghost"} btn-sm`}
          onClick={() => setCategory("media")}
        >
          🎵 Voicemails &amp; Audio ({counts.media})
        </button>
      </div>

      {/* Loading Progress Bar */}
      {loading && (
        <div className="card mb-4" style={{ padding: "14px 18px", background: "var(--bg-2)", border: "1px solid var(--accent)", boxShadow: "0 4px 20px rgba(0,0,0,0.25)" }}>
          <div className="row between mb-2">
            <span style={{ fontSize: 13, fontWeight: 600, color: "var(--text-0)" }}>
              ⚡ Loading and verifying cryptographic attachment signatures...
            </span>
          </div>
          <div style={{ width: "100%", height: 6, background: "var(--bg-0)", borderRadius: 3, overflow: "hidden" }}>
            <div
              style={{
                width: "100%",
                height: "100%",
                background: "linear-gradient(90deg, #3b82f6, #06b6d4, #10b981)",
                borderRadius: 3,
              }}
            />
          </div>
        </div>
      )}

      {/* Filter & Search Bar */}
      <div className="card mb-4" style={{ padding: "12px 16px" }}>
        <form onSubmit={handleSearchSubmit} className="row between gap-4">
          <div className="row gap-2" style={{ flex: 1 }}>
            <input
              className="input"
              style={{ flex: 1, padding: "8px 12px", fontSize: 13 }}
              placeholder="Search by filename, extension (.exe, .pdf, .jpg), or SHA-256 hash..."
              value={search}
              onChange={(e) => setSearch(e.target.value)}
            />
            <button type="submit" className="btn btn-primary btn-sm">🔍 Search</button>
            {search && (
              <button 
                type="button" 
                className="btn btn-ghost btn-sm" 
                onClick={() => { setSearch(""); setTimeout(loadData, 50); }}
              >
                Clear
              </button>
            )}
          </div>
          <div className="muted text-sm">
            Total <strong>{attachments.length}</strong> attachment files
          </div>
        </form>
      </div>

      {/* Main Grid: Attachment List & Inspector Drawer */}
      <div style={{ display: "grid", gridTemplateColumns: selectedAtt ? "1fr 380px" : "1fr", gap: 16 }}>
        
        {/* Content: Table or Photo Grid */}
        <div>
          {attachments.length === 0 && !loading ? (
            <div className="card empty" style={{ padding: 40 }}>
              No attachments found matching the current filter.
            </div>
          ) : viewMode === "grid" ? (
            /* Photo / Scan Thumbnail Gallery Grid */
            <div style={{ display: "grid", gridTemplateColumns: "repeat(auto-fill, minmax(200px, 1fr))", gap: 16 }}>
              {attachments.map((att) => (
                <div
                  key={att.id}
                  className="card"
                  style={{
                    padding: 12,
                    display: "flex",
                    flexDirection: "column",
                    cursor: "pointer",
                    border: selectedAtt?.id === att.id ? "2px solid var(--accent)" : "1px solid var(--border)",
                    background: selectedAtt?.id === att.id ? "var(--bg-3)" : "var(--bg-1)",
                    transition: "transform 0.15s ease, border-color 0.15s ease",
                  }}
                  onClick={() => setSelectedAtt(att)}
                >
                  <div style={{ width: "100%", height: 140, marginBottom: 10, position: "relative" }}>
                    <AttachmentThumbnail 
                      attachmentId={att.id} 
                      storedPath={att.stored_path}
                      filename={att.filename} 
                      category={att.category}
                      onZoom={(src) => setZoomImage({ src, filename: att.filename })}
                    />
                  </div>
                  <div style={{ fontWeight: 600, fontSize: 13, color: "var(--text-0)", overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }} title={att.filename}>
                    {att.filename}
                  </div>
                  <div className="row between mt-1" style={{ fontSize: 11, color: "var(--text-3)" }}>
                    <span>{formatSize(att.size_bytes)}</span>
                    <span className="badge" style={{ fontSize: 9 }}>{att.category}</span>
                  </div>
                  <div className="row gap-1 mt-2" style={{ justifyContent: "flex-end", flexWrap: "wrap" }}>
                    <button
                      className="btn btn-primary btn-sm"
                      style={{ padding: "2px 6px", fontSize: 10 }}
                      onClick={(e) => { e.stopPropagation(); handleOpenSystem(att.id, att.filename); }}
                      title="Open file in default system application (Preview / Acrobat / Office)"
                    >
                      👁️ Open
                    </button>
                    <button
                      className="btn btn-ghost btn-sm"
                      style={{ padding: "2px 6px", fontSize: 10 }}
                      onClick={(e) => { e.stopPropagation(); exportSingleAttachment(att); }}
                      title="Export to Downloads"
                    >
                      📥 Export
                    </button>
                    {onSelectEmail && (
                      <button
                        className="btn btn-ghost btn-sm"
                        style={{ padding: "2px 6px", fontSize: 10 }}
                        onClick={(e) => { e.stopPropagation(); onSelectEmail(att.email_id); }}
                        title="Jump to parent email"
                      >
                        ✉️ Email
                      </button>
                    )}
                  </div>
                </div>
              ))}
            </div>
          ) : (
            /* Table View */
            <div className="card" style={{ padding: 0, overflow: "hidden" }}>
              <table style={{ width: "100%", borderCollapse: "collapse", textAlign: "left" }}>
                <thead>
                  <tr style={{ background: "var(--bg-1)", borderBottom: "1px solid var(--border)" }}>
                    <th className="th" style={{ width: 50 }}></th>
                    <th className="th">Filename &amp; Type</th>
                    <th className="th">Size</th>
                    <th className="th">SHA-256 Hash</th>
                    <th className="th">Parent Email</th>
                    <th className="th" style={{ textAlign: "right" }}>Actions</th>
                  </tr>
                </thead>
                <tbody>
                  {attachments.map((att) => {
                    const isDangerous = att.category === "dangerous";
                    const isSelected = selectedAtt?.id === att.id;

                    return (
                      <tr 
                        key={att.id} 
                        className={`tr ${isSelected ? "selected" : ""}`}
                        style={{ 
                          cursor: "pointer", 
                          background: isDangerous ? "rgba(239, 68, 68, 0.05)" : isSelected ? "var(--bg-3)" : undefined 
                        }}
                        onClick={() => setSelectedAtt(att)}
                      >
                        <td className="td" style={{ textAlign: "center", fontSize: 20 }}>
                          {getFileIcon(att.category, att.filename)}
                        </td>
                        <td className="td">
                          <div style={{ fontWeight: 600, fontSize: 13, color: isDangerous ? "var(--danger)" : "var(--text-0)" }}>
                            {att.filename}
                          </div>
                          <div style={{ fontSize: 11, color: "var(--text-3)", fontFamily: "var(--mono)" }}>
                            {att.mime_type}
                          </div>
                        </td>
                        <td className="td" style={{ fontSize: 12, fontFamily: "var(--mono)" }}>
                          {formatSize(att.size_bytes)}
                        </td>
                        <td className="td">
                          <div className="row gap-1" style={{ alignItems: "center" }}>
                            <span 
                              style={{ 
                                fontFamily: "var(--mono)", 
                                fontSize: 11, 
                                color: "#38bdf8",
                                maxWidth: 120,
                                overflow: "hidden",
                                textOverflow: "ellipsis",
                                whiteSpace: "nowrap"
                              }}
                              title={att.sha256}
                            >
                              {att.sha256 ? `${att.sha256.slice(0, 10)}…` : "—"}
                            </span>
                            {isDangerous && <span className="badge badge-red">RISK</span>}
                          </div>
                        </td>
                        <td className="td">
                          <div style={{ fontSize: 12, color: "var(--text-1)", maxWidth: 180, overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
                            {att.email_subject || "(No Subject)"}
                          </div>
                          <div style={{ fontSize: 10, color: "var(--text-3)" }}>
                            {att.email_from}
                          </div>
                        </td>
                        <td className="td" style={{ textAlign: "right" }}>
                          <div className="row gap-1" style={{ justifyContent: "flex-end" }}>
                            <button 
                              className="btn btn-primary btn-sm" 
                              style={{ padding: "3px 8px", fontSize: 11 }}
                              onClick={(e) => { e.stopPropagation(); handleOpenSystem(att.id, att.filename); }}
                              title="Open file in default system viewer"
                            >
                              👁️ Open
                            </button>
                            <button 
                              className="btn btn-ghost btn-sm" 
                              style={{ padding: "3px 8px", fontSize: 11 }}
                              onClick={(e) => { e.stopPropagation(); exportSingleAttachment(att); }}
                              title="Export attachment to Downloads"
                            >
                              📥 Export
                            </button>
                            {onSelectEmail && (
                              <button 
                                className="btn btn-ghost btn-sm" 
                                style={{ padding: "3px 8px", fontSize: 11 }}
                                onClick={(e) => { e.stopPropagation(); onSelectEmail(att.email_id); }}
                                title="Jump to parent email"
                              >
                                ✉️ Email
                              </button>
                            )}
                          </div>
                        </td>
                      </tr>
                    );
                  })}
                </tbody>
              </table>
            </div>
          )}
        </div>

        {/* Live Detail Inspector Drawer */}
        {selectedAtt && (
          <div className="card" style={{ position: "sticky", top: 16, height: "fit-content", padding: 20 }}>
            <div className="row between mb-4">
              <h3 style={{ fontSize: 15, fontWeight: 700, color: "var(--text-0)", margin: 0 }}>
                {getFileIcon(selectedAtt.category, selectedAtt.filename)} File Metadata Dossier
              </h3>
              <button className="btn btn-ghost btn-sm" onClick={() => setSelectedAtt(null)}>✕</button>
            </div>

            {/* Photo Preview in Inspector */}
            {selectedAtt.category === "images" && (
              <div style={{ marginBottom: 16, borderRadius: "var(--r-md)", overflow: "hidden", border: "1px solid var(--border)", background: "#000" }}>
                <InspectorPhotoViewer 
                  attachmentId={selectedAtt.id} 
                  storedPath={selectedAtt.stored_path}
                  filename={selectedAtt.filename} 
                  onZoom={(src) => setZoomImage({ src, filename: selectedAtt.filename })}
                />
              </div>
            )}

            <div style={{ marginBottom: 14 }}>
              <div className="label">FILENAME</div>
              <div style={{ fontSize: 14, fontWeight: 700, color: "var(--text-0)", wordBreak: "break-all" }}>
                {selectedAtt.filename}
              </div>
              <div style={{ fontSize: 11, color: "var(--text-3)", fontFamily: "var(--mono)", marginTop: 2 }}>
                Category: <strong>{selectedAtt.category.toUpperCase()}</strong> · {formatSize(selectedAtt.size_bytes)}
              </div>
            </div>

            <div style={{ marginBottom: 14 }}>
              <div className="label">CRYPTOGRAPHIC SHA-256 HASH</div>
              <div 
                style={{
                  background: "var(--bg-3)",
                  border: "1px solid var(--border)",
                  borderRadius: "var(--r-sm)",
                  padding: "8px 10px",
                  fontFamily: "var(--mono)",
                  fontSize: 10,
                  color: "#38bdf8",
                  wordBreak: "break-all"
                }}
              >
                {selectedAtt.sha256}
              </div>
              <button 
                className="btn btn-ghost btn-sm" 
                style={{ width: "100%", marginTop: 6, fontSize: 11 }}
                onClick={() => copyToClipboard(selectedAtt.sha256)}
              >
                📋 Copy Full SHA-256 Hash
              </button>
            </div>

            <div style={{ marginBottom: 14 }}>
              <div className="label">ENTROPY &amp; HEURISTIC SCORE</div>
              <div className="row between" style={{ marginBottom: 4 }}>
                <span style={{ fontSize: 12 }}>Shannon Entropy:</span>
                <span style={{ fontFamily: "var(--mono)", fontWeight: 700, color: (selectedAtt.entropy || 0) > 7.4 ? "var(--danger)" : "var(--success)" }}>
                  {selectedAtt.entropy ? `${selectedAtt.entropy.toFixed(3)} / 8.00` : "Not calculated"}
                </span>
              </div>
              {selectedAtt.risk_flags && (
                <div style={{ fontSize: 11, color: "var(--danger)", background: "rgba(239,68,68,0.08)", padding: "6px 8px", borderRadius: "var(--r-sm)" }}>
                  ⚠️ {selectedAtt.risk_flags}
                </div>
              )}
            </div>

            <div style={{ marginBottom: 16 }}>
              <div className="label">PARENT EMAIL CONTEXT</div>
              <div style={{ fontSize: 12, color: "var(--text-1)", marginBottom: 4 }}>
                <strong>Subject:</strong> {selectedAtt.email_subject || "(No Subject)"}
              </div>
              <div style={{ fontSize: 11, color: "var(--text-3)", fontFamily: "var(--mono)", marginBottom: 4 }}>
                <strong>From:</strong> {selectedAtt.email_from}
              </div>
              <div style={{ fontSize: 11, color: "var(--text-3)" }}>
                <strong>Date:</strong> {selectedAtt.email_date || "Unknown"}
              </div>
            </div>

            <div className="row gap-2" style={{ flexWrap: "wrap" }}>
              <button 
                className="btn btn-primary btn-sm" 
                style={{ flex: 1, minWidth: 100 }}
                onClick={() => handleOpenSystem(selectedAtt.id, selectedAtt.filename)}
                title="Open file with system viewer"
              >
                👁️ Open File
              </button>
              <button 
                className="btn btn-ghost btn-sm" 
                style={{ padding: "4px 8px" }}
                onClick={() => handleRevealFinder(selectedAtt.id)}
                title="Reveal file in Finder / Explorer"
              >
                📂 Finder
              </button>
              <button 
                className="btn btn-ghost btn-sm" 
                style={{ padding: "4px 8px" }}
                onClick={() => exportSingleAttachment(selectedAtt)}
                title="Export to Downloads"
              >
                📥 Export
              </button>
              {onSelectEmail && (
                <button 
                  className="btn btn-ghost btn-sm" 
                  style={{ padding: "4px 8px" }}
                  onClick={() => onSelectEmail(selectedAtt.email_id)}
                >
                  ✉️ Email
                </button>
              )}
            </div>
          </div>
        )}
      </div>
    </div>
  );
}

function AttachmentThumbnail({ 
  attachmentId, 
  storedPath,
  filename, 
  category, 
  onZoom 
}: { 
  attachmentId: string; 
  storedPath?: string | null;
  filename: string; 
  category: string; 
  onZoom?: (src: string) => void;
}) {
  const [src, setSrc] = useState<string | null>(null);
  const [loading, setLoading] = useState(category === "images");

  useEffect(() => {
    if (category === "images") {
      invoke<string | null>("get_attachment_preview", { 
        input: { 
          attachment_id: attachmentId,
          stored_path: storedPath 
        } 
      })
        .then((data) => {
          if (data) setSrc(data);
        })
        .catch(console.error)
        .finally(() => setLoading(false));
    }
  }, [attachmentId, storedPath, category]);

  if (loading) {
    return (
      <div style={{ width: "100%", height: "100%", background: "var(--bg-0)", borderRadius: 6, display: "flex", alignItems: "center", justifyContent: "center", color: "var(--text-3)" }}>
        <span style={{ fontSize: 11 }}>Loading...</span>
      </div>
    );
  }

  if (src) {
    return (
      <div 
        style={{ width: "100%", height: "100%", position: "relative", overflow: "hidden", borderRadius: 6, cursor: "zoom-in" }}
        onClick={(e) => { e.stopPropagation(); onZoom?.(src); }}
        title="Click to zoom image"
      >
        <img 
          src={src} 
          alt={filename} 
          style={{ width: "100%", height: "100%", objectFit: "cover" }} 
        />
        <span style={{ position: "absolute", bottom: 4, right: 4, background: "rgba(0,0,0,0.6)", color: "#fff", padding: "2px 6px", borderRadius: 4, fontSize: 9 }}>
          🔍 Zoom
        </span>
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

function InspectorPhotoViewer({ 
  attachmentId, 
  storedPath,
  filename, 
  onZoom 
}: { 
  attachmentId: string; 
  storedPath?: string | null;
  filename: string; 
  onZoom?: (src: string) => void;
}) {
  const [src, setSrc] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    invoke<string | null>("get_attachment_preview", { 
      input: { 
        attachment_id: attachmentId,
        stored_path: storedPath 
      } 
    })
      .then((data) => {
        if (data) setSrc(data);
      })
      .catch(console.error)
      .finally(() => setLoading(false));
  }, [attachmentId, storedPath]);

  if (loading) {
    return <div style={{ height: 160, display: "flex", alignItems: "center", justifyContent: "center", color: "var(--text-3)", fontSize: 12 }}>Loading preview...</div>;
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
