import { useState } from "react";
import { useExaminerProfile } from "../auth";

interface Props {
  isOpen: boolean;
  onClose: () => void;
}

export function ExaminerProfileModal({ isOpen, onClose }: Props) {
  const { profile, updateProfile, resetProfile } = useExaminerProfile();

  const [form, setForm] = useState({
    fullName: profile.fullName || "",
    title: profile.title || "",
    agency: profile.agency || "",
    badgeNumber: profile.badgeNumber || "",
    email: profile.email || "",
    certifications: profile.certifications || "",
    signatureNotes: profile.signatureNotes || "",
  });

  const [savedToast, setSavedToast] = useState(false);

  if (!isOpen) return null;

  const handleSave = (e: React.FormEvent) => {
    e.preventDefault();
    updateProfile(form);
    setSavedToast(true);
    setTimeout(() => {
      setSavedToast(false);
      onClose();
    }, 800);
  };

  const handleReset = () => {
    resetProfile();
    onClose();
  };

  return (
    <div
      style={{
        position: "fixed",
        inset: 0,
        background: "rgba(0, 0, 0, 0.75)",
        backdropFilter: "blur(6px)",
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
        zIndex: 10000,
        padding: 20,
      }}
      onClick={onClose}
    >
      <div
        className="card"
        style={{
          maxWidth: 620,
          width: "100%",
          maxHeight: "90vh",
          overflowY: "auto",
          background: "var(--bg-1)",
          border: "1px solid var(--border)",
          boxShadow: "0 25px 50px -12px rgba(0, 0, 0, 0.7)",
          padding: 24,
          borderRadius: "var(--r-md)",
        }}
        onClick={(e) => e.stopPropagation()}
      >
        <div className="row between mb-4" style={{ alignItems: "center", borderBottom: "1px solid var(--border)", paddingBottom: 14 }}>
          <div className="row gap-2" style={{ alignItems: "center" }}>
            <span style={{ fontSize: 24 }}>🛡️</span>
            <div>
              <h3 style={{ fontSize: 17, fontWeight: 700, margin: 0, color: "var(--text-0)" }}>
                Examiner Profile &amp; Forensic Credentials
              </h3>
              <p className="muted" style={{ margin: 0, fontSize: 12 }}>
                Credentials automatically stamped onto generated reports, audit logs &amp; chain of custody
              </p>
            </div>
          </div>
          <button className="btn btn-ghost btn-sm" onClick={onClose}>✕</button>
        </div>

        {savedToast && (
          <div style={{ background: "rgba(34, 197, 94, 0.15)", border: "1px solid #22c55e", color: "#4ade80", padding: "8px 14px", borderRadius: 6, marginBottom: 16, fontSize: 13, fontWeight: 600 }}>
            ✓ Examiner Credentials Saved Successfully!
          </div>
        )}

        <form onSubmit={handleSave}>
          <div className="grid-2">
            <div className="field">
              <label className="label">Examiner Full Name *</label>
              <input
                className="input"
                required
                value={form.fullName}
                onChange={(e) => setForm({ ...form, fullName: e.target.value })}
                placeholder="e.g. Special Agent John Miller"
              />
            </div>
            <div className="field">
              <label className="label">Professional Title</label>
              <input
                className="input"
                value={form.title}
                onChange={(e) => setForm({ ...form, title: e.target.value })}
                placeholder="e.g. Lead Forensic Investigator"
              />
            </div>
          </div>

          <div className="grid-2" style={{ marginTop: 12 }}>
            <div className="field">
              <label className="label">Agency / Laboratory / Organization</label>
              <input
                className="input"
                value={form.agency}
                onChange={(e) => setForm({ ...form, agency: e.target.value })}
                placeholder="e.g. State Cyber Crimes Division"
              />
            </div>
            <div className="field">
              <label className="label">Badge / Credential ID</label>
              <input
                className="input"
                value={form.badgeNumber}
                onChange={(e) => setForm({ ...form, badgeNumber: e.target.value })}
                placeholder="e.g. CCD-9042"
              />
            </div>
          </div>

          <div className="grid-2" style={{ marginTop: 12 }}>
            <div className="field">
              <label className="label">Official Contact Email</label>
              <input
                className="input"
                type="email"
                value={form.email}
                onChange={(e) => setForm({ ...form, email: e.target.value })}
                placeholder="e.g. examiner@dfir-lab.gov"
              />
            </div>
            <div className="field">
              <label className="label">Certifications / Qualifications</label>
              <input
                className="input"
                value={form.certifications}
                onChange={(e) => setForm({ ...form, certifications: e.target.value })}
                placeholder="e.g. GCFA, EnCE, CCE, CISSP"
              />
            </div>
          </div>

          <div className="field" style={{ marginTop: 12 }}>
            <label className="label">Forensic Certification / Signature Note</label>
            <input
              className="input"
              value={form.signatureNotes}
              onChange={(e) => setForm({ ...form, signatureNotes: e.target.value })}
              placeholder="e.g. Certified Digital Evidence Handling & ISO 27037 Compliance"
            />
          </div>

          {/* Live Preview Card */}
          <div style={{ marginTop: 18, padding: 14, background: "var(--bg-2)", borderRadius: "var(--r-sm)", border: "1px dashed var(--border)" }}>
            <div style={{ fontSize: 11, fontWeight: 700, color: "var(--text-3)", letterSpacing: "0.05em", marginBottom: 8 }}>
              OFFICIAL REPORT SIGNATURE BLOCK PREVIEW
            </div>
            <div style={{ fontSize: 13, fontWeight: 700, color: "var(--text-0)" }}>
              {form.fullName || "Examiner Name"}
            </div>
            <div style={{ fontSize: 11.5, color: "var(--accent)" }}>
              {form.title || "Forensic Investigator"} · {form.agency || "Forensic Lab"}
            </div>
            <div style={{ fontSize: 11, color: "var(--text-3)", marginTop: 4 }}>
              Badge: <strong style={{ color: "var(--text-1)" }}>{form.badgeNumber || "N/A"}</strong> | Credentials: <strong style={{ color: "var(--text-1)" }}>{form.certifications || "Certified"}</strong>
            </div>
          </div>

          <div className="row between mt-4" style={{ alignItems: "center" }}>
            <button
              type="button"
              className="btn btn-ghost btn-sm"
              style={{ color: "var(--text-3)", fontSize: 11 }}
              onClick={handleReset}
            >
              ↻ Reset to Default
            </button>
            <div className="row gap-2">
              <button type="button" className="btn btn-ghost" onClick={onClose}>
                Cancel
              </button>
              <button type="submit" className="btn btn-primary" style={{ fontWeight: 700 }}>
                💾 Save Credentials
              </button>
            </div>
          </div>
        </form>
      </div>
    </div>
  );
}
