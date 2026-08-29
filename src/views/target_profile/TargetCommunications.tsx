import { TargetProfile, formatDateSpan } from "./types";

interface Props {
  profile: TargetProfile | null;
  onSelectEmail?: (emailId: string) => void;
}

export function TargetCommunications({ profile, onSelectEmail }: Props) {
  if (!profile?.recent_communications || profile.recent_communications.length === 0) {
    return null;
  }

  return (
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
  );
}
