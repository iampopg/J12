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

interface Props {
  caseId: string;
  caseData: any;
}

export function TargetProfileView({ caseId, caseData }: Props) {
  const [profile, setProfile] = useState<TargetProfile | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    loadProfile();
  }, [caseId]);

  const loadProfile = async () => {
    setLoading(true);
    try {
      const data = await invoke<TargetProfile>("target_profile", { caseId });
      setProfile(data);
    } catch (e) {
      console.error("Failed to load target profile:", e);
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
        <h2 style={{ fontSize: 22, fontWeight: 700, color: "var(--text-0)", marginBottom: 16 }}>Target Profile</h2>
        <div className="card" style={{ textAlign: "center", padding: "60px 40px" }}>
          <div style={{ fontSize: 48, marginBottom: 16 }}>👤</div>
          <h3 style={{ fontSize: 18, marginBottom: 8, color: "var(--text-0)" }}>No Target Defined</h3>
          <p className="muted">Edit this case to add a target email address and name for investigation profiling.</p>
        </div>
      </div>
    );
  }

  const riskColor = (profile?.risk_score || 0) >= 50 ? "var(--danger)" : (profile?.risk_score || 0) >= 25 ? "var(--warning)" : "var(--success)";
  const riskLabel = (profile?.risk_score || 0) >= 50 ? "HIGH RISK" : (profile?.risk_score || 0) >= 25 ? "MEDIUM RISK" : "LOW RISK";

  return (
    <div>
      <div className="row between mb-4">
        <div>
          <h2 style={{ fontSize: 22, fontWeight: 700, color: "var(--text-0)" }}>Target Profile</h2>
          <p className="muted">Subject investigation overview — identity, communications, and risk assessment</p>
        </div>
        <button className="btn btn-ghost btn-sm" onClick={loadProfile}>↻ Refresh</button>
      </div>

      {/* Identity Card */}
      <div className="card mb-4" style={{ borderLeft: "4px solid var(--accent)" }}>
        <div className="row between" style={{ marginBottom: 16 }}>
          <div style={{ display: "flex", alignItems: "center", gap: 16 }}>
            <div style={{ width: 64, height: 64, borderRadius: "50%", background: "var(--bg-4)", display: "flex", alignItems: "center", justifyContent: "center", fontSize: 28 }}>
              {targetName ? targetName.charAt(0).toUpperCase() : "?"}
            </div>
            <div>
              <h3 style={{ fontSize: 20, fontWeight: 700, color: "var(--text-0)" }}>{targetName || "Unknown"}</h3>
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

        {/* Display Names / Aliases */}
        {profile?.display_names && profile.display_names.length > 0 && (
          <div style={{ marginTop: 16, paddingTop: 16, borderTop: "1px solid var(--border)" }}>
            <div style={{ fontSize: 10, fontWeight: 600, color: "var(--text-3)", letterSpacing: "0.05em", marginBottom: 8 }}>KNOWN ALIASES / DISPLAY NAMES</div>
            <div className="row gap-2" style={{ flexWrap: "wrap" }}>
              {profile.display_names.map((name, i) => (
                <span key={i} className="badge badge-gray" style={{ fontSize: 12 }}>{name}</span>
              ))}
            </div>
          </div>
        )}
      </div>

      {/* Communication Stats */}
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

      {/* Top Correspondents & Subjects */}
      <div className="grid-2">
        {/* Top Correspondents */}
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
            <div className="empty" style={{ padding: 24 }}>No correspondents found</div>
          )}
        </div>

        {/* Top Subjects */}
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
            <div className="empty" style={{ padding: 24 }}>No subjects found</div>
          )}
        </div>
      </div>

      {/* Communication Activity */}
      <div className="card mt-4">
        <h3 style={{ fontSize: 14, fontWeight: 600, marginBottom: 16 }}>Activity Summary</h3>
        <div className="grid-2" style={{ fontSize: 13 }}>
          <div>
            <span className="muted">Email Address:</span>
            <p style={{ fontFamily: "var(--mono)", color: "var(--accent)", marginTop: 4 }}>{targetEmail || "—"}</p>
          </div>
          <div>
            <span className="muted">Full Name:</span>
            <p style={{ color: "var(--text-0)", marginTop: 4 }}>{targetName || "—"}</p>
          </div>
          <div>
            <span className="muted">Organization:</span>
            <p style={{ color: "var(--text-0)", marginTop: 4 }}>{targetOrg || "—"}</p>
          </div>
          <div>
            <span className="muted">Investigation:</span>
            <p style={{ color: "var(--text-0)", marginTop: 4 }}>{caseData?.case_title || "—"}</p>
          </div>
        </div>
      </div>
    </div>
  );
}
