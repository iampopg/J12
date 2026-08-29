import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { EmailTag } from "./types";

interface Props {
  emailId: string;
  caseId: string;
  tags: EmailTag[];
  onTagsChanged: () => void;
}

export function EmailNotesAndTagsTab({
  emailId,
  caseId,
  tags,
  onTagsChanged,
}: Props) {
  const [notes, setNotes] = useState<any[]>([]);
  const [newNote, setNewNote] = useState("");
  const [customTag, setCustomTag] = useState("");
  const [loading, setLoading] = useState(true);
  const [savingNote, setSavingNote] = useState(false);

  const PRESET_TAGS = [
    { name: "Key Evidence", color: "#ef4444" },
    { name: "Privileged", color: "#8b5cf6" },
    { name: "Hot", color: "#f97316" },
    { name: "Responsive", color: "#22c55e" },
    { name: "Suspicious", color: "#eab308" },
    { name: "Reviewed", color: "#3b82f6" },
  ];

  const loadNotes = async () => {
    setLoading(true);
    try {
      const emailNotes = await invoke<any[]>("email_notes_list", { emailId });
      setNotes(emailNotes);
    } catch (e) {
      console.error("Failed to load email notes:", e);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    loadNotes();
  }, [emailId]);

  const handleToggleTag = async (tagName: string, color?: string) => {
    const existing = tags.find((t) => t.tag.toLowerCase() === tagName.toLowerCase());
    try {
      if (existing) {
        await invoke("email_tag_remove", {
          input: { case_id: caseId, email_id: emailId, tag: existing.tag },
        });
      } else {
        await invoke("email_tag_add", {
          input: {
            case_id: caseId,
            email_id: emailId,
            tag: tagName,
            color: color || "#3b82f6",
            created_by: "Investigator",
          },
        });
      }
      onTagsChanged();
    } catch (err: any) {
      alert(`Error updating tag: ${err}`);
    }
  };

  const handleAddCustomTag = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!customTag.trim()) return;
    await handleToggleTag(customTag.trim(), "#3b82f6");
    setCustomTag("");
  };

  const handleAddNote = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!newNote.trim()) return;
    setSavingNote(true);
    try {
      await invoke("email_note_add", {
        input: {
          case_id: caseId,
          email_id: emailId,
          content: newNote.trim(),
          author: "Investigator",
        },
      });
      setNewNote("");
      loadNotes();
    } catch (err: any) {
      alert(`Error saving email note: ${err}`);
    } finally {
      setSavingNote(false);
    }
  };

  const handleDeleteNote = async (noteId: string) => {
    if (!confirm("Delete this email note?")) return;
    try {
      await invoke("email_note_delete", { noteId });
      loadNotes();
    } catch (err: any) {
      alert(`Error deleting note: ${err}`);
    }
  };

  return (
    <div>
      {/* Tagging Section */}
      <div className="mb-4">
        <h4 style={{ fontSize: 14, fontWeight: 600, color: "var(--text-0)", marginBottom: 8 }}>
          Forensic Email Tags
        </h4>
        <p className="muted mb-4" style={{ fontSize: 12 }}>
          Click a preset tag to toggle it on/off for this email message.
        </p>

        <div className="row gap-2 mb-4" style={{ flexWrap: "wrap" }}>
          {PRESET_TAGS.map((pt) => {
            const active = tags.some((t) => t.tag.toLowerCase() === pt.name.toLowerCase());
            return (
              <button
                key={pt.name}
                className="btn btn-sm"
                style={{
                  background: active ? pt.color : "var(--bg-3)",
                  color: active ? "#fff" : "var(--text-1)",
                  border: `1px solid ${active ? pt.color : "var(--border)"}`,
                  fontWeight: active ? 600 : 400,
                  fontSize: 12,
                  padding: "5px 12px",
                }}
                onClick={() => handleToggleTag(pt.name, pt.color)}
              >
                {active ? "✓ " : "+ "}
                {pt.name}
              </button>
            );
          })}
        </div>

        {/* Custom Tag Input */}
        <form onSubmit={handleAddCustomTag} className="row gap-2" style={{ maxWidth: 360 }}>
          <input
            className="input"
            style={{ padding: "6px 12px", fontSize: 12 }}
            placeholder="Add custom tag (e.g. 'Accounting Lead')..."
            value={customTag}
            onChange={(e) => setCustomTag(e.target.value)}
          />
          <button type="submit" className="btn btn-ghost btn-sm" disabled={!customTag.trim()}>
            + Add Tag
          </button>
        </form>
      </div>

      <hr style={{ borderColor: "var(--border)", margin: "24px 0" }} />

      {/* Email Specific Notes */}
      <div>
        <h4 style={{ fontSize: 14, fontWeight: 600, color: "var(--text-0)", marginBottom: 8 }}>
          Investigator Observations for this Email
        </h4>
        <p className="muted mb-4" style={{ fontSize: 12 }}>
          Recorded notes are timestamped and tied directly to this email message.
        </p>

        {/* Add Note Form */}
        <form onSubmit={handleAddNote} className="mb-4">
          <textarea
            className="textarea"
            placeholder="Write an observation regarding this email's contents, headers, or relevance..."
            value={newNote}
            onChange={(e) => setNewNote(e.target.value)}
            style={{ minHeight: 80, marginBottom: 8 }}
          />
          <button
            type="submit"
            className="btn btn-primary btn-sm"
            disabled={savingNote || !newNote.trim()}
          >
            {savingNote ? "Saving..." : "+ Record Email Note"}
          </button>
        </form>

        {/* Notes List */}
        {loading ? (
          <div className="empty">Loading email notes...</div>
        ) : notes.length === 0 ? (
          <div
            style={{
              padding: 16,
              background: "var(--bg-0)",
              borderRadius: "var(--r-md)",
              border: "1px solid var(--border)",
              color: "var(--text-3)",
              fontSize: 12,
              textAlign: "center",
            }}
          >
            No specific notes recorded for this email yet.
          </div>
        ) : (
          <div style={{ display: "flex", flexDirection: "column", gap: 10 }}>
            {notes.map((n) => (
              <div
                key={n.id}
                style={{
                  padding: 14,
                  background: "var(--bg-0)",
                  borderRadius: "var(--r-md)",
                  border: "1px solid var(--border)",
                }}
              >
                <div className="row between mb-2">
                  <div style={{ fontSize: 11, color: "var(--text-3)" }}>
                    <strong style={{ color: "var(--text-1)" }}>{n.author}</strong> ·{" "}
                    {new Date(n.created_at).toLocaleString()}
                  </div>
                  <button
                    className="btn btn-ghost btn-sm"
                    style={{ padding: "2px 6px", fontSize: 10, color: "var(--danger)" }}
                    onClick={() => handleDeleteNote(n.id)}
                  >
                    Delete
                  </button>
                </div>
                <div
                  style={{
                    fontSize: 13,
                    color: "var(--text-1)",
                    whiteSpace: "pre-wrap",
                    lineHeight: 1.5,
                  }}
                >
                  {n.content}
                </div>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
