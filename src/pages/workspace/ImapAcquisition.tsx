import { useState, useEffect, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useAcquisition } from "../../context/AcquisitionContext";
import { ImapPreferencesPanel } from "./ImapPreferencesPanel";
import { ImapStreamingLogs } from "./ImapStreamingLogs";
import { ImapOAuthSection } from "./ImapOAuthSection";

interface Props {
  caseId: string;
  onComplete: () => void;
}

export function ImapAcquisition({ caseId, onComplete }: Props) {
  const {
    isAcquiring,
    pipelineStep,
    activeCaseId,
    progress,
    percent,
    logs,
    result,
    preferences,
    setPreferences,
    startAcquisition,
    stopAcquisition,
    clearLogs,
  } = useAcquisition();

  const getSaved = () => {
    try {
      return JSON.parse(localStorage.getItem(`imap_creds_${caseId}`) || "{}");
    } catch { return {}; }
  };
  const saved = getSaved();

  const [protocol, setProtocol] = useState<"imap" | "pop3">(saved.protocol || "imap");
  const [authType, setAuthType] = useState<"password" | "oauth2">(saved.authType || "password");
  const [oauthProvider, setOauthProvider] = useState<"google" | "microsoft">("google");
  const [accessToken, setAccessToken] = useState<string>(saved.accessToken || "");
  const [deviceFlow, setDeviceFlow] = useState<{ user_code: string; verification_uri: string; device_code: string } | null>(null);
  const [username, setUsername] = useState(saved.username || "");
  const [password, setPassword] = useState(saved.password || "");
  const [showPassword, setShowPassword] = useState(false);
  const [savePassword, setSavePassword] = useState(saved.savePassword !== false);
  const [server, setServer] = useState(saved.server || "imap.gmail.com");
  const [port, setPort] = useState(saved.port || "993");
  const [useSsl, setUseSsl] = useState(saved.useSsl !== undefined ? saved.useSsl : true);
  const [mailboxScope, setMailboxScope] = useState(saved.mailboxScope || "ALL");
  const [showAdvanced, setShowAdvanced] = useState(false);
  const [showPreferences, setShowPreferences] = useState(false);
  const [mailboxes, setMailboxes] = useState<string[]>(saved.mailboxes || []);
  const [connecting, setConnecting] = useState(false);
  const [localLogs, setLocalLogs] = useState<string[]>([]);

  const logsEndRef = useRef<HTMLDivElement | null>(null);

  useEffect(() => {
    localStorage.setItem(`imap_creds_${caseId}`, JSON.stringify({
      protocol,
      authType,
      username,
      password: savePassword ? password : "",
      accessToken: savePassword ? accessToken : "",
      savePassword,
      server,
      port,
      useSsl,
      mailboxScope,
      mailboxes
    }));
  }, [caseId, protocol, authType, username, password, accessToken, savePassword, server, port, useSsl, mailboxScope, mailboxes]);

  const handleProtocolChange = (newProtocol: "imap" | "pop3") => {
    setProtocol(newProtocol);
    if (newProtocol === "imap") {
      setPort("993");
      setUseSsl(true);
      if (server.includes("pop.") || server === "imap.gmail.com") {
        setServer(oauthProvider === "microsoft" ? "outlook.office365.com" : "imap.gmail.com");
      }
    } else {
      setPort("995");
      setUseSsl(true);
      if (server.includes("imap.")) {
        setServer(server.replace("imap.", "pop."));
      }
    }
  };

  const handleOauthProviderChange = (prov: "google" | "microsoft") => {
    setOauthProvider(prov);
    setServer(prov === "google" ? "imap.gmail.com" : "outlook.office365.com");
    setPort("993");
    setUseSsl(true);
  };

  const handleEmailChange = (val: string) => {
    setUsername(val);
    const domain = val.includes("@") ? val.split("@")[1].toLowerCase().trim() : "";
    if (protocol === "imap") {
      if (domain.includes("gmail") || domain.includes("googlemail")) {
        setServer("imap.gmail.com");
        setPort("993");
        setOauthProvider("google");
      } else if (domain.includes("outlook") || domain.includes("hotmail") || domain.includes("office365") || domain.includes("live.com")) {
        setServer("outlook.office365.com");
        setPort("993");
        setOauthProvider("microsoft");
      } else if (domain.includes(".") && !domain.endsWith(".")) {
        setServer(`imap.${domain}`);
        setPort("993");
      }
    }
  };

  const addLocalLog = (msg: string) => setLocalLogs(prev => [...prev, `[${new Date().toLocaleTimeString()}] ${msg}`]);

  const handleStartDeviceAuth = async () => {
    addLocalLog(`Starting OAuth 2.0 Device Flow for ${oauthProvider.toUpperCase()}...`);
    try {
      const flow = await invoke<any>("imap_device_flow_start", { input: { provider: oauthProvider } });
      if (flow?.user_code) {
        setDeviceFlow(flow);
        addLocalLog(`👉 Open browser: ${flow.verification_uri} | Enter code: ${flow.user_code}`);
        pollForToken(flow.device_code);
      }
    } catch (e: any) { addLocalLog(`❌ OAuth Error: ${e}`); }
  };

  const pollForToken = (deviceCode: string) => {
    let attempts = 0;
    const interval = setInterval(async () => {
      attempts++;
      if (attempts > 60) { clearInterval(interval); addLocalLog("⚠️ OAuth authentication timed out."); return; }
      try {
        const res = await invoke<any>("imap_device_flow_poll", { input: { provider: oauthProvider, device_code: deviceCode } });
        if (res?.access_token) {
          clearInterval(interval);
          setAccessToken(res.access_token);
          setPassword(res.access_token);
          setDeviceFlow(null);
          addLocalLog(`✓ OAuth 2.0 Access Token acquired! SASL XOAUTH2 ready.`);
        }
      } catch {}
    }, 4000);
  };

  const testConnection = async () => {
    const cleanUser = username.trim();
    const tokenOrPass = authType === "oauth2" ? (accessToken.trim() || password.trim()) : password.trim();
    setConnecting(true);
    setLocalLogs([]);
    addLocalLog(`Testing ${protocol.toUpperCase()} connection to ${server}:${port}...`);
    try {
      if (protocol === "imap") {
        const boxes = await invoke<string[]>("imap_list_mailboxes", {
          input: {
            server: server.trim(),
            port: parseInt(port) || 993,
            username: cleanUser,
            password: tokenOrPass,
            auth_type: authType,
            access_token: authType === "oauth2" ? tokenOrPass : undefined,
            use_ssl: useSsl,
          }
        });
        setMailboxes(boxes);
        addLocalLog(`✓ SASL XOAUTH2 Authentication Successful! Found ${boxes.length} folders.`);
      } else {
        await invoke("pop3_test_connection", {
          input: {
            server: server.trim(),
            port: parseInt(port) || 995,
            username: cleanUser,
            password: tokenOrPass,
            use_ssl: useSsl,
          }
        });
        setMailboxes(["INBOX"]);
        addLocalLog(`✓ POP3 Connection Successful!`);
      }
    } catch (e: any) {
      addLocalLog(`✗ Connection failed: ${e}`);
    }
    setConnecting(false);
  };

  const acquireEmails = async () => {
    const cleanUser = username.trim();
    const tokenOrPass = authType === "oauth2" ? (accessToken.trim() || password.trim()) : password.trim();

    try {
      await startAcquisition({
        caseId,
        protocol,
        server: server.trim(),
        port: parseInt(port) || (protocol === "imap" ? 993 : 995),
        username: cleanUser,
        password: tokenOrPass,
        authType,
        accessToken: authType === "oauth2" ? tokenOrPass : undefined,
        useSsl,
        mailboxScope,
        maxMessages: null,
        onPipelineComplete: () => { onComplete(); },
      });
    } catch (e: any) {
      console.error("Acquisition error:", e);
    }
  };

  const combinedLogs = logs.length > 0 ? logs : localLogs;
  const isThisCaseAcquiring = isAcquiring && activeCaseId === caseId;

  return (
    <div>
      <div className="row between mb-3">
        <div>
          <h3 style={{ fontSize: 16, fontWeight: 600, color: "var(--text-0)" }}>Live Mail Server Acquisition (IMAP / POP3)</h3>
          <p className="muted" style={{ fontSize: 12 }}>
            Forensic multi-folder streaming extraction over TLS with SASL XOAUTH2 modern auth &amp; automated intelligence pipeline
          </p>
        </div>
        <div className="row gap-2">
          <button 
            type="button" 
            className="btn btn-ghost btn-sm"
            onClick={() => setShowPreferences(!showPreferences)}
            style={{ fontSize: 11 }}
          >
            ⚙️ Preferences &amp; Tuning
          </button>
          <span className="badge badge-green">SASL XOAUTH2 READY</span>
          <span className="badge badge-blue">TLS 1.3 / SSL</span>
        </div>
      </div>

      <ImapPreferencesPanel
        show={showPreferences}
        preferences={preferences}
        setPreferences={setPreferences}
      />
      
      <div className="card mb-4" style={{ background: "var(--bg-1)", border: "1px solid var(--border)" }}>
        {/* Authentication Mode Tabs */}
        <div className="row gap-2 mb-3" style={{ borderBottom: "1px solid var(--border)", paddingBottom: 12 }}>
          <button
            type="button"
            className={`btn btn-sm ${authType === "password" ? "btn-primary" : "btn-ghost"}`}
            style={{ fontWeight: 600, fontSize: 12 }}
            onClick={() => setAuthType("password")}
          >
            🔐 Password / App-Specific Password
          </button>
          <button
            type="button"
            className={`btn btn-sm ${authType === "oauth2" ? "btn-primary" : "btn-ghost"}`}
            style={{ fontWeight: 600, fontSize: 12 }}
            onClick={() => {
              setAuthType("oauth2");
              setProtocol("imap");
            }}
          >
            ⚡ Modern OAuth 2.0 (Google Workspace / Microsoft 365)
          </button>
        </div>

        <div className="grid-2">
          <div className="field">
            <label className="label">Protocol</label>
            <div className="row gap-2" style={{ paddingTop: 4 }}>
              <button
                type="button"
                className={`btn btn-sm ${protocol === "imap" ? "btn-primary" : "btn-ghost"}`}
                onClick={() => handleProtocolChange("imap")}
                style={{ flex: 1, padding: "8px 16px" }}
              >
                IMAP (Port 993)
              </button>
              <button
                type="button"
                className={`btn btn-sm ${protocol === "pop3" ? "btn-primary" : "btn-ghost"}`}
                onClick={() => handleProtocolChange("pop3")}
                disabled={authType === "oauth2"}
                style={{ flex: 1, padding: "8px 16px" }}
              >
                POP3 (Port 995)
              </button>
            </div>
          </div>
          <div className="field">
            <label className="label">Mailbox Scope</label>
            <select className="input" value={mailboxScope} onChange={e => setMailboxScope(e.target.value)}>
              <option value="ALL">📦 Entire Account (All Folders: Inbox, Sent, Trash, Spam, Archive)</option>
              <option value="INBOX">📥 Inbox Only</option>
              {mailboxes.filter(b => b.toUpperCase() !== "INBOX").map(b => (
                <option key={b} value={b}>📁 {b}</option>
              ))}
            </select>
          </div>
        </div>

        {authType === "oauth2" && (
          <ImapOAuthSection
            oauthProvider={oauthProvider}
            onProviderChange={handleOauthProviderChange}
            deviceFlow={deviceFlow}
            onStartDeviceAuth={handleStartDeviceAuth}
          />
        )}

        <div className="grid-2" style={{ marginTop: 12 }}>
          <div className="field">
            <label className="label">Target Email / Account *</label>
            <input 
              className="input" 
              value={username} 
              onChange={e => handleEmailChange(e.target.value)} 
              placeholder="e.g. suspect@gmail.com, target@company.com" 
              required
            />
          </div>
          <div className="field">
            <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center" }}>
              <label className="label" style={{ marginBottom: 0 }}>
                {authType === "oauth2" ? "OAuth 2.0 Bearer Token (Auto-populated or Paste)" : "Password / App-Specific Password *"}
              </label>
              <button
                type="button"
                className="btn btn-ghost"
                style={{ fontSize: 11, padding: "2px 6px", height: "auto", minHeight: 0, color: "var(--text-3)" }}
                onClick={() => setShowPassword(!showPassword)}
                title={showPassword ? "Hide password" : "Show password"}
              >
                {showPassword ? "👁️ Hide" : "🔒 Show"}
              </button>
            </div>
            <input 
              className="input" 
              type={showPassword ? "text" : "password"}
              value={authType === "oauth2" ? (accessToken || password) : password} 
              onChange={e => {
                setPassword(e.target.value);
                if (authType === "oauth2") setAccessToken(e.target.value);
              }} 
              placeholder={authType === "oauth2" ? "ya29... or eyJhbGci..." : "••••••••••••••••"} 
              required
              autoComplete="new-password"
            />
            <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginTop: 4 }}>
              <label style={{ fontSize: 11, color: "var(--text-2)", display: "flex", alignItems: "center", gap: 6, cursor: "pointer" }}>
                <input
                  type="checkbox"
                  checked={savePassword}
                  onChange={e => setSavePassword(e.target.checked)}
                />
                Remember password for this case
              </label>
              {password && savePassword && (
                <span style={{ fontSize: 10, color: "#10b981" }}>✓ Saved locally</span>
              )}
            </div>
          </div>
        </div>

        <div className="grid-2" style={{ marginTop: 12 }}>
          <div className="field" style={{ display: "flex", flexDirection: "column", justifyContent: "flex-end" }}>
            <button 
              type="button" 
              className="btn btn-ghost" 
              style={{ fontSize: 12, textAlign: "left", width: "fit-content", padding: "8px 12px" }}
              onClick={() => setShowAdvanced(!showAdvanced)}
            >
              {showAdvanced ? "▲ Hide Server Configuration" : "⚙️ Custom Server Settings (Auto-Configured)"}
            </button>
          </div>
        </div>

        {showAdvanced && (
          <div style={{ marginTop: 16, padding: 14, background: "var(--bg-3)", borderRadius: "var(--r-md)" }}>
            <h5 style={{ fontSize: 12, fontWeight: 600, color: "var(--text-1)", marginBottom: 10 }}>{protocol.toUpperCase()} Server Parameters</h5>
            <div className="grid-3">
              <div className="field">
                <label className="label">Host / Server</label>
                <input className="input" value={server} onChange={e => setServer(e.target.value)} placeholder={protocol === "imap" ? "imap.server.com" : "pop.server.com"} />
              </div>
              <div className="field">
                <label className="label">Port</label>
                <input className="input" value={port} onChange={e => setPort(e.target.value)} placeholder={protocol === "imap" ? "993" : "995"} />
              </div>
              <div className="field" style={{ display: "flex", alignItems: "center", paddingTop: 20 }}>
                <label className="row gap-2" style={{ cursor: "pointer" }}>
                  <input type="checkbox" checked={useSsl} onChange={e => setUseSsl(e.target.checked)} />
                  <span style={{ fontSize: 12, fontWeight: 500 }}>Use SSL / TLS (Port {protocol === "imap" ? "993" : "995"})</span>
                </label>
              </div>
            </div>
          </div>
        )}

        <div className="row between" style={{ marginTop: 20 }}>
          <div className="row gap-2">
            <button 
              type="button" 
              className="btn btn-ghost" 
              onClick={testConnection} 
              disabled={connecting || isThisCaseAcquiring || !username || (!password && !accessToken)}
            >
              {connecting ? "Testing Connection..." : "🔗 Test Connection & Enumerate Folders"}
            </button>
            <button 
              type="button" 
              className="btn btn-primary" 
              onClick={acquireEmails} 
              disabled={isThisCaseAcquiring || connecting || !username || (!password && !accessToken)}
            >
              {isThisCaseAcquiring ? "⏳ Acquiring Live Account..." : "📥 Acquire & Ingest Live Emails"}
            </button>
          </div>

          {isThisCaseAcquiring && (
            <button 
              type="button" 
              className="btn btn-danger" 
              onClick={stopAcquisition}
              style={{ background: "#dc2626", color: "#fff", borderColor: "#dc2626" }}
            >
              ⏹ Stop / Pause Acquisition
            </button>
          )}
        </div>
      </div>

      {(isThisCaseAcquiring || progress) && (
        <div className="card mb-4" style={{ border: "1px solid var(--accent)", background: "var(--bg-2)" }}>
          <div className="row between mb-2">
            <div className="row gap-2" style={{ alignItems: "center" }}>
              <span className="badge badge-blue">FOLDER {progress?.folderIndex || 1} OF {progress?.totalFolders || 1}</span>
              <strong style={{ fontSize: 13, color: "var(--text-0)" }}>{progress?.folder || "Scanning mailboxes..."}</strong>
            </div>
            <div style={{ fontSize: 12, fontWeight: 700, color: "var(--accent)" }}>
              {percent}% ({progress?.overallSeq || progress?.msgSeq || 0}/{progress?.overallTotal || progress?.folderTotal || 0} messages)
            </div>
          </div>
          <div style={{ width: "100%", height: 8, background: "var(--bg-3)", borderRadius: 4, overflow: "hidden", marginBottom: 12 }}>
            <div style={{ width: `${percent}%`, height: "100%", background: "linear-gradient(90deg, #3b82f6, #6366f1)", transition: "width 0.2s ease" }} />
          </div>
          <div className="grid-3" style={{ fontSize: 12 }}>
            <div><span className="muted">Total Ingested:</span> <strong style={{ color: "var(--success)" }}>{progress?.ingested || 0}</strong></div>
            <div><span className="muted">Duplicates Skipped:</span> <strong style={{ color: "#38bdf8" }}>{progress?.duplicatesSkipped || 0}</strong></div>
            <div style={{ overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}><span className="muted">Current:</span> <span>{progress?.subject || "..."}</span></div>
          </div>
        </div>
      )}

      {result && (
        <div className="card mb-4">
          <h4 style={{ fontSize: 14, fontWeight: 600, color: "var(--text-0)", marginBottom: 12 }}>
            {result.was_cancelled ? "⏹ Acquisition Paused / Stopped" : "✓ Acquisition Results"}
          </h4>
          <div className="grid-3 mb-3">
            <div className="card" style={{ textAlign: "center", padding: 12 }}>
              <div style={{ fontSize: 22, fontWeight: 700, color: "var(--accent)" }}>{result.total_found}</div>
              <div className="muted text-sm">Discovered on Server</div>
            </div>
            <div className="card" style={{ textAlign: "center", padding: 12 }}>
              <div style={{ fontSize: 22, fontWeight: 700, color: "var(--success)" }}>{result.downloaded}</div>
              <div className="muted text-sm">Ingested &amp; Saved to DB</div>
            </div>
            <div className="card" style={{ textAlign: "center", padding: 12 }}>
              <div style={{ fontSize: 22, fontWeight: 700, color: "#38bdf8" }}>{result.duplicates_skipped || 0}</div>
              <div className="muted text-sm">Skipped (Saved Bandwidth)</div>
            </div>
          </div>
          {result.folders_acquired && result.folders_acquired.length > 0 && (
            <div style={{ fontSize: 12, color: "var(--text-2)" }}>
              <strong>Folders Acquired:</strong> {result.folders_acquired.join(", ")}
            </div>
          )}
        </div>
      )}

      <ImapStreamingLogs
        combinedLogs={combinedLogs}
        clearLogs={clearLogs}
        caseId={caseId}
        logsEndRef={logsEndRef}
      />
    </div>
  );
}
