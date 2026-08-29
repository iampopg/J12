interface Props {
  oauthProvider: "google" | "microsoft";
  onProviderChange: (p: "google" | "microsoft") => void;
  deviceFlow: { user_code: string; verification_uri: string; device_code: string } | null;
  onStartDeviceAuth: () => void;
}

export function ImapOAuthSection({
  oauthProvider,
  onProviderChange,
  deviceFlow,
  onStartDeviceAuth,
}: Props) {
  return (
    <div style={{ marginTop: 12, padding: 14, background: "var(--bg-2)", borderRadius: "var(--r-sm)", border: "1px solid var(--accent-subtle)" }}>
      <div className="row between mb-2">
        <span style={{ fontSize: 11, fontWeight: 700, color: "var(--accent)" }}>SELECT OAUTH2 CLOUD PROVIDER</span>
        <div className="row gap-1">
          <button
            type="button"
            className={`btn btn-sm ${oauthProvider === "google" ? "btn-primary" : "btn-ghost"}`}
            style={{ fontSize: 11, padding: "2px 8px" }}
            onClick={() => onProviderChange("google")}
          >
            Google Workspace
          </button>
          <button
            type="button"
            className={`btn btn-sm ${oauthProvider === "microsoft" ? "btn-primary" : "btn-ghost"}`}
            style={{ fontSize: 11, padding: "2px 8px" }}
            onClick={() => onProviderChange("microsoft")}
          >
            Microsoft 365
          </button>
        </div>
      </div>

      {deviceFlow ? (
        <div style={{ background: "var(--bg-0)", padding: 12, borderRadius: "var(--r-xs)", border: "1px solid #38bdf8", marginTop: 8 }}>
          <div style={{ fontSize: 12, fontWeight: 600, color: "#38bdf8", marginBottom: 4 }}>
            👉 Open Browser &amp; Authorize:
          </div>
          <div style={{ fontSize: 12, marginBottom: 6 }}>
            URL: <a href={deviceFlow.verification_uri} target="_blank" rel="noreferrer" style={{ color: "#60a5fa", textDecoration: "underline" }}>{deviceFlow.verification_uri}</a>
          </div>
          <div style={{ fontSize: 13 }}>
            Code: <strong style={{ color: "#fbbf24", fontFamily: "var(--mono)", fontSize: 16, background: "var(--bg-2)", padding: "2px 8px", borderRadius: 4 }}>{deviceFlow.user_code}</strong>
          </div>
          <div className="muted mt-2" style={{ fontSize: 11 }}>
            ⏳ Polling for token confirmation... You will be logged in automatically once confirmed.
          </div>
        </div>
      ) : (
        <div className="row between mt-2" style={{ alignItems: "center" }}>
          <span style={{ fontSize: 11.5, color: "var(--text-2)" }}>
            Authenticate via standard browser consent without app passwords:
          </span>
          <button
            type="button"
            className="btn btn-primary btn-sm"
            style={{ fontWeight: 700, fontSize: 11.5 }}
            onClick={onStartDeviceAuth}
          >
            ⚡ Start Interactive Browser Login
          </button>
        </div>
      )}
    </div>
  );
}
