import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";

interface LogEntry {
  timestamp: string;
  level: "info" | "error" | "success";
  message: string;
}

export function EvidenceView({ evidence, caseId, onRefresh }: { evidence: any[]; caseId: string; onRefresh: () => void }) {
  const [uploading, setUploading] = useState(false);
  const [logs, setLogs] = useState<LogEntry[]>([]);
  const [selectedEvidence, setSelectedEvidence] = useState<string | null>(null);

  const addLog = (level: LogEntry["level"], message: string) => {
    setLogs(prev => [...prev, { timestamp: new Date().toLocaleTimeString(), level, message }]);
  };

  const handleUpload = async () => {
    try {
      const selected = await invoke<string | null>("open_file_dialog");
      if (!selected) return;
      setUploading(true);
      addLog("info", `Uploading: ${selected}`);
      
      const ev = await invoke<any>("evidence_upload", { input: { case_id: caseId, file_path: selected, source_description: null } });
      addLog("success", `Uploaded: ${ev.filename} (${ev.format})`);
      
      addLog("info", "Starting parse...");
      invoke("parse_evidence", { evidenceId: ev.id }).then((count) => {
        addLog("success", `Parse complete: ${count} emails`);
        onRefresh();
      }).catch((err) => {
        addLog("error", `Parse failed: ${err}`);
      });
      onRefresh();
    } catch (e) { addLog("error", `Upload failed: ${e}`); }
    setUploading(false);
  };

  const handleParse = async (evidenceId: string, filename: string) => {
    addLog("info", `Parsing: ${filename}`);
    try {
      const count = await invoke<number>("parse_evidence", { evidenceId });
      addLog("success", `Parsed ${count} emails from ${filename}`);
      onRefresh();
    } catch (e) {
      addLog("error", `Parse failed: ${e}`);
    }
  };

  return (
    <div>
      <div className="row between mb-4">
        <div>
          <h2 style={{ fontSize: 22, fontWeight: 700, color: "var(--text-0)" }}>Evidence</h2>
          <p className="muted">Manage evidence sources for this case</p>
        </div>
        <button className="btn btn-primary" onClick={handleUpload} disabled={uploading}>
          {uploading ? "Uploading..." : "+ Add Evidence"}
        </button>
      </div>

      {/* Log output */}
      {logs.length > 0 && (
        <div className="card mb-4" style={{ maxHeight: 200, overflowY: "auto", background: "var(--bg-0)" }}>
          <div className="row between mb-4">
            <h4 style={{ fontSize: 13, fontWeight: 600 }}>Activity Log</h4>
            <button className="btn btn-ghost btn-sm" onClick={() => setLogs([])}>Clear</button>
          </div>
          {logs.map((log, i) => (
            <div key={i} className="row gap-2" style={{ fontSize: 12, fontFamily: "var(--mono)", marginBottom: 4 }}>
              <span className="muted">{log.timestamp}</span>
              <span className={`badge badge-${log.level === "error" ? "red" : log.level === "success" ? "green" : "blue"}`}>{log.level}</span>
              <span>{log.message}</span>
            </div>
          ))}
        </div>
      )}

      {evidence.length === 0 ? (
        <div className="card" style={{ textAlign: "center", padding: "60px 40px" }}>
          <div style={{ fontSize: 48, marginBottom: 16 }}>📁</div>
          <h3 style={{ fontSize: 18, marginBottom: 8, color: "var(--text-0)" }}>No evidence yet</h3>
          <p className="muted mb-4">Upload email files to begin analysis.</p>
          <button className="btn btn-primary" onClick={handleUpload}>+ Upload Evidence</button>
        </div>
      ) : (
        <div className="card">
          <table>
            <thead>
              <tr>
                <th className="th">File</th>
                <th className="th">Format</th>
                <th className="th">Size</th>
                <th className="th">Status</th>
                <th className="th">Messages</th>
                <th className="th">SHA-256</th>
                <th className="th">Actions</th>
              </tr>
            </thead>
            <tbody>
              {evidence.map((e) => (
                <tr key={e.id} className="tr-click" onClick={() => setSelectedEvidence(selectedEvidence === e.id ? null : e.id)}>
                  <td className="td">{e.filename}</td>
                  <td className="td"><span className={`badge badge-${e.format === "eml" ? "blue" : e.format === "mbox" ? "green" : "orange"}`}>{e.format}</span></td>
                  <td className="td muted">{(e.size_bytes / 1024).toFixed(0)} KB</td>
                  <td className="td">
                    <span className={`badge ${e.parse_status === "done" ? "badge-green" : e.parse_status === "error" ? "badge-red" : e.parse_status === "parsing" ? "badge-blue" : "badge-gray"}`}>
                      {e.parse_status}
                    </span>
                  </td>
                  <td className="td">{e.message_count}</td>
                  <td className="td mono muted">{e.sha256.slice(0, 12)}…</td>
                  <td className="td">
                    {e.parse_status === "pending" && (
                      <button className="btn btn-primary btn-sm" onClick={(ev) => { ev.stopPropagation(); handleParse(e.id, e.filename); }}>
                        Parse
                      </button>
                    )}
                    {e.parse_status === "parsing" && <span className="muted">Parsing...</span>}
                    {e.parse_status === "error" && (
                      <button className="btn btn-ghost btn-sm" onClick={(ev) => { ev.stopPropagation(); handleParse(e.id, e.filename); }}>
                        Retry
                      </button>
                    )}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}

      {/* Detail panel for selected evidence */}
      {selectedEvidence && (() => {
        const ev = evidence.find(e => e.id === selectedEvidence);
        if (!ev) return null;
        return (
          <div className="card mt-4">
            <h4 style={{ fontSize: 14, fontWeight: 600, marginBottom: 12 }}>Evidence Details</h4>
            <div className="grid-2" style={{ fontSize: 13 }}>
              <div><span className="muted">ID:</span> <span className="mono">{ev.id}</span></div>
              <div><span className="muted">Format:</span> {ev.format}</div>
              <div><span className="muted">Size:</span> {ev.size_bytes} bytes</div>
              <div><span className="muted">Status:</span> {ev.parse_status}</div>
              <div><span className="muted">Messages:</span> {ev.message_count}</div>
              <div><span className="muted">Deleted Recovered:</span> {ev.deleted_recovered}</div>
              <div><span className="muted">SHA-256:</span> <span className="mono">{ev.sha256}</span></div>
              <div><span className="muted">Source:</span> {ev.source_description || "—"}</div>
            </div>
            {ev.parse_error && (
              <div style={{ marginTop: 12, padding: 12, background: "rgba(239,68,68,0.1)", borderRadius: 8, fontSize: 12, color: "var(--danger)" }}>
                Error: {ev.parse_error}
              </div>
            )}
          </div>
        );
      })()}
    </div>
  );
}