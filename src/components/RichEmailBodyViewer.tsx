import { useState, useMemo, useEffect, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";

export interface CaseAttachmentItem {
  id: string;
  email_id: string;
  filename: string;
  sha256: string;
  mime_type: string;
  size_bytes: number;
  stored_path: string | null;
  entropy?: number | null;
  risk_flags?: string | null;
  category?: string;
}

interface Props {
  bodyText?: string | null;
  bodyHtml?: string | null;
  emailId?: string;
  defaultMode?: "rendered" | "text" | "raw";
  attachments?: CaseAttachmentItem[];
}

interface ParsedEmailBody {
  cleanHtml: string | null;
  cleanText: string | null;
  raw: string;
  inlineImages: Array<{ id: string; filename: string; mime: string; dataUrl: string }>;
}

export function decodeQuotedPrintable(input: string): string {
  if (!input) return "";
  
  // 1. Remove soft line breaks: =\r\n or =\n or =\r
  let text = input.replace(/=\r?\n/g, "").replace(/=\r/g, "");
  
  // 2. Decode hex-encoded bytes: =XX (e.g. =20, =3D, =21, =2E, =22)
  try {
    text = text.replace(/(?:=[0-9A-Fa-f]{2})+/g, (match) => {
      try {
        const hexPairs = match.match(/=[0-9A-Fa-f]{2}/g) || [];
        const bytes = new Uint8Array(hexPairs.map(h => parseInt(h.substring(1), 16)));
        return new TextDecoder("utf-8").decode(bytes);
      } catch {
        return match;
      }
    });
  } catch {
    text = text.replace(/=([0-9A-Fa-f]{2})/g, (_, hex) => {
      try {
        return String.fromCharCode(parseInt(hex, 16));
      } catch {
        return `=${hex}`;
      }
    });
  }
  
  return text;
}

export function parseMimeBody(bodyText?: string | null, bodyHtml?: string | null): ParsedEmailBody {
  let raw = (bodyHtml && bodyHtml.trim().length > 0) ? bodyHtml : (bodyText || "");
  if (!raw) {
    return { cleanHtml: null, cleanText: null, raw: "", inlineImages: [] };
  }

  // Decode Quoted-Printable if detected
  if (raw.includes("=3D") || raw.includes("=21") || raw.includes("=20") || raw.includes("=\r\n") || raw.includes("=\n") || raw.includes("<=21doctype")) {
    raw = decodeQuotedPrintable(raw);
  }

  const inlineImages: Array<{ id: string; filename: string; mime: string; dataUrl: string }> = [];

  // Extract base64 image attachments from MIME parts
  const imagePartRegex = /Content-Type:\s*(image\/[a-zA-Z0-9.-]+)(?:[^\n]*\n)*?(?:Content-Disposition:[^\n]*filename="?([^"\n]+)"?)?(?:[^\n]*\n)*?(?:X-Attachment-Id:\s*([^\s\n]+)|Content-ID:\s*<([^>]+)>)(?:[^\n]*\n)*?\r?\n\r?\n([A-Za-z0-9+/=\r\n\s]+)(?=(?:--[^\n]+|$))/gi;
  
  let match;
  while ((match = imagePartRegex.exec(raw)) !== null) {
    const mime = match[1];
    const filename = match[2] || "attached_image.jpg";
    const cid = match[3] || match[4] || "";
    const b64 = match[5].replace(/\s+/g, "");
    if (b64.length > 50) {
      inlineImages.push({
        id: cid,
        filename,
        mime,
        dataUrl: `data:${mime};base64,${b64}`,
      });
    }
  }

  let cleanHtml: string | null = null;
  let cleanText: string | null = null;

  // 1. If bodyHtml has content
  if (bodyHtml && bodyHtml.trim().length > 0 && !bodyHtml.includes("Content-Type: multipart/")) {
    cleanHtml = decodeQuotedPrintable(bodyHtml);
  }

  // 2. If raw/bodyText contains HTML markup
  if (!cleanHtml) {
    const textToTest = (bodyText || raw).trim();
    if (
      textToTest.toLowerCase().startsWith("<!doctype html") ||
      textToTest.toLowerCase().startsWith("<html") ||
      textToTest.toLowerCase().startsWith("<div") ||
      textToTest.toLowerCase().startsWith("<table") ||
      /<(html|head|body|div|table|h1|h2|p|style|img|a\s+href)[^>]*>/i.test(textToTest)
    ) {
      cleanHtml = textToTest;
    }
  }

  // 3. Extract HTML section from multipart raw MIME
  if (!cleanHtml) {
    const htmlSectionRegex = /Content-Type:\s*text\/html(?:;[^\n]*)?\r?\n(?:Content-Transfer-Encoding:[^\n]*\r?\n)?\r?\n([\s\S]*?)(?=(?:\r?\n--[^\n]+|$))/i;
    const htmlMatch = htmlSectionRegex.exec(raw);
    if (htmlMatch && htmlMatch[1]) {
      cleanHtml = decodeQuotedPrintable(htmlMatch[1].trim());
    }
  }

  // 4. Extract plain text section
  const textSectionRegex = /Content-Type:\s*text\/plain(?:;[^\n]*)?\r?\n(?:Content-Transfer-Encoding:[^\n]*\r?\n)?\r?\n([\s\S]*?)(?=(?:\r?\n--[^\n]+|$))/i;
  const textMatch = textSectionRegex.exec(raw);
  if (textMatch && textMatch[1]) {
    cleanText = decodeQuotedPrintable(textMatch[1].trim());
  } else if (bodyText && !bodyText.startsWith("<") && !bodyText.includes("Content-Type:")) {
    cleanText = decodeQuotedPrintable(bodyText.trim());
  } else if (cleanHtml) {
    // Generate clean text from HTML
    cleanText = cleanHtml
      .replace(/<style[^>]*>[\s\S]*?<\/style>/gi, "")
      .replace(/<script[^>]*>[\s\S]*?<\/script>/gi, "")
      .replace(/<br\s*\/?>/gi, "\n")
      .replace(/<\/p>/gi, "\n\n")
      .replace(/<\/tr>/gi, "\n")
      .replace(/<\/div>/gi, "\n")
      .replace(/<[^>]+>/g, " ")
      .replace(/&nbsp;/gi, " ")
      .replace(/&amp;/gi, "&")
      .replace(/&lt;/gi, "<")
      .replace(/&gt;/gi, ">")
      .replace(/&quot;/gi, '"')
      .replace(/\n\s*\n\s*\n/g, "\n\n")
      .trim();
  } else {
    cleanText = decodeQuotedPrintable(raw.trim());
  }

  // Replace cid: with inline base64 data URLs
  if (cleanHtml && inlineImages.length > 0) {
    for (const img of inlineImages) {
      if (img.id) {
        const cleanCid = img.id.replace(/^<|>$/g, "").trim();
        const cidRegex = new RegExp(`src=["']cid:${cleanCid}["']`, "gi");
        cleanHtml = cleanHtml.replace(cidRegex, `src="${img.dataUrl}"`);
      }
    }
  }

  return { cleanHtml, cleanText, raw, inlineImages };
}

export function RichEmailBodyViewer({
  bodyText,
  bodyHtml,
  emailId,
  defaultMode = "rendered",
  attachments: externalAttachments = [],
}: Props) {
  const parsed = useMemo(() => parseMimeBody(bodyText, bodyHtml), [bodyText, bodyHtml]);
  
  const hasHtml = Boolean(parsed.cleanHtml);
  const [mode, setMode] = useState<"rendered" | "text" | "raw">(hasHtml ? defaultMode : "text");
  const [zoomImg, setZoomImg] = useState<{ src: string; filename: string } | null>(null);
  const [loadedImages, setLoadedImages] = useState<Array<{ id: string; filename: string; mime: string; dataUrl: string }>>([]);
  const [iframeHeight, setIframeHeight] = useState<number>(550);
  const [fullscreen, setFullscreen] = useState<boolean>(false);
  const iframeRef = useRef<HTMLIFrameElement | null>(null);

  // Auto-switch mode when email content changes
  useEffect(() => {
    setMode(hasHtml ? "rendered" : "text");
  }, [hasHtml, emailId]);

  // Load preview data for attachments
  useEffect(() => {
    let isMounted = true;
    async function loadPreviews() {
      const allImgs = [...parsed.inlineImages];
      for (const att of (externalAttachments || [])) {
        const lowerName = (att.filename || "").toLowerCase();
        const isImg = lowerName.endsWith(".png") || lowerName.endsWith(".jpg") || lowerName.endsWith(".jpeg") || lowerName.endsWith(".gif") || lowerName.endsWith(".webp") || (att.mime_type && att.mime_type.startsWith("image/"));
        if (isImg && !allImgs.some(img => img.filename === att.filename)) {
          try {
            const preview = await invoke<string | null>("get_attachment_preview", {
              input: {
                attachment_id: att.id,
                stored_path: att.stored_path
              }
            });
            if (preview && isMounted) {
              allImgs.push({
                id: att.id,
                filename: att.filename,
                mime: att.mime_type || "image/png",
                dataUrl: preview,
              });
            }
          } catch (e) {
            console.error("Failed to load attachment preview:", e);
          }
        }
      }
      if (isMounted) setLoadedImages(allImgs);
    }
    loadPreviews();
    return () => { isMounted = false; };
  }, [emailId, externalAttachments?.length, parsed.inlineImages.length]);

  // Build sandboxed HTML document
  const srcDocHtml = useMemo(() => {
    if (!parsed.cleanHtml) return "";
    let content = parsed.cleanHtml;
    const hasHtmlTag = /<html[^>]*>/i.test(content);
    if (hasHtmlTag) {
      // Inject base tag and responsive styles inside head
      if (/<head[^>]*>/i.test(content)) {
        return content.replace(
          /<head[^>]*>/i,
          `$&<base target="_blank"><style>html,body{margin:0;padding:16px;font-family:-apple-system,BlinkMacSystemFont,"Segoe UI",Roboto,sans-serif;font-size:14px;color:#1e293b;background:#ffffff;word-break:break-word}img{max-width:100%!important;height:auto!important}table{max-width:100%!important}</style>`
        );
      }
      return content;
    }
    return `<!DOCTYPE html>
<html>
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <base target="_blank">
  <style>
    html, body {
      margin: 0;
      padding: 16px;
      font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
      font-size: 14px;
      line-height: 1.6;
      color: #1e293b;
      background: #ffffff;
      word-break: break-word;
      overflow-wrap: break-word;
    }
    img {
      max-width: 100% !important;
      height: auto !important;
    }
    table {
      max-width: 100% !important;
    }
    a {
      color: #2563eb;
    }
  </style>
</head>
<body>
  ${content}
</body>
</html>`;
  }, [parsed.cleanHtml]);

  const handleIframeLoad = () => {
    try {
      if (iframeRef.current && iframeRef.current.contentDocument) {
        const doc = iframeRef.current.contentDocument;
        const scrollH = Math.max(doc.body.scrollHeight, doc.documentElement.scrollHeight, 350);
        setIframeHeight(Math.min(Math.max(scrollH + 30, 400), 1200));
      }
    } catch {}
  };

  const hasNoText = (!parsed.cleanText || parsed.cleanText === "(No text content)" || parsed.cleanText.trim() === "") && !parsed.cleanHtml;

  return (
    <div style={{ marginTop: 8 }}>
      {/* Zoom Modal */}
      {zoomImg && (
        <div
          style={{ position: "fixed", inset: 0, background: "rgba(0,0,0,0.88)", backdropFilter: "blur(8px)", display: "flex", flexDirection: "column", alignItems: "center", justifyContent: "center", zIndex: 100000, padding: 24 }}
          onClick={() => setZoomImg(null)}
        >
          <div style={{ maxWidth: "92vw", maxHeight: "88vh", display: "flex", flexDirection: "column", alignItems: "center", background: "#0f172a", borderRadius: "var(--r-md)", padding: 20, border: "1px solid #334155" }} onClick={e => e.stopPropagation()}>
            <div className="row between" style={{ width: "100%", marginBottom: 12 }}>
              <span style={{ fontSize: 14, fontWeight: 700, color: "#f8fafc" }}>🖼️ {zoomImg.filename}</span>
              <button className="btn btn-ghost btn-sm" onClick={() => setZoomImg(null)}>✕ Close</button>
            </div>
            <img src={zoomImg.src} alt={zoomImg.filename} style={{ maxWidth: "100%", maxHeight: "75vh", objectFit: "contain", borderRadius: 6 }} />
          </div>
        </div>
      )}

      {/* Mode Switcher */}
      <div className="row between mb-2" style={{ alignItems: "center" }}>
        <div className="row gap-1" style={{ background: "var(--bg-2)", padding: 2, borderRadius: "var(--r-sm)", border: "1px solid var(--border)" }}>
          {parsed.cleanHtml && (
            <button className={`btn btn-sm ${mode === "rendered" ? "btn-primary" : "btn-ghost"}`} style={{ padding: "4px 12px", fontSize: 11, fontWeight: 600 }} onClick={() => setMode("rendered")}>
              🌐 Rendered HTML
            </button>
          )}
          <button className={`btn btn-sm ${mode === "text" ? "btn-primary" : "btn-ghost"}`} style={{ padding: "4px 12px", fontSize: 11, fontWeight: 600 }} onClick={() => setMode("text")}>
            📄 Clean Text
          </button>
          <button className={`btn btn-sm ${mode === "raw" ? "btn-primary" : "btn-ghost"}`} style={{ padding: "4px 12px", fontSize: 11, fontWeight: 600 }} onClick={() => setMode("raw")}>
            🧬 Raw MIME
          </button>
        </div>

        <div className="row gap-2" style={{ alignItems: "center" }}>
          {mode === "rendered" && (
            <button className="btn btn-ghost btn-sm" style={{ fontSize: 11, padding: "3px 8px" }} onClick={() => setFullscreen(!fullscreen)}>
              {fullscreen ? "🗗 Standard View" : "⛶ Full Window"}
            </button>
          )}
          {loadedImages.length > 0 && (
            <span className="badge badge-blue" style={{ fontSize: 11, padding: "4px 8px" }}>
              🖼️ {loadedImages.length} Images / Attachments
            </span>
          )}
        </div>
      </div>

      {/* Embedded Images Gallery Strip */}
      {loadedImages.length > 0 && (
        <div style={{ marginBottom: 14, padding: 12, background: "#0f172a", borderRadius: "var(--r-md)", border: "1px solid #334155" }}>
          <div className="row between mb-2" style={{ alignItems: "center" }}>
            <div style={{ fontSize: 11, fontWeight: 700, color: "#38bdf8", textTransform: "uppercase", letterSpacing: "0.5px" }}>
              📎 Attached Images &amp; Scanned Documents ({loadedImages.length})
            </div>
            <span className="muted" style={{ fontSize: 10 }}>Click image to zoom</span>
          </div>
          <div style={{ display: "flex", gap: 10, overflowX: "auto", paddingBottom: 4 }}>
            {loadedImages.map((img, idx) => (
              <div key={idx} style={{ flexShrink: 0, width: 140, background: "#1e293b", border: "1px solid #475569", borderRadius: "var(--r-sm)", padding: 6, cursor: "zoom-in" }} onClick={() => setZoomImg({ src: img.dataUrl, filename: img.filename })}>
                <div style={{ width: "100%", height: 80, background: "#020617", borderRadius: 4, overflow: "hidden", display: "flex", alignItems: "center", justifyContent: "center" }}>
                  <img src={img.dataUrl} alt={img.filename} style={{ maxWidth: "100%", maxHeight: "100%", objectFit: "contain" }} />
                </div>
                <div style={{ fontSize: 10, color: "#f8fafc", fontWeight: 600, marginTop: 4, overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
                  {img.filename}
                </div>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* Main Content Box */}
      {mode === "rendered" && parsed.cleanHtml ? (
        <div style={{ width: "100%", background: "#ffffff", borderRadius: "var(--r-md)", border: "1px solid var(--border)", overflow: "hidden", boxShadow: "0 2px 8px rgba(0,0,0,0.15)" }}>
          <iframe
            ref={iframeRef}
            srcDoc={srcDocHtml}
            onLoad={handleIframeLoad}
            sandbox="allow-popups allow-popups-to-escape-sandbox allow-same-origin"
            style={{ width: "100%", height: fullscreen ? 750 : iframeHeight, border: "none", display: "block" }}
            title="Email Rendered HTML"
          />
        </div>
      ) : mode === "text" ? (
        <div>
          {hasNoText && loadedImages.length > 0 ? (
            <div style={{ padding: 24, background: "var(--bg-1)", border: "1px solid var(--border)", borderRadius: "var(--r-md)", textAlign: "center" }}>
              <div style={{ fontSize: 32, marginBottom: 8 }}>🖼️</div>
              <h5 style={{ fontSize: 14, fontWeight: 700, color: "var(--text-0)", marginBottom: 4 }}>Image-Only Email Message</h5>
              <p className="muted" style={{ fontSize: 12, maxWidth: 460, margin: "0 auto" }}>
                This message contains no plaintext body. The sender attached {loadedImages.length} image files (shown above).
              </p>
            </div>
          ) : (
            <pre style={{ background: "var(--bg-0)", border: "1px solid var(--border)", borderRadius: "var(--r-md)", padding: 18, fontSize: 13, maxHeight: 520, overflow: "auto", whiteSpace: "pre-wrap", wordBreak: "break-word", fontFamily: "var(--font-sans)", color: "var(--text-0)", lineHeight: 1.6 }}>
              {parsed.cleanText || "(No text content)"}
            </pre>
          )}
        </div>
      ) : (
        <pre style={{ background: "var(--bg-0)", border: "1px solid var(--border)", borderRadius: "var(--r-md)", padding: 18, fontSize: 11, maxHeight: 520, overflow: "auto", whiteSpace: "pre-wrap", wordBreak: "break-all", fontFamily: "var(--mono)", color: "var(--text-2)", lineHeight: 1.5 }}>
          {parsed.raw}
        </pre>
      )}
    </div>
  );
}
