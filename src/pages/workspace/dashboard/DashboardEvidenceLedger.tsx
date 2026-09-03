import { useState } from "react";
import { Evidence, View } from "../types";

interface Props {
  evidence: Evidence[];
  onNavigate: (view: View) => void;
}

export function DashboardEvidenceLedger({ evidence, onNavigate }: Props) {
  const [copiedHash, setCopiedHash] = useState<string | null>(null);

  if (evidence.length === 0) return null;

  const handleCopy = (hash: string) => {
    navigator.clipboard.writeText(hash);
    setCopiedHash(hash);
    setTimeout(() => setCopiedHash(null), 2000);
  };

  const uniqueEvidence = evidence.reduce((unique: Evidence[], e) => {
    const existing = unique.find((u) => u.filename === e.filename);
    if (!existing) unique.push(e);
    else if (e.message_count > existing.message_count) {
      const idx = unique.indexOf(existing);
      unique[idx] = e;
    }
    return unique;
  }, []);

  return (
    <div className="card" style={{ padding: 18 }}>
      <div className="row between mb-3" style={{ alignItems: "center" }}>
        <div>
          <h3 style={{ fontSize: 14, fontWeight: 700, margin: 0, color: "var(--text-0)" }}>
            ⚖️ Evidence Containers &amp; Provenance Ledger
          </h3>
          <span className="muted" style={{ fontSize: 11 }}>
            Cryptographically sealed source containers associated with this case
          </span>
        </div>
        <button
          className="btn btn-ghost btn-sm"
          style={{ fontSize: 11, padding: "4px 10px" }}
          onClick={() => onNavigate("evidence")}
        >
          Manage Evidence Vault →
        </button>
      </div>

      <div style={{ overflowX: "auto" }}>
        <table style={{ width: "100%", borderCollapse: "collapse" }}>
          <thead>
            <tr>
              <th className="th">Container File</th>
              <th className="th" style={{ width: 85 }}>Format</th>
              <th className="th" style={{ width: 95 }}>Status</th>
              <th className="th" style={{ width: 100, textAlign: "right" }}>Messages</th>
              <th className="th" style={{ minWidth: 260 }}>SHA-256 Cryptographic Seal</th>
            </tr>
          </thead>
          <tbody>
            {uniqueEvidence.map((e) => (
              <tr key={e.id} className="tr-click">
                <td className="td">
                  <div className="row gap-2" style={{ alignItems: "center" }}>
                    <span style={{ fontSize: 14 }}>
                      {e.format.toLowerCase() === "imap"
                        ? "☁️"
                        : e.format.toLowerCase() === "pst"
                        ? "💾"
                        : e.format.toLowerCase() === "mbox"
                        ? "📦"
                        : "📄"}
                    </span>
                    <strong style={{ color: "var(--text-0)" }}>{e.filename}</strong>
                  </div>
                </td>
                <td className="td">
                  <span className="badge badge-blue" style={{ fontSize: 10 }}>
                    {e.format.toUpperCase()}
                  </span>
                </td>
                <td className="td">
                  <span
                    className={`badge badge-${
                      e.parse_status === "done"
                        ? "green"
                        : e.parse_status === "error"
                        ? "red"
                        : e.parse_status === "parsing"
                        ? "blue"
                        : "gray"
                    }`}
                    style={{ fontSize: 10, textTransform: "uppercase" }}
                  >
                    {e.parse_status}
                  </span>
                </td>
                <td className="td" style={{ textAlign: "right", fontFamily: "var(--mono)", fontWeight: 700 }}>
                  {e.message_count.toLocaleString()}
                </td>
                <td className="td">
                  <div className="row gap-2" style={{ alignItems: "center" }}>
                    <span
                      style={{
                        fontSize: 10,
                        fontFamily: "var(--mono)",
                        color: "var(--accent)",
                        background: "rgba(99, 102, 241, 0.08)",
                        padding: "2px 6px",
                        borderRadius: 3,
                        border: "1px solid rgba(99, 102, 241, 0.2)",
                        maxWidth: 240,
                        overflow: "hidden",
                        textOverflow: "ellipsis",
                        whiteSpace: "nowrap",
                        display: "inline-block",
                      }}
                      title={e.sha256}
                    >
                      {e.sha256}
                    </span>
                    <button
                      className="btn btn-ghost btn-xs"
                      onClick={() => handleCopy(e.sha256)}
                      title="Copy SHA-256 acquisition hash"
                      style={{ fontSize: 10, padding: "1px 6px" }}
                    >
                      {copiedHash === e.sha256 ? "✓ Copied" : "📋 Copy"}
                    </button>
                  </div>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
}
