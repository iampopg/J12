import { CaseAttachmentItem, formatSize, getFileIcon } from "./types";
import { BookmarkButton } from "../../components/BookmarkButton";

interface Props {
  caseId: string;
  attachments: CaseAttachmentItem[];
  selectedAtt: CaseAttachmentItem | null;
  onSelectAttachment: (att: CaseAttachmentItem) => void;
  onOpenSystem: (id: string, filename: string) => void;
  onExport: (att: CaseAttachmentItem) => void;
  onOpenEmailModal: (emailId: string) => void;
}

export function AttachmentsTable({
  caseId,
  attachments,
  selectedAtt,
  onSelectAttachment,
  onOpenSystem,
  onExport,
  onOpenEmailModal,
}: Props) {
  return (
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
                onClick={() => onSelectAttachment(att)}
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
                  <div className="row gap-1" style={{ justifyContent: "flex-end", alignItems: "center" }} onClick={(e) => e.stopPropagation()}>
                    <BookmarkButton
                      caseId={caseId}
                      itemId={att.id}
                      itemType="attachment"
                      compact={true}
                    />
                    <button 
                      className="btn btn-primary btn-sm" 
                      style={{ padding: "3px 8px", fontSize: 11 }}
                      onClick={() => onOpenSystem(att.id, att.filename)}
                      title="Open file in default system viewer"
                    >
                      👁️ Open
                    </button>
                    <button 
                      className="btn btn-ghost btn-sm" 
                      style={{ padding: "3px 8px", fontSize: 11 }}
                      onClick={() => onExport(att)}
                      title="Export attachment to Downloads"
                    >
                      📥 Export
                    </button>
                    <button 
                      className="btn btn-ghost btn-sm" 
                      style={{ padding: "3px 8px", fontSize: 11 }}
                      onClick={() => onOpenEmailModal(att.email_id)}
                      title="Open parent email in popup"
                    >
                      ✉️ Email
                    </button>
                  </div>
                </td>
              </tr>
            );
          })}
        </tbody>
      </table>
    </div>
  );
}
