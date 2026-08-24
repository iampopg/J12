import { invoke } from "@tauri-apps/api/core";

interface FooterProps {
  compact?: boolean;
  style?: React.CSSProperties;
}

export function FooterSignature({ compact = false, style }: FooterProps) {
  const openLink = async (url: string) => {
    try {
      await invoke("open_external_url", { url });
    } catch {
      window.open(url, "_blank");
    }
  };

  if (compact) {
    return (
      <div
        style={{
          padding: "10px 14px",
          borderTop: "1px solid var(--border)",
          fontSize: 11,
          color: "var(--text-3)",
          display: "flex",
          flexDirection: "column",
          gap: 6,
          ...style,
        }}
      >
        <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between" }}>
          <span>
            By{" "}
            <a
              href="#"
              onClick={(e) => {
                e.preventDefault();
                openLink("https://github.com/iampopg");
              }}
              style={{
                color: "var(--accent)",
                textDecoration: "none",
                fontWeight: 600,
              }}
              title="Visit @iampopg on GitHub"
            >
              @iampopg
            </a>
          </span>
          <span style={{ fontSize: 10, opacity: 0.7 }}>v1.0.0</span>
        </div>
        <button
          className="btn btn-ghost btn-sm"
          style={{
            width: "100%",
            fontSize: 11,
            padding: "4px 8px",
            color: "#eab308",
            border: "1px solid rgba(234, 179, 8, 0.25)",
            background: "rgba(234, 179, 8, 0.05)",
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
            gap: 6,
          }}
          onClick={() => openLink("https://github.com/iampopg/J12")}
          title="Star J12 on GitHub"
        >
          ⭐ Star on GitHub
        </button>
      </div>
    );
  }

  return (
    <footer
      style={{
        display: "flex",
        alignItems: "center",
        justifyContent: "space-between",
        padding: "16px 24px",
        marginTop: 32,
        borderTop: "1px solid var(--border)",
        fontSize: 12,
        color: "var(--text-3)",
        flexWrap: "wrap",
        gap: 12,
        ...style,
      }}
    >
      <div style={{ display: "flex", alignItems: "center", gap: 8 }}>
        <span style={{ fontWeight: 700, color: "var(--text-1)" }}>
          <span style={{ color: "#ffffff" }}>J</span>
          <span style={{ color: "#22c55e" }}>12</span> Forensic Suite
        </span>
        <span>•</span>
        <span>
          Developed with precision by{" "}
          <a
            href="#"
            onClick={(e) => {
              e.preventDefault();
              openLink("https://github.com/iampopg");
            }}
            style={{
              color: "var(--accent)",
              textDecoration: "none",
              fontWeight: 700,
            }}
            title="Visit author profile on GitHub"
          >
            @iampopg
          </a>
        </span>
      </div>

      <div style={{ display: "flex", alignItems: "center", gap: 14 }}>
        <button
          className="btn btn-ghost btn-sm"
          style={{
            color: "#eab308",
            border: "1px solid rgba(234, 179, 8, 0.3)",
            background: "rgba(234, 179, 8, 0.06)",
            fontWeight: 600,
            display: "flex",
            alignItems: "center",
            gap: 6,
            padding: "4px 12px",
            fontSize: 12,
          }}
          onClick={() => openLink("https://github.com/iampopg/J12")}
          title="Open repository and star this project on GitHub"
        >
          ⭐ Star This Project on GitHub
        </button>
        <span style={{ fontFamily: "var(--mono)", fontSize: 11, opacity: 0.7 }}>
          v1.0.0
        </span>
      </div>
    </footer>
  );
}
