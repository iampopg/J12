import { useState } from "react";
import { useAuth } from "../auth";
import { J12Logo } from "../components/J12Logo";

export function LoginPage() {
  const { login } = useAuth();
  const [user, setUser] = useState("admin");
  const [pass, setPass] = useState("admin123");
  const [err, setErr] = useState("");
  const [loading, setLoading] = useState(false);

  const submit = async (e: React.FormEvent) => {
    e.preventDefault();
    setLoading(true);
    setErr("");
    const ok = await login(user, pass);
    if (!ok) setErr("Invalid credentials");
    setLoading(false);
  };

  return (
    <div className="login-wrap">
      <div className="login-card">
        <div className="login-brand" style={{ display: "flex", flexDirection: "column", alignItems: "center" }}>
          <J12Logo size={64} />
          <h1 className="login-title" style={{ marginTop: 12 }}>
            <span style={{ color: "#ffffff" }}>J</span>
            <span style={{ color: "#22c55e" }}>12</span>
          </h1>
          <p className="login-subtitle">Email Forensic Investigation Platform</p>
        </div>

        {err && <div className="login-error">{err}</div>}

        <form onSubmit={submit}>
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
            style={{ width: "100%", marginTop: 8 }}
          >
            {loading ? "Signing in..." : "Sign In"}
          </button>
        </form>
      </div>
    </div>
  );
}
