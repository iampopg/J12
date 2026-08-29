export interface ItemBookmark {
  id: string;
  case_id: string;
  item_id: string;
  item_type: "email" | "attachment" | "finding" | "artifact";
  label: string;
  color: string;
  note: string;
  created_at: string;
  item_title?: string | null;
  item_from?: string | null;
  item_date?: string | null;
}

export interface EvidenceLockerProps {
  caseId: string;
  evidenceFilter?: string | null;
  onNavigate?: (view: string, filter?: string) => void;
}

export function getItemTypeBadge(type: string) {
  switch (type) {
    case "email":
      return { label: "EMAIL", icon: "✉️", color: "var(--accent-blue)" };
    case "attachment":
      return { label: "ATTACHMENT", icon: "📎", color: "var(--accent-green)" };
    case "artifact":
      return { label: "ARTIFACT", icon: "🧩", color: "#8b5cf6" };
    case "finding":
      return { label: "FINDING", icon: "🎯", color: "var(--accent-amber)" };
    default:
      return { label: type.toUpperCase(), icon: "📄", color: "var(--text-2)" };
  }
}
