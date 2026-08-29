export interface ReportSection {
  id: string;
  title: string;
  description: string;
  enabled: boolean;
}

export interface Exhibit {
  id: string;
  exhibit_number: string;
  email_id: string;
  from_addr: string;
  from_display: string | null;
  subject: string;
  date_sent: string;
  sha256: string;
  notes: string;
}

export interface ReportData {
  case_info: any;
  methodology: any;
  custody_chain: any[];
  evidence_inventory: any[];
  findings: any[];
  entities?: any[];
  email_stats?: any;
  hash_manifest: any[];
  target_profile?: any;
  folder_breakdown?: any[];
  attachments_manifest?: any[];
  key_messages_ledger?: any[];
}

export const REPORT_SECTIONS: ReportSection[] = [
  { id: "case_info", title: "1. Case Overview & Identification", description: "Case metadata, subject identity, examiner and agency information", enabled: true },
  { id: "sources", title: "2. Evidence Sources & Provenance", description: "Container technical specs, file size in bytes, SHA-256 acquisition hashes", enabled: true },
  { id: "exec_summary", title: "3. Executive Analytics & Volume Ledger", description: "Total email counts, sent/received/deleted metrics and temporal spans", enabled: true },
  { id: "folders", title: "4. Mailbox Structure & Folder Hierarchy", description: "Breakdown of folders (Inbox, Sent, Deleted) with item tallies and date spans", enabled: true },
  { id: "findings", title: "5. Security Findings & Tampering Matrix", description: "Full technical descriptions of spoofing, BEC, and risk anomalies", enabled: true },
  { id: "target_dossier", title: "6. Subject Profile & Top Correspondents", description: "Primary case subject profile, discovered aliases, and entity matrix", enabled: true },
  { id: "key_ledger", title: "7. Evidentiary & Flagged Email Ledger", description: "Itemized list of suspicious, high-risk, and recovered deleted messages", enabled: true },
  { id: "attachments", title: "8. Attachments & File Artifacts", description: "Inventory of extracted attachment files, types, and cryptographic hashes", enabled: true },
  { id: "exhibits", title: "9. Marked Court Exhibits", description: "Bookmarked emails entered into formal evidence record with annotations", enabled: true },
  { id: "custody", title: "10. Chain of Custody & Audit Trail", description: "Step-by-step verification history and evidence handling log", enabled: true },
  { id: "certification", title: "11. Methodology & Examiner Certification", description: "Forensic tool versioning, standards compliance, and sworn signature block", enabled: true },
];

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
