import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";

export function IntegrityView({ caseId }: { caseId: string }) {
  const [verification, setVerification] = useState<any>(null);
  const [chainCheck, setChainCheck] = useState<any>(null);
  const [loading, setLoading] = useState(false);

  const verifyHashes = async () => {
    setLoading(true);
    try {
      const result = await invoke<any>("verify_evidence_hashes", { input: { case_id: caseId } });
      setVerification(result);
    } catch (e) {
      console.error(e);
    }
    setLoading(false);
  };

  const checkChain = async () => {
    setLoading(true);
    try {
      const result = await invoke<any>("check_custody_chain", { input: { case_id: caseId } });
      setChainCheck(result);
    } catch (e) {
      console.error(e);
    }
    setLoading(false);
  };

  const exportAudit = async () => {
    try {
      const path = await invoke<string>("export_audit_log", { input: { case_id: caseId } });
      alert(`Audit log exported to: ${path}`);
    } catch (e) {
      console.error(e);
    }
  };

  return (
    <div>
      <h2 style={{ fontSize: 22, fontWeight: 700, color: "var(--text-0)", marginBottom: 16 }}>Verify Evidence Integrity</h2>
      <p className="muted mb-4">Validate cryptographic SHA-256 evidence seals and export audit logs</p>

      <div className="row gap-2 mb-4">
        <button className="btn btn-primary" onClick={verifyHashes} disabled={loading}>🔍 Verify Evidence Hashes</button>
        <button className="btn btn-ghost" onClick={checkChain} disabled={loading}>🔗 Check Custody Chain</button>
        <button className="btn btn-ghost" onClick={exportAudit}>📥 Export Audit Log</button>
      </div>

      {verification && (
        <div className="card mb-4">
          <h3 style={{ fontSize: 15, fontWeight: 600, marginBottom: 16 }}>Hash Verification Results</h3>
          <div className="grid-3 mb-4">
            <div className="card" style={{ textAlign: "center", padding: 16 }}>
              <div style={{ fontSize: 24, fontWeight: 700, color: "var(--success)" }}>{verification.verified}</div>
              <div className="muted">Verified</div>
            </div>
            <div className="card" style={{ textAlign: "center", padding: 16 }}>
              <div style={{ fontSize: 24, fontWeight: 700, color: "var(--red)" }}>{verification.failed}</div>
              <div className="muted">Modified</div>
            </div>
            <div className="card" style={{ textAlign: "center", padding: 16 }}>
              <div style={{ fontSize: 24, fontWeight: 700, color: "var(--text-2)" }}>{verification.missing}</div>
              <div className="muted">Missing</div>
            </div>
          </div>
          <table>
            <thead><tr><th>Filename</th><th>Status</th></tr></thead>
            <tbody>
              {verification.results.map((r: any, i: number) => (
                <tr key={i}>
                  <td>{r.filename}</td>
                  <td><span className={`badge ${r.status === "verified" ? "badge-green" : r.status === "modified" ? "badge-red" : "badge-gray"}`}>{r.status}</span></td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}

      {chainCheck && (
        <div className="card">
          <h3 style={{ fontSize: 15, fontWeight: 600, marginBottom: 16 }}>Custody Chain Check</h3>
          <p>Chain Intact: <span className={`badge ${chainCheck.chain_intact || chainCheck.is_valid ? "badge-green" : "badge-red"}`}>{chainCheck.chain_intact || chainCheck.is_valid ? "YES" : "NO"}</span></p>
          {chainCheck.events_count !== undefined && (
            <p className="muted" style={{ marginTop: 6, fontSize: 12 }}>Custody Events Logged: {chainCheck.events_count}</p>
          )}
          {Array.isArray(chainCheck.gaps) && chainCheck.gaps.length > 0 && (
            <div style={{ marginTop: 12 }}>
              <strong>Gaps Found:</strong>
              <ul style={{ paddingLeft: 20, marginTop: 8 }}>
                 {chainCheck.gaps.map((g: any, i: number) => (
                  <li key={i}>{g.evidence}: {g.issue}</li>
                ))}
              </ul>
            </div>
          )}
        </div>
      )}
    </div>
  );
}
