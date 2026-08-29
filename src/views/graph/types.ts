export interface GraphNode {
  id: string;
  name: string | null;
  sent: number;
  received: number;
  total: number;
  is_target?: boolean;
  x: number;
  y: number;
  vx: number;
  vy: number;
}

export interface GraphEdge {
  source: string;
  target: string;
  weight: number;
}

export interface ExchangedEmail {
  id: string;
  from_addr: string;
  from_display: string | null;
  to_addrs: string;
  subject: string | null;
  date_sent_utc: string;
  risk_score: number;
  body_text: string | null;
}

export interface GraphProps {
  caseId: string;
  evidenceFilter?: string | null;
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
