import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { CaseAttachmentItem, formatSize, getFileIcon, isImagePreviewable } from "./types";
import { InspectorPhotoViewer } from "./InspectorPhotoViewer";
import { BookmarkButton } from "../../components/BookmarkButton";

interface Props {
  caseId: string;
  selectedAtt: CaseAttachmentItem;
  onClose: () => void;
  onZoom: (src: string, filename: string) => void;
  onCopyHash: (hash: string) => void;
  onOpenSystem: (id: string, filename: string) => void;
  onRevealFinder: (id: string) => void;
  onExport: (att: CaseAttachmentItem) => void;
  onOpenEmailModal: (emailId: string) => void;
  onTextExtracted?: (attId: string, text: string) => void;
}

export function AttachmentInspectorDrawer({
  caseId,
  selectedAtt,
  onClose,
  onZoom,
  onCopyHash,
  onOpenSystem,
  onRevealFinder,
  onExport,
  onOpenEmailModal,
  onTextExtracted,
}: Props) {
  const [extracting, setExtracting] = useState(false);
  const [extractedText, setExtractedText] = useState<string | null>(selectedAtt.extracted_text || null);

  const handleExtractText = async () => {
    setExtracting(true);
    try {
      const res = await invoke<{ attachment_id: string; extracted_text: string; ocr_status: string }>("extract_attachment_text", {
        attachmentId: selectedAtt.id,
      });
      if (res?.extracted_text) {
        setExtractedText(res.extracted_text);
        onTextExtracted?.(selectedAtt.id, res.extracted_text);
      }
    } catch (e) {
      console.error("Text extraction failed:", e);
    } finally {
      setExtracting(false);
    }
  };

  return (
    <div className="card" style={{ position: "sticky", top: 16, height: "fit-content", padding: 18, background: "var(--bg-1)", border: "1px solid var(--border)" }}>
      <div className="row between mb-3" style={{ alignItems: "center" }}>
        <h3 style={{ fontSize: 14, fontWeight: 700, color: "var(--text-0)", margin: 0 }}>
          {getFileIcon(selectedAtt.category, selectedAtt.filename)} File Metadata Dossier
        </h3>
        <div className="row gap-1" style={{ alignItems: "center" }}>
          <BookmarkButton
            caseId={caseId}
            itemId={selectedAtt.id}
            itemType="attachment"
            compact={true}
          />
          <button className="btn btn-ghost btn-sm" onClick={onClose}>✕</button>
        </div>
      </div>

      {/* Photo Preview in Inspector */}
      {isImagePreviewable(selectedAtt.category, selectedAtt.filename) && (
        <div style={{ marginBottom: 14, borderRadius: "var(--r-md)", overflow: "hidden", border: "1px solid var(--border)", background: "#000" }}>
          <InspectorPhotoViewer 
            attachmentId={selectedAtt.id} 
            storedPath={selectedAtt.stored_path}
            filename={selectedAtt.filename} 
            onZoom={(src) => onZoom(src, selectedAtt.filename)}
          />
        </div>
      )}

      <div style={{ marginBottom: 12 }}>
        <div className="label">FILENAME</div>
        <div style={{ fontSize: 13.5, fontWeight: 700, color: "var(--text-0)", wordBreak: "break-all" }}>
          {selectedAtt.filename}
        </div>
        <div style={{ fontSize: 11, color: "var(--text-3)", fontFamily: "var(--mono)", marginTop: 2 }}>
          Category: <strong>{selectedAtt.category.toUpperCase()}</strong> · {formatSize(selectedAtt.size_bytes)}
        </div>
      </div>

      {/* Extracted Document Text & OCR Section */}
      <div style={{ marginBottom: 14, background: "var(--bg-2)", padding: 12, borderRadius: "var(--r-sm)", border: "1px solid var(--border)" }}>
        <div className="row between mb-2" style={{ alignItems: "center" }}>
          <div className="label" style={{ margin: 0 }}>📄 DOCUMENT TEXT &amp; OCR</div>
          <button
            type="button"
            className="btn btn-primary btn-sm"
            style={{ fontSize: 10.5, padding: "2px 8px", fontWeight: 700 }}
            onClick={handleExtractText}
            disabled={extracting}
          >
            {extracting ? "Scanning..." : extractedText ? "⚡ Re-Extract" : "⚡ Extract Text"}
          </button>
        </div>

        {extractedText ? (
          <div>
            <pre
              style={{
                background: "var(--bg-0)",
                border: "1px solid var(--border)",
                borderRadius: "var(--r-xs)",
                padding: 8,
                fontSize: 11,
                maxHeight: 140,
                overflowY: "auto",
                whiteSpace: "pre-wrap",
                wordBreak: "break-word",
                color: "var(--text-1)",
                fontFamily: "var(--mono)",
              }}
            >
              {extractedText}
            </pre>
            <div className="row between mt-1" style={{ fontSize: 10, color: "var(--text-3)" }}>
              <span>{extractedText.length.toLocaleString()} characters indexed</span>
              <button
                type="button"
                className="btn btn-ghost btn-sm"
                style={{ fontSize: 10, padding: "1px 5px" }}
                onClick={() => navigator.clipboard.writeText(extractedText)}
              >
                📋 Copy Text
              </button>
            </div>
          </div>
        ) : (
          <div style={{ fontSize: 11, color: "var(--text-3)", fontStyle: "italic" }}>
            Click 'Extract Text' to parse PDF/DOCX/XLSX or run image OCR.
          </div>
        )}
      </div>

      <div style={{ marginBottom: 12 }}>
        <div className="label">CRYPTOGRAPHIC SHA-256 HASH</div>
        <div 
          style={{
            background: "var(--bg-3)",
            border: "1px solid var(--border)",
            borderRadius: "var(--r-sm)",
            padding: "6px 8px",
            fontFamily: "var(--mono)",
            fontSize: 10,
            color: "#38bdf8",
            wordBreak: "break-all"
          }}
        >
          {selectedAtt.sha256}
        </div>
      </div>

      <div style={{ marginBottom: 12 }}>
        <div className="label">PARENT EMAIL CONTEXT</div>
        <div style={{ fontSize: 11.5, color: "var(--text-1)", marginBottom: 2 }}>
          <strong>Subject:</strong> {selectedAtt.email_subject || "(No Subject)"}
        </div>
        <div style={{ fontSize: 10.5, color: "var(--text-3)", fontFamily: "var(--mono)" }}>
          <strong>From:</strong> {selectedAtt.email_from}
        </div>
      </div>

      <div className="row gap-2" style={{ flexWrap: "wrap" }}>
        <button className="btn btn-secondary btn-sm" style={{ flex: 1, fontSize: 11 }} onClick={() => onOpenEmailModal(selectedAtt.email_id)}>
          ✉️ View Email
        </button>
        <button className="btn btn-primary btn-sm" style={{ flex: 1, fontSize: 11 }} onClick={() => onExport(selectedAtt)}>
          💾 Export File
        </button>
        <button className="btn btn-ghost btn-sm" style={{ flex: 1, fontSize: 11 }} onClick={() => onOpenSystem(selectedAtt.id, selectedAtt.filename)}>
          ⚡ Open System
        </button>
      </div>
    </div>
  );
}
