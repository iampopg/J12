export interface Email {
  id: string;
  evidence_id: string;
  message_id: string | null;
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
  flags?: string | null;
  is_deleted?: boolean;
  recovery_status: string;
  deleted_recovered: boolean;
  risk_score: number;
  attachment_count?: number;
  image_count?: number;
}

export interface ColumnSettings {
  name: boolean;
  from: boolean;
  to: boolean;
  subject: boolean;
  attachments: boolean;
  date: boolean;
  folder: boolean;
  risk: boolean;
  tag: boolean;
}

export const DEFAULT_COLUMNS: ColumnSettings = {
  name: true,
  from: true,
  to: false,
  subject: true,
  attachments: true,
  date: true,
  folder: true,
  risk: true,
  tag: true,
};

export interface ColumnWidths {
  name: number;
  from: number;
  to: number;
  subject: number;
  attachments: number;
  date: number;
  folder: number;
  risk: number;
  tag: number;
}

export const DEFAULT_COL_WIDTHS: ColumnWidths = {
  name: 150,
  from: 180,
  to: 160,
  subject: 320,
  attachments: 85,
  date: 105,
  folder: 85,
  risk: 65,
  tag: 65,
};

export interface EmailTag {
  id: string;
  case_id: string;
  email_id: string;
  tag: string;
  color: string;
  created_by: string;
  created_at: string;
}

export interface Evidence {
  id: string;
  filename: string;
}

export type SortField = "date" | "name" | "from" | "subject" | "risk" | "folder";
export type SortDir = "asc" | "desc";

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
    const parts = cleaned.split("@");
    return parts[0].trim() || cleaned;
  }
  return cleaned;
}

export function formatBytes(bytes: number): string {
  if (bytes === 0) return "0 B";
  const k = 1024;
  const sizes = ["B", "KB", "MB", "GB"];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + " " + sizes[i];
}
