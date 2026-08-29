export interface Entity {
  id: string;
  email_address: string;
  display_name: string | null;
  first_seen: string | null;
  last_seen: string | null;
  sent_count: number;
  received_count: number;
  role: string;
  aliases?: string | null;
}

export interface EntityDetail {
  email: string;
  display_name: string | null;
  first_seen: string | null;
  last_seen: string | null;
  sent_count: number;
  received_count: number;
  deleted_count: number;
  flagged_count: number;
  total_count: number;
  aliases: string[];
  sent_to: [string, number][];
  received_from: [string, number][];
  top_subjects: [string, number][];
}

export interface EntityEmail {
  id: string;
  evidence_id: string;
  from_addr: string;
  from_display: string | null;
  to_addrs: string;
  cc_addrs: string;
  subject: string | null;
  date_sent: string | null;
  date_sent_utc: string;
  risk_score: number;
  folder_category: string;
  is_deleted: boolean;
  deleted_recovered: boolean;
  body_text: string | null;
  body_html?: string | null;
  headers_raw: string | null;
}

export type TabType = "all" | "sent" | "received" | "deleted" | "flagged" | "partners";
export type EntityTier = "key" | "internal" | "all";

export interface EntityDiveProps {
  caseId: string;
  evidenceFilter?: string | null;
  onSelectEmail?: (id: string) => void;
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
