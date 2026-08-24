import { useState, useEffect, useCallback, useMemo } from "react";
import { invoke } from "@tauri-apps/api/core";

export interface CaseNote {
  id: string;
  case_id: string;
  author: string;
  title: string;
  content: string;
  category: string;
  pinned: boolean;
  created_at: string;
  updated_at: string;
}

const CATEGORIES = [
  { id: "all", label: "All Notes" },
  { id: "lead", label: "Key Lead", color: "#f97316", badge: "badge-orange" },
  { id: "observation", label: "Evidence Observation", color: "#3b82f6", badge: "badge-blue" },
  { id: "legal", label: "Legal / Privileged", color: "#a855f7", badge: "badge-purple" },
  { id: "hypothesis", label: "Hypothesis", color: "#eab308", badge: "badge-yellow" },
  { id: "general", label: "General", color: "#94a3b8", badge: "badge-gray" },
];

export function NotesView({ caseId, onNotesCountChange }: { caseId: string; onNotesCountChange?: (count: number) => void }) {
  const [notes, setNotes] = useState<CaseNote[]>([]);
  const [loading, setLoading] = useState(true);
  const [search, setSearch] = useState("");
  const [selectedCategory, setSelectedCategory] = useState("all");
  const [showModal, setShowModal] = useState(false);
  const [editingNote, setEditingNote] = useState<CaseNote | null>(null);

  // Form state
  const [formTitle, setFormTitle] = useState("");
  const [formContent, setFormContent] = useState("");
  const [formCategory, setFormCategory] = useState("general");
  const [formPinned, setFormPinned] = useState(false);
  const [formAuthor, setFormAuthor] = useState("Lead Investigator");
  const [saving, setSaving] = useState(false);
  const [copiedId, setCopiedId] = useState<string | null>(null);

  const loadNotes = useCallback(async () => {
    setLoading(true);
    try {
      const data = await invoke<CaseNote[]>("case_notes_list", { input: { case_id: caseId } });
      setNotes(data);
      if (onNotesCountChange) {
        onNotesCountChange(data.length);
      }
    } catch (e) {
      console.error("Failed to load case notes:", e);
    } finally {
      setLoading(false);
    }
  }, [caseId, onNotesCountChange]);

  useEffect(() => {
    loadNotes();
  }, [loadNotes]);

  const openCreateModal = () => {
    setEditingNote(null);
    setFormTitle("");
    setFormContent("");
    setFormCategory("general");
    setFormPinned(false);
    setShowModal(true);
  };

  const openEditModal = (note: CaseNote) => {
    setEditingNote(note);
    setFormTitle(note.title);
    setFormContent(note.content);
    setFormCategory(note.category);
    setFormPinned(note.pinned);
    setFormAuthor(note.author);
    setShowModal(true);
  };

  const handleSave = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!formTitle.trim() || !formContent.trim()) return;
    setSaving(true);
    try {
      if (editingNote) {
        await invoke("case_note_update", {
          input: {
            id: editingNote.id,
            title: formTitle.trim(),
            content: formContent.trim(),
            category: formCategory,
            pinned: formPinned,
          },
        });
      } else {
        await invoke("case_note_create", {
          input: {
            case_id: caseId,
            author: formAuthor.trim() || "Investigator",
            title: formTitle.trim(),
            content: formContent.trim(),
            category: formCategory,
            pinned: formPinned,
          },
        });
      }
      setShowModal(false);
      loadNotes();
    } catch (err: any) {
      alert(`Error saving note: ${err}`);
    } finally {
      setSaving(false);
    }
  };

  const handleDelete = async (noteId: string) => {
    if (!confirm("Are you sure you want to delete this case note? This action is logged to the forensic audit trail.")) return;
    try {
      await invoke("case_note_delete", { noteId });
      loadNotes();
    } catch (err: any) {
      alert(`Error deleting note: ${err}`);
    }
  };

  const handleTogglePin = async (noteId: string) => {
    try {
      await invoke("case_note_toggle_pin", { noteId });
      loadNotes();
    } catch (err: any) {
      alert(`Error toggling pin: ${err}`);
    }
  };

  const handleCopy = (note: CaseNote) => {
    const text = `[NOTE] ${note.title}\nCategory: ${note.category}\nAuthor: ${note.author} (${new Date(note.created_at).toLocaleString()})\n\n${note.content}`;
    navigator.clipboard.writeText(text);
    setCopiedId(note.id);
    setTimeout(() => setCopiedId(null), 2000);
  };

  const handleExportAll = () => {
    if (notes.length === 0) return;
    const header = `# Case Investigation Notes\nCase ID: ${caseId}\nExported: ${new Date().toLocaleString()}\nTotal Notes: ${notes.length}\n\n========================================\n\n`;
    const body = notes.map((n, i) => (
      `### ${i + 1}. ${n.title} ${n.pinned ? "[PINNED]" : ""}\n- **Category**: ${n.category.toUpperCase()}\n- **Author**: ${n.author}\n- **Date**: ${new Date(n.created_at).toLocaleString()}\n\n${n.content}\n\n----------------------------------------\n`
    )).join("\n");

    const blob = new Blob([header + body], { type: "text/markdown" });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = `case_notes_${caseId.slice(0, 8)}.md`;
    a.click();
    URL.revokeObjectURL(url);
  };

  // Filtered list
  const filteredNotes = useMemo(() => {
    return notes.filter((n) => {
      const matchCat = selectedCategory === "all" || n.category.toLowerCase() === selectedCategory.toLowerCase();
      const matchSearch =
        !search.trim() ||
        n.title.toLowerCase().includes(search.toLowerCase()) ||
        n.content.toLowerCase().includes(search.toLowerCase()) ||
        n.author.toLowerCase().includes(search.toLowerCase());
      return matchCat && matchSearch;
    });
  }, [notes, selectedCategory, search]);

  const getCategoryMeta = (catId: string) => {
    return CATEGORIES.find((c) => c.id === catId.toLowerCase()) || { label: catId, color: "#94a3b8", badge: "badge-gray" };
  };

  return (
    <div>
      {/* Header */}
      <div className="row between mb-4">
        <div>
          <h2 style={{ fontSize: 22, fontWeight: 700, color: "var(--text-0)" }}>Case Notes & Observations</h2>
          <p className="muted">Forensic notes, investigative leads, and hypothesis tracking</p>
        </div>
        <div className="row gap-2">
          {notes.length > 0 && (
            <button className="btn btn-ghost" onClick={handleExportAll} title="Export notes as Markdown">
              📄 Export Notes
            </button>
          )}
          <button className="btn btn-primary" onClick={openCreateModal}>
            + New Case Note
          </button>
        </div>
      </div>

      {/* Filter and Search Bar */}
      <div className="card mb-4" style={{ padding: "14px 18px" }}>
        <div className="row between" style={{ flexWrap: "wrap", gap: 12 }}>
          <div className="row gap-2" style={{ flexWrap: "wrap" }}>
            {CATEGORIES.map((cat) => (
              <button
                key={cat.id}
                className={`btn btn-sm ${selectedCategory === cat.id ? "btn-primary" : "btn-ghost"}`}
                onClick={() => setSelectedCategory(cat.id)}
                style={{ fontSize: 12, padding: "5px 12px" }}
              >
                {cat.label}
                {cat.id === "all" ? (
                  <span style={{ opacity: 0.7, marginLeft: 4 }}>({notes.length})</span>
                ) : (
                  <span style={{ opacity: 0.7, marginLeft: 4 }}>
                    ({notes.filter((n) => n.category.toLowerCase() === cat.id).length})
                  </span>
                )}
              </button>
            ))}
          </div>
          <div style={{ minWidth: 220 }}>
            <input
              className="input"
              style={{ padding: "6px 12px", fontSize: 13 }}
              placeholder="Search notes or authors..."
              value={search}
              onChange={(e) => setSearch(e.target.value)}
            />
          </div>
        </div>
      </div>

      {/* Content Area */}
      {loading ? (
        <div className="empty">Loading case notes...</div>
      ) : filteredNotes.length === 0 ? (
        <div className="card" style={{ textAlign: "center", padding: "60px 40px" }}>
          <div style={{ fontSize: 44, marginBottom: 16 }}>📝</div>
          <h3 style={{ fontSize: 18, color: "var(--text-0)", marginBottom: 8 }}>
            {search || selectedCategory !== "all" ? "No matching notes found" : "No case notes recorded yet"}
          </h3>
          <p className="muted mb-4">
            {search || selectedCategory !== "all"
              ? "Try adjusting your search terms or category filter."
              : "Record investigative leads, evidence observations, or legal notes for this case."}
          </p>
          <button className="btn btn-primary" onClick={openCreateModal}>
            + Add First Case Note
          </button>
        </div>
      ) : (
        <div style={{ display: "grid", gridTemplateColumns: "repeat(auto-fill, minmax(360px, 1fr))", gap: 16 }}>
          {filteredNotes.map((note) => {
            const meta = getCategoryMeta(note.category);
            return (
              <div
                key={note.id}
                className="card"
                style={{
                  margin: 0,
                  display: "flex",
                  flexDirection: "column",
                  justifyContent: "space-between",
                  borderLeft: note.pinned ? `4px solid var(--accent)` : `4px solid ${meta.color}`,
                  background: note.pinned ? "rgba(59,130,246,0.04)" : "var(--bg-2)",
                  boxShadow: note.pinned ? "0 4px 16px rgba(59,130,246,0.12)" : undefined,
                }}
              >
                <div>
                  {/* Top Bar inside Card */}
                  <div className="row between mb-2">
                    <div className="row gap-2">
                      <span
                        className="badge"
                        style={{
                          background: `${meta.color}22`,
                          color: meta.color,
                          border: `1px solid ${meta.color}44`,
                          fontWeight: 600,
                        }}
                      >
                        {meta.label}
                      </span>
                      {note.pinned && (
                        <span className="badge badge-blue" style={{ fontSize: 10, fontWeight: 700 }}>
                          📌 PINNED
                        </span>
                      )}
                    </div>
                    <div className="row gap-2">
                      <button
                        className="btn btn-ghost btn-sm"
                        style={{ padding: "2px 6px", fontSize: 11 }}
                        onClick={() => handleTogglePin(note.id)}
                        title={note.pinned ? "Unpin Note" : "Pin to Top"}
                      >
                        {note.pinned ? "Unpin" : "Pin"}
                      </button>
                      <button
                        className="btn btn-ghost btn-sm"
                        style={{ padding: "2px 6px", fontSize: 11 }}
                        onClick={() => handleCopy(note)}
                        title="Copy note to clipboard"
                      >
                        {copiedId === note.id ? "✓ Copied" : "Copy"}
                      </button>
                    </div>
                  </div>

                  {/* Title */}
                  <h3 style={{ fontSize: 16, fontWeight: 600, color: "var(--text-0)", marginBottom: 8 }}>
                    {note.title}
                  </h3>

                  {/* Body Content */}
                  <div
                    style={{
                      fontSize: 13,
                      color: "var(--text-1)",
                      lineHeight: 1.6,
                      whiteSpace: "pre-wrap",
                      marginBottom: 16,
                      maxHeight: 220,
                      overflowY: "auto",
                    }}
                  >
                    {note.content}
                  </div>
                </div>

                {/* Footer Metadata & Actions */}
                <div
                  style={{
                    paddingTop: 12,
                    borderTop: "1px solid var(--border)",
                    display: "flex",
                    alignItems: "center",
                    justifyContent: "space-between",
                    fontSize: 11,
                    color: "var(--text-3)",
                  }}
                >
                  <div>
                    <span style={{ color: "var(--text-2)", fontWeight: 500 }}>{note.author}</span> ·{" "}
                    <span>{new Date(note.created_at).toLocaleDateString()} {new Date(note.created_at).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })}</span>
                  </div>
                  <div className="row gap-2">
                    <button
                      className="btn btn-ghost btn-sm"
                      style={{ padding: "2px 8px", fontSize: 11 }}
                      onClick={() => openEditModal(note)}
                    >
                      Edit
                    </button>
                    <button
                      className="btn btn-ghost btn-sm"
                      style={{ padding: "2px 8px", fontSize: 11, color: "var(--danger)" }}
                      onClick={() => handleDelete(note.id)}
                    >
                      Delete
                    </button>
                  </div>
                </div>
              </div>
            );
          })}
        </div>
      )}

      {/* Note Creation / Editing Modal */}
      {showModal && (
        <div className="overlay" onClick={() => setShowModal(false)}>
          <div className="panel" style={{ width: 560 }} onClick={(e) => e.stopPropagation()}>
            <div className="panel-hdr">
              <h3 style={{ fontSize: 16, fontWeight: 600, color: "var(--text-0)" }}>
                {editingNote ? "Edit Case Note" : "New Case Note"}
              </h3>
              <button className="btn btn-ghost btn-sm" onClick={() => setShowModal(false)}>
                ✕
              </button>
            </div>
            <form onSubmit={handleSave} className="panel-bdy">
              <div className="mb-4">
                <label className="label">Note Title *</label>
                <input
                  className="input"
                  placeholder="e.g. Lead: Inconsistent server timestamps on CEO emails"
                  value={formTitle}
                  onChange={(e) => setFormTitle(e.target.value)}
                  required
                  autoFocus
                />
              </div>

              <div className="grid-2 mb-4">
                <div>
                  <label className="label">Category</label>
                  <select
                    className="select input"
                    value={formCategory}
                    onChange={(e) => setFormCategory(e.target.value)}
                  >
                    <option value="general">General</option>
                    <option value="lead">Key Lead</option>
                    <option value="observation">Evidence Observation</option>
                    <option value="legal">Legal / Privileged</option>
                    <option value="hypothesis">Hypothesis</option>
                  </select>
                </div>
                <div>
                  <label className="label">Author / Investigator</label>
                  <input
                    className="input"
                    value={formAuthor}
                    onChange={(e) => setFormAuthor(e.target.value)}
                    placeholder="Investigator name"
                  />
                </div>
              </div>

              <div className="mb-4">
                <label className="label">Note Content (Markdown supported) *</label>
                <textarea
                  className="textarea"
                  style={{ minHeight: 140 }}
                  placeholder="Enter detailed observations, byte offset findings, or follow-up tasks..."
                  value={formContent}
                  onChange={(e) => setFormContent(e.target.value)}
                  required
                />
              </div>

              <div className="mb-4" style={{ display: "flex", alignItems: "center", gap: 8 }}>
                <input
                  type="checkbox"
                  id="pinCheck"
                  checked={formPinned}
                  onChange={(e) => setFormPinned(e.target.checked)}
                  style={{ width: 16, height: 16, cursor: "pointer" }}
                />
                <label htmlFor="pinCheck" style={{ fontSize: 13, color: "var(--text-1)", cursor: "pointer" }}>
                  Pin this note to the top of the case workspace
                </label>
              </div>

              <div className="row gap-2" style={{ justifyContent: "flex-end" }}>
                <button type="button" className="btn btn-ghost" onClick={() => setShowModal(false)}>
                  Cancel
                </button>
                <button type="submit" className="btn btn-primary" disabled={saving}>
                  {saving ? "Saving..." : editingNote ? "Update Note" : "Save Note"}
                </button>
              </div>
            </form>
          </div>
        </div>
      )}
    </div>
  );
}
