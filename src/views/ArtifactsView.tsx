import { useState, useEffect, useMemo } from "react";
import { invoke, Channel } from "@tauri-apps/api/core";
import { EmailDetailModal } from "../components/EmailDetailModal";
import { useScanState, scanStore } from "../utils/scanState";
import {
  TaxonomyDomainSummary,
  ForensicTaxonomyArtifact,
  EmailMessage,
  ArtifactsProps,
} from "./artifacts/types";
import { ArtifactsCategoryTree } from "./artifacts/ArtifactsCategoryTree";
import { ArtifactsFeed } from "./artifacts/ArtifactsFeed";
import { ArtifactInspectorDrawer } from "./artifacts/ArtifactInspectorDrawer";

export function ArtifactsView({ caseId, evidenceFilter }: ArtifactsProps) {
  const [taxonomy, setTaxonomy] = useState<TaxonomyDomainSummary[]>([]);
  const [artifacts, setArtifacts] = useState<ForensicTaxonomyArtifact[]>([]);
  const [selectedDomain, setSelectedDomain] = useState<string>("all");
  const [selectedSubcategory, setSelectedSubcategory] = useState<string>("all");
  const [selectedArtifactType, setSelectedArtifactType] = useState<string>("all");
  const [search, setSearch] = useState<string>("");
  const [showEmptyDomains, setShowEmptyDomains] = useState<boolean>(false);
  const [dedupUnique, setDedupUnique] = useState<boolean>(true);
  const [loading, setLoading] = useState<boolean>(true);
  const [scanState] = useScanState();
  const [selectedArtifact, setSelectedArtifact] = useState<ForensicTaxonomyArtifact | null>(null);
  const [previewEmail, setPreviewEmail] = useState<EmailMessage | null>(null);
  const [toastMessage, setToastMessage] = useState<string | null>(null);

  const showToast = (msg: string) => {
    setToastMessage(msg);
    setTimeout(() => setToastMessage(null), 3000);
  };

  useEffect(() => {
    loadTaxonomy();
  }, [caseId, evidenceFilter, showEmptyDomains]);

  useEffect(() => {
    loadArtifacts();
  }, [caseId, evidenceFilter, selectedDomain, selectedSubcategory, selectedArtifactType]);

  const loadTaxonomy = async () => {
    try {
      const domains = await invoke<TaxonomyDomainSummary[]>("case_artifacts_summary", { 
        input: { 
          case_id: caseId,
          evidence_id: evidenceFilter || undefined,
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
          evidence_id: evidenceFilter || undefined,
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
    scanStore.setState({
      scanning: true,
      progress: 2,
      stage: "Initializing parallel forensic taxonomy scanner...",
    });

    const onProgress = new Channel<any>();
    onProgress.onmessage = (msg: any) => {
      if (msg) {
        scanStore.setState({
          scanning: msg.scanning !== false,
          progress: typeof msg.percent === "number" ? msg.percent : 0,
          stage: msg.stage || "Scanning message corpora...",
        });
      }
    };

    try {
      const count = await invoke<number>("rescan_case_artifacts", { 
        input: { case_id: caseId },
        onEvent: onProgress
      });
      scanStore.setState({
        progress: 100,
        stage: `Completed! Indexed ${count} forensic artifacts.`,
      });
      showToast(`✓ Scanned and indexed ${count} artifacts`);
      await Promise.all([loadTaxonomy(), loadArtifacts()]);
    } catch (e: any) {
      console.error("Error scanning artifacts:", e);
      showToast(`❌ Error scanning artifacts: ${e}`);
    } finally {
      setTimeout(() => {
        scanStore.setState({
          scanning: false,
          progress: 0,
          stage: "",
        });
      }, 1200);
    }
  };

  const copyToClipboard = (text: string) => {
    navigator.clipboard.writeText(text);
    showToast("✓ Copied value to clipboard");
  };

  const openEmailModal = async (emailId: string) => {
    if (!emailId) return;
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
    }
  };

  const visibleTaxonomy = useMemo(() => {
    return taxonomy
      .filter(d => showEmptyDomains || d.total_count > 0)
      .slice()
      .sort((a, b) => a.name.localeCompare(b.name, undefined, { sensitivity: "base" }));
  }, [taxonomy, showEmptyDomains]);

  const totalAllArtifacts = useMemo(() => {
    return visibleTaxonomy.reduce((acc, d) => acc + d.total_count, 0);
  }, [visibleTaxonomy]);

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

  return (
    <div>
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

      {/* Scanning Progress Bar */}
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

      {/* Main Two-Column Workspace */}
      <div style={{ display: "grid", gridTemplateColumns: "220px minmax(0, 1fr)", gap: 14, minWidth: 0, width: "100%" }}>
        {/* Left Category Tree */}
        <ArtifactsCategoryTree
          visibleTaxonomy={visibleTaxonomy}
          totalAllArtifacts={totalAllArtifacts}
          selectedDomain={selectedDomain}
          selectedSubcategory={selectedSubcategory}
          showEmptyDomains={showEmptyDomains}
          setShowEmptyDomains={setShowEmptyDomains}
          onSelectDomain={(d) => { setSelectedDomain(d); setSelectedSubcategory("all"); }}
          onSelectSubcategory={setSelectedSubcategory}
        />

        {/* Right Column: Feed & Inspector */}
        <div style={{ display: "flex", flexDirection: "column", gap: 12, minWidth: 0, overflow: "hidden" }}>
          {/* Filter Bar */}
          <div className="card" style={{ padding: "10px 14px", minWidth: 0 }}>
            <form onSubmit={(e) => { e.preventDefault(); loadArtifacts(); }} className="row between gap-2" style={{ flexWrap: "wrap", minWidth: 0 }}>
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

              <div className="row gap-2" style={{ alignItems: "center", flexWrap: "wrap" }}>
                <button
                  type="button"
                  className={`btn btn-sm ${dedupUnique ? "btn-primary" : "btn-ghost"}`}
                  style={{ fontSize: 11, fontWeight: 600, padding: "5px 10px" }}
                  onClick={() => setDedupUnique(!dedupUnique)}
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

          <div style={{ display: "grid", gridTemplateColumns: selectedArtifact ? "minmax(0, 1fr) 330px" : "1fr", gap: 14, minWidth: 0 }}>
            <ArtifactsFeed
              caseId={caseId}
              displayedArtifacts={displayedArtifacts}
              selectedArtifact={selectedArtifact}
              loading={loading}
              onSelectArtifact={setSelectedArtifact}
              onCopyToClipboard={copyToClipboard}
              onOpenEmailModal={openEmailModal}
            />

            {selectedArtifact && (
              <ArtifactInspectorDrawer
                caseId={caseId}
                selectedArtifact={selectedArtifact}
                onClose={() => setSelectedArtifact(null)}
                onCopyToClipboard={copyToClipboard}
                onOpenEmailModal={openEmailModal}
              />
            )}
          </div>
        </div>
      </div>

      {previewEmail && (
        <EmailDetailModal
          email={previewEmail}
          onClose={() => setPreviewEmail(null)}
          titleSuffix="Return to Artifacts"
        />
      )}
    </div>
  );
}
