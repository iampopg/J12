import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { EmailDetailModal, EmailModalData } from "../components/EmailDetailModal";
import {
  CaseAttachmentItem,
  AttachmentsProps,
} from "./attachments/types";
import { AttachmentsTable } from "./attachments/AttachmentsTable";
import { AttachmentsGrid } from "./attachments/AttachmentsGrid";
import { AttachmentInspectorDrawer } from "./attachments/AttachmentInspectorDrawer";

export function AttachmentsView({ caseId, evidenceFilter }: AttachmentsProps) {
  const [attachments, setAttachments] = useState<CaseAttachmentItem[]>([]);
  const [category, setCategory] = useState<string>("all");
  const [search, setSearch] = useState<string>("");
  const [loading, setLoading] = useState<boolean>(true);
  const [viewMode, setViewMode] = useState<"table" | "grid">("table");
  const [selectedAtt, setSelectedAtt] = useState<CaseAttachmentItem | null>(null);
  const [zoomImage, setZoomImage] = useState<{ src: string; filename: string } | null>(null);
  const [previewEmail, setPreviewEmail] = useState<EmailModalData | null>(null);
  const [toastMessage, setToastMessage] = useState<string | null>(null);
  const [counts, setCounts] = useState<{
    all: number;
    dangerous: number;
    documents: number;
    images: number;
    archives: number;
    media: number;
  }>({
    all: 0,
    dangerous: 0,
    documents: 0,
    images: 0,
    archives: 0,
    media: 0,
  });

  const showToast = (msg: string) => {
    setToastMessage(msg);
    setTimeout(() => setToastMessage(null), 3000);
  };

  useEffect(() => {
    loadSummary();
  }, [caseId, evidenceFilter]);

  useEffect(() => {
    loadData();
  }, [caseId, evidenceFilter, category]);

  const loadSummary = async () => {
    try {
      const c = await invoke<{
        all: number;
        dangerous: number;
        documents: number;
        images: number;
        archives: number;
        media: number;
      }>("case_attachments_summary", { input: { case_id: caseId, evidence_id: evidenceFilter || undefined } });
      if (c) {
        setCounts(c);
      }
    } catch (e) {
      console.error("Failed to load attachment summary counts:", e);
    }
  };

  const openEmailModal = async (emailId: string) => {
    if (!emailId) return;
    try {
      const em = await invoke<EmailModalData | null>("email_get", { input: { id: emailId } });
      if (em) {
        setPreviewEmail(em);
      } else {
        showToast("⚠️ Could not load email content");
      }
    } catch (e) {
      console.error("Failed to fetch email:", e);
      showToast("❌ Error loading email");
    }
  };

  const loadData = async () => {
    setLoading(true);
    try {
      const list = await invoke<CaseAttachmentItem[]>("case_attachments_list", {
        input: { case_id: caseId, evidence_id: evidenceFilter || undefined, category, search }
      });
      setAttachments(list);
    } catch (e) {
      console.error("Failed to load attachments:", e);
    } finally {
      setLoading(false);
    }
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

  const [batchExtracting, setBatchExtracting] = useState(false);

  const handleBatchExtract = async () => {
    setBatchExtracting(true);
    try {
      const res = await invoke<{ total_processed: number; successful: number; failed: number }>("batch_extract_case_attachments", {
        caseId: caseId,
      });
      showToast(`✓ Processed ${res.total_processed} attachments (${res.successful} extracted)`);
      loadData();
    } catch (e: any) {
      showToast(`❌ Batch extraction error: ${e}`);
    } finally {
      setBatchExtracting(false);
    }
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
            Forensic catalog of all extracted files, deep document text parsing (PDF/DOCX/XLSX), image OCR, and cryptographic hashes.
          </p>
        </div>
        <div className="row gap-2">
          <button
            type="button"
            className="btn btn-primary btn-sm"
            style={{ fontWeight: 700, fontSize: 11.5 }}
            onClick={handleBatchExtract}
            disabled={batchExtracting}
          >
            {batchExtracting ? "⚡ Extracting All..." : "⚡ Batch OCR / Parse All"}
          </button>
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
          <button className="btn btn-ghost btn-sm" onClick={() => { loadSummary(); loadData(); }}>
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

      {/* Filter & Search Bar */}
      <div className="card mb-4" style={{ padding: "12px 16px" }}>
        <form onSubmit={(e) => { e.preventDefault(); loadData(); }} className="row between gap-4">
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
        <div>
          {attachments.length === 0 && !loading ? (
            <div className="card empty" style={{ padding: 40 }}>
              No attachments found matching the current filter.
            </div>
          ) : viewMode === "grid" ? (
            <AttachmentsGrid
              caseId={caseId}
              attachments={attachments}
              selectedAtt={selectedAtt}
              onSelectAttachment={setSelectedAtt}
              onZoom={(src, filename) => setZoomImage({ src, filename })}
              onOpenSystem={handleOpenSystem}
              onExport={exportSingleAttachment}
              onOpenEmailModal={openEmailModal}
            />
          ) : (
            <AttachmentsTable
              caseId={caseId}
              attachments={attachments}
              selectedAtt={selectedAtt}
              onSelectAttachment={setSelectedAtt}
              onOpenSystem={handleOpenSystem}
              onExport={exportSingleAttachment}
              onOpenEmailModal={openEmailModal}
            />
          )}
        </div>

        {selectedAtt && (
          <AttachmentInspectorDrawer
            caseId={caseId}
            selectedAtt={selectedAtt}
            onClose={() => setSelectedAtt(null)}
            onZoom={(src, filename) => setZoomImage({ src, filename })}
            onCopyHash={(hash) => {
              navigator.clipboard.writeText(hash);
              showToast("✓ Copied SHA-256 hash to clipboard");
            }}
            onOpenSystem={handleOpenSystem}
            onRevealFinder={handleRevealFinder}
            onExport={exportSingleAttachment}
            onOpenEmailModal={openEmailModal}
          />
        )}
      </div>

      {previewEmail && (
        <EmailDetailModal
          email={previewEmail}
          onClose={() => setPreviewEmail(null)}
          titleSuffix="Return to Attachments"
        />
      )}
    </div>
  );
}
