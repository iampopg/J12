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
  const [selectedAtt, setSelectedAtt] = useState<CaseAttachmentItem | null>(null);
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
            Forensic catalog of all extracted files, cryptographic hashes, entropy levels, and dangerous executable lures.
          </p>
        </div>
        <div className="row gap-2">
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
          className={`btn ${category === "documents" ? "btn-primary" : "btn-ghost"} btn-sm`}
          onClick={() => setCategory("documents")}
        >
          📄 Documents &amp; PDFs ({counts.documents})
        </button>
        <button
          className={`btn ${category === "images" ? "btn-primary" : "btn-ghost"} btn-sm`}
          onClick={() => setCategory("images")}
        >
          🖼️ Images &amp; Scans ({counts.images})
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

      {/* Filter & Search Bar */}
      <div className="card mb-4" style={{ padding: "12px 16px" }}>
        <form onSubmit={handleSearchSubmit} className="row between gap-4">
          <div className="row gap-2" style={{ flex: 1 }}>
            <input
              className="input"
              style={{ flex: 1, padding: "8px 12px", fontSize: 13 }}
              placeholder="Search by filename, extension (.exe, .pdf, .zip), or SHA-256 hash..."
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
        {/* Table View */}
        <div className="card" style={{ padding: 0, overflow: "hidden" }}>
          {loading ? (
            <div className="empty" style={{ padding: 32 }}>Loading and verifying attachment signatures...</div>
          ) : attachments.length === 0 ? (
            <div className="empty" style={{ padding: 32 }}>
              No attachments found matching the current filter.
            </div>
          ) : (
            <table style={{ width: "100%", borderCollapse: "collapse", textAlign: "left" }}>
              <thead>
                <tr style={{ background: "var(--bg-1)", borderBottom: "1px solid var(--border)" }}>
                  <th className="th" style={{ width: 40 }}></th>
                  <th className="th">Filename &amp; Type</th>
                  <th className="th">Size</th>
                  <th className="th">SHA-256 Hash</th>
                  <th className="th">Entropy &amp; Risk</th>
                  <th className="th">Source Email</th>
                  <th className="th" style={{ textAlign: "right" }}>Actions</th>
                </tr>
              </thead>
              <tbody>
                {attachments.map((att) => {
                  const isDangerous = att.category === "dangerous";
                  const entropyVal = att.entropy || 0;
                  const isSelected = selectedAtt?.id === att.id;

                  return (
                    <tr 
                      key={att.id}
                      className="tr tr-click"
                      style={{ 
                        background: isSelected ? "var(--bg-3)" : isDangerous ? "rgba(239, 68, 68, 0.05)" : undefined,
                        borderBottom: "1px solid var(--border)"
                      }}
                      onClick={() => setSelectedAtt(att)}
                    >
                      <td className="td" style={{ fontSize: 18, textAlign: "center" }}>
                        {getFileIcon(att.category, att.filename)}
                      </td>
                      <td className="td">
                        <div style={{ fontWeight: 600, color: isDangerous ? "var(--danger)" : "var(--text-0)" }}>
                          {att.filename}
                        </div>
                        <div style={{ fontSize: 11, color: "var(--text-3)", fontFamily: "var(--mono)" }}>
                          {att.mime_type}
                        </div>
                      </td>
                      <td className="td" style={{ fontFamily: "var(--mono)", fontSize: 12 }}>
                        {formatSize(att.size_bytes)}
                      </td>
                      <td className="td">
                        <div 
                          style={{ fontFamily: "var(--mono)", fontSize: 11, color: "var(--accent)", cursor: "pointer" }}
                          onClick={(e) => { e.stopPropagation(); copyToClipboard(att.sha256); }}
                          title="Click to copy SHA-256 hash"
                        >
                          {att.sha256.slice(0, 16)}...
                        </div>
                      </td>
                      <td className="td">
                        <div className="row gap-2" style={{ alignItems: "center" }}>
                          {entropyVal > 0 && (
                            <span 
                              className="badge" 
                              style={{ 
                                background: entropyVal > 7.4 ? "rgba(239,68,68,0.15)" : entropyVal > 6.5 ? "rgba(249,115,22,0.15)" : "rgba(34,197,94,0.15)",
                                color: entropyVal > 7.4 ? "var(--danger)" : entropyVal > 6.5 ? "var(--warning)" : "var(--success)"
                              }}
                            >
                              H: {entropyVal.toFixed(2)}
                            </span>
                          )}
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

            <div className="row gap-2">
              <button 
                className="btn btn-primary btn-sm" 
                style={{ flex: 1 }}
                onClick={() => exportSingleAttachment(selectedAtt)}
              >
                📥 Export to Downloads
              </button>
              {onSelectEmail && (
                <button 
                  className="btn btn-ghost btn-sm" 
                  onClick={() => onSelectEmail(selectedAtt.email_id)}
                >
                  ✉️ Open Email
                </button>
              )}
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
