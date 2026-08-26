import React, { useState } from "react";

export function DocumentationView() {
  const [activeTab, setActiveTab] = useState<"quickstart" | "taxonomy" | "formats" | "integrity" | "shortcuts">("quickstart");

  return (
    <div style={{ padding: "20px 28px", maxWidth: 1200, margin: "0 auto" }}>
      {/* Header */}
      <div className="row between mb-4" style={{ alignItems: "center", borderBottom: "1px solid var(--border)", paddingBottom: 16 }}>
        <div>
          <div className="row gap-2" style={{ alignItems: "center" }}>
            <span style={{ fontSize: 24 }}>📖</span>
            <h2 style={{ fontSize: 22, fontWeight: 700, color: "var(--text-0)", margin: 0 }}>
              Forensic Suite Documentation &amp; User Guide
            </h2>
          </div>
          <p className="muted" style={{ fontSize: 13, marginTop: 4 }}>
            Operational handbook, evidence standards, taxonomy reference, and forensic shortcuts for J12 Forensic Suite v1.0.0
          </p>
        </div>
        <div className="row gap-2">
          <span className="badge badge-blue" style={{ fontSize: 11, padding: "4px 8px" }}>
            DFIR Certified Standard
          </span>
          <span className="badge badge-green" style={{ fontSize: 11, padding: "4px 8px" }}>
            ISO/IEC 27037 Compliant
          </span>
        </div>
      </div>

      {/* Tabs */}
      <div className="row gap-2 mb-4" style={{ borderBottom: "1px solid var(--border)", paddingBottom: 10 }}>
        <button
          className={`btn btn-sm ${activeTab === "quickstart" ? "btn-primary" : "btn-ghost"}`}
          onClick={() => setActiveTab("quickstart")}
        >
          🚀 Investigative Workflow
        </button>
        <button
          className={`btn btn-sm ${activeTab === "taxonomy" ? "btn-primary" : "btn-ghost"}`}
          onClick={() => setActiveTab("taxonomy")}
        >
          🧩 Evidence Taxonomy
        </button>
        <button
          className={`btn btn-sm ${activeTab === "formats" ? "btn-primary" : "btn-ghost"}`}
          onClick={() => setActiveTab("formats")}
        >
          📁 Supported Formats &amp; Ingestion
        </button>
        <button
          className={`btn btn-sm ${activeTab === "integrity" ? "btn-primary" : "btn-ghost"}`}
          onClick={() => setActiveTab("integrity")}
        >
          🔒 Chain of Custody &amp; Hashes
        </button>
        <button
          className={`btn btn-sm ${activeTab === "shortcuts" ? "btn-primary" : "btn-ghost"}`}
          onClick={() => setActiveTab("shortcuts")}
        >
          ⌨️ Shortcuts &amp; Best Practices
        </button>
      </div>

      {/* Tab 1: Quickstart Workflow */}
      {activeTab === "quickstart" && (
        <div style={{ display: "flex", flexDirection: "column", gap: 20 }}>
          <div className="card" style={{ background: "rgba(15, 23, 42, 0.6)" }}>
            <h3 style={{ fontSize: 16, fontWeight: 700, color: "var(--accent)", marginBottom: 12 }}>
              1. End-to-End Forensic Investigation Lifecycle
            </h3>
            <div style={{ display: "grid", gridTemplateColumns: "repeat(auto-fit, minmax(240px, 1fr))", gap: 16 }}>
              <div style={{ background: "var(--bg-1)", padding: 14, borderRadius: "var(--r-sm)", border: "1px solid var(--border)" }}>
                <div style={{ fontSize: 14, fontWeight: 700, color: "var(--text-0)", marginBottom: 6 }}>
                  Step 1: Evidence Acquisition
                </div>
                <p style={{ fontSize: 12, color: "var(--text-2)", lineHeight: 1.5 }}>
                  Ingest raw mail repositories (.eml, .msg, .mbox, .pst, .ost) or connect directly to live mailboxes via read-only TLS 1.3 IMAP/POP3 acquisition with automated SHA-256 seal calculation.
                </p>
              </div>

              <div style={{ background: "var(--bg-1)", padding: 14, borderRadius: "var(--r-sm)", border: "1px solid var(--border)" }}>
                <div style={{ fontSize: 14, fontWeight: 700, color: "var(--text-0)", marginBottom: 6 }}>
                  Step 2: Triage &amp; Dossier
                </div>
                <p style={{ fontSize: 12, color: "var(--text-2)", lineHeight: 1.5 }}>
                  Inspect the Case Dashboard for risk scoring, spoofing detection, authentication alignment (SPF/DKIM/DMARC), and target profile dossier correlation.
                </p>
              </div>

              <div style={{ background: "var(--bg-1)", padding: 14, borderRadius: "var(--r-sm)", border: "1px solid var(--border)" }}>
                <div style={{ fontSize: 14, fontWeight: 700, color: "var(--text-0)", marginBottom: 6 }}>
                  Step 3: Intelligence &amp; Artifacts
                </div>
                <p style={{ fontSize: 12, color: "var(--text-2)", lineHeight: 1.5 }}>
                  Explore the Artifacts Hub for automatically classified financial, banking, credentials, crypto wallets, and investigative URLs with strict false-positive suppression.
                </p>
              </div>

              <div style={{ background: "var(--bg-1)", padding: 14, borderRadius: "var(--r-sm)", border: "1px solid var(--border)" }}>
                <div style={{ fontSize: 14, fontWeight: 700, color: "var(--text-0)", marginBottom: 6 }}>
                  Step 4: Report &amp; Court Export
                </div>
                <p style={{ fontSize: 12, color: "var(--text-2)", lineHeight: 1.5 }}>
                  Generate comprehensive forensic summary reports, audit logs, chain of custody logs, and export PDF packages ready for legal submission.
                </p>
              </div>
            </div>
          </div>
        </div>
      )}

      {/* Tab 2: Evidence Taxonomy */}
      {activeTab === "taxonomy" && (
        <div style={{ display: "flex", flexDirection: "column", gap: 16 }}>
          <div className="card">
            <h3 style={{ fontSize: 16, fontWeight: 700, color: "var(--text-0)", marginBottom: 8 }}>
              Taxonomy Classification Engine
            </h3>
            <p className="muted" style={{ fontSize: 12.5, marginBottom: 16 }}>
              The forensic engine indexes high-signal evidence across standardized domain classifiers:
            </p>

            <div style={{ display: "grid", gridTemplateColumns: "repeat(auto-fit, minmax(280px, 1fr))", gap: 14 }}>
              <div style={{ background: "var(--bg-1)", padding: 12, borderRadius: "var(--r-sm)", border: "1px solid var(--border)" }}>
                <strong style={{ color: "#22c55e", fontSize: 13 }}>💳 Financial &amp; Banking</strong>
                <ul style={{ fontSize: 12, color: "var(--text-2)", marginTop: 6, paddingLeft: 18, lineHeight: 1.6 }}>
                  <li>Bank Accounts &amp; SWIFT / IBAN numbers</li>
                  <li>Nigerian BVN (Bank Verification Number)</li>
                  <li>NIN (National Identification Number)</li>
                  <li>Credit / Debit card payment references</li>
                </ul>
              </div>

              <div style={{ background: "var(--bg-1)", padding: 12, borderRadius: "var(--r-sm)", border: "1px solid var(--border)" }}>
                <strong style={{ color: "#eab308", fontSize: 13 }}>🪙 Cryptocurrency Wallets</strong>
                <ul style={{ fontSize: 12, color: "var(--text-2)", marginTop: 6, paddingLeft: 18, lineHeight: 1.6 }}>
                  <li>Bitcoin (BTC) Legacy &amp; Bech32 addresses</li>
                  <li>Ethereum (ETH) &amp; ERC-20 hex addresses</li>
                  <li>Tron (TRX / USDT TRC-20) addresses</li>
                  <li>Monero (XMR) stealth addresses</li>
                </ul>
              </div>

              <div style={{ background: "var(--bg-1)", padding: 12, borderRadius: "var(--r-sm)", border: "1px solid var(--border)" }}>
                <strong style={{ color: "#f43f5e", fontSize: 13 }}>🔑 Credentials &amp; Secrets</strong>
                <ul style={{ fontSize: 12, color: "var(--text-2)", marginTop: 6, paddingLeft: 18, lineHeight: 1.6 }}>
                  <li>Cleartext passwords and login pairs</li>
                  <li>API Keys, JWT Tokens &amp; Bearer tokens</li>
                  <li>Private keys &amp; SSH credentials</li>
                </ul>
              </div>

              <div style={{ background: "var(--bg-1)", padding: 12, borderRadius: "var(--r-sm)", border: "1px solid var(--border)" }}>
                <strong style={{ color: "#06b6d4", fontSize: 13 }}>🌐 Investigative Links &amp; Relays</strong>
                <ul style={{ fontSize: 12, color: "var(--text-2)", marginTop: 6, paddingLeft: 18, lineHeight: 1.6 }}>
                  <li>Telegram channels and direct chat links</li>
                  <li>WhatsApp invite links</li>
                  <li>Cloud file storage drops (Drive, Mega, Dropbox)</li>
                  <li>Tor .onion hidden services &amp; shorteners</li>
                </ul>
              </div>
            </div>
          </div>
        </div>
      )}

      {/* Tab 3: Formats */}
      {activeTab === "formats" && (
        <div className="card">
          <h3 style={{ fontSize: 16, fontWeight: 700, color: "var(--text-0)", marginBottom: 12 }}>
            Supported File &amp; Protocol Formats
          </h3>
          <table style={{ width: "100%", fontSize: 12.5, borderCollapse: "collapse" }}>
            <thead>
              <tr style={{ borderBottom: "1px solid var(--border)", textAlign: "left", color: "var(--text-2)" }}>
                <th style={{ padding: "8px 12px" }}>Format</th>
                <th style={{ padding: "8px 12px" }}>Standard</th>
                <th style={{ padding: "8px 12px" }}>Capabilities</th>
                <th style={{ padding: "8px 12px" }}>Integrity Verification</th>
              </tr>
            </thead>
            <tbody>
              <tr style={{ borderBottom: "1px solid var(--border)" }}>
                <td style={{ padding: "10px 12px", fontWeight: 600 }}>.EML / .EMLX</td>
                <td style={{ padding: "10px 12px", color: "var(--text-2)" }}>RFC 822 / 2822 / 5322</td>
                <td style={{ padding: "10px 12px" }}>Single message, full transport headers, multipart attachments</td>
                <td style={{ padding: "10px 12px", color: "var(--accent)" }}>SHA-256 bitstream match</td>
              </tr>
              <tr style={{ borderBottom: "1px solid var(--border)" }}>
                <td style={{ padding: "10px 12px", fontWeight: 600 }}>.MBOX</td>
                <td style={{ padding: "10px 12px", color: "var(--text-2)" }}>RFC 4155 (mboxo/mboxrd)</td>
                <td style={{ padding: "10px 12px" }}>Bulk multi-folder mailbox extraction, zero-corruption parser</td>
                <td style={{ padding: "10px 12px", color: "var(--accent)" }}>Container + Email hashes</td>
              </tr>
              <tr style={{ borderBottom: "1px solid var(--border)" }}>
                <td style={{ padding: "10px 12px", fontWeight: 600 }}>.PST / .OST</td>
                <td style={{ padding: "10px 12px", color: "var(--text-2)" }}>MS-PST (Outlook 97-2016+)</td>
                <td style={{ padding: "10px 12px" }}>Folder hierarchy, soft-deleted item recovery, RTF/HTML decomp</td>
                <td style={{ padding: "10px 12px", color: "var(--accent)" }}>Binary hash seal</td>
              </tr>
              <tr>
                <td style={{ padding: "10px 12px", fontWeight: 600 }}>IMAP4 / POP3</td>
                <td style={{ padding: "10px 12px", color: "var(--text-2)" }}>RFC 3501 / RFC 1939 (TLS)</td>
                <td style={{ padding: "10px 12px" }}>Live remote server acquisition, folder mapping, streaming</td>
                <td style={{ padding: "10px 12px", color: "var(--accent)" }}>Message-level SHA-256</td>
              </tr>
            </tbody>
          </table>
        </div>
      )}

      {/* Tab 4: Integrity */}
      {activeTab === "integrity" && (
        <div className="card">
          <h3 style={{ fontSize: 16, fontWeight: 700, color: "var(--text-0)", marginBottom: 12 }}>
            Digital Chain of Custody &amp; Hash Verification
          </h3>
          <p style={{ fontSize: 13, color: "var(--text-1)", lineHeight: 1.6, marginBottom: 16 }}>
            Every piece of evidence acquired into J12 Forensic Suite is assigned a cryptographic SHA-256 seal at the exact moment of ingestion. The original source file remains completely unmodified in accordance with standard forensic rules of evidence.
          </p>
          <div style={{ background: "var(--bg-1)", padding: 16, borderRadius: "var(--r-sm)", border: "1px solid var(--border)", marginBottom: 16 }}>
            <strong style={{ fontSize: 13, color: "var(--text-0)" }}>Verification Protocol:</strong>
            <p style={{ fontSize: 12, color: "var(--text-2)", marginTop: 6, lineHeight: 1.5 }}>
              Examiners can verify that the raw evidence repository on disk has not suffered bit rot or tampering by navigating to <strong>📁 Case Management ➔ Verify Evidence Integrity</strong>. The system recalculates real-time hashes against the baseline custody log.
            </p>
          </div>
        </div>
      )}

      {/* Tab 5: Shortcuts */}
      {activeTab === "shortcuts" && (
        <div className="card">
          <h3 style={{ fontSize: 16, fontWeight: 700, color: "var(--text-0)", marginBottom: 12 }}>
            Keyboard Shortcuts &amp; Quick Navigation
          </h3>
          <div style={{ display: "grid", gridTemplateColumns: "repeat(auto-fit, minmax(280px, 1fr))", gap: 14 }}>
            <div className="row between" style={{ background: "var(--bg-1)", padding: 10, borderRadius: "var(--r-sm)", border: "1px solid var(--border)" }}>
              <span style={{ fontSize: 12.5 }}>Collapse / Expand Sidebar</span>
              <kbd style={{ background: "var(--bg-3)", padding: "2px 8px", borderRadius: 4, fontFamily: "var(--mono)", fontSize: 11 }}>← / →</kbd>
            </div>
            <div className="row between" style={{ background: "var(--bg-1)", padding: 10, borderRadius: "var(--r-sm)", border: "1px solid var(--border)" }}>
              <span style={{ fontSize: 12.5 }}>Interactive Image Zoom</span>
              <kbd style={{ background: "var(--bg-3)", padding: "2px 8px", borderRadius: 4, fontFamily: "var(--mono)", fontSize: 11 }}>Click Thumbnail</kbd>
            </div>
            <div className="row between" style={{ background: "var(--bg-1)", padding: 10, borderRadius: "var(--r-sm)", border: "1px solid var(--border)" }}>
              <span style={{ fontSize: 12.5 }}>Quick Copy Extracted Artifact</span>
              <kbd style={{ background: "var(--bg-3)", padding: "2px 8px", borderRadius: 4, fontFamily: "var(--mono)", fontSize: 11 }}>📋 Copy</kbd>
            </div>
            <div className="row between" style={{ background: "var(--bg-1)", padding: 10, borderRadius: "var(--r-sm)", border: "1px solid var(--border)" }}>
              <span style={{ fontSize: 12.5 }}>Export Artifacts as CSV</span>
              <kbd style={{ background: "var(--bg-3)", padding: "2px 8px", borderRadius: 4, fontFamily: "var(--mono)", fontSize: 11 }}>📥 Export CSV</kbd>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
