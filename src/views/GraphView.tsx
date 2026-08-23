import { useState, useEffect, useRef, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";

interface GraphNode {
  id: string;
  name: string | null;
  sent: number;
  received: number;
  total: number;
  x?: number;
  y?: number;
  vx?: number;
  vy?: number;
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
  const [filterMin, setFilterMin] = useState(5);
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const animRef = useRef<number>(0);
  const dragRef = useRef<{ node: GraphNode; offsetX: number; offsetY: number } | null>(null);

  useEffect(() => {
    loadData();
  }, [caseId]);

  useEffect(() => {
    if (nodes.length > 0) {
      runSimulation();
    }
  }, [nodes, edges, filterMin]);

  useEffect(() => {
    drawGraph();
  }, [nodes, edges, selectedNode]);

  const loadData = async () => {
    setLoading(true);
    try {
      const res = await invoke<any>("graph_data", { input: { case_id: caseId } });
      setNodes((res.nodes || []).map((n: any) => ({ ...n, x: Math.random() * 600, y: Math.random() * 400 })));
      setEdges(res.edges || []);
    } catch (e) {
      console.error("Failed to load graph:", e);
    }
    setLoading(false);
  };

  const runSimulation = useCallback(() => {
    const filteredNodes = nodes.filter(n => n.total >= filterMin);
    const nodeIds = new Set(filteredNodes.map(n => n.id));
    const filteredEdges = edges.filter(e => nodeIds.has(e.source) && nodeIds.has(e.target));

    // Simple force-directed layout
    const iterations = 100;
    const repulsion = 5000;
    const attraction = 0.01;
    const damping = 0.9;

    for (let iter = 0; iter < iterations; iter++) {
      // Repulsion between all nodes
      for (let i = 0; i < filteredNodes.length; i++) {
        for (let j = i + 1; j < filteredNodes.length; j++) {
          const a = filteredNodes[i];
          const b = filteredNodes[j];
          const dx = (a.x || 0) - (b.x || 0);
          const dy = (a.y || 0) - (b.y || 0);
          const dist = Math.sqrt(dx * dx + dy * dy) || 1;
          const force = repulsion / (dist * dist);
          const fx = (dx / dist) * force;
          const fy = (dy / dist) * force;
          a.vx = (a.vx || 0) + fx;
          a.vy = (a.vy || 0) + fy;
          b.vx = (b.vx || 0) - fx;
          b.vy = (b.vy || 0) - fy;
        }
      }

      // Attraction along edges
      for (const edge of filteredEdges) {
        const source = filteredNodes.find(n => n.id === edge.source);
        const target = filteredNodes.find(n => n.id === edge.target);
        if (!source || !target) continue;
        const dx = (target.x || 0) - (source.x || 0);
        const dy = (target.y || 0) - (source.y || 0);
        const dist = Math.sqrt(dx * dx + dy * dy) || 1;
        const force = dist * attraction * edge.weight;
        const fx = (dx / dist) * force;
        const fy = (dy / dist) * force;
        source.vx = (source.vx || 0) + fx;
        source.vy = (source.vy || 0) + fy;
        target.vx = (target.vx || 0) - fx;
        target.vy = (target.vy || 0) - fy;
      }

      // Apply velocities
      for (const node of filteredNodes) {
        node.vx = (node.vx || 0) * damping;
        node.vy = (node.vy || 0) * damping;
        node.x = (node.x || 0) + (node.vx || 0);
        node.y = (node.y || 0) + (node.vy || 0);
        // Keep in bounds
        node.x = Math.max(50, Math.min(750, node.x || 0));
        node.y = Math.max(50, Math.min(550, node.y || 0));
      }
    }

    setNodes([...filteredNodes]);
  }, [nodes, edges, filterMin]);

  const drawGraph = useCallback(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;

    const ctx = canvas.getContext("2d");
    if (!ctx) return;

    const dpr = window.devicePixelRatio || 1;
    const rect = canvas.getBoundingClientRect();
    canvas.width = rect.width * dpr;
    canvas.height = rect.height * dpr;
    ctx.scale(dpr, dpr);

    const width = rect.width;
    const height = rect.height;

    // Clear
    ctx.fillStyle = "#151a23";
    ctx.fillRect(0, 0, width, height);

    // Center the graph
    const centerX = width / 2;
    const centerY = height / 2;
    const scale = Math.min(width / 800, height / 600) * 0.8;

    ctx.save();
    ctx.translate(centerX, centerY);
    ctx.scale(scale, scale);

    // Draw edges
    for (const edge of edges) {
      const source = nodes.find(n => n.id === edge.source);
      const target = nodes.find(n => n.id === edge.target);
      if (!source || !target) continue;

      ctx.strokeStyle = "rgba(100, 116, 139, 0.3)";
      ctx.lineWidth = Math.min(3, edge.weight / 5);
      ctx.beginPath();
      ctx.moveTo(source.x || 0, source.y || 0);
      ctx.lineTo(target.x || 0, target.y || 0);
      ctx.stroke();
    }

    // Draw nodes
    for (const node of nodes) {
      const radius = Math.max(5, Math.min(25, node.total / 10));
      const isSelected = selectedNode?.id === node.id;

      // Node circle
      ctx.beginPath();
      ctx.arc(node.x || 0, node.y || 0, radius, 0, Math.PI * 2);
      ctx.fillStyle = isSelected ? "#fbbf24" : node.sent > node.received ? "#3b82f6" : "#22c55e";
      ctx.fill();

      if (isSelected) {
        ctx.strokeStyle = "#fff";
        ctx.lineWidth = 3;
        ctx.stroke();
      }

      // Label
      if (node.total > 10 || isSelected) {
        ctx.fillStyle = "#e2e8f0";
        ctx.font = `${Math.max(10, radius)}px system-ui`;
        ctx.textAlign = "center";
        const label = node.name || node.id.split("@")[0];
        ctx.fillText(label.slice(0, 15), node.x || 0, node.y || 0 - radius - 5);
      }
    }

    ctx.restore();

    // Legend
    ctx.fillStyle = "#64748b";
    ctx.font = "11px system-ui";
    ctx.textAlign = "left";
    ctx.fillText("● Blue = More sent  ● Green = More received  ● Yellow = Selected", 15, height - 15);
    ctx.fillText("Drag nodes to rearrange · Scroll to filter by activity", 15, height - 30);

  }, [nodes, edges, selectedNode]);

  const handleCanvasClick = (e: React.MouseEvent<HTMLCanvasElement>) => {
    const canvas = canvasRef.current;
    if (!canvas) return;

    const rect = canvas.getBoundingClientRect();
    const x = (e.clientX - rect.left - rect.width / 2) / (Math.min(rect.width / 800, rect.height / 600) * 0.8);
    const y = (e.clientY - rect.top - rect.height / 2) / (Math.min(rect.width / 800, rect.height / 600) * 0.8);

    // Find clicked node
    for (const node of nodes) {
      const dx = (node.x || 0) - x;
      const dy = (node.y || 0) - y;
      const radius = Math.max(5, Math.min(25, node.total / 10));
      if (dx * dx + dy * dy < radius * radius * 4) {
        setSelectedNode(node);
        dragRef.current = { node, offsetX: dx, offsetY: dy };
        return;
      }
    }
    setSelectedNode(null);
  };

  const handleMouseMove = (e: React.MouseEvent<HTMLCanvasElement>) => {
    if (!dragRef.current) return;
    const canvas = canvasRef.current;
    if (!canvas) return;

    const rect = canvas.getBoundingClientRect();
    const scale = Math.min(rect.width / 800, rect.height / 600) * 0.8;
    const x = (e.clientX - rect.left - rect.width / 2) / scale - dragRef.current.offsetX;
    const y = (e.clientY - rect.top - rect.height / 2) / scale - dragRef.current.offsetY;

    dragRef.current.node.x = x;
    dragRef.current.node.y = y;
    drawGraph();
  };

  const handleMouseUp = () => {
    dragRef.current = null;
  };

  if (loading) return <div className="empty">Loading communication graph...</div>;

  if (nodes.length === 0) {
    return (
      <div>
        <h2 style={{ fontSize: 22, fontWeight: 700, color: "var(--text-0)", marginBottom: 16 }}>Communication Graph</h2>
        <div className="card empty">No entities found. Run entity extraction first.</div>
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
            <input
              type="number"
              value={filterMin}
              onChange={(e) => setFilterMin(parseInt(e.target.value) || 1)}
              style={{ width: 60 }}
              className="input input-sm"
            />
          </label>
          <button className="btn btn-ghost btn-sm" onClick={loadData}>↻ Refresh</button>
        </div>
      </div>

      {/* Graph Canvas */}
      <div className="card mb-4" style={{ padding: 0, overflow: "hidden" }}>
        <canvas
          ref={canvasRef}
          style={{ width: "100%", height: 500, cursor: dragRef.current ? "grabbing" : "pointer" }}
          onClick={handleCanvasClick}
          onMouseMove={handleMouseMove}
          onMouseUp={handleMouseUp}
          onMouseLeave={handleMouseUp}
        />
      </div>

      {/* Selected Node Details */}
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
