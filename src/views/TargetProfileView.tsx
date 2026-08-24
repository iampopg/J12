import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";

export interface TargetProfile {
  case_id: string;
  case_title: string;
  case_number: string;
  target_email: string | null;
  target_name: string | null;
  target_organization: string | null;
  sent_count: number;
  received_count: number;
  total_emails: number;
  first_seen: string | null;
  last_seen: string | null;
  top_correspondents: [string, number][];
  top_subjects: [string, number][];
  display_names: string[];
  x_mailers: string[];
  originating_ips: string[];
  risk_score: number;
  flagged_count: number;
  attachment_count: number;
  recent_communications: Array<{
    id: string;
    subject: string;
    date: string | null;
    from: string;
    to: string;
    risk_score: number;
  }>;
}

export interface DetectedTarget {
  email: string;
  display_name: string | null;
  organization: string;
  total_emails: number;
  sent: number;
  received: number;
  confidence: "high" | "medium" | "low";
  is_primary_target: boolean;
}

interface Props {
  caseId: string;
  caseData?: any;
  onSelectEmail?: (emailId: string) => void;
}

export function TargetProfileView({ caseId, caseData, onSelectEmail }: Props) {
  const [profile, setProfile] = useState<TargetProfile | null>(null);
  const [detected, setDetected] = useState<DetectedTarget[]>([]);
  const [totalEntities, setTotalEntities] = useState<number>(0);
  const [loading, setLoading] = useState(true);
  const [selectedEmail, setSelectedEmail] = useState<string | null>(null);
  const [showAllCandidates, setShowAllCandidates] = useState(false);
  const [toastMessage, setToastMessage] = useState<string | null>(null);

  const showToast = (msg: string) => {
    setToastMessage(msg);
    setTimeout(() => setToastMessage(null), 3500);
  };

  useEffect(() => {
    loadData();
  }, [caseId]);

  const loadData = async () => {
    setLoading(true);
    try {
      const det = await invoke<any>("auto_detect_targets", { input: { case_id: caseId } });
      const targets: DetectedTarget[] = det.targets || det.candidates || [];
      setDetected(targets);
      setTotalEntities(det.total_case_entities || targets.length);

      const targetToLoad = selectedEmail || (targets.length > 0 ? targets[0].email : caseData?.target_email);
      if (targetToLoad) {
        setSelectedEmail(targetToLoad);
        await loadProfile(targetToLoad);
      } else if (caseData?.target_email) {
        setSelectedEmail(caseData.target_email);
        await loadProfile(caseData.target_email);
      }
    } catch (e) {
      console.error("Failed to load target data:", e);
    } finally {
      setLoading(false);
    }
  };

  const loadProfile = async (email: string) => {
    try {
      const prof = await invoke<TargetProfile>("target_profile", {
        input: { case_id: caseId, target_email: email }
      });
      setProfile(prof);
    } catch (e) {
      console.error("Failed to load profile:", e);
    }
  };

  const reExtract = async () => {
    setLoading(true);
    try {
      const count = await invoke<number>("extract_entities", { input: { case_id: caseId } });
      showToast(`⚡ Re-extracted and unified ${count} entities across case`);
      await loadData();
    } catch (e) {
      console.error(e);
    } finally {
      setLoading(false);
    }
  };

  const handleSelectCandidate = (email: string) => {
    setSelectedEmail(email);
    loadProfile(email);
  };

  // Helper to format clean display name
  const formatName = (d: string | null | undefined, email: string) => {
    if (d && d.trim() && d !== email && !d.startsWith("/")) {
      if (d.includes("..")) {
        const p = d.split("..");
        if (p.length === 2) return `${p[0].toUpperCase()}. ${p[1].charAt(0).toUpperCase() + p[1].slice(1)}`;
      }
      return d;
    }
    const local = email.split("@")[0] || email;
    if (local.includes("..")) {
      const p = local.split("..");
      if (p.length === 2) return `${p[0].toUpperCase()}. ${p[1].charAt(0).toUpperCase() + p[1].slice(1)}`;
    } else if (local.includes(".")) {
      return local.split(".").map(w => w.charAt(0).toUpperCase() + w.slice(1)).join(" ");
    }
    return local.charAt(0).toUpperCase() + local.slice(1);
  };

  const currentEmail = selectedEmail || caseData?.target_email || profile?.target_email || "";
  const currentName = formatName(profile?.target_name || caseData?.target_name, currentEmail);
  const currentOrg = profile?.target_organization || caseData?.target_organization || (currentEmail.includes("@") ? currentEmail.split("@")[1] : "N/A");

  const riskScore = profile?.risk_score || 0;
  const riskColor = riskScore >= 50 ? "var(--danger)" : riskScore >= 25 ? "var(--warning)" : "var(--success)";
  const riskLabel = riskScore >= 50 ? "HIGH RISK" : riskScore >= 25 ? "MODERATE" : "LOW RISK";

  const formatDateSpan = (d: string | null) => {
    if (!d) return "N/A";
    try {
      const date = new Date(d);
      return date.toLocaleDateString(undefined, { year: "numeric", month: "short", day: "numeric" });
    } catch {
      return d;
    }
  };

  if (loading && !profile) {
    return <div className="empty" style={{ padding: 40 }}>Loading Target Subject Dossier...</div>;
  }

  return (
    <div>
      {/* Toast Notification */}
      {toastMessage && (
        <div 
          className="card"
          style={{
            position: "fixed",
            bottom: 24,
            right: 24,
            zIndex: 9999,
            background: "#1e293b",
            border: "1px solid #22c55e",
            color: "#4ade80",
            padding: "12px 20px",
            fontWeight: 600,
            fontSize: 13,
            boxShadow: "0 10px 25px rgba(0,0,0,0.5)",
            display: "flex",
            alignItems: "center",
            gap: 10,
          }}
        >
          <span>✓</span>
          <span>{toastMessage}</span>
        </div>
      )}

      {/* Top Header */}
      <div className="row between mb-4">
        <div>
          <h2 style={{ fontSize: 22, fontWeight: 700, color: "var(--text-0)" }}>Target Subject Dossier</h2>
          <p className="muted" style={{ fontSize: 13 }}>
            Principal Person of Interest Intelligence &amp; Communication Topology
          </p>
        </div>
        <div className="row gap-2">
          <button className="btn btn-ghost btn-sm" onClick={reExtract} title="Re-scan and normalize all entities">
            ⚡ Re-Extract Entities
          </button>
          <button className="btn btn-ghost btn-sm" onClick={loadData}>
            ↻ Refresh
          </button>
        </div>
      </div>

      {/* Main Identity Banner Card */}
      <div className="card mb-4" style={{ borderLeft: "4px solid var(--accent)", boxShadow: "0 4px 20px rgba(0,0,0,0.15)", padding: 24 }}>
        <div className="row between" style={{ alignItems: "flex-start", flexWrap: "wrap", gap: 16 }}>
          <div style={{ display: "flex", alignItems: "center", gap: 20 }}>
            <div style={{
              width: 80,
              height: 80,
              borderRadius: "50%",
              background: "linear-gradient(135deg, #2563eb, #7c3aed)",
              display: "flex",
              alignItems: "center",
              justifyContent: "center",
              fontSize: 34,
              color: "#fff",
              fontWeight: 800,
              boxShadow: "0 4px 12px rgba(37,99,235,0.3)"
            }}>
              {(currentName ? currentName.charAt(0) : "T").toUpperCase()}
            </div>
            <div>
              <div className="row gap-2" style={{ alignItems: "center", marginBottom: 4 }}>
                <h3 style={{ fontSize: 22, fontWeight: 700, color: "var(--text-0)", margin: 0 }}>
                  {currentName || "Unknown Subject"}
                </h3>
                <span className="badge badge-blue" style={{ fontSize: 11 }}>
                  {selectedEmail === caseData?.target_email ? "🎯 PRIMARY TARGET" : "👤 PERSON OF INTEREST"}
                </span>
                {caseData?.case_number && (
                  <span className="badge badge-gray" style={{ fontSize: 10 }}>
                    CASE #{caseData.case_number}
                  </span>
                )}
              </div>
              <div style={{ fontSize: 14, color: "var(--accent)", fontFamily: "var(--mono)", marginBottom: 6 }}>
                {currentEmail || "No primary email assigned"}
              </div>
              <div style={{ fontSize: 12, color: "var(--text-2)" }}>
                Organization: <strong>{currentOrg}</strong> · Investigation: <strong>{caseData?.investigation_type || "General Forensic"}</strong>
              </div>
            </div>
          </div>

          <div style={{ textAlign: "right", minWidth: 120 }}>
            <div style={{ fontSize: 10, fontWeight: 700, color: "var(--text-3)", letterSpacing: "0.06em", marginBottom: 2 }}>
              SUBJECT THREAT SCORE
            </div>
            <div style={{ fontSize: 36, fontWeight: 800, color: riskColor, lineHeight: 1.1 }}>
              {riskScore}
            </div>
            <div style={{ fontSize: 11, fontWeight: 700, color: riskColor }}>
              {riskLabel}
            </div>
          </div>
        </div>

        {/* Display Names & Aliases */}
        {profile?.display_names && profile.display_names.length > 0 && (
          <div style={{ marginTop: 18, paddingTop: 14, borderTop: "1px solid var(--border)", display: "flex", alignItems: "center", gap: 8, flexWrap: "wrap" }}>
            <span style={{ fontSize: 11, fontWeight: 600, color: "var(--text-3)" }}>KNOWN ALIASES / SIGNATURES:</span>
            {profile.display_names.map((name, i) => (
              <span key={i} className="badge badge-gray" style={{ fontSize: 11, padding: "3px 8px" }}>
                "{name}"
              </span>
            ))}
          </div>
        )}
      </div>

      {/* KPI Stats Grid */}
      <div className="kpi-grid mb-4">
        <div className="kpi">
          <div className="kpi-val" style={{ color: "var(--accent)" }}>
            {(profile?.sent_count || 0).toLocaleString()}
          </div>
          <div className="kpi-label">📤 Outbound Sent</div>
        </div>
        <div className="kpi">
          <div className="kpi-val" style={{ color: "var(--success)" }}>
            {(profile?.received_count || 0).toLocaleString()}
          </div>
          <div className="kpi-label">📥 Inbound Received</div>
        </div>
        <div className="kpi">
          <div className="kpi-val">
            {(profile?.total_emails || 0).toLocaleString()}
          </div>
          <div className="kpi-label">✉️ Total Interactions</div>
        </div>
        <div className="kpi">
          <div className="kpi-val" style={{ color: (profile?.flagged_count || 0) > 0 ? "var(--danger)" : "var(--text-1)" }}>
            {(profile?.flagged_count || 0).toLocaleString()}
          </div>
          <div className="kpi-label">🚨 Flagged Suspicious</div>
        </div>
        <div className="kpi">
          <div className="kpi-val" style={{ color: "var(--warning)" }}>
            {(profile?.attachment_count || 0).toLocaleString()}
          </div>
          <div className="kpi-label">📎 Files Exchanged</div>
        </div>
      </div>

      {/* Candidate Persons of Interest Selector */}
      {detected.length > 1 && (
        <div className="card mb-4">
          <div className="row between mb-3">
            <div>
              <h4 style={{ fontSize: 13, fontWeight: 700, color: "var(--text-0)" }}>
                Detected Candidate Subjects ({detected.length})
              </h4>
              <p className="muted" style={{ fontSize: 11, margin: 0 }}>
                Select any subject to pivot and inspect their individual communication network:
              </p>
            </div>
            <button className="btn btn-ghost btn-sm" onClick={() => setShowAllCandidates(!showAllCandidates)}>
              {showAllCandidates ? "▲ Show Fewer" : `▼ View All (${detected.length})`}
            </button>
          </div>
          <div style={{ display: "grid", gridTemplateColumns: "repeat(auto-fill, minmax(240px, 1fr))", gap: 8 }}>
            {(showAllCandidates ? detected : detected.slice(0, 6)).map((t, i) => {
              const name = formatName(t.display_name, t.email);
              const isSelected = t.email === currentEmail;
              return (
                <div
                  key={i}
                  className="row between tr-click"
                  style={{
                    padding: "8px 12px",
                    background: isSelected ? "var(--accent-subtle)" : "var(--bg-3)",
                    border: isSelected ? "1px solid var(--accent)" : "1px solid var(--border)",
                    borderRadius: "var(--r-sm)",
                    cursor: "pointer"
                  }}
                  onClick={() => handleSelectCandidate(t.email)}
                >
                  <div style={{ overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap", paddingRight: 8 }}>
                    <div style={{ fontSize: 12, fontWeight: 600, color: isSelected ? "var(--accent)" : "var(--text-0)" }}>
                      {name}
                    </div>
                    <div style={{ fontSize: 10, color: "var(--text-3)", fontFamily: "var(--mono)" }}>
                      {t.email}
                    </div>
                  </div>
                  <span className={`badge ${isSelected ? "badge-blue" : "badge-gray"}`} style={{ fontSize: 10 }}>
                    {t.total_emails} msgs
                  </span>
                </div>
              );
            })}
          </div>
        </div>
      )}

      {/* Forensic Telemetry Grid (Span, Mailers, IPs) */}
      <div className="grid-3 mb-4">
        {/* Active Communication Span */}
        <div className="card mb-0">
          <div style={{ fontSize: 11, fontWeight: 700, color: "var(--text-3)", letterSpacing: "0.05em", marginBottom: 8 }}>
            📅 COMMUNICATION TIMELINE SPAN
          </div>
          <div style={{ display: "flex", flexDirection: "column", gap: 8 }}>
            <div>
              <div style={{ fontSize: 10, color: "var(--text-3)" }}>FIRST INTERACTION:</div>
              <div style={{ fontSize: 13, fontWeight: 600, color: "var(--text-1)", fontFamily: "var(--mono)" }}>
                {formatDateSpan(profile?.first_seen || null)}
              </div>
            </div>
            <div>
              <div style={{ fontSize: 10, color: "var(--text-3)" }}>LAST RECORDED:</div>
              <div style={{ fontSize: 13, fontWeight: 600, color: "var(--text-1)", fontFamily: "var(--mono)" }}>
                {formatDateSpan(profile?.last_seen || null)}
              </div>
            </div>
          </div>
        </div>

        {/* Detected Mail Clients / Software */}
        <div className="card mb-0">
          <div style={{ fontSize: 11, fontWeight: 700, color: "var(--text-3)", letterSpacing: "0.05em", marginBottom: 8 }}>
            💻 DETECTED MAIL CLIENT SOFTWARE
          </div>
          {profile?.x_mailers && profile.x_mailers.length > 0 ? (
            <div style={{ display: "flex", flexDirection: "column", gap: 4 }}>
              {profile.x_mailers.map((m, i) => (
                <div key={i} style={{ fontSize: 11, color: "var(--text-1)", fontFamily: "var(--mono)", background: "var(--bg-3)", padding: "4px 8px", borderRadius: "var(--r-xs)" }}>
                  {m}
                </div>
              ))}
            </div>
          ) : (
            <div className="muted" style={{ fontSize: 12 }}>No X-Mailer headers extracted</div>
          )}
        </div>

        {/* Originating IP Addresses */}
        <div className="card mb-0">
          <div style={{ fontSize: 11, fontWeight: 700, color: "var(--text-3)", letterSpacing: "0.05em", marginBottom: 8 }}>
            🌐 ORIGINATING IP ADDRESSES
          </div>
          {profile?.originating_ips && profile.originating_ips.length > 0 ? (
            <div style={{ display: "flex", flexDirection: "column", gap: 4 }}>
              {profile.originating_ips.map((ip, i) => (
                <div key={i} style={{ fontSize: 11, color: "var(--accent)", fontFamily: "var(--mono)", background: "var(--bg-3)", padding: "4px 8px", borderRadius: "var(--r-xs)" }}>
                  {ip}
                </div>
              ))}
            </div>
          ) : (
            <div className="muted" style={{ fontSize: 12 }}>No IP headers extracted</div>
          )}
        </div>
      </div>

      {/* Network & Topics Grid */}
      <div className="grid-2 mb-4">
        {/* Top Direct Correspondents */}
        <div className="card mb-0">
          <h3 style={{ fontSize: 14, fontWeight: 700, marginBottom: 12, color: "var(--text-0)" }}>
            👥 Top Direct Correspondents Network
          </h3>
          {profile?.top_correspondents && profile.top_correspondents.length > 0 ? (
            <div style={{ display: "flex", flexDirection: "column", gap: 6 }}>
              {profile.top_correspondents.map(([email, count], i) => (
                <div 
                  key={i} 
                  className="row between tr-click" 
                  style={{ padding: "8px 10px", background: "var(--bg-3)", borderRadius: "var(--r-xs)" }}
                  onClick={() => handleSelectCandidate(email)}
                  title="Click to pivot target to this contact"
                >
                  <span style={{ fontSize: 12, fontFamily: "var(--mono)", color: "var(--text-1)", overflow: "hidden", textOverflow: "ellipsis" }}>
                    {email}
                  </span>
                  <span className="badge badge-blue" style={{ fontSize: 10 }}>{count} msgs</span>
                </div>
              ))}
            </div>
          ) : (
            <div className="muted text-sm">No direct correspondent data available.</div>
          )}
        </div>

        {/* Top Subjects & Topics */}
        <div className="card mb-0">
          <h3 style={{ fontSize: 14, fontWeight: 700, marginBottom: 12, color: "var(--text-0)" }}>
            ✉️ Frequent Investigation Subjects &amp; Threads
          </h3>
          {profile?.top_subjects && profile.top_subjects.length > 0 ? (
            <div style={{ display: "flex", flexDirection: "column", gap: 6 }}>
              {profile.top_subjects.map(([subject, count], i) => (
                <div key={i} className="row between" style={{ padding: "8px 10px", background: "var(--bg-3)", borderRadius: "var(--r-xs)" }}>
                  <span style={{ fontSize: 12, color: "var(--text-1)", overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap", maxWidth: "80%" }}>
                    {subject}
                  </span>
                  <span className="badge badge-gray" style={{ fontSize: 10 }}>{count}</span>
                </div>
              ))}
            </div>
          ) : (
            <div className="muted text-sm">No frequent subject topics recorded.</div>
          )}
        </div>
      </div>

      {/* Recent Communications Stream */}
      {profile?.recent_communications && profile.recent_communications.length > 0 && (
        <div className="card">
          <h3 style={{ fontSize: 14, fontWeight: 700, marginBottom: 12, color: "var(--text-0)" }}>
            🕒 Recent Communications Stream ({profile.recent_communications.length} Messages)
          </h3>
          <table>
            <thead>
              <tr>
                <th className="th">Date</th>
                <th className="th">From</th>
                <th className="th">To</th>
                <th className="th">Subject</th>
                <th className="th">Risk</th>
              </tr>
            </thead>
            <tbody>
              {profile.recent_communications.map((msg) => (
                <tr 
                  key={msg.id} 
                  className="tr-click"
                  onClick={() => onSelectEmail && onSelectEmail(msg.id)}
                  title="Click to view email details"
                >
                  <td className="td muted" style={{ fontSize: 11, fontFamily: "var(--mono)", whiteSpace: "nowrap" }}>
                    {formatDateSpan(msg.date)}
                  </td>
                  <td className="td" style={{ fontSize: 11, fontFamily: "var(--mono)", maxWidth: 140, overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
                    {msg.from}
                  </td>
                  <td className="td" style={{ fontSize: 11, fontFamily: "var(--mono)", maxWidth: 140, overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
                    {msg.to}
                  </td>
                  <td className="td" style={{ fontSize: 12, fontWeight: 500, color: "var(--text-0)" }}>
                    {msg.subject}
                  </td>
                  <td className="td">
                    <span className={`badge ${msg.risk_score >= 50 ? "badge-red" : msg.risk_score >= 25 ? "badge-orange" : "badge-gray"}`} style={{ fontSize: 10 }}>
                      {msg.risk_score > 0 ? `Risk: ${msg.risk_score}` : "Normal"}
                    </span>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
    </div>
  );
}
