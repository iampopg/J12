import { TargetProfile, DetectedTarget } from "./types";

interface Props {
  profile: TargetProfile | null;
  detected: DetectedTarget[];
  caseData?: any;
  currentEmail: string;
  currentName: string;
  currentOrg: string;
  selectedEmail: string | null;
}

export function TargetBanner({
  profile,
  detected,
  caseData,
  currentEmail,
  currentName,
  currentOrg,
  selectedEmail,
}: Props) {
  const riskScore = profile?.risk_score || 0;
  const riskColor = riskScore >= 50 ? "var(--danger)" : riskScore >= 25 ? "var(--warning)" : "var(--success)";
  const riskLabel = riskScore >= 50 ? "HIGH RISK" : riskScore >= 25 ? "MODERATE" : "LOW RISK";

  return (
    <div className="card mb-4" style={{ borderLeft: "4px solid var(--accent)", boxShadow: "0 4px 20px rgba(0,0,0,0.15)", padding: 24 }}>
      <div className="row between" style={{ alignItems: "flex-start", flexWrap: "wrap", gap: 16 }}>
        <div style={{ display: "flex", alignItems: "center", gap: 20 }}>
          <div style={{
            width: 80,
            height: 80,
            borderRadius: "50%",
            background: profile?.is_automated ? "linear-gradient(135deg, #64748b, #475569)" : "linear-gradient(135deg, #2563eb, #7c3aed)",
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
              <span className={`badge ${selectedEmail === caseData?.target_email || profile?.role?.includes("Custodian") || detected.find(d => d.email === currentEmail)?.is_primary_target ? "badge-blue" : profile?.is_automated ? "badge-gray" : "badge-purple"}`} style={{ fontSize: 11 }}>
                {selectedEmail === caseData?.target_email || profile?.role?.includes("Custodian") || detected.find(d => d.email === currentEmail)?.is_primary_target
                  ? "🎯 PRIMARY CUSTODIAN / TARGET"
                  : profile?.is_automated
                  ? "🤖 AUTOMATED SERVICE / BOT"
                  : profile?.role || "👤 PERSON OF INTEREST"}
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
  );
}
