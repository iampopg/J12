import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { J12Logo } from "../components/J12Logo";

interface Case {
  id: string;
  title: string;
  case_number: string;
  description: string;
  status: string;
  target_email: string | null;
  target_name: string | null;
  target_organization: string | null;
  investigation_type: string;
  created_at: string;
}

export function CaseListPage({ onSelectCase }: { onSelectCase: (id: string) => void }) {
  const [cases, setCases] = useState<Case[]>([]);
  const [loading, setLoading] = useState(true);
  const [showCreate, setShowCreate] = useState(false);
  const [form, setForm] = useState({
    title: "",
    case_number: "",
    description: "",
    target_email: "",
    target_name: "",
    target_organization: "",
    investigation_type: "general"
  });

  const load = async () => {
    setLoading(true);
    try {
      const result = await invoke<Case[]>("case_list");
      setCases(result);
    } catch (e) {
      console.error(e);
    } finally { setLoading(false); }
  };

  useEffect(() => { load(); }, []);

  const create = async (e: React.FormEvent) => {
    e.preventDefault();
    try {
      const newCase = await invoke<Case>("case_create", {
        input: {
          title: form.title,
          case_number: form.case_number || null,
          description: form.description || null,
          target_email: form.target_email || null,
          target_name: form.target_name || null,
          target_organization: form.target_organization || null,
          investigation_type: form.investigation_type || null,
        }
      });
      setShowCreate(false);
      setForm({ title: "", case_number: "", description: "", target_email: "", target_name: "", target_organization: "", investigation_type: "general" });
      onSelectCase(newCase.id);
    } catch (e) {
      console.error(e);
    }
  };

  const invTypeLabel = (type: string) => {
    const types: Record<string, string> = {
      general: "General",
      fraud: "Fraud",
      bec: "BEC",
      phishing: "Phishing",
      harassment: "Harharment",
      ip: "IP Theft",
      compliance: "Compliance",
      litigation: "Litigation"
    };
    return types[type] || type;
  };

  return (
    <div className="app">
      <header className="topbar">
        <div className="brand">
          <J12Logo size={32} showText={true} />
        </div>
        <div className="row gap-4">
          <span className="muted">admin · investigator</span>
        </div>
      </header>

      <div className="content" style={{ maxWidth: 1000, margin: "0 auto", width: "100%" }}>
        <div className="row between mb-4">
          <div>
            <h2 style={{ fontSize: 24, fontWeight: 700, color: "var(--text-0)", marginBottom: 4 }}>Investigations</h2>
            <p className="muted">Select a case to begin or create a new investigation</p>
          </div>
          <button className="btn btn-primary" onClick={() => setShowCreate(true)}>+ New Case</button>
        </div>

        {showCreate && (
          <div className="card" style={{ maxWidth: 600, marginBottom: 24 }}>
            <h3 style={{ fontSize: 16, fontWeight: 600, marginBottom: 20 }}>Create New Case</h3>
            <form onSubmit={create}>
              <div className="field">
                <label className="label">Case Title *</label>
                <input className="input" required placeholder="e.g. Fraud Investigation Q4 2024" value={form.title}
                  onChange={(e) => setForm({ ...form, title: e.target.value })} />
              </div>
              <div className="grid-2">
                <div className="field">
                  <label className="label">Case Number</label>
                  <input className="input" placeholder="e.g. 2024-001" value={form.case_number}
                    onChange={(e) => setForm({ ...form, case_number: e.target.value })} />
                </div>
                <div className="field">
                  <label className="label">Investigation Type</label>
                  <select className="input" value={form.investigation_type}
                    onChange={(e) => setForm({ ...form, investigation_type: e.target.value })}>
                    <option value="general">General</option>
                    <option value="fraud">Fraud</option>
                    <option value="bec">BEC (Business Email Compromise)</option>
                    <option value="phishing">Phishing</option>
                    <option value="harassment">Harassment</option>
                    <option value="ip">IP Theft</option>
                    <option value="compliance">Compliance</option>
                    <option value="litigation">Litigation</option>
                  </select>
                </div>
              </div>

              <div style={{ margin: "20px 0", padding: "16px", background: "var(--bg-3)", borderRadius: "var(--r-md)" }}>
                <h4 style={{ fontSize: 13, fontWeight: 600, color: "var(--text-1)", marginBottom: 16 }}>Target / Subject Information</h4>
                <div className="grid-2">
                  <div className="field">
                    <label className="label">Target Email Address</label>
                    <input className="input" placeholder="suspect@company.com" value={form.target_email}
                      onChange={(e) => setForm({ ...form, target_email: e.target.value })} />
                  </div>
                  <div className="field">
                    <label className="label">Target Name</label>
                    <input className="input" placeholder="John Smith" value={form.target_name}
                      onChange={(e) => setForm({ ...form, target_name: e.target.value })} />
                  </div>
                </div>
                <div className="field" style={{ marginTop: 12 }}>
                  <label className="label">Target Organization</label>
                  <input className="input" placeholder="Company Inc." value={form.target_organization}
                    onChange={(e) => setForm({ ...form, target_organization: e.target.value })} />
                </div>
              </div>

              <div className="field">
                <label className="label">Description</label>
                <textarea className="textarea" placeholder="Investigation scope, suspects, legal context..." value={form.description}
                  onChange={(e) => setForm({ ...form, description: e.target.value })} />
              </div>
              <div className="row gap-2" style={{ justifyContent: "flex-end" }}>
                <button type="button" className="btn btn-ghost" onClick={() => setShowCreate(false)}>Cancel</button>
                <button type="submit" className="btn btn-primary">Create Case</button>
              </div>
            </form>
          </div>
        )}

        {loading ? (
          <div className="empty">Loading cases...</div>
        ) : cases.length === 0 ? (
          <div className="card" style={{ textAlign: "center", padding: "60px 40px" }}>
            <div style={{ fontSize: 48, marginBottom: 16 }}>📁</div>
            <h3 style={{ fontSize: 18, marginBottom: 8, color: "var(--text-0)" }}>No cases yet</h3>
            <p className="muted mb-4">Create your first investigation to begin analyzing email evidence.</p>
            <button className="btn btn-primary" onClick={() => setShowCreate(true)}>+ Create First Case</button>
          </div>
        ) : (
          <div style={{ display: "grid", gridTemplateColumns: "repeat(auto-fill, minmax(340px, 1fr))", gap: 16 }}>
            {cases.map((c) => (
              <div key={c.id} className="card case-card" onClick={() => onSelectCase(c.id)}>
                <div className="row between mb-4">
                  <span className={`badge badge-${c.status === "open" ? "green" : "gray"}`}>{c.status}</span>
                  <span className="badge badge-blue">{invTypeLabel(c.investigation_type)}</span>
                </div>
                <h3 style={{ fontSize: 16, fontWeight: 600, color: "var(--text-0)", marginBottom: 6 }}>{c.title}</h3>
                {c.case_number && <p className="mono muted" style={{ fontSize: 12, marginBottom: 8 }}>{c.case_number}</p>}
                {(c.target_name || c.target_email) && (
                  <div style={{ padding: "10px 12px", background: "var(--bg-3)", borderRadius: "var(--r-sm)", marginBottom: 12 }}>
                    <div style={{ fontSize: 11, color: "var(--text-3)", marginBottom: 2 }}>TARGET</div>
                    {c.target_name && <div style={{ fontSize: 13, color: "var(--text-1)", fontWeight: 500 }}>{c.target_name}</div>}
                    {c.target_email && <div style={{ fontSize: 12, color: "var(--accent)", fontFamily: "var(--mono)" }}>{c.target_email}</div>}
                    {c.target_organization && <div style={{ fontSize: 11, color: "var(--text-3)", marginTop: 2 }}>{c.target_organization}</div>}
                  </div>
                )}
                <p style={{ fontSize: 13, color: "var(--text-2)", lineHeight: 1.6 }}>
                  {c.description || "No description provided"}
                </p>
                <div className="row between" style={{ marginTop: 12, paddingTop: 12, borderTop: "1px solid var(--border)" }}>
                  <span className="muted" style={{ fontSize: 11 }}>{new Date(c.created_at).toLocaleDateString()}</span>
                  <span style={{ fontSize: 11, color: "var(--accent)" }}>Open →</span>
                </div>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
