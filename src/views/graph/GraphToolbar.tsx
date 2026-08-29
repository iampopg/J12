import { GraphNode } from "./types";

interface Props {
  layoutMode: "force" | "radial";
  setLayoutMode: (m: "force" | "radial") => void;
  maxNodes: number;
  setMaxNodes: (n: number) => void;
  minEmails: number;
  setMinEmails: (n: number) => void;
  searchTerm: string;
  setSearchTerm: (s: string) => void;
  activeNodes: GraphNode[];
  onSelectNode: (node: GraphNode) => void;
}

export function GraphToolbar({
  layoutMode,
  setLayoutMode,
  maxNodes,
  setMaxNodes,
  minEmails,
  setMinEmails,
  searchTerm,
  setSearchTerm,
  activeNodes,
  onSelectNode,
}: Props) {
  return (
    <div
      className="card mb-3"
      style={{
        padding: "10px 14px",
        display: "flex",
        alignItems: "center",
        gap: 16,
        flexWrap: "wrap",
      }}
    >
      <div style={{ display: "flex", alignItems: "center", gap: 8 }}>
        <span style={{ fontSize: 11, fontWeight: 600, color: "var(--text-2)" }}>Layout:</span>
        <div className="row gap-1">
          <button
            className={`btn btn-sm ${layoutMode === "force" ? "btn-primary" : "btn-ghost"}`}
            style={{ fontSize: 11, padding: "3px 8px" }}
            onClick={() => setLayoutMode("force")}
          >
            🕸️ Organic Force
          </button>
          <button
            className={`btn btn-sm ${layoutMode === "radial" ? "btn-primary" : "btn-ghost"}`}
            style={{ fontSize: 11, padding: "3px 8px" }}
            onClick={() => setLayoutMode("radial")}
          >
            🎯 Target Concentric
          </button>
        </div>
      </div>

      <div style={{ display: "flex", alignItems: "center", gap: 8 }}>
        <span style={{ fontSize: 11, fontWeight: 600, color: "var(--text-2)" }}>Density:</span>
        <select
          className="select input"
          style={{ fontSize: 11, padding: "4px 8px" }}
          value={maxNodes}
          onChange={(e) => setMaxNodes(Number(e.target.value))}
        >
          <option value={15}>Top 15 Key Actors</option>
          <option value={35}>Top 35 Active Entities</option>
          <option value={60}>Top 60 Entities</option>
          <option value={100}>Top 100 Entities</option>
        </select>
      </div>

      <div style={{ display: "flex", alignItems: "center", gap: 8 }}>
        <span style={{ fontSize: 11, fontWeight: 600, color: "var(--text-2)" }}>Min Volume:</span>
        <input
          type="number"
          className="input"
          style={{ width: 55, fontSize: 11, padding: "3px 6px" }}
          value={minEmails}
          onChange={(e) => setMinEmails(Math.max(1, Number(e.target.value) || 1))}
        />
      </div>

      <div style={{ flex: 1, minWidth: 180 }}>
        <input
          className="input"
          style={{ fontSize: 11, padding: "5px 8px", width: "100%" }}
          placeholder="Search person in network..."
          value={searchTerm}
          onChange={(e) => {
            setSearchTerm(e.target.value);
            const found = activeNodes.find(
              (n) =>
                n.id.toLowerCase().includes(e.target.value.toLowerCase()) ||
                (n.name || "").toLowerCase().includes(e.target.value.toLowerCase())
            );
            if (found) {
              onSelectNode(found);
            }
          }}
        />
      </div>
    </div>
  );
}
