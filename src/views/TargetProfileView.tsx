import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  TargetProfile,
  DetectedTarget,
  TargetProfileProps,
  formatName,
} from "./target_profile/types";
import { TargetBanner } from "./target_profile/TargetBanner";
import { TargetKPIs } from "./target_profile/TargetKPIs";
import { TargetCandidates } from "./target_profile/TargetCandidates";
import { TargetTelemetry } from "./target_profile/TargetTelemetry";
import { TargetCommunications } from "./target_profile/TargetCommunications";

export function TargetProfileView({ caseId, caseData, evidenceFilter, onSelectEmail }: TargetProfileProps) {
  const [profile, setProfile] = useState<TargetProfile | null>(null);
  const [detected, setDetected] = useState<DetectedTarget[]>([]);
  const [loading, setLoading] = useState(true);
  const [selectedEmail, setSelectedEmail] = useState<string | null>(null);
  const [toastMessage, setToastMessage] = useState<string | null>(null);

  const showToast = (msg: string) => {
    setToastMessage(msg);
    setTimeout(() => setToastMessage(null), 3500);
  };

  useEffect(() => {
    setSelectedEmail(null);
    loadData(null);
  }, [caseId, evidenceFilter]);

  const loadData = async (overrideEmail?: string | null) => {
    setLoading(true);
    try {
      const det = await invoke<any>("auto_detect_targets", { input: { case_id: caseId, evidence_id: evidenceFilter || undefined } });
      const targets: DetectedTarget[] = det.targets || det.candidates || [];
      setDetected(targets);

      let targetToLoad: string | null = null;
      if (overrideEmail) {
        targetToLoad = overrideEmail;
      } else if (targets.length > 0) {
        targetToLoad = targets[0].email;
      } else if (caseData?.target_email) {
        targetToLoad = caseData.target_email;
      }

      if (targetToLoad) {
        setSelectedEmail(targetToLoad);
        await loadProfile(targetToLoad);
      } else {
        setProfile(null);
      }
    } catch (e) {
      console.error("Failed to load target data:", e);
    } finally {
      setLoading(false);
    }
  };

  const loadProfile = async (email: string) => {
    try {
      const prof = await invoke<TargetProfile>("target_profile", {
        input: { case_id: caseId, target_email: email, evidence_id: evidenceFilter || undefined }
      });
      setProfile(prof);
    } catch (e) {
      console.error("Failed to load profile:", e);
    }
  };

  const reExtract = async () => {
    setLoading(true);
    try {
      const count = await invoke<number>("extract_entities", { input: { case_id: caseId } });
      showToast(`⚡ Re-extracted and unified ${count} entities across case`);
      await loadData(selectedEmail);
    } catch (e) {
      console.error(e);
    } finally {
      setLoading(false);
    }
  };

  const handleSelectCandidate = (email: string) => {
    setSelectedEmail(email);
    loadProfile(email);
  };

  const currentEmail = selectedEmail || profile?.target_email || caseData?.target_email || "";
  const currentName = profile?.target_name || (selectedEmail === caseData?.target_email ? caseData?.target_name : null) || formatName(null, currentEmail);
  const currentOrg = profile?.target_organization || (selectedEmail === caseData?.target_email ? caseData?.target_organization : null) || (currentEmail.includes("@") ? currentEmail.split("@")[1] : "N/A");

  if (loading && !profile) {
    return <div className="empty" style={{ padding: 40 }}>Loading Target Subject Dossier...</div>;
  }

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
            padding: "12px 20px",
            fontWeight: 600,
            fontSize: 13,
            boxShadow: "0 10px 25px rgba(0,0,0,0.5)",
            display: "flex",
            alignItems: "center",
            gap: 10,
          }}
        >
          <span>✓</span>
          <span>{toastMessage}</span>
        </div>
      )}

      {/* Top Header */}
      <div className="row between mb-3">
        <div>
          <h2 style={{ fontSize: 22, fontWeight: 700, color: "var(--text-0)" }}>Target Subject Dossier</h2>
          <p className="muted" style={{ fontSize: 13 }}>
            Principal Person of Interest Intelligence &amp; Communication Topology
          </p>
        </div>
        <div className="row gap-2">
          <button className="btn btn-ghost btn-sm" onClick={reExtract} title="Re-scan and normalize all entities">
            ⚡ Re-Extract Entities
          </button>
          <button className="btn btn-ghost btn-sm" onClick={() => loadData(selectedEmail)}>
            ↻ Refresh
          </button>
        </div>
      </div>

      {/* Target / Subject Switcher Bar */}
      {detected.length > 0 && (
        <div
          className="card mb-4"
          style={{
            padding: "8px 14px",
            display: "flex",
            alignItems: "center",
            justifyContent: "space-between",
            flexWrap: "wrap",
            gap: 10,
            background: "var(--bg-2)",
            border: "1px solid var(--border)",
          }}
        >
          <div style={{ display: "flex", alignItems: "center", gap: 8 }}>
            <span style={{ fontSize: 10, fontWeight: 700, color: "var(--text-3)", letterSpacing: "0.05em" }}>
              ACTIVE SUBJECT:
            </span>
            <select
              className="input"
              style={{
                fontSize: 12,
                padding: "3px 10px",
                height: 28,
                minWidth: 260,
                maxWidth: 420,
                fontWeight: 600,
                background: "var(--bg-3)",
                color: "var(--text-0)",
                borderColor: "var(--border)",
                cursor: "pointer",
              }}
              value={selectedEmail || profile?.target_email || ""}
              onChange={(e) => handleSelectCandidate(e.target.value)}
            >
              {detected.map((t) => (
                <option key={t.email} value={t.email}>
                  {t.is_primary_target || t.is_custodian ? "🎯 [CUSTODIAN] " : t.is_automated ? "🤖 [BOT] " : "👤 "}
                  {t.display_name || t.email} ({t.email}) — {t.total_emails} msgs ({t.sent} sent, {t.received} recvd) {t.role ? `· ${t.role}` : ""}
                </option>
              ))}
            </select>
          </div>
          <div style={{ fontSize: 11, color: "var(--text-3)" }}>
            Found <strong>{detected.length}</strong> candidate subjects in active scope
          </div>
        </div>
      )}

      {/* Main Identity Banner Card */}
      <TargetBanner
        profile={profile}
        detected={detected}
        caseData={caseData}
        currentEmail={currentEmail}
        currentName={currentName}
        currentOrg={currentOrg}
        selectedEmail={selectedEmail}
      />

      {/* KPI Stats Grid */}
      <TargetKPIs profile={profile} />

      {/* Candidate Persons of Interest Selector */}
      <TargetCandidates
        detected={detected}
        currentEmail={currentEmail}
        onSelectCandidate={handleSelectCandidate}
      />

      {/* Forensic Telemetry Grid (Span, Mailers, IPs, Top Correspondents, Top Subjects) */}
      <TargetTelemetry
        profile={profile}
        onSelectCandidate={handleSelectCandidate}
      />

      {/* Recent Communications Stream */}
      <TargetCommunications
        profile={profile}
        onSelectEmail={onSelectEmail}
      />
    </div>
  );
}
