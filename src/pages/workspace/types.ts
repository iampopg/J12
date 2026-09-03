export interface Case {
  id: string;
  title: string;
  case_number: string;
  description: string;
  status: string;
  target_email: string | null;
  target_name: string | null;
  target_organization: string | null;
  investigation_type: string;
  working_dir?: string | null;
}

export interface Evidence {
  id: string;
  case_id: string;
  filename: string;
  format: string;
  sha256: string;
  size_bytes: number;
  parse_status: string;
  message_count: number;
  deleted_recovered: number;
  acquired_at: string;
  source_description: string;
  parse_error: string | null;
}

export interface Dashboard {
  evidence_count: number;
  email_count: number;
  deleted_recovered: number;
  entity_count: number;
  finding_count: number;
  severity_breakdown: Record<string, number>;
  date_range: [string | null, string | null];
  sent_count: number;
  inbox_count: number;
  important_count?: number;
  soft_deleted_count: number;
  drafts_count: number;
  spam_count: number;
  other_count: number;
  high_risk_emails: number;
  top_correspondents?: Array<{ email: string; sent: number; received: number }>;
}

export type View =
  | "dashboard"
  | "evidence"
  | "emails"
  | "sent"
  | "inbox"
  | "drafts"
  | "soft_deleted"
  | "spam"
  | "other"
  | "search"
  | "timeline"
  | "graph"
  | "entities"
  | "findings"
  | "custody"
  | "target"
  | "notes"
  | "case_manage"
  | "report"
  | "integrity"
  | "artifacts"
  | "attachments"
  | "docs"
  | "ai_setup"
  | "locker";

export type FolderFilter = "all" | "inbox" | "sent" | "drafts" | "soft_deleted" | "spam" | "other";

export function cleanDisplayName(name: string | null): string {
  if (!name) return "";
  let n = name.trim();
  n = n.replace(/^['"]+|['"]+$/g, "");
  if (n.startsWith("/O=") || n.startsWith("/o=")) {
    const parts = n.split("/");
    for (const part of parts) {
      if (part.toUpperCase().startsWith("CN=")) {
        return part.substring(3).trim();
      }
    }
    return n;
  }
  if (n.includes(",")) {
    const parts = n.split(",");
    if (parts.length === 2) {
      const last = parts[0].trim();
      const first = parts[1].trim();
      if (!first.includes(" ") && !last.includes(" ")) {
        return `${first} ${last}`;
      }
    }
  }
  n = n.replace(/<.*$/, "").trim();
  return n;
}
