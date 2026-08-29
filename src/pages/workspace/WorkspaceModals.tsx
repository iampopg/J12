import { invoke } from "@tauri-apps/api/core";
import { Case, Evidence, Dashboard, View } from "./types";

interface ExportModalProps {
  show: boolean;
  onClose: () => void;
  caseId: string;
  caseData: Case | null;
  onSetToast: (msg: string) => void;
  onNavigate: (view: View) => void;
}

export function ExportCaseModal({
  show,
  onClose,
  caseId,
  caseData,
  onSetToast,
  onNavigate,
}: ExportModalProps) {
  if (!show) return null;

  return (
    <div style={{ position: "fixed", inset: 0, background: "rgba(0,0,0,0.8)", backdropFilter: "blur(5px)", display: "flex", alignItems: "center", justifyContent: "center", zIndex: 10000 }}>
      <div className="card" style={{ maxWidth: 520, width: "90%", padding: 24, border: "1px solid var(--accent)", boxShadow: "0 25px 60px rgba(0,0,0,0.8)" }}>
        <div className="row between mb-3" style={{ alignItems: "center" }}>
          <div className="row gap-2" style={{ alignItems: "center" }}>
            <span style={{ fontSize: 22 }}>📦</span>
            <h3 style={{ fontSize: 16, fontWeight: 700, margin: 0, color: "var(--text-0)" }}>Export Forensic Case Archive</h3>
          </div>
          <button className="btn btn-ghost btn-sm" onClick={onClose}>✕</button>
        </div>

        <p style={{ fontSize: 12.5, color: "var(--text-2)", marginBottom: 16, lineHeight: 1.5 }}>
          Export complete digital case files, forensic taxonomy summaries, audit trails, and SHA-256 chain of custody records for case <strong>"{caseData?.title}"</strong>.
        </p>

        <div style={{ display: "flex", flexDirection: "column", gap: 10, marginBottom: 20 }}>
          <div 
            style={{ background: "var(--bg-1)", padding: 12, borderRadius: "var(--r-sm)", border: "1px solid var(--border)", cursor: "pointer" }}
            onClick={async () => {
              try {
                const auditPath = await invoke<string>("export_audit_log", { input: { case_id: caseId } });
                onSetToast(`✓ Exported Case Audit Log to: ${auditPath}`);
                onClose();
              } catch (err) {
                onSetToast(`❌ Export failed: ${err}`);
              }
            }}
          >
            <div className="row between" style={{ alignItems: "center" }}>
              <div>
                <div style={{ fontWeight: 600, fontSize: 13, color: "var(--text-0)" }}>📋 Export Chain of Custody &amp; Audit Log (CSV)</div>
                <div style={{ fontSize: 11, color: "var(--text-3)", marginTop: 2 }}>Cryptographic verification timestamps, examiner records, SHA-256 seals</div>
              </div>
              <button className="btn btn-sm btn-ghost">Export</button>
            </div>
          </div>

          <div 
            style={{ background: "var(--bg-1)", padding: 12, borderRadius: "var(--r-sm)", border: "1px solid var(--border)", cursor: "pointer" }}
            onClick={() => {
              onNavigate("report");
              onClose();
            }}
          >
            <div className="row between" style={{ alignItems: "center" }}>
              <div>
                <div style={{ fontWeight: 600, fontSize: 13, color: "var(--text-0)" }}>📄 Comprehensive Investigation Report (PDF / Markdown)</div>
                <div style={{ fontSize: 11, color: "var(--text-3)", marginTop: 2 }}>Executive overview, findings breakdown, entity graph, timeline analysis</div>
              </div>
              <button className="btn btn-sm btn-primary">Go to Reports</button>
            </div>
          </div>

          <div 
            style={{ background: "var(--bg-1)", padding: 12, borderRadius: "var(--r-sm)", border: "1px solid var(--border)", cursor: "pointer" }}
            onClick={() => {
              onNavigate("artifacts");
              onClose();
            }}
          >
            <div className="row between" style={{ alignItems: "center" }}>
              <div>
                <div style={{ fontWeight: 600, fontSize: 13, color: "var(--text-0)" }}>🧩 Extracted Taxonomy Artifacts (CSV)</div>
                <div style={{ fontSize: 11, color: "var(--text-3)", marginTop: 2 }}>Credentials, banking IOCs, crypto wallets, and derived evidence</div>
              </div>
              <button className="btn btn-sm btn-ghost">Artifacts Hub</button>
            </div>
          </div>
        </div>

        <div className="row end">
          <button className="btn btn-ghost btn-sm" onClick={onClose}>Close</button>
        </div>
      </div>
    </div>
  );
}

interface DeleteModalProps {
  show: boolean;
  onClose: () => void;
  caseData: Case | null;
  evidence: Evidence[];
  dashboard: Dashboard | null;
  deleteConfirmText: string;
  setDeleteConfirmText: (s: string) => void;
  deletingCase: boolean;
  onDeleteCase: () => void;
}

export function DeleteCaseModal({
  show,
  onClose,
  caseData,
  evidence,
  dashboard,
  deleteConfirmText,
  setDeleteConfirmText,
  deletingCase,
  onDeleteCase,
}: DeleteModalProps) {
  if (!show) return null;

  return (
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
          Are you sure you want to delete case <strong>"{caseData?.title}"</strong>? This will permanently erase:
        </p>
        <ul style={{ fontSize: 12, color: "var(--text-2)", marginBottom: 16, paddingLeft: 20, lineHeight: 1.8 }}>
          <li>All evidence sources ({evidence.length} files)</li>
          <li>All parsed emails ({dashboard?.email_count?.toLocaleString() || 0} messages)</li>
          <li>All extracted forensic artifacts and security findings</li>
          <li>Chain of custody and audit records</li>
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
          <button className="btn btn-ghost" onClick={onClose} disabled={deletingCase}>
            Cancel
          </button>
          <button 
            className="btn btn-danger" 
            style={{ background: "#dc2626", color: "#fff", fontWeight: 700 }} 
            onClick={onDeleteCase} 
            disabled={deletingCase || deleteConfirmText.trim() !== "DELETE"}
          >
            {deletingCase ? "Deleting Case..." : "Delete Case Permanently"}
          </button>
        </div>
      </div>
    </div>
  );
}
