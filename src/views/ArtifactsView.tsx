import { useState, useEffect, useMemo } from "react";
import { invoke } from "@tauri-apps/api/core";
import { RichEmailBodyViewer } from "../components/RichEmailBodyViewer";
import { EmailDetailModal } from "../components/EmailDetailModal";
import { useScanState } from "../utils/scanState";

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
  confidence?: "high" | "medium" | "low" | null;
  email_id: string;
  email_subject: string | null;
  email_from: string;
  date_sent_utc: string | null;
  occurrenceCount?: number;
}

export interface EmailMessage {
  id: string;
  evidence_id: string;
  case_id: string;
  message_id: string | null;
  from_addr: string;
  from_display: string | null;
  to_addrs: string;
  cc_addrs: string;
  subject: string | null;
  date_sent: string | null;
  date_sent_utc: string | null;
  headers_raw: string | null;
  body_text: string | null;
  body_html: string | null;
  folder_name: string | null;
  folder_category: string;
  recovery_status: string;
  deleted_recovered: boolean;
  risk_score: number;
  flags: string;
}

interface Props {
  caseId: string;
  onSelectEmail?: (emailId: string) => void;
}

export function ArtifactsView({ caseId }: Props) {
  const [taxonomy, setTaxonomy] = useState<TaxonomyDomainSummary[]>([]);
  const [artifacts, setArtifacts] = useState<ForensicTaxonomyArtifact[]>([]);
  const [selectedDomain, setSelectedDomain] = useState<string>("all");
  const [selectedSubcategory, setSelectedSubcategory] = useState<string>("all");
  const [selectedArtifactType, setSelectedArtifactType] = useState<string>("all");
  const [search, setSearch] = useState<string>("");
  const [showEmptyDomains, setShowEmptyDomains] = useState<boolean>(false);
  const [dedupUnique, setDedupUnique] = useState<boolean>(true);
  const [loading, setLoading] = useState<boolean>(true);
  const [scanState, setScanState] = useScanState();
  const [selectedArtifact, setSelectedArtifact] = useState<ForensicTaxonomyArtifact | null>(null);
  const [previewEmail, setPreviewEmail] = useState<EmailMessage | null>(null);
  const [_loadingEmail, setLoadingEmail] = useState<boolean>(false);
  const [toastMessage, setToastMessage] = useState<string | null>(null);

  const showToast = (msg: string) => {
    setToastMessage(msg);
    setTimeout(() => setToastMessage(null), 3000);
  };

  useEffect(() => {
    loadTaxonomy();
  }, [caseId, showEmptyDomains]);

  useEffect(() => {
    loadArtifacts();
  }, [caseId, selectedDomain, selectedSubcategory, selectedArtifactType]);

  const loadTaxonomy = async () => {
    try {
      const domains = await invoke<TaxonomyDomainSummary[]>("case_artifacts_summary", { 
        input: { 
          case_id: caseId,
          show_all: showEmptyDomains
        } 
      });
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

  const displayedArtifacts = useMemo(() => {
    if (!dedupUnique) return artifacts;
    const map = new Map<string, { item: ForensicTaxonomyArtifact; count: number }>();
    for (const a of artifacts) {
      const key = `${a.domain_id}|${a.subcategory_id}|${(a.primary_value || a.title || "").trim().toLowerCase()}`;
      const existing = map.get(key);
      if (existing) {
        existing.count += 1;
      } else {
        map.set(key, { item: a, count: 1 });
      }
    }
    return Array.from(map.values()).map(({ item, count }) => ({
      ...item,
      occurrenceCount: count,
    }));
  }, [artifacts, dedupUnique]);

  const handleRescan = async () => {
    setScanState({
      scanning: true,
      progress: 15,
      stage: "Reading emails and headers from database...",
    });
    
    const progressInterval = setInterval(() => {
      setScanState({
        progress: Math.min(scanState.progress + 8, 92),
      });
    }, 300);

    try {
      setTimeout(() => setScanState({ stage: "Classifying financial, banking, crypto, credentials, and app accounts..." }), 400);
      setTimeout(() => setScanState({ stage: "Extracting attachment signatures and forensic IOCs..." }), 1000);
      
      const count = await invoke<number>("rescan_case_artifacts", { input: { case_id: caseId } });
      clearInterval(progressInterval);
      setScanState({
        progress: 100,
        stage: `Completed! Indexed ${count} forensic artifacts.`,
      });
      showToast(`✓ Scanned and indexed ${count} artifacts`);
      await Promise.all([loadTaxonomy(), loadArtifacts()]);
    } catch (e: any) {
      clearInterval(progressInterval);
      console.error(e);
      showToast(`❌ Error scanning artifacts: ${e}`);
    } finally {
      setTimeout(() => {
        setScanState({
          scanning: false,
          progress: 0,
          stage: "",
        });
      }, 1200);
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

  const openEmailModal = async (emailId: string) => {
    if (!emailId) return;
    setLoadingEmail(true);
    try {
      const em = await invoke<EmailMessage | null>("email_get", { input: { id: emailId } });
      if (em) {
        setPreviewEmail(em);
      } else {
        showToast("⚠️ Could not load email content");
      }
    } catch (e) {
      console.error("Failed to fetch email:", e);
      showToast("❌ Error loading email");
    } finally {
      setLoadingEmail(false);
    }
  };

  // Only display domains with count > 0 unless showEmptyDomains is active
  const visibleTaxonomy = taxonomy.filter(d => showEmptyDomains || d.total_count > 0);
  const totalAllArtifacts = visibleTaxonomy.reduce((acc, d) => acc + d.total_count, 0);

  const exportArtifactsCSV = () => {
    if (artifacts.length === 0) return;
    const headers = ["Domain", "Subcategory", "Title", "Primary Value", "Secondary Value", "Confidence", "Type", "Severity", "Subject", "From", "Date Sent"];
    const rows = artifacts.map(a => [
      `"${a.domain_id}"`,
      `"${a.subcategory_id}"`,
      `"${(a.title || "").replace(/"/g, '""')}"`,
      `"${(a.primary_value || "").replace(/"/g, '""')}"`,
      `"${(a.secondary_value || "").replace(/"/g, '""')}"`,
      `"${a.confidence || "high"}"`,
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
    link.setAttribute("download", `J12_Forensic_Artifacts_${caseId.slice(0, 8)}.csv`);
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

  const getConfidenceBadge = (c?: string | null) => {
    const conf = c || "high";
    if (conf === "high") {
      return <span className="badge" style={{ background: "rgba(34, 197, 94, 0.15)", color: "#22c55e", fontSize: 10 }}>✓ VALIDATED</span>;
    } else if (conf === "medium") {
      return <span className="badge" style={{ background: "rgba(234, 179, 8, 0.15)", color: "#eab308", fontSize: 10 }}>⚡ PATTERN</span>;
    }
    return <span className="badge" style={{ background: "rgba(148, 163, 184, 0.15)", color: "#94a3b8", fontSize: 10 }}>HEURISTIC</span>;
  };

  return (
    <div>
      {/* Toast Notification */}
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
            Forensic Artifact Taxonomy &amp; Intelligence Hub
          </h2>
          <p className="muted" style={{ margin: 0 }}>
            Structured evidence taxonomy — Credentials, Banking, Crypto Wallets, Contraband, Secrets, and Relays with False-Positive validation.
          </p>
        </div>
        <div className="row gap-2">
          <button className="btn btn-ghost btn-sm" onClick={exportArtifactsCSV} title="Export artifacts to CSV">
            📥 Export CSV
          </button>
          <button 
            className="btn btn-primary btn-sm" 
            onClick={handleRescan} 
            disabled={scanState.scanning}
            style={{ fontWeight: 600 }}
            title="Scan case emails and extract forensic taxonomy artifacts"
          >
            {scanState.scanning ? "⚡ Scanning..." : "⚡ Scan / Rescan Artifacts"}
          </button>
        </div>
      </div>

      {/* Scanning Progress Bar with Percentage and Stage Text */}
      {scanState.scanning && (
        <div className="card mb-4" style={{ padding: "14px 18px", background: "var(--bg-2)", border: "1px solid var(--accent)", boxShadow: "0 4px 20px rgba(0,0,0,0.25)" }}>
          <div className="row between mb-2">
            <span style={{ fontSize: 13, fontWeight: 600, color: "var(--text-0)" }}>
              ⚡ {scanState.stage || "Scanning and classifying forensic artifacts..."}
            </span>
            <span style={{ fontSize: 13, fontWeight: 700, color: "var(--accent)" }}>
              {scanState.progress}%
            </span>
          </div>
          <div style={{ width: "100%", height: 8, background: "var(--bg-0)", borderRadius: 4, overflow: "hidden" }}>
            <div
              style={{
                width: `${scanState.progress}%`,
                height: "100%",
                background: "linear-gradient(90deg, #3b82f6, #06b6d4, #10b981)",
                transition: "width 0.25s ease-in-out",
                borderRadius: 4,
              }}
            />
          </div>
        </div>
      )}

      {/* Main Two-Column Taxonomy Workspace */}
      <div style={{ display: "grid", gridTemplateColumns: "220px minmax(0, 1fr)", gap: 14, minWidth: 0, width: "100%" }}>
        
        {/* Left Column: Artifact Taxonomy Category Tree */}
        <div className="card" style={{ padding: 10, maxHeight: "calc(100vh - 160px)", overflowY: "auto", minWidth: 0 }}>
          <div className="row between mb-2" style={{ padding: "4px 6px" }}>
            <span style={{ fontSize: 10, fontWeight: 800, letterSpacing: "0.8px", color: "var(--text-3)" }}>
              AVAILABLE ARTIFACTS ({visibleTaxonomy.length})
            </span>
            <button 
              className="btn btn-ghost btn-sm"
              style={{ fontSize: 10, padding: "1px 6px", height: "auto" }}
              onClick={() => setShowEmptyDomains(!showEmptyDomains)}
              title={showEmptyDomains ? "Hide empty categories" : "Show all categories including 0"}
            >
              {showEmptyDomains ? "Hide 0s" : "Show All"}
            </button>
          </div>

          {/* All Artifacts Root */}
          <div 
            style={{
              display: "flex",
              alignItems: "center",
              justifyContent: "space-between",
              padding: "7px 8px",
              borderRadius: "var(--r-sm)",
              cursor: "pointer",
              marginBottom: 6,
              background: selectedDomain === "all" ? "var(--accent)" : "transparent",
              color: selectedDomain === "all" ? "#000" : "var(--text-0)",
              fontWeight: selectedDomain === "all" ? 700 : 500,
              fontSize: 12,
            }}
            onClick={() => { setSelectedDomain("all"); setSelectedSubcategory("all"); }}
          >
            <div className="row gap-1" style={{ alignItems: "center", overflow: "hidden" }}>
              <span>📁</span>
              <span style={{ overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>All Artifacts</span>
            </div>
            <span 
              className="badge" 
              style={{ 
                background: selectedDomain === "all" ? "#000" : "var(--bg-3)", 
                color: selectedDomain === "all" ? "#fff" : "var(--text-1)",
                fontSize: 10 
              }}
            >
              {totalAllArtifacts}
            </span>
          </div>

          {/* Domain List */}
          <div style={{ display: "flex", flexDirection: "column", gap: 2 }}>
            {visibleTaxonomy.length === 0 ? (
              <div className="muted text-xs p-3 text-center">No forensic artifacts detected in this case yet.</div>
            ) : (
              visibleTaxonomy.map((dom) => {
                const isDomainSelected = selectedDomain === dom.domain_id;
                return (
                  <div key={dom.domain_id} style={{ display: "flex", flexDirection: "column" }}>
                    <div
                      style={{
                        display: "flex",
                        alignItems: "center",
                        justifyContent: "space-between",
                        padding: "6px 8px",
                        borderRadius: "var(--r-sm)",
                        cursor: "pointer",
                        background: isDomainSelected && selectedSubcategory === "all" ? "var(--bg-3)" : "transparent",
                        color: isDomainSelected ? "var(--accent)" : "var(--text-1)",
                        fontWeight: isDomainSelected ? 700 : 500,
                        fontSize: 12,
                        borderLeft: isDomainSelected ? "3px solid var(--accent)" : "3px solid transparent",
                      }}
                      onClick={() => {
                        setSelectedDomain(dom.domain_id);
                        setSelectedSubcategory("all");
                      }}
                    >
                      <div className="row gap-1" style={{ alignItems: "center", overflow: "hidden", minWidth: 0 }}>
                        <span>{dom.icon}</span>
                        <span style={{ textOverflow: "ellipsis", whiteSpace: "nowrap", overflow: "hidden" }}>{dom.name}</span>
                      </div>
                      <span 
                        style={{ 
                          fontSize: 10.5, 
                          fontFamily: "var(--mono)",
                          color: isDomainSelected ? "var(--accent)" : "var(--text-3)",
                          fontWeight: 600,
                          flexShrink: 0
                        }}
                      >
                        {dom.total_count}
                      </span>
                    </div>

                    {/* Subcategories (if domain active) */}
                    {isDomainSelected && dom.subcategories.filter(s => showEmptyDomains || s.count > 0).length > 0 && (
                      <div style={{ display: "flex", flexDirection: "column", paddingLeft: 18, marginTop: 2, marginBottom: 4, gap: 2 }}>
                        {dom.subcategories.filter(s => showEmptyDomains || s.count > 0).map((sub) => {
                          const isSubSelected = selectedSubcategory === sub.subcategory_id;
                          return (
                            <div
                              key={sub.subcategory_id}
                              style={{
                                display: "flex",
                                alignItems: "center",
                                justifyContent: "space-between",
                                padding: "3px 6px",
                                borderRadius: "var(--r-sm)",
                                cursor: "pointer",
                                fontSize: 11,
                                color: isSubSelected ? "var(--accent)" : "var(--text-2)",
                                background: isSubSelected ? "rgba(56, 189, 248, 0.1)" : "transparent",
                                fontWeight: isSubSelected ? 700 : 400,
                              }}
                              onClick={(e) => {
                                e.stopPropagation();
                                setSelectedSubcategory(sub.subcategory_id);
                              }}
                            >
                              <span style={{ overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>↳ {sub.name}</span>
                              <span style={{ fontSize: 9.5, fontFamily: "var(--mono)", color: "var(--text-3)", flexShrink: 0 }}>
                                {sub.count}
                              </span>
                            </div>
                          );
                        })}
                      </div>
                    )}
                  </div>
                );
              })
            )}
          </div>
        </div>

        {/* Right Column: Artifacts Explorer & Live Inspector */}
        <div style={{ display: "flex", flexDirection: "column", gap: 12, minWidth: 0, overflow: "hidden" }}>
          
          {/* Filter Bar */}
          <div className="card" style={{ padding: "10px 14px", minWidth: 0 }}>
            <form onSubmit={handleSearchSubmit} className="row between gap-2" style={{ flexWrap: "wrap", minWidth: 0 }}>
              <div className="row gap-2" style={{ flex: 1, minWidth: 200 }}>
                <input
                  className="input"
                  style={{ flex: 1, padding: "7px 10px", fontSize: 12.5 }}
                  placeholder="Search artifacts (e.g. password, routing, credit card, anydesk)..."
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

              {/* Artifact Type Filter & Deduplication */}
              <div className="row gap-2" style={{ alignItems: "center", flexWrap: "wrap" }}>
                <button
                  type="button"
                  className={`btn btn-sm ${dedupUnique ? "btn-primary" : "btn-ghost"}`}
                  style={{ fontSize: 11, fontWeight: 600, padding: "5px 10px" }}
                  onClick={() => setDedupUnique(!dedupUnique)}
                  title="Collapse identical artifacts and display unique findings with occurrence counts"
                >
                  ⚡ {dedupUnique ? `Unique (${displayedArtifacts.length})` : `All Raw (${artifacts.length})`}
                </button>

                <span className="muted text-xs">TYPE:</span>
                <select 
                  className="input text-xs" 
                  style={{ padding: "5px 8px" }}
                  value={selectedArtifactType}
                  onChange={(e) => setSelectedArtifactType(e.target.value)}
                >
                  <option value="all">All Types</option>
                  <option value="native">📄 Native</option>
                  <option value="recovered">🗑️ Recovered</option>
                  <option value="derived">🧠 Derived</option>
                </select>
              </div>
            </form>
          </div>

          {/* Main Grid: Feed & Inspector */}
          <div style={{ display: "grid", gridTemplateColumns: selectedArtifact ? "minmax(0, 1fr) 330px" : "1fr", gap: 14, minWidth: 0 }}>
            
            {/* Artifacts Feed */}
            <div style={{ display: "flex", flexDirection: "column", gap: 8, minWidth: 0 }}>
              {loading ? (
                <div className="empty" style={{ padding: 40 }}>Classifying and indexing forensic taxonomy artifacts...</div>
              ) : displayedArtifacts.length === 0 ? (
                <div className="card empty" style={{ padding: 40 }}>
                  No artifacts found for the selected taxonomy domain or query.
                </div>
              ) : (
                displayedArtifacts.map((a) => {
                  const isSelected = selectedArtifact?.id === a.id;
                  return (
                    <div 
                      key={a.id}
                      className="card"
                      style={{
                        padding: "10px 14px",
                        margin: 0,
                        cursor: "pointer",
                        borderLeft: a.severity === "critical" ? "4px solid var(--danger)" : a.severity === "high" ? "4px solid var(--warning)" : a.severity === "medium" ? "4px solid var(--accent)" : "4px solid var(--border)",
                        background: isSelected ? "var(--bg-3)" : "var(--bg-2)",
                        transition: "all 0.15s ease",
                        minWidth: 0,
                        overflow: "hidden"
                      }}
                      onClick={() => setSelectedArtifact(a)}
                    >
                      <div className="row between mb-2" style={{ flexWrap: "wrap", gap: 6 }}>
                        <div className="row gap-2" style={{ alignItems: "center", flexWrap: "wrap", minWidth: 0 }}>
                          <span style={{ fontSize: 12.5, fontWeight: 700, color: "var(--text-0)" }}>{a.title}</span>
                          {getSeverityBadge(a.severity)}
                          {getTypeBadge(a.artifact_type)}
                          {getConfidenceBadge(a.confidence)}
                          {a.occurrenceCount && a.occurrenceCount > 1 && (
                            <span className="badge badge-blue" style={{ fontSize: 9.5 }}>
                              x{a.occurrenceCount}
                            </span>
                          )}
                        </div>
                        <span style={{ fontSize: 10.5, color: "var(--text-3)", fontFamily: "var(--mono)" }}>
                          {a.date_sent_utc ? new Date(a.date_sent_utc).toLocaleDateString() : ""}
                        </span>
                      </div>

                      {/* Highlighted Extracted Value Box */}
                      <div 
                        style={{
                          background: "rgba(15, 23, 42, 0.9)",
                          border: "1px solid var(--border)",
                          borderRadius: "var(--r-sm)",
                          padding: "7px 10px",
                          fontFamily: "var(--mono)",
                          fontSize: 12,
                          color: a.domain_id === "credentials" ? "#f43f5e" : a.domain_id === "financial" ? "#22c55e" : a.domain_id === "crypto" ? "#eab308" : a.domain_id === "contraband" ? "#ef4444" : "#38bdf8",
                          marginBottom: 6,
                          display: "flex",
                          alignItems: "center",
                          justifyContent: "space-between",
                          gap: 8,
                          wordBreak: "break-all",
                          overflow: "hidden"
                        }}
                      >
                        <span style={{ overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap", flex: 1 }}>
                          {a.primary_value}
                        </span>
                        <button 
                          className="btn btn-ghost btn-sm" 
                          style={{ padding: "1px 6px", fontSize: 10, height: "auto", flexShrink: 0 }}
                          onClick={(e) => {
                            e.stopPropagation();
                            copyToClipboard(a.primary_value);
                          }}
                          title="Copy extracted value"
                        >
                          📋 Copy
                        </button>
                      </div>

                      {/* Context & Source Row */}
                      <div className="row between" style={{ fontSize: 11, color: "var(--text-3)", minWidth: 0 }}>
                        <div style={{ overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap", flex: 1 }}>
                          Source: <strong style={{ color: "var(--text-2)" }}>{a.email_from}</strong>
                          {a.email_subject && ` · Subject: ${a.email_subject}`}
                        </div>
                        {a.email_id && (
                          <button 
                            className="btn btn-ghost btn-sm" 
                            style={{ padding: "1px 6px", fontSize: 10, height: "auto", marginLeft: 8, flexShrink: 0 }}
                            onClick={(e) => {
                              e.stopPropagation();
                              openEmailModal(a.email_id);
                            }}
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
              <div className="card" style={{ position: "sticky", top: 16, height: "fit-content", padding: 16, minWidth: 0, overflow: "hidden" }}>
                <div className="row between mb-3">
                  <h3 style={{ fontSize: 14, fontWeight: 700, color: "var(--text-0)", margin: 0 }}>
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

                {selectedArtifact.email_id ? (
                  <>
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

                    <button 
                      className="btn btn-primary btn-sm" 
                      style={{ width: "100%" }}
                      onClick={() => openEmailModal(selectedArtifact.email_id)}
                    >
                      ✉️ Open Email in Forensic Viewer
                    </button>
                  </>
                ) : null}
              </div>
            )}
            {previewEmail && (
              <EmailDetailModal
                email={previewEmail}
                onClose={() => setPreviewEmail(null)}
                titleSuffix="Return to Artifacts"
              />
            )}
          </div>
        </div>
      </div>
    </div>
  );
}
