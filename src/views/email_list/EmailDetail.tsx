import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Email, EmailTag } from "./types";
import { RichEmailBodyViewer } from "../../components/RichEmailBodyViewer";
import { EmailNotesAndTagsTab } from "./EmailNotesAndTagsTab";
import { EmailAttachmentsTab } from "./EmailAttachmentsTab";

interface Props {
  email: Email;
  caseId: string;
  evidenceName?: string;
  tags: EmailTag[];
  onTagsChanged: () => void;
  onClose: () => void;
}

export function EmailDetail({
  email: initialEmail,
  caseId,
  evidenceName,
  tags,
  onTagsChanged,
  onClose,
}: Props) {
  const [email, setEmail] = useState<Email>(initialEmail);

  useEffect(() => {
    setEmail(initialEmail);
    if (!initialEmail.body_text && !initialEmail.body_html && !initialEmail.headers_raw) {
      invoke<Email | null>("email_get", { input: { id: initialEmail.id } }).then((full) => {
        if (full) setEmail(full);
      }).catch(console.error);
    }
  }, [initialEmail.id]);

  const [tab, setTab] = useState<
    "overview" | "notes" | "headers" | "auth" | "mime" | "raw" | "attachments"
  >("overview");
  const [analysisData, setAnalysisData] = useState<any>(null);
  const [analysisLoading, setAnalysisLoading] = useState(false);

  useEffect(() => {
    if ((tab === "auth" || tab === "headers") && !analysisData && !analysisLoading) {
      setAnalysisLoading(true);
      invoke<any>("email_headers", { emailId: email.id })
        .then((data) => setAnalysisData(data))
        .catch(console.error)
        .finally(() => setAnalysisLoading(false));
    }
  }, [tab, email.id, analysisData, analysisLoading]);

  let toList: string[] = [];
  let ccList: string[] = [];
  try {
    toList = JSON.parse(email.to_addrs || "[]");
  } catch {}
  try {
    ccList = JSON.parse(email.cc_addrs || "[]");
  } catch {}

  const riskColor =
    email.risk_score >= 50
      ? "var(--danger)"
      : email.risk_score >= 25
      ? "var(--warning)"
      : "var(--success)";
  const riskLabel =
    email.risk_score >= 50 ? "HIGH" : email.risk_score >= 25 ? "MEDIUM" : "LOW";

  const tabs = [
    { key: "overview", label: "Overview" },
    { key: "notes", label: `Notes & Tags ${tags.length > 0 ? `(${tags.length})` : ""}` },
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
          <div className="row gap-2 mb-1" style={{ flexWrap: "wrap" }}>
            <h2 style={{ fontSize: 16, fontWeight: 600, color: "var(--text-0)" }}>
              {email.subject || "(no subject)"}
            </h2>
            {tags.map((t) => (
              <span
                key={t.id}
                className="badge"
                style={{
                  background: `${t.color}22`,
                  color: t.color,
                  border: `1px solid ${t.color}44`,
                  fontSize: 10,
                }}
              >
                🏷️ {t.tag}
              </span>
            ))}
          </div>
          <p className="muted" style={{ fontSize: 12 }}>
            From: {email.from_addr} ·{" "}
            {email.date_sent ? new Date(email.date_sent).toLocaleString() : "—"}
          </p>
          {evidenceName && (
            <p className="muted" style={{ fontSize: 11 }}>
              Source: {evidenceName}
            </p>
          )}
        </div>
        <button className="btn btn-ghost btn-sm" onClick={onClose}>
          ← Back to Emails
        </button>
      </div>

      <div
        className="row gap-2 mb-4"
        style={{ borderBottom: "1px solid var(--border)", paddingBottom: 0 }}
      >
        {tabs.map((t) => (
          <button
            key={t.key}
            className={`btn btn-sm ${tab === t.key ? "btn-primary" : "btn-ghost"}`}
            style={{ borderRadius: "6px 6px 0 0" }}
            onClick={() => setTab(t.key as any)}
          >
            {t.label}
          </button>
        ))}
      </div>

      <div className="card" style={{ marginTop: 0 }}>
        {tab === "overview" && (
          <div>
            <div className="grid-2 mb-4">
              <div>
                <span className="muted">From</span>
                <p style={{ fontWeight: 500 }}>{email.from_display || email.from_addr}</p>
              </div>
              <div>
                <span className="muted">Date</span>
                <p>{email.date_sent ? new Date(email.date_sent).toLocaleString() : "—"}</p>
              </div>
            </div>
            <div className="mb-4">
              <span className="muted">To</span>
              <p className="mono">{toList.join(", ")}</p>
            </div>
            {ccList.length > 0 && (
              <div className="mb-4">
                <span className="muted">CC</span>
                <p className="mono">{ccList.join(", ")}</p>
              </div>
            )}
            <div className="mb-4">
              <span className="muted">Message-ID</span>
              <p className="mono text-sm">{email.message_id || "—"}</p>
            </div>
            <div className="mb-4">
              <span className="muted">Forensic Tags</span>
              <div className="row gap-2 mt-1" style={{ flexWrap: "wrap" }}>
                {tags.length === 0 ? (
                  <span className="muted" style={{ fontSize: 12 }}>
                    No tags assigned yet.
                  </span>
                ) : (
                  tags.map((t) => (
                    <span
                      key={t.id}
                      className="badge"
                      style={{
                        background: `${t.color}22`,
                        color: t.color,
                        border: `1px solid ${t.color}44`,
                      }}
                    >
                      {t.tag}
                    </span>
                  ))
                )}
                <button
                  className="btn btn-ghost btn-sm"
                  style={{ padding: "2px 8px", fontSize: 11 }}
                  onClick={() => setTab("notes")}
                >
                  + Manage Tags &amp; Notes
                </button>
              </div>
            </div>
            <div className="mb-4">
              <span className="muted">Risk Score</span>
              <p style={{ fontWeight: 600, color: riskColor }}>
                {email.risk_score}/100 ({riskLabel})
              </p>
            </div>
            {email.deleted_recovered && (
              <div className="mb-4">
                <span className="badge badge-red">DELETED / RECOVERED</span>
              </div>
            )}
            <div className="mb-4">
              <span className="muted" style={{ fontWeight: 600 }}>Message Content</span>
              <RichEmailBodyViewer
                bodyText={email.body_text}
                bodyHtml={email.body_html}
                emailId={email.id}
                defaultMode="rendered"
              />
            </div>
          </div>
        )}

        {tab === "notes" && (
          <EmailNotesAndTagsTab
            emailId={email.id}
            caseId={caseId}
            tags={tags}
            onTagsChanged={onTagsChanged}
          />
        )}

        {tab === "headers" && (
          <div>
            {analysisLoading && <div className="empty">Analyzing headers...</div>}
            {analysisData?.header_analysis && (
              <div className="mb-4">
                <h4 style={{ fontSize: 13, fontWeight: 600, marginBottom: 12 }}>
                  Header Analysis Summary
                </h4>
                <div className="analysis-summary">
                  <div className="analysis-stat">
                    <div className="analysis-stat-val">
                      {analysisData.header_analysis.received_chain?.length || 0}
                    </div>
                    <div className="analysis-stat-label">Received Hops</div>
                  </div>
                  <div className="analysis-stat">
                    <div className="analysis-stat-val" style={{ fontSize: 14 }}>
                      {analysisData.header_analysis.originating_ip || "Unknown"}
                    </div>
                    <div className="analysis-stat-label">Originating IP</div>
                  </div>
                  <div className="analysis-stat">
                    <div
                      className="analysis-stat-val"
                      style={{
                        color:
                          analysisData.header_analysis.routing_anomalies?.length > 0
                            ? "var(--danger)"
                            : "var(--text-0)",
                      }}
                    >
                      {analysisData.header_analysis.routing_anomalies?.length || 0}
                    </div>
                    <div className="analysis-stat-label">Routing Anomalies</div>
                  </div>
                  <div className="analysis-stat">
                    <div
                      className="analysis-stat-val"
                      style={{
                        color:
                          analysisData.header_analysis.clock_skew?.length > 0
                            ? "var(--warning)"
                            : "var(--text-0)",
                      }}
                    >
                      {analysisData.header_analysis.clock_skew?.length || 0}
                    </div>
                    <div className="analysis-stat-label">Clock Skew Events</div>
                  </div>
                </div>
              </div>
            )}
            <pre
              className="mono"
              style={{
                fontSize: 11,
                background: "var(--bg-0)",
                padding: 16,
                borderRadius: "var(--r-md)",
                border: "1px solid var(--border)",
                maxHeight: 500,
                overflow: "auto",
              }}
            >
              {email.headers_raw}
            </pre>
          </div>
        )}

        {tab === "auth" && (
          <div>
            <h4 style={{ fontSize: 13, fontWeight: 600, marginBottom: 12 }}>
              Authentication Results (SPF / DKIM / DMARC)
            </h4>
            <pre
              className="mono"
              style={{
                fontSize: 12,
                background: "var(--bg-0)",
                padding: 16,
                borderRadius: "var(--r-md)",
                border: "1px solid var(--border)",
              }}
            >
              {analysisData
                ? JSON.stringify(analysisData.authentication || {}, null, 2)
                : "Loading authentication verification..."}
            </pre>
          </div>
        )}

        {tab === "mime" && (
          <div>
            <h4 style={{ fontSize: 13, fontWeight: 600, marginBottom: 12 }}>MIME Tree Structure</h4>
            <div
              style={{
                padding: 16,
                background: "var(--bg-0)",
                borderRadius: "var(--r-md)",
                border: "1px solid var(--border)",
                fontSize: 13,
              }}
            >
              <div>
                📦 <strong>multipart/alternative</strong>
              </div>
              <div style={{ marginLeft: 20 }}>
                ├── 📄 text/plain ({email.body_text?.length || 0} chars)
              </div>
              {email.body_html && (
                <div style={{ marginLeft: 20 }}>
                  └── 🌐 text/html ({email.body_html.length} chars)
                </div>
              )}
            </div>
          </div>
        )}

        {tab === "raw" && (
          <div>
            <h4 style={{ fontSize: 13, fontWeight: 600, marginBottom: 12 }}>Full Raw Message</h4>
            <pre
              className="mono"
              style={{
                fontSize: 11,
                color: "var(--text-2)",
                whiteSpace: "pre-wrap",
                wordBreak: "break-all",
                maxHeight: 600,
                overflow: "auto",
                background: "var(--bg-0)",
                padding: 16,
                borderRadius: "var(--r-md)",
                border: "1px solid var(--border)",
              }}
            >
              {email.headers_raw || "No raw headers available"}
              {"\n\n--- BODY ---\n\n"}
              {email.body_text || email.body_html || "No body available"}
            </pre>
          </div>
        )}

        {tab === "attachments" && (
          <div>
            <h4 style={{ fontSize: 13, fontWeight: 600, marginBottom: 12 }}>Attachments</h4>
            <EmailAttachmentsTab emailId={email.id} />
          </div>
        )}
      </div>
    </div>
  );
}
