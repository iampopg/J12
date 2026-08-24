import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";

interface Entity {
  id: string;
  email_address: string;
  display_name: string | null;
  sent_count: number;
  received_count: number;
}

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

export function TargetProfileView({ caseId }: Props) {
  const [profile, setProfile] = useState<TargetProfile | null>(null);
  const [detected, setDetected] = useState<DetectedTarget[]>([]);
  const [totalEntities, setTotalEntities] = useState<number>(0);
  const [loading, setLoading] = useState(true);
  const [selectedEmail, setSelectedEmail] = useState<string | null>(null);
  const [showAll, setShowAll] = useState(false);
  const [toastMessage, setToastMessage] = useState<string | null>(null);

  const showToast = (msg: string) => {
    setToastMessage(msg);
    setTimeout(() => setToastMessage(null), 3500);
  };

  useEffect(() => { loadData(); }, [caseId]);

  const loadData = async () => {
    setLoading(true);
    try {
      // First ensure entities exist
      const existing = await invoke<Entity[]>("entity_list", { input: { case_id: caseId } });
      if (existing.length === 0) {
        await invoke<number>("extract_entities", { input: { case_id: caseId } });
      }
      const det = await invoke<any>("auto_detect_targets", { input: { case_id: caseId } });
      const targets: DetectedTarget[] = det.targets || [];
      setDetected(targets);
      setTotalEntities(det.total_case_entities || targets.length);

      if (targets.length > 0 && !selectedEmail) {
        const top = targets[0];
        setSelectedEmail(top.email);
        loadProfile(top.email);
      } else if (selectedEmail) {
        loadProfile(selectedEmail);
      }
    } catch (e) { 
      console.error(e); 
    }
    setLoading(false);
  };

  const reExtract = async () => {
    setLoading(true);
    try {
      const count = await invoke<number>("extract_entities", { input: { case_id: caseId } });
      showToast(`⚡ Re-extracted and resolved ${count} entities across case`);
      await loadData();
    } catch (e) { 
      console.error(e); 
    } finally { 
      setLoading(false); 
    }
  };

  const loadProfile = async (email: string) => {
    try {
      const prof = await invoke<TargetProfile>("target_profile", { input: { case_id: caseId, target_email: email } });
      setProfile(prof);
    } catch (e) { 
      console.error(e); 
    }
  };

  if (loading) return <div className="empty">Loading target profile...</div>;

  if (detected.length === 0) {
    return (
      <div>
        <h2 style={{ fontSize: 22, fontWeight: 700, color: "var(--text-0)", marginBottom: 16 }}>Target Profile</h2>
        <div className="card empty">No targets detected. Upload and parse email data first.</div>
      </div>
    );
  }

  // Helper to format clean display name
  const formatName = (d: string | null, email: string) => {
    if (d && d.trim() && d !== email && !d.startsWith('/')) {
      if (d.includes("..")) {
        const p = d.split("..");
        if (p.length === 2) return `${p[0].toUpperCase()}. ${p[1].charAt(0).toUpperCase() + p[1].slice(1)}`;
      }
      return d;
    }
    const local = email.split('@')[0] || email;
    if (local.includes("..")) {
      const p = local.split("..");
      if (p.length === 2) return `${p[0].toUpperCase()}. ${p[1].charAt(0).toUpperCase() + p[1].slice(1)}`;
    } else if (local.includes('.')) {
      return local.split('.').map(w => w.charAt(0).toUpperCase() + w.slice(1)).join(' ');
    }
    return local.charAt(0).toUpperCase() + local.slice(1);
  };

  // Get the selected target's data
  const selected = detected.find(t => t.email === selectedEmail) || detected[0];
  const targetDisplayName = formatName(selected.display_name, selected.email);
  const riskColor = (profile?.risk_score || 0) >= 50 ? "var(--danger)" : (profile?.risk_score || 0) >= 25 ? "var(--warning)" : "var(--success)";
  const riskLabel = (profile?.risk_score || 0) >= 50 ? "HIGH RISK" : (profile?.risk_score || 0) >= 25 ? "MEDIUM RISK" : "LOW RISK";

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

      <div className="row between mb-4">
        <div>
          <h2 style={{ fontSize: 22, fontWeight: 700, color: "var(--text-0)" }}>Target Profile &amp; Subject Dossier</h2>
          <p className="muted">Main investigation subject — <strong>{targetDisplayName}</strong> ({selected.email})</p>
        </div>
        <div className="row gap-2">
          <button className="btn btn-ghost btn-sm" onClick={reExtract} title="Re-scan and clean all entities and aliases">⚡ Re-Extract &amp; Unify</button>
          <button className="btn btn-ghost btn-sm" onClick={loadData}>↻ Refresh</button>
        </div>
      </div>

      {/* Main Identity Card */}
      <div className="card mb-4" style={{ borderLeft: "4px solid var(--accent)", boxShadow: "0 4px 20px rgba(0,0,0,0.15)" }}>
        <div className="row between" style={{ marginBottom: 16 }}>
          <div style={{ display: "flex", alignItems: "center", gap: 16 }}>
            <div style={{ width: 72, height: 72, borderRadius: "50%", background: "linear-gradient(135deg, #3b82f6, #6366f1)", display: "flex", alignItems: "center", justifyContent: "center", fontSize: 32, color: "#fff", fontWeight: 700 }}>
              {targetDisplayName.charAt(0).toUpperCase()}
            </div>
            <div>
              <h3 style={{ fontSize: 22, fontWeight: 700, color: "var(--text-0)", margin: 0 }}>{targetDisplayName}</h3>
              <p style={{ fontSize: 14, color: "var(--accent)", fontFamily: "var(--mono)", marginTop: 4, marginBottom: 2 }}>{selected.email}</p>
              <p style={{ fontSize: 12, color: "var(--text-3)", margin: 0 }}>
                Involved in <strong>{selected.total_emails.toLocaleString()}</strong> case emails ({selected.sent.toLocaleString()} sent, {selected.received.toLocaleString()} received)
              </p>
            </div>
          </div>
          <div style={{ textAlign: "right" }}>
            <div style={{ fontSize: 10, fontWeight: 600, color: "var(--text-3)", letterSpacing: "0.05em", marginBottom: 4 }}>RISK SCORE</div>
            <div style={{ fontSize: 36, fontWeight: 800, color: riskColor }}>{profile?.risk_score || 0}</div>
            <div style={{ fontSize: 11, color: riskColor, fontWeight: 600 }}>{riskLabel}</div>
          </div>
        </div>
      </div>

      {/* Stats KPI Grid */}
      <div className="kpi-grid mb-4">
        <div className="kpi">
          <div className="kpi-val" style={{ color: "var(--accent)" }}>{selected.sent.toLocaleString()}</div>
          <div className="kpi-label">Emails Sent</div>
        </div>
        <div className="kpi">
          <div className="kpi-val" style={{ color: "var(--success)" }}>{selected.received.toLocaleString()}</div>
          <div className="kpi-label">Emails Received</div>
        </div>
        <div className="kpi">
          <div className="kpi-val">{selected.total_emails.toLocaleString()}</div>
          <div className="kpi-label">Total Involvement</div>
        </div>
        <div className="kpi">
          <div className="kpi-val" style={{ color: "#38bdf8" }}>{totalEntities.toLocaleString()}</div>
          <div className="kpi-label">Total Entities in Case</div>
        </div>
      </div>

      {/* Other Detected Targets / High-Activity Persons */}
      {detected.length > 1 && (
        <div className="card mb-4">
          <div className="row between mb-4">
            <div>
              <h4 style={{ fontSize: 13, fontWeight: 700, color: "var(--text-0)" }}>
                Top Candidate Persons of Interest ({detected.length - 1} Candidates)
              </h4>
              <p className="muted text-sm" style={{ margin: 0 }}>
                Click any subject below to switch and inspect their complete correspondent dossier:
              </p>
            </div>
            <button className="btn btn-ghost btn-sm" onClick={() => setShowAll(!showAll)}>
              {showAll ? "▲ Collapse" : `▼ View All (${detected.length - 1})`}
            </button>
          </div>
          <div style={{ display: "grid", gridTemplateColumns: "repeat(auto-fill, minmax(220px, 1fr))", gap: 8 }}>
            {(showAll ? detected.filter(t => t.email !== selected.email) : detected.filter(t => t.email !== selected.email).slice(0, 6)).map((t, i) => {
              const name = formatName(t.display_name, t.email);
              return (
                <div 
                  key={i} 
                  className="row between" 
                  style={{ 
                    padding: "8px 12px", 
                    background: "var(--bg-3)", 
                    borderRadius: "var(--r-sm)", 
                    cursor: "pointer",
                    border: "1px solid var(--border)",
                    transition: "all 0.15s ease"
                  }} 
                  onClick={() => { setSelectedEmail(t.email); loadProfile(t.email); }}
                >
                  <div style={{ overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap", paddingRight: 8 }}>
                    <div style={{ fontSize: 12, fontWeight: 600, color: "var(--text-0)" }}>{name}</div>
                    <div style={{ fontSize: 10, color: "var(--text-3)", fontFamily: "var(--mono)" }}>{t.email}</div>
                  </div>
                  <span className="badge badge-gray" style={{ fontSize: 11 }}>{t.total_emails}</span>
                </div>
              );
            })}
          </div>
        </div>
      )}

      {/* Profile details */}
      {profile && (
        <div className="grid-2">
          <div className="card">
            <h3 style={{ fontSize: 14, fontWeight: 700, marginBottom: 12, color: "var(--text-0)" }}>
              👥 Top Direct Correspondents Network
            </h3>
            {profile.top_correspondents?.length > 0 ? profile.top_correspondents.map(([email, count], i) => (
              <div key={i} className="row between" style={{ padding: "8px 0", borderBottom: "1px solid var(--border)" }}>
                <span style={{ fontSize: 12, fontFamily: "var(--mono)", color: "var(--text-1)" }}>{email}</span>
                <span className="badge badge-blue">{count} msgs</span>
              </div>
            )) : <div className="muted text-sm">No direct correspondent data available.</div>}
          </div>

          <div className="card">
            <h3 style={{ fontSize: 14, fontWeight: 700, marginBottom: 12, color: "var(--text-0)" }}>
              ✉️ Frequent Investigation Subjects &amp; Topics
            </h3>
            {profile.top_subjects?.length > 0 ? profile.top_subjects.map(([subject, count], i) => (
              <div key={i} className="row between" style={{ padding: "8px 0", borderBottom: "1px solid var(--border)" }}>
                <span style={{ fontSize: 12, color: "var(--text-1)", overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap", maxWidth: "80%" }}>
                  {subject}
                </span>
                <span className="badge badge-gray">{count}</span>
              </div>
            )) : <div className="muted text-sm">No frequent subject topics recorded.</div>}
          </div>
        </div>
      )}
    </div>
  );
}
