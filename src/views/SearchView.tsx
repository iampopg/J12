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

type SortField = "date" | "from" | "subject" | "risk";

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
  const [sortField, setSortField] = useState<SortField>("date");
  const [sortDir, setSortDir] = useState<"asc" | "desc">("desc");
  const [selectedEmailId, setSelectedEmailId] = useState<string | null>(null);
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    inputRef.current?.focus();
  }, []);

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

  const sorted = [...results].sort((a, b) => {
    let cmp = 0;
    switch (sortField) {
      case "date": cmp = (a.date_sent || "").localeCompare(b.date_sent || ""); break;
      case "from": cmp = a.from_addr.localeCompare(b.from_addr); break;
      case "subject": cmp = (a.subject || "").localeCompare(b.subject || ""); break;
      case "risk": cmp = a.risk_score - b.risk_score; break;
    }
    return sortDir === "asc" ? cmp : -cmp;
  });

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
    { op: "folder:sent", desc: "In folder" },
  ];

  return (
    <div>
      <div className="row between mb-4">
        <div>
          <h2 style={{ fontSize: 22, fontWeight: 700, color: "var(--text-0)" }}>Advanced Search</h2>
          <p className="muted">Search emails with operators and filters</p>
        </div>
      </div>

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

      {loading && <div className="empty">Searching...</div>}

      {!loading && searched && results.length === 0 && (
        <div className="empty">No emails match your search</div>
      )}

      {!loading && results.length > 0 && (
        <div>
          <div className="row between mb-4">
            <span className="muted">{results.length} result{results.length !== 1 ? "s" : ""}</span>
            <div className="row gap-2">
              <span className="muted text-sm">Sort by:</span>
              {([["date", "Date"], ["from", "From"], ["subject", "Subject"], ["risk", "Risk"]] as const).map(([field, label]) => (
                <button
                  key={field}
                  className={`btn btn-sm ${sortField === field ? "btn-primary" : "btn-ghost"}`}
                  onClick={() => {
                    if (sortField === field) setSortDir(d => d === "asc" ? "desc" : "asc");
                    else { setSortField(field as SortField); setSortDir("desc"); }
                  }}
                >
                  {label} {sortField === field ? (sortDir === "asc" ? "▲" : "▼") : ""}
                </button>
              ))}
            </div>
          </div>
          <div className="card">
            <table>
              <thead>
                <tr>
                  <th className="th" style={{ width: 180 }}>From</th>
                  <th className="th" style={{ width: 180 }}>To</th>
                  <th className="th">Subject</th>
                  <th className="th" style={{ width: 90 }}>Date</th>
                  <th className="th" style={{ width: 50 }}>Risk</th>
                  <th className="th" style={{ width: 60 }}>Attach</th>
                </tr>
              </thead>
              <tbody>
                {sorted.map((e) => {
                  const toList = JSON.parse(e.to_addrs || "[]");
                  return (
                    <tr key={e.id} className="tr-click" onClick={() => setSelectedEmailId(e.id)}>
                      <td className="td" style={{ fontSize: 12 }}>
                        <ClickableEmail addr={e.from_addr} name={e.from_display} onView={onViewEntity} />
                      </td>
                      <td className="td" style={{ fontSize: 12, fontFamily: "var(--mono)", color: "var(--text-2)" }}>
                        {toList.slice(0, 2).join(", ")}
                        {toList.length > 2 && <span className="muted"> +{toList.length - 2}</span>}
                      </td>
                      <td className="td">{e.subject || <span className="muted">(no subject)</span>}</td>
                      <td className="td muted">{e.date_sent ? new Date(e.date_sent).toLocaleDateString() : "—"}</td>
                      <td className="td">
                        <span className={`badge ${e.risk_score >= 50 ? "badge-red" : e.risk_score >= 25 ? "badge-orange" : "badge-green"}`}>
                          {e.risk_score}
                        </span>
                      </td>
                      <td className="td"><HasAttachments emailId={e.id} /></td>
                    </tr>
                  );
                })}
              </tbody>
             </table>
          </div>
        </div>
      )}

      {/* Email detail modal */}
      {selectedEmailId && <EmailDetailModal emailId={selectedEmailId} onClose={() => setSelectedEmailId(null)} />}
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

// Email detail modal for search results
function EmailDetailModal({ emailId, onClose }: { emailId: string; onClose: () => void }) {
  const [email, setEmail] = useState<any>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    invoke<any>("email_get", { input: { case_id: emailId } })
      .then(data => { setEmail(data); setLoading(false); })
      .catch(() => setLoading(false));
  }, [emailId]);

  if (loading) return <Modal title="Loading..." onClose={onClose}><div className="empty">Loading...</div></Modal>;
  if (!email) return <Modal title="Error" onClose={onClose}><div className="empty">Email not found</div></Modal>;

  let toList: string[] = [];
  try { toList = JSON.parse(email.to_addrs || "[]"); } catch {}

  return (
    <Modal title={email.subject || "(no subject)"} onClose={onClose}>
      <div style={{ fontSize: 13 }}>
        <div className="grid-2 mb-4">
          <div><span className="muted">From:</span> <strong>{email.from_display || email.from_addr}</strong></div>
          <div><span className="muted">Date:</span> {email.date_sent ? new Date(email.date_sent).toLocaleString() : "—"}</div>
        </div>
        <div className="mb-4"><span className="muted">To:</span> {toList.join(", ")}</div>
        <div className="mb-4"><span className="muted">Risk Score:</span> <span className={`badge ${email.risk_score >= 50 ? "badge-red" : email.risk_score >= 25 ? "badge-orange" : "badge-green"}`}>{email.risk_score}</span></div>
        {email.headers_raw && (
          <details className="mb-4">
            <summary style={{ cursor: "pointer", fontWeight: 600, marginBottom: 8 }}>View Headers</summary>
            <pre style={{ background: "var(--bg-3)", padding: 12, borderRadius: "var(--r-sm)", fontSize: 11, maxHeight: 200, overflow: "auto", whiteSpace: "pre-wrap" }}>{email.headers_raw.slice(0, 3000)}</pre>
          </details>
        )}
        {email.body_text && (
          <div>
            <span className="muted">Body:</span>
            <pre style={{ background: "var(--bg-3)", padding: 16, borderRadius: "var(--r-md)", fontSize: 13, marginTop: 8, maxHeight: 300, overflow: "auto", whiteSpace: "pre-wrap" }}>{email.body_text.slice(0, 5000)}</pre>
          </div>
        )}
      </div>
    </Modal>
  );
}

function Modal({ title, onClose, children }: { title: string; onClose: () => void; children: React.ReactNode }) {
  return (
    <div style={{ position: "fixed", top: 0, left: 0, right: 0, bottom: 0, background: "rgba(0,0,0,0.7)", display: "flex", alignItems: "center", justifyContent: "center", zIndex: 1000 }} onClick={onClose}>
      <div style={{ background: "var(--bg-2)", borderRadius: "var(--r-lg)", padding: 24, maxWidth: 800, width: "90%", maxHeight: "80vh", overflow: "auto", border: "1px solid var(--border)" }} onClick={e => e.stopPropagation()}>
        <div className="row between mb-4">
          <h3 style={{ fontSize: 18, fontWeight: 600 }}>{title}</h3>
          <button className="btn btn-ghost btn-sm" onClick={onClose}>✕</button>
        </div>
        {children}
      </div>
    </div>
  );
}
