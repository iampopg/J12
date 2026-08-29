export interface Finding {
  id: string;
  case_id: string;
  type_: string;
  severity: string;
  confidence: string;
  title: string;
  description: string | null;
  evidence_refs: string;
  email_ids: string;
  status: string;
  created_at: string;
  reviewed_by: string | null;
  reviewed_at: string | null;
  notes: string | null;
}

export interface FindingEmailItem {
  id: string;
  evidence_id: string;
  from_addr: string;
  from_display: string | null;
  to_addrs: string;
  cc_addrs: string;
  subject: string | null;
  date_sent: string | null;
  body_text: string | null;
  body_html: string | null;
  headers_raw: string | null;
  folder_name: string | null;
  folder_category: string;
  risk_score: number;
}

export interface FindingsProps {
  caseId: string;
  evidenceFilter?: string | null;
  onGoToEvidence?: () => void;
}

export const severityColor = (severity: string) => {
  switch (severity.toLowerCase()) {
    case "critical": return "var(--danger)";
    case "high": return "#f97316";
    case "medium": return "#eab308";
    case "low": return "#3b82f6";
    default: return "#6b7280";
  }
};

export const statusBadge = (status: string) => {
  switch (status.toLowerCase()) {
    case "open": return "badge-blue";
    case "confirmed": return "badge-green";
    case "rejected": return "badge-red";
    case "reviewed": return "badge-yellow";
    default: return "badge-gray";
  }
};
