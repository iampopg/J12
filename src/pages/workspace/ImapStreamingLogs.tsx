import { RefObject } from "react";
import { invoke } from "@tauri-apps/api/core";

interface Props {
  combinedLogs: string[];
  clearLogs: () => void;
  caseId: string;
  logsEndRef: React.RefObject<HTMLDivElement>;
}

export function ImapStreamingLogs({ combinedLogs, clearLogs, caseId, logsEndRef }: Props) {
  if (combinedLogs.length === 0) return null;

  return (
    <div className="card" style={{ maxHeight: 260, overflowY: "auto", background: "#0b0f19", border: "1px solid #1e293b", padding: 14 }}>
      <div className="row between mb-2">
        <h4 style={{ fontSize: 12, fontWeight: 700, color: "#94a3b8", letterSpacing: "0.05em", margin: 0 }}>
          📡 LIVE FORENSIC ACQUISITION AUDIT STREAM
        </h4>
        <div className="row gap-2">
          <button 
            className="btn btn-ghost btn-sm" 
            style={{ fontSize: 10, padding: "2px 6px" }}
            onClick={clearLogs}
          >
            Clear Stream
          </button>
          <button 
            className="btn btn-ghost btn-sm" 
            style={{ fontSize: 10, padding: "2px 6px" }}
            onClick={async () => {
              try {
                await invoke("open_forensic_logs_folder", { case_id: caseId });
              } catch (e) {
                console.error("Failed to open log folder:", e);
              }
            }}
          >
            📁 Open Disk Log Folder
          </button>
          <span style={{ fontSize: 10, color: "#64748b" }}>{combinedLogs.length} EVENTS</span>
        </div>
      </div>
      {combinedLogs.map((log, i) => {
        const isSuccess = log.includes("✓") || log.includes("Ingested") || log.includes("Complete");
        const isError = log.includes("✗") || log.includes("Error") || log.includes("failed");
        const isSkip = log.includes("Skipped") || log.includes("duplicate");
        return (
          <div 
            key={i} 
            style={{ 
              fontSize: 11, 
              fontFamily: "var(--mono)", 
              marginBottom: 3, 
              lineHeight: 1.4,
              color: isSuccess ? "#4ade80" : isError ? "#f87171" : isSkip ? "#38bdf8" : "#cbd5e1" 
            }}
          >
            {log}
          </div>
        );
      })}
      <div ref={logsEndRef} />
    </div>
  );
}
