export interface CaseAttachmentItem {
  id: string;
  email_id: string;
  filename: string;
  sha256: string;
  mime_type: string;
  size_bytes: number;
  stored_path: string | null;
  entropy: number | null;
  risk_flags: string | null;
  email_subject: string | null;
  email_from: string;
  email_date: string | null;
  email_risk_score: number;
  category: "dangerous" | "documents" | "images" | "archives" | "media" | "other";
  extracted_text?: string | null;
  ocr_status?: string | null;
}

export interface AttachmentsProps {
  caseId: string;
  evidenceFilter?: string | null;
  onSelectEmail?: (emailId: string) => void;
}

export const formatSize = (bytes: number) => {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(2)} MB`;
};

export const getFileIcon = (cat: string, filename: string) => {
  const lower = filename.toLowerCase();
  if (cat === "dangerous" || lower.endsWith(".exe") || lower.endsWith(".scr") || lower.endsWith(".vbs")) return "🚨";
  if (lower.endsWith(".pdf")) return "📕";
  if (lower.endsWith(".doc") || lower.endsWith(".docx")) return "📘";
  if (lower.endsWith(".xls") || lower.endsWith(".xlsx") || lower.endsWith(".csv")) return "📗";
  if (lower.endsWith(".ppt") || lower.endsWith(".pptx")) return "📙";
  if (cat === "images") return "🖼️";
  if (cat === "archives") return "📦";
  if (cat === "media") return "🎵";
  return "📎";
};

export const isImagePreviewable = (cat: string, name: string) => {
  if (cat === "images") return true;
  const lower = name.toLowerCase();
  return (
    lower.endsWith(".png") ||
    lower.endsWith(".jpg") ||
    lower.endsWith(".jpeg") ||
    lower.endsWith(".gif") ||
    lower.endsWith(".webp") ||
    lower.endsWith(".bmp") ||
    lower.endsWith(".svg") ||
    lower.endsWith(".ico") ||
    lower.endsWith(".pdf")
  );
};
