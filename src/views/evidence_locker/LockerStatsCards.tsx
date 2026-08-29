interface Props {
  stats: {
    total: number;
    emails: number;
    attachments: number;
    artifacts: number;
    findings: number;
    withNotes: number;
  };
}

export function LockerStatsCards({ stats }: Props) {
  return (
    <div
      style={{
        display: "grid",
        gridTemplateColumns: "repeat(auto-fit, minmax(160px, 1fr))",
        gap: 12,
      }}
    >
      <div className="card" style={{ padding: "14px 18px", borderLeft: "4px solid var(--accent-blue)" }}>
        <div style={{ fontSize: 11, fontWeight: 700, color: "var(--text-2)", textTransform: "uppercase" }}>
          Total Evidence Items
        </div>
        <div style={{ fontSize: 24, fontWeight: 800, color: "var(--text-0)", marginTop: 4 }}>
          {stats.total.toLocaleString()}
        </div>
        <div style={{ fontSize: 11, color: "var(--text-2)", marginTop: 2 }}>All tagged records</div>
      </div>

      <div className="card" style={{ padding: "14px 18px", borderLeft: "4px solid #3b82f6" }}>
        <div style={{ fontSize: 11, fontWeight: 700, color: "var(--text-2)", textTransform: "uppercase" }}>
          ✉️ Tagged Emails
        </div>
        <div style={{ fontSize: 24, fontWeight: 800, color: "var(--text-0)", marginTop: 4 }}>
          {stats.emails.toLocaleString()}
        </div>
        <div style={{ fontSize: 11, color: "var(--text-2)", marginTop: 2 }}>Key correspondence</div>
      </div>

      <div className="card" style={{ padding: "14px 18px", borderLeft: "4px solid #10b981" }}>
        <div style={{ fontSize: 11, fontWeight: 700, color: "var(--text-2)", textTransform: "uppercase" }}>
          📎 Tagged Attachments
        </div>
        <div style={{ fontSize: 24, fontWeight: 800, color: "var(--text-0)", marginTop: 4 }}>
          {stats.attachments.toLocaleString()}
        </div>
        <div style={{ fontSize: 11, color: "var(--text-2)", marginTop: 2 }}>Files &amp; images</div>
      </div>

      <div className="card" style={{ padding: "14px 18px", borderLeft: "4px solid #8b5cf6" }}>
        <div style={{ fontSize: 11, fontWeight: 700, color: "var(--text-2)", textTransform: "uppercase" }}>
          🧩 Tagged Artifacts
        </div>
        <div style={{ fontSize: 24, fontWeight: 800, color: "var(--text-0)", marginTop: 4 }}>
          {stats.artifacts.toLocaleString()}
        </div>
        <div style={{ fontSize: 11, color: "var(--text-2)", marginTop: 2 }}>Credentials &amp; forensics</div>
      </div>

      <div className="card" style={{ padding: "14px 18px", borderLeft: "4px solid #f59e0b" }}>
        <div style={{ fontSize: 11, fontWeight: 700, color: "var(--text-2)", textTransform: "uppercase" }}>
          🎯 Tagged Findings
        </div>
        <div style={{ fontSize: 24, fontWeight: 800, color: "var(--text-0)", marginTop: 4 }}>
          {stats.findings.toLocaleString()}
        </div>
        <div style={{ fontSize: 11, color: "var(--text-2)", marginTop: 2 }}>Forensic observations</div>
      </div>

      <div className="card" style={{ padding: "14px 18px", borderLeft: "4px solid #ec4899" }}>
        <div style={{ fontSize: 11, fontWeight: 700, color: "var(--text-2)", textTransform: "uppercase" }}>
          📝 With Notes
        </div>
        <div style={{ fontSize: 24, fontWeight: 800, color: "var(--text-0)", marginTop: 4 }}>
          {stats.withNotes.toLocaleString()}
        </div>
        <div style={{ fontSize: 11, color: "var(--text-2)", marginTop: 2 }}>Annotated items</div>
      </div>
    </div>
  );
}
