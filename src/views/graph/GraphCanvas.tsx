import { useRef, useEffect, useCallback } from "react";
import { GraphNode, GraphEdge, cleanDisplayName } from "./types";

interface Props {
  activeNodes: GraphNode[];
  activeEdges: GraphEdge[];
  selectedNode: GraphNode | null;
  selectedEdge: GraphEdge | null;
  targetEmail: string | null;
  layoutMode: "force" | "radial";
  zoom: number;
  setZoom: React.Dispatch<React.SetStateAction<number>>;
  pan: { x: number; y: number };
  setPan: React.Dispatch<React.SetStateAction<{ x: number; y: number }>>;
  onSelectNode: (node: GraphNode) => void;
  onClearEdge: () => void;
}

export function GraphCanvas({
  activeNodes,
  activeEdges,
  selectedNode,
  selectedEdge,
  targetEmail,
  layoutMode,
  zoom,
  setZoom,
  pan,
  setPan,
  onSelectNode,
  onClearEdge,
}: Props) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const isDragging = useRef(false);
  const dragStart = useRef({ x: 0, y: 0 });
  const draggedNode = useRef<GraphNode | null>(null);
  const animFrameId = useRef<number | null>(null);

  // Physics Simulation Step
  useEffect(() => {
    if (activeNodes.length === 0) return;

    if (layoutMode === "radial") {
      const target = activeNodes.find((n) => n.is_target) || activeNodes[0];
      const others = activeNodes.filter((n) => n.id !== target.id);

      target.x = 0;
      target.y = 0;

      const numRings = Math.ceil(others.length / 10);
      others.forEach((node, i) => {
        const ring = Math.floor(i / 10) + 1;
        const indexInRing = i % 10;
        const totalInRing = Math.min(10, others.length - (ring - 1) * 10);
        const radius = ring * 160;
        const angle = (2 * Math.PI * indexInRing) / totalInRing;
        node.x = Math.cos(angle) * radius;
        node.y = Math.sin(angle) * radius;
      });
      drawGraph();
      return;
    }

    let iterations = 0;
    const runPhysics = () => {
      iterations++;
      for (let i = 0; i < activeNodes.length; i++) {
        for (let j = i + 1; j < activeNodes.length; j++) {
          const a = activeNodes[i];
          const b = activeNodes[j];
          let dx = b.x - a.x;
          let dy = b.y - a.y;
          let dist = Math.sqrt(dx * dx + dy * dy) || 1;
          if (dist < 260) {
            const force = (260 - dist) / dist * 0.8;
            if (draggedNode.current !== a) {
              a.x -= dx * force * 0.05;
              a.y -= dy * force * 0.05;
            }
            if (draggedNode.current !== b) {
              b.x += dx * force * 0.05;
              b.y += dy * force * 0.05;
            }
          }
        }
      }

      for (const edge of activeEdges) {
        const s = activeNodes.find((n) => n.id === edge.source);
        const t = activeNodes.find((n) => n.id === edge.target);
        if (!s || !t) continue;
        let dx = t.x - s.x;
        let dy = t.y - s.y;
        let dist = Math.sqrt(dx * dx + dy * dy) || 1;
        const idealDist = Math.max(90, 200 - Math.min(80, edge.weight * 2));
        const force = (dist - idealDist) * 0.008;

        if (draggedNode.current !== s) {
          s.x += (dx / dist) * force;
          s.y += (dy / dist) * force;
        }
        if (draggedNode.current !== t) {
          t.x -= (dx / dist) * force;
          t.y -= (dy / dist) * force;
        }
      }

      for (const node of activeNodes) {
        if (draggedNode.current !== node) {
          node.x *= 0.98;
          node.y *= 0.98;
        }
      }

      drawGraph();
      if (iterations < 120 || draggedNode.current !== null) {
        animFrameId.current = requestAnimationFrame(runPhysics);
      }
    };

    runPhysics();
    return () => {
      if (animFrameId.current) cancelAnimationFrame(animFrameId.current);
    };
  }, [activeNodes, activeEdges, layoutMode]);

  const drawGraph = useCallback(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    const ctx = canvas.getContext("2d");
    if (!ctx) return;

    const rect = canvas.getBoundingClientRect();
    const dpr = window.devicePixelRatio || 1;
    canvas.width = rect.width * dpr;
    canvas.height = rect.height * dpr;
    ctx.scale(dpr, dpr);

    ctx.fillStyle = "#0c1017";
    ctx.fillRect(0, 0, rect.width, rect.height);

    ctx.strokeStyle = "rgba(255, 255, 255, 0.03)";
    ctx.lineWidth = 1;
    const gridSize = 40 * zoom;
    const offsetX = (rect.width / 2 + pan.x) % gridSize;
    const offsetY = (rect.height / 2 + pan.y) % gridSize;
    for (let x = offsetX; x < rect.width; x += gridSize) {
      ctx.beginPath();
      ctx.moveTo(x, 0);
      ctx.lineTo(x, rect.height);
      ctx.stroke();
    }
    for (let y = offsetY; y < rect.height; y += gridSize) {
      ctx.beginPath();
      ctx.moveTo(0, y);
      ctx.lineTo(rect.width, y);
      ctx.stroke();
    }

    ctx.save();
    ctx.translate(rect.width / 2 + pan.x, rect.height / 2 + pan.y);
    ctx.scale(zoom, zoom);

    // Draw Edges
    for (const edge of activeEdges) {
      const s = activeNodes.find((n) => n.id === edge.source);
      const t = activeNodes.find((n) => n.id === edge.target);
      if (!s || !t) continue;

      const isEdgeSelected =
        selectedEdge &&
        ((selectedEdge.source === edge.source && selectedEdge.target === edge.target) ||
          (selectedEdge.source === edge.target && selectedEdge.target === edge.source));

      const isConnectedToSelected =
        selectedNode && (s.id === selectedNode.id || t.id === selectedNode.id);

      ctx.beginPath();
      ctx.moveTo(s.x, s.y);
      ctx.lineTo(t.x, t.y);

      if (isEdgeSelected) {
        ctx.strokeStyle = "#38bdf8";
        ctx.lineWidth = 3.5;
      } else if (isConnectedToSelected) {
        ctx.strokeStyle = "rgba(59, 130, 246, 0.7)";
        ctx.lineWidth = Math.max(1.5, Math.min(4, edge.weight / 15));
      } else {
        ctx.strokeStyle = "rgba(100, 116, 139, 0.18)";
        ctx.lineWidth = Math.max(1, Math.min(2.5, edge.weight / 25));
      }
      ctx.stroke();

      if (isConnectedToSelected || isEdgeSelected || edge.weight > 30) {
        const midX = (s.x + t.x) / 2;
        const midY = (s.y + t.y) / 2;
        ctx.fillStyle = "rgba(15, 23, 42, 0.85)";
        ctx.beginPath();
        ctx.arc(midX, midY, 9, 0, Math.PI * 2);
        ctx.fill();
        ctx.fillStyle = "#94a3b8";
        ctx.font = "9px system-ui";
        ctx.textAlign = "center";
        ctx.textBaseline = "middle";
        ctx.fillText(String(edge.weight), midX, midY);
      }
    }

    // Draw Nodes
    for (const node of activeNodes) {
      const isSelected = selectedNode?.id === node.id;
      const isTarget = node.is_target || node.id === targetEmail;
      const radius = isTarget ? 24 : Math.max(14, Math.min(28, 12 + Math.log2(node.total + 1) * 2.5));

      if (isSelected || isTarget) {
        ctx.shadowColor = isTarget ? "#f59e0b" : "#3b82f6";
        ctx.shadowBlur = 18;
      } else {
        ctx.shadowBlur = 0;
      }

      ctx.beginPath();
      ctx.arc(node.x, node.y, radius, 0, Math.PI * 2);

      if (isTarget) {
        ctx.fillStyle = "#f59e0b";
      } else if (isSelected) {
        ctx.fillStyle = "#3b82f6";
      } else if (node.sent > node.received * 1.5) {
        ctx.fillStyle = "#2563eb";
      } else if (node.received > node.sent * 1.5) {
        ctx.fillStyle = "#10b981";
      } else {
        ctx.fillStyle = "#6366f1";
      }
      ctx.fill();
      ctx.shadowBlur = 0;

      ctx.strokeStyle = isSelected ? "#ffffff" : isTarget ? "#fde68a" : "rgba(255,255,255,0.2)";
      ctx.lineWidth = isSelected ? 3 : isTarget ? 2.5 : 1.5;
      ctx.stroke();

      ctx.fillStyle = "#ffffff";
      ctx.font = `bold ${Math.round(radius * 0.75)}px system-ui`;
      ctx.textAlign = "center";
      ctx.textBaseline = "middle";
      const initial = (node.name || node.id).charAt(0).toUpperCase();
      ctx.fillText(initial, node.x, node.y);

      const displayName = cleanDisplayName(node.name) || node.id.split("@")[0];
      ctx.font = isSelected || isTarget ? "bold 11px system-ui" : "10px system-ui";
      ctx.fillStyle = isSelected ? "#38bdf8" : isTarget ? "#fbbf24" : "#e2e8f0";
      ctx.fillText(displayName, node.x, node.y + radius + 12);
    }

    ctx.restore();
  }, [activeNodes, activeEdges, selectedNode, selectedEdge, targetEmail, zoom, pan]);

  const handleMouseDown = (e: React.MouseEvent<HTMLCanvasElement>) => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    const rect = canvas.getBoundingClientRect();
    const mx = e.clientX - rect.left;
    const my = e.clientY - rect.top;

    const gx = (mx - (rect.width / 2 + pan.x)) / zoom;
    const gy = (my - (rect.height / 2 + pan.y)) / zoom;

    for (const node of activeNodes) {
      const radius = node.is_target ? 24 : Math.max(14, Math.min(28, 12 + Math.log2(node.total + 1) * 2.5));
      const dx = gx - node.x;
      const dy = gy - node.y;
      if (dx * dx + dy * dy <= (radius + 4) * (radius + 4)) {
        draggedNode.current = node;
        onSelectNode(node);
        onClearEdge();
        return;
      }
    }

    isDragging.current = true;
    dragStart.current = { x: e.clientX - pan.x, y: e.clientY - pan.y };
  };

  const handleMouseMove = (e: React.MouseEvent<HTMLCanvasElement>) => {
    const canvas = canvasRef.current;
    if (!canvas) return;

    if (draggedNode.current) {
      const rect = canvas.getBoundingClientRect();
      const mx = e.clientX - rect.left;
      const my = e.clientY - rect.top;
      draggedNode.current.x = (mx - (rect.width / 2 + pan.x)) / zoom;
      draggedNode.current.y = (my - (rect.height / 2 + pan.y)) / zoom;
      drawGraph();
    } else if (isDragging.current) {
      setPan({
        x: e.clientX - dragStart.current.x,
        y: e.clientY - dragStart.current.y,
      });
    }
  };

  const handleMouseUp = () => {
    isDragging.current = false;
    draggedNode.current = null;
  };

  const handleWheel = (e: React.WheelEvent<HTMLCanvasElement>) => {
    e.preventDefault();
    const zoomFactor = e.deltaY < 0 ? 1.15 : 0.88;
    setZoom((prev) => Math.max(0.3, Math.min(3.5, prev * zoomFactor)));
  };

  return (
    <div
      className="card mb-0"
      style={{
        padding: 0,
        overflow: "hidden",
        borderRadius: "var(--r-md)",
        border: "1px solid var(--border)",
        position: "relative",
        height: "72vh",
        background: "#0c1017",
      }}
    >
      <canvas
        ref={canvasRef}
        style={{ width: "100%", height: "100%", cursor: isDragging.current ? "grabbing" : "grab" }}
        onMouseDown={handleMouseDown}
        onMouseMove={handleMouseMove}
        onMouseUp={handleMouseUp}
        onWheel={handleWheel}
      />

      {/* Canvas Overlay Legend */}
      <div
        style={{
          position: "absolute",
          bottom: 12,
          left: 12,
          background: "rgba(15, 23, 42, 0.85)",
          backdropFilter: "blur(6px)",
          padding: "6px 12px",
          borderRadius: "var(--r-sm)",
          border: "1px solid rgba(255,255,255,0.08)",
          fontSize: 10,
          display: "flex",
          gap: 12,
          color: "var(--text-2)",
        }}
      >
        <span>🟡 <strong>Target Person</strong></span>
        <span>🔵 <strong>Active Sender</strong></span>
        <span>🟢 <strong>Active Receiver</strong></span>
        <span>🟣 <strong>Balanced Hub</strong></span>
      </div>

      <div
        style={{
          position: "absolute",
          top: 12,
          right: 12,
          background: "rgba(15, 23, 42, 0.85)",
          padding: "4px 8px",
          borderRadius: "var(--r-sm)",
          fontSize: 10,
          color: "var(--text-3)",
        }}
      >
        Zoom: {Math.round(zoom * 100)}%
      </div>
    </div>
  );
}
