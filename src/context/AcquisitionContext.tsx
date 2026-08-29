import React, { createContext, useContext, useEffect, useState, useRef, useCallback } from "react";
import { invoke, Channel } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

export interface AcquisitionProgress {
  folder?: string;
  folderIndex?: number;
  totalFolders?: number;
  msgSeq?: number;
  folderTotal?: number;
  overallSeq?: number;
  overallTotal?: number;
  ingested?: number;
  duplicatesSkipped?: number;
  subject?: string;
  from?: string;
}

export type PipelineStep = "idle" | "ingesting" | "artifacts" | "analysis" | "complete" | "error";

export interface AcquisitionPreferences {
  chunkSize: number; // 25, 50, 100, 250
  autoExtractArtifacts: boolean;
  autoRunAnalysis: boolean;
  deduplicationMode: "message_id" | "body_hash" | "strict";
}

interface AcquisitionContextType {
  isAcquiring: boolean;
  pipelineStep: PipelineStep;
  activeCaseId: string | null;
  account: string;
  protocol: "imap" | "pop3";
  status: string;
  progress: AcquisitionProgress | null;
  percent: number;
  logs: string[];
  result: any | null;
  error: string | null;
  preferences: AcquisitionPreferences;
  setPreferences: (prefs: Partial<AcquisitionPreferences>) => void;
  startAcquisition: (params: {
    caseId: string;
    protocol: "imap" | "pop3";
    server: string;
    port: number;
    username: string;
    password: string;
    authType?: "password" | "oauth2";
    accessToken?: string;
    useSsl: boolean;
    mailboxScope: string;
    maxMessages?: number | null;
    onPipelineComplete?: () => void;
  }) => Promise<any>;
  stopAcquisition: () => Promise<void>;
  runFullPostIngestPipeline: (caseId: string) => Promise<void>;
  clearLogs: () => void;
}

const defaultPreferences: AcquisitionPreferences = {
  chunkSize: 50,
  autoExtractArtifacts: true,
  autoRunAnalysis: true,
  deduplicationMode: "message_id",
};

const AcquisitionContext = createContext<AcquisitionContextType | null>(null);

export const AcquisitionProvider: React.FC<{ children: React.ReactNode }> = ({ children }) => {
  const [isAcquiring, setIsAcquiring] = useState(false);
  const [pipelineStep, setPipelineStep] = useState<PipelineStep>("idle");
  const [activeCaseId, setActiveCaseId] = useState<string | null>(null);
  const [account, setAccount] = useState("");
  const [protocol, setProtocol] = useState<"imap" | "pop3">("imap");
  const [status, setStatus] = useState("idle");
  const [progress, setProgress] = useState<AcquisitionProgress | null>(null);
  const [percent, setPercent] = useState(0);
  const [logs, setLogs] = useState<string[]>([]);
  const [result, setResult] = useState<any | null>(null);
  const [error, setError] = useState<string | null>(null);

  const [preferences, setPreferencesState] = useState<AcquisitionPreferences>(() => {
    try {
      const saved = localStorage.getItem("j12_acquisition_prefs");
      return saved ? { ...defaultPreferences, ...JSON.parse(saved) } : defaultPreferences;
    } catch {
      return defaultPreferences;
    }
  });

  const setPreferences = useCallback((newPrefs: Partial<AcquisitionPreferences>) => {
    setPreferencesState(prev => {
      const updated = { ...prev, ...newPrefs };
      localStorage.setItem("j12_acquisition_prefs", JSON.stringify(updated));
      return updated;
    });
  }, []);

  const addLog = useCallback((msg: string) => {
    const timestamp = new Date().toLocaleTimeString();
    const entry = `[${timestamp}] ${msg}`;
    setLogs(prev => {
      if (prev.length > 0 && prev[prev.length - 1] === entry) return prev;
      return [...prev.slice(-499), entry];
    });
  }, []);

  const clearLogs = useCallback(() => {
    setLogs([]);
  }, []);

  // Run automated post-ingestion pipeline: Artifact Extraction -> Threat Analysis -> Entities
  const runFullPostIngestPipeline = useCallback(async (caseId: string) => {
    if (!caseId) return;
    try {
      setPipelineStep("artifacts");
      addLog("🚀 Auto-Pipeline Initiated: Step 1/3 — Extracting Forensic Artifacts (Taxonomy, Cryptos, IPs, Identifiers)...");
      try {
        const onArtEvent = new Channel<any>();
        onArtEvent.onmessage = (msg: any) => {
          if (msg && msg.stage && (msg.percent === 5 || msg.percent === 50 || msg.percent === 90 || msg.percent === 100)) {
            addLog(`⚡ [Artifacts ${msg.percent}%] ${msg.stage}`);
          }
        };
        const artCount = await invoke<number>("rescan_case_artifacts", { 
          input: { case_id: caseId },
          onEvent: onArtEvent
        });
        addLog(`✓ Forensic Artifact Extraction Complete: ${artCount} taxonomy artifacts classified.`);
      } catch (e: any) {
        addLog(`⚠ Artifact extraction notice: ${e}`);
      }

      setPipelineStep("analysis");
      addLog("🔍 Auto-Pipeline Step 2/3: Executing Security Risk Analysis, Authentication Checks & Spoofing Detection...");
      try {
        const findCount = await invoke<number>("run_analysis", { input: { case_id: caseId } });
        addLog(`✓ Forensic Intelligence Analysis Complete: ${findCount} findings generated & risk-scored.`);
      } catch (e: any) {
        addLog(`⚠ Analysis notice: ${e}`);
      }

      addLog("👥 Auto-Pipeline Step 3/3: Indexing Target Dossier Entities & Communication Network Graph...");
      try {
        await invoke<number>("extract_entities", { input: { case_id: caseId } });
        addLog("✓ Entity Resolution & Graph Indexing Synchronized.");
      } catch (e: any) {
        addLog(`⚠ Entity indexing notice: ${e}`);
      }

      setPipelineStep("complete");
      addLog("🎉 Comprehensive Forensic Pipeline Complete: All emails, attachments, artifacts, and findings are ready for investigation!");
    } catch (e: any) {
      setPipelineStep("error");
      addLog(`✗ Pipeline error: ${e}`);
    }
  }, [addLog]);

  // Handle incoming progress payloads from Tauri backend (Tauri events or IPC Channel)
  const handleProgressPayload = useCallback((p: any) => {
    if (!p) return;

    if (p.log) {
      addLog(p.log);
    }

    if (p.status) {
      setStatus(p.status);
    }

    if (p.status === "ingested" || p.status === "folder_discovered" || p.status === "duplicate_skipped") {
      setProgress(prev => {
        const updated: AcquisitionProgress = {
          ...prev,
          folder: p.folder || prev?.folder,
          folderIndex: p.folder_index !== undefined ? p.folder_index : prev?.folderIndex,
          totalFolders: p.total_folders !== undefined ? p.total_folders : prev?.totalFolders,
          msgSeq: p.msg_seq !== undefined ? p.msg_seq : prev?.msgSeq,
          folderTotal: p.folder_total !== undefined ? p.folder_total : prev?.folderTotal,
          overallSeq: p.overall_seq !== undefined ? p.overall_seq : prev?.overallSeq,
          overallTotal: p.overall_total !== undefined ? p.overall_total : prev?.overallTotal,
          ingested: p.ingested_count !== undefined ? p.ingested_count : prev?.ingested,
          duplicatesSkipped: p.duplicates_skipped !== undefined ? p.duplicates_skipped : prev?.duplicatesSkipped,
          subject: p.subject !== undefined ? p.subject : prev?.subject,
          from: p.from !== undefined ? p.from : prev?.from,
        };

        // Calculate accurate overall percentage
        let calcPercent = 0;
        if (updated.overallTotal && updated.overallTotal > 0 && updated.overallSeq !== undefined) {
          calcPercent = Math.min(100, Math.round((updated.overallSeq / updated.overallTotal) * 100));
        } else if (updated.folderTotal && updated.folderTotal > 0 && updated.msgSeq !== undefined) {
          calcPercent = Math.min(100, Math.round((updated.msgSeq / updated.folderTotal) * 100));
        }
        setPercent(calcPercent);

        return updated;
      });
    }

    if (p.status === "done" || p.status === "cancelled") {
      setIsAcquiring(false);
      setResult(p);
      setPercent(100);
    }
  }, [addLog]);

  // Global listener for Tauri progress events (persists across all unmounts / view changes)
  useEffect(() => {
    let unlistenImap: (() => void) | null = null;
    let unlistenPop3: (() => void) | null = null;
    let isMounted = true;

    listen("imap_progress", (event) => {
      if (isMounted) handleProgressPayload(event.payload);
    })
      .then(unlisten => {
        if (isMounted) {
          unlistenImap = unlisten;
        } else {
          unlisten();
        }
      })
      .catch(err => {
        console.debug("Tauri imap_progress listener not registered:", err);
      });

    listen("pop3_progress", (event) => {
      if (isMounted) handleProgressPayload(event.payload);
    })
      .then(unlisten => {
        if (isMounted) {
          unlistenPop3 = unlisten;
        } else {
          unlisten();
        }
      })
      .catch(err => {
        console.debug("Tauri pop3_progress listener not registered:", err);
      });

    return () => {
      isMounted = false;
      unlistenImap?.();
      unlistenPop3?.();
    };
  }, [handleProgressPayload]);

  const startAcquisition = useCallback(async ({
    caseId,
    protocol: proto,
    server,
    port,
    username,
    password,
    authType = "password",
    accessToken,
    useSsl,
    mailboxScope,
    maxMessages = null,
    onPipelineComplete,
  }: {
    caseId: string;
    protocol: "imap" | "pop3";
    server: string;
    port: number;
    username: string;
    password: string;
    authType?: "password" | "oauth2";
    accessToken?: string;
    useSsl: boolean;
    mailboxScope: string;
    maxMessages?: number | null;
    onPipelineComplete?: () => void;
  }) => {
    setIsAcquiring(true);
    setPipelineStep("ingesting");
    setActiveCaseId(caseId);
    setAccount(username);
    setProtocol(proto);
    setStatus("starting");
    setProgress(null);
    setPercent(0);
    setLogs([]);
    setResult(null);
    setError(null);

    addLog(`🚀 Starting live forensic stream for account: ${username}...`);
    addLog(`Protocol: ${proto.toUpperCase()} | Server: ${server}:${port} (Auth: ${authType}, SSL/TLS: ${useSsl ? "YES" : "NO"})`);
    addLog(`Scope: ${mailboxScope === "ALL" ? "Entire Account (All Mailboxes)" : mailboxScope}`);

    const onEvent = new Channel<any>();
    onEvent.onmessage = (payload) => {
      handleProgressPayload(payload);
    };

    try {
      let res: any;
      if (proto === "imap") {
        res = await invoke<any>("imap_fetch_emails", {
          input: {
            case_id: caseId,
            caseId,
            evidence_id: `imap_${caseId}_${username.replace(/[^a-zA-Z0-9]/g, "_")}`,
            evidenceId: `imap_${caseId}_${username.replace(/[^a-zA-Z0-9]/g, "_")}`,
            server: server.trim(),
            port,
            username: username.trim(),
            password: password.trim(),
            auth_type: authType,
            access_token: accessToken,
            use_ssl: useSsl,
            useSsl,
            mailbox: mailboxScope,
            max_messages: maxMessages,
          },
          on_event: onEvent,
          onEvent,
        });
      } else {
        res = await invoke<any>("pop3_fetch_emails", {
          input: {
            case_id: caseId,
            caseId,
            evidence_id: `pop3_${caseId}_${username.replace(/[^a-zA-Z0-9]/g, "_")}`,
            evidenceId: `pop3_${caseId}_${username.replace(/[^a-zA-Z0-9]/g, "_")}`,
            server: server.trim(),
            port,
            username: username.trim(),
            password: password.trim(),
            auth_type: authType,
            access_token: accessToken,
            use_ssl: useSsl,
            useSsl,
            max_messages: maxMessages,
          },
          on_event: onEvent,
          onEvent,
        });
      }

      setResult(res);
      addLog(`✓ Ingestion Complete: ${res.downloaded || 0} emails ingested (${res.duplicates_skipped || 0} duplicates skipped).`);

      // Trigger automatic post-ingest pipeline if enabled in preferences
      if (preferences.autoExtractArtifacts || preferences.autoRunAnalysis) {
        await runFullPostIngestPipeline(caseId);
      } else {
        setPipelineStep("complete");
      }

      onPipelineComplete?.();
      return res;
    } catch (e: any) {
      setError(String(e));
      setStatus("error");
      setPipelineStep("error");
      addLog(`✗ Acquisition failed: ${e}`);
      throw e;
    } finally {
      setIsAcquiring(false);
    }
  }, [addLog, handleProgressPayload, preferences, runFullPostIngestPipeline]);

  const stopAcquisition = useCallback(async () => {
    try {
      addLog("⏹ Stop requested by investigator. Wrapping up current message and finalizing database records...");
      await invoke("imap_cancel_acquisition");
      setStatus("stopping");
    } catch (e: any) {
      addLog(`Error stopping acquisition: ${e}`);
    }
  }, [addLog]);

  return (
    <AcquisitionContext.Provider
      value={{
        isAcquiring,
        pipelineStep,
        activeCaseId,
        account,
        protocol,
        status,
        progress,
        percent,
        logs,
        result,
        error,
        preferences,
        setPreferences,
        startAcquisition,
        stopAcquisition,
        runFullPostIngestPipeline,
        clearLogs,
      }}
    >
      {children}
    </AcquisitionContext.Provider>
  );
};

export const useAcquisition = () => {
  const ctx = useContext(AcquisitionContext);
  if (!ctx) {
    throw new Error("useAcquisition must be used within an AcquisitionProvider");
  }
  return ctx;
};
