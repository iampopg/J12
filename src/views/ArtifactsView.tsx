import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";

export interface TaxonomySubcategorySummary {
  subcategory_id: string;
  name: string;
  count: number;
}

export interface TaxonomyDomainSummary {
  domain_id: string;
  name: string;
  icon: string;
  total_count: number;
  subcategories: TaxonomySubcategorySummary[];
}

export interface ForensicTaxonomyArtifact {
  id: string;
  domain_id: string;
  subcategory_id: string;
  title: string;
  primary_value: string;
  secondary_value: string | null;
  details: string;
  severity: "critical" | "high" | "medium" | "low" | "info";
  artifact_type: "native" | "recovered" | "derived";
  email_id: string;
  email_subject: string | null;
  email_from: string;
  date_sent_utc: string | null;
}

interface Props {
  caseId: string;
  onSelectEmail?: (emailId: string) => void;
}

export function ArtifactsView({ caseId, onSelectEmail }: Props) {
  const [taxonomy, setTaxonomy] = useState<TaxonomyDomainSummary[]>([]);
  const [artifacts, setArtifacts] = useState<ForensicTaxonomyArtifact[]>([]);
  const [selectedDomain, setSelectedDomain] = useState<string>("all");
  const [selectedSubcategory, setSelectedSubcategory] = useState<string>("all");
  const [selectedArtifactType, setSelectedArtifactType] = useState<string>("all");
  const [search, setSearch] = useState<string>("");
  const [loading, setLoading] = useState<boolean>(true);
  const [selectedArtifact, setSelectedArtifact] = useState<ForensicTaxonomyArtifact | null>(null);
  const [toastMessage, setToastMessage] = useState<string | null>(null);

  const showToast = (msg: string) => {
    setToastMessage(msg);
    setTimeout(() => setToastMessage(null), 3000);
  };

  useEffect(() => {
    loadTaxonomy();
  }, [caseId]);

  useEffect(() => {
    loadArtifacts();
  }, [caseId, selectedDomain, selectedSubcategory, selectedArtifactType]);

  const loadTaxonomy = async () => {
    try {
      const domains = await invoke<TaxonomyDomainSummary[]>("case_artifacts_summary", { input: { case_id: caseId } });
      setTaxonomy(domains);
    } catch (e) {
      console.error("Failed to load taxonomy:", e);
    }
  };

  const loadArtifacts = async () => {
    setLoading(true);
    try {
      const list = await invoke<ForensicTaxonomyArtifact[]>("case_artifacts_list", {
        input: {
          case_id: caseId,
          domain: selectedDomain,
          subcategory: selectedSubcategory,
          artifact_type: selectedArtifactType,
          search
        }
      });
      setArtifacts(list);
    } catch (e) {
      console.error("Failed to load artifacts:", e);
    } finally {
      setLoading(false);
    }
  };

  const handleSearchSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    loadArtifacts();
  };

  const copyToClipboard = (text: string) => {
    navigator.clipboard.writeText(text);
    showToast("✓ Copied value to clipboard");
  };

  const totalAllArtifacts = taxonomy.reduce((acc, d) => acc + d.total_count, 0);

  const exportArtifactsCSV = () => {
    if (artifacts.length === 0) return;
    const headers = ["Domain", "Subcategory", "Title", "Primary Value", "Secondary Value", "Type", "Severity", "Subject", "From", "Date Sent"];
    const rows = artifacts.map(a => [
      `"${a.domain_id}"`,
      `"${a.subcategory_id}"`,
      `"${(a.title || "").replace(/"/g, '""')}"`,
      `"${(a.primary_value || "").replace(/"/g, '""')}"`,
      `"${(a.secondary_value || "").replace(/"/g, '""')}"`,
      `"${a.artifact_type}"`,
      `"${a.severity}"`,
      `"${(a.email_subject || "").replace(/"/g, '""')}"`,
      `"${a.email_from}"`,
      `"${a.date_sent_utc || ""}"`
    ]);
    const csvContent = [headers.join(","), ...rows.map(r => r.join(","))].join("\n");
    const blob = new Blob([csvContent], { type: "text/csv;charset=utf-8;" });
    const url = URL.createObjectURL(blob);
    const link = document.createElement("a");
    link.setAttribute("href", url);
    link.setAttribute("download", `J12_Forensic_Taxonomy_Artifacts_${caseId.slice(0, 8)}.csv`);
    document.body.appendChild(link);
    link.click();
    document.body.removeChild(link);
    showToast("📥 Exported Forensic Artifacts Dossier (CSV)");
  };

  const getSeverityBadge = (sev: string) => {
    switch (sev) {
      case "critical": return <span className="badge badge-red">CRITICAL</span>;
      case "high": return <span className="badge badge-orange">HIGH</span>;
      case "medium": return <span className="badge badge-blue">MEDIUM</span>;
      default: return <span className="badge badge-gray">INFO</span>;
    }
  };

  const getTypeBadge = (t: string) => {
    switch (t) {
      case "recovered": return <span className="badge" style={{ background: "rgba(239, 68, 68, 0.15)", color: "#ef4444" }}>🗑️ RECOVERED</span>;
      case "derived": return <span className="badge" style={{ background: "rgba(168, 85, 247, 0.15)", color: "#c084fc" }}>🧠 DERIVED</span>;
      default: return <span className="badge" style={{ background: "rgba(56, 189, 248, 0.15)", color: "#38bdf8" }}>📄 NATIVE</span>;
    }
  };

  return (
    <div>
      {/* Toast */}
      {toastMessage && (
        <div 
          className="card"
          style={{
            position: "fixed",
            bottom: 24,
            right: 24,
            zIndex: 9999,
            background: "#1e293b",
            border: "1px solid #22c55e",
            color: "#4ade80",
            padding: "10px 18px",
            fontWeight: 600,
            fontSize: 13,
            boxShadow: "0 10px 25px rgba(0,0,0,0.5)",
          }}
        >
          {toastMessage}
        </div>
      )}

      {/* Top Header */}
      <div className="row between mb-4">
        <div>
          <h2 style={{ fontSize: 22, fontWeight: 700, color: "var(--text-0)" }}>
            Forensic Artifact Taxonomy &amp; Evidence Intelligence
          </h2>
          <p className="muted" style={{ margin: 0 }}>
            Belkasoft-grade structured evidence taxonomy — Native, Recovered, and Derived intelligence categorized across 14 forensic domains.
          </p>
        </div>
        <div className="row gap-2">
          <button className="btn btn-ghost btn-sm" onClick={exportArtifactsCSV} title="Export artifacts to CSV">
            📥 Export Intelligence CSV
          </button>
          <button className="btn btn-ghost btn-sm" onClick={() => { loadTaxonomy(); loadArtifacts(); }}>
            ↻ Refresh
          </button>
        </div>
      </div>

      {/* Main Two-Column Taxonomy Workspace */}
      <div style={{ display: "grid", gridTemplateColumns: "300px 1fr", gap: 16 }}>
        
        {/* Left Column: Artifact Taxonomy Category Tree */}
        <div className="card" style={{ padding: 12, maxHeight: "calc(100vh - 180px)", overflowY: "auto" }}>
          <div 
            style={{ 
              fontSize: 11, 
              fontWeight: 800, 
              letterSpacing: "0.8px", 
              color: "var(--text-3)", 
              marginBottom: 10,
              padding: "4px 8px"
            }}
          >
            ARTIFACT TAXONOMY
          </div>

          {/* All Artifacts Root */}
          <div 
            style={{
              display: "flex",
              alignItems: "center",
              justifyContent: "space-between",
              padding: "8px 10px",
              borderRadius: "var(--r-sm)",
              cursor: "pointer",
              marginBottom: 6,
              background: selectedDomain === "all" ? "var(--accent)" : "transparent",
              color: selectedDomain === "all" ? "#000" : "var(--text-0)",
              fontWeight: selectedDomain === "all" ? 700 : 500,
              fontSize: 13,
            }}
            onClick={() => { setSelectedDomain("all"); setSelectedSubcategory("all"); }}
          >
            <div className="row gap-2" style={{ alignItems: "center" }}>
              <span>📁</span>
              <span>All Artifacts</span>
            </div>
            <span 
              className="badge" 
              style={{ 
                background: selectedDomain === "all" ? "#000" : "var(--bg-3)", 
                color: selectedDomain === "all" ? "#fff" : "var(--text-1)",
                fontSize: 11 
              }}
            >
              {totalAllArtifacts}
            </span>
          </div>

          {/* Domain List */}
          <div style={{ display: "flex", flexDirection: "column", gap: 3 }}>
            {taxonomy.map((dom) => {
              const isDomainSelected = selectedDomain === dom.domain_id;
              return (
                <div key={dom.domain_id} style={{ display: "flex", flexDirection: "column" }}>
                  <div
                    style={{
                      display: "flex",
                      alignItems: "center",
                      justifyContent: "space-between",
                      padding: "7px 10px",
                      borderRadius: "var(--r-sm)",
                      cursor: "pointer",
                      background: isDomainSelected && selectedSubcategory === "all" ? "var(--bg-3)" : "transparent",
                      color: isDomainSelected ? "var(--accent)" : "var(--text-1)",
                      fontWeight: isDomainSelected ? 700 : 500,
                      fontSize: 12.5,
                      borderLeft: isDomainSelected ? "3px solid var(--accent)" : "3px solid transparent",
                    }}
                    onClick={() => {
                      setSelectedDomain(dom.domain_id);
                      setSelectedSubcategory("all");
                    }}
                  >
                    <div className="row gap-2" style={{ alignItems: "center", overflow: "hidden" }}>
                      <span>{dom.icon}</span>
                      <span style={{ textOverflow: "ellipsis", whiteSpace: "nowrap", overflow: "hidden" }}>{dom.name}</span>
                    </div>
                    <span 
                      style={{ 
                        fontSize: 11, 
                        fontFamily: "var(--mono)",
                        color: isDomainSelected ? "var(--accent)" : "var(--text-3)",
                        fontWeight: 600
                      }}
                    >
                      {dom.total_count}
                    </span>
                  </div>

                  {/* Subcategories (if domain active) */}
                  {isDomainSelected && dom.subcategories.length > 0 && (
                    <div style={{ display: "flex", flexDirection: "column", paddingLeft: 24, marginTop: 2, marginBottom: 4, gap: 2 }}>
                      {dom.subcategories.map((sub) => {
                        const isSubSelected = selectedSubcategory === sub.subcategory_id;
                        return (
                          <div
                            key={sub.subcategory_id}
                            style={{
                              display: "flex",
                              alignItems: "center",
                              justifyContent: "space-between",
                              padding: "4px 8px",
                              borderRadius: "var(--r-sm)",
                              cursor: "pointer",
                              fontSize: 11.5,
                              color: isSubSelected ? "var(--accent)" : "var(--text-2)",
                              background: isSubSelected ? "rgba(56, 189, 248, 0.1)" : "transparent",
                              fontWeight: isSubSelected ? 700 : 400,
                            }}
                            onClick={(e) => {
                              e.stopPropagation();
                              setSelectedSubcategory(sub.subcategory_id);
                            }}
                          >
                            <span>↳ {sub.name}</span>
                            <span style={{ fontSize: 10, fontFamily: "var(--mono)", color: "var(--text-3)" }}>
                              {sub.count}
                            </span>
                          </div>
                        );
                      })}
                    </div>
                  )}
                </div>
              );
            })}
          </div>
        </div>

        {/* Right Column: Artifacts Explorer & Live Inspector */}
        <div style={{ display: "flex", flexDirection: "column", gap: 12 }}>
          
          {/* Filter Bar */}
          <div className="card" style={{ padding: "12px 16px" }}>
            <form onSubmit={handleSearchSubmit} className="row between gap-3" style={{ flexWrap: "wrap" }}>
              <div className="row gap-2" style={{ flex: 1, minWidth: 260 }}>
                <input
                  className="input"
                  style={{ flex: 1, padding: "8px 12px", fontSize: 13 }}
                  placeholder="Search artifacts (e.g. phone, OTP code, IP, BTC wallet, URL, AnyDesk)..."
                  value={search}
                  onChange={(e) => setSearch(e.target.value)}
                />
                <button type="submit" className="btn btn-primary btn-sm">🔍 Search</button>
                {search && (
                  <button 
                    type="button" 
                    className="btn btn-ghost btn-sm" 
                    onClick={() => { setSearch(""); setTimeout(loadArtifacts, 50); }}
                  >
                    Clear
                  </button>
                )}
              </div>

              {/* Artifact Type Filter */}
              <div className="row gap-2" style={{ alignItems: "center" }}>
                <span className="muted text-xs">TYPE:</span>
                <select 
                  className="input text-xs" 
                  style={{ padding: "6px 10px" }}
                  value={selectedArtifactType}
                  onChange={(e) => setSelectedArtifactType(e.target.value)}
                >
                  <option value="all">All Types</option>
                  <option value="native">📄 Native Evidence</option>
                  <option value="recovered">🗑️ Recovered / Carved</option>
                  <option value="derived">🧠 Derived Intelligence</option>
                </select>
              </div>
            </form>
          </div>

          {/* Main Grid: Feed & Inspector */}
          <div style={{ display: "grid", gridTemplateColumns: selectedArtifact ? "1fr 380px" : "1fr", gap: 14 }}>
            
            {/* Artifacts Feed */}
            <div style={{ display: "flex", flexDirection: "column", gap: 8 }}>
              {loading ? (
                <div className="empty" style={{ padding: 40 }}>Classifying and indexing forensic taxonomy artifacts...</div>
              ) : artifacts.length === 0 ? (
                <div className="card empty" style={{ padding: 40 }}>
                  No artifacts found for the selected taxonomy domain or query.
                </div>
              ) : (
                artifacts.map((a) => {
                  const isSelected = selectedArtifact?.id === a.id;
                  return (
                    <div 
                      key={a.id}
                      className="card"
                      style={{
                        padding: "12px 16px",
                        margin: 0,
                        cursor: "pointer",
                        borderLeft: a.severity === "critical" ? "4px solid var(--danger)" : a.severity === "high" ? "4px solid var(--warning)" : a.severity === "medium" ? "4px solid var(--accent)" : "4px solid var(--border)",
                        background: isSelected ? "var(--bg-3)" : "var(--bg-2)",
                        transition: "all 0.15s ease",
                      }}
                      onClick={() => setSelectedArtifact(a)}
                    >
                      <div className="row between mb-2">
                        <div className="row gap-2" style={{ alignItems: "center" }}>
                          <span style={{ fontSize: 13, fontWeight: 700, color: "var(--text-0)" }}>{a.title}</span>
                          {getSeverityBadge(a.severity)}
                          {getTypeBadge(a.artifact_type)}
                        </div>
                        <span style={{ fontSize: 11, color: "var(--text-3)", fontFamily: "var(--mono)" }}>
                          {a.date_sent_utc ? new Date(a.date_sent_utc).toLocaleDateString() : ""}
                        </span>
                      </div>

                      {/* Highlighted Extracted Value Box */}
                      <div 
                        style={{
                          background: "rgba(15, 23, 42, 0.9)",
                          border: "1px solid var(--border)",
                          borderRadius: "var(--r-sm)",
                          padding: "8px 12px",
                          fontFamily: "var(--mono)",
                          fontSize: 12.5,
                          color: "#38bdf8",
                          marginBottom: 8,
                          display: "flex",
                          alignItems: "center",
                          justifyContent: "space-between",
                          wordBreak: "break-all"
                        }}
                      >
                        <span style={{ fontWeight: 600 }}>{a.primary_value}</span>
                        <button 
                          className="btn btn-ghost btn-sm" 
                          style={{ padding: "2px 8px", fontSize: 10 }}
                          onClick={(e) => { e.stopPropagation(); copyToClipboard(a.primary_value); }}
                        >
                          📋 Copy
                        </button>
                      </div>

                      <div className="row between">
                        <div style={{ fontSize: 11.5, color: "var(--text-2)", overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap", maxWidth: "70%" }}>
                          <span className="muted">Source:</span> {a.email_from} · <span className="muted">Subject:</span> {a.email_subject || "(No Subject)"}
                        </div>
                        {onSelectEmail && (
                          <button 
                            className="btn btn-ghost btn-sm" 
                            style={{ fontSize: 11, padding: "2px 8px" }}
                            onClick={(e) => { e.stopPropagation(); onSelectEmail(a.email_id); }}
                          >
                            ✉️ View Email
                          </button>
                        )}
                      </div>
                    </div>
                  );
                })
              )}
            </div>

            {/* Live Inspector Drawer */}
            {selectedArtifact && (
              <div className="card" style={{ position: "sticky", top: 16, height: "fit-content", padding: 18 }}>
                <div className="row between mb-3">
                  <h3 style={{ fontSize: 15, fontWeight: 700, color: "var(--text-0)", margin: 0 }}>
                    Artifact Forensic Dossier
                  </h3>
                  <button className="btn btn-ghost btn-sm" onClick={() => setSelectedArtifact(null)}>✕</button>
                </div>

                <div style={{ marginBottom: 12 }}>
                  <div className="label">TAXONOMY CLASSIFICATION</div>
                  <div style={{ fontSize: 13, fontWeight: 700, color: "var(--text-0)" }}>
                    {selectedArtifact.title}
                  </div>
                  <div style={{ fontSize: 11, fontFamily: "var(--mono)", color: "var(--text-3)", marginTop: 2 }}>
                    Domain: <strong>{selectedArtifact.domain_id}</strong> / {selectedArtifact.subcategory_id}
                  </div>
                </div>

                <div style={{ marginBottom: 12 }}>
                  <div className="label">EXTRACTED PRIMARY VALUE</div>
                  <div 
                    style={{
                      background: "var(--bg-3)",
                      border: "1px solid var(--accent)",
                      borderRadius: "var(--r-sm)",
                      padding: "10px 12px",
                      fontFamily: "var(--mono)",
                      fontSize: 12.5,
                      color: "#4ade80",
                      fontWeight: 700,
                      wordBreak: "break-all"
                    }}
                  >
                    {selectedArtifact.primary_value}
                  </div>
                  <button 
                    className="btn btn-ghost btn-sm" 
                    style={{ width: "100%", marginTop: 6, fontSize: 11 }}
                    onClick={() => copyToClipboard(selectedArtifact.primary_value)}
                  >
                    📋 Copy Value
                  </button>
                </div>

                {selectedArtifact.secondary_value && (
                  <div style={{ marginBottom: 12 }}>
                    <div className="label">SECONDARY METADATA / PROVENANCE</div>
                    <div style={{ fontSize: 11, fontFamily: "var(--mono)", color: "var(--text-1)", wordBreak: "break-all" }}>
                      {selectedArtifact.secondary_value}
                    </div>
                  </div>
                )}

                <div style={{ marginBottom: 12 }}>
                  <div className="label">EVIDENCE CONTEXT PREVIEW</div>
                  <div 
                    style={{
                      background: "var(--bg-1)",
                      border: "1px solid var(--border)",
                      borderRadius: "var(--r-sm)",
                      padding: "10px 12px",
                      fontSize: 11.5,
                      color: "var(--text-1)",
                      maxHeight: 160,
                      overflowY: "auto",
                      lineHeight: 1.4
                    }}
                  >
                    {selectedArtifact.details || "No preview snippet available."}
                  </div>
                </div>

                <div style={{ marginBottom: 16 }}>
                  <div className="label">ORIGINATING EMAIL</div>
                  <div style={{ fontSize: 12, color: "var(--text-1)", marginBottom: 4 }}>
                    <strong>Subject:</strong> {selectedArtifact.email_subject || "(No Subject)"}
                  </div>
                  <div style={{ fontSize: 11, color: "var(--text-3)", fontFamily: "var(--mono)", marginBottom: 4 }}>
                    <strong>From:</strong> {selectedArtifact.email_from}
                  </div>
                  <div style={{ fontSize: 11, color: "var(--text-3)" }}>
                    <strong>Date:</strong> {selectedArtifact.date_sent_utc || "Unknown"}
                  </div>
                </div>

                {onSelectEmail && (
                  <button 
                    className="btn btn-primary btn-sm" 
                    style={{ width: "100%" }}
                    onClick={() => onSelectEmail(selectedArtifact.email_id)}
                  >
                    ✉️ Open Email in Forensic Viewer
                  </button>
                )}
              </div>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}
