export interface TargetProfile {
  case_id: string;
  case_title: string;
  case_number: string;
  target_email: string | null;
  target_name: string | null;
  target_organization: string | null;
  sent_count: number;
  received_count: number;
  total_emails: number;
  first_seen: string | null;
  last_seen: string | null;
  top_correspondents: [string, number][];
  top_subjects: [string, number][];
  display_names: string[];
  x_mailers: string[];
  originating_ips: string[];
  risk_score: number;
  flagged_count: number;
  attachment_count: number;
  recent_communications: Array<{
    id: string;
    subject: string;
    date: string | null;
    from: string;
    to: string;
    risk_score: number;
  }>;
  is_automated?: boolean;
  role?: string;
}

export interface DetectedTarget {
  email: string;
  display_name: string | null;
  organization: string;
  total_emails: number;
  sent: number;
  received: number;
  confidence: string;
  is_primary_target: boolean;
  is_custodian?: boolean;
  is_automated?: boolean;
  role?: string;
  detection_note?: string;
}

export interface TargetProfileProps {
  caseId: string;
  caseData?: any;
  evidenceFilter?: string | null;
  onSelectEmail?: (emailId: string) => void;
}

export function formatName(d: string | null | undefined, email: string): string {
  if (d && d.trim() && d !== email && !d.startsWith("/")) {
    if (d.includes("..")) {
      const p = d.split("..");
      if (p.length === 2) return `${p[0].toUpperCase()}. ${p[1].charAt(0).toUpperCase() + p[1].slice(1)}`;
    }
    return d;
  }
  const local = email.split("@")[0] || email;
  if (local.includes("..")) {
    const p = local.split("..");
    if (p.length === 2) return `${p[0].toUpperCase()}. ${p[1].charAt(0).toUpperCase() + p[1].slice(1)}`;
  } else if (local.includes(".")) {
    return local.split(".").map(w => w.charAt(0).toUpperCase() + w.slice(1)).join(" ");
  }
  return local.charAt(0).toUpperCase() + local.slice(1);
}

export function formatDateSpan(d: string | null): string {
  if (!d) return "N/A";
  try {
    const date = new Date(d);
    return date.toLocaleDateString(undefined, { year: "numeric", month: "short", day: "numeric" });
  } catch {
    return d;
  }
}
