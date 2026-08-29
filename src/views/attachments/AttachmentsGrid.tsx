import { CaseAttachmentItem, formatSize } from "./types";
import { AttachmentThumbnail } from "./AttachmentThumbnail";
import { BookmarkButton } from "../../components/BookmarkButton";

interface Props {
  caseId: string;
  attachments: CaseAttachmentItem[];
  selectedAtt: CaseAttachmentItem | null;
  onSelectAttachment: (att: CaseAttachmentItem) => void;
  onZoom: (src: string, filename: string) => void;
  onOpenSystem: (id: string, filename: string) => void;
  onExport: (att: CaseAttachmentItem) => void;
  onOpenEmailModal: (emailId: string) => void;
}

export function AttachmentsGrid({
  caseId,
  attachments,
  selectedAtt,
  onSelectAttachment,
  onZoom,
  onOpenSystem,
  onExport,
  onOpenEmailModal,
}: Props) {
  return (
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
          onClick={() => onSelectAttachment(att)}
        >
          <div style={{ width: "100%", height: 140, marginBottom: 10, position: "relative" }}>
            <AttachmentThumbnail 
              attachmentId={att.id} 
              storedPath={att.stored_path}
              filename={att.filename} 
              category={att.category}
              onZoom={(src) => onZoom(src, att.filename)}
            />
          </div>
          <div style={{ fontWeight: 600, fontSize: 13, color: "var(--text-0)", overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }} title={att.filename}>
            {att.filename}
          </div>
          <div className="row between mt-1" style={{ fontSize: 11, color: "var(--text-3)" }}>
            <span>{formatSize(att.size_bytes)}</span>
            <span className="badge" style={{ fontSize: 9 }}>{att.category}</span>
          </div>
          <div className="row gap-1 mt-2" style={{ justifyContent: "flex-end", flexWrap: "wrap", alignItems: "center" }} onClick={(e) => e.stopPropagation()}>
            <BookmarkButton
              caseId={caseId}
              itemId={att.id}
              itemType="attachment"
              compact={true}
            />
            <button
              className="btn btn-primary btn-sm"
              style={{ padding: "2px 6px", fontSize: 10 }}
              onClick={() => onOpenSystem(att.id, att.filename)}
              title="Open file in default system application (Preview / Acrobat / Office)"
            >
              👁️ Open
            </button>
            <button
              className="btn btn-ghost btn-sm"
              style={{ padding: "2px 6px", fontSize: 10 }}
              onClick={() => onExport(att)}
              title="Export to Downloads"
            >
              📥 Export
            </button>
            <button
              className="btn btn-ghost btn-sm"
              style={{ padding: "2px 6px", fontSize: 10 }}
              onClick={() => onOpenEmailModal(att.email_id)}
              title="Open parent email in popup"
            >
              ✉️ Email
            </button>
          </div>
        </div>
      ))}
    </div>
  );
}
