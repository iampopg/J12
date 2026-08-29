import { Finding, severityColor, statusBadge } from "./types";
import { BookmarkButton } from "../../components/BookmarkButton";

interface Props {
  caseId: string;
  findings: Finding[];
  filtered: Finding[];
  selectedFinding: Finding | null;
  searchTerm: string;
  setSearchTerm: (s: string) => void;
  filterSeverity: string;
  setFilterSeverity: (s: string) => void;
  filterStatus: string;
  setFilterStatus: (st: string) => void;
  loading: boolean;
  analyzing: boolean;
  onSelectFinding: (f: Finding) => void;
  onUpdateStatus: (id: string, st: string) => void;
  onRunAnalysis: () => void;
}

export function FindingsTable({
  caseId,
  findings,
  filtered,
  selectedFinding,
  searchTerm,
  setSearchTerm,
  filterSeverity,
  setFilterSeverity,
  filterStatus,
  setFilterStatus,
  loading,
  analyzing,
  onSelectFinding,
  onUpdateStatus,
  onRunAnalysis,
}: Props) {
  return (
    <>
      {/* Filter & Search Toolbar */}
      <div className="card mb-4" style={{ padding: "12px 16px" }}>
        <div className="row between" style={{ flexWrap: "wrap", gap: 12 }}>
          <div style={{ flex: 1, minWidth: 260 }}>
            <input
              className="input"
              style={{ width: "100%", padding: "6px 12px", fontSize: 13 }}
              placeholder="Search findings by keyword, indicator, brand, domain..."
              value={searchTerm}
              onChange={e => setSearchTerm(e.target.value)}
            />
          </div>

          <div className="row gap-2" style={{ flexWrap: "wrap" }}>
            <span className="muted" style={{ fontSize: 12, alignSelf: "center" }}>Severity:</span>
            {["all", "critical", "high", "medium", "low"].map(s => (
              <button
                key={s}
                className={`btn btn-sm ${filterSeverity === s ? "btn-primary" : "btn-ghost"}`}
                style={{ fontSize: 11, padding: "4px 8px" }}
                onClick={() => setFilterSeverity(s)}
              >
                {s.toUpperCase()}
              </button>
            ))}
          </div>

          <div className="row gap-2" style={{ flexWrap: "wrap" }}>
            <span className="muted" style={{ fontSize: 12, alignSelf: "center" }}>Status:</span>
            {["all", "open", "confirmed", "rejected", "reviewed"].map(st => (
              <button
                key={st}
                className={`btn btn-sm ${filterStatus === st ? "btn-primary" : "btn-ghost"}`}
                style={{ fontSize: 11, padding: "4px 8px" }}
                onClick={() => setFilterStatus(st)}
              >
                {st.toUpperCase()}
              </button>
            ))}
          </div>
        </div>
      </div>

      {/* Findings Table */}
      {loading ? (
        <div className="empty">Loading forensic findings...</div>
      ) : filtered.length === 0 ? (
        <div className="card" style={{ textAlign: "center", padding: "50px 30px" }}>
          <div style={{ fontSize: 40, marginBottom: 12 }}>🛡️</div>
          <h3 style={{ fontSize: 18, color: "var(--text-0)", marginBottom: 6 }}>
            {findings.length === 0 ? "No Findings Generated Yet" : "No findings match your filter criteria"}
          </h3>
          <p className="muted mb-4">
            {findings.length === 0
              ? "Run automated deep analysis to scan email headers, wire fraud indicators, brand spoofing, and file attachments."
              : "Try resetting your search query or severity filters above."}
          </p>
          {findings.length === 0 && (
            <button className="btn btn-primary" onClick={onRunAnalysis} disabled={analyzing}>
              {analyzing ? "Analyzing..." : "▶ Run Analysis Now"}
            </button>
          )}
        </div>
      ) : (
        <div className="card" style={{ padding: 0, overflow: "hidden", marginBottom: 20 }}>
          <div className="row between" style={{ padding: "10px 16px", background: "var(--bg-3)", borderBottom: "1px solid var(--border)", fontSize: 12, fontWeight: 600, color: "var(--text-1)" }}>
            <div>
              {filtered.length} Forensic Finding{filtered.length === 1 ? "" : "s"} — Select a finding to inspect evidentiary emails &amp; record notes
            </div>
            <div className="muted text-sm">
              Showing {filtered.length} of {findings.length} total
            </div>
          </div>
          <div style={{ overflowX: "auto" }}>
            <table>
              <thead>
                <tr>
                  <th className="th" style={{ width: 95 }}>Severity</th>
                  <th className="th" style={{ width: 110 }}>Category</th>
                  <th className="th">Finding Description &amp; Indicator</th>
                  <th className="th" style={{ width: 105 }}>Status</th>
                  <th className="th" style={{ width: 75 }}>Evidentiary</th>
                  <th className="th" style={{ width: 150 }}>Review Action</th>
                </tr>
              </thead>
              <tbody>
                {filtered.map(f => (
                  <tr
                    key={f.id}
                    className="tr-click"
                    style={{
                      background: selectedFinding?.id === f.id ? "rgba(59,130,246,0.12)" : undefined,
                      borderLeft: selectedFinding?.id === f.id ? "4px solid var(--accent)" : "4px solid transparent",
                    }}
                    onClick={() => onSelectFinding(f)}
                  >
                    <td>
                      <span className="badge" style={{ background: `${severityColor(f.severity)}22`, color: severityColor(f.severity), border: `1px solid ${severityColor(f.severity)}44`, fontWeight: 700 }}>
                        {f.severity.toUpperCase()}
                      </span>
                    </td>
                    <td><span className="badge badge-gray" style={{ fontWeight: 600 }}>{f.type_}</span></td>
                    <td style={{ maxWidth: 380 }}>
                      <div style={{ fontWeight: 600, color: "var(--text-0)", fontSize: 13 }}>{f.title}</div>
                      {f.description && (
                        <div className="muted text-sm" style={{ marginTop: 2, whiteSpace: "nowrap", overflow: "hidden", textOverflow: "ellipsis" }}>
                          {f.description}
                        </div>
                      )}
                    </td>
                    <td><span className={`badge ${statusBadge(f.status)}`}>{f.status.toUpperCase()}</span></td>
                    <td className="mono" style={{ fontSize: 12 }}>
                      {(() => {
                        try {
                          const ids = JSON.parse(f.email_ids || "[]");
                          return `${ids.length} msg`;
                        } catch { return "0"; }
                      })()}
                    </td>
                    <td>
                      <div className="row gap-2" style={{ alignItems: "center" }} onClick={e => e.stopPropagation()}>
                        <BookmarkButton
                          caseId={caseId}
                          itemId={f.id}
                          itemType="finding"
                          compact={true}
                        />
                        {f.status === "open" && (
                          <>
                            <button className="btn btn-ghost btn-sm" style={{ color: "var(--success)", padding: "2px 6px" }} onClick={() => onUpdateStatus(f.id, "confirmed")} title="Confirm finding">
                              ✓ Confirm
                            </button>
                            <button className="btn btn-ghost btn-sm" style={{ color: "var(--danger)", padding: "2px 6px" }} onClick={() => onUpdateStatus(f.id, "rejected")} title="Reject (False Positive)">
                              ✗ Reject
                            </button>
                          </>
                        )}
                        {f.status === "confirmed" && (
                          <button className="btn btn-ghost btn-sm" style={{ color: "var(--warning)", padding: "2px 6px" }} onClick={() => onUpdateStatus(f.id, "reviewed")} title="Mark reviewed">
                            👁 Review
                          </button>
                        )}
                        {f.status !== "open" && (
                          <button className="btn btn-ghost btn-sm" style={{ padding: "2px 6px", fontSize: 11 }} onClick={() => onUpdateStatus(f.id, "open")} title="Reopen finding">
                            Reopen
                          </button>
                        )}
                      </div>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </div>
      )}
    </>
  );
}
