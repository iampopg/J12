import { ReportData, Exhibit, cleanDisplayName } from "./types";
import { ReportCertificationCard } from "./ReportCertificationCard";

interface Props {
  reportData: ReportData | null;
  caseData: any;
  enabledSections: Set<string>;
  exhibits: Exhibit[];
}

export function ReportDossierPreview({
  reportData,
  caseData,
  enabledSections,
  exhibits,
}: Props) {
  return (
    <div
      className="card"
      style={{
        background: "var(--bg-1)",
        padding: "44px 50px",
        borderRadius: "var(--r-md)",
        border: "1px solid var(--border)",
        maxWidth: 1020,
        margin: "0 auto",
        color: "var(--text-0)",
      }}
    >
      {/* COVER PAGE BANNER */}
      <div
        style={{
          textAlign: "center",
          borderBottom: "3px double var(--border)",
          paddingBottom: 28,
          marginBottom: 32,
        }}
      >
        <div style={{ fontSize: 12, fontWeight: 700, letterSpacing: "0.15em", color: "var(--accent)", textTransform: "uppercase", marginBottom: 6 }}>
          DIGITAL FORENSICS &amp; eDISCOVERY EXAMINATION REPORT
        </div>
        <h1 style={{ fontSize: 28, fontWeight: 900, margin: "8px 0 6px", color: "var(--text-0)" }}>
          {reportData?.case_info?.title || caseData?.title || "Email Investigation"}
        </h1>
        <div style={{ fontSize: 13, color: "var(--text-2)", marginBottom: 12 }}>
          Case File Reference: <strong>#{reportData?.case_info?.case_number || caseData?.case_number || "J12-001"}</strong>
        </div>
        <div style={{ fontSize: 11, color: "var(--text-3)", display: "flex", justifyContent: "center", gap: 20 }}>
          <span>Generated: {new Date().toUTCString()}</span>
          <span>Classification: <strong>CONFIDENTIAL / LAW ENFORCEMENT &amp; LEGAL PRIVILEGED</strong></span>
        </div>
      </div>

      {/* 1. Case & Investigation Information */}
      {enabledSections.has("case_info") && (
        <div style={{ marginBottom: 32 }}>
          <h3 style={{ fontSize: 15, fontWeight: 700, borderBottom: "1px solid var(--border)", paddingBottom: 6, marginBottom: 12, color: "var(--accent)" }}>
            1. Case Overview &amp; Subject Identification
          </h3>
          <table style={{ width: "100%", fontSize: 12, marginBottom: 8 }}>
            <tbody>
              <tr>
                <td style={{ width: 180, fontWeight: 600, background: "var(--bg-3)" }}>Case Title</td>
                <td>{reportData?.case_info?.title}</td>
                <td style={{ width: 180, fontWeight: 600, background: "var(--bg-3)" }}>Case Number</td>
                <td>{reportData?.case_info?.case_number || "—"}</td>
              </tr>
              <tr>
                <td style={{ fontWeight: 600, background: "var(--bg-3)" }}>Target Subject</td>
                <td><strong>{reportData?.case_info?.target_name || "—"}</strong></td>
                <td style={{ fontWeight: 600, background: "var(--bg-3)" }}>Target Email Address</td>
                <td><code>{reportData?.case_info?.target_email || "—"}</code></td>
              </tr>
              <tr>
                <td style={{ fontWeight: 600, background: "var(--bg-3)" }}>Organization / Entity</td>
                <td>{reportData?.case_info?.target_organization || "—"}</td>
                <td style={{ fontWeight: 600, background: "var(--bg-3)" }}>Investigation Status</td>
                <td><span className="badge badge-green">{reportData?.case_info?.status || "ACTIVE"}</span></td>
              </tr>
            </tbody>
          </table>
        </div>
      )}

      {/* 2. Evidence Sources & Provenance */}
      {enabledSections.has("sources") && (
        <div style={{ marginBottom: 32 }}>
          <h3 style={{ fontSize: 15, fontWeight: 700, borderBottom: "1px solid var(--border)", paddingBottom: 6, marginBottom: 12, color: "var(--accent)" }}>
            2. Evidence Sources &amp; Cryptographic Provenance (Per Source Data)
          </h3>
          <p style={{ fontSize: 12, color: "var(--text-2)", marginBottom: 10 }}>
            Inventory of physical/digital forensic mail containers acquired, verified, and parsed into the investigative database.
          </p>
          <table style={{ width: "100%", fontSize: 11, marginBottom: 14 }}>
            <thead>
              <tr>
                <th className="th">Source Container</th>
                <th className="th" style={{ width: 70 }}>Format</th>
                <th className="th" style={{ width: 90 }}>Size (Bytes)</th>
                <th className="th" style={{ width: 90 }}>Messages</th>
                <th className="th">SHA-256 Acquisition Hash</th>
              </tr>
            </thead>
            <tbody>
              {(reportData?.evidence_inventory || []).map((ev) => (
                <tr key={ev.id}>
                  <td className="td">
                    <strong>{ev.filename}</strong>
                    <div style={{ fontSize: 10, color: "var(--text-3)" }}>
                      Acquisition: {new Date(ev.acquired_at).toLocaleString()}
                    </div>
                  </td>
                  <td className="td">
                    <span className="badge badge-blue">{ev.format.toUpperCase()}</span>
                  </td>
                  <td className="td muted">
                    {(ev.size_bytes / (1024 * 1024)).toFixed(2)} MB ({ev.size_bytes.toLocaleString()} B)
                  </td>
                  <td className="td">
                    <strong>{ev.message_count.toLocaleString()}</strong> items
                  </td>
                  <td className="td" style={{ fontFamily: "var(--mono)", fontSize: 10, color: "var(--accent)" }}>
                    {ev.sha256}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}

      {/* 3. Executive Analytics & Volume Ledger */}
      {enabledSections.has("exec_summary") && reportData?.email_stats && (
        <div style={{ marginBottom: 32 }}>
          <h3 style={{ fontSize: 15, fontWeight: 700, borderBottom: "1px solid var(--border)", paddingBottom: 6, marginBottom: 12, color: "var(--accent)" }}>
            3. Executive Summary &amp; Mailbox Analytics
          </h3>
          <div
            style={{
              display: "grid",
              gridTemplateColumns: "repeat(4, 1fr)",
              gap: 12,
              marginBottom: 14,
            }}
          >
            <div style={{ background: "var(--bg-3)", padding: 12, borderRadius: "var(--r-sm)", textAlign: "center" }}>
              <div style={{ fontSize: 22, fontWeight: 800, color: "var(--accent)" }}>
                {reportData.email_stats.total?.toLocaleString() || 0}
              </div>
              <div style={{ fontSize: 10, color: "var(--text-3)", marginTop: 2 }}>TOTAL MESSAGES</div>
            </div>

            <div style={{ background: "var(--bg-3)", padding: 12, borderRadius: "var(--r-sm)", textAlign: "center" }}>
              <div style={{ fontSize: 22, fontWeight: 800, color: "#3b82f6" }}>
                {reportData.email_stats.sent?.toLocaleString() || 0}
              </div>
              <div style={{ fontSize: 10, color: "var(--text-3)", marginTop: 2 }}>OUTBOUND / SENT</div>
            </div>

            <div style={{ background: "var(--bg-3)", padding: 12, borderRadius: "var(--r-sm)", textAlign: "center" }}>
              <div style={{ fontSize: 22, fontWeight: 800, color: "#22c55e" }}>
                {reportData.email_stats.inbox?.toLocaleString() || 0}
              </div>
              <div style={{ fontSize: 10, color: "var(--text-3)", marginTop: 2 }}>INBOUND / INBOX</div>
            </div>

            <div style={{ background: "var(--bg-3)", padding: 12, borderRadius: "var(--r-sm)", textAlign: "center" }}>
              <div style={{ fontSize: 22, fontWeight: 800, color: "#ef4444" }}>
                {reportData.email_stats.deleted?.toLocaleString() || 0}
              </div>
              <div style={{ fontSize: 10, color: "var(--text-3)", marginTop: 2 }}>DELETED / RECOVERED</div>
            </div>
          </div>
          <div style={{ fontSize: 11, color: "var(--text-2)" }}>
            Temporal Range: <strong>{reportData.email_stats.date_from?.slice(0, 10) || "—"}</strong> to <strong>{reportData.email_stats.date_to?.slice(0, 10) || "—"}</strong>
          </div>
        </div>
      )}

      {/* 4. Mailbox Structure & Folder Hierarchy Breakdown */}
      {enabledSections.has("folders") && (reportData?.folder_breakdown || []).length > 0 && (
        <div style={{ marginBottom: 32 }}>
          <h3 style={{ fontSize: 15, fontWeight: 700, borderBottom: "1px solid var(--border)", paddingBottom: 6, marginBottom: 12, color: "var(--accent)" }}>
            4. Mailbox Folder Structure &amp; Item Tally
          </h3>
          <table style={{ width: "100%", fontSize: 11, marginBottom: 10 }}>
            <thead>
              <tr>
                <th className="th">Folder Name</th>
                <th className="th" style={{ width: 120 }}>Category</th>
                <th className="th" style={{ width: 90 }}>Item Count</th>
                <th className="th" style={{ width: 110 }}>Earliest Date</th>
                <th className="th" style={{ width: 110 }}>Latest Date</th>
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
      )}

      {/* 5. Forensic Findings Matrix */}
      {enabledSections.has("findings") && (
        <div style={{ marginBottom: 32 }}>
          <h3 style={{ fontSize: 15, fontWeight: 700, borderBottom: "1px solid var(--border)", paddingBottom: 6, marginBottom: 12, color: "var(--accent)" }}>
            5. Forensic Security Violations &amp; Risk Matrix
          </h3>
          {(reportData?.findings || []).length === 0 ? (
            <div className="muted text-sm">No security violations or tampering findings flagged.</div>
          ) : (
            <div style={{ display: "flex", flexDirection: "column", gap: 12 }}>
              {reportData?.findings.map((f: any) => (
                <div
                  key={f.id}
                  style={{
                    padding: 14,
                    background: "var(--bg-3)",
                    borderRadius: "var(--r-sm)",
                    borderLeft:
                      f.severity === "critical"
                        ? "4px solid #ef4444"
                        : f.severity === "high"
                        ? "4px solid #f97316"
                        : "4px solid #eab308",
                  }}
                >
                  <div className="row between mb-2">
                    <strong style={{ fontSize: 14, color: "var(--text-0)" }}>
                      {f.title}
                    </strong>
                    <div className="row gap-2">
                      <span
                        className={`badge ${
                          f.severity === "critical"
                            ? "badge-red"
                            : f.severity === "high"
                            ? "badge-orange"
                            : "badge-yellow"
                        }`}
                        style={{ fontSize: 9 }}
                      >
                        {f.severity.toUpperCase()}
                      </span>
                      <span className="badge badge-blue" style={{ fontSize: 9 }}>
                        TYPE: {f.type}
                      </span>
                      <span className="badge badge-green" style={{ fontSize: 9 }}>
                        STATUS: {f.status}
                      </span>
                    </div>
                  </div>
                  <p style={{ fontSize: 12, color: "var(--text-2)", margin: 0, lineHeight: 1.5 }}>
                    {f.description}
                  </p>
                </div>
              ))}
            </div>
          )}
        </div>
      )}

      {/* 6. Key Subject Dossier & Entity Matrix */}
      {enabledSections.has("target_dossier") && (
        <div style={{ marginBottom: 32 }}>
          <h3 style={{ fontSize: 15, fontWeight: 700, borderBottom: "1px solid var(--border)", paddingBottom: 6, marginBottom: 12, color: "var(--accent)" }}>
            6. Subject Dossier &amp; Top Correspondents Network (Top 30 Entities)
          </h3>
          {reportData?.target_profile ? (
            <div style={{ background: "var(--bg-3)", padding: 14, borderRadius: "var(--r-sm)", marginBottom: 14 }}>
              <div className="row between mb-2">
                <strong style={{ fontSize: 14, color: "var(--text-0)" }}>
                  {cleanDisplayName(reportData.target_profile.display_name) || reportData.target_profile.email}
                </strong>
                <span className="badge badge-orange">CASE TARGET</span>
              </div>
              <div style={{ fontSize: 11, color: "var(--text-2)", marginBottom: 6 }}>
                Primary Email: <code style={{ color: "var(--accent)" }}>{reportData.target_profile.email}</code>
              </div>
              {reportData.target_profile.aliases && (
                <div style={{ fontSize: 11, color: "var(--text-3)", marginBottom: 8 }}>
                  Discovered Aliases &amp; Exchange DNs: {reportData.target_profile.aliases}
                </div>
              )}
              <div className="row gap-4" style={{ fontSize: 12 }}>
                <div>Sent: <strong>{reportData.target_profile.sent}</strong></div>
                <div>Received: <strong>{reportData.target_profile.received}</strong></div>
                <div>Total Involvement: <strong>{reportData.target_profile.sent + reportData.target_profile.received}</strong></div>
              </div>
            </div>
          ) : null}

          <table style={{ width: "100%", fontSize: 11 }}>
            <thead>
              <tr>
                <th className="th">Entity Name</th>
                <th className="th">Email / Address</th>
                <th className="th" style={{ width: 80 }}>Sent</th>
                <th className="th" style={{ width: 90 }}>Received</th>
                <th className="th" style={{ width: 80 }}>Total</th>
              </tr>
            </thead>
            <tbody>
              {(reportData?.entities || []).slice(0, 30).map((e: any, i: number) => (
                <tr key={i}>
                  <td className="td">
                    <strong>{cleanDisplayName(e.display_name) || e.email.split("@")[0]}</strong>
                  </td>
                  <td className="td" style={{ fontFamily: "var(--mono)", fontSize: 10, color: "var(--text-2)" }}>
                    {e.email}
                  </td>
                  <td className="td" style={{ color: "#3b82f6" }}>{e.sent}</td>
                  <td className="td" style={{ color: "#22c55e" }}>{e.received}</td>
                  <td className="td"><strong>{e.sent + e.received}</strong></td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}

      {/* 7. Key Messages Ledger */}
      {enabledSections.has("key_ledger") && (reportData?.key_messages_ledger || []).length > 0 && (
        <div style={{ marginBottom: 32 }}>
          <h3 style={{ fontSize: 15, fontWeight: 700, borderBottom: "1px solid var(--border)", paddingBottom: 6, marginBottom: 12, color: "var(--accent)" }}>
            7. Evidentiary &amp; Flagged Messages Ledger (Top Suspicious / Deleted Items)
          </h3>
          <table style={{ width: "100%", fontSize: 10 }}>
            <thead>
              <tr>
                <th className="th" style={{ width: 140 }}>Sender</th>
                <th className="th">Subject</th>
                <th className="th" style={{ width: 80 }}>Date</th>
                <th className="th" style={{ width: 70 }}>Category</th>
                <th className="th" style={{ width: 45 }}>Risk</th>
              </tr>
            </thead>
            <tbody>
              {(reportData?.key_messages_ledger || []).slice(0, 50).map((em: any) => (
                <tr key={em.id}>
                  <td className="td" style={{ overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
                    {cleanDisplayName(em.from_display) || em.from_addr}
                  </td>
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
      )}

      {/* 8. Attachments Manifest */}
      {enabledSections.has("attachments") && (reportData?.attachments_manifest || []).length > 0 && (
        <div style={{ marginBottom: 32 }}>
          <h3 style={{ fontSize: 15, fontWeight: 700, borderBottom: "1px solid var(--border)", paddingBottom: 6, marginBottom: 12, color: "var(--accent)" }}>
            8. Extracted Attachments &amp; File Artifacts Manifest
          </h3>
          <table style={{ width: "100%", fontSize: 11 }}>
            <thead>
              <tr>
                <th className="th">Filename</th>
                <th className="th" style={{ width: 90 }}>Type</th>
                <th className="th" style={{ width: 80 }}>Size</th>
                <th className="th">Parent Email Subject</th>
                <th className="th">SHA-256 Hash</th>
              </tr>
            </thead>
            <tbody>
              {(reportData?.attachments_manifest || []).slice(0, 30).map((att: any, i: number) => (
                <tr key={i}>
                  <td className="td"><strong>{att.filename}</strong></td>
                  <td className="td muted">{att.file_type || "Binary"}</td>
                  <td className="td muted">{(att.size_bytes / 1024).toFixed(1)} KB</td>
                  <td className="td" style={{ overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap", maxWidth: 160 }}>
                    {att.email_subject || "—"}
                  </td>
                  <td className="td" style={{ fontFamily: "var(--mono)", fontSize: 9, color: "var(--accent)" }}>
                    {att.sha256 ? `${att.sha256.slice(0, 24)}...` : "—"}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}

      {/* 9. Marked Exhibits */}
      {enabledSections.has("exhibits") && exhibits.length > 0 && (
        <div style={{ marginBottom: 32 }}>
          <h3 style={{ fontSize: 15, fontWeight: 700, borderBottom: "1px solid var(--border)", paddingBottom: 6, marginBottom: 12, color: "var(--accent)" }}>
            9. Formal Evidentiary Exhibits &amp; Court Appendices
          </h3>
          <div style={{ display: "flex", flexDirection: "column", gap: 12 }}>
            {exhibits.map((ex) => (
              <div
                key={ex.id}
                style={{
                  padding: 14,
                  background: "var(--bg-3)",
                  borderRadius: "var(--r-sm)",
                  border: "1px solid var(--border)",
                }}
              >
                <div className="row between mb-2">
                  <strong style={{ fontSize: 14, color: "var(--accent)" }}>
                    {ex.exhibit_number}: {ex.subject}
                  </strong>
                  <span className="muted text-sm">{ex.date_sent}</span>
                </div>
                <div style={{ fontSize: 11, color: "var(--text-2)", marginBottom: 4 }}>
                  From: <strong>{ex.from_display || ex.from_addr}</strong>
                </div>
                <div style={{ fontSize: 10, color: "var(--text-3)" }}>
                  Investigator Annotation: {ex.notes}
                </div>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* 10. Chain of Custody */}
      {enabledSections.has("custody") && (
        <div style={{ marginBottom: 32 }}>
          <h3 style={{ fontSize: 15, fontWeight: 700, borderBottom: "1px solid var(--border)", paddingBottom: 6, marginBottom: 12, color: "var(--accent)" }}>
            10. Cryptographic Chain of Custody &amp; Verification Log
          </h3>
          <table style={{ width: "100%", fontSize: 11 }}>
            <thead>
              <tr>
                <th className="th" style={{ width: 140 }}>Timestamp</th>
                <th className="th" style={{ width: 120 }}>Action</th>
                <th className="th" style={{ width: 100 }}>Examiner</th>
                <th className="th">Forensic Verification Details</th>
              </tr>
            </thead>
            <tbody>
              {(reportData?.custody_chain || []).map((c: any, i: number) => (
                <tr key={i}>
                  <td className="td muted">{new Date(c.timestamp).toLocaleString()}</td>
                  <td className="td"><strong>{c.action}</strong></td>
                  <td className="td">{c.actor}</td>
                  <td className="td">{c.detail || "Verifiable acquisition integrity check"}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}

      {/* 11. Examiner Certification */}
      {enabledSections.has("certification") && <ReportCertificationCard />}
    </div>
  );
}
