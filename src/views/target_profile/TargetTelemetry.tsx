import { TargetProfile, formatDateSpan } from "./types";

interface Props {
  profile: TargetProfile | null;
  onSelectCandidate: (email: string) => void;
}

export function TargetTelemetry({ profile, onSelectCandidate }: Props) {
  return (
    <>
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
                  onClick={() => onSelectCandidate(email)}
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
    </>
  );
}
