import { EntityDetail, cleanDisplayName } from "./types";

interface Props {
  selectedEntity: EntityDetail;
  onSetAsTarget: () => void;
  settingTarget: boolean;
}

export function EntityProfileHeader({
  selectedEntity,
  onSetAsTarget,
  settingTarget,
}: Props) {
  return (
    <div
      className="card mb-0"
      style={{
        padding: 20,
        borderLeft: "4px solid var(--accent)",
        background: "var(--bg-2)",
      }}
    >
      <div className="row between" style={{ alignItems: "flex-start" }}>
        <div style={{ display: "flex", alignItems: "center", gap: 16 }}>
          <div
            style={{
              width: 56,
              height: 56,
              borderRadius: "50%",
              background: "linear-gradient(135deg, #3b82f6, #6366f1)",
              display: "flex",
              alignItems: "center",
              justifyContent: "center",
              fontSize: 24,
              color: "#fff",
              fontWeight: 700,
              boxShadow: "0 4px 12px rgba(59,130,246,0.3)",
            }}
          >
            {((selectedEntity.display_name || selectedEntity.email || "?").charAt(0) || "?")
              .toUpperCase()}
          </div>
          <div>
            <h3 style={{ fontSize: 20, fontWeight: 700, color: "var(--text-0)" }}>
              {cleanDisplayName(selectedEntity.display_name) || selectedEntity.email || "Unknown Entity"}
            </h3>
            <p
              style={{
                fontSize: 13,
                color: "var(--accent)",
                fontFamily: "var(--mono)",
                marginBottom: 4,
              }}
            >
              {selectedEntity.email}
            </p>

            {/* Merged Aliases List */}
            {selectedEntity.aliases && selectedEntity.aliases.length > 0 && (
              <div className="row gap-1 mb-2" style={{ flexWrap: "wrap" }}>
                <span style={{ fontSize: 10, color: "var(--text-3)", fontWeight: 600 }}>
                  🔗 Unified Aliases:
                </span>
                {selectedEntity.aliases.map((alias) => (
                  <span
                    key={alias}
                    className="badge"
                    style={{
                      fontSize: 10,
                      fontFamily: "var(--mono)",
                      background: "var(--bg-4)",
                      color: "var(--text-2)",
                    }}
                  >
                    {alias}
                  </span>
                ))}
              </div>
            )}

            <div className="row gap-3" style={{ fontSize: 11, color: "var(--text-3)" }}>
              <span>
                📅 First Seen:{" "}
                <strong>
                  {selectedEntity.first_seen
                    ? new Date(selectedEntity.first_seen).toLocaleDateString()
                    : "—"}
                </strong>
              </span>
              <span>·</span>
              <span>
                📅 Last Seen:{" "}
                <strong>
                  {selectedEntity.last_seen
                    ? new Date(selectedEntity.last_seen).toLocaleDateString()
                    : "—"}
                </strong>
              </span>
            </div>
          </div>
        </div>

        <div className="row gap-2">
          <button
            className="btn btn-primary btn-sm"
            onClick={onSetAsTarget}
            disabled={settingTarget}
            title="Set this person as the primary target for this case"
          >
            🎯 {settingTarget ? "Setting..." : "Set as Target Profile"}
          </button>
        </div>
      </div>
    </div>
  );
}
