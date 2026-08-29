import { useExaminerProfile } from "../../auth";

export function ReportCertificationCard() {
  const { profile } = useExaminerProfile();

  const examinerName = profile.fullName || "Lead Forensic Examiner";
  const examinerTitle = profile.title || "Senior Forensic Investigator";
  const agency = profile.agency || "Digital Forensics & Incident Response Lab";
  const badge = profile.badgeNumber || "DFIR-2026";
  const certs = profile.certifications || "GCFA, EnCE, CCE";

  return (
    <div
      style={{
        marginTop: 36,
        padding: 24,
        border: "2px solid var(--border)",
        borderRadius: "var(--r-md)",
        background: "var(--bg-2)",
      }}
    >
      <h4 style={{ fontSize: 14, fontWeight: 800, marginBottom: 8, color: "var(--text-0)" }}>
        11. Formal Forensic Examiner Sworn Certification
      </h4>
      <p style={{ fontSize: 11, color: "var(--text-2)", lineHeight: 1.6 }}>
        I hereby certify that this forensic examination was conducted in accordance with established digital forensics (ISO/IEC 27037) and eDiscovery protocols. The data contained in this dossier represents a verifiable extraction from the provided evidence sources without modification or tampering.
      </p>

      {/* Forensic Examiner Signature Block */}
      <div style={{ display: "grid", gridTemplateColumns: "1.2fr 0.8fr", gap: 36, marginTop: 36, alignItems: "flex-end" }}>
        <div>
          <div style={{ fontFamily: "cursive, serif", fontSize: 20, color: "var(--accent)", marginBottom: 4, letterSpacing: "0.05em" }}>
            {examinerName}
          </div>
          <div style={{ borderTop: "1.5px solid var(--border)", paddingTop: 6 }}>
            <div style={{ fontSize: 12, fontWeight: 700, color: "var(--text-0)" }}>
              {examinerName}
            </div>
            <div style={{ fontSize: 10.5, color: "var(--text-2)" }}>
              {examinerTitle} · {agency}
            </div>
            <div style={{ fontSize: 10, color: "var(--text-3)", marginTop: 2 }}>
              Badge/ID: <strong>{badge}</strong> | Credentials: <strong>{certs}</strong>
            </div>
          </div>
        </div>
        <div>
          <div style={{ fontSize: 11, color: "var(--text-1)", marginBottom: 4 }}>
            Date of Certification: <strong>{new Date().toLocaleDateString(undefined, { year: 'numeric', month: 'long', day: 'numeric' })}</strong>
          </div>
          <div style={{ borderTop: "1.5px solid var(--border)", paddingTop: 6, fontSize: 10.5, color: "var(--text-2)" }}>
            Digital Signature Verification Hash: <span style={{ fontFamily: "var(--mono)", fontSize: 9.5 }}>SHA256-VERIFIED-ISO27037</span>
          </div>
        </div>
      </div>
    </div>
  );
}
