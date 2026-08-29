import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { RichEmailBodyViewer, CaseAttachmentItem } from "./RichEmailBodyViewer";

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

export interface EmailTag {
  id: string;
  case_id: string;
  email_id: string;
  tag: string;
  color: string;
  created_by?: string;
  created_at: string;
}

interface Props {
  email: EmailModalData | null;
  onClose: () => void;
  titleSuffix?: string;
  onTagsChanged?: () => void;
}

const PRESET_TAGS = [
  { name: "Key Evidence", color: "#ef4444" },
  { name: "Privileged", color: "#8b5cf6" },
  { name: "Hot", color: "#f97316" },
  { name: "Responsive", color: "#22c55e" },
  { name: "Suspicious", color: "#eab308" },
  { name: "Reviewed", color: "#3b82f6" },
];

export function EmailDetailModal({ email, onClose, titleSuffix = "Return", onTagsChanged }: Props) {
  const [attachments, setAttachments] = useState<CaseAttachmentItem[]>([]);
  const [_loadingAtts, setLoadingAtts] = useState<boolean>(false);
  const [tags, setTags] = useState<EmailTag[]>([]);
  const [customTag, setCustomTag] = useState<string>("");
  const [toastMessage, setToastMessage] = useState<string | null>(null);

  const showToast = (msg: string) => {
    setToastMessage(msg);
    setTimeout(() => setToastMessage(null), 3000);
  };

  const loadTags = async () => {
    if (!email?.id) return;
    try {
      const res = await invoke<EmailTag[]>("email_tags_list", { email_id: email.id });
      setTags(res || []);
    } catch (e) {
      console.error("Failed to load email tags:", e);
    }
  };

  useEffect(() => {
    if (!email?.id) {
      setAttachments([]);
      setTags([]);
      return;
    }

    let isMounted = true;
    setLoadingAtts(true);

    invoke<any[]>("email_attachments", { input: { email_id: email.id } })
      .then((res) => {
        if (isMounted) {
          setAttachments(res || []);
        }
      })
      .catch((err) => {
        console.error("Failed to load attachments for email:", err);
      })
      .finally(() => {
        if (isMounted) setLoadingAtts(false);
      });

    loadTags();

    return () => {
      isMounted = false;
    };
  }, [email?.id]);

  if (!email) return null;

  const handleToggleTag = async (tagName: string, color?: string) => {
    const existing = tags.find((t) => t.tag.toLowerCase() === tagName.toLowerCase());
    try {
      if (existing) {
        await invoke("email_tag_remove", {
          input: { email_id: email.id, tag: existing.tag },
        });
        showToast(`🏷️ Removed tag: ${tagName}`);
      } else {
        await invoke("email_tag_add", {
          input: {
            case_id: email.case_id || "",
            email_id: email.id,
            tag: tagName,
            color: color || "#3b82f6",
            created_by: "Investigator",
          },
        });
        showToast(`✓ Tagged as: ${tagName}`);
      }
      await loadTags();
      onTagsChanged?.();
    } catch (err: any) {
      showToast(`❌ Error updating tag: ${err}`);
    }
  };

  const handleAddCustomTag = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!customTag.trim()) return;
    await handleToggleTag(customTag.trim(), "#3b82f6");
    setCustomTag("");
  };

  const handleOpenAttachment = async (attId: string) => {
    try {
      await invoke("open_attachment_in_system", { input: { attachment_id: attId } });
      showToast("✓ Opened in native system viewer");
    } catch (e) {
      console.error(e);
      showToast(`❌ Could not open file: ${e}`);
    }
  };

  const handleExportAttachment = async (attId: string) => {
    try {
      const path = await invoke<string>("export_attachment", {
        input: { attachment_id: attId, destination_dir: "/Users/macbookpro/Downloads" }
      });
      showToast(`📥 Exported to: ${path}`);
    } catch (e) {
      console.error(e);
      showToast(`❌ Export failed: ${e}`);
    }
  };

  return (
    <div
      style={{
        position: "fixed",
        inset: 0,
        background: "rgba(0, 0, 0, 0.85)",
        backdropFilter: "blur(8px)",
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
        zIndex: 10000,
        padding: 24,
      }}
      onClick={onClose}
    >
      {toastMessage && (
        <div
          style={{
            position: "fixed",
            bottom: 30,
            right: 30,
            background: "#0284c7",
            color: "#fff",
            padding: "10px 20px",
            borderRadius: "var(--r-sm)",
            boxShadow: "0 10px 25px rgba(0,0,0,0.5)",
            zIndex: 110000,
            fontSize: 13,
            fontWeight: 600,
          }}
        >
          {toastMessage}
        </div>
      )}

      <div
        style={{
          background: "#0f172a",
          border: "1px solid #334155",
          borderRadius: "var(--r-md)",
          width: "100%",
          maxWidth: "min(1100px, 95vw)",
          maxHeight: "92vh",
          overflowY: "auto",
          padding: 24,
          boxShadow: "0 25px 60px rgba(0,0,0,0.85)",
        }}
        onClick={(e) => e.stopPropagation()}
      >
        <div className="row between mb-3" style={{ borderBottom: "1px solid #1e293b", paddingBottom: 12 }}>
          <div className="row gap-2" style={{ alignItems: "center", overflow: "hidden", minWidth: 0, flex: 1 }}>
            <span style={{ fontSize: 20 }}>✉️</span>
            <div style={{ overflow: "hidden", minWidth: 0 }}>
              <h3 style={{ fontSize: 16, fontWeight: 700, margin: 0, color: "#f8fafc", overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
                {email.subject || "(No Subject)"}
              </h3>
              <div className="row gap-2" style={{ alignItems: "center", marginTop: 2 }}>
                <span className="muted" style={{ fontSize: 11 }}>Message ID: {email.message_id || email.id}</span>
                {tags.map((t) => (
                  <span
                    key={t.id}
                    className="badge"
                    style={{
                      background: t.color,
                      color: "#fff",
                      fontSize: 10,
                      fontWeight: 600,
                      padding: "1px 6px",
                      borderRadius: 4,
                    }}
                  >
                    🏷️ {t.tag}
                  </span>
                ))}
              </div>
            </div>
          </div>
          <button className="btn btn-ghost btn-sm" style={{ flexShrink: 0, marginLeft: 12 }} onClick={onClose}>
            ✕ Close &amp; {titleSuffix}
          </button>
        </div>

        {/* Evidence Tagging Bar */}
        <div
          style={{
            background: "#1e293b",
            border: "1px solid #334155",
            borderRadius: "var(--r-sm)",
            padding: "10px 14px",
            marginBottom: 16,
            display: "flex",
            flexDirection: "column",
            gap: 8,
          }}
        >
          <div className="row between" style={{ alignItems: "center" }}>
            <span style={{ fontSize: 11, fontWeight: 700, color: "#94a3b8", textTransform: "uppercase", letterSpacing: "0.5px" }}>
              🏷️ Evidence Tags &amp; Classification
            </span>
            <span className="muted" style={{ fontSize: 10 }}>Click to toggle tag on/off</span>
          </div>

          <div className="row gap-2" style={{ flexWrap: "wrap", alignItems: "center" }}>
            {PRESET_TAGS.map((pt) => {
              const isApplied = tags.some((t) => t.tag.toLowerCase() === pt.name.toLowerCase());
              return (
                <button
                  key={pt.name}
                  type="button"
                  className="btn btn-sm"
                  style={{
                    background: isApplied ? pt.color : "transparent",
                    color: isApplied ? "#fff" : "#94a3b8",
                    border: `1px solid ${isApplied ? pt.color : "#475569"}`,
                    fontWeight: isApplied ? 700 : 500,
                    fontSize: 11,
                    padding: "3px 10px",
                    borderRadius: 14,
                    cursor: "pointer",
                    transition: "all 0.15s ease",
                  }}
                  onClick={() => handleToggleTag(pt.name, pt.color)}
                >
                  {isApplied ? "✓ " : "+ "}
                  {pt.name}
                </button>
              );
            })}

            <form onSubmit={handleAddCustomTag} style={{ display: "flex", gap: 4, marginLeft: "auto" }}>
              <input
                type="text"
                className="input"
                placeholder="Custom tag..."
                style={{ fontSize: 11, padding: "3px 8px", width: 110, height: 26 }}
                value={customTag}
                onChange={(e) => setCustomTag(e.target.value)}
              />
              <button
                type="submit"
                className="btn btn-ghost btn-sm"
                style={{ fontSize: 11, padding: "2px 8px", height: 26 }}
                disabled={!customTag.trim()}
              >
                + Add
              </button>
            </form>
          </div>
        </div>

        {/* Email Metadata Header Table */}
        <div style={{ background: "#1e293b", padding: 14, borderRadius: "var(--r-sm)", marginBottom: 16, fontSize: 12.5, border: "1px solid #334155" }}>
          <div className="row mb-1">
            <span className="muted" style={{ width: 80, fontWeight: 600 }}>From:</span>
            <span style={{ color: "#38bdf8", fontWeight: 600 }}>{email.from_display ? `${email.from_display} <${email.from_addr}>` : email.from_addr}</span>
          </div>
          <div className="row mb-1">
            <span className="muted" style={{ width: 80, fontWeight: 600 }}>To:</span>
            <span style={{ color: "#e2e8f0" }}>
              {(() => {
                try {
                  const p = JSON.parse(email.to_addrs);
                  return Array.isArray(p) ? p.join(", ") : email.to_addrs;
                } catch {
                  return email.to_addrs;
                }
              })()}
            </span>
          </div>
          {email.cc_addrs && (
            <div className="row mb-1">
              <span className="muted" style={{ width: 80, fontWeight: 600 }}>Cc:</span>
              <span style={{ color: "#94a3b8" }}>
                {(() => {
                  try {
                    const p = JSON.parse(email.cc_addrs);
                    return Array.isArray(p) ? p.join(", ") : email.cc_addrs;
                  } catch {
                    return email.cc_addrs;
                  }
                })()}
              </span>
            </div>
          )}
          <div className="row mb-1">
            <span className="muted" style={{ width: 80, fontWeight: 600 }}>Date UTC:</span>
            <span style={{ color: "#94a3b8", fontFamily: "var(--mono)" }}>{email.date_sent_utc || email.date_sent || "Unknown"}</span>
          </div>
          <div className="row">
            <span className="muted" style={{ width: 80, fontWeight: 600 }}>Folder:</span>
            <span className="badge badge-gray">{email.folder_name || email.folder_category || "inbox"}</span>
          </div>
        </div>

        {/* Email Body View */}
        <div style={{ marginBottom: 16 }}>
          <div className="label" style={{ marginBottom: 6 }}>EMAIL MESSAGE CONTENT</div>
          <RichEmailBodyViewer
            bodyText={email.body_text}
            bodyHtml={email.body_html}
            emailId={email.id}
            defaultMode="rendered"
            attachments={attachments}
          />
        </div>

        {/* Attached Files Section */}
        {attachments.length > 0 && (
          <div style={{ marginTop: 16, marginBottom: 16, padding: 14, background: "#1e293b", borderRadius: "var(--r-sm)", border: "1px solid #334155" }}>
            <div className="row between mb-2" style={{ alignItems: "center" }}>
              <span style={{ fontSize: 12, fontWeight: 700, color: "#f8fafc", textTransform: "uppercase" }}>
                📎 Attached Evidence Files ({attachments.length})
              </span>
              <span className="muted" style={{ fontSize: 11 }}>NIST Cryptographic Verification</span>
            </div>
            <div style={{ display: "flex", flexDirection: "column", gap: 8 }}>
              {attachments.map((att) => (
                <div
                  key={att.id}
                  style={{
                    display: "flex",
                    alignItems: "center",
                    justifyContent: "space-between",
                    padding: "8px 12px",
                    background: "#0f172a",
                    border: "1px solid #334155",
                    borderRadius: 4,
                    fontSize: 12,
                  }}
                >
                  <div style={{ display: "flex", alignItems: "center", gap: 8, overflow: "hidden", minWidth: 0 }}>
                    <span style={{ fontSize: 16 }}>📄</span>
                    <div style={{ overflow: "hidden" }}>
                      <div style={{ color: "#f8fafc", fontWeight: 600, textOverflow: "ellipsis", overflow: "hidden", whiteSpace: "nowrap" }}>
                        {att.filename}
                      </div>
                      <div style={{ color: "#64748b", fontSize: 10, fontFamily: "var(--mono)" }}>
                        {att.mime_type} · {(att.size_bytes / 1024).toFixed(1)} KB · SHA256: {att.sha256 ? `${att.sha256.slice(0, 16)}...` : "N/A"}
                      </div>
                    </div>
                  </div>
                  <div className="row gap-1" style={{ flexShrink: 0, marginLeft: 12 }}>
                    <button
                      className="btn btn-sm btn-ghost"
                      style={{ fontSize: 11, padding: "3px 8px" }}
                      onClick={() => handleOpenAttachment(att.id)}
                    >
                      📂 Open
                    </button>
                    <button
                      className="btn btn-sm btn-primary"
                      style={{ fontSize: 11, padding: "3px 8px" }}
                      onClick={() => handleExportAttachment(att.id)}
                    >
                      📥 Export
                    </button>
                  </div>
                </div>
              ))}
            </div>
          </div>
        )}

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
