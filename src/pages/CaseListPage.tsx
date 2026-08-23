import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";

interface Case {
  id: string;
  title: string;
  case_number: string;
  description: string;
  status: string;
  created_at: string;
}

export function CaseListPage({ onSelectCase }: { onSelectCase: (id: string) => void }) {
  const [cases, setCases] = useState<Case[]>([]);
  const [loading, setLoading] = useState(true);
  const [showCreate, setShowCreate] = useState(false);
  const [form, setForm] = useState({ title: "", case_number: "", description: "" });

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
        }
      });
      setShowCreate(false);
      setForm({ title: "", case_number: "", description: "" });
      onSelectCase(newCase.id);
    } catch (e) {
      console.error(e);
    }
  };

  return (
    <div className="app">
      <header className="topbar">
        <div className="brand">
          <div className="logo">EF</div>
          <div>
            <div className="brand-title">Email Forensic Platform</div>
            <div className="brand-sub">Court-admissible investigation</div>
          </div>
        </div>
        <div className="row gap-4">
          <span className="muted">admin · administrator</span>
        </div>
      </header>

      <div className="content" style={{ maxWidth: 960, margin: "0 auto", width: "100%" }}>
        <div className="row between mb-4">
          <div>
            <h2 style={{ fontSize: 24, fontWeight: 700, color: "var(--text-0)", marginBottom: 4 }}>Investigations</h2>
            <p className="muted">Select a case to begin or create a new investigation</p>
          </div>
          <button className="btn btn-primary" onClick={() => setShowCreate(true)}>+ New Case</button>
        </div>

        {showCreate && (
          <div className="card" style={{ maxWidth: 540, marginBottom: 24 }}>
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
                  <label className="label">Status</label>
                  <select className="input" onChange={(e) => setForm({ ...form, description: e.target.value })}>
                    <option value="open">Open</option>
                    <option value="closed">Closed</option>
                  </select>
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
          <div style={{ display: "grid", gridTemplateColumns: "repeat(auto-fill, minmax(320px, 1fr))", gap: 16 }}>
            {cases.map((c) => (
              <div key={c.id} className="card case-card" onClick={() => onSelectCase(c.id)}>
                <div className="row between mb-4">
                  <span className={`badge badge-${c.status === "open" ? "green" : "gray"}`}>{c.status}</span>
                  <span className="muted" style={{ fontSize: 12 }}>{new Date(c.created_at).toLocaleDateString()}</span>
                </div>
                <h3 style={{ fontSize: 16, fontWeight: 600, color: "var(--text-0)", marginBottom: 6 }}>{c.title}</h3>
                {c.case_number && <p className="mono muted" style={{ fontSize: 12, marginBottom: 8 }}>{c.case_number}</p>}
                <p style={{ fontSize: 13, color: "var(--text-2)", lineHeight: 1.6 }}>
                  {c.description || "No description provided"}
                </p>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
