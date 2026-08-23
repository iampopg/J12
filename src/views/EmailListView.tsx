import { useState, useMemo, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";

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
  body_text: string | null;
  body_html: string | null;
  headers_raw: string | null;
  folder_name: string | null;
  folder_category: string;
  recovery_status: string;
  deleted_recovered: boolean;
  risk_score: number;
}

interface Evidence { id: string; filename: string; }

type SortField = "date" | "from" | "subject";
type SortDir = "asc" | "desc";

export function EmailListView({ caseId, filter }: { caseId: string; filter?: string }) {
  const [emails, setEmails] = useState<Email[]>([]);
  const [evidence, setEvidence] = useState<Evidence[]>([]);
  const [loading, setLoading] = useState(true);
  const [q, setQ] = useState("");
  const [sortField, setSortField] = useState<SortField>("date");
  const [sortDir, setSortDir] = useState<SortDir>("desc");
  const [selected, setSelected] = useState<Email | null>(null);
  const [showUnique, setShowUnique] = useState(false);

  const load = async () => {
    setLoading(true);
    try {
      const [em, ev] = await Promise.all([
        invoke<Email[]>("email_list", { input: { case_id: caseId, limit: 10000 } }),
        invoke<Evidence[]>("evidence_list", { input: { case_id: caseId } }),
      ]);
      setEmails(em);
      setEvidence(ev);
    } catch (e) { console.error(e); }
    finally { setLoading(false); }
  };

  useEffect(() => { load(); }, []);

  const evidenceMap = useMemo(() => {
    const m = new Map<string, Evidence>();
    evidence.forEach(e => m.set(e.id, e));
    return m;
  }, [evidence]);

  // Apply folder filter based on folder_category from database
  const uniqueEmails = useMemo(() => {
    if (!showUnique) return emails;
    const seen = new Set<string>();
    return emails.filter(e => {
      const key = e.message_id || `${e.from_addr}-${e.subject}-${e.date_sent}`;
      if (seen.has(key)) return false;
      seen.add(key);
      return true;
    });
  }, [emails, showUnique]);

  // Apply folder filter based on folder_category from database
  const filteredByFolder = useMemo(() => {
    if (!filter || filter === "all") return uniqueEmails;
    if (filter === "sent") {
      return uniqueEmails.filter(e => e.folder_category === "sent");
    }
    if (filter === "inbox") {
      return uniqueEmails.filter(e => e.folder_category === "inbox" || e.folder_category === "other");
    }
    if (filter === "soft_deleted") {
      return uniqueEmails.filter(e => e.folder_category === "soft_deleted" || e.recovery_status === "soft_deleted");
    }
    if (filter === "hard_deleted") {
      return uniqueEmails.filter(e => e.recovery_status === "hard_deleted" || e.recovery_status === "purged");
    }
    if (filter === "recoverable") {
      return uniqueEmails.filter(e => e.recovery_status === "recoverable");
    }
    if (filter === "drafts") {
      return uniqueEmails.filter(e => e.folder_category === "drafts");
    }
    if (filter === "spam") {
      return uniqueEmails.filter(e => e.folder_category === "spam");
    }
    if (filter === "other") {
      return uniqueEmails.filter(e => e.folder_category === "other");
    }
    return uniqueEmails;
  }, [uniqueEmails, filter]);

  const filtered = useMemo(() => {
    let result = filteredByFolder;
    if (q) {
      const qq = q.toLowerCase();
      result = result.filter(e =>
        (e.subject || "").toLowerCase().includes(qq) ||
        e.from_addr.toLowerCase().includes(qq) ||
        (e.body_text || "").toLowerCase().includes(qq)
      );
    }
    result = [...result].sort((a, b) => {
      let cmp = 0;
      switch (sortField) {
        case "date": cmp = (a.date_sent || "").localeCompare(b.date_sent || ""); break;
        case "from": cmp = a.from_addr.localeCompare(b.from_addr); break;
        case "subject": cmp = (a.subject || "").localeCompare(b.subject || ""); break;
      }
      return sortDir === "asc" ? cmp : -cmp;
    });
    return result;
  }, [filteredByFolder, q, sortField, sortDir]);

  const toggleSort = (field: SortField) => {
    if (sortField === field) setSortDir(d => d === "asc" ? "desc" : "asc");
    else { setSortField(field); setSortDir("desc"); }
  };

  const SortIcon = ({ field }: { field: SortField }) => (
    <span style={{ opacity: sortField === field ? 1 : 0.3, marginLeft: 4, fontSize: 10 }}>
      {sortField === field ? (sortDir === "asc" ? "▲" : "▼") : "⇅"}
    </span>
  );

  if (loading) return <div className="empty">Loading emails...</div>;

  return (
    <div>
      {selected ? (
        <EmailDetail email={selected} evidenceName={evidenceMap.get(selected.evidence_id)?.filename} onClose={() => setSelected(null)} />
      ) : (
        <>
          <div className="row between mb-4">
            <div>
              <h2 style={{ fontSize: 22, fontWeight: 700, color: "var(--text-0)" }}>
                {filter === "sent" ? "Sent" : filter === "deleted" ? "Deleted" : "All Emails"} ({filtered.length.toLocaleString()})
              </h2>
              <p className="muted">Click any row for forensic analysis</p>
            </div>
            <div className="row gap-2">
              <label className="row gap-2" style={{ fontSize: 12, color: "var(--text-2)", cursor: "pointer" }}>
                <input type="checkbox" checked={showUnique} onChange={e => setShowUnique(e.target.checked)} />
                Unique only
              </label>
              <button className="btn btn-ghost btn-sm" onClick={load}>↻ Refresh</button>
            </div>
          </div>

          <input className="input mb-4" placeholder="Search subject, sender, body..." value={q} onChange={(e) => setQ(e.target.value)} />

          <div className="card">
            <table>
              <thead>
                <tr>
                  <th className="th sort-header" onClick={() => toggleSort("from")}>From <SortIcon field="from" /></th>
                  <th className="th sort-header" onClick={() => toggleSort("subject")}>Subject <SortIcon field="subject" /></th>
                  <th className="th sort-header" onClick={() => toggleSort("date")}>Date <SortIcon field="date" /></th>
                  <th className="th" style={{ width: 60 }}>Deleted</th>
                </tr>
              </thead>
              <tbody>
                {filtered.slice(0, 500).map((e) => (
                  <tr key={e.id} className="tr-click" onClick={() => setSelected(e)}>
                    <td className="td from-cell">
                      <span style={{ color: "var(--text-1)" }}>{e.from_display || e.from_addr}</span>
                    </td>
                    <td className="td subject-cell">
                      {e.subject || <span className="muted">(no subject)</span>}
                    </td>
                    <td className="td muted date-cell">
                      {e.date_sent ? new Date(e.date_sent).toLocaleDateString() : "—"}
                    </td>
                    <td className="td">{e.deleted_recovered && <span className="badge badge-red">DEL</span>}</td>
                  </tr>
                ))}
              </tbody>
            </table>
            {filtered.length === 0 && <div className="empty">No emails match filters</div>}
            {filtered.length > 500 && <p className="muted text-sm mt-4">Showing 500 of {filtered.length}</p>}
          </div>
        </>
      )}
    </div>
  );
}

function EmailDetail({ email, evidenceName, onClose }: { email: Email; evidenceName?: string; onClose: () => void }) {
  const [tab, setTab] = useState<"overview" | "headers" | "auth" | "mime" | "raw" | "attachments">("overview");
  const [analysisData, setAnalysisData] = useState<any>(null);
  const [analysisLoading, setAnalysisLoading] = useState(false);

  // Load analysis data when auth or headers tab is selected
  useEffect(() => {
    if ((tab === "auth" || tab === "headers") && !analysisData && !analysisLoading) {
      setAnalysisLoading(true);
      invoke<any>("email_headers", { emailId: email.id })
        .then(data => setAnalysisData(data))
        .catch(console.error)
        .finally(() => setAnalysisLoading(false));
    }
  }, [tab]);

  let toList: string[] = [];
  let ccList: string[] = [];
  try { toList = JSON.parse(email.to_addrs || "[]"); } catch {}
  try { ccList = JSON.parse(email.cc_addrs || "[]"); } catch {}

  // Parse headers preserving order and duplicates
  const parsedHeaders = (email.headers_raw || "").split("\n").reduce((acc, line) => {
    const colonIdx = line.indexOf(":");
    if (colonIdx > 0) {
      const key = line.substring(0, colonIdx).trim();
      const value = line.substring(colonIdx + 1).trim();
      acc.push({ key, value });
    }
    return acc;
  }, [] as Array<{ key: string; value: string }>);

  // Extract Received chain
  const receivedChain = (email.headers_raw || "").split("\n").filter(l => l.trim().startsWith("Received:"));

  // Extract Authentication-Results
  const authResults = (email.headers_raw || "").split("\n").filter(l => l.trim().startsWith("Authentication-Results:"));

  // Extract URLs from body
  const urls = (email.body_text || "").match(/https?:\/\/[^\s<>"')]+/gi) || [];

  // Extract IPs from headers
  const ips = (email.headers_raw || "").match(/\b(?:\d{1,3}\.){3}\d{1,1,3}\b/g) || [];

  // Risk score color
  const riskColor = email.risk_score >= 50 ? "var(--red)" : email.risk_score >= 25 ? "var(--yellow)" : "var(--green)";
  const riskLabel = email.risk_score >= 50 ? "HIGH" : email.risk_score >= 25 ? "MEDIUM" : "LOW";

  const tabs = [
    { key: "overview", label: "Overview" },
    { key: "headers", label: "Headers" },
    { key: "auth", label: "Authentication" },
    { key: "mime", label: "MIME" },
    { key: "raw", label: "Raw" },
    { key: "attachments", label: "Attachments" },
  ];

  return (
    <div>
      <div className="row between mb-4">
        <div style={{ flex: 1, minWidth: 0 }}>
          <h2 style={{ fontSize: 16, fontWeight: 600, color: "var(--text-0)" }}>{email.subject || "(no subject)"}</h2>
          <p className="muted" style={{ fontSize: 12 }}>From: {email.from_addr} · {email.date_sent ? new Date(email.date_sent).toLocaleString() : "—"}</p>
          {evidenceName && <p className="muted" style={{ fontSize: 11 }}>Source: {evidenceName}</p>}
        </div>
        <button className="btn btn-ghost btn-sm" onClick={onClose}>← Back</button>
      </div>

      <div className="row gap-2 mb-4" style={{ borderBottom: "1px solid var(--border)", paddingBottom: 0 }}>
        {tabs.map((t) => (
          <button key={t.key} className={`btn btn-sm ${tab === t.key ? "btn-primary" : "btn-ghost"}`} style={{ borderRadius: "6px 6px 0 0" }} onClick={() => setTab(t.key as any)}>
            {t.label}
          </button>
        ))}
      </div>

      <div className="card" style={{ marginTop: 0 }}>
        {tab === "overview" && (
          <div>
            <div className="grid-2 mb-4">
              <div><span className="muted">From</span><p style={{ fontWeight: 500 }}>{email.from_display || email.from_addr}</p></div>
              <div><span className="muted">Date</span><p>{email.date_sent ? new Date(email.date_sent).toLocaleString() : "—"}</p></div>
            </div>
            <div className="mb-4"><span className="muted">To</span><p className="mono">{toList.join(", ")}</p></div>
            {ccList.length > 0 && <div className="mb-4"><span className="muted">CC</span><p className="mono">{ccList.join(", ")}</p></div>}
            <div className="mb-4"><span className="muted">Message-ID</span><p className="mono text-sm">{email.message_id || "—"}</p></div>
            <div className="mb-4">
              <span className="muted">Risk Score</span>
              <p style={{ fontWeight: 600, color: riskColor }}>{email.risk_score}/100 ({riskLabel})</p>
            </div>
            {email.deleted_recovered && <div className="mb-4"><span className="badge badge-red">DELETED / RECOVERED</span></div>}
            {ips.length > 0 && (
              <div className="mb-4"><span className="muted">Extracted IPs</span><div className="row gap-2 mt-4" style={{ flexWrap: "wrap" }}>{[...new Set(ips)].map((ip, i) => <span key={i} className="badge badge-gray mono">{ip}</span>)}</div></div>
            )}
            {urls.length > 0 && (
              <div className="mb-4"><span className="muted">Extracted URLs ({urls.length})</span>{urls.slice(0, 10).map((url, i) => <p key={i} className="mono text-sm" style={{ color: "var(--accent)", wordBreak: "break-all", marginTop: 4 }}>{url}</p>)}</div>
            )}
            {email.body_text && (
              <div>
                <span className="muted">Body</span>
                <pre style={{ background: "var(--bg-0)", border: "1px solid var(--border)", borderRadius: "var(--r-md)", padding: 16, fontSize: 13, marginTop: 8, maxHeight: 300, overflow: "auto", whiteSpace: "pre-wrap" }}>
                  {email.body_text.slice(0, 5000)}
                </pre>
              </div>
            )}
          </div>
        )}

        {tab === "headers" && (
          <div>
            {analysisLoading && <div className="empty">Analyzing headers...</div>}
            {analysisData?.header_analysis && (
              <div className="mb-4">
                <h4 style={{ fontSize: 13, fontWeight: 600, marginBottom: 12 }}>Header Analysis Summary</h4>
                <div className="analysis-summary">
                  <div className="analysis-stat">
                    <div className="analysis-stat-val">{analysisData.header_analysis.received_chain?.length || 0}</div>
                    <div className="analysis-stat-label">Received Hops</div>
                  </div>
                  <div className="analysis-stat">
                    <div className="analysis-stat-val" style={{ fontSize: 14 }}>
                      {analysisData.header_analysis.originating_ip || "Unknown"}
                    </div>
                    <div className="analysis-stat-label">Originating IP</div>
                  </div>
                  <div className="analysis-stat">
                    <div className="analysis-stat-val" style={{ color: analysisData.header_analysis.routing_anomalies?.length > 0 ? "var(--danger)" : "var(--text-0)" }}>
                      {analysisData.header_analysis.routing_anomalies?.length || 0}
                    </div>
                    <div className="analysis-stat-label">Routing Anomalies</div>
                  </div>
                  <div className="analysis-stat">
                    <div className="analysis-stat-val" style={{ color: analysisData.header_analysis.clock_skew?.length > 0 ? "var(--warning)" : "var(--text-0)" }}>
                      {analysisData.header_analysis.clock_skew?.length || 0}
                    </div>
                    <div className="analysis-stat-label">Clock Skew Events</div>
                  </div>
                </div>
                {analysisData.header_analysis.routing_anomalies?.length > 0 && (
                  <div className="mt-4">
                    <h4 style={{ fontSize: 13, fontWeight: 600, marginBottom: 8, color: "var(--danger)" }}>Routing Anomalies Detected</h4>
                    {analysisData.header_analysis.routing_anomalies.map((a: any, i: number) => (
                      <div key={i} className={`finding-card finding-card--${a.severity || 'medium'}`}>
                        <span className={`badge badge-${a.severity === 'critical' || a.severity === 'high' ? 'red' : 'orange'}`}>{a.severity?.toUpperCase()}</span>
                        <p style={{ marginTop: 8, fontSize: 13 }}>{a.description}</p>
                      </div>
                    ))}
                  </div>
                )}
                {analysisData.header_analysis.clock_skew?.length > 0 && (
                  <div className="mt-4">
                    <h4 style={{ fontSize: 13, fontWeight: 600, marginBottom: 8, color: "var(--warning)" }}>Clock Skew Detected</h4>
                    {analysisData.header_analysis.clock_skew.map((s: any, i: number) => (
                      <div key={i} className="finding-card finding-card--medium">
                        <p style={{ fontSize: 12 }}>
                          {s.hop_from} → {s.hop_to}: {s.skew_seconds}s skew
                        </p>
                      </div>
                    ))}
                  </div>
                )}
              </div>
            )}
            {analysisData?.header_analysis?.received_chain?.length > 0 && (
              <div className="mb-4">
                <h4 style={{ fontSize: 13, fontWeight: 600, marginBottom: 12 }}>Received Chain (bottom = oldest)</h4>
                {analysisData.header_analysis.received_chain.map((hop: any, i: number) => (
                  <div key={i} style={{ padding: 12, background: "var(--bg-3)", borderRadius: "var(--r-sm)", marginBottom: 8, fontSize: 12, fontFamily: "var(--mono)", wordBreak: "break-all" }}>
                    <div style={{ display: "flex", justifyContent: "space-between", marginBottom: 4 }}>
                      <strong>Hop {i + 1}</strong>
                      {hop.transit_time_seconds !== null && hop.transit_time_seconds !== undefined && (
                        <span style={{ color: hop.transit_time_seconds < 0 ? "var(--red)" : "var(--text-3)" }}>
                          {hop.transit_time_seconds > 0 ? "+" : ""}{hop.transit_time_seconds}s
                        </span>
                      )}
                    </div>
                    {hop.from && <div><span className="muted">From:</span> {hop.from}</div>}
                    {hop.by && <div><span className="muted">By:</span> {hop.by}</div>}
                    {hop.with && <div><span className="muted">With:</span> {hop.with}</div>}
                    {hop.timestamp && <div><span className="muted">Time:</span> {hop.timestamp}</div>}
                  </div>
                ))}
              </div>
            )}
            <h4 style={{ fontSize: 13, fontWeight: 600, marginBottom: 12 }}>All Headers ({parsedHeaders.length})</h4>
            <table style={{ fontSize: 12 }}>
              <tbody>
                {parsedHeaders.map((h, i) => (
                  <tr key={i}>
                    <td style={{ padding: "6px 12px", color: "var(--text-3)", fontWeight: 600, verticalAlign: "top", whiteSpace: "nowrap", borderBottom: "1px solid var(--border)", width: 200 }}>{h.key}</td>
                    <td style={{ padding: "6px 12px", wordBreak: "break-all", borderBottom: "1px solid var(--border)", fontFamily: "var(--mono)" }}>{h.value}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}

        {tab === "auth" && (
          <div>
            {analysisLoading && <div className="empty">Running authentication analysis...</div>}
            {analysisData?.auth_results && (
              <div>
                {/* SPF */}
                <div className="mb-4" style={{ padding: 16, background: "var(--bg-3)", borderRadius: "var(--r-md)" }}>
                  <div className="row between">
                    <h4 style={{ fontSize: 14, fontWeight: 600 }}>SPF</h4>
                    <span className={`badge ${analysisData.auth_results.spf.result === 'pass' ? 'badge-green' : analysisData.auth_results.spf.result === 'fail' ? 'badge-red' : analysisData.auth_results.spf.result === 'softfail' ? 'badge-orange' : 'badge-gray'}`}>
                      {analysisData.auth_results.spf.result?.toUpperCase() || "NONE"}
                    </span>
                  </div>
                  <p className="muted text-sm mt-4">{analysisData.auth_results.spf.detail}</p>
                  {analysisData.auth_results.spf.domain && (
                    <p className="mono text-sm mt-4">Domain: {analysisData.auth_results.spf.domain}</p>
                  )}
                </div>

                {/* DKIM */}
                {analysisData.auth_results.dkim?.length > 0 && analysisData.auth_results.dkim.map((dkim: any, i: number) => (
                  <div key={i} className="mb-4" style={{ padding: 16, background: "var(--bg-3)", borderRadius: "var(--r-md)" }}>
                    <div className="row between">
                      <h4 style={{ fontSize: 14, fontWeight: 600 }}>DKIM #{i + 1}</h4>
                      <span className={`badge ${dkim.result === 'pass' ? 'badge-green' : dkim.result === 'fail' ? 'badge-red' : 'badge-gray'}`}>
                        {dkim.result?.toUpperCase() || "NONE"}
                      </span>
                    </div>
                    <p className="muted text-sm mt-4">{dkim.detail}</p>
                    {dkim.domain && <p className="mono text-sm mt-4">Domain: {dkim.domain}</p>}
                  </div>
                ))}

                {/* DMARC */}
                <div className="mb-4" style={{ padding: 16, background: "var(--bg-3)", borderRadius: "var(--r-md)" }}>
                  <div className="row between">
                    <h4 style={{ fontSize: 14, fontWeight: 600 }}>DMARC</h4>
                    <span className={`badge ${analysisData.auth_results.dmarc.result === 'pass' ? 'badge-green' : analysisData.auth_results.dmarc.result === 'fail' ? 'badge-red' : 'badge-gray'}`}>
                      {analysisData.auth_results.dmarc.result?.toUpperCase() || "NONE"}
                    </span>
                  </div>
                  <p className="muted text-sm mt-4">{analysisData.auth_results.dmarc.detail}</p>
                  {analysisData.auth_results.dmarc.result !== 'none' && analysisData.auth_results.dmarc.result !== '' && (
                    <p className="text-sm mt-4">Alignment: <span className={analysisData.auth_results.dmarc.aligned ? "badge badge-green" : "badge badge-red"}>{analysisData.auth_results.dmarc.aligned ? "ALIGNED" : "NOT ALIGNED"}</span></p>
                  )}
                </div>

                {/* ARC */}
                {analysisData.auth_results.arc?.length > 0 && (
                  <div className="mb-4" style={{ padding: 16, background: "var(--bg-3)", borderRadius: "var(--r-md)" }}>
                    <h4 style={{ fontSize: 14, fontWeight: 600, marginBottom: 12 }}>ARC (Authenticated Received Chain)</h4>
                    {analysisData.auth_results.arc.map((arc: any, i: number) => (
                      <div key={i} className="row between" style={{ marginBottom: 8 }}>
                        <span>Instance {arc.instance}</span>
                        <span className={`badge ${arc.result === 'pass' ? 'badge-green' : 'badge-red'}`}>{arc.result?.toUpperCase()}</span>
                        <span className="muted">CV: {arc.cv}</span>
                      </div>
                    ))}
                  </div>
                )}
              </div>
            )}

            {/* Spoofing Findings */}
            {analysisData?.spoof_findings?.length > 0 && (
              <div className="mt-4">
                <h4 style={{ fontSize: 14, fontWeight: 600, marginBottom: 12, color: "var(--danger)" }}>Spoofing Findings ({analysisData.spoof_findings.length})</h4>
                {analysisData.spoof_findings.map((f: any, i: number) => (
                  <div key={i} className="finding-card finding-card--{f.severity}">
                    <div className="row between" style={{ marginBottom: 8 }}>
                      <span className={`badge badge-${f.severity === 'critical' || f.severity === 'high' ? 'red' : 'orange'}`}>{f.severity?.toUpperCase()}</span>
                      <span className="muted text-sm">{f.finding_type}</span>
                    </div>
                    <h4 style={{ fontSize: 13, fontWeight: 600, marginBottom: 4 }}>{f.title}</h4>
                    <p className="text-sm" style={{ marginBottom: 8 }}>{f.description}</p>
                    <p className="mono text-xs muted">Indicator: {f.indicator}</p>
                  </div>
                ))}
              </div>
            )}

            {!analysisLoading && !analysisData && (
              <div className="empty">Loading authentication data...</div>
            )}
            {!analysisLoading && analysisData && !analysisData.auth_results && (
              <div className="empty">No authentication data available</div>
            )}
          </div>
        )}

        {tab === "mime" && (
          <div>
            <h4 style={{ fontSize: 13, fontWeight: 600, marginBottom: 12 }}>MIME Structure</h4>
            {(() => {
              const contentType = parsedHeaders.find(h => h.key === "Content-Type")?.value;
              const mimeVersion = parsedHeaders.find(h => h.key === "MIME-Version")?.value;
              return contentType ? (
                <div style={{ padding: 16, background: "var(--bg-3)", borderRadius: "var(--r-md)", marginBottom: 16 }}>
                  <p className="mono text-sm"><strong>Content-Type:</strong> {contentType}</p>
                  {mimeVersion && <p className="mono text-sm mt-4"><strong>MIME-Version:</strong> {mimeVersion}</p>}
                </div>
              ) : (
                <div className="empty">No MIME headers found (plain text email)</div>
              );
            })()}
            <div style={{ marginTop: 20 }}>
              <h4 style={{ fontSize: 13, fontWeight: 600, marginBottom: 8 }}>Body Parts</h4>
              {email.body_text && (
                <div style={{ padding: 12, background: "var(--bg-2)", borderRadius: "var(--r-sm)", marginBottom: 8, border: "1px solid var(--border)" }}>
                  <span className="badge badge-blue">text/plain</span>
                  <span className="muted text-sm" style={{ marginLeft: 8 }}>{email.body_text.length} chars</span>
                </div>
              )}
              {email.body_html && (
                <div style={{ padding: 12, background: "var(--bg-2)", borderRadius: "var(--r-sm)", marginBottom: 8, border: "1px solid var(--border)" }}>
                  <span className="badge badge-green">text/html</span>
                  <span className="muted text-sm" style={{ marginLeft: 8 }}>{email.body_html.length} chars</span>
                </div>
              )}
              {!email.body_text && !email.body_html && (
                <div className="empty">No body content</div>
              )}
            </div>
          </div>
        )}

        {tab === "raw" && (
          <div>
            <h4 style={{ fontSize: 13, fontWeight: 600, marginBottom: 12 }}>Full Raw Message</h4>
            <pre className="mono" style={{ fontSize: 11, color: "var(--text-2)", whiteSpace: "pre-wrap", wordBreak: "break-all", maxHeight: 600, overflow: "auto", background: "var(--bg-0)", padding: 16, borderRadius: "var(--r-md)", border: "1px solid var(--border)" }}>
              {email.headers_raw || "No raw headers available"}
              {"\n\n"}
              {"--- BODY ---\n\n"}
              {email.body_text || email.body_html || "No body available"}
            </pre>
          </div>
        )}

        {tab === "attachments" && (
          <div>
            <h4 style={{ fontSize: 13, fontWeight: 600, marginBottom: 12 }}>Attachments</h4>
            <div className="empty">Attachment extraction from MIME parts will be available in Phase 2</div>
            <div style={{ marginTop: 20, padding: 16, background: "var(--bg-3)", borderRadius: "var(--r-md)" }}>
              <p className="muted text-sm">Full attachment extraction (with SHA-256 hashes, magic byte detection, entropy analysis) requires MIME part byte extraction.</p>
              <p className="muted text-sm mt-4">For EML/MBOX files, attachments are embedded in the MIME structure and need to be decoded from base64/quoted-printable encoding.</p>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}