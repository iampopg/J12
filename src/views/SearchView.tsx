import { useState, useRef, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";

interface Email {
  id: string;
  message_id: string | null;
  from_addr: string;
  from_display: string | null;
  to_addrs: string;
  subject: string | null;
  date_sent: string | null;
  folder_category: string;
  risk_score: number;
}

interface Props {
  caseId: string;
  onSelectEmail?: (email: Email) => void;
  onViewEntity?: (email: string) => void;
}

export function SearchView({ caseId, onSelectEmail, onViewEntity }: Props) {
  const [query, setQuery] = useState("");
  const [results, setResults] = useState<Email[]>([]);
  const [loading, setLoading] = useState(false);
  const [searched, setSearched] = useState(false);
  const inputRef = useRef<HTMLInputElement>(null);

  const doSearch = async () => {
    if (!query.trim()) return;
    setLoading(true);
    setSearched(true);
    try {
      const res = await invoke<Email[]>("advanced_search", {
        input: { case_id: caseId, query: query.trim(), limit: 500 }
      });
      setResults(res);
    } catch (e) {
      console.error("Search failed:", e);
    }
    setLoading(false);
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Enter") doSearch();
  };

  const operators = [
    { op: "from:", desc: "Sender contains" },
    { op: "to:", desc: "Recipient contains" },
    { op: "subject:", desc: "Subject contains" },
    { op: "body:", desc: "Body contains" },
    { op: "domain:", desc: "Domain in any address" },
    { op: "after:2001-06-01", desc: "Sent after date" },
    { op: "before:2002-01-01", desc: "Sent before date" },
    { op: "risk:>50", desc: "Risk score above N" },
    { op: "has:attachment", desc: "Has attachments" },
    { op: "has:url", desc: "Contains URLs" },
    { op: "ip:192.168", desc: "IP in headers" },
    { op: "hash:abc123", desc: "Attachment hash" },
    { op: "filename:report", desc: "Attachment name" },
    { op: "folder:sent", desc: "In sent folder" },
  ];

  return (
    <div>
      <div className="row between mb-4">
        <div>
          <h2 style={{ fontSize: 22, fontWeight: 700, color: "var(--text-0)" }}>Advanced Search</h2>
          <p className="muted">Search emails with operators and filters</p>
        </div>
      </div>

      {/* Search Input */}
      <div className="card mb-4">
        <div className="row gap-2">
          <input
            ref={inputRef}
            className="input"
            style={{ flex: 1, fontSize: 16, padding: "12px 16px" }}
            placeholder='Search... e.g. from:enron.com subject:urgent has:attachment'
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            onKeyDown={handleKeyDown}
          />
          <button className="btn btn-primary" onClick={doSearch} disabled={loading}>
            {loading ? "Searching..." : "🔍 Search"}
          </button>
        </div>

        {/* Operator Hints */}
        <div style={{ marginTop: 16 }}>
          <div style={{ fontSize: 11, color: "var(--text-3)", marginBottom: 8 }}>SEARCH OPERATORS (click to add)</div>
          <div style={{ display: "grid", gridTemplateColumns: "repeat(auto-fill, minmax(220px, 1fr))", gap: 6 }}>
            {operators.map((op) => (
              <div
                key={op.op}
                style={{ padding: "6px 10px", background: "var(--bg-3)", borderRadius: "var(--r-sm)", cursor: "pointer", fontSize: 12 }}
                onClick={() => setQuery(prev => prev + (prev.endsWith(" ") || prev === "" ? "" : " ") + op.op)}
              >
                <code style={{ color: "var(--accent)", fontFamily: "var(--mono)" }}>{op.op}</code>
                <span style={{ color: "var(--text-3)", marginLeft: 8, fontSize: 11 }}>{op.desc}</span>
              </div>
            ))}
          </div>
        </div>
      </div>

      {/* Results */}
      {loading && <div className="empty">Searching...</div>}

      {!loading && searched && results.length === 0 && (
        <div className="empty">No emails match your search</div>
      )}

      {!loading && results.length > 0 && (
        <div>
          <div className="row between mb-4">
            <span className="muted">{results.length} result{results.length !== 1 ? "s" : ""}</span>
            <span className="muted text-sm">Click row to view · Click email to see person</span>
          </div>
          <div className="card">
            <table>
              <thead>
                <tr>
                  <th className="th">From</th>
                  <th className="th">To</th>
                  <th className="th">Subject</th>
                  <th className="th" style={{ width: 90 }}>Date</th>
                  <th className="th" style={{ width: 50 }}>Risk</th>
                  <th className="th" style={{ width: 60 }}>Attach</th>
                </tr>
              </thead>
              <tbody>
                {results.map((e) => (
                  <tr key={e.id} className="tr-click" onClick={() => onSelectEmail?.(e)}>
                    <td className="td">
                      <ClickableEmail addr={e.from_addr} name={e.from_display} onView={onViewEntity} />
                    </td>
                    <td className="td" style={{ fontSize: 11, maxWidth: 150, overflow: "hidden", textOverflow: "ellipsis" }}>
                      {(JSON.parse(e.to_addrs || "[]")[0] || "").slice(0, 25)}
                    </td>
                    <td className="td">
                      {e.subject || <span className="muted">(no subject)</span>}
                    </td>
                    <td className="td muted">
                      {e.date_sent ? new Date(e.date_sent).toLocaleDateString() : "—"}
                    </td>
                    <td className="td">
                      <span className={`badge ${e.risk_score >= 50 ? "badge-red" : e.risk_score >= 25 ? "badge-orange" : "badge-green"}`}>
                        {e.risk_score}
                      </span>
                    </td>
                    <td className="td">
                      <HasAttachments emailId={e.id} />
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </div>
      )}
    </div>
  );
}

function ClickableEmail({ addr, name, onView }: { addr: string; name: string | null; onView?: (email: string) => void }) {
  return (
    <span
      style={{ color: "var(--accent)", cursor: "pointer", textDecoration: "underline" }}
      onClick={(e) => { e.stopPropagation(); onView?.(addr); }}
      title="Click to view person profile"
    >
      {name || addr}
    </span>
  );
}

function HasAttachments({ emailId }: { emailId: string }) {
  const [count, setCount] = useState(0);

  useEffect(() => {
    invoke<any[]>("email_attachments", { emailId })
      .then(a => setCount(a.length))
      .catch(() => setCount(0));
  }, [emailId]);

  if (count === 0) return <span className="muted">—</span>;
  return <span className="badge badge-blue">📎 {count}</span>;
}
