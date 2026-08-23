import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";

interface TargetProfile {
  case_id: string;
  case_title: string;
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
  risk_score: number;
  display_names: string[];
}

interface DetectedTarget {
  email: string;
  display_name: string | null;
  total_emails: number;
  sent: number;
  received: number;
}

interface Props {
  caseId: string;
  caseData: any;
}

export function TargetProfileView({ caseId, caseData }: Props) {
  const [profile, setProfile] = useState<TargetProfile | null>(null);
  const [detected, setDetected] = useState<DetectedTarget[]>([]);
  const [loading, setLoading] = useState(true);
  const [showSelector, setShowSelector] = useState(false);

  useEffect(() => {
    loadData();
  }, [caseId]);

  const loadData = async () => {
    setLoading(true);
    try {
      const [prof, det] = await Promise.all([
        invoke<TargetProfile>("target_profile", { caseId }),
        invoke<any>("auto_detect_targets", { caseId }),
      ]);
      setProfile(prof);
      setDetected(det.targets || []);
    } catch (e) {
      console.error("Failed to load target data:", e);
    }
    setLoading(false);
  };

  if (loading) return <div className="empty">Loading target profile...</div>;

  const targetEmail = profile?.target_email || caseData?.target_email;
  const targetName = profile?.target_name || caseData?.target_name;
  const targetOrg = profile?.target_organization || caseData?.target_organization;

  if (!targetEmail && !targetName) {
    return (
      <div>
        <div className="row between mb-4">
          <div>
            <h2 style={{ fontSize: 22, fontWeight: 700, color: "var(--text-0)" }}>Target Profile</h2>
            <p className="muted">Auto-detected potential targets from email data</p>
          </div>
          <button className="btn btn-ghost btn-sm" onClick={loadData}>↻ Refresh</button>
        </div>

        {/* Auto-detected targets */}
        {detected.length > 0 ? (
          <div className="card">
            <h3 style={{ fontSize: 14, fontWeight: 600, marginBottom: 16 }}>Select a Target to Investigate</h3>
            <p className="muted mb-4" style={{ fontSize: 12 }}>
              The following email addresses appear most frequently in this case. Select one to set as the investigation target.
            </p>
            <table>
              <thead>
                <tr>
                  <th className="th">Email Address</th>
                  <th className="th">Display Name</th>
                  <th className="th" style={{ width: 80 }}>Sent</th>
                  <th className="th" style={{ width: 80 }}>Received</th>
                  <th className="th" style={{ width: 80 }}>Total</th>
                  <th className="th" style={{ width: 100 }}>Action</th>
                </tr>
              </thead>
              <tbody>
                {detected.map((t, i) => (
                  <tr key={i}>
                    <td className="td" style={{ fontFamily: "var(--mono)", color: "var(--accent)" }}>{t.email}</td>
                    <td className="td">{t.display_name || <span className="muted">—</span>}</td>
                    <td className="td">{t.sent}</td>
                    <td className="td">{t.received}</td>
                    <td className="td"><strong>{t.total_emails}</strong></td>
                    <td className="td">
                      <button className="btn btn-primary btn-sm" onClick={() => selectTarget(t)}>
                        Select
                      </button>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        ) : (
          <div className="card" style={{ textAlign: "center", padding: "60px 40px" }}>
            <div style={{ fontSize: 48, marginBottom: 16 }}>👤</div>
            <h3 style={{ fontSize: 18, marginBottom: 8, color: "var(--text-0)" }}>No Targets Detected</h3>
            <p className="muted">Upload and parse email data to auto-detect potential investigation targets.</p>
          </div>
        )}
      </div>
    );
  }

  const riskColor = (profile?.risk_score || 0) >= 50 ? "var(--danger)" : (profile?.risk_score || 0) >= 25 ? "var(--warning)" : "var(--success)";
  const riskLabel = (profile?.risk_score || 0) >= 50 ? "HIGH RISK" : (profile?.risk_score || 0) >= 25 ? "MEDIUM RISK" : "LOW RISK";

  const selectTarget = async (t: DetectedTarget) => {
    // TODO: Save selected target to case
    setShowSelector(false);
    loadData();
  };

  return (
    <div>
      <div className="row between mb-4">
        <div>
          <h2 style={{ fontSize: 22, fontWeight: 700, color: "var(--text-0)" }}>Target Profile</h2>
          <p className="muted">Subject investigation overview</p>
        </div>
        <div className="row gap-2">
          <button className="btn btn-ghost btn-sm" onClick={() => setShowSelector(!showSelector)}>
            {showSelector ? "Cancel" : "🔄 Change Target"}
          </button>
          <button className="btn btn-ghost btn-sm" onClick={loadData}>↻ Refresh</button>
        </div>
      </div>

      {/* Target selector */}
      {showSelector && detected.length > 0 && (
        <div className="card mb-4" style={{ borderLeft: "4px solid var(--warning)" }}>
          <h3 style={{ fontSize: 14, fontWeight: 600, marginBottom: 12 }}>Select Different Target</h3>
          <table>
            <thead>
              <tr>
                <th className="th">Email</th>
                <th className="th">Name</th>
                <th className="th">Total</th>
                <th className="th">Action</th>
              </tr>
            </thead>
            <tbody>
              {detected.map((t, i) => (
                <tr key={i} style={{ background: t.email === targetEmail ? "var(--accent-subtle)" : "transparent" }}>
                  <td className="td mono">{t.email}</td>
                  <td className="td">{t.display_name || "—"}</td>
                  <td className="td">{t.total_emails}</td>
                  <td className="td">
                    {t.email === targetEmail ? (
                      <span className="badge badge-green">Active</span>
                    ) : (
                      <button className="btn btn-primary btn-sm" onClick={() => selectTarget(t)}>Select</button>
                    )}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}

      {/* Identity Card */}
      <div className="card mb-4" style={{ borderLeft: "4px solid var(--accent)" }}>
        <div className="row between" style={{ marginBottom: 16 }}>
          <div style={{ display: "flex", alignItems: "center", gap: 16 }}>
            <div style={{ width: 64, height: 64, borderRadius: "50%", background: "linear-gradient(135deg, #3b82f6, #6366f1)", display: "flex", alignItems: "center", justifyContent: "center", fontSize: 28, color: "#fff", fontWeight: 700 }}>
              {targetName ? targetName.charAt(0).toUpperCase() : targetEmail ? targetEmail.charAt(0).toUpperCase() : "?"}
            </div>
            <div>
              <h3 style={{ fontSize: 20, fontWeight: 700, color: "var(--text-0)" }}>{targetName || targetEmail || "Unknown"}</h3>
              {targetEmail && <p style={{ fontSize: 14, color: "var(--accent)", fontFamily: "var(--mono)" }}>{targetEmail}</p>}
              {targetOrg && <p style={{ fontSize: 12, color: "var(--text-3)" }}>{targetOrg}</p>}
            </div>
          </div>
          <div style={{ textAlign: "right" }}>
            <div style={{ fontSize: 10, fontWeight: 600, color: "var(--text-3)", letterSpacing: "0.05em", marginBottom: 4 }}>RISK SCORE</div>
            <div style={{ fontSize: 32, fontWeight: 800, color: riskColor }}>{profile?.risk_score || 0}</div>
            <div style={{ fontSize: 11, color: riskColor, fontWeight: 600 }}>{riskLabel}</div>
          </div>
        </div>

        {/* Aliases */}
        {profile?.display_names && profile.display_names.length > 0 && (
          <div style={{ marginTop: 16, paddingTop: 16, borderTop: "1px solid var(--border)" }}>
            <div style={{ fontSize: 10, fontWeight: 600, color: "var(--text-3)", letterSpacing: "0.05em", marginBottom: 8 }}>KNOWN ALIASES</div>
            <div className="row gap-2" style={{ flexWrap: "wrap" }}>
              {profile.display_names.map((name, i) => (
                <span key={i} className="badge badge-gray" style={{ fontSize: 12 }}>{name}</span>
              ))}
            </div>
          </div>
        )}
      </div>

      {/* Stats */}
      <div className="kpi-grid mb-4">
        <div className="kpi">
          <div className="kpi-val" style={{ color: "var(--accent)" }}>{profile?.sent_count?.toLocaleString() || 0}</div>
          <div className="kpi-label">Sent</div>
        </div>
        <div className="kpi">
          <div className="kpi-val" style={{ color: "var(--success)" }}>{profile?.received_count?.toLocaleString() || 0}</div>
          <div className="kpi-label">Received</div>
        </div>
        <div className="kpi">
          <div className="kpi-val">{profile?.total_emails?.toLocaleString() || 0}</div>
          <div className="kpi-label">Total Involved</div>
        </div>
        <div className="kpi">
          <div className="kpi-val" style={{ fontSize: 18 }}>
            {profile?.first_seen ? new Date(profile.first_seen).toLocaleDateString() : "—"}
          </div>
          <div className="kpi-label">First Seen</div>
        </div>
        <div className="kpi">
          <div className="kpi-val" style={{ fontSize: 18 }}>
            {profile?.last_seen ? new Date(profile.last_seen).toLocaleDateString() : "—"}
          </div>
          <div className="kpi-label">Last Seen</div>
        </div>
      </div>

      {/* Correspondents & Subjects */}
      <div className="grid-2">
        <div className="card">
          <h3 style={{ fontSize: 14, fontWeight: 600, marginBottom: 16 }}>Top Correspondents</h3>
          {profile?.top_correspondents && profile.top_correspondents.length > 0 ? (
            <div style={{ display: "flex", flexDirection: "column", gap: 8 }}>
              {profile.top_correspondents.map(([email, count], i) => (
                <div key={i} className="row between" style={{ padding: "8px 12px", background: "var(--bg-3)", borderRadius: "var(--r-sm)" }}>
                  <div style={{ display: "flex", alignItems: "center", gap: 8 }}>
                    <span style={{ fontSize: 11, color: "var(--text-3)", width: 20 }}>#{i + 1}</span>
                    <span style={{ fontSize: 12, color: "var(--text-1)", fontFamily: "var(--mono)", maxWidth: 200, overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>{email}</span>
                  </div>
                  <span className="badge badge-blue">{count}</span>
                </div>
              ))}
            </div>
          ) : (
            <div className="empty" style={{ padding: 24 }}>No correspondents</div>
          )}
        </div>

        <div className="card">
          <h3 style={{ fontSize: 14, fontWeight: 600, marginBottom: 16 }}>Frequent Subjects</h3>
          {profile?.top_subjects && profile.top_subjects.length > 0 ? (
            <div style={{ display: "flex", flexDirection: "column", gap: 8 }}>
              {profile.top_subjects.map(([subject, count], i) => (
                <div key={i} className="row between" style={{ padding: "8px 12px", background: "var(--bg-3)", borderRadius: "var(--r-sm)" }}>
                  <div style={{ display: "flex", alignItems: "center", gap: 8 }}>
                    <span style={{ fontSize: 11, color: "var(--text-3)", width: 20 }}>#{i + 1}</span>
                    <span style={{ fontSize: 12, color: "var(--text-1)", maxWidth: 200, overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>{subject}</span>
                  </div>
                  <span className="badge badge-gray">{count}</span>
                </div>
              ))}
            </div>
          ) : (
            <div className="empty" style={{ padding: 24 }}>No subjects</div>
          )}
        </div>
      </div>
    </div>
  );
}
