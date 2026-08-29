import { Entity, EntityTier, cleanDisplayName } from "./types";

interface Props {
  entitiesCount: number;
  filteredEntities: Entity[];
  selectedEmail?: string;
  searchTerm: string;
  setSearchTerm: (s: string) => void;
  entityTier: EntityTier;
  setEntityTier: (t: EntityTier) => void;
  sortOption: "total" | "sent" | "received" | "name";
  setSortOption: (s: "total" | "sent" | "received" | "name") => void;
  onSelectEntity: (email: string) => void;
}

export function EntityDirectorySidebar({
  entitiesCount,
  filteredEntities,
  selectedEmail,
  searchTerm,
  setSearchTerm,
  entityTier,
  setEntityTier,
  sortOption,
  setSortOption,
  onSelectEntity,
}: Props) {
  return (
    <div
      className="card"
      style={{
        padding: 14,
        maxHeight: "82vh",
        display: "flex",
        flexDirection: "column",
        marginBottom: 0,
      }}
    >
      {/* Entity Tier Tabs */}
      <div
        style={{
          display: "grid",
          gridTemplateColumns: "1fr 1fr 1fr",
          gap: 4,
          background: "var(--bg-3)",
          padding: 3,
          borderRadius: "var(--r-sm)",
          marginBottom: 10,
        }}
      >
        <button
          type="button"
          style={{
            padding: "4px 6px",
            fontSize: 11,
            fontWeight: 600,
            border: "none",
            borderRadius: "var(--r-xs)",
            background: entityTier === "key" ? "var(--accent)" : "transparent",
            color: entityTier === "key" ? "#fff" : "var(--text-2)",
            cursor: "pointer",
          }}
          onClick={() => setEntityTier("key")}
        >
          Key People
        </button>
        <button
          type="button"
          style={{
            padding: "4px 6px",
            fontSize: 11,
            fontWeight: 600,
            border: "none",
            borderRadius: "var(--r-xs)",
            background: entityTier === "internal" ? "var(--accent)" : "transparent",
            color: entityTier === "internal" ? "#fff" : "var(--text-2)",
            cursor: "pointer",
          }}
          onClick={() => setEntityTier("internal")}
        >
          Internal Org
        </button>
        <button
          type="button"
          style={{
            padding: "4px 6px",
            fontSize: 11,
            fontWeight: 600,
            border: "none",
            borderRadius: "var(--r-xs)",
            background: entityTier === "all" ? "var(--accent)" : "transparent",
            color: entityTier === "all" ? "#fff" : "var(--text-2)",
            cursor: "pointer",
          }}
          onClick={() => setEntityTier("all")}
        >
          All ({entitiesCount})
        </button>
      </div>

      {/* Search and Sort */}
      <div className="mb-2">
        <input
          className="input mb-2"
          style={{ fontSize: 12, padding: "6px 10px" }}
          placeholder="Search name, email, alias..."
          value={searchTerm}
          onChange={(e) => setSearchTerm(e.target.value)}
        />
        <select
          className="select input"
          style={{ fontSize: 11, padding: "5px 8px" }}
          value={sortOption}
          onChange={(e) => setSortOption(e.target.value as any)}
        >
          <option value="total">Sort: Total Messages (High → Low)</option>
          <option value="sent">Sort: Sent Count (High → Low)</option>
          <option value="received">Sort: Received Count (High → Low)</option>
          <option value="name">Sort: Name (A → Z)</option>
        </select>
      </div>

      <div style={{ fontSize: 11, color: "var(--text-3)", marginBottom: 8, paddingLeft: 4 }}>
        Showing <strong>{filteredEntities.length}</strong> {entityTier === "key" ? "key participants" : "entities"}
      </div>

      {/* List */}
      <div
        style={{
          flex: 1,
          overflowY: "auto",
          display: "flex",
          flexDirection: "column",
          gap: 4,
          paddingRight: 4,
        }}
      >
        {filteredEntities.map((e) => {
          const isSelected = selectedEmail === e.email_address;
          const total = e.sent_count + e.received_count;
          const nameOrEmail = e.display_name || e.email_address || "?";
          const initial = (nameOrEmail.charAt(0) || "?").toUpperCase();

          return (
            <div
              key={e.id}
              className="tr-click"
              style={{
                padding: "9px 10px",
                borderRadius: "var(--r-md)",
                background: isSelected ? "var(--accent-subtle)" : "var(--bg-3)",
                border: isSelected ? "1px solid var(--accent)" : "1px solid transparent",
                display: "flex",
                alignItems: "center",
                gap: 10,
                transition: "all 0.15s",
              }}
              onClick={() => onSelectEntity(e.email_address)}
            >
              <div
                style={{
                  width: 32,
                  height: 32,
                  borderRadius: "50%",
                  background: isSelected
                    ? "var(--accent)"
                    : "linear-gradient(135deg, #3b82f6, #6366f1)",
                  display: "flex",
                  alignItems: "center",
                  justifyContent: "center",
                  fontSize: 13,
                  color: "#fff",
                  fontWeight: 700,
                  flexShrink: 0,
                }}
              >
                {initial}
              </div>

              <div style={{ flex: 1, minWidth: 0 }}>
                <div
                  style={{
                    fontSize: 12,
                    fontWeight: 600,
                    color: isSelected ? "var(--accent)" : "var(--text-0)",
                    overflow: "hidden",
                    textOverflow: "ellipsis",
                    whiteSpace: "nowrap",
                  }}
                >
                  {cleanDisplayName(e.display_name) || e.email_address}
                </div>
                <div
                  style={{
                    fontSize: 11,
                    color: "var(--text-3)",
                    fontFamily: "var(--mono)",
                    overflow: "hidden",
                    textOverflow: "ellipsis",
                    whiteSpace: "nowrap",
                  }}
                >
                  {e.email_address}
                </div>
              </div>

              <div style={{ textAlign: "right", flexShrink: 0 }}>
                <span
                  className="badge"
                  style={{
                    background: isSelected ? "var(--accent)" : "var(--bg-4)",
                    color: isSelected ? "#fff" : "var(--text-1)",
                    fontSize: 10,
                    fontWeight: 600,
                  }}
                >
                  {total.toLocaleString()}
                </span>
              </div>
            </div>
          );
        })}
      </div>
    </div>
  );
}
