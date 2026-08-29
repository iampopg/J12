export interface TaxonomySubcategorySummary {
  subcategory_id: string;
  name: string;
  count: number;
}

export interface TaxonomyDomainSummary {
  domain_id: string;
  name: string;
  icon: string;
  total_count: number;
  subcategories: TaxonomySubcategorySummary[];
}

export interface ForensicTaxonomyArtifact {
  id: string;
  domain_id: string;
  subcategory_id: string;
  title: string;
  primary_value: string;
  secondary_value: string | null;
  details: string;
  severity: "critical" | "high" | "medium" | "low" | "info";
  artifact_type: "native" | "recovered" | "derived";
  confidence?: "high" | "medium" | "low" | null;
  email_id: string;
  email_subject: string | null;
  email_from: string;
  date_sent_utc: string | null;
  occurrenceCount?: number;
}

export interface EmailMessage {
  id: string;
  evidence_id: string;
  case_id: string;
  message_id: string | null;
  from_addr: string;
  from_display: string | null;
  to_addrs: string;
  cc_addrs: string;
  subject: string | null;
  date_sent: string | null;
  date_sent_utc: string | null;
  headers_raw: string | null;
  body_text: string | null;
  body_html: string | null;
  folder_name: string | null;
  folder_category: string;
  recovery_status: string;
  deleted_recovered: boolean;
  risk_score: number;
  flags: string;
}

export interface ArtifactsProps {
  caseId: string;
  evidenceFilter?: string | null;
  onSelectEmail?: (emailId: string) => void;
}

export const getSeverityBadge = (sev: string) => {
  switch (sev) {
    case "critical": return <span className="badge badge-red">CRITICAL</span>;
    case "high": return <span className="badge badge-orange">HIGH</span>;
    case "medium": return <span className="badge badge-blue">MEDIUM</span>;
    default: return <span className="badge badge-gray">INFO</span>;
  }
};

export const getTypeBadge = (t: string) => {
  switch (t) {
    case "recovered": return <span className="badge" style={{ background: "rgba(239, 68, 68, 0.15)", color: "#ef4444" }}>🗑️ RECOVERED</span>;
    case "derived": return <span className="badge" style={{ background: "rgba(168, 85, 247, 0.15)", color: "#c084fc" }}>🧠 DERIVED</span>;
    default: return <span className="badge" style={{ background: "rgba(56, 189, 248, 0.15)", color: "#38bdf8" }}>📄 NATIVE</span>;
  }
};

export const getConfidenceBadge = (c?: string | null) => {
  const conf = c || "high";
  if (conf === "high") {
    return <span className="badge" style={{ background: "rgba(34, 197, 94, 0.15)", color: "#22c55e", fontSize: 10 }}>✓ VALIDATED</span>;
  } else if (conf === "medium") {
    return <span className="badge" style={{ background: "rgba(234, 179, 8, 0.15)", color: "#eab308", fontSize: 10 }}>⚡ PATTERN</span>;
  }
  return <span className="badge" style={{ background: "rgba(148, 163, 184, 0.15)", color: "#94a3b8", fontSize: 10 }}>HEURISTIC</span>;
};
