import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";

function cleanDisplayName(name: string | null): string {
  if (!name) return '';
  let cleaned = name
    .replace(/@ENRON.*$/g, '')
    .replace(/IMCEANOTES-[^<]*/g, '')
    .replace(/<[^>]*>/g, '')
    .replace(/"/g, '')
    .replace(/\s+/g, ' ')
    .trim();
  if (cleaned.includes('@')) {
    return cleaned.split('@')[0].trim() || cleaned;
  }
  return cleaned;
}

interface Entity {
  id: string;
  email_address: string;
  display_name: string | null;
  first_seen: string | null;
  last_seen: string | null;
  sent_count: number;
  received_count: number;
  role: string;
}

interface EntityDetail {
  email: string;
  display_name: string | null;
  first_seen: string | null;
  last_seen: string | null;
  sent_count: number;
  received_count: number;
  sent_to: [string, number][];
  received_from: [string, number][];
}

interface EntityEmail {
  id: string;
  from_addr: string;
  from_display: string | null;
  to_addrs: string;
  subject: string | null;
  date_sent_utc: string;
  risk_score: number;
  folder_category: string;
}

interface Props {
  caseId: string;
  onSelectEmail?: (id: string) => void;
}

export function EntityDiveView({ caseId, onSelectEmail }: Props) {
  const [entities, setEntities] = useState<Entity[]>([]);
  const [selectedEntity, setSelectedEntity] = useState<EntityDetail | null>(null);
  const [loading, setLoading] = useState(true);
  const [diveLoading, setDiveLoading] = useState(false);
  const [searchTerm, setSearchTerm] = useState("");
  const [showEmailList, setShowEmailList] = useState(false);
  const [entityEmails, setEntityEmails] = useState<EntityEmail[]>([]);
  const [emailsLoading, setEmailsLoading] = useState(false);
  const [selectedEmailId, setSelectedEmailId] = useState<string | null>(null);

  // Filters
  const [dateFrom, setDateFrom] = useState("");
  const [dateTo, setDateTo] = useState("");
  const [hasAttachment, setHasAttachment] = useState(false);

  useEffect(() => { loadEntities(); }, [caseId]);

  const loadEntities = async () => {
    setLoading(true);
    try {
      let data = await invoke<Entity[]>("entity_list", { input: { case_id: caseId } });
      // Auto-extract if empty
      if (data.length === 0) {
        await invoke<number>("extract_entities", { caseId });
        data = await invoke<Entity[]>("entity_list", { input: { case_id: caseId } });
      }
      setEntities(data);
      if (data.length > 0) loadEntityDive(data[0].email_address);
    } catch (e) { console.error(e); }
    setLoading(false);
  };

  const loadEntityDive = async (email: string) => {
    setDiveLoading(true);
    try {
      const data = await invoke<EntityDetail>("entity_dive", { input: { case_id: caseId, email_address: email } });
      setSelectedEntity(data);
      setShowEmailList(false);
      setDateFrom(""); setDateTo(""); setHasAttachment(false);
    } catch (e) { console.error(e); }
    setDiveLoading(false);
  };

  const loadEntityEmails = async () => {
    if (!selectedEntity) return;
    setEmailsLoading(true);
    try {
      const data = await invoke<EntityEmail[]>("entity_emails", {
        input: {
          case_id: caseId, email: selectedEntity.email,
          date_from: dateFrom, date_to: dateTo, has_attachment: hasAttachment
        }
      });
      setEntityEmails(data);
      setShowEmailList(true);
    } catch (e) { console.error(e); }
    setEmailsLoading(false);
  };

  const filteredEntities = entities.filter(e =>
    e.email_address.toLowerCase().includes(searchTerm.toLowerCase()) ||
    (e.display_name || "").toLowerCase().includes(searchTerm.toLowerCase())
  );

  // Calculate risk score for selected entity
  const entityRisk = selectedEntity ? Math.min(100, Math.round((selectedEntity.sent_count + selectedEntity.received_count) > 100 ? 50 : (selectedEntity.sent_count + selectedEntity.received_count) / 2)) : 0;

  if (loading) return <div className="empty">Loading entities...</div>;

  if (entities.length === 0) {
    return (
      <div>
        <h2 style={{ fontSize: 22, fontWeight: 700, color: "var(--text-0)", marginBottom: 16 }}>Entity Profiles</h2>
        <div className="card empty">No entities found. Extract entities from email data.</div>
      </div>
    );
  }

  return (
    <div>
      <div className="row between mb-4">
        <div>
          <h2 style={{ fontSize: 22, fontWeight: 700, color: "var(--text-0)" }}>Entity Profiles</h2>
          <p className="muted">{entities.length} entities · Click to explore</p>
        </div>
        <button className="btn btn-ghost btn-sm" onClick={loadEntities}>↻ Refresh</button>
      </div>

      <div className="grid-2" style={{ gap: 16 }}>
        {/* Entity List */}
        <div className="card" style={{ maxHeight: "70vh", overflowY: "auto" }}>
          <input className="input mb-4" placeholder="Search entities..." value={searchTerm} onChange={(e) => setSearchTerm(e.target.value)} />
          <div style={{ display: "flex", flexDirection: "column", gap: 4 }}>
            {filteredEntities.map((e) => (
              <div key={e.id} className="row between tr-click" style={{
                padding: "10px 12px", borderRadius: "var(--r-sm)",
                background: selectedEntity?.email === e.email_address ? "var(--accent-subtle)" : "transparent",
                border: selectedEntity?.email === e.email_address ? "1px solid var(--accent)" : "1px solid transparent",
              }} onClick={() => loadEntityDive(e.email_address)}>
                <div>
                  <div style={{ fontSize: 12, fontWeight: 500 }}>{e.display_name || e.email_address}</div>
                  {e.display_name && <div style={{ fontSize: 11, color: "var(--accent)", fontFamily: "var(--mono)" }}>{e.email_address}</div>}
                </div>
                <div style={{ textAlign: "right" }}>
                  <div style={{ fontSize: 12, fontWeight: 600 }}>{e.sent_count + e.received_count}</div>
                  <div style={{ fontSize: 10, color: "var(--text-3)" }}>emails</div>
                </div>
              </div>
            ))}
          </div>
        </div>

        {/* Entity Detail */}
        <div className="card" style={{ maxHeight: "70vh", overflowY: "auto" }}>
          {diveLoading && <div className="empty">Loading profile...</div>}
          {!diveLoading && selectedEntity && (
            <div>
              {/* Header */}
              <div style={{ display: "flex", alignItems: "center", gap: 12, marginBottom: 16 }}>
                <div style={{ width: 48, height: 48, borderRadius: "50%", background: "linear-gradient(135deg, #3b82f6, #6366f1)", display: "flex", alignItems: "center", justifyContent: "center", fontSize: 20, color: "#fff", fontWeight: 700 }}>
                  {(selectedEntity.display_name || selectedEntity.email).charAt(0).toUpperCase()}
                </div>
                <div>
                  <h3 style={{ fontSize: 18, fontWeight: 700 }}>{selectedEntity.display_name || selectedEntity.email}</h3>
                  <p style={{ fontSize: 13, color: "var(--accent)", fontFamily: "var(--mono)" }}>{selectedEntity.email}</p>
                </div>
              </div>

              {/* Stats */}
              <div className="row gap-4 mb-4">
                <div style={{ flex: 1, padding: 12, background: "var(--bg-3)", borderRadius: "var(--r-sm)", textAlign: "center" }}>
                  <div style={{ fontSize: 20, fontWeight: 700, color: "#3b82f6" }}>{selectedEntity.sent_count}</div>
                  <div style={{ fontSize: 10, color: "var(--text-3)" }}>SENT</div>
                </div>
                <div style={{ flex: 1, padding: 12, background: "var(--bg-3)", borderRadius: "var(--r-sm)", textAlign: "center" }}>
                  <div style={{ fontSize: 20, fontWeight: 700, color: "#22c55e" }}>{selectedEntity.received_count}</div>
                  <div style={{ fontSize: 10, color: "var(--text-3)" }}>RECEIVED</div>
                </div>
                <div style={{ flex: 1, padding: 12, background: "var(--bg-3)", borderRadius: "var(--r-sm)", textAlign: "center" }}>
                  <div style={{ fontSize: 20, fontWeight: 700, color: entityRisk >= 50 ? "var(--danger)" : entityRisk >= 25 ? "var(--warning)" : "var(--success)" }}>{entityRisk}</div>
                  <div style={{ fontSize: 10, color: "var(--text-3)" }}>RISK</div>
                </div>
              </div>

              {/* Date Range */}
              <div style={{ padding: 12, background: "var(--bg-3)", borderRadius: "var(--r-sm)", marginBottom: 16 }}>
                <div className="row between">
                  <div><div style={{ fontSize: 10, color: "var(--text-3)" }}>FIRST SEEN</div><div style={{ fontSize: 13 }}>{selectedEntity.first_seen ? new Date(selectedEntity.first_seen).toLocaleDateString() : "—"}</div></div>
                  <div style={{ textAlign: "right" }}><div style={{ fontSize: 10, color: "var(--text-3)" }}>LAST SEEN</div><div style={{ fontSize: 13 }}>{selectedEntity.last_seen ? new Date(selectedEntity.last_seen).toLocaleDateString() : "—"}</div></div>
                </div>
              </div>

              {/* Filter Controls */}
              <div style={{ padding: 12, background: "var(--bg-3)", borderRadius: "var(--r-sm)", marginBottom: 16 }}>
                <div className="row between mb-4">
                  <span style={{ fontSize: 12, fontWeight: 600 }}>Filter Emails</span>
                  <button className="btn btn-primary btn-sm" onClick={loadEntityEmails}>Show Emails</button>
                </div>
                <div className="row gap-4">
                  <div style={{ flex: 1 }}>
                    <label style={{ fontSize: 10, color: "var(--text-3)" }}>From Date</label>
                    <input type="date" className="input" value={dateFrom} onChange={e => setDateFrom(e.target.value)} />
                  </div>
                  <div style={{ flex: 1 }}>
                    <label style={{ fontSize: 10, color: "var(--text-3)" }}>To Date</label>
                    <input type="date" className="input" value={dateTo} onChange={e => setDateTo(e.target.value)} />
                  </div>
                  <label className="row gap-2" style={{ fontSize: 12, color: "var(--text-2)", paddingTop: 16 }}>
                    <input type="checkbox" checked={hasAttachment} onChange={e => setHasAttachment(e.target.checked)} />
                    Has attachments
                  </label>
                </div>
              </div>

              {/* Email List */}
              {showEmailList && (
                <div style={{ marginBottom: 16 }}>
                  <h4 style={{ fontSize: 13, fontWeight: 600, marginBottom: 8 }}>Emails ({entityEmails.length})</h4>
                  {emailsLoading ? <div className="empty">Loading...</div> : entityEmails.length > 0 ? (
                    <div style={{ maxHeight: 250, overflowY: "auto" }}>
                      <table style={{ marginTop: 0 }}>
                        <thead><tr><th className="th">From</th><th className="th">Subject</th><th className="th" style={{ width: 80 }}>Date</th><th className="th" style={{ width: 50 }}>Risk</th></tr></thead>
                        <tbody>
                          {entityEmails.map(e => (
                             <tr key={e.id} className="tr-click" onClick={() => setSelectedEmailId(e.id)}>
                              <td className="td" style={{ fontSize: 12 }}>{cleanDisplayName(e.from_display) || e.from_addr}</td>
                              <td className="td" style={{ fontSize: 12 }}>{e.subject || <span className="muted">—</span>}</td>
                              <td className="td muted" style={{ fontSize: 11 }}>{new Date(e.date_sent_utc).toLocaleDateString()}</td>
                              <td className="td"><span className={`badge ${e.risk_score >= 50 ? "badge-red" : e.risk_score >= 25 ? "badge-orange" : "badge-green"}`}>{e.risk_score}</span></td>
                            </tr>
                          ))}
                        </tbody>
                      </table>
                    </div>
                  ) : <div className="empty">No emails match filters</div>}
                </div>
              )}

              {/* Partners */}
              <div style={{ marginBottom: 16 }}>
                <h4 style={{ fontSize: 13, fontWeight: 600, marginBottom: 8 }}>Top Sent To</h4>
                {selectedEntity.sent_to.length > 0 ? selectedEntity.sent_to.map(([email, count], i) => (
                  <div key={i} className="row between" style={{ padding: "4px 0", borderBottom: "1px solid var(--border)" }}>
                    <span style={{ fontSize: 12, fontFamily: "var(--mono)" }}>{email}</span>
                    <span className="badge badge-blue">{count}</span>
                  </div>
                )) : <div className="muted text-sm">No data</div>}
              </div>
              <div style={{ marginBottom: 16 }}>
                <h4 style={{ fontSize: 13, fontWeight: 600, marginBottom: 8 }}>Top Received From</h4>
                {selectedEntity.received_from.length > 0 ? selectedEntity.received_from.map(([email, count], i) => (
                  <div key={i} className="row between" style={{ padding: "4px 0", borderBottom: "1px solid var(--border)" }}>
                    <span style={{ fontSize: 12, fontFamily: "var(--mono)" }}>{email}</span>
                    <span className="badge badge-gray">{count}</span>
                  </div>
                )) : <div className="muted text-sm">No data</div>}
              </div>

              {/* Heatmap */}
              <CommunicationHeatmap email={selectedEntity.email} caseId={caseId} />
            </div>
          )}
        </div>
      </div>
    </div>
  );
}

function CommunicationHeatmap({ email, caseId }: { email: string; caseId: string }) {
  const [data, setData] = useState<{ date: string; count: number }[]>([]);

  useEffect(() => {
    invoke<any>("entity_heatmap", { input: { case_id: caseId, email_address: email } })
      .then(d => setData(d.data || []))
      .catch(() => setData([]));
  }, [email, caseId]);

  if (data.length === 0) return null;
  const maxCount = Math.max(...data.map(d => d.count), 1);

  return (
    <div style={{ marginTop: 16 }}>
      <h4 style={{ fontSize: 13, fontWeight: 600, marginBottom: 8 }}>Activity Heatmap</h4>
      <div style={{ display: "flex", flexWrap: "wrap", gap: 2 }}>
        {data.map((d, i) => {
          const intensity = d.count / maxCount;
          const bg = d.count === 0 ? "var(--bg-3)" : `rgba(59, 130, 246, ${0.2 + intensity * 0.8})`;
          return <div key={i} title={`${d.date}: ${d.count} emails`} style={{ width: 12, height: 12, borderRadius: 2, background: bg }} />;
        })}
      </div>
      <div className="row between" style={{ marginTop: 8 }}>
        <span style={{ fontSize: 10, color: "var(--text-3)" }}>{data[0]?.date}</span>
        <span style={{ fontSize: 10, color: "var(--text-3)" }}>Less</span>
        <div style={{ display: "flex", gap: 2 }}>
          {[0.2, 0.4, 0.6, 0.8, 1.0].map((v, i) => <div key={i} style={{ width: 10, height: 10, borderRadius: 2, background: `rgba(59, 130, 246, ${v})` }} />)}
        </div>
        <span style={{ fontSize: 10, color: "var(--text-3)" }}>More</span>
        <span style={{ fontSize: 10, color: "var(--text-3)" }}>{data[data.length - 1]?.date}</span>
      </div>
    </div>
  );
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
          <div><span className="muted">From:</span> <strong>{cleanDisplayName(email.from_display) || email.from_addr}</strong></div>
          <div><span className="muted">Date:</span> {email.date_sent ? new Date(email.date_sent).toLocaleString() : "—"}</div>
        </div>
        <div className="mb-4"><span className="muted">To:</span> {toList.join(", ")}</div>
        <div className="mb-4"><span className="muted">Risk:</span> <span className={`badge ${email.risk_score >= 50 ? "badge-red" : email.risk_score >= 25 ? "badge-orange" : "badge-green"}`}>{email.risk_score}</span></div>
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
