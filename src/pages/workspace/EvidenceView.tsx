import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useAcquisition } from "../../context/AcquisitionContext";
import { Evidence } from "./types";
import { ImapAcquisition } from "./ImapAcquisition";

interface Props {
  evidence: Evidence[];
  caseId: string;
  onRefresh: () => void;
  onViewEmails?: (evidenceId: string) => void;
}

export function EvidenceView({ evidence, caseId, onRefresh, onViewEmails }: Props) {
  const { runFullPostIngestPipeline } = useAcquisition();
  const [uploading, setUploading] = useState(false);
  const [logs, setLogs] = useState<any[]>([]);
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [dragOver, setDragOver] = useState(false);
  const [acqMethod, setAcqMethod] = useState<"file" | "server" | "client" | "imaging">("file");

  const [deletingId, setDeletingId] = useState<string | null>(null);
  const [confirmDeleteModal, setConfirmDeleteModal] = useState<{ id: string; filename: string } | null>(null);
  const [deleteEvidenceConfirmText, setDeleteEvidenceConfirmText] = useState("");

  const addLog = (level: string, message: string) => {
    setLogs(prev => [...prev, { time: new Date().toLocaleTimeString(), level, message }]);
  };

  const handleUpload = async () => {
    try {
      const selected = await invoke<string | null>("open_file_dialog");
      if (!selected) return;
      processFile(selected);
    } catch (e: any) { addLog("error", `Upload failed: ${e}`); }
  };

  const processFile = async (path: string) => {
    setUploading(true);
    addLog("info", `Uploading: ${path}`);
    try {
      const ev = await invoke<any>("evidence_upload", { input: { case_id: caseId, file_path: path, source_description: null } });
      addLog("success", `Uploaded: ${ev.filename} (${ev.format}, ${(ev.size_bytes / 1024).toFixed(0)} KB)`);
      addLog("info", "Auto-parsing RFC message structures...");
      invoke("parse_evidence", { evidenceId: ev.id }).then(async (count: any) => {
        addLog("success", `Parsed ${count} emails`);
        onRefresh();
        addLog("info", "Starting automated post-ingest forensic intelligence pipeline...");
        await runFullPostIngestPipeline(caseId);
        onRefresh();
      }).catch((err: any) => {
        addLog("error", `Parse failed: ${err}`);
      });
      onRefresh();
    } catch (e: any) { addLog("error", `Upload failed: ${e}`); }
    setUploading(false);
  };

  const handleDrop = async (e: React.DragEvent) => {
    e.preventDefault();
    setDragOver(false);
    addLog("info", "Please use the upload button to select files (browser security restricts drag-drop paths)");
  };

  const handleDragOver = (e: React.DragEvent) => {
    e.preventDefault();
    setDragOver(true);
  };

  const handleDragLeave = () => {
    setDragOver(false);
  };

  const handleParse = async (evidenceId: string, filename: string) => {
    addLog("info", `Parsing ${filename}...`);
    try {
      const count = await invoke<number>("parse_evidence", { evidenceId });
      addLog("success", `Parsed ${count} emails from ${filename}`);
      onRefresh();
      addLog("info", "Starting automated post-ingest forensic intelligence pipeline...");
      await runFullPostIngestPipeline(caseId);
      onRefresh();
    } catch (e: any) {
      addLog("error", `Parse failed: ${e}`);
      onRefresh();
    }
  };

  const handleDeleteEvidence = async (evidenceId: string, filename: string) => {
    setDeletingId(evidenceId);
    try {
      await invoke("evidence_delete", { input: { evidence_id: evidenceId } });
      addLog("success", `Deleted evidence source "${filename}" and its associated emails.`);
      if (selectedId === evidenceId) setSelectedId(null);
      setConfirmDeleteModal(null);
      setDeleteEvidenceConfirmText("");
      onRefresh();
    } catch (e: any) {
      addLog("error", `Failed to delete evidence: ${e}`);
    } finally {
      setDeletingId(null);
    }
  };

  const selectedEvidence = selectedId ? evidence.find(e => e.id === selectedId) : null;

  return (
    <div>
      {confirmDeleteModal && (
        <div
          style={{
            position: "fixed",
            inset: 0,
            background: "rgba(0,0,0,0.75)",
            backdropFilter: "blur(4px)",
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
            zIndex: 9999,
          }}
          onClick={() => { setConfirmDeleteModal(null); setDeleteEvidenceConfirmText(""); }}
        >
          <div
            className="card"
            style={{
              maxWidth: 480,
              width: "92%",
              padding: 24,
              border: "1px solid rgba(239, 68, 68, 0.4)",
              boxShadow: "0 20px 40px rgba(0,0,0,0.6)",
              background: "var(--bg-1)",
            }}
            onClick={(e) => e.stopPropagation()}
          >
            <div style={{ display: "flex", alignItems: "center", gap: 12, marginBottom: 16 }}>
              <span style={{ fontSize: 32 }}>⚠️</span>
              <div>
                <h3 style={{ fontSize: 16, fontWeight: 700, color: "var(--danger)", margin: 0 }}>
                  Delete Evidence Source?
                </h3>
                <p className="muted" style={{ fontSize: 12, margin: "4px 0 0" }}>
                  Irreversible Forensic Action
                </p>
              </div>
            </div>
            <p style={{ fontSize: 13, lineHeight: 1.5, color: "var(--text-1)", marginBottom: 16 }}>
              Are you sure you want to permanently delete <strong>"{confirmDeleteModal.filename}"</strong>? 
              This will remove all associated emails, extracted attachments, and chain-of-custody records for this container.
            </p>

            <div style={{ marginBottom: 18 }}>
              <label className="label" style={{ color: "var(--danger)", fontWeight: 700, fontSize: 11 }}>
                Type <span style={{ textDecoration: "underline" }}>DELETE</span> to confirm:
              </label>
              <input
                className="input"
                style={{ borderColor: deleteEvidenceConfirmText === "DELETE" ? "var(--danger)" : "var(--border)", fontWeight: 700, letterSpacing: "0.08em" }}
                placeholder="Type DELETE"
                value={deleteEvidenceConfirmText}
                onChange={e => setDeleteEvidenceConfirmText(e.target.value)}
                autoFocus
              />
            </div>

            <div style={{ display: "flex", justifyContent: "flex-end", gap: 10 }}>
              <button className="btn btn-ghost" onClick={() => { setConfirmDeleteModal(null); setDeleteEvidenceConfirmText(""); }}>
                Cancel
              </button>
              <button
                className="btn btn-danger"
                style={{ background: "#dc2626", color: "#fff", fontWeight: 700 }}
                onClick={() => handleDeleteEvidence(confirmDeleteModal.id, confirmDeleteModal.filename)}
                disabled={deletingId !== null || deleteEvidenceConfirmText.trim() !== "DELETE"}
              >
                {deletingId ? "Deleting..." : "Delete Evidence Source"}
              </button>
            </div>
          </div>
        </div>
      )}

      <div className="row between mb-4">
        <div>
          <h2 style={{ fontSize: 22, fontWeight: 700, color: "var(--text-0)" }}>Evidence Acquisition</h2>
          <p className="muted">Import evidence into this case</p>
        </div>
      </div>

      <div className="card mb-4">
        <div className="row gap-2 mb-4" style={{ borderBottom: "1px solid var(--border)", paddingBottom: 0 }}>
          {[
            { key: "file", label: "📁 File Import", desc: "EML, MBOX, PST, MSG", active: true },
            { key: "server", label: "☁️ Mail Server", desc: "IMAP / POP3 / TLS Live", active: true },
            { key: "client", label: "💻 Mail Client", desc: "Outlook, Apple Mail, Thunderbird", active: false },
            { key: "imaging", label: "💾 Forensic Imaging", desc: "Disk, Device, E01", active: false },
          ].map(tab => (
            <button
              key={tab.key}
              className={`btn btn-sm ${acqMethod === tab.key ? "btn-primary" : "btn-ghost"}`}
              style={{ borderRadius: "6px 6px 0 0", display: "flex", flexDirection: "column", alignItems: "center", padding: "8px 14px", opacity: tab.active ? 1 : 0.7 }}
              onClick={() => setAcqMethod(tab.key as any)}
            >
              <span style={{ fontSize: 12, fontWeight: 600 }}>{tab.label}</span>
              <span style={{ fontSize: 10, opacity: 0.7 }}>{tab.desc}</span>
              {!tab.active && <span style={{ fontSize: 9, color: "var(--text-3)" }}>Coming Soon</span>}
            </button>
          ))}
        </div>

        {acqMethod === "file" && (
          <div>
            <div
              style={{
                textAlign: "center",
                padding: "40px 20px",
                border: dragOver ? "2px dashed var(--accent)" : "2px dashed var(--border)",
                borderRadius: "var(--r-md)",
                background: dragOver ? "var(--accent-subtle)" : "transparent",
              }}
              onDragOver={handleDragOver}
              onDragLeave={handleDragLeave}
              onDrop={handleDrop}
            >
              <div style={{ fontSize: 40, marginBottom: 12 }}>📧</div>
              <h4 style={{ fontSize: 15, fontWeight: 600, marginBottom: 6 }}>Upload Email Files</h4>
              <p className="muted mb-4" style={{ fontSize: 12 }}>Supports: EML, MBOX, PST, OST, MSG, EMLX</p>
              <button className="btn btn-primary" onClick={handleUpload} disabled={uploading}>
                {uploading ? "Uploading..." : "+ Select Files"}
              </button>
            </div>
          </div>
        )}

        {acqMethod === "server" && (
          <ImapAcquisition caseId={caseId} onComplete={onRefresh} />
        )}

        {acqMethod === "client" && (
          <div style={{ textAlign: "center", padding: "40px 20px" }}>
            <div style={{ fontSize: 40, marginBottom: 12 }}>💻</div>
            <h4 style={{ fontSize: 15, fontWeight: 600, marginBottom: 6 }}>Local Mail Client Extraction</h4>
            <p className="muted mb-4" style={{ fontSize: 12 }}>
              Auto-detect and extract from installed mail applications on this computer
            </p>
            <div className="row gap-2" style={{ justifyContent: "center", flexWrap: "wrap" }}>
              <div className="card" style={{ padding: 16, minWidth: 180 }}>
                <div style={{ fontSize: 24, marginBottom: 8 }}>📮</div>
                <h5 style={{ fontSize: 13, fontWeight: 600, marginBottom: 4 }}>Apple Mail</h5>
                <p className="muted" style={{ fontSize: 11 }}>macOS native mail app</p>
                <span className="badge badge-gray mt-2">Coming Soon</span>
              </div>
              <div className="card" style={{ padding: 16, minWidth: 180 }}>
                <div style={{ fontSize: 24, marginBottom: 8 }}>🔷</div>
                <h5 style={{ fontSize: 13, fontWeight: 600, marginBottom: 4 }}>Outlook</h5>
                <p className="muted" style={{ fontSize: 11 }}>Mac/Windows, OST/PST</p>
                <span className="badge badge-gray mt-2">Coming Soon</span>
              </div>
              <div className="card" style={{ padding: 16, minWidth: 180 }}>
                <div style={{ fontSize: 24, marginBottom: 8 }}>🦅</div>
                <h5 style={{ fontSize: 13, fontWeight: 600, marginBottom: 4 }}>Thunderbird</h5>
                <p className="muted" style={{ fontSize: 11 }}>MBOX-based storage</p>
                <span className="badge badge-gray mt-2">Coming Soon</span>
              </div>
            </div>
          </div>
        )}

        {acqMethod === "imaging" && (
          <div style={{ textAlign: "center", padding: "40px 20px" }}>
            <div style={{ fontSize: 40, marginBottom: 12 }}>💾</div>
            <h4 style={{ fontSize: 15, fontWeight: 600, marginBottom: 6 }}>Forensic Physical &amp; Logical Imaging</h4>
            <p className="muted mb-4" style={{ fontSize: 12 }}>
              Extract email stores from physical drives, device dumps, and E01 forensic images
            </p>
            <span className="badge badge-gray">Coming Soon</span>
          </div>
        )}
      </div>

      {logs.length > 0 && (
        <div className="card mb-4" style={{ maxHeight: 150, overflowY: "auto", fontFamily: "monospace", fontSize: 12 }}>
          <div className="row between mb-4">
            <h4 style={{ fontSize: 12, fontWeight: 600 }}>Activity Log</h4>
            <button className="btn btn-ghost btn-sm" onClick={() => setLogs([])}>Clear</button>
          </div>
          {logs.map((log, i) => (
            <div key={i} className={`log-${log.level}`} style={{ padding: "2px 0" }}>
              <span className="muted">[{log.time}]</span>{" "}
              <span className={`badge badge-${log.level === "error" ? "red" : log.level === "success" ? "green" : "blue"}`}>{log.level}</span>
              <span>{log.message}</span>
            </div>
          ))}
        </div>
      )}

      {evidence.length > 0 && (
        <div className="card">
          <div className="row between mb-3" style={{ alignItems: "center" }}>
            <h3 style={{ fontSize: 15, fontWeight: 600, margin: 0 }}>Evidence Sources ({evidence.length})</h3>
            <button
              className="btn btn-ghost btn-sm"
              style={{ fontSize: 11 }}
              onClick={async () => {
                try {
                  await invoke("open_forensic_logs_folder", { case_id: caseId });
                } catch (e) {
                  console.error("Failed to open log folder:", e);
                }
              }}
              title="Open the case forensic audit log folder on disk"
            >
              📁 Forensic Audit Log Folder
            </button>
          </div>
          <table>
            <thead>
              <tr>
                <th className="th">File</th>
                <th className="th">Format</th>
                <th className="th">Size</th>
                <th className="th">Status</th>
                <th className="th">Messages</th>
                <th className="th">SHA-256</th>
                <th className="th" style={{ textAlign: "right" }}>Actions</th>
              </tr>
            </thead>
            <tbody>
              {evidence.map((e) => (
                <tr key={e.id} onClick={() => setSelectedId(selectedId === e.id ? null : e.id)} className="tr-click" style={{ background: selectedId === e.id ? "var(--bg-3)" : "transparent" }}>
                  <td className="td" style={{ fontWeight: 600 }}>{e.filename}</td>
                  <td className="td"><span className={`badge badge-${e.format === "eml" ? "blue" : e.format === "mbox" ? "green" : e.format === "imap" ? "purple" : "orange"}`}>{e.format}</span></td>
                  <td className="td muted">{(e.size_bytes / 1024).toFixed(0)} KB</td>
                  <td className="td"><span className={`badge ${e.parse_status === "done" ? "badge-green" : e.parse_status === "error" ? "badge-red" : e.parse_status === "parsing" || e.parse_status === "ingesting" ? "badge-blue" : "badge-gray"}`}>{e.parse_status}</span></td>
                  <td className="td">{e.message_count}</td>
                  <td className="td mono muted">{e.sha256 ? `${e.sha256.slice(0, 12)}…` : "—"}</td>
                  <td className="td" style={{ textAlign: "right" }}>
                    <div style={{ display: "inline-flex", gap: 6, alignItems: "center" }}>
                      {(e.parse_status === "done" || e.parse_status === "parsed") && (
                        <button
                          className="btn btn-primary btn-sm"
                          style={{ padding: "4px 9px", fontSize: 11, fontWeight: 600 }}
                          onClick={(ev) => {
                            ev.stopPropagation();
                            onViewEmails?.(e.id);
                          }}
                          title={`View emails specifically from ${e.filename}`}
                        >
                          📬 View Emails
                        </button>
                      )}
                      {e.parse_status === "pending" && <button className="btn btn-primary btn-sm" onClick={(ev) => { ev.stopPropagation(); handleParse(e.id, e.filename); }}>Parse</button>}
                      {e.parse_status === "parsing" && <span className="muted text-sm">Parsing...</span>}
                      {e.parse_status === "error" && <button className="btn btn-ghost btn-sm" onClick={(ev) => { ev.stopPropagation(); handleParse(e.id, e.filename); }}>Retry</button>}
                      <button
                        className="btn btn-danger btn-sm"
                        style={{ padding: "4px 8px", fontSize: 12, background: "rgba(239, 68, 68, 0.15)", color: "#ef4444", border: "1px solid rgba(239, 68, 68, 0.3)" }}
                        title={`Delete evidence source: ${e.filename}`}
                        onClick={(ev) => {
                          ev.stopPropagation();
                          setConfirmDeleteModal({ id: e.id, filename: e.filename });
                        }}
                        disabled={deletingId === e.id}
                      >
                        {deletingId === e.id ? "Deleting..." : "🗑️ Delete"}
                      </button>
                    </div>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}

      {selectedEvidence && (
        <div className="card mt-4">
          <div className="row between mb-4">
            <h4 style={{ fontSize: 14, fontWeight: 600 }}>Evidence Details</h4>
            <div style={{ display: "flex", gap: 8 }}>
              <button
                className="btn btn-danger btn-sm"
                style={{ padding: "4px 10px", fontSize: 12, background: "rgba(239, 68, 68, 0.15)", color: "#ef4444", border: "1px solid rgba(239, 68, 68, 0.3)" }}
                onClick={() => setConfirmDeleteModal({ id: selectedEvidence.id, filename: selectedEvidence.filename })}
              >
                🗑️ Delete Evidence Source
              </button>
              <button className="btn btn-ghost btn-sm" onClick={() => setSelectedId(null)}>Close</button>
            </div>
          </div>
          <div className="grid-2" style={{ fontSize: 13 }}>
            <div><span className="muted">File:</span> {selectedEvidence.filename}</div>
            <div><span className="muted">Format:</span> {selectedEvidence.format}</div>
            <div><span className="muted">Size:</span> {selectedEvidence.size_bytes} bytes</div>
            <div><span className="muted">Status:</span> <span className={`badge ${selectedEvidence.parse_status === "done" ? "badge-green" : selectedEvidence.parse_status === "error" ? "badge-red" : "badge-gray"}`}>{selectedEvidence.parse_status}</span></div>
            <div><span className="muted">Messages:</span> {selectedEvidence.message_count}</div>
            <div><span className="muted">Deleted Recovered:</span> {selectedEvidence.deleted_recovered}</div>
            <div><span className="muted">SHA-256:</span> <span className="mono">{selectedEvidence.sha256}</span></div>
            <div><span className="muted">Acquired:</span> {new Date(selectedEvidence.acquired_at).toLocaleString()}</div>
            <div style={{ gridColumn: "1 / -1" }}><span className="muted">Source:</span> {selectedEvidence.source_description || "—"}</div>
          </div>
          {selectedEvidence.parse_error && (
            <div style={{ marginTop: 12, padding: 12, background: "rgba(239,68,68,0.1)", borderRadius: 8, fontSize: 12, color: "var(--danger)" }}>
              <strong>Error:</strong> {selectedEvidence.parse_error}
            </div>
          )}
        </div>
      )}
    </div>
  );
}
