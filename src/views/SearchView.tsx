import { useState, useRef, useEffect, useMemo } from "react";
import { invoke } from "@tauri-apps/api/core";

function cleanDisplayName(name: string | null): string {
  if (!name) return "";
  let cleaned = name
    .replace(/@ENRON.*$/g, "")
    .replace(/IMCEANOTES-[^<]*/g, "")
    .replace(/<[^>]*>/g, "")
    .replace(/"/g, "")
    .replace(/\s+/g, " ")
    .trim();
  if (cleaned.includes("@")) {
    return cleaned.split("@")[0].trim() || cleaned;
  }
  return cleaned;
}

interface Email {
  id: string;
  evidence_id: string;
  message_id: string | null;
  from_addr: string;
  from_display: string | null;
  to_addrs: string;
  cc_addrs: string;
  subject: string | null;
  date_sent: string | null;
  date_sent_utc: string;
  folder_name: string | null;
  folder_category: string;
  is_deleted: boolean;
  deleted_recovered: boolean;
  risk_score: number;
  flags: string | null;
  body_text: string | null;
  headers_raw: string | null;
}

type SortField = "date" | "from" | "subject" | "risk";

interface Props {
  caseId: string;
  onSelectEmail?: (email: Email) => void;
  onViewEntity?: (email: string) => void;
}

export function SearchView({ caseId, onViewEntity }: Props) {
  const [query, setQuery] = useState("");
  const [results, setResults] = useState<Email[]>([]);
  const [loading, setLoading] = useState(false);
  const [searched, setSearched] = useState(false);
  const [sortField, setSortField] = useState<SortField>("date");
  const [sortDir, setSortDir] = useState<"asc" | "desc">("desc");
  const [selectedEmail, setSelectedEmail] = useState<Email | null>(null);
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    inputRef.current?.focus();
    // Run initial search to show recent emails
    doSearch("");
  }, [caseId]);

  const doSearch = async (searchQuery?: string) => {
    const q = searchQuery !== undefined ? searchQuery : query;
    setLoading(true);
    setSearched(true);
    setSelectedEmail(null);
    try {
      const res = await invoke<Email[]>("advanced_search", {
        input: { case_id: caseId, query: q.trim(), limit: 500 },
      });
      setResults(res || []);
    } catch (e) {
      console.error("Search failed:", e);
      setResults([]);
    } finally {
      setLoading(false);
    }
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Enter") doSearch();
  };

  const handleQuickSearch = (q: string) => {
    setQuery(q);
    doSearch(q);
  };

  const handleAddOperator = (op: string) => {
    setQuery((prev) => {
      const trimmed = prev.trim();
      return trimmed ? `${trimmed} ${op}` : op;
    });
    inputRef.current?.focus();
  };

  const sortedResults = useMemo(() => {
    let sorted = [...results];
    sorted.sort((a, b) => {
      let cmp = 0;
      switch (sortField) {
        case "date":
          cmp = (a.date_sent_utc || "").localeCompare(b.date_sent_utc || "");
          break;
        case "from":
          cmp = (a.from_display || a.from_addr).localeCompare(b.from_display || b.from_addr);
          break;
        case "subject":
          cmp = (a.subject || "").localeCompare(b.subject || "");
          break;
        case "risk":
          cmp = a.risk_score - b.risk_score;
          break;
      }
      return sortDir === "asc" ? cmp : -cmp;
    });
    return sorted;
  }, [results, sortField, sortDir]);

  const handleSort = (field: SortField) => {
    if (sortField === field) {
      setSortDir((d) => (d === "asc" ? "desc" : "asc"));
    } else {
      setSortField(field);
      setSortDir("desc");
    }
  };

  const quickPresets = [
    { label: "🚨 High Risk Flags", query: "risk:high" },
    { label: "🗑️ Deleted & Recovered", query: "is:deleted" },
    { label: "📎 Has Attachments", query: "has:attachment" },
    { label: "🔗 Contains URLs", query: "has:url" },
    { label: "📬 Sent Items", query: "folder:sent" },
    { label: "📥 Inbox Items", query: "folder:inbox" },
  ];

  const operatorChips = [
    { op: "from:", desc: "Sender contains (e.g. from:stacey)" },
    { op: "to:", desc: "Recipient contains (e.g. to:casey)" },
    { op: "subject:", desc: "Subject line keyword" },
    { op: "body:", desc: "Email body text search" },
    { op: "risk:>50", desc: "Risk score threshold" },
    { op: "is:deleted", desc: "Deleted emails only" },
    { op: "after:2001-08-01", desc: "Sent after date" },
    { op: "before:2002-01-01", desc: "Sent before date" },
    { op: "filename:pdf", desc: "Attachment filename" },
  ];

  return (
    <div>
      {/* Top Title */}
      <div className="row between mb-4">
        <div>
          <h2 style={{ fontSize: 22, fontWeight: 700, color: "var(--text-0)" }}>
            Forensic & eDiscovery Search
          </h2>
          <p className="muted" style={{ fontSize: 12 }}>
            Multi-field keyword indexing, Boolean operators, metadata targeting, and risk filtering.
          </p>
        </div>
      </div>

      {/* Main Search Bar Card */}
      <div className="card mb-3" style={{ padding: 16 }}>
        <div className="row gap-2 mb-3">
          <input
            ref={inputRef}
            className="input"
            style={{ flex: 1, fontSize: 14, padding: "10px 14px" }}
            placeholder='Search keywords or operators (e.g., from:stacey subject:urgent risk:>25 is:deleted)...'
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            onKeyDown={handleKeyDown}
          />
          <button
            className="btn btn-primary"
            style={{ padding: "0 20px" }}
            onClick={() => doSearch()}
            disabled={loading}
          >
            {loading ? "Searching..." : "🔍 Search"}
          </button>
          {query && (
            <button
              className="btn btn-ghost"
              onClick={() => {
                setQuery("");
                doSearch("");
              }}
            >
              Clear
            </button>
          )}
        </div>

        {/* Quick Search Preset Badges */}
        <div className="row gap-2 mb-3" style={{ flexWrap: "wrap" }}>
          <span style={{ fontSize: 11, fontWeight: 600, color: "var(--text-3)" }}>
            QUICK FILTERS:
          </span>
          {quickPresets.map((preset) => (
            <button
              key={preset.label}
              className="btn btn-ghost btn-sm"
              style={{
                fontSize: 11,
                padding: "3px 9px",
                background: query === preset.query ? "var(--accent-subtle)" : "var(--bg-3)",
                border: query === preset.query ? "1px solid var(--accent)" : "1px solid transparent",
              }}
              onClick={() => handleQuickSearch(preset.query)}
            >
              {preset.label}
            </button>
          ))}
        </div>

        {/* Search Operator Helper Chips */}
        <div>
          <details style={{ fontSize: 11, color: "var(--text-3)" }}>
            <summary style={{ cursor: "pointer", fontWeight: 600, userSelect: "none" }}>
              💡 Search Operators & Syntax Guide (Click to expand)
            </summary>
            <div
              style={{
                display: "grid",
                gridTemplateColumns: "repeat(auto-fill, minmax(230px, 1fr))",
                gap: 6,
                marginTop: 10,
              }}
            >
              {operatorChips.map((chip) => (
                <div
                  key={chip.op}
                  className="tr-click"
                  style={{
                    padding: "6px 10px",
                    background: "var(--bg-3)",
                    borderRadius: "var(--r-sm)",
                    fontSize: 11,
                  }}
                  onClick={() => handleAddOperator(chip.op)}
                >
                  <code style={{ color: "var(--accent)", fontWeight: 600 }}>{chip.op}</code>
                  <span style={{ color: "var(--text-3)", marginLeft: 6 }}>{chip.desc}</span>
                </div>
              ))}
            </div>
          </details>
        </div>
      </div>

      {/* Results Header */}
      <div className="row between mb-2">
        <span className="muted" style={{ fontSize: 12 }}>
          Found <strong>{results.length}</strong> matching message{results.length !== 1 ? "s" : ""}
        </span>

        {/* Sort Controls */}
        <div className="row gap-2">
          <span className="muted" style={{ fontSize: 11 }}>Sort:</span>
          {(
            [
              ["date", "Date"],
              ["from", "Sender"],
              ["subject", "Subject"],
              ["risk", "Risk Score"],
            ] as const
          ).map(([field, label]) => (
            <button
              key={field}
              className={`btn btn-sm ${sortField === field ? "btn-primary" : "btn-ghost"}`}
              style={{ fontSize: 11, padding: "2px 8px" }}
              onClick={() => handleSort(field as SortField)}
            >
              {label} {sortField === field ? (sortDir === "asc" ? "▲" : "▼") : ""}
            </button>
          ))}
        </div>
      </div>

      {/* Results Area */}
      {loading ? (
        <div className="card empty">Searching database...</div>
      ) : searched && results.length === 0 ? (
        <div className="card empty">No emails match the query "{query}"</div>
      ) : (
        <div
          style={{
            display: "grid",
            gridTemplateColumns: selectedEmail ? "1fr 420px" : "1fr",
            gap: 16,
            alignItems: "start",
          }}
        >
          {/* Results Table */}
          <div
            className="card mb-0"
            style={{
              padding: 0,
              overflow: "hidden",
              borderRadius: "var(--r-md)",
              border: "1px solid var(--border)",
            }}
          >
            {/* Table Header */}
            <div
              style={{
                display: "grid",
                gridTemplateColumns: "170px 200px 1fr 100px 65px",
                padding: "10px 14px",
                background: "var(--bg-1)",
                borderBottom: "1px solid var(--border)",
                fontSize: 10,
                fontWeight: 700,
                textTransform: "uppercase",
                letterSpacing: "0.06em",
                color: "var(--text-3)",
              }}
            >
              <div>Sender</div>
              <div>Recipient(s)</div>
              <div>Subject & Flags</div>
              <div style={{ textAlign: "right" }}>Date</div>
              <div style={{ textAlign: "center" }}>Risk</div>
            </div>

            {/* Table Rows */}
            <div style={{ maxHeight: "68vh", overflowY: "auto" }}>
              {sortedResults.map((em) => {
                const isSelected = selectedEmail?.id === em.id;
                let toList: string[] = [];
                try {
                  toList = em.to_addrs.startsWith("[")
                    ? JSON.parse(em.to_addrs)
                    : [em.to_addrs];
                } catch {
                  toList = [em.to_addrs];
                }

                return (
                  <div
                    key={em.id}
                    className="tr-click"
                    style={{
                      display: "grid",
                      gridTemplateColumns: "170px 200px 1fr 100px 65px",
                      alignItems: "center",
                      padding: "9px 14px",
                      borderBottom: "1px solid var(--border)",
                      background: isSelected ? "var(--accent-subtle)" : "transparent",
                      fontSize: 12,
                    }}
                    onClick={() => setSelectedEmail(isSelected ? null : em)}
                  >
                    {/* Sender */}
                    <div
                      style={{
                        overflow: "hidden",
                        textOverflow: "ellipsis",
                        whiteSpace: "nowrap",
                        color: "var(--text-0)",
                        fontWeight: 500,
                      }}
                      title={em.from_addr}
                    >
                      <span
                        style={{ cursor: "pointer" }}
                        onClick={(e) => {
                          e.stopPropagation();
                          onViewEntity?.(em.from_addr);
                        }}
                      >
                        {cleanDisplayName(em.from_display) || em.from_addr}
                      </span>
                    </div>

                    {/* Recipient */}
                    <div
                      style={{
                        overflow: "hidden",
                        textOverflow: "ellipsis",
                        whiteSpace: "nowrap",
                        color: "var(--text-2)",
                        fontFamily: "var(--mono)",
                        fontSize: 11,
                      }}
                      title={toList.join(", ")}
                    >
                      {toList.slice(0, 2).map(cleanDisplayName).join(", ")}
                      {toList.length > 2 && <span className="muted"> +{toList.length - 2}</span>}
                    </div>

                    {/* Subject */}
                    <div
                      style={{
                        overflow: "hidden",
                        textOverflow: "ellipsis",
                        whiteSpace: "nowrap",
                        color: "var(--text-0)",
                      }}
                      title={em.subject || ""}
                    >
                      {em.subject || <span className="muted">(no subject)</span>}
                      {em.deleted_recovered && (
                        <span className="badge badge-red" style={{ fontSize: 9, marginLeft: 6 }}>
                          DELETED
                        </span>
                      )}
                    </div>

                    {/* Date */}
                    <div
                      style={{
                        textAlign: "right",
                        fontSize: 11,
                        color: "var(--text-3)",
                      }}
                    >
                      {em.date_sent_utc ? new Date(em.date_sent_utc).toLocaleDateString() : "—"}
                    </div>

                    {/* Risk Score */}
                    <div style={{ textAlign: "center" }}>
                      <span
                        className={`badge ${
                          em.risk_score >= 50
                            ? "badge-red"
                            : em.risk_score >= 25
                            ? "badge-orange"
                            : "badge-green"
                        }`}
                        style={{ fontSize: 9 }}
                      >
                        {em.risk_score}
                      </span>
                    </div>
                  </div>
                );
              })}
            </div>
          </div>

          {/* Split-Pane Message Inspector */}
          {selectedEmail && (
            <div
              className="card mb-0"
              style={{
                padding: 16,
                maxHeight: "72vh",
                overflowY: "auto",
                borderLeft: "4px solid var(--accent)",
              }}
            >
              <div className="row between mb-3">
                <strong style={{ fontSize: 15, color: "var(--text-0)" }}>
                  {selectedEmail.subject || "(no subject)"}
                </strong>
                <button
                  className="btn btn-ghost btn-sm"
                  style={{ padding: "2px 6px", fontSize: 11 }}
                  onClick={() => setSelectedEmail(null)}
                >
                  ✕ Close
                </button>
              </div>

              {/* Metadata Grid */}
              <div
                style={{
                  background: "var(--bg-3)",
                  padding: 12,
                  borderRadius: "var(--r-md)",
                  display: "flex",
                  flexDirection: "column",
                  gap: 6,
                  fontSize: 12,
                  marginBottom: 12,
                }}
              >
                <div>
                  <span className="muted">From: </span>
                  <strong style={{ color: "var(--accent)" }}>
                    {selectedEmail.from_display
                      ? `${selectedEmail.from_display} <${selectedEmail.from_addr}>`
                      : selectedEmail.from_addr}
                  </strong>
                </div>

                <div>
                  <span className="muted">To: </span>
                  <span style={{ fontFamily: "var(--mono)", fontSize: 11 }}>
                    {selectedEmail.to_addrs}
                  </span>
                </div>

                {selectedEmail.cc_addrs && selectedEmail.cc_addrs !== "[]" && (
                  <div>
                    <span className="muted">CC: </span>
                    <span style={{ fontFamily: "var(--mono)", fontSize: 11 }}>
                      {selectedEmail.cc_addrs}
                    </span>
                  </div>
                )}

                <div className="row between" style={{ marginTop: 4 }}>
                  <div>
                    <span className="muted">Date: </span>
                    {selectedEmail.date_sent_utc
                      ? new Date(selectedEmail.date_sent_utc).toLocaleString()
                      : "—"}
                  </div>
                  <div>
                    <span className="muted">Risk: </span>
                    <span
                      className={`badge ${
                        selectedEmail.risk_score >= 50
                          ? "badge-red"
                          : selectedEmail.risk_score >= 25
                          ? "badge-orange"
                          : "badge-green"
                      }`}
                    >
                      {selectedEmail.risk_score}
                    </span>
                  </div>
                </div>
              </div>

              {/* Email Body Content */}
              <div>
                <span className="muted" style={{ fontSize: 11, fontWeight: 600 }}>
                  MESSAGE BODY:
                </span>
                <pre
                  style={{
                    background: "var(--bg-1)",
                    border: "1px solid var(--border)",
                    borderRadius: "var(--r-md)",
                    padding: 12,
                    fontSize: 12,
                    maxHeight: 250,
                    overflow: "auto",
                    whiteSpace: "pre-wrap",
                    marginTop: 6,
                    color: "var(--text-1)",
                  }}
                >
                  {selectedEmail.body_text || "(No message body)"}
                </pre>
              </div>

              {/* Collapsible Transport Headers */}
              {selectedEmail.headers_raw && (
                <details style={{ marginTop: 12 }}>
                  <summary
                    style={{
                      cursor: "pointer",
                      fontSize: 11,
                      fontWeight: 600,
                      color: "var(--text-3)",
                    }}
                  >
                    View Raw Transport Headers
                  </summary>
                  <pre
                    style={{
                      background: "var(--bg-1)",
                      border: "1px solid var(--border)",
                      borderRadius: "var(--r-sm)",
                      padding: 10,
                      fontSize: 10,
                      fontFamily: "var(--mono)",
                      maxHeight: 180,
                      overflow: "auto",
                      whiteSpace: "pre-wrap",
                      marginTop: 6,
                      color: "var(--text-2)",
                    }}
                  >
                    {selectedEmail.headers_raw.slice(0, 3000)}
                  </pre>
                </details>
              )}
            </div>
          )}
        </div>
      )}
    </div>
  );
}
