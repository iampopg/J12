import { useState, useEffect, useRef, useCallback, useMemo } from "react";
import { invoke } from "@tauri-apps/api/core";

function cleanDisplayName(name: string | null): string {
  if (!name) return "";
  let cleaned = name
    .replace(/@ENRON.*$/g, "")
    .replace(/IMCEANOTES-[^<]*/g, "")
    .replace(/<[^>]*>/g, "")
    .replace(/"/g, "")
    .replace(/\s+/g, " ")
    .trim();
  if (cleaned.includes("@")) {
    return cleaned.split("@")[0].trim() || cleaned;
  }
  return cleaned;
}

interface GraphNode {
  id: string;
  name: string | null;
  sent: number;
  received: number;
  total: number;
  is_target?: boolean;
  x: number;
  y: number;
  vx: number;
  vy: number;
}

interface GraphEdge {
  source: string;
  target: string;
  weight: number;
}

interface ExchangedEmail {
  id: string;
  from_addr: string;
  from_display: string | null;
  to_addrs: string;
  subject: string | null;
  date_sent_utc: string;
  risk_score: number;
  body_text: string | null;
}

interface Props {
  caseId: string;
  evidenceFilter?: string | null;
}

export function GraphView({ caseId, evidenceFilter }: Props) {
  const [nodes, setNodes] = useState<GraphNode[]>([]);
  const [edges, setEdges] = useState<GraphEdge[]>([]);
  const [targetEmail, setTargetEmail] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);

  // Interaction states
  const [selectedNode, setSelectedNode] = useState<GraphNode | null>(null);
  const [selectedEdge, setSelectedEdge] = useState<GraphEdge | null>(null);
  const [searchTerm, setSearchTerm] = useState("");
  const [maxNodes, setMaxNodes] = useState<number>(35);
  const [minEmails, setMinEmails] = useState<number>(5);
  const [layoutMode, setLayoutMode] = useState<"force" | "radial">("force");

  // Inspector email list states
  const [inspectorEmails, setInspectorEmails] = useState<ExchangedEmail[]>([]);
  const [loadingEmails, setLoadingEmails] = useState(false);
  const [selectedEmail, setSelectedEmail] = useState<ExchangedEmail | null>(null);

  // Canvas Pan & Zoom
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const [zoom, setZoom] = useState(1);
  const [pan, setPan] = useState({ x: 0, y: 0 });
  const isDragging = useRef(false);
  const dragStart = useRef({ x: 0, y: 0 });
  const draggedNode = useRef<GraphNode | null>(null);
  const animFrameId = useRef<number | null>(null);

  useEffect(() => {
    loadData();
  }, [caseId, evidenceFilter]);

  const loadData = async () => {
    setLoading(true);
    try {
      const res = await invoke<any>("graph_data", { 
        input: { 
          case_id: caseId,
          evidence_id: evidenceFilter || undefined
        } 
      });
      const rawNodes: GraphNode[] = (res.nodes || []).map((n: any) => ({
        id: n.id,
        name: n.name,
        sent: n.sent || 0,
        received: n.received || 0,
        total: n.total || 0,
        is_target: n.is_target || false,
        x: (Math.random() - 0.5) * 500,
        y: (Math.random() - 0.5) * 400,
        vx: 0,
        vy: 0,
      }));

      setNodes(rawNodes);
      setEdges(res.edges || []);
      setTargetEmail(res.target_email || null);

      if (rawNodes.length > 0) {
        const initialTarget = rawNodes.find((n) => n.is_target) || rawNodes[0];
        setSelectedNode(initialTarget);
        loadEmailsForEntity(initialTarget.id);
      }
    } catch (e) {
      console.error("Failed to load graph:", e);
    } finally {
      setLoading(false);
    }
  };

  // Load emails when inspecting an entity
  const loadEmailsForEntity = async (email: string) => {
    setLoadingEmails(true);
    setSelectedEmail(null);
    try {
      const res = await invoke<ExchangedEmail[]>("entity_emails", {
        input: {
          case_id: caseId,
          email,
          filter_type: "all",
          partner_email: "",
          q: "",
          date_from: "",
          date_to: "",
          has_attachment: false,
        },
      });
      setInspectorEmails(res || []);
    } catch (e) {
      console.error(e);
      setInspectorEmails([]);
    } finally {
      setLoadingEmails(false);
    }
  };

  // Load emails between two entities
  const loadEmailsBetween = async (from: string, to: string) => {
    setLoadingEmails(true);
    setSelectedEmail(null);
    try {
      const res = await invoke<ExchangedEmail[]>("emails_between", {
        input: { case_id: caseId, from, to },
      });
      setInspectorEmails(res || []);
    } catch (e) {
      console.error(e);
      setInspectorEmails([]);
    } finally {
      setLoadingEmails(false);
    }
  };

  // Filter nodes according to density and minEmails
  const activeNodes = useMemo(() => {
    let filtered = nodes.filter((n) => n.total >= minEmails);
    filtered.sort((a, b) => b.total - a.total);
    return filtered.slice(0, maxNodes);
  }, [nodes, minEmails, maxNodes]);

  const activeNodeIds = useMemo(() => {
    return new Set(activeNodes.map((n) => n.id));
  }, [activeNodes]);

  const activeEdges = useMemo(() => {
    return edges.filter(
      (e) => activeNodeIds.has(e.source) && activeNodeIds.has(e.target)
    );
  }, [edges, activeNodeIds]);

  // Connected partners for currently selected node
  const connectedPartners = useMemo(() => {
    if (!selectedNode) return [];
    const partners: { id: string; name: string; count: number }[] = [];
    activeEdges.forEach((edge) => {
      if (edge.source === selectedNode.id) {
        const partner = nodes.find((n) => n.id === edge.target);
        if (partner) {
          partners.push({
            id: partner.id,
            name: cleanDisplayName(partner.name) || partner.id,
            count: edge.weight,
          });
        }
      } else if (edge.target === selectedNode.id) {
        const partner = nodes.find((n) => n.id === edge.source);
        if (partner) {
          partners.push({
            id: partner.id,
            name: cleanDisplayName(partner.name) || partner.id,
            count: edge.weight,
          });
        }
      }
    });
    partners.sort((a, b) => b.count - a.count);
    return partners;
  }, [selectedNode, activeEdges, nodes]);

  // Physics Simulation Step
  useEffect(() => {
    if (activeNodes.length === 0) return;

    if (layoutMode === "radial") {
      // Position target at center and others in concentric orbits
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
      // Repulsion between all nodes
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

      // Spring attraction along edges
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

      // Center gravity
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

  // Main Canvas Rendering
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

    // Dark background
    ctx.fillStyle = "#0c1017";
    ctx.fillRect(0, 0, rect.width, rect.height);

    // Draw Subtle Grid
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
    // Center camera
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

      // Edge weight tag if connected or selected
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

      // Glow effect for target or selected
      if (isSelected || isTarget) {
        ctx.shadowColor = isTarget ? "#f59e0b" : "#3b82f6";
        ctx.shadowBlur = 18;
      } else {
        ctx.shadowBlur = 0;
      }

      ctx.beginPath();
      ctx.arc(node.x, node.y, radius, 0, Math.PI * 2);

      if (isTarget) {
        ctx.fillStyle = "linear-gradient(135deg, #f59e0b, #d97706)";
        ctx.fillStyle = "#f59e0b";
      } else if (isSelected) {
        ctx.fillStyle = "#3b82f6";
      } else if (node.sent > node.received * 1.5) {
        ctx.fillStyle = "#2563eb"; // High sender
      } else if (node.received > node.sent * 1.5) {
        ctx.fillStyle = "#10b981"; // High receiver
      } else {
        ctx.fillStyle = "#6366f1"; // Balanced
      }
      ctx.fill();
      ctx.shadowBlur = 0; // reset

      ctx.strokeStyle = isSelected ? "#ffffff" : isTarget ? "#fde68a" : "rgba(255,255,255,0.2)";
      ctx.lineWidth = isSelected ? 3 : isTarget ? 2.5 : 1.5;
      ctx.stroke();

      // Node Initial Icon
      ctx.fillStyle = "#ffffff";
      ctx.font = `bold ${Math.round(radius * 0.75)}px system-ui`;
      ctx.textAlign = "center";
      ctx.textBaseline = "middle";
      const initial = (node.name || node.id).charAt(0).toUpperCase();
      ctx.fillText(initial, node.x, node.y);

      // Clean Node Label
      const displayName = cleanDisplayName(node.name) || node.id.split("@")[0];
      ctx.font = isSelected || isTarget ? "bold 11px system-ui" : "10px system-ui";
      ctx.fillStyle = isSelected ? "#38bdf8" : isTarget ? "#fbbf24" : "#e2e8f0";
      ctx.fillText(displayName, node.x, node.y + radius + 12);
    }

    ctx.restore();
  }, [activeNodes, activeEdges, selectedNode, selectedEdge, targetEmail, zoom, pan]);

  // Handle Canvas Mouse Events (Pan, Zoom, Node Drag, Click)
  const handleMouseDown = (e: React.MouseEvent<HTMLCanvasElement>) => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    const rect = canvas.getBoundingClientRect();
    const mx = e.clientX - rect.left;
    const my = e.clientY - rect.top;

    // Convert mouse coords to graph coordinates
    const gx = (mx - (rect.width / 2 + pan.x)) / zoom;
    const gy = (my - (rect.height / 2 + pan.y)) / zoom;

    // Check if clicked on a node
    for (const node of activeNodes) {
      const radius = node.is_target ? 24 : Math.max(14, Math.min(28, 12 + Math.log2(node.total + 1) * 2.5));
      const dx = gx - node.x;
      const dy = gy - node.y;
      if (dx * dx + dy * dy <= (radius + 6) * (radius + 6)) {
        draggedNode.current = node;
        setSelectedNode(node);
        setSelectedEdge(null);
        loadEmailsForEntity(node.id);
        return;
      }
    }

    // Otherwise initiate canvas panning
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

  // Cursor-centered zoom calculation
  const handleWheel = (e: React.WheelEvent<HTMLCanvasElement>) => {
    e.preventDefault();
    const canvas = canvasRef.current;
    if (!canvas) return;
    const rect = canvas.getBoundingClientRect();
    const mx = e.clientX - rect.left;
    const my = e.clientY - rect.top;
    const cx = rect.width / 2;
    const cy = rect.height / 2;

    const rawDelta = e.deltaY;
    const zoomStep = -Math.sign(rawDelta) * Math.min(0.18, Math.max(0.04, Math.abs(rawDelta) * 0.0015));
    const factor = 1 + zoomStep;
    const newZoom = Math.max(0.2, Math.min(4.0, zoom * factor));

    const newPanX = pan.x - (mx - cx - pan.x) * (newZoom / zoom - 1);
    const newPanY = pan.y - (my - cy - pan.y) * (newZoom / zoom - 1);

    setZoom(newZoom);
    setPan({ x: newPanX, y: newPanY });
  };

  // Zoom In button
  const handleZoomIn = () => {
    const newZoom = Math.min(4.0, zoom * 1.25);
    setZoom(newZoom);
  };

  // Zoom Out button
  const handleZoomOut = () => {
    const newZoom = Math.max(0.2, zoom * 0.8);
    setZoom(newZoom);
  };

  // Direct Zoom Slider
  const handleZoomSlider = (val: number) => {
    const newZoom = Math.max(0.2, Math.min(4.0, val / 100));
    setZoom(newZoom);
  };

  // Reset to 100% and Center
  const handleResetCamera = () => {
    setZoom(1);
    setPan({ x: 0, y: 0 });
  };

  // Fit all visible nodes within canvas with padding
  const handleFitView = useCallback(() => {
    const canvas = canvasRef.current;
    if (!canvas || activeNodes.length === 0) {
      setZoom(1);
      setPan({ x: 0, y: 0 });
      return;
    }
    const rect = canvas.getBoundingClientRect();
    let minX = Infinity;
    let maxX = -Infinity;
    let minY = Infinity;
    let maxY = -Infinity;

    activeNodes.forEach((n) => {
      if (n.x < minX) minX = n.x;
      if (n.x > maxX) maxX = n.x;
      if (n.y < minY) minY = n.y;
      if (n.y > maxY) maxY = n.y;
    });

    const graphWidth = Math.max(120, maxX - minX + 160);
    const graphHeight = Math.max(120, maxY - minY + 160);
    const centerX = (minX + maxX) / 2;
    const centerY = (minY + maxY) / 2;

    const padding = 60;
    const availWidth = Math.max(100, rect.width - padding * 2);
    const availHeight = Math.max(100, rect.height - padding * 2);

    const fitScale = Math.max(0.25, Math.min(2.0, Math.min(availWidth / graphWidth, availHeight / graphHeight)));
    setZoom(fitScale);
    setPan({ x: -centerX * fitScale, y: -centerY * fitScale });
  }, [activeNodes]);

  // Double click on canvas / node
  const handleDoubleClick = (e: React.MouseEvent<HTMLCanvasElement>) => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    const rect = canvas.getBoundingClientRect();
    const mx = e.clientX - rect.left;
    const my = e.clientY - rect.top;
    const gx = (mx - (rect.width / 2 + pan.x)) / zoom;
    const gy = (my - (rect.height / 2 + pan.y)) / zoom;

    // Check if double-clicked a node -> center & focus on node
    for (const node of activeNodes) {
      const radius = node.is_target ? 24 : Math.max(14, Math.min(28, 12 + Math.log2(node.total + 1) * 2.5));
      const dx = gx - node.x;
      const dy = gy - node.y;
      if (dx * dx + dy * dy <= (radius + 6) * (radius + 6)) {
        setSelectedNode(node);
        loadEmailsForEntity(node.id);
        const focusZoom = 1.35;
        setZoom(focusZoom);
        setPan({ x: -node.x * focusZoom, y: -node.y * focusZoom });
        return;
      }
    }

    // Double clicking empty space fits the view
    handleFitView();
  };

  const handlePartnerClick = (partnerId: string) => {
    const partnerNode = nodes.find((n) => n.id === partnerId);
    if (!partnerNode || !selectedNode) return;

    setSelectedEdge({
      source: selectedNode.id,
      target: partnerId,
      weight: 0,
    });
    loadEmailsBetween(selectedNode.id, partnerId);
  };

  if (loading) return <div className="empty">Loading communication graph...</div>;

  return (
    <div>
      {/* Top Header */}
      <div className="row between mb-4">
        <div>
          <h2 style={{ fontSize: 22, fontWeight: 700, color: "var(--text-0)" }}>
            Communication Network Graph
          </h2>
          <p className="muted" style={{ fontSize: 12 }}>
            Interactive relationship mapping. Drag nodes to explore, zoom with buttons or mouse wheel, and click any person to inspect communications.
          </p>
        </div>
        <div className="row gap-2">
          <button className="btn btn-ghost btn-sm" onClick={handleFitView} title="Fit entire network within screen">
            ⛶ Fit to Screen
          </button>
          <button className="btn btn-ghost btn-sm" onClick={handleResetCamera} title="Reset camera to center at 100% zoom">
            🎯 100% Center
          </button>
          <button className="btn btn-ghost btn-sm" onClick={loadData} title="Reload network data">
            ↻ Refresh
          </button>
        </div>
      </div>

      {/* Control Toolbar */}
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
                setSelectedNode(found);
                loadEmailsForEntity(found.id);
                // Center on found entity
                const searchZoom = Math.max(1.0, zoom);
                setZoom(searchZoom);
                setPan({ x: -found.x * searchZoom, y: -found.y * searchZoom });
              }
            }}
          />
        </div>
      </div>

      {/* Main Grid: Graph Canvas + Deep Workstation Inspector */}
      <div
        style={{
          display: "grid",
          gridTemplateColumns: "1fr 380px",
          gap: 16,
          alignItems: "start",
        }}
      >
        {/* Left: Canvas Area */}
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
            onDoubleClick={handleDoubleClick}
          />

          {/* Floating On-Canvas Zoom & Navigation Toolbar (Top-Right) */}
          <div
            style={{
              position: "absolute",
              top: 14,
              right: 14,
              background: "rgba(15, 23, 42, 0.88)",
              backdropFilter: "blur(10px)",
              padding: "6px 10px",
              borderRadius: "var(--r-sm)",
              border: "1px solid rgba(255,255,255,0.12)",
              display: "flex",
              alignItems: "center",
              gap: 8,
              boxShadow: "0 6px 18px rgba(0,0,0,0.45)",
              zIndex: 10,
            }}
          >
            {/* Zoom Out Button */}
            <button
              className="btn btn-ghost btn-sm"
              style={{ padding: "4px 8px", fontSize: 13, fontWeight: 700 }}
              onClick={handleZoomOut}
              title="Zoom Out (-)"
            >
              ➖
            </button>

            {/* Interactive Zoom Slider */}
            <input
              type="range"
              min="20"
              max="350"
              value={Math.round(zoom * 100)}
              onChange={(e) => handleZoomSlider(Number(e.target.value))}
              style={{ width: 70, cursor: "pointer", accentColor: "var(--accent)" }}
              title="Drag to zoom"
            />

            {/* Zoom In Button */}
            <button
              className="btn btn-ghost btn-sm"
              style={{ padding: "4px 8px", fontSize: 13, fontWeight: 700 }}
              onClick={handleZoomIn}
              title="Zoom In (+)"
            >
              ➕
            </button>

            {/* Zoom Percentage Clickable Badge */}
            <button
              className="btn btn-ghost btn-sm"
              style={{
                fontSize: 11,
                fontFamily: "var(--mono)",
                color: "var(--accent)",
                padding: "2px 6px",
                border: "1px solid rgba(56, 189, 248, 0.3)",
                borderRadius: 4,
              }}
              onClick={handleResetCamera}
              title="Click to reset to 100%"
            >
              {Math.round(zoom * 100)}%
            </button>

            {/* Fit Network Button */}
            <button
              className="btn btn-ghost btn-sm"
              style={{ padding: "4px 8px", fontSize: 11 }}
              onClick={handleFitView}
              title="Fit all nodes on screen"
            >
              ⛶ Fit
            </button>
          </div>

          {/* Canvas Helper Pill (Bottom Center) */}
          <div
            style={{
              position: "absolute",
              bottom: 12,
              right: 14,
              background: "rgba(15, 23, 42, 0.8)",
              backdropFilter: "blur(6px)",
              padding: "4px 10px",
              borderRadius: "var(--r-xs)",
              fontSize: 10,
              color: "var(--text-3)",
              border: "1px solid rgba(255,255,255,0.06)",
            }}
          >
            💡 Scroll / buttons to zoom · Drag canvas to pan · Double-click to fit
          </div>

          {/* Canvas Overlay Legend (Bottom Left) */}
          <div
            style={{
              position: "absolute",
              bottom: 12,
              left: 12,
              background: "rgba(15, 23, 42, 0.88)",
              backdropFilter: "blur(8px)",
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
        </div>

        {/* Right: Forensic Relationship & Exchanged Messages Inspector */}
        <div
          className="card mb-0"
          style={{
            padding: 16,
            height: "72vh",
            overflowY: "auto",
            display: "flex",
            flexDirection: "column",
            gap: 14,
          }}
        >
          {selectedNode ? (
            <>
              {/* Selected Node Summary */}
              <div
                style={{
                  padding: 14,
                  background: "var(--bg-3)",
                  borderRadius: "var(--r-md)",
                  borderLeft: selectedNode.is_target
                    ? "4px solid #f59e0b"
                    : "4px solid var(--accent)",
                }}
              >
                <div className="row between mb-2">
                  <strong style={{ fontSize: 14, color: "var(--text-0)" }}>
                    {cleanDisplayName(selectedNode.name) || selectedNode.id}
                  </strong>
                  {selectedNode.is_target && (
                    <span className="badge badge-orange" style={{ fontSize: 10 }}>
                      TARGET
                    </span>
                  )}
                </div>
                <div
                  style={{
                    fontSize: 11,
                    color: "var(--accent)",
                    fontFamily: "var(--mono)",
                    marginBottom: 10,
                  }}
                >
                  {selectedNode.id}
                </div>

                <div className="grid-3" style={{ gap: 6, textAlign: "center" }}>
                  <div style={{ background: "var(--bg-2)", padding: 6, borderRadius: "var(--r-xs)" }}>
                    <div style={{ fontSize: 14, fontWeight: 700, color: "#3b82f6" }}>
                      {selectedNode.sent}
                    </div>
                    <div style={{ fontSize: 9, color: "var(--text-3)" }}>SENT</div>
                  </div>
                  <div style={{ background: "var(--bg-2)", padding: 6, borderRadius: "var(--r-xs)" }}>
                    <div style={{ fontSize: 14, fontWeight: 700, color: "#22c55e" }}>
                      {selectedNode.received}
                    </div>
                    <div style={{ fontSize: 9, color: "var(--text-3)" }}>RECEIVED</div>
                  </div>
                  <div style={{ background: "var(--bg-2)", padding: 6, borderRadius: "var(--r-xs)" }}>
                    <div style={{ fontSize: 14, fontWeight: 700, color: "var(--text-0)" }}>
                      {selectedNode.total}
                    </div>
                    <div style={{ fontSize: 9, color: "var(--text-3)" }}>TOTAL</div>
                  </div>
                </div>
              </div>

              {/* Connected Communication Partners */}
              <div>
                <div className="row between mb-2">
                  <strong style={{ fontSize: 12, color: "var(--text-0)" }}>
                    🔗 Direct Partners in Network ({connectedPartners.length})
                  </strong>
                </div>
                {connectedPartners.length > 0 ? (
                  <div style={{ display: "flex", flexDirection: "column", gap: 3, maxHeight: 130, overflowY: "auto" }}>
                    {connectedPartners.map((p) => {
                      const isPartnerSelected =
                        selectedEdge &&
                        ((selectedEdge.source === p.id && selectedEdge.target === selectedNode.id) ||
                          (selectedEdge.target === p.id && selectedEdge.source === selectedNode.id));

                      return (
                        <div
                          key={p.id}
                          className="row between tr-click"
                          style={{
                            padding: "5px 8px",
                            borderRadius: "var(--r-xs)",
                            background: isPartnerSelected
                              ? "var(--accent-subtle)"
                              : "var(--bg-3)",
                            border: isPartnerSelected
                              ? "1px solid var(--accent)"
                              : "1px solid transparent",
                          }}
                          onClick={() => handlePartnerClick(p.id)}
                        >
                          <span
                            style={{
                              fontSize: 11,
                              color: isPartnerSelected ? "var(--accent)" : "var(--text-1)",
                              overflow: "hidden",
                              textOverflow: "ellipsis",
                              whiteSpace: "nowrap",
                            }}
                          >
                            {p.name}
                          </span>
                          <span className="badge badge-blue" style={{ fontSize: 10 }}>
                            {p.count} emails
                          </span>
                        </div>
                      );
                    })}
                  </div>
                ) : (
                  <div className="muted text-sm">No connected links within current threshold</div>
                )}
              </div>

              {/* Exchanged Messages Feed */}
              <div style={{ flex: 1, display: "flex", flexDirection: "column", minHeight: 0 }}>
                <div className="row between mb-2">
                  <strong style={{ fontSize: 12, color: "var(--text-0)" }}>
                    📧 {selectedEdge ? "Thread Between Partners" : "Recent Communications"} (
                    {inspectorEmails.length})
                  </strong>
                  {selectedEdge && (
                    <button
                      className="btn btn-ghost btn-sm"
                      style={{ fontSize: 10, padding: "1px 6px" }}
                      onClick={() => {
                        setSelectedEdge(null);
                        loadEmailsForEntity(selectedNode.id);
                      }}
                    >
                      Clear Link Filter
                    </button>
                  )}
                </div>

                {loadingEmails ? (
                  <div className="empty" style={{ padding: 16 }}>Loading messages...</div>
                ) : inspectorEmails.length === 0 ? (
                  <div className="empty" style={{ padding: 16 }}>No exchanged emails found</div>
                ) : (
                  <div
                    style={{
                      flex: 1,
                      overflowY: "auto",
                      border: "1px solid var(--border)",
                      borderRadius: "var(--r-sm)",
                    }}
                  >
                    {inspectorEmails.map((em) => {
                      const isEmailActive = selectedEmail?.id === em.id;
                      return (
                        <div
                          key={em.id}
                          className="tr-click"
                          style={{
                            padding: "7px 10px",
                            borderBottom: "1px solid var(--border)",
                            background: isEmailActive ? "var(--accent-subtle)" : "transparent",
                            fontSize: 11,
                          }}
                          onClick={() => setSelectedEmail(isEmailActive ? null : em)}
                        >
                          <div
                            style={{
                              fontWeight: 600,
                              color: "var(--text-0)",
                              overflow: "hidden",
                              textOverflow: "ellipsis",
                              whiteSpace: "nowrap",
                            }}
                          >
                            {em.subject || "(no subject)"}
                          </div>
                          <div className="row between mt-1" style={{ fontSize: 10, color: "var(--text-3)" }}>
                            <span style={{ overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap", maxWidth: 180 }}>
                              {cleanDisplayName(em.from_display) || em.from_addr}
                            </span>
                            <span>{em.date_sent_utc ? new Date(em.date_sent_utc).toLocaleDateString() : ""}</span>
                          </div>
                        </div>
                      );
                    })}
                  </div>
                )}

                {/* Inline Message Preview */}
                {selectedEmail && (
                  <div
                    style={{
                      marginTop: 10,
                      padding: 10,
                      background: "var(--bg-1)",
                      border: "1px solid var(--border)",
                      borderRadius: "var(--r-sm)",
                    }}
                  >
                    <div className="row between mb-1">
                      <strong style={{ fontSize: 11, color: "var(--text-0)" }}>
                        {selectedEmail.subject || "(no subject)"}
                      </strong>
                      <button
                        className="btn btn-ghost btn-sm"
                        style={{ fontSize: 9, padding: "0 4px" }}
                        onClick={() => setSelectedEmail(null)}
                      >
                        ✕
                      </button>
                    </div>
                    {selectedEmail.body_text && (
                      <pre
                        style={{
                          background: "var(--bg-0)",
                          padding: 8,
                          borderRadius: "var(--r-xs)",
                          fontSize: 10,
                          maxHeight: 90,
                          overflow: "auto",
                          whiteSpace: "pre-wrap",
                          marginTop: 4,
                        }}
                      >
                        {selectedEmail.body_text}
                      </pre>
                    )}
                  </div>
                )}
              </div>
            </>
          ) : (
            <div className="card empty">Click any entity on the canvas to inspect its relationships</div>
          )}
        </div>
      </div>
    </div>
  );
}
