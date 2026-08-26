import { useState, useEffect, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";

interface AIMessage {
  id: string;
  role: "user" | "assistant";
  content: string;
  timestamp: number;
}

interface Props {
  caseId: string;
  aiEnabled: boolean;
  aiConfig: {
    provider: string;
    api_key: string;
    model: string;
    endpoint: string;
  };
}

export function AIChatWidget({ caseId, aiEnabled, aiConfig }: Props) {
  const [isOpen, setIsOpen] = useState(false);
  const [messages, setMessages] = useState<AIMessage[]>([]);
  const [input, setInput] = useState("");
  const [loading, setLoading] = useState(false);
  const messagesEndRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [messages]);

  const sendMessage = async () => {
    if (!input.trim() || loading) return;

    const userMessage: AIMessage = {
      id: `msg_${Date.now()}`,
      role: "user",
      content: input,
      timestamp: Date.now(),
    };

    setMessages(prev => [...prev, userMessage]);
    setInput("");
    setLoading(true);

    try {
      // Build context from case data
      let caseContext = "You are investigating case: " + caseId + "\n\n";
      
      try {
        const stats = await invoke<any>("ai_get_case_statistics", { caseId });
        caseContext += `Case Statistics:
- Total Emails: ${stats.total_emails}
- Inbox: ${stats.inbox_count}
- Sent: ${stats.sent_count}
- Deleted: ${stats.deleted_count}
- Spam: ${stats.spam_count}
- Entities: ${stats.total_entities}
- Attachments: ${stats.total_attachments}
- Findings: ${stats.total_findings}

`;
      } catch (e) {
        // Ignore errors fetching stats
      }

      // Get findings
      try {
        const findings = await invoke<any[]>("ai_get_findings", { caseId });
        if (findings && findings.length > 0) {
          caseContext += `Forensic Findings (${findings.length}):
`;
          for (const f of findings.slice(0, 10)) {
            caseContext += `- [${f.severity.toUpperCase()}] ${f.title}: ${f.description || "No description"}\n`;
          }
          caseContext += "\n";
        }
      } catch (e) {
        // Ignore errors fetching findings
      }

      // Get entities
      try {
        const entities = await invoke<any[]>("get_entity_list", { caseId });
        if (entities && entities.length > 0) {
          caseContext += `Top Entities (${entities.length}):
`;
          for (const e of entities.slice(0, 10)) {
            caseContext += `- ${e.display_name || e.email_address} (Sent: ${e.sent_count}, Received: ${e.received_count})\n`;
          }
          caseContext += "\n";
        }
      } catch (e) {
        // Ignore errors fetching entities
      }

      // Call AI through backend (avoids CORS)
      const response = await invoke<string>("ai_chat", {
        input: {
          provider: aiConfig.provider,
          api_key: aiConfig.api_key,
          model: aiConfig.model,
          endpoint: aiConfig.endpoint,
          prompt: caseContext + "\n\nUser Question: " + input + "\n\nPlease analyze the case data above and provide specific evidence-based conclusions.",
        },
      });

      const assistantMessage: AIMessage = {
        id: `msg_${Date.now()}_resp`,
        role: "assistant",
        content: response,
        timestamp: Date.now(),
      };

      setMessages(prev => [...prev, assistantMessage]);
    } catch (e: any) {
      const errorMessage: AIMessage = {
        id: `msg_${Date.now()}_err`,
        role: "assistant",
        content: `Error: ${e.message || e || "Failed to get response"}`,
        timestamp: Date.now(),
      };
      setMessages(prev => [...prev, errorMessage]);
    }

    setLoading(false);
  };

  // Don't render if AI is not enabled
  if (!aiEnabled) return null;

  return (
    <>
      {/* Floating AI Button */}
      <button
        onClick={() => setIsOpen(!isOpen)}
        style={{
          position: "fixed",
          bottom: 24,
          right: 24,
          width: 56,
          height: 56,
          borderRadius: "50%",
          background: "var(--accent)",
          color: "#fff",
          border: "none",
          cursor: "pointer",
          boxShadow: "0 4px 20px rgba(59, 130, 246, 0.4)",
          zIndex: 9999,
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
          fontSize: 24,
        }}
        title="AI Investigator"
      >
        🤖
      </button>

      {/* Chat Panel */}
      {isOpen && (
        <div
          style={{
            position: "fixed",
            bottom: 88,
            right: 24,
            width: 380,
            height: 500,
            background: "var(--bg-1)",
            border: "1px solid var(--border)",
            borderRadius: "var(--r-md)",
            boxShadow: "0 10px 40px rgba(0,0,0,0.3)",
            zIndex: 9999,
            display: "flex",
            flexDirection: "column",
          }}
        >
          {/* Header */}
          <div
            style={{
              padding: 12,
              borderBottom: "1px solid var(--border)",
              display: "flex",
              justifyContent: "space-between",
              alignItems: "center",
            }}
          >
            <div className="row gap-2">
              <span>🤖</span>
              <span style={{ fontSize: 13, fontWeight: 600 }}>AI Investigator</span>
              <span className="badge badge-green" style={{ fontSize: 9 }}>{aiConfig.provider}</span>
            </div>
            <button
              onClick={() => setIsOpen(false)}
              className="btn btn-ghost btn-sm"
              style={{ padding: "2px 8px" }}
            >
              ✕
            </button>
          </div>

          {/* Messages */}
          <div style={{ flex: 1, overflowY: "auto", padding: 12 }}>
            {messages.length === 0 ? (
              <div style={{ textAlign: "center", padding: 40, color: "var(--text-3)" }}>
                <div style={{ fontSize: 32, marginBottom: 8 }}>🤖</div>
                <p style={{ fontSize: 12 }}>
                  Ask me anything about your case evidence.
                </p>
              </div>
            ) : (
              messages.map(msg => (
                <div
                  key={msg.id}
                  style={{
                    marginBottom: 8,
                    padding: 8,
                    background: msg.role === "user" ? "rgba(59, 130, 246, 0.1)" : "var(--bg-3)",
                    borderRadius: "var(--r-sm)",
                    fontSize: 12,
                  }}
                >
                  <div style={{ fontSize: 10, color: "var(--text-3)", marginBottom: 4 }}>
                    {msg.role === "user" ? "You" : "AI"}
                  </div>
                  <div style={{ lineHeight: 1.5, whiteSpace: "pre-wrap" }}>{msg.content}</div>
                </div>
              ))
            )}
            {loading && (
              <div style={{ textAlign: "center", padding: 8, color: "var(--text-3)", fontSize: 11 }}>
                AI is thinking...
              </div>
            )}
            <div ref={messagesEndRef} />
          </div>

          {/* Input */}
          <div style={{ padding: 12, borderTop: "1px solid var(--border)" }}>
            <div className="row gap-2">
              <input
                className="input"
                value={input}
                onChange={e => setInput(e.target.value)}
                onKeyDown={e => e.key === "Enter" && sendMessage()}
                placeholder="Ask about evidence..."
                style={{ flex: 1, fontSize: 12 }}
              />
              <button
                className="btn btn-primary btn-sm"
                onClick={sendMessage}
                disabled={loading || !input.trim()}
              >
                Send
              </button>
            </div>
          </div>
        </div>
      )}
    </>
  );
}
