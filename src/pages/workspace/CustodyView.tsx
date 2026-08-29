import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Evidence } from "./types";

export function CustodyView({ evidence, caseId }: { evidence: Evidence[]; caseId: string }) {
  const [custody, setCustody] = useState<any[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    invoke<any[]>("custody_chain", { input: { case_id: caseId } })
      .then((events) => {
        setCustody(events);
        setLoading(false);
      })
      .catch(() => setLoading(false));
  }, [caseId]);

  return (
    <div>
      <h2 style={{ fontSize: 22, fontWeight: 700, color: "var(--text-0)", marginBottom: 16 }}>Chain of Custody</h2>
      {loading ? (
        <div className="empty">Loading...</div>
      ) : custody.length === 0 ? (
        <div className="card">
          <div className="empty">No custody events yet</div>
        </div>
      ) : (
        <div className="card">
          <table>
            <thead>
              <tr>
                <th className="th">Action</th>
                <th className="th">Timestamp</th>
                <th className="th">Tool</th>
                <th className="th">Detail</th>
                <th className="th">Hash</th>
              </tr>
            </thead>
            <tbody>
              {custody.map((e, i) => (
                <tr key={i}>
                  <td className="td">
                    <span className="badge badge-blue">{e.action}</span>
                  </td>
                  <td className="td muted">{new Date(e.timestamp).toLocaleString()}</td>
                  <td className="td">
                    {e.tool} v{e.tool_version}
                  </td>
                  <td className="td muted">{e.detail}</td>
                  <td className="td mono muted">{e.hash_after?.slice(0, 12) || "—"}…</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
    </div>
  );
}
