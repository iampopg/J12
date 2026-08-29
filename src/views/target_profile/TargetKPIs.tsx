import { TargetProfile } from "./types";

interface Props {
  profile: TargetProfile | null;
}

export function TargetKPIs({ profile }: Props) {
  return (
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
  );
}
