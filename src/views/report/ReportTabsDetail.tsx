import { ReportData, Exhibit, ReportSection, cleanDisplayName } from "./types";

interface Props {
  activeTab: "sources" | "folders" | "findings" | "ledger" | "exhibits" | "sections";
  reportData: ReportData | null;
  exhibits: Exhibit[];
  sections: ReportSection[];
  onAddExhibit: () => void;
  onRemoveExhibit: (id: string) => void;
  onUpdateExhibitNotes: (id: string, notes: string) => void;
  onToggleSection: (id: string) => void;
}

export function ReportTabsDetail({
  activeTab,
  reportData,
  exhibits,
  sections,
  onAddExhibit,
  onRemoveExhibit,
  onUpdateExhibitNotes,
  onToggleSection,
}: Props) {
  if (activeTab === "sources") {
    return (
      <div className="card">
        <h3 style={{ fontSize: 16, fontWeight: 700, marginBottom: 12 }}>
          Source Data Provenance &amp; Container Verification
        </h3>
        <p className="muted mb-4" style={{ fontSize: 12 }}>
          Comprehensive technical manifest of all evidence containers attached to this case.
        </p>

        <div style={{ display: "flex", flexDirection: "column", gap: 14 }}>
          {(reportData?.evidence_inventory || []).map((ev) => (
            <div
              key={ev.id}
              style={{
                background: "var(--bg-3)",
                padding: 16,
                borderRadius: "var(--r-md)",
                border: "1px solid var(--border)",
              }}
            >
              <div className="row between mb-2">
                <strong style={{ fontSize: 15, color: "var(--text-0)" }}>{ev.filename}</strong>
                <span className="badge badge-green">VERIFIED INTEGRITY</span>
              </div>

              <div className="grid-3 mb-3" style={{ fontSize: 12 }}>
                <div>
                  <span className="muted">Format: </span>
                  <strong>{ev.format.toUpperCase()}</strong>
                </div>
                <div>
                  <span className="muted">Size: </span>
                  <strong>{(ev.size_bytes / (1024 * 1024)).toFixed(2)} MB</strong> ({ev.size_bytes.toLocaleString()} bytes)
                </div>
                <div>
                  <span className="muted">Extracted Emails: </span>
                  <strong>{ev.message_count.toLocaleString()}</strong>
                </div>
              </div>

              <div style={{ fontSize: 11, background: "var(--bg-1)", padding: 10, borderRadius: "var(--r-xs)" }}>
                <div className="muted mb-1">CRYPTOGRAPHIC ACQUISITION HASH (SHA-256):</div>
                <code style={{ color: "var(--accent)", fontFamily: "var(--mono)" }}>{ev.sha256}</code>
              </div>
            </div>
          ))}
        </div>
      </div>
    );
  }

  if (activeTab === "folders") {
    return (
      <div className="card">
        <h3 style={{ fontSize: 16, fontWeight: 700, marginBottom: 12 }}>
          Mailbox Storage &amp; Folder Hierarchy Breakdown
        </h3>
        <table style={{ width: "100%", fontSize: 12 }}>
          <thead>
            <tr>
              <th className="th">Folder Name</th>
              <th className="th" style={{ width: 140 }}>Category</th>
              <th className="th" style={{ width: 110 }}>Item Count</th>
              <th className="th" style={{ width: 130 }}>Earliest Date</th>
              <th className="th" style={{ width: 130 }}>Latest Date</th>
            </tr>
          </thead>
          <tbody>
            {(reportData?.folder_breakdown || []).map((f: any, i: number) => (
              <tr key={i}>
                <td className="td"><strong>{f.folder_name}</strong></td>
                <td className="td">
                  <span className={`badge ${f.folder_category === "sent" ? "badge-blue" : f.folder_category === "soft_deleted" ? "badge-red" : "badge-green"}`}>
                    {f.folder_category}
                  </span>
                </td>
                <td className="td"><strong>{f.count.toLocaleString()}</strong></td>
                <td className="td muted">{f.date_from ? f.date_from.slice(0, 10) : "—"}</td>
                <td className="td muted">{f.date_to ? f.date_to.slice(0, 10) : "—"}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    );
  }

  if (activeTab === "findings") {
    return (
      <div className="card">
        <h3 style={{ fontSize: 16, fontWeight: 700, marginBottom: 12 }}>
          Forensic Findings &amp; Security Violations Matrix
        </h3>
        <table style={{ width: "100%", fontSize: 12 }}>
          <thead>
            <tr>
              <th className="th" style={{ width: 100 }}>Severity</th>
              <th className="th" style={{ width: 120 }}>Finding Type</th>
              <th className="th">Finding Description</th>
              <th className="th" style={{ width: 100 }}>Status</th>
            </tr>
          </thead>
          <tbody>
            {(reportData?.findings || []).map((f: any) => (
              <tr key={f.id}>
                <td className="td">
                  <span
                    className={`badge ${
                      f.severity === "critical"
                        ? "badge-red"
                        : f.severity === "high"
                        ? "badge-orange"
                        : "badge-yellow"
                    }`}
                  >
                    {f.severity.toUpperCase()}
                  </span>
                </td>
                <td className="td"><strong>{f.type}</strong></td>
                <td className="td">
                  <div style={{ fontWeight: 600, color: "var(--text-0)" }}>{f.title}</div>
                  <div className="muted text-sm">{f.description}</div>
                </td>
                <td className="td">
                  <span className="badge badge-blue">{f.status}</span>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    );
  }

  if (activeTab === "ledger") {
    return (
      <div className="card">
        <h3 style={{ fontSize: 16, fontWeight: 700, marginBottom: 12 }}>
          Evidentiary &amp; Flagged Messages Ledger
        </h3>
        <p className="muted mb-4" style={{ fontSize: 12 }}>
          Itemized record of suspicious, high-risk, and recovered deleted messages extracted during analysis.
        </p>
        <table style={{ width: "100%", fontSize: 11 }}>
          <thead>
            <tr>
              <th className="th" style={{ width: 160 }}>Sender</th>
              <th className="th">Subject</th>
              <th className="th" style={{ width: 90 }}>Date</th>
              <th className="th" style={{ width: 80 }}>Folder</th>
              <th className="th" style={{ width: 50 }}>Risk</th>
            </tr>
          </thead>
          <tbody>
            {(reportData?.key_messages_ledger || []).map((em: any) => (
              <tr key={em.id}>
                <td className="td">{cleanDisplayName(em.from_display) || em.from_addr}</td>
                <td className="td">
                  <strong>{em.subject || "(no subject)"}</strong>
                  {em.deleted_recovered && (
                    <span className="badge badge-red" style={{ fontSize: 8, marginLeft: 6 }}>
                      DELETED
                    </span>
                  )}
                </td>
                <td className="td muted">{em.date_sent_utc ? em.date_sent_utc.slice(0, 10) : "—"}</td>
                <td className="td muted">{em.folder_category}</td>
                <td className="td">
                  <span className={`badge ${em.risk_score >= 50 ? "badge-red" : em.risk_score >= 25 ? "badge-orange" : "badge-green"}`} style={{ fontSize: 8 }}>
                    {em.risk_score}
                  </span>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    );
  }

  if (activeTab === "exhibits") {
    return (
      <div className="card">
        <div className="row between mb-4">
          <div>
            <h3 style={{ fontSize: 16, fontWeight: 700 }}>Marked Court Exhibits</h3>
            <p className="muted" style={{ fontSize: 12 }}>
              Bookmarked evidentiary emails to include in formal report appendices.
            </p>
          </div>
          <button className="btn btn-primary btn-sm" onClick={onAddExhibit}>
            + Add Exhibit by Email ID
          </button>
        </div>

        {exhibits.length === 0 ? (
          <div className="empty">No exhibits bookmarked yet. Use "+ Add Exhibit" to enter emails into the record.</div>
        ) : (
          <div style={{ display: "flex", flexDirection: "column", gap: 10 }}>
            {exhibits.map((ex) => (
              <div
                key={ex.id}
                style={{
                  padding: 14,
                  background: "var(--bg-3)",
                  borderRadius: "var(--r-md)",
                  border: "1px solid var(--border)",
                }}
              >
                <div className="row between mb-2">
                  <strong style={{ fontSize: 14, color: "var(--accent)" }}>
                    {ex.exhibit_number}: {ex.subject}
                  </strong>
                  <button
                    className="btn btn-ghost btn-sm"
                    style={{ color: "var(--red)", fontSize: 11 }}
                    onClick={() => onRemoveExhibit(ex.id)}
                  >
                    ✕ Remove
                  </button>
                </div>
                <div className="grid-2 text-sm mb-2">
                  <div>From: <strong>{ex.from_display || ex.from_addr}</strong></div>
                  <div>Date: <strong>{ex.date_sent}</strong></div>
                </div>
                <input
                  className="input"
                  style={{ fontSize: 11, padding: "4px 8px", width: "100%" }}
                  placeholder="Add investigator annotation / notes for this exhibit..."
                  value={ex.notes}
                  onChange={(e) => onUpdateExhibitNotes(ex.id, e.target.value)}
                />
              </div>
            ))}
          </div>
        )}
      </div>
    );
  }

  if (activeTab === "sections") {
    return (
      <div className="card">
        <h3 style={{ fontSize: 16, fontWeight: 700, marginBottom: 6 }}>Configure Report Chapters</h3>
        <p className="muted mb-4" style={{ fontSize: 12 }}>
          Toggle sections on or off to tailor the final report for court submission, internal review, or executive presentation.
        </p>

        <div style={{ display: "grid", gridTemplateColumns: "repeat(auto-fill, minmax(320px, 1fr))", gap: 12 }}>
          {sections.map((sec) => (
            <label
              key={sec.id}
              style={{
                display: "flex",
                alignItems: "flex-start",
                gap: 12,
                padding: 14,
                background: "var(--bg-3)",
                borderRadius: "var(--r-md)",
                cursor: "pointer",
                border: sec.enabled ? "1px solid var(--accent)" : "1px solid transparent",
              }}
            >
              <input
                type="checkbox"
                checked={sec.enabled}
                onChange={() => onToggleSection(sec.id)}
                style={{ marginTop: 2 }}
              />
              <div>
                <div style={{ fontSize: 13, fontWeight: 600, color: "var(--text-0)" }}>
                  {sec.title}
                </div>
                <div style={{ fontSize: 11, color: "var(--text-3)", marginTop: 2 }}>
                  {sec.description}
                </div>
              </div>
            </label>
          ))}
        </div>
      </div>
    );
  }

  return null;
}
