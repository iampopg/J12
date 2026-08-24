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

export function TargetProfileView({ caseId, caseData }: Props) {
  const [profile, setProfile] = useState<TargetProfile | null>(null);
  const [detected, setDetected] = useState<DetectedTarget[]>([]);
  const [loading, setLoading] = useState(true);
  const [selectedEmail, setSelectedEmail] = useState<string | null>(null);
  const [showAll, setShowAll] = useState(false);

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
      if (targets.length > 0 && !selectedEmail) {
        const top = targets[0];
        setSelectedEmail(top.email);
        loadProfile(top.email);
      } else if (selectedEmail) {
        loadProfile(selectedEmail);
      }
    } catch (e) { console.error(e); }
    setLoading(false);
  };

  const reExtract = async () => {
    setLoading(true);
    try {
      await invoke<number>("extract_entities", { input: { case_id: caseId } });
      await loadData();
    } catch (e) { console.error(e); }
    finally { setLoading(false); }
  };

  const loadProfile = async (email: string) => {
    try {
      const prof = await invoke<TargetProfile>("target_profile", { input: { case_id: caseId, target_email: email } });
      setProfile(prof);
    } catch (e) { console.error(e); }
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

  // Get the selected target's data
  const selected = detected.find(t => t.email === selectedEmail) || detected[0];
  const riskColor = (profile?.risk_score || 0) >= 50 ? "var(--danger)" : (profile?.risk_score || 0) >= 25 ? "var(--warning)" : "var(--success)";
  const riskLabel = (profile?.risk_score || 0) >= 50 ? "HIGH RISK" : (profile?.risk_score || 0) >= 25 ? "MEDIUM RISK" : "LOW RISK";

  return (
    <div>
      <div className="row between mb-4">
        <div>
          <h2 style={{ fontSize: 22, fontWeight: 700, color: "var(--text-0)" }}>Target Profile</h2>
          <p className="muted">Main investigation subject — {selected.display_name || selected.email}</p>
        </div>
        <div className="row gap-2">
          <button className="btn btn-ghost btn-sm" onClick={reExtract} title="Re-scan and clean all entities and aliases">⚡ Re-Extract & Clean</button>
          <button className="btn btn-ghost btn-sm" onClick={loadData}>↻ Refresh</button>
        </div>
      </div>

      {/* Main Identity Card */}
      <div className="card mb-4" style={{ borderLeft: "4px solid var(--accent)" }}>
        <div className="row between" style={{ marginBottom: 16 }}>
          <div style={{ display: "flex", alignItems: "center", gap: 16 }}>
            <div style={{ width: 72, height: 72, borderRadius: "50%", background: "linear-gradient(135deg, #3b82f6, #6366f1)", display: "flex", alignItems: "center", justifyContent: "center", fontSize: 32, color: "#fff", fontWeight: 700 }}>
              {(selected.display_name || selected.email).charAt(0).toUpperCase()}
            </div>
            <div>
              <h3 style={{ fontSize: 22, fontWeight: 700 }}>{selected.display_name || selected.email}</h3>
              <p style={{ fontSize: 14, color: "var(--accent)", fontFamily: "var(--mono)" }}>{selected.email}</p>
              <p style={{ fontSize: 12, color: "var(--text-3)" }}>Appears in {selected.total_emails} emails</p>
            </div>
          </div>
          <div style={{ textAlign: "right" }}>
            <div style={{ fontSize: 10, fontWeight: 600, color: "var(--text-3)", letterSpacing: "0.05em", marginBottom: 4 }}>RISK SCORE</div>
            <div style={{ fontSize: 36, fontWeight: 800, color: riskColor }}>{profile?.risk_score || 0}</div>
            <div style={{ fontSize: 11, color: riskColor, fontWeight: 600 }}>{riskLabel}</div>
          </div>
        </div>
      </div>

      {/* Stats */}
      <div className="kpi-grid mb-4">
        <div className="kpi"><div className="kpi-val" style={{ color: "var(--accent)" }}>{selected.sent}</div><div className="kpi-label">Sent</div></div>
        <div className="kpi"><div className="kpi-val" style={{ color: "var(--success)" }}>{selected.received}</div><div className="kpi-label">Received</div></div>
        <div className="kpi"><div className="kpi-val">{selected.total_emails}</div><div className="kpi-label">Total Involved</div></div>
        <div className="kpi"><div className="kpi-val" style={{ fontSize: 16 }}>{detected.length}</div><div className="kpi-label">People in Case</div></div>
      </div>

      {/* Other detected targets */}
      {detected.length > 1 && (
        <div className="card mb-4">
          <div className="row between mb-4">
            <h4 style={{ fontSize: 13, fontWeight: 600 }}>Other People in Case ({detected.length - 1})</h4>
            <button className="btn btn-ghost btn-sm" onClick={() => setShowAll(!showAll)}>{showAll ? "Hide" : "Show All"}</button>
          </div>
          {showAll && (
            <div style={{ display: "grid", gridTemplateColumns: "repeat(auto-fill, minmax(200px, 1fr))", gap: 8 }}>
              {detected.filter(t => t.email !== selected.email).map((t, i) => (
                <div key={i} className="row between" style={{ padding: "8px 12px", background: "var(--bg-3)", borderRadius: "var(--r-sm)", cursor: "pointer" }} onClick={() => { setSelectedEmail(t.email); loadProfile(t.email); }}>
                  <span style={{ fontSize: 12, fontFamily: "var(--mono)" }}>{t.display_name || t.email}</span>
                  <span className="badge badge-gray">{t.total_emails}</span>
                </div>
              ))}
            </div>
          )}
        </div>
      )}

      {/* Profile details */}
      {profile && (
        <div className="grid-2">
          <div className="card">
            <h3 style={{ fontSize: 14, fontWeight: 600, marginBottom: 12 }}>Top Correspondents</h3>
            {profile.top_correspondents?.length > 0 ? profile.top_correspondents.map(([email, count], i) => (
              <div key={i} className="row between" style={{ padding: "6px 0", borderBottom: "1px solid var(--border)" }}>
                <span style={{ fontSize: 12, fontFamily: "var(--mono)" }}>{email}</span>
                <span className="badge badge-blue">{count}</span>
              </div>
            )) : <div className="muted text-sm">No data</div>}
          </div>
          <div className="card">
            <h3 style={{ fontSize: 14, fontWeight: 600, marginBottom: 12 }}>Frequent Subjects</h3>
            {profile.top_subjects?.length > 0 ? profile.top_subjects.map(([subject, count], i) => (
              <div key={i} className="row between" style={{ padding: "6px 0", borderBottom: "1px solid var(--border)" }}>
                <span style={{ fontSize: 12 }}>{subject}</span>
                <span className="badge badge-gray">{count}</span>
              </div>
            )) : <div className="muted text-sm">No data</div>}
          </div>
        </div>
      )}
    </div>
  );
}
