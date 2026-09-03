import { Dashboard, View } from "../types";

interface Props {
  data: Dashboard;
  totalFindings: number;
  onNavigate: (view: View) => void;
}

export function DashboardFolderTally({ data, totalFindings, onNavigate }: Props) {
  const folders = [
    { label: "Inbox", count: data.inbox_count, color: "#3b82f6", view: "inbox" as View, icon: "📥" },
    { label: "Sent Items", count: data.sent_count, color: "#10b981", view: "sent" as View, icon: "📤" },
    { label: "Deleted / Trash", count: data.soft_deleted_count, color: "#f43f5e", view: "soft_deleted" as View, icon: "🗑️" },
    { label: "Drafts", count: data.drafts_count, color: "#a855f7", view: "drafts" as View, icon: "📝" },
    { label: "Spam / Junk", count: data.spam_count, color: "#f59e0b", view: "spam" as View, icon: "🚫" },
    { label: "Other Folders", count: data.other_count, color: "#64748b", view: "other" as View, icon: "📁" },
  ];

  const severityData = [
    { label: "Critical", value: data.severity_breakdown?.critical || 0, color: "#ef4444" },
    { label: "High", value: data.severity_breakdown?.high || 0, color: "#f97316" },
    { label: "Medium", value: data.severity_breakdown?.medium || 0, color: "#eab308" },
    { label: "Low", value: data.severity_breakdown?.low || 0, color: "#22c55e" },
  ];
  const maxSeverity = Math.max(...severityData.map((s) => s.value), 1);

  return (
    <div className="mb-4">
      {/* Folder Taxonomy Grid */}
      <div className="card mb-3" style={{ padding: 18 }}>
        <div className="row between mb-3" style={{ alignItems: "center" }}>
          <div>
            <h3 style={{ fontSize: 14, fontWeight: 700, margin: 0, color: "var(--text-0)" }}>
              📂 Mailbox Folder Distribution
            </h3>
            <span className="muted" style={{ fontSize: 11 }}>
              Click any folder card to jump directly to its virtual message list
            </span>
          </div>
          <span className="badge badge-blue" style={{ fontSize: 10 }}>
            {data.email_count.toLocaleString()} TOTAL EMAILS
          </span>
        </div>

        <div
          style={{
            display: "grid",
            gridTemplateColumns: "repeat(auto-fit, minmax(130px, 1fr))",
            gap: 10,
          }}
        >
          {folders.map((folder) => (
            <div
              key={folder.label}
              className="tr-click"
              style={{
                padding: "12px 10px",
                background: "var(--bg-3)",
                borderRadius: "var(--r-sm)",
                textAlign: "center",
                cursor: "pointer",
                border: "1px solid var(--border)",
                transition: "all 0.15s ease",
              }}
              onClick={() => onNavigate(folder.view)}
              onMouseEnter={(e) => {
                e.currentTarget.style.borderColor = folder.color;
                e.currentTarget.style.background = `${folder.color}15`;
              }}
              onMouseLeave={(e) => {
                e.currentTarget.style.borderColor = "var(--border)";
                e.currentTarget.style.background = "var(--bg-3)";
              }}
            >
              <div style={{ fontSize: 14, marginBottom: 2 }}>{folder.icon}</div>
              <div style={{ fontSize: 18, fontWeight: 800, color: folder.color, fontFamily: "var(--mono)" }}>
                {folder.count.toLocaleString()}
              </div>
              <div style={{ fontSize: 11, fontWeight: 600, color: "var(--text-2)", marginTop: 2 }}>
                {folder.label}
              </div>
            </div>
          ))}
        </div>
      </div>

      {/* Threat Severity Breakdown Bar */}
      {totalFindings > 0 && (
        <div className="card" style={{ padding: 18 }}>
          <h3 style={{ fontSize: 14, fontWeight: 700, marginBottom: 12, color: "var(--text-0)" }}>
            ⚡ Threat Severity Spectrum
          </h3>
          <div style={{ display: "flex", flexDirection: "column", gap: 10 }}>
            {severityData.map((sev) => (
              <div key={sev.label} className="row gap-3" style={{ alignItems: "center" }}>
                <span style={{ width: 75, fontSize: 11, color: sev.color, fontWeight: 700 }}>
                  {sev.label}
                </span>
                <div
                  style={{
                    flex: 1,
                    height: 14,
                    background: "var(--bg-3)",
                    borderRadius: 4,
                    overflow: "hidden",
                    border: "1px solid var(--border)",
                  }}
                >
                  <div
                    style={{
                      width: `${(sev.value / maxSeverity) * 100}%`,
                      height: "100%",
                      background: sev.color,
                      borderRadius: 4,
                      opacity: 0.85,
                      transition: "width 0.4s ease-out",
                    }}
                  />
                </div>
                <span
                  style={{
                    width: 45,
                    textAlign: "right",
                    fontSize: 12,
                    fontWeight: 700,
                    fontFamily: "var(--mono)",
                    color: "var(--text-1)",
                  }}
                >
                  {sev.value}
                </span>
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  );
}
