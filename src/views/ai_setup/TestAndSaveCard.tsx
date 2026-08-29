interface Props {
  testing: boolean;
  saving: boolean;
  testResult: { success: boolean; message: string } | null;
  aiSetupComplete: boolean;
  configEnabled: boolean;
  onTestConnection: () => void;
  onSaveConfig: () => void;
}

export function TestAndSaveCard({
  testing,
  saving,
  testResult,
  aiSetupComplete,
  configEnabled,
  onTestConnection,
  onSaveConfig,
}: Props) {
  return (
    <>
      <div className="card mb-4">
        <h3 style={{ fontSize: 15, fontWeight: 600, marginBottom: 16 }}>4. Test & Save</h3>
        <div className="row gap-2">
          <button
            className="btn btn-ghost"
            onClick={onTestConnection}
            disabled={testing}
          >
            {testing ? "Testing..." : "🔗 Test Connection"}
          </button>
          <button
            className="btn btn-primary"
            onClick={onSaveConfig}
            disabled={saving}
          >
            {saving ? "Saving..." : "💾 Save Configuration"}
          </button>
        </div>
        {testResult && (
          <div style={{
            marginTop: 12,
            padding: 12,
            borderRadius: "var(--r-sm)",
            background: testResult.success ? "rgba(34, 197, 94, 0.1)" : "rgba(239, 68, 68, 0.1)",
            border: `1px solid ${testResult.success ? "#22c55e" : "#ef4444"}`,
            fontSize: 12,
          }}>
            {testResult.success ? "✅" : "❌"} {testResult.message}
          </div>
        )}
      </div>

      {/* AI Data Access (shown after save) */}
      {aiSetupComplete && configEnabled && (
        <div className="card mb-4">
          <h3 style={{ fontSize: 15, fontWeight: 600, marginBottom: 16 }}>5. AI Data Access</h3>
          <p className="muted mb-4" style={{ fontSize: 12 }}>
            Configure what case data the AI can access. AI can only read data, never modify it.
          </p>
          <div style={{ display: "grid", gap: 8 }}>
            {[
              { key: "emails", label: "Email metadata (from, to, subject, date)", default: true },
              { key: "headers", label: "Email headers (Received, Authentication-Results)", default: true },
              { key: "body", label: "Email body text", default: true },
              { key: "attachments", label: "Attachment metadata (filename, hash, type)", default: true },
              { key: "findings", label: "Forensic findings", default: true },
              { key: "entities", label: "Entity profiles", default: true },
              { key: "timeline", label: "Timeline events", default: true },
              { key: "graph", label: "Communication graph", default: false },
              { key: "notes", label: "Case notes", default: false },
              { key: "custody", label: "Chain of custody", default: false },
            ].map(item => (
              <label key={item.key} className="row gap-2" style={{ padding: 8, background: "var(--bg-3)", borderRadius: "var(--r-sm)", cursor: "pointer" }}>
                <input type="checkbox" defaultChecked={item.default} />
                <span style={{ fontSize: 12 }}>{item.label}</span>
              </label>
            ))}
          </div>
        </div>
      )}
    </>
  );
}
