import { useState, useRef, useEffect, useMemo } from "react";
import { invoke } from "@tauri-apps/api/core";
import { RichEmailBodyViewer } from "../components/RichEmailBodyViewer";
import { EmailDetailModal, EmailModalData } from "../components/EmailDetailModal";
import { BookmarkButton } from "../components/BookmarkButton";

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
  body_html: string | null;
  headers_raw: string | null;
}

type SortField = "date" | "from" | "subject" | "risk";

interface Props {
  caseId: string;
  evidenceFilter?: string | null;
  onSelectEmail?: (email: Email) => void;
  onViewEntity?: (email: string) => void;
}

const SEARCH_OPERATORS = [
  { op: "from:", desc: "Filter by sender address" },
  { op: "to:", desc: "Filter by recipient address" },
  { op: "subject:", desc: "Search subject lines" },
  { op: "body:", desc: "Email body text search" },
  { op: "has:attachment", desc: "Emails containing file attachments" },
  { op: "is:deleted", desc: "Recovered and soft-deleted messages" },
  { op: "risk:>50", desc: "High-risk threat scored messages" },
];

export function SearchView({ caseId, evidenceFilter, onViewEntity }: Props) {
  const [query, setQuery] = useState("");
  const [results, setResults] = useState<Email[]>([]);
  const [loading, setLoading] = useState(false);
  const [searched, setSearched] = useState(false);
  const [sortField, setSortField] = useState<SortField>("date");
  const [sortDir, setSortDir] = useState<"asc" | "desc">("desc");
  const [selectedEmail, setSelectedEmail] = useState<Email | null>(null);
  const [fullEmailData, setFullEmailData] = useState<EmailModalData | null>(null);
  const [showFullModal, setShowFullModal] = useState(false);
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    inputRef.current?.focus();
    doSearch("");
  }, [caseId, evidenceFilter]);

  const doSearch = async (searchQuery?: string) => {
    const q = searchQuery !== undefined ? searchQuery : query;
    setLoading(true);
    setSearched(true);
    setSelectedEmail(null);
    try {
      const res = await invoke<Email[]>("advanced_search", {
        input: { 
          case_id: caseId, 
          query: q.trim(), 
          limit: 500,
          evidence_id: evidenceFilter || undefined
        },
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

  const handleAddOperator = (op: string) => {
    setQuery((prev) => {
      const trimmed = prev.trim();
      return trimmed ? `${trimmed} ${op}` : op;
    });
    inputRef.current?.focus();
  };

  // When selectedEmail changes, fetch full email record to get complete body_html and inline images
  useEffect(() => {
    if (!selectedEmail) {
      setFullEmailData(null);
      return;
    }
    invoke<EmailModalData | null>("email_get", { input: { id: selectedEmail.id } })
      .then((data) => {
        if (data) setFullEmailData(data);
      })
      .catch((e) => console.error("Failed to load full email details:", e));
  }, [selectedEmail]);

  const sortedResults = useMemo(() => {
    let sorted = [...results];
    sorted.sort((a, b) => {
      let cmp = 0;
      switch (sortField) {
        case "date":
          cmp = (a.date_sent_utc || "").localeCompare(b.date_sent_utc || "");
          break;
        case "from":
          cmp = (cleanDisplayName(a.from_display) || a.from_addr).localeCompare(
            cleanDisplayName(b.from_display) || b.from_addr
          );
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

  const toggleSort = (field: SortField) => {
    if (sortField === field) {
      setSortDir((d) => (d === "asc" ? "desc" : "asc"));
    } else {
      setSortField(field);
      setSortDir("desc");
    }
  };

  return (
    <div className="view-content" style={{ display: "flex", flexDirection: "column", height: "100%" }}>
      {/* Search Header */}
      <div className="row between mb-3" style={{ flexWrap: "wrap", gap: 10 }}>
        <div>
          <h2 style={{ fontSize: 22, fontWeight: 700, color: "var(--text-0)" }}>
            ⚡ Advanced Search &amp; Discovery
          </h2>
          <p className="muted" style={{ fontSize: 12 }}>
            Instant forensic queries across all headers, bodies, senders, recipients, and reconstructed text.
          </p>
        </div>
      </div>

      {/* Main Search Input Box */}
      <div className="card mb-3" style={{ padding: 14 }}>
        <div className="row gap-2" style={{ alignItems: "center" }}>
          <div style={{ flex: 1, position: "relative" }}>
            <input
              ref={inputRef}
              className="input"
              style={{
                width: "100%",
                paddingLeft: 36,
                fontSize: 14,
                fontWeight: 500,
                background: "var(--bg-0)",
              }}
              placeholder="Search across all emails (e.g. from:enron.com has:attachment risk:>50 wire transfer)..."
              value={query}
              onChange={(e) => setQuery(e.target.value)}
              onKeyDown={handleKeyDown}
            />
            <span
              style={{
                position: "absolute",
                left: 12,
                top: "50%",
                transform: "translateY(-50%)",
                fontSize: 15,
                color: "var(--text-2)",
              }}
            >
              🔍
            </span>
          </div>
          <button
            className="btn btn-primary"
            disabled={loading}
            onClick={() => doSearch()}
            style={{ padding: "8px 20px" }}
          >
            {loading ? "Searching…" : "Search"}
          </button>
        </div>

        {/* Search Operators Suggestions */}
        <div className="row gap-2 mt-2" style={{ flexWrap: "wrap", alignItems: "center" }}>
          <span style={{ fontSize: 11, color: "var(--text-3)", fontWeight: 600 }}>
            QUICK OPERATORS:
          </span>
          {SEARCH_OPERATORS.map((op) => (
            <button
              key={op.op}
              className="btn btn-ghost btn-sm"
              style={{
                fontSize: 11,
                padding: "2px 8px",
                fontFamily: "var(--mono)",
                background: "var(--bg-2)",
                border: "1px solid var(--border)",
              }}
              onClick={() => handleAddOperator(op.op)}
              title={op.desc}
            >
              {op.op}
            </button>
          ))}
        </div>
      </div>

      {/* Results Section */}
      {searched && (
        <div
          style={{
            flex: 1,
            display: "grid",
            gridTemplateColumns: selectedEmail ? "1fr 500px" : "1fr",
            gap: 16,
            minHeight: 0,
          }}
        >
          {/* Results List Card */}
          <div className="card" style={{ padding: 0, overflow: "hidden", display: "flex", flexDirection: "column" }}>
            {/* Results Table Header */}
            <div
              style={{
                display: "grid",
                gridTemplateColumns: "160px 180px 1fr 95px 60px 65px",
                alignItems: "center",
                padding: "8px 14px",
                background: "var(--bg-2)",
                borderBottom: "1px solid var(--border)",
                fontSize: 10,
                fontWeight: 700,
                textTransform: "uppercase",
                letterSpacing: "0.06em",
                color: "var(--text-3)",
                gap: 6,
              }}
            >
              <div style={{ cursor: "pointer" }} onClick={() => toggleSort("from")}>
                Sender {sortField === "from" && (sortDir === "asc" ? "▲" : "▼")}
              </div>
              <div>Recipient(s)</div>
              <div style={{ cursor: "pointer" }} onClick={() => toggleSort("subject")}>
                Subject {sortField === "subject" && (sortDir === "asc" ? "▲" : "▼")}
              </div>
              <div style={{ textAlign: "right", cursor: "pointer" }} onClick={() => toggleSort("date")}>
                Date {sortField === "date" && (sortDir === "asc" ? "▲" : "▼")}
              </div>
              <div style={{ textAlign: "center", cursor: "pointer" }} onClick={() => toggleSort("risk")}>
                Risk {sortField === "risk" && (sortDir === "asc" ? "▲" : "▼")}
              </div>
              <div style={{ textAlign: "center" }}>Locker</div>
            </div>

            {/* Table Rows */}
            <div style={{ flex: 1, overflowY: "auto" }}>
              {sortedResults.length === 0 ? (
                <div style={{ padding: 40, textAlign: "center", color: "var(--text-2)" }}>
                  <span style={{ fontSize: 24 }}>🔍</span>
                  <div style={{ marginTop: 8, fontSize: 13 }}>No emails matched your query.</div>
                </div>
              ) : (
                sortedResults.map((em) => {
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
                        gridTemplateColumns: "160px 180px 1fr 95px 60px 65px",
                        alignItems: "center",
                        padding: "9px 14px",
                        borderBottom: "1px solid var(--border)",
                        background: isSelected ? "var(--accent-subtle)" : "transparent",
                        fontSize: 12,
                        gap: 6,
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
                          whiteSpace: "nowrap",
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
                          style={{ fontSize: 9, fontWeight: 700 }}
                        >
                          {em.risk_score}
                        </span>
                      </div>

                      {/* Tag / Locker Bookmark Button */}
                      <div
                        style={{ display: "flex", justifyContent: "center" }}
                        onClick={(e) => e.stopPropagation()}
                      >
                        <BookmarkButton
                          caseId={caseId}
                          itemId={em.id}
                          itemType="email"
                          compact={true}
                        />
                      </div>
                    </div>
                  );
                })
              )}
            </div>
          </div>

          {/* Split-Pane Message Inspector with Rich HTML & Photo Viewer */}
          {selectedEmail && (
            <div
              className="card mb-0"
              style={{
                padding: 16,
                maxHeight: "72vh",
                overflowY: "auto",
                borderLeft: "4px solid var(--accent)",
                display: "flex",
                flexDirection: "column",
                gap: 12,
              }}
            >
              {/* Top Controls Header */}
              <div className="row between" style={{ alignItems: "center" }}>
                <strong style={{ fontSize: 15, color: "var(--text-0)", overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap", flex: 1, paddingRight: 8 }}>
                  {selectedEmail.subject || "(no subject)"}
                </strong>
                <div className="row gap-1" style={{ alignItems: "center", flexShrink: 0 }}>
                  <BookmarkButton
                    caseId={caseId}
                    itemId={selectedEmail.id}
                    itemType="email"
                    compact={false}
                  />
                  {fullEmailData && (
                    <button
                      className="btn btn-secondary btn-sm"
                      style={{ fontSize: 11, padding: "3px 8px" }}
                      onClick={() => setShowFullModal(true)}
                      title="Open full expanded modal viewer"
                    >
                      ⛶ Expand
                    </button>
                  )}
                  <button
                    className="btn btn-ghost btn-sm"
                    style={{ padding: "3px 8px", fontSize: 11 }}
                    onClick={() => setSelectedEmail(null)}
                  >
                    ✕
                  </button>
                </div>
              </div>

              {/* Metadata Box */}
              <div
                style={{
                  background: "var(--bg-3)",
                  padding: 12,
                  borderRadius: "var(--r-md)",
                  display: "flex",
                  flexDirection: "column",
                  gap: 6,
                  fontSize: 12,
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

              {/* Rich Email Body Viewer (HTML, Clean Text, Raw MIME, Inline Photos) */}
              <div>
                <RichEmailBodyViewer
                  bodyText={fullEmailData?.body_text ?? selectedEmail.body_text}
                  bodyHtml={fullEmailData?.body_html ?? selectedEmail.body_html}
                  emailId={selectedEmail.id}
                  defaultMode="rendered"
                />
              </div>

              {/* Collapsible Transport Headers */}
              {(fullEmailData?.headers_raw || selectedEmail.headers_raw) && (
                <details style={{ marginTop: 8 }}>
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
                    {(fullEmailData?.headers_raw || selectedEmail.headers_raw || "").slice(0, 3000)}
                  </pre>
                </details>
              )}
            </div>
          )}
        </div>
      )}

      {/* Full Modal Viewer */}
      {showFullModal && fullEmailData && (
        <EmailDetailModal
          email={fullEmailData}
          onClose={() => setShowFullModal(false)}
          titleSuffix="Back to Search Results"
        />
      )}
    </div>
  );
}
