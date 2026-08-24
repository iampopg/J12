import { useState } from "react";
import { useAuth } from "../auth";
import { J12Logo } from "../components/J12Logo";
import { FooterSignature } from "../components/FooterSignature";

export function LoginPage() {
  const { login, register } = useAuth();
  const [mode, setMode] = useState<"login" | "register">("login");

  // Login form state
  const [user, setUser] = useState("admin");
  const [pass, setPass] = useState("admin123");

  // Register form state
  const [regUser, setRegUser] = useState("");
  const [regFullName, setRegFullName] = useState("");
  const [regPass, setRegPass] = useState("");
  const [regPassConfirm, setRegPassConfirm] = useState("");
  const [agreeLegal, setAgreeLegal] = useState(false);

  const [err, setErr] = useState("");
  const [successMsg, setSuccessMsg] = useState("");
  const [loading, setLoading] = useState(false);

  const submitLogin = async (e: React.FormEvent) => {
    e.preventDefault();
    setLoading(true);
    setErr("");
    setSuccessMsg("");
    const ok = await login(user, pass);
    if (!ok) setErr("Invalid username or password. Please verify your credentials.");
    setLoading(false);
  };

  const submitRegister = async (e: React.FormEvent) => {
    e.preventDefault();
    setLoading(true);
    setErr("");
    setSuccessMsg("");

    if (!regUser.trim() || regUser.trim().length < 3) {
      setErr("Username must be at least 3 characters long.");
      setLoading(false);
      return;
    }

    if (!regPass || regPass.length < 4) {
      setErr("Password must be at least 4 characters long.");
      setLoading(false);
      return;
    }

    if (regPass !== regPassConfirm) {
      setErr("Passwords do not match.");
      setLoading(false);
      return;
    }

    if (!agreeLegal) {
      setErr("You must review and accept the Forensic Legal Disclaimer & Ethical Compliance Terms to proceed.");
      setLoading(false);
      return;
    }

    const res = await register({
      username: regUser.trim(),
      password: regPass,
      fullName: regFullName.trim() || regUser.trim(),
      agency: "Digital Forensics Unit",
    });

    if (!res.success) {
      setErr(res.message || "Failed to create account.");
      setLoading(false);
    }
  };

  return (
    <div className="login-wrap" style={{ display: "flex", flexDirection: "column", minHeight: "100vh", justifyContent: "center", alignItems: "center", padding: "20px" }}>
      <div className="login-card" style={{ maxWidth: mode === "register" ? 540 : 440, width: "100%", padding: "32px 28px", transition: "max-width 0.2s ease" }}>
        <div className="login-brand" style={{ display: "flex", flexDirection: "column", alignItems: "center", marginBottom: 20 }}>
          <J12Logo size={60} />
          <h1 className="login-title" style={{ marginTop: 10, fontSize: 24 }}>
            <span style={{ color: "#ffffff" }}>J</span>
            <span style={{ color: "#22c55e" }}>12</span>
          </h1>
          <p className="login-subtitle" style={{ fontSize: 12, color: "var(--text-3)" }}>
            Email Forensic Investigation Platform
          </p>
        </div>

        {/* Mode Switcher Tabs */}
        <div
          style={{
            display: "grid",
            gridTemplateColumns: "1fr 1fr",
            gap: 6,
            background: "var(--bg-3)",
            padding: 4,
            borderRadius: "var(--r-md)",
            marginBottom: 20,
          }}
        >
          <button
            type="button"
            className={`btn btn-sm ${mode === "login" ? "btn-primary" : "btn-ghost"}`}
            style={{ fontWeight: 600 }}
            onClick={() => { setMode("login"); setErr(""); setSuccessMsg(""); }}
          >
            🔑 Sign In
          </button>
          <button
            type="button"
            className={`btn btn-sm ${mode === "register" ? "btn-primary" : "btn-ghost"}`}
            style={{ fontWeight: 600 }}
            onClick={() => { setMode("register"); setErr(""); setSuccessMsg(""); }}
          >
            📝 Register Examiner
          </button>
        </div>

        {err && <div className="login-error" style={{ marginBottom: 16 }}>{err}</div>}
        {successMsg && (
          <div style={{ background: "rgba(34,197,94,0.1)", border: "1px solid rgba(34,197,94,0.3)", color: "var(--green)", padding: "10px 14px", borderRadius: "var(--r-md)", marginBottom: 16, fontSize: 13 }}>
            {successMsg}
          </div>
        )}

        {mode === "login" ? (
          <form onSubmit={submitLogin}>
            <div className="field">
              <label className="label">Username</label>
              <input
                className="input"
                value={user}
                onChange={(e) => setUser(e.target.value)}
                placeholder="Enter username"
                autoFocus
              />
            </div>

            <div className="field">
              <label className="label">Password</label>
              <input
                className="input"
                type="password"
                value={pass}
                onChange={(e) => setPass(e.target.value)}
                placeholder="Enter password"
              />
            </div>

            <button
              type="submit"
              className="btn btn-primary"
              disabled={loading}
              style={{ width: "100%", marginTop: 12, padding: "10px", fontWeight: 700 }}
            >
              {loading ? "Signing in..." : "Sign In to Workspace"}
            </button>

            <div style={{ marginTop: 14, textAlign: "center", fontSize: 11, color: "var(--text-3)" }}>
              Default Local Admin: <code style={{ color: "var(--accent)" }}>admin</code> / <code style={{ color: "var(--accent)" }}>admin123</code>
            </div>
          </form>
        ) : (
          <form onSubmit={submitRegister}>
            <div className="grid-2" style={{ gap: 10 }}>
              <div className="field">
                <label className="label">Username *</label>
                <input
                  className="input"
                  value={regUser}
                  onChange={(e) => setRegUser(e.target.value)}
                  placeholder="e.g. examiner1"
                  required
                  autoFocus
                />
              </div>
              <div className="field">
                <label className="label">Full Name / Examiner Title (Optional)</label>
                <input
                  className="input"
                  value={regFullName}
                  onChange={(e) => setRegFullName(e.target.value)}
                  placeholder="e.g. John Doe, CCE"
                />
              </div>
            </div>

            <div className="grid-2" style={{ gap: 10 }}>
              <div className="field">
                <label className="label">Local Password *</label>
                <input
                  className="input"
                  type="password"
                  value={regPass}
                  onChange={(e) => setRegPass(e.target.value)}
                  placeholder="Min 4 characters"
                  required
                />
              </div>
              <div className="field">
                <label className="label">Confirm Password *</label>
                <input
                  className="input"
                  type="password"
                  value={regPassConfirm}
                  onChange={(e) => setRegPassConfirm(e.target.value)}
                  placeholder="Repeat password"
                  required
                />
              </div>
            </div>

            {/* Legal Disclaimer & Compliance Box */}
            <div
              style={{
                marginTop: 12,
                marginBottom: 16,
                background: "var(--bg-3)",
                border: "1px solid var(--border)",
                borderRadius: "var(--r-sm)",
                padding: "12px 14px",
              }}
            >
              <div style={{ display: "flex", alignItems: "center", gap: 6, marginBottom: 6 }}>
                <span style={{ fontSize: 13 }}>⚖️</span>
                <strong style={{ fontSize: 11, color: "var(--text-1)", textTransform: "uppercase", letterSpacing: "0.05em" }}>
                  Forensic Legal Disclaimer &amp; Compliance Policy
                </strong>
              </div>
              <div
                style={{
                  maxHeight: 110,
                  overflowY: "auto",
                  fontSize: 11,
                  color: "var(--text-2)",
                  lineHeight: 1.5,
                  paddingRight: 6,
                  borderBottom: "1px solid var(--border)",
                  paddingBottom: 8,
                  marginBottom: 10,
                }}
              >
                <p style={{ margin: "0 0 6px" }}>
                  <strong>1. Authorized Lawful Use:</strong> J12 Forensic Suite is designed exclusively for authorized law enforcement, legal discovery (eDiscovery), internal incident response, and cybersecurity investigations. You certify that you possess lawful authority, search warrant, subpoena, or explicit consent to acquire and examine target data.
                </p>
                <p style={{ margin: "0 0 6px" }}>
                  <strong>2. Evidentiary Integrity &amp; Standards:</strong> All evidence processing complies with NIST SP 800-86 and ISO/IEC 27037 standards. Chain-of-custody logs and cryptographic SHA-256 hashes are immutable records stored locally on your workstation.
                </p>
                <p style={{ margin: 0 }}>
                  <strong>3. Data Privacy &amp; Local Storage:</strong> All parsed emails, attachments, and forensic artifacts remain entirely on your local machine. No evidence is transmitted to third-party cloud servers without your explicit configuration.
                </p>
              </div>

              <label style={{ display: "flex", alignItems: "flex-start", gap: 8, cursor: "pointer", fontSize: 12, color: "var(--text-1)" }}>
                <input
                  type="checkbox"
                  checked={agreeLegal}
                  onChange={(e) => setAgreeLegal(e.target.checked)}
                  style={{ marginTop: 2, accentColor: "var(--accent)" }}
                  required
                />
                <span>
                  I have read and agree to the <strong>Forensic Compliance Policy &amp; Legal Terms</strong>, and confirm all investigations will be conducted lawfully.
                </span>
              </label>
            </div>

            <button
              type="submit"
              className="btn btn-primary"
              disabled={loading || !agreeLegal}
              style={{
                width: "100%",
                padding: "10px",
                fontWeight: 700,
                opacity: agreeLegal ? 1 : 0.6,
              }}
            >
              {loading ? "Registering..." : "Accept Terms & Create Account"}
            </button>
          </form>
        )}
      </div>

      {/* Footer Signature */}
      <FooterSignature style={{ maxWidth: mode === "register" ? 540 : 440, width: "100%", marginTop: 20 }} />
    </div>
  );
}

