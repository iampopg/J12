export interface SearchEmail {
  id: string;
  evidence_id: string;
  message_id: string | null;
  from_addr: string;
  from_display: string | null;
  to_addrs: string;
  cc_addrs: string;
  subject: string | null;
  date_sent: string | null;
  date_sent_utc: string;
  folder_name?: string | null;
  folder_category: string;
  is_deleted: boolean;
  deleted_recovered: boolean;
  risk_score: number;
  flags: string | null;
  body_text?: string | null;
  headers_raw?: string | null;
  snippet?: string | null;
  match_rank?: number;
}

export type SortField = "date" | "from" | "subject" | "risk" | "rank";

export interface SearchProps {
  caseId: string;
  evidenceFilter?: string | null;
  onSelectEmail?: (email: SearchEmail) => void;
  onViewEntity?: (email: string) => void;
}

export function cleanDisplayName(name: string | null): string {
  if (!name) return "";
  let cleaned = name
    .replace(/@ENRON.*$/g, "")
    .replace(/IMCEANOTES-[^<]*/g, "")
    .replace(/<[^>]*>/g, "")
    .replace(/"/g, "")
    .replace(/\s+/g, " ")
    .trim();
  if (cleaned.includes("@")) {
    return cleaned.split("@")[0].trim() || cleaned;
  }
  return cleaned;
}

export const quickPresets = [
  { label: "⚡ Boolean AND", query: "fraud AND wire" },
  { label: "🔀 Boolean OR", query: "offshore OR confidential" },
  { label: "⛔ Boolean NOT", query: "wire NOT payroll" },
  { label: "📏 Proximity NEAR/5", query: 'NEAR("wire" "transfer", 5)' },
  { label: "🔤 Exact Phrase", query: '"strictly confidential"' },
  { label: "⭐ Prefix Wildcard", query: "crypt*" },
  { label: "🚨 High Risk Flags", query: "risk:high" },
  { label: "🗑️ Deleted Items", query: "is:deleted" },
];

export const operatorChips = [
  { op: "AND", desc: "Both terms required (e.g. fraud AND wire)" },
  { op: "OR", desc: "Either term matches (e.g. wire OR offshore)" },
  { op: "NOT", desc: "Exclude term (e.g. payment NOT salary)" },
  { op: 'NEAR("", 5)', desc: "Within 5 words of each other" },
  { op: '""', desc: 'Exact phrase match (e.g. "secret key")' },
  { op: "*", desc: "Prefix wildcard (e.g. inves*)" },
  { op: "from:", desc: "Sender address or name" },
  { op: "subject:", desc: "Subject line targeting" },
];
