import { Dashboard, View } from "../types";

interface Props {
  data: Dashboard;
  totalFindings: number;
  onNavigate: (view: View) => void;
}

export function DashboardKpis({ data, totalFindings, onNavigate }: Props) {
  const kpis = [
    {
      id: "emails",
      label: "Processed Emails",
      value: data.email_count.toLocaleString(),
      icon: "✉️",
      color: "#6366f1",
      bgGrad: "linear-gradient(135deg, rgba(99, 102, 241, 0.15) 0%, rgba(99, 102, 241, 0.03) 100%)",
      borderColor: "rgba(99, 102, 241, 0.3)",
      view: "search" as View,
      subtext: "Search full index",
    },
    {
      id: "entities",
      label: "Entities Discovered",
      value: (data.entity_count || 0).toLocaleString(),
      icon: "👥",
      color: "#0ea5e9",
      bgGrad: "linear-gradient(135deg, rgba(14, 165, 233, 0.15) 0%, rgba(14, 165, 233, 0.03) 100%)",
      borderColor: "rgba(14, 165, 233, 0.3)",
      view: "entities" as View,
      subtext: "Profile participants",
    },
    {
      id: "deleted",
      label: "Deleted Recovered",
      value: data.deleted_recovered.toLocaleString(),
      icon: "🗑️",
      color: "#f43f5e",
      bgGrad: "linear-gradient(135deg, rgba(244, 63, 94, 0.15) 0%, rgba(244, 63, 94, 0.03) 100%)",
      borderColor: "rgba(244, 63, 94, 0.3)",
      view: "soft_deleted" as View,
      subtext: "Carved & purged items",
    },
    {
      id: "findings",
      label: "Security Findings",
      value: totalFindings.toLocaleString(),
      icon: "🚨",
      color: totalFindings > 0 ? "#f59e0b" : "#10b981",
      bgGrad: totalFindings > 0
        ? "linear-gradient(135deg, rgba(245, 158, 11, 0.15) 0%, rgba(245, 158, 11, 0.03) 100%)"
        : "linear-gradient(135deg, rgba(16, 185, 129, 0.15) 0%, rgba(16, 185, 129, 0.03) 100%)",
      borderColor: totalFindings > 0 ? "rgba(245, 158, 11, 0.3)" : "rgba(16, 185, 129, 0.3)",
      view: "findings" as View,
      subtext: totalFindings > 0 ? "Inspect threat matrix" : "No open threats",
    },
    {
      id: "evidence",
      label: "Evidence Vault",
      value: data.evidence_count.toLocaleString(),
      icon: "📁",
      color: "#10b981",
      bgGrad: "linear-gradient(135deg, rgba(16, 185, 129, 0.15) 0%, rgba(16, 185, 129, 0.03) 100%)",
      borderColor: "rgba(16, 185, 129, 0.3)",
      view: "evidence" as View,
      subtext: "Container sources",
    },
  ];

  return (
    <div
      style={{
        display: "grid",
        gridTemplateColumns: "repeat(auto-fit, minmax(180px, 1fr))",
        gap: 12,
        marginBottom: 16,
      }}
    >
      {kpis.map((kpi) => (
        <div
          key={kpi.id}
          className="tr-click"
          onClick={() => onNavigate(kpi.view)}
          style={{
            background: kpi.bgGrad,
            border: `1px solid ${kpi.borderColor}`,
            borderRadius: "var(--r-md)",
            padding: "16px 14px",
            cursor: "pointer",
            transition: "all 0.2s cubic-bezier(0.16, 1, 0.3, 1)",
            position: "relative",
            overflow: "hidden",
          }}
          onMouseEnter={(e) => {
            e.currentTarget.style.transform = "translateY(-3px)";
            e.currentTarget.style.boxShadow = `0 8px 24px -4px ${kpi.color}33`;
          }}
          onMouseLeave={(e) => {
            e.currentTarget.style.transform = "translateY(0)";
            e.currentTarget.style.boxShadow = "none";
          }}
        >
          <div className="row between mb-2" style={{ alignItems: "center" }}>
            <span style={{ fontSize: 18 }}>{kpi.icon}</span>
            <span style={{ fontSize: 10, fontWeight: 700, color: kpi.color, textTransform: "uppercase" }}>
              Explore →
            </span>
          </div>

          <div
            style={{
              fontSize: 26,
              fontWeight: 800,
              color: kpi.color,
              lineHeight: 1.1,
              fontFamily: "var(--mono)",
              marginBottom: 4,
            }}
          >
            {kpi.value}
          </div>

          <div style={{ fontSize: 12, fontWeight: 600, color: "var(--text-1)" }}>
            {kpi.label}
          </div>

          <div style={{ fontSize: 10, color: "var(--text-3)", marginTop: 2 }}>
            {kpi.subtext}
          </div>
        </div>
      ))}
    </div>
  );
}
