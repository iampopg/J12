import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Dashboard, Evidence, Case, View } from "./types";
import { DashboardHeader } from "./dashboard/DashboardHeader";
import { DashboardKpis } from "./dashboard/DashboardKpis";
import { DashboardThreatRadar } from "./dashboard/DashboardThreatRadar";
import { DashboardFolderTally } from "./dashboard/DashboardFolderTally";
import { DashboardEvidenceLedger } from "./dashboard/DashboardEvidenceLedger";

interface Props {
  data: Dashboard;
  evidence: Evidence[];
  caseData: Case | null;
  caseId: string;
  onNavigate: (view: View) => void;
  onRefresh: () => void;
}

export function DashboardView({
  data,
  evidence,
  caseData,
  caseId,
  onNavigate,
  onRefresh,
}: Props) {
  const [criticalFindings, setCriticalFindings] = useState<any[]>([]);
  const [targetPartners, setTargetPartners] = useState<any[]>([]);
  const [analyzing, setAnalyzing] = useState(false);

  useEffect(() => {
    invoke<any[]>("findings_list", { input: { case_id: caseId } })
      .then((res) => {
        const critical = (res || []).filter((f) => f.severity === "critical" || f.severity === "high");
        setCriticalFindings(critical.slice(0, 4));
      })
      .catch(() => setCriticalFindings([]));

    if (caseData?.target_email) {
      invoke<any>("entity_dive", { input: { case_id: caseId, email: caseData.target_email } })
        .then((res) => {
          if (res?.top_sent_to || res?.top_received_from) {
            const combined = [...(res.top_sent_to || []), ...(res.top_received_from || [])];
            setTargetPartners(combined.slice(0, 4));
          }
        })
        .catch(() => setTargetPartners([]));
    }
  }, [caseId, caseData?.target_email]);

  const totalFindings =
    (data.severity_breakdown?.critical || 0) +
    (data.severity_breakdown?.high || 0) +
    (data.severity_breakdown?.medium || 0) +
    (data.severity_breakdown?.low || 0);

  const handleRunAnalysis = async () => {
    setAnalyzing(true);
    try {
      await invoke("run_analysis", { input: { case_id: caseId } });
      onRefresh();
    } catch (e) {
      console.error("Analysis failed:", e);
    } finally {
      setAnalyzing(false);
    }
  };

  return (
    <div style={{ maxWidth: 1400, margin: "0 auto", paddingBottom: 32 }}>
      {/* 1. Top HUD Command Header */}
      <DashboardHeader
        caseData={caseData}
        analyzing={analyzing}
        onRunAnalysis={handleRunAnalysis}
        onRefresh={onRefresh}
        onNavigate={onNavigate}
      />

      {/* 2. Interactive 5-Card Metric KPIs */}
      <DashboardKpis
        data={data}
        totalFindings={totalFindings}
        onNavigate={onNavigate}
      />

      {/* 3. Investigation Target Dossier & Active Threat Radar */}
      <DashboardThreatRadar
        caseData={caseData}
        evidence={evidence}
        targetPartners={
          targetPartners.length > 0
            ? targetPartners
            : (data.top_correspondents || []).map((c) => ({
                email: c.email,
                display_name: c.email,
                count: c.sent || c.received || 1,
              }))
        }
        criticalFindings={criticalFindings}
        totalFindings={totalFindings}
        onNavigate={onNavigate}
      />

      {/* 4. Folder Taxonomy & Threat Severity Spectrum */}
      <DashboardFolderTally
        data={data}
        totalFindings={totalFindings}
        onNavigate={onNavigate}
      />

      {/* 5. Cryptographic Evidence Containers & Provenance Ledger */}
      <DashboardEvidenceLedger
        evidence={evidence}
        onNavigate={onNavigate}
      />
    </div>
  );
}
