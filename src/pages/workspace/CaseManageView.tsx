import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Case } from "./types";

export function CaseManageView({
  caseData,
  caseId,
  onUpdate,
  onBack,
}: {
  caseData: Case | null;
  caseId: string;
  onUpdate: () => void;
  onBack: () => void;
}) {
  const [title, setTitle] = useState(caseData?.title || "");
  const [description, setDescription] = useState(caseData?.description || "");
  const [status, setStatus] = useState(caseData?.status || "open");
  const [targetName, setTargetName] = useState(caseData?.target_name || "");
  const [targetEmail, setTargetEmail] = useState(caseData?.target_email || "");
  const [targetOrg, setTargetOrg] = useState(caseData?.target_organization || "");
  const [saving, setSaving] = useState(false);
  const [showDelete, setShowDelete] = useState(false);
  const [deleteConfirmText, setDeleteConfirmText] = useState("");
  const [deleting, setDeleting] = useState(false);

  const handleSave = async () => {
    setSaving(true);
    try {
      await invoke("case_update", {
        input: {
          case_id: caseId,
          title,
          description,
          status,
          target_name: targetName,
          target_email: targetEmail,
          target_organization: targetOrg,
        },
      });
      onUpdate();
    } catch (e) {
      console.error("Failed to update case:", e);
    } finally {
      setSaving(false);
    }
  };

  const handleDelete = async () => {
    setDeleting(true);
    try {
      await invoke("case_delete", { input: { case_id: caseId } });
      try {
        Object.keys(localStorage).forEach((key) => {
          if (key.includes(caseId)) {
            localStorage.removeItem(key);
          }
        });
      } catch (storageErr) {
        console.warn("Failed to purge localStorage for deleted case:", storageErr);
      }
      onBack();
    } catch (e) {
      console.error("Failed to delete case:", e);
    } finally {
      setDeleting(false);
    }
  };

  return (
    <div>
      <div className="row between mb-4">
        <div>
          <h2 style={{ fontSize: 22, fontWeight: 700, color: "var(--text-0)" }}>Case Settings</h2>
          <p className="muted">Configure investigation metadata, target parameters, and case lifecycle</p>
        </div>
        <button className="btn btn-ghost btn-sm" onClick={onBack}>← Back to Dashboard</button>
      </div>

      <div className="card mb-4">
        <h3 style={{ fontSize: 15, fontWeight: 600, marginBottom: 16 }}>Case Information</h3>
        <div className="field">
          <label className="label">Case ID (not editable)</label>
          <input className="input" value={caseId} disabled style={{ opacity: 0.6 }} />
        </div>
        <div className="field">
          <label className="label">Case Number</label>
          <input className="input" value={caseData?.case_number || "—"} disabled style={{ opacity: 0.6 }} />
        </div>
        <div className="field">
          <label className="label">Title</label>
          <input className="input" value={title} onChange={(e) => setTitle(e.target.value)} />
        </div>
        <div className="field">
          <label className="label">Description</label>
          <textarea className="textarea" value={description} onChange={(e) => setDescription(e.target.value)} rows={3} />
        </div>
        <div className="field">
          <label className="label">Status</label>
          <select className="input" value={status} onChange={(e) => setStatus(e.target.value)}>
            <option value="open">Open</option>
            <option value="closed">Closed</option>
            <option value="archived">Archived</option>
          </select>
        </div>
      </div>

      <div className="card mb-4">
        <h3 style={{ fontSize: 15, fontWeight: 600, marginBottom: 16 }}>Investigation Target</h3>
        <div className="field">
          <label className="label">Target Name</label>
          <input className="input" value={targetName} onChange={(e) => setTargetName(e.target.value)} placeholder="e.g. John Doe" />
        </div>
        <div className="field">
          <label className="label">Target Email</label>
          <input className="input" value={targetEmail} onChange={(e) => setTargetEmail(e.target.value)} placeholder="e.g. john@example.com" />
        </div>
        <div className="field">
          <label className="label">Target Organization</label>
          <input className="input" value={targetOrg} onChange={(e) => setTargetOrg(e.target.value)} placeholder="e.g. Acme Corp" />
        </div>
      </div>

      <div className="row gap-2">
        <button className="btn btn-primary" onClick={handleSave} disabled={saving}>
          {saving ? "Saving..." : "Save Changes"}
        </button>
        <button className="btn btn-ghost" onClick={onBack}>Cancel</button>
      </div>

      <div className="card mt-4" style={{ borderColor: "var(--red)", border: "1px solid var(--red)" }}>
        <h3 style={{ fontSize: 15, fontWeight: 600, color: "var(--red)", marginBottom: 12 }}>Danger Zone</h3>
        <p style={{ fontSize: 13, color: "var(--text-2)", marginBottom: 16 }}>
          Deleting this case will permanently remove all evidence, emails, findings, and chain of custody records.
        </p>
        <button className="btn" style={{ background: "var(--red)", color: "#fff" }} onClick={() => setShowDelete(true)}>
          Delete Case &amp; All Data
        </button>
      </div>

      {showDelete && (
        <div style={{ position: "fixed", inset: 0, background: "rgba(0,0,0,0.75)", backdropFilter: "blur(4px)", display: "flex", alignItems: "center", justifyContent: "center", zIndex: 10000 }}>
          <div className="card" style={{ maxWidth: 460, width: "90%", padding: 24, border: "1px solid rgba(239, 68, 68, 0.4)", boxShadow: "0 20px 50px rgba(0,0,0,0.7)" }}>
            <div style={{ display: "flex", alignItems: "center", gap: 12, marginBottom: 16 }}>
              <span style={{ fontSize: 32 }}>⚠️</span>
              <div>
                <h3 style={{ fontSize: 18, fontWeight: 700, color: "var(--danger)", margin: 0 }}>Delete Case</h3>
                <p className="muted" style={{ fontSize: 12, margin: "4px 0 0" }}>Permanent &amp; Irreversible Destruction</p>
              </div>
            </div>
            <p style={{ fontSize: 13, color: "var(--text-1)", marginBottom: 12, lineHeight: 1.6 }}>
              Are you sure you want to delete case <strong>"{caseData?.title}"</strong>? This will permanently remove:
            </p>
            <ul style={{ fontSize: 12, color: "var(--text-2)", marginBottom: 16, paddingLeft: 20, lineHeight: 1.8 }}>
              <li>All evidence sources</li>
              <li>All parsed emails</li>
              <li>All findings, artifacts, and analysis results</li>
              <li>Chain of custody records</li>
            </ul>

            <div style={{ marginBottom: 18 }}>
              <label className="label" style={{ color: "var(--danger)", fontWeight: 700, fontSize: 11 }}>
                Type <span style={{ textDecoration: "underline" }}>DELETE</span> to confirm:
              </label>
              <input
                className="input"
                style={{ borderColor: deleteConfirmText === "DELETE" ? "var(--danger)" : "var(--border)", fontWeight: 700, letterSpacing: "0.08em" }}
                placeholder="Type DELETE"
                value={deleteConfirmText}
                onChange={e => setDeleteConfirmText(e.target.value)}
                autoFocus
              />
            </div>

            <div className="row gap-2" style={{ justifyContent: "flex-end" }}>
              <button className="btn btn-ghost" onClick={() => { setShowDelete(false); setDeleteConfirmText(""); }} disabled={deleting}>
                Cancel
              </button>
              <button 
                className="btn btn-danger" 
                style={{ background: "#dc2626", color: "#fff", fontWeight: 700 }} 
                onClick={handleDelete} 
                disabled={deleting || deleteConfirmText.trim() !== "DELETE"}
              >
                {deleting ? "Deleting..." : "Delete Permanently"}
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
