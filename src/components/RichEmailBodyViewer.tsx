import { useState, useMemo } from "react";

interface Props {
  bodyText?: string | null;
  bodyHtml?: string | null;
  emailId?: string;
  defaultMode?: "rendered" | "text" | "raw";
}

interface ParsedEmailBody {
  cleanHtml: string | null;
  cleanText: string | null;
  raw: string;
  inlineImages: Array<{ id: string; filename: string; mime: string; dataUrl: string }>;
}

export function parseMimeBody(bodyText?: string | null, bodyHtml?: string | null): ParsedEmailBody {
  const raw = bodyHtml || bodyText || "";
  if (!raw) {
    return { cleanHtml: null, cleanText: null, raw: "", inlineImages: [] };
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

  // Check if raw contains multipart boundaries
  let cleanHtml: string | null = null;
  let cleanText: string | null = null;

  if (bodyHtml && !bodyHtml.includes("Content-Type: multipart/")) {
    cleanHtml = bodyHtml;
  }

  // Extract HTML section from multipart
  if (!cleanHtml) {
    const htmlSectionRegex = /Content-Type:\s*text\/html(?:;[^\n]*)?\r?\n(?:Content-Transfer-Encoding:[^\n]*\r?\n)?\r?\n([\s\S]*?)(?=(?:\r?\n--[^\n]+|$))/i;
    const htmlMatch = htmlSectionRegex.exec(raw);
    if (htmlMatch && htmlMatch[1]) {
      let extracted = htmlMatch[1].trim();
      // Handle quoted-printable decoded lines if needed
      extracted = extracted.replace(/=\r?\n/g, "").replace(/=3D/gi, "=").replace(/=20/g, " ");
      cleanHtml = extracted;
    }
  }

  // Extract plain text section
  const textSectionRegex = /Content-Type:\s*text\/plain(?:;[^\n]*)?\r?\n(?:Content-Transfer-Encoding:[^\n]*\r?\n)?\r?\n([\s\S]*?)(?=(?:\r?\n--[^\n]+|$))/i;
  const textMatch = textSectionRegex.exec(raw);
  if (textMatch && textMatch[1]) {
    cleanText = textMatch[1].trim().replace(/=\r?\n/g, "").replace(/=3D/gi, "=");
  } else if (!raw.includes("Content-Type:") && !raw.startsWith("--")) {
    cleanText = raw;
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
  defaultMode = "rendered",
}: Props) {
  const parsed = useMemo(() => parseMimeBody(bodyText, bodyHtml), [bodyText, bodyHtml]);
  
  const hasHtml = Boolean(parsed.cleanHtml);
  const initialMode = hasHtml ? defaultMode : "text";
  const [mode, setMode] = useState<"rendered" | "text" | "raw">(initialMode);
  const [zoomImg, setZoomImg] = useState<{ src: string; filename: string } | null>(null);

  return (
    <div style={{ marginTop: 8 }}>
      {/* Zoom Modal */}
      {zoomImg && (
        <div
          style={{
            position: "fixed",
            inset: 0,
            background: "rgba(0,0,0,0.85)",
            backdropFilter: "blur(6px)",
            display: "flex",
            flexDirection: "column",
            alignItems: "center",
            justifyContent: "center",
            zIndex: 10000,
            padding: 24,
          }}
          onClick={() => setZoomImg(null)}
        >
          <div
            style={{
              maxWidth: "90vw",
              maxHeight: "85vh",
              display: "flex",
              flexDirection: "column",
              alignItems: "center",
              background: "var(--bg-1)",
              borderRadius: "var(--r-md)",
              padding: 16,
              border: "1px solid var(--border)",
              boxShadow: "0 25px 50px rgba(0,0,0,0.7)",
            }}
            onClick={(e) => e.stopPropagation()}
          >
            <div className="row between" style={{ width: "100%", marginBottom: 12 }}>
              <span style={{ fontSize: 14, fontWeight: 700, color: "var(--text-0)" }}>
                🖼️ {zoomImg.filename}
              </span>
              <button className="btn btn-ghost btn-sm" onClick={() => setZoomImg(null)}>✕ Close</button>
            </div>
            <img
              src={zoomImg.src}
              alt={zoomImg.filename}
              style={{ maxWidth: "100%", maxHeight: "75vh", objectFit: "contain", borderRadius: 4 }}
            />
          </div>
        </div>
      )}

      {/* Mode Switcher */}
      <div className="row between mb-2" style={{ alignItems: "center" }}>
        <div className="row gap-1" style={{ background: "var(--bg-2)", padding: 2, borderRadius: "var(--r-sm)", border: "1px solid var(--border)" }}>
          {parsed.cleanHtml && (
            <button
              className={`btn btn-sm ${mode === "rendered" ? "btn-primary" : "btn-ghost"}`}
              style={{ padding: "3px 10px", fontSize: 11 }}
              onClick={() => setMode("rendered")}
            >
              🌐 Rendered HTML
            </button>
          )}
          <button
            className={`btn btn-sm ${mode === "text" ? "btn-primary" : "btn-ghost"}`}
            style={{ padding: "3px 10px", fontSize: 11 }}
            onClick={() => setMode("text")}
          >
            📄 Clean Text
          </button>
          <button
            className={`btn btn-sm ${mode === "raw" ? "btn-primary" : "btn-ghost"}`}
            style={{ padding: "3px 10px", fontSize: 11 }}
            onClick={() => setMode("raw")}
          >
            🧬 Raw MIME
          </button>
        </div>

        {parsed.inlineImages.length > 0 && (
          <span className="badge badge-blue" style={{ fontSize: 11 }}>
            🖼️ {parsed.inlineImages.length} Inline Photos Detected
          </span>
        )}
      </div>

      {/* Embedded Images Strip */}
      {parsed.inlineImages.length > 0 && (
        <div style={{ marginBottom: 12, padding: 10, background: "var(--bg-2)", borderRadius: "var(--r-sm)", border: "1px solid var(--border)" }}>
          <div style={{ fontSize: 11, fontWeight: 700, color: "var(--text-2)", textTransform: "uppercase", marginBottom: 8 }}>
            📎 Embedded Photos &amp; Check Scans ({parsed.inlineImages.length})
          </div>
          <div style={{ display: "flex", gap: 10, overflowX: "auto", paddingBottom: 4 }}>
            {parsed.inlineImages.map((img, idx) => (
              <div
                key={idx}
                style={{
                  flexShrink: 0,
                  width: 140,
                  background: "var(--bg-0)",
                  border: "1px solid var(--border)",
                  borderRadius: 4,
                  padding: 4,
                  cursor: "zoom-in",
                }}
                onClick={() => setZoomImg({ src: img.dataUrl, filename: img.filename })}
                title="Click to view full resolution"
              >
                <img
                  src={img.dataUrl}
                  alt={img.filename}
                  style={{ width: "100%", height: 90, objectFit: "cover", borderRadius: 2 }}
                />
                <div style={{ fontSize: 10, color: "var(--text-1)", marginTop: 4, overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
                  {img.filename}
                </div>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* Main Content Box */}
      {mode === "rendered" && parsed.cleanHtml ? (
        <div
          style={{
            background: "#ffffff",
            color: "#1e293b",
            borderRadius: "var(--r-md)",
            border: "1px solid var(--border)",
            padding: 20,
            maxHeight: 500,
            overflowY: "auto",
            fontSize: 14,
            lineHeight: 1.6,
          }}
          dangerouslySetInnerHTML={{ __html: parsed.cleanHtml }}
        />
      ) : mode === "text" ? (
        <pre
          style={{
            background: "var(--bg-0)",
            border: "1px solid var(--border)",
            borderRadius: "var(--r-md)",
            padding: 16,
            fontSize: 13,
            maxHeight: 450,
            overflow: "auto",
            whiteSpace: "pre-wrap",
            fontFamily: "var(--font-sans)",
            color: "var(--text-0)",
            lineHeight: 1.6,
          }}
        >
          {parsed.cleanText || parsed.cleanHtml?.replace(/<[^>]+>/g, " ") || "(No text content)"}
        </pre>
      ) : (
        <pre
          style={{
            background: "var(--bg-0)",
            border: "1px solid var(--border)",
            borderRadius: "var(--r-md)",
            padding: 16,
            fontSize: 11,
            maxHeight: 450,
            overflow: "auto",
            whiteSpace: "pre-wrap",
            fontFamily: "var(--mono)",
            color: "var(--text-2)",
            lineHeight: 1.5,
          }}
        >
          {parsed.raw}
        </pre>
      )}
    </div>
  );
}
