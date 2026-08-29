import { EntityDetail, TabType } from "./types";

interface Props {
  selectedEntity: EntityDetail;
  activeTab: TabType;
  onTabSelect: (tab: TabType) => void;
}

export function EntityStatCards({
  selectedEntity,
  activeTab,
  onTabSelect,
}: Props) {
  return (
    <div
      style={{
        display: "grid",
        gridTemplateColumns: "repeat(5, 1fr)",
        gap: 8,
      }}
    >
      <div
        className="tr-click"
        style={{
          padding: "12px 14px",
          background: activeTab === "all" ? "var(--accent-subtle)" : "var(--bg-2)",
          border: activeTab === "all" ? "1px solid var(--accent)" : "1px solid var(--border)",
          borderRadius: "var(--r-md)",
          textAlign: "center",
        }}
        onClick={() => onTabSelect("all")}
      >
        <div style={{ fontSize: 18, fontWeight: 700, color: "var(--text-0)" }}>
          {selectedEntity.total_count}
        </div>
        <div style={{ fontSize: 10, color: "var(--text-3)", fontWeight: 600, marginTop: 2 }}>
          ALL MESSAGES
        </div>
      </div>

      <div
        className="tr-click"
        style={{
          padding: "12px 14px",
          background: activeTab === "sent" ? "rgba(59,130,246,0.15)" : "var(--bg-2)",
          border: activeTab === "sent" ? "1px solid #3b82f6" : "1px solid var(--border)",
          borderRadius: "var(--r-md)",
          textAlign: "center",
        }}
        onClick={() => onTabSelect("sent")}
      >
        <div style={{ fontSize: 18, fontWeight: 700, color: "#3b82f6" }}>
          {selectedEntity.sent_count}
        </div>
        <div style={{ fontSize: 10, color: "var(--text-3)", fontWeight: 600, marginTop: 2 }}>
          SENT BY THIS PERSON
        </div>
      </div>

      <div
        className="tr-click"
        style={{
          padding: "12px 14px",
          background: activeTab === "received" ? "rgba(34,197,94,0.15)" : "var(--bg-2)",
          border: activeTab === "received" ? "1px solid #22c55e" : "1px solid var(--border)",
          borderRadius: "var(--r-md)",
          textAlign: "center",
        }}
        onClick={() => onTabSelect("received")}
      >
        <div style={{ fontSize: 18, fontWeight: 700, color: "#22c55e" }}>
          {selectedEntity.received_count}
        </div>
        <div style={{ fontSize: 10, color: "var(--text-3)", fontWeight: 600, marginTop: 2 }}>
          RECEIVED (TO / CC)
        </div>
      </div>

      <div
        className="tr-click"
        style={{
          padding: "12px 14px",
          background: activeTab === "deleted" ? "rgba(239,68,68,0.15)" : "var(--bg-2)",
          border: activeTab === "deleted" ? "1px solid #ef4444" : "1px solid var(--border)",
          borderRadius: "var(--r-md)",
          textAlign: "center",
        }}
        onClick={() => onTabSelect("deleted")}
      >
        <div style={{ fontSize: 18, fontWeight: 700, color: "#ef4444" }}>
          {selectedEntity.deleted_count}
        </div>
        <div style={{ fontSize: 10, color: "var(--text-3)", fontWeight: 600, marginTop: 2 }}>
          DELETED / RECOVERED
        </div>
      </div>

      <div
        className="tr-click"
        style={{
          padding: "12px 14px",
          background: activeTab === "flagged" ? "rgba(234,179,8,0.15)" : "var(--bg-2)",
          border: activeTab === "flagged" ? "1px solid #eab308" : "1px solid var(--border)",
          borderRadius: "var(--r-md)",
          textAlign: "center",
        }}
        onClick={() => onTabSelect("flagged")}
      >
        <div style={{ fontSize: 18, fontWeight: 700, color: "#eab308" }}>
          {selectedEntity.flagged_count}
        </div>
        <div style={{ fontSize: 10, color: "var(--text-3)", fontWeight: 600, marginTop: 2 }}>
          FLAGGED / HIGH RISK
        </div>
      </div>
    </div>
  );
}
