export interface DailyRecord {
  date: string;
  total: number;
  sent: number;
  received: number;
}

export interface MonthlyRecord {
  month: string;
  total: number;
  sent: number;
  received: number;
}

export interface TimelineEmail {
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
  folder_name: string | null;
  folder_category: string;
  is_deleted: boolean;
  deleted_recovered: boolean;
  risk_score: number;
  flags: string | null;
  body_text: string | null;
  headers_raw: string | null;
}

export type FilterCategory = "all" | "sent" | "received" | "deleted" | "flagged" | "after_hours";

export interface TimelineProps {
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
