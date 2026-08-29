import { useState, useEffect, useMemo } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  GraphNode,
  GraphEdge,
  ExchangedEmail,
  GraphProps,
  cleanDisplayName,
} from "./graph/types";
import { GraphToolbar } from "./graph/GraphToolbar";
import { GraphCanvas } from "./graph/GraphCanvas";
import { GraphInspector } from "./graph/GraphInspector";

export function GraphView({ caseId, evidenceFilter }: GraphProps) {
  const [nodes, setNodes] = useState<GraphNode[]>([]);
  const [edges, setEdges] = useState<GraphEdge[]>([]);
  const [targetEmail, setTargetEmail] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);

  const [selectedNode, setSelectedNode] = useState<GraphNode | null>(null);
  const [selectedEdge, setSelectedEdge] = useState<GraphEdge | null>(null);
  const [searchTerm, setSearchTerm] = useState("");
  const [maxNodes, setMaxNodes] = useState<number>(35);
  const [minEmails, setMinEmails] = useState<number>(5);
  const [layoutMode, setLayoutMode] = useState<"force" | "radial">("force");

  const [inspectorEmails, setInspectorEmails] = useState<ExchangedEmail[]>([]);
  const [loadingEmails, setLoadingEmails] = useState(false);
  const [selectedEmail, setSelectedEmail] = useState<ExchangedEmail | null>(null);

  const [zoom, setZoom] = useState(1);
  const [pan, setPan] = useState({ x: 0, y: 0 });

  useEffect(() => {
    loadData();
  }, [caseId, evidenceFilter]);

  const loadData = async () => {
    setLoading(true);
    try {
      const res = await invoke<any>("graph_data", { input: { case_id: caseId, evidence_id: evidenceFilter || undefined } });
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

  const loadEmailsForEntity = async (email: string) => {
    setLoadingEmails(true);
    setSelectedEmail(null);
    try {
      const res = await invoke<ExchangedEmail[]>("entity_emails", {
        input: {
          case_id: caseId,
          evidence_id: evidenceFilter || undefined,
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

  const handleResetCamera = () => {
    setZoom(1);
    setPan({ x: 0, y: 0 });
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

  const handleSelectNode = (node: GraphNode) => {
    setSelectedNode(node);
    loadEmailsForEntity(node.id);
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
            Interactive sociogram and relationship mapping. Drag nodes, zoom with mouse wheel, and click any person or link to inspect shared threads.
          </p>
        </div>
        <div className="row gap-2">
          <button className="btn btn-ghost btn-sm" onClick={handleResetCamera} title="Center camera">
            🎯 Reset View
          </button>
          <button className="btn btn-ghost btn-sm" onClick={loadData}>
            ↻ Refresh
          </button>
        </div>
      </div>

      {/* Control Toolbar */}
      <GraphToolbar
        layoutMode={layoutMode}
        setLayoutMode={setLayoutMode}
        maxNodes={maxNodes}
        setMaxNodes={setMaxNodes}
        minEmails={minEmails}
        setMinEmails={setMinEmails}
        searchTerm={searchTerm}
        setSearchTerm={setSearchTerm}
        activeNodes={activeNodes}
        onSelectNode={handleSelectNode}
      />

      {/* Main Grid: Graph Canvas + Deep Workstation Inspector */}
      <div
        style={{
          display: "grid",
          gridTemplateColumns: "1fr 380px",
          gap: 16,
          alignItems: "start",
        }}
      >
        <GraphCanvas
          activeNodes={activeNodes}
          activeEdges={activeEdges}
          selectedNode={selectedNode}
          selectedEdge={selectedEdge}
          targetEmail={targetEmail}
          layoutMode={layoutMode}
          zoom={zoom}
          setZoom={setZoom}
          pan={pan}
          setPan={setPan}
          onSelectNode={handleSelectNode}
          onClearEdge={() => setSelectedEdge(null)}
        />

        <GraphInspector
          selectedNode={selectedNode}
          selectedEdge={selectedEdge}
          connectedPartners={connectedPartners}
          inspectorEmails={inspectorEmails}
          loadingEmails={loadingEmails}
          selectedEmail={selectedEmail}
          setSelectedEmail={setSelectedEmail}
          onPartnerClick={handlePartnerClick}
          onClearLinkFilter={() => {
            setSelectedEdge(null);
            if (selectedNode) loadEmailsForEntity(selectedNode.id);
          }}
        />
      </div>
    </div>
  );
}
