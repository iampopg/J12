interface Props {
  show: boolean;
  preferences: {
    chunkSize: number;
    autoExtractArtifacts: boolean;
    autoRunAnalysis: boolean;
    deduplicationMode: "message_id" | "body_hash" | "strict";
  };
  setPreferences: (p: any) => void;
}

export function ImapPreferencesPanel({ show, preferences, setPreferences }: Props) {
  if (!show) return null;

  return (
    <div className="card mb-4" style={{ background: "var(--bg-2)", border: "1px solid var(--accent-subtle)" }}>
      <h4 style={{ fontSize: 13, fontWeight: 600, color: "var(--accent)", marginBottom: 12 }}>
        ⚡ Acquisition Performance &amp; Pipeline Preferences
      </h4>
      <div className="grid-3" style={{ fontSize: 12 }}>
        <div>
          <label className="label">Chunk Size (Pipelined Batch)</label>
          <select 
            className="input" 
            value={preferences.chunkSize} 
            onChange={e => setPreferences({ chunkSize: parseInt(e.target.value) || 50 })}
          >
            <option value={25}>25 msgs/batch (High Reliability)</option>
            <option value={50}>50 msgs/batch (Recommended Balance)</option>
            <option value={100}>100 msgs/batch (High Speed)</option>
            <option value={250}>250 msgs/batch (Maximum Turbo)</option>
          </select>
        </div>

        <div>
          <label className="label">Automated Intelligence Pipeline</label>
          <div style={{ display: "flex", flexDirection: "column", gap: 6, paddingTop: 4 }}>
            <label className="row gap-2" style={{ cursor: "pointer", fontSize: 12 }}>
              <input 
                type="checkbox" 
                checked={preferences.autoExtractArtifacts} 
                onChange={e => setPreferences({ autoExtractArtifacts: e.target.checked })} 
              />
              <span>Auto-extract artifacts upon completion</span>
            </label>
            <label className="row gap-2" style={{ cursor: "pointer", fontSize: 12 }}>
              <input 
                type="checkbox" 
                checked={preferences.autoRunAnalysis} 
                onChange={e => setPreferences({ autoRunAnalysis: e.target.checked })} 
              />
              <span>Auto-run security risk &amp; spoofing analysis</span>
            </label>
          </div>
        </div>

        <div>
          <label className="label">Deduplication Mode</label>
          <select 
            className="input" 
            value={preferences.deduplicationMode} 
            onChange={e => setPreferences({ deduplicationMode: e.target.value as any })}
          >
            <option value="message_id">Standard RFC Message-ID (Recommended)</option>
            <option value="body_hash">Cryptographic SHA-256 Payload Hash</option>
            <option value="strict">Strict Composite Header Match</option>
          </select>
        </div>
      </div>
    </div>
  );
}
