import { useState, useEffect, useRef, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";

interface GraphNode {
  id: string;
  name: string | null;
  sent: number;
  received: number;
  total: number;
  x: number;
  y: number;
}

interface GraphEdge {
  source: string;
  target: string;
  weight: number;
}

interface Props {
  caseId: string;
}

export function GraphView({ caseId }: Props) {
  const [nodes, setNodes] = useState<GraphNode[]>([]);
  const [edges, setEdges] = useState<GraphEdge[]>([]);
  const [loading, setLoading] = useState(true);
  const [selectedNode, setSelectedNode] = useState<GraphNode | null>(null);
  const [filterMin, setFilterMin] = useState(10);
  const canvasRef = useRef<HTMLCanvasElement>(null);

  useEffect(() => { loadData(); }, [caseId]);

  useEffect(() => {
    if (nodes.length > 0) {
      const timer = setTimeout(() => layoutGraph(), 100);
      return () => clearTimeout(timer);
    }
  }, [nodes.length, filterMin]);

  useEffect(() => {
    drawGraph();
  }, [nodes, edges, selectedNode]);

  const loadData = async () => {
    setLoading(true);
    try {
      // Ensure entities exist
      const entCheck = await invoke<any>("entity_list", { input: { case_id: caseId } });
      if (!entCheck || entCheck.length === 0) {
        await invoke<number>("extract_entities", { caseId });
      }
      const res = await invoke<any>("graph_data", { input: { case_id: caseId } });
      const rawNodes: GraphNode[] = (res.nodes || []).map((n: any) => ({
        id: n.id,
        name: n.name,
        sent: n.sent || 0,
        received: n.received || 0,
        total: n.total || 0,
        x: 400 + (Math.random() - 0.5) * 300,
        y: 300 + (Math.random() - 0.5) * 200,
      }));
      setNodes(rawNodes);
      setEdges(res.edges || []);
    } catch (e) {
      console.error("Failed to load graph:", e);
    }
    setLoading(false);
  };

  const layoutGraph = useCallback(() => {
    setNodes(prevNodes => {
      const filtered = prevNodes.filter(n => n.total >= filterMin);
      if (filtered.length === 0) return prevNodes;

      // Simple circular layout as base
      const cx = 400, cy = 300;
      const radius = Math.min(300, filtered.length * 15);

      filtered.forEach((node, i) => {
        const angle = (2 * Math.PI * i) / filtered.length - Math.PI / 2;
        node.x = cx + radius * Math.cos(angle);
        node.y = cy + radius * Math.sin(angle);
      });

      // Run simple force simulation
      for (let iter = 0; iter < 50; iter++) {
        // Repulsion
        for (let i = 0; i < filtered.length; i++) {
          for (let j = i + 1; j < filtered.length; j++) {
            const a = filtered[i], b = filtered[j];
            let dx = a.x - b.x, dy = a.y - b.y;
            let dist = Math.sqrt(dx * dx + dy * dy) || 1;
            if (dist < 80) {
              const force = 500 / dist;
              dx /= dist; dy /= dist;
              a.x += dx * force; a.y += dy * force;
              b.x -= dx * force; b.y -= dy * force;
            }
          }
        }
        // Attraction along edges
        for (const edge of edges) {
          const s = filtered.find(n => n.id === edge.source);
          const t = filtered.find(n => n.id === edge.target);
          if (!s || !t) continue;
          const dx = t.x - s.x, dy = t.y - s.y;
          const dist = Math.sqrt(dx * dx + dy * dy) || 1;
          const force = dist * 0.01;
          s.x += (dx / dist) * force;
          s.y += (dy / dist) * force;
          t.x -= (dx / dist) * force;
          t.y -= (dy / dist) * force;
        }
        // Keep in bounds
        for (const n of filtered) {
          n.x = Math.max(50, Math.min(750, n.x));
          n.y = Math.max(50, Math.min(550, n.y));
        }
      }

      return [...filtered];
    });
  }, [edges, filterMin]);

  const drawGraph = useCallback(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    const ctx = canvas.getContext("2d");
    if (!ctx) return;

    const rect = canvas.getBoundingClientRect();
    canvas.width = rect.width * (window.devicePixelRatio || 1);
    canvas.height = rect.height * (window.devicePixelRatio || 1);
    ctx.scale(window.devicePixelRatio || 1, window.devicePixelRatio || 1);

    ctx.fillStyle = "#151a23";
    ctx.fillRect(0, 0, rect.width, rect.height);

    const scale = Math.min(rect.width / 800, rect.height / 600) * 0.85;
    const offsetX = rect.width / 2 - 400 * scale;
    const offsetY = rect.height / 2 - 300 * scale;

    // Draw edges
    for (const edge of edges) {
      const s = nodes.find(n => n.id === edge.source);
      const t = nodes.find(n => n.id === edge.target);
      if (!s || !t) continue;
      ctx.strokeStyle = "rgba(100,116,139,0.2)";
      ctx.lineWidth = Math.min(3, edge.weight / 10);
      ctx.beginPath();
      ctx.moveTo(offsetX + s.x * scale, offsetY + s.y * scale);
      ctx.lineTo(offsetX + t.x * scale, offsetY + t.y * scale);
      ctx.stroke();
    }

    // Draw nodes
    for (const node of nodes) {
      const radius = Math.max(8, Math.min(30, node.total / 20));
      const x = offsetX + node.x * scale;
      const y = offsetY + node.y * scale;

      ctx.beginPath();
      ctx.arc(x, y, radius, 0, Math.PI * 2);
      ctx.fillStyle = node === selectedNode ? "#fbbf24" : node.sent > node.received ? "#3b82f6" : "#22c55e";
      ctx.fill();
      ctx.strokeStyle = node === selectedNode ? "#fff" : "rgba(255,255,255,0.1)";
      ctx.lineWidth = node === selectedNode ? 3 : 1;
      ctx.stroke();

      // Label
      if (node.total > 15) {
        ctx.fillStyle = "#e2e8f0";
        ctx.font = "10px system-ui";
        ctx.textAlign = "center";
        const label = node.name || node.id.split("@")[0];
        ctx.fillText(label.slice(0, 12), x, y - radius - 5);
      }
    }

    // Legend
    ctx.fillStyle = "#64748b";
    ctx.font = "10px system-ui";
    ctx.textAlign = "left";
    ctx.fillText("● Blue = More sent  ● Green = More received  ● Yellow = Selected", 10, rect.height - 10);
  }, [nodes, edges, selectedNode]);

  const handleClick = (e: React.MouseEvent<HTMLCanvasElement>) => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    const rect = canvas.getBoundingClientRect();
    const scale = Math.min(rect.width / 800, rect.height / 600) * 0.85;
    const offsetX = rect.width / 2 - 400 * scale;
    const offsetY = rect.height / 2 - 300 * scale;
    const mx = e.clientX - rect.left;
    const my = e.clientY - rect.top;

    for (const node of nodes) {
      const x = offsetX + node.x * scale;
      const y = offsetY + node.y * scale;
      const radius = Math.max(8, Math.min(30, node.total / 20));
      const dx = mx - x, dy = my - y;
      if (dx * dx + dy * dy < (radius + 5) * (radius + 5)) {
        setSelectedNode(node);
        return;
      }
    }
    setSelectedNode(null);
  };

  if (loading) return <div className="empty">Loading graph...</div>;

  if (nodes.length === 0) {
    return (
      <div>
        <h2 style={{ fontSize: 22, fontWeight: 700, color: "var(--text-0)", marginBottom: 16 }}>Communication Graph</h2>
        <div className="card empty">No entities found. Upload and parse email data first.</div>
      </div>
    );
  }

  return (
    <div>
      <div className="row between mb-4">
        <div>
          <h2 style={{ fontSize: 22, fontWeight: 700, color: "var(--text-0)" }}>Communication Graph</h2>
          <p className="muted">{nodes.length} entities · {edges.length} connections</p>
        </div>
        <div className="row gap-2">
          <label className="row gap-2" style={{ fontSize: 12, color: "var(--text-2)" }}>
            Min emails:
            <input type="number" value={filterMin} onChange={e => setFilterMin(parseInt(e.target.value) || 1)} style={{ width: 60 }} className="input" />
          </label>
          <button className="btn btn-ghost btn-sm" onClick={loadData}>↻ Refresh</button>
        </div>
      </div>

      <div className="card mb-4" style={{ padding: 0, overflow: "hidden" }}>
        <canvas
          ref={canvasRef}
          style={{ width: "100%", height: 500, cursor: "pointer" }}
          onClick={handleClick}
        />
      </div>

      {selectedNode && (
        <div className="card" style={{ borderLeft: "4px solid var(--accent)" }}>
          <h3 style={{ fontSize: 15, fontWeight: 600, marginBottom: 12 }}>
            {selectedNode.name || selectedNode.id}
          </h3>
          <p style={{ fontSize: 13, color: "var(--accent)", fontFamily: "var(--mono)", marginBottom: 12 }}>
            {selectedNode.id}
          </p>
          <div className="row gap-4">
            <div style={{ flex: 1, padding: 12, background: "var(--bg-3)", borderRadius: "var(--r-sm)", textAlign: "center" }}>
              <div style={{ fontSize: 20, fontWeight: 700, color: "#3b82f6" }}>{selectedNode.sent}</div>
              <div style={{ fontSize: 10, color: "var(--text-3)" }}>Sent</div>
            </div>
            <div style={{ flex: 1, padding: 12, background: "var(--bg-3)", borderRadius: "var(--r-sm)", textAlign: "center" }}>
              <div style={{ fontSize: 20, fontWeight: 700, color: "#22c55e" }}>{selectedNode.received}</div>
              <div style={{ fontSize: 10, color: "var(--text-3)" }}>Received</div>
            </div>
            <div style={{ flex: 1, padding: 12, background: "var(--bg-3)", borderRadius: "var(--r-sm)", textAlign: "center" }}>
              <div style={{ fontSize: 20, fontWeight: 700, color: "var(--text-0)" }}>{selectedNode.total}</div>
              <div style={{ fontSize: 10, color: "var(--text-3)" }}>Total</div>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
