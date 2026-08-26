import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";

interface AIConfig {
  enabled: boolean;
  provider: string;
  api_key: string;
  model: string;
  endpoint: string;
}

interface Props {
  caseId: string;
  onAIEnabled: () => void;
  onAIConfigured: (config: AIConfig) => void;
}

interface DetectedInstance {
  endpoint: string;
  status: "running" | "not_found";
  models: string[];
}

const PROVIDERS = [
  { id: "local", name: "Local AI (Ollama)", description: "Auto-detect or provide URL", needsApiKey: false, needsEndpoint: true, defaultEndpoint: "http://localhost:11434" },
  { id: "kiloai", name: "kilo.ai (Free Testing)", description: "J12's free AI - uses Kilo Gateway API", needsApiKey: true, needsEndpoint: false, defaultEndpoint: "https://api.kilo.ai/api/gateway" },
  { id: "openrouter", name: "OpenRouter", description: "Access 300+ models via OpenRouter", needsApiKey: true, needsEndpoint: false, defaultEndpoint: "https://openrouter.ai/api/v1/chat/completions" },
  { id: "gemini", name: "Google Gemini", description: "Google's AI - data leaves your device", needsApiKey: true, needsEndpoint: false },
  { id: "chatgpt", name: "OpenAI ChatGPT", description: "OpenAI's GPT-4o - data leaves your device", needsApiKey: true, needsEndpoint: false },
  { id: "claude", name: "Anthropic Claude", description: "Anthropic's Claude - data leaves your device", needsApiKey: true, needsEndpoint: false },
];

const MODELS: Record<string, string[]> = {
  local: ["llama3.2", "llama3.1", "mistral", "qwen2.5", "phi3", "deepseek-r1"],
  kiloai: ["kilo-default", "kilo-fast"],
  gemini: ["gemini-1.5-pro", "gemini-1.5-flash", "gemini-2.0-flash"],
  chatgpt: ["gpt-4o", "gpt-4o-mini", "gpt-4-turbo"],
  claude: ["claude-3-5-sonnet-20241022", "claude-3-5-haiku-20241022", "claude-3-opus-20240229"],
};

// Common Ollama ports to auto-detect
const OLLAMA_PORTS = [11434, 11435, 11436, 8080, 8081, 5000, 5001];

interface KiloAIModel {
  id: string;
  name: string;
  description: string;
  contextLength?: number;
  isRecommended?: boolean;
}

export function AISetupPage({ caseId, onAIEnabled, onAIConfigured }: Props) {
  const [config, setConfig] = useState<AIConfig>({
    enabled: false,
    provider: "local",
    api_key: "",
    model: "llama3.2",
    endpoint: "http://localhost:11434",
  });
  const [testing, setTesting] = useState(false);
  const [testResult, setTestResult] = useState<{ success: boolean; message: string } | null>(null);
  const [saving, setSaving] = useState(false);
  const [showPrivacyWarning, setShowPrivacyWarning] = useState(false);
  
  // Local AI auto-detection
  const [detecting, setDetecting] = useState(false);
  const [detectedInstances, setDetectedInstances] = useState<DetectedInstance[]>([]);
  const [availableModels, setAvailableModels] = useState<string[]>([]);
  const [customEndpoint, setCustomEndpoint] = useState("");
  const [useCustomEndpoint, setUseCustomEndpoint] = useState(false);
  const [kiloAIModels, setKiloAIModels] = useState<KiloAIModel[]>([]);
  const [aiSetupComplete, setAiSetupComplete] = useState(false);

  useEffect(() => {
    loadConfig();
  }, [caseId]);

  // Auto-detect local AI when provider is local
  useEffect(() => {
    if (config.provider === "local" && config.enabled) {
      autoDetectLocalAI();
    }
  }, [config.provider, config.enabled]);

  const loadConfig = async () => {
    try {
      const saved = localStorage.getItem(`ai_config_${caseId}`);
      if (saved) {
        setConfig(JSON.parse(saved));
      }
    } catch (e) {
      console.error("Failed to load AI config:", e);
    }
  };

  // Auto-detect Ollama instances on common ports
  const autoDetectLocalAI = async () => {
    if (config.provider !== "local") return;
    
    setDetecting(true);
    setDetectedInstances([]);
    
    const instances: DetectedInstance[] = [];
    
    for (const port of OLLAMA_PORTS) {
      const endpoint = `http://localhost:${port}`;
      try {
        const response = await fetch(`${endpoint}/api/tags`, {
          method: "GET",
          signal: AbortSignal.timeout(2000), // 2 second timeout
        });
        
        if (response.ok) {
          const data = await response.json();
          const models = data.models?.map((m: any) => m.name) || [];
          instances.push({
            endpoint,
            status: "running",
            models,
          });
        }
      } catch (e) {
        // Port not available, skip
      }
    }
    
    setDetectedInstances(instances);
    
    // Auto-select first running instance with models
    const runningWithModels = instances.find(i => i.status === "running" && i.models.length > 0);
    if (runningWithModels) {
      setConfig(prev => ({
        ...prev,
        endpoint: runningWithModels.endpoint,
        model: runningWithModels.models[0] || prev.model,
      }));
      setAvailableModels(runningWithModels.models);
    } else if (instances.length === 0) {
      // No instances found
      setAvailableModels([]);
    }
    
    setDetecting(false);
  };

  // Fetch models from OpenRouter API via backend (avoids CORS)
  const fetchOpenRouterModels = async () => {
    setDetecting(true);
    setTestResult(null);
    
    try {
      const models = await invoke<any[]>("fetch_openrouter_models");
      
      setKiloAIModels(models);
      setAvailableModels(models.map((m: any) => m.id));
      if (models.length > 0) {
        setConfig(prev => ({ ...prev, model: models[0].id }));
      }
      setTestResult({ success: true, message: `Found ${models.length} free models` });
    } catch (e: any) {
      setTestResult({ success: false, message: `Failed to fetch: ${e}` });
    }
    
    setDetecting(false);
  };

  // Fetch models from kilo.ai API via backend (avoids CORS)
  const fetchKiloAIModels = async () => {
    setDetecting(true);
    setTestResult(null);
    
    try {
      const models = await invoke<any[]>("fetch_kiloai_models");
      
      setKiloAIModels(models);
      setAvailableModels(models.map((m: any) => m.id));
      if (models.length > 0) {
        setConfig(prev => ({ ...prev, model: models[0].id }));
      }
      setTestResult({ success: true, message: `Found ${models.length} free models` });
    } catch (e: any) {
      setTestResult({ success: false, message: `Failed to fetch: ${e}` });
    }
    
    setDetecting(false);
  };

  const fetchLocalModels = async (endpoint: string) => {
    setDetecting(true);
    try {
      const response = await fetch(`${endpoint}/api/tags`, {
        method: "GET",
        signal: AbortSignal.timeout(5000),
      });
      
      if (response.ok) {
        const data = await response.json();
        const models = data.models?.map((m: any) => m.name) || [];
        setAvailableModels(models);
        if (models.length > 0) {
          setConfig(prev => ({ ...prev, model: models[0] }));
        }
        setTestResult({ success: true, message: `Found ${models.length} models` });
      } else {
        setTestResult({ success: false, message: `Failed: ${response.status}` });
        setAvailableModels([]);
      }
    } catch (e: any) {
      setTestResult({ success: false, message: `Connection failed: ${e.message}` });
      setAvailableModels([]);
    }
    setDetecting(false);
  };

  const saveConfig = async () => {
    setSaving(true);
    try {
      localStorage.setItem(`ai_config_${caseId}`, JSON.stringify(config));
      setAiSetupComplete(true);
      onAIConfigured(config);
      if (config.enabled) {
        onAIEnabled();
      }
    } catch (e) {
      console.error("Failed to save AI config:", e);
    }
    setSaving(false);
  };

  const testConnection = async () => {
    setTesting(true);
    setTestResult(null);
    
    try {
      if (config.provider === "local") {
        // Test Ollama connection
        const response = await fetch(`${config.endpoint}/api/tags`, {
          method: "GET",
        });
        if (response.ok) {
          setTestResult({ success: true, message: "Connected to Local AI successfully!" });
        } else {
          setTestResult({ success: false, message: `Failed to connect: ${response.status}` });
        }
      } else if (config.provider === "kiloai") {
        // Test kilo.ai connection
        if (!config.api_key) {
          setTestResult({ success: false, message: "API key is required for kilo.ai" });
        } else {
          setTestResult({ success: true, message: "kilo.ai configuration saved. Will test on first use." });
        }
      } else {
        // For cloud providers, just validate API key presence
        if (!config.api_key) {
          setTestResult({ success: false, message: "API key is required" });
        } else {
          setTestResult({ success: true, message: `${PROVIDERS.find(p => p.id === config.provider)?.name} configuration saved` });
        }
      }
    } catch (e: any) {
      setTestResult({ success: false, message: `Connection failed: ${e.message}` });
    }
    
    setTesting(false);
  };

  const selectedProvider = PROVIDERS.find(p => p.id === config.provider);
  const isRemote = config.provider !== "local";

  return (
    <div style={{ maxWidth: 700, margin: "0 auto" }}>
      <div className="row between mb-4">
        <div>
          <h2 style={{ fontSize: 22, fontWeight: 700, color: "var(--text-0)" }}>AI Investigator Setup</h2>
          <p className="muted">Configure AI to assist with your investigation</p>
        </div>
        <label className="row gap-2" style={{ cursor: "pointer" }}>
          <input
            type="checkbox"
            checked={config.enabled}
            onChange={e => setConfig({ ...config, enabled: e.target.checked })}
          />
          <span style={{ fontSize: 14, fontWeight: 600 }}>Enable AI for this project</span>
        </label>
      </div>

      {!config.enabled && (
        <div className="card" style={{ background: "var(--accent-subtle)", borderColor: "var(--accent)", marginBottom: 24 }}>
          <div className="row gap-4">
            <span style={{ fontSize: 32 }}>🤖</span>
            <div>
              <h4 style={{ fontSize: 14, fontWeight: 600, marginBottom: 4 }}>AI is Disabled</h4>
              <p style={{ fontSize: 12, color: "var(--text-2)" }}>
                Enable AI to get investigation assistance, natural language search, evidence explanations, and more.
                AI features will only appear after you enable and configure them.
              </p>
            </div>
          </div>
        </div>
      )}

      {config.enabled && (
        <>
          {/* Privacy Warning for Remote AI */}
          {isRemote && (
            <div className="card mb-4" style={{ borderColor: "var(--red)", border: "1px solid var(--red)" }}>
              <h4 style={{ fontSize: 14, fontWeight: 600, color: "var(--red)", marginBottom: 8 }}>⚠️ Privacy Warning</h4>
              <p style={{ fontSize: 12, color: "var(--text-2)", lineHeight: 1.6 }}>
                You are configuring a <strong>remote AI provider</strong>. This means:
              </p>
              <ul style={{ fontSize: 12, color: "var(--text-2)", marginTop: 8, paddingLeft: 20, lineHeight: 1.8 }}>
                <li>Evidence data will leave your device</li>
                <li>Data will be stored on third-party servers</li>
                <li>Case confidentiality may be compromised</li>
              </ul>
              <p style={{ fontSize: 12, color: "var(--text-2)", marginTop: 8 }}>
                <strong>Recommendation:</strong> Use Local AI for sensitive investigations.
              </p>
            </div>
          )}

          {/* Provider Selection */}
          <div className="card mb-4">
            <h3 style={{ fontSize: 15, fontWeight: 600, marginBottom: 16 }}>1. Select AI Provider</h3>
            <div style={{ display: "grid", gap: 8 }}>
              {PROVIDERS.map(provider => (
                <label
                  key={provider.id}
                  className="row gap-4"
                  style={{
                    padding: 12,
                    background: config.provider === provider.id ? "rgba(59, 130, 246, 0.1)" : "var(--bg-3)",
                    borderRadius: "var(--r-sm)",
                    border: `1px solid ${config.provider === provider.id ? "#3b82f6" : "var(--border)"}`,
                    cursor: "pointer",
                  }}
                >
                  <input
                    type="radio"
                    name="provider"
                    value={provider.id}
                    checked={config.provider === provider.id}
                    onChange={() => setConfig({
                      ...config,
                      provider: provider.id,
                      endpoint: provider.defaultEndpoint || "",
                      model: MODELS[provider.id]?.[0] || "",
                    })}
                  />
                  <div style={{ flex: 1 }}>
                    <div style={{ fontSize: 13, fontWeight: 600 }}>{provider.name}</div>
                    <div style={{ fontSize: 11, color: "var(--text-3)", marginTop: 2 }}>{provider.description}</div>
                  </div>
                  {provider.id === "local" && <span className="badge badge-green">Recommended</span>}
                </label>
              ))}
            </div>
          </div>

          {/* Model Selection */}
          <div className="card mb-4">
            <h3 style={{ fontSize: 15, fontWeight: 600, marginBottom: 16 }}>2. Select Model</h3>
            
            {/* Show detected models for local AI */}
            {config.provider === "local" && availableModels.length > 0 && (
              <div style={{ marginBottom: 12 }}>
                <div style={{ fontSize: 11, fontWeight: 600, color: "var(--text-2)", marginBottom: 8 }}>
                  Detected Models ({availableModels.length})
                </div>
                <div style={{ display: "flex", flexWrap: "wrap", gap: 6 }}>
                  {availableModels.map(model => (
                    <button
                      key={model}
                      className={`btn btn-sm ${config.model === model ? "btn-primary" : "btn-ghost"}`}
                      style={{ fontSize: 11, padding: "4px 10px" }}
                      onClick={() => setConfig(prev => ({ ...prev, model }))}
                    >
                      {model}
                    </button>
                  ))}
                </div>
              </div>
            )}
            
            {/* Show free models for kilo.ai */}
            {config.provider === "kiloai" && (
              <div style={{ marginBottom: 12 }}>
                <div className="row between mb-2">
                  <div style={{ fontSize: 11, fontWeight: 600, color: "var(--text-2)" }}>
                    Free Models ({kiloAIModels.length})
                  </div>
                  <button
                    className="btn btn-ghost btn-sm"
                    onClick={fetchKiloAIModels}
                    disabled={detecting}
                    style={{ fontSize: 11, padding: "4px 12px" }}
                  >
                    {detecting ? "⏳ Loading..." : "🔄 Fetch Free Models"}
                  </button>
                </div>
                
                {kiloAIModels.length > 0 ? (
                  <div style={{ display: "grid", gap: 8, maxHeight: 300, overflowY: "auto" }}>
                    {kiloAIModels.map(model => (
                      <button
                        key={model.id}
                        className="sb-item"
                        style={{
                          width: "100%",
                          justifyContent: "flex-start",
                          padding: 10,
                          background: config.model === model.id ? "rgba(59, 130, 246, 0.1)" : "var(--bg-3)",
                          border: `1px solid ${config.model === model.id ? "#3b82f6" : "var(--border)"}`,
                        }}
                        onClick={() => setConfig(prev => ({ ...prev, model: model.id }))}
                      >
                        <div style={{ flex: 1, textAlign: "left" }}>
                          <div style={{ fontSize: 12, fontWeight: 600 }}>
                            {model.name}
                            {model.isRecommended && <span style={{ color: "var(--accent)", marginLeft: 4, fontSize: 10 }}>★ Recommended</span>}
                          </div>
                          <div style={{ fontSize: 10, color: "var(--text-3)", marginTop: 2 }}>
                            {model.description}
                          </div>
                          {model.contextLength && (
                            <div style={{ fontSize: 9, color: "var(--text-3)", marginTop: 2 }}>
                              Context: {(model.contextLength / 1000).toFixed(0)}K tokens
                            </div>
                          )}
                        </div>
                        <span className="badge badge-green" style={{ fontSize: 9 }}>FREE</span>
                      </button>
                    ))}
                  </div>
                ) : (
                  <div style={{ padding: 12, background: "var(--bg-3)", borderRadius: "var(--r-sm)" }}>
                    <p style={{ fontSize: 11, color: "var(--text-3)" }}>
                      {detecting ? "Fetching free models..." : "Click 'Fetch Free Models' to load available free models from kilo.ai"}
                    </p>
                  </div>
                )}
              </div>
            )}
            
            {/* Show free models for OpenRouter */}
            {config.provider === "openrouter" && (
              <div style={{ marginBottom: 12 }}>
                <div className="row between mb-2">
                  <div style={{ fontSize: 11, fontWeight: 600, color: "var(--text-2)" }}>
                    Free Models ({kiloAIModels.length})
                  </div>
                  <button
                    className="btn btn-ghost btn-sm"
                    onClick={fetchOpenRouterModels}
                    disabled={detecting}
                    style={{ fontSize: 11, padding: "4px 12px" }}
                  >
                    {detecting ? "⏳ Loading..." : "🔄 Fetch Free Models"}
                  </button>
                </div>
                
                {kiloAIModels.length > 0 ? (
                  <div style={{ display: "grid", gap: 8, maxHeight: 300, overflowY: "auto" }}>
                    {kiloAIModels.map(model => (
                      <button
                        key={model.id}
                        className="sb-item"
                        style={{
                          width: "100%",
                          justifyContent: "flex-start",
                          padding: 10,
                          background: config.model === model.id ? "rgba(59, 130, 246, 0.1)" : "var(--bg-3)",
                          border: `1px solid ${config.model === model.id ? "#3b82f6" : "var(--border)"}`,
                        }}
                        onClick={() => setConfig(prev => ({ ...prev, model: model.id }))}
                      >
                        <div style={{ flex: 1, textAlign: "left" }}>
                          <div style={{ fontSize: 12, fontWeight: 600 }}>{model.name}</div>
                          <div style={{ fontSize: 10, color: "var(--text-3)", marginTop: 2 }}>
                            {model.description}
                          </div>
                          {model.contextLength && (
                            <div style={{ fontSize: 9, color: "var(--text-3)", marginTop: 2 }}>
                              Context: {(model.contextLength / 1000).toFixed(0)}K tokens
                            </div>
                          )}
                        </div>
                        <span className="badge badge-green" style={{ fontSize: 9 }}>FREE</span>
                      </button>
                    ))}
                  </div>
                ) : (
                  <div style={{ padding: 12, background: "var(--bg-3)", borderRadius: "var(--r-sm)" }}>
                    <p style={{ fontSize: 11, color: "var(--text-3)" }}>
                      {detecting ? "Fetching free models..." : "Click 'Fetch Free Models' to load available free models from OpenRouter"}
                    </p>
                  </div>
                )}
              </div>
            )}
            
            {/* Default model selector for other providers */}
            {config.provider !== "local" && config.provider !== "kiloai" && (
              <select
                className="input"
                value={config.model}
                onChange={e => setConfig({ ...config, model: e.target.value })}
              >
                {(availableModels.length > 0 ? availableModels : MODELS[config.provider] || []).map(model => (
                  <option key={model} value={model}>{model}</option>
                ))}
              </select>
            )}
            
            {config.provider === "local" && availableModels.length === 0 && (
              <p style={{ fontSize: 10, color: "var(--text-3)", marginTop: 8 }}>
                No models detected. Click "Detect Models" in the Endpoint section or install models in Ollama.
              </p>
            )}
          </div>

          {/* API Key (for remote providers) */}
          {selectedProvider?.needsApiKey && (
            <div className="card mb-4">
              <h3 style={{ fontSize: 15, fontWeight: 600, marginBottom: 16 }}>3. API Key</h3>
              <input
                className="input"
                type="password"
                value={config.api_key}
                onChange={e => setConfig({ ...config, api_key: e.target.value })}
                placeholder="Enter your API key..."
              />
              <p style={{ fontSize: 11, color: "var(--text-3)", marginTop: 8 }}>
                {config.provider === "kiloai" 
                  ? "kilo.ai uses OpenRouter's API. Get your API key at openrouter.ai/keys"
                  : "Your API key is stored locally and never shared."}
              </p>
            </div>
          )}

          {/* Endpoint (for local/kiloai) */}
          {selectedProvider?.needsEndpoint && (
            <div className="card mb-4">
              <h3 style={{ fontSize: 15, fontWeight: 600, marginBottom: 16 }}>
                {selectedProvider.needsApiKey ? "4" : "3"}. Endpoint URL
              </h3>
              
              {/* Local AI Auto-Detection */}
              {config.provider === "local" && (
                <div style={{ marginBottom: 16 }}>
                  <div className="row between mb-2">
                    <span style={{ fontSize: 12, fontWeight: 600 }}>Auto-Detected Instances</span>
                    <button
                      className="btn btn-ghost btn-sm"
                      onClick={autoDetectLocalAI}
                      disabled={detecting}
                      style={{ fontSize: 11, padding: "4px 12px" }}
                    >
                      {detecting ? "🔍 Scanning..." : "🔄 Re-scan"}
                    </button>
                  </div>
                  
                  {detectedInstances.length > 0 ? (
                    <div style={{ display: "grid", gap: 8, marginBottom: 12 }}>
                      {detectedInstances.map((instance, idx) => (
                        <button
                          key={idx}
                          className="sb-item"
                          style={{
                            width: "100%",
                            justifyContent: "flex-start",
                            padding: 10,
                            background: config.endpoint === instance.endpoint ? "rgba(59, 130, 246, 0.1)" : "var(--bg-3)",
                            border: `1px solid ${config.endpoint === instance.endpoint ? "#3b82f6" : "var(--border)"}`,
                          }}
                          onClick={() => {
                            setConfig(prev => ({
                              ...prev,
                              endpoint: instance.endpoint,
                              model: instance.models[0] || prev.model,
                            }));
                            setAvailableModels(instance.models);
                            setUseCustomEndpoint(false);
                          }}
                        >
                          <div style={{ flex: 1, textAlign: "left" }}>
                            <div style={{ fontSize: 12, fontWeight: 600 }}>{instance.endpoint}</div>
                            <div style={{ fontSize: 10, color: "var(--text-3)", marginTop: 2 }}>
                              {instance.models.length} models: {instance.models.slice(0, 3).join(", ")}
                              {instance.models.length > 3 && "..."}
                            </div>
                          </div>
                          <span className="badge badge-green" style={{ fontSize: 9 }}>RUNNING</span>
                        </button>
                      ))}
                    </div>
                  ) : (
                    <div style={{ padding: 12, background: "var(--bg-3)", borderRadius: "var(--r-sm)", marginBottom: 12 }}>
                      <p style={{ fontSize: 11, color: "var(--text-3)" }}>
                        {detecting ? "Scanning for local AI instances..." : "No local AI instances detected. Provide a custom URL below or install Ollama."}
                      </p>
                    </div>
                  )}
                  
                  <div className="row gap-2" style={{ marginBottom: 8 }}>
                    <input
                      className="input"
                      value={useCustomEndpoint ? customEndpoint : config.endpoint}
                      onChange={e => {
                        setCustomEndpoint(e.target.value);
                        setUseCustomEndpoint(true);
                      }}
                      placeholder="http://localhost:11434"
                      style={{ flex: 1 }}
                    />
                    <button
                      className="btn btn-ghost btn-sm"
                      onClick={() => fetchLocalModels(useCustomEndpoint ? customEndpoint : config.endpoint)}
                      disabled={detecting}
                      style={{ fontSize: 11 }}
                    >
                      {detecting ? "..." : "Detect Models"}
                    </button>
                  </div>
                  <p style={{ fontSize: 10, color: "var(--text-3)" }}>
                    Enter custom URL or select detected instance above
                  </p>
                </div>
              )}
              
              {/* Non-local providers (kiloai, etc.) */}
              {config.provider !== "local" && (
                <input
                  className="input"
                  value={config.endpoint}
                  onChange={e => setConfig({ ...config, endpoint: e.target.value })}
                  placeholder="http://localhost:11434"
                />
              )}
            </div>
          )}

          {/* Test & Save */}
          <div className="card mb-4">
            <h3 style={{ fontSize: 15, fontWeight: 600, marginBottom: 16 }}>4. Test & Save</h3>
            <div className="row gap-2">
              <button
                className="btn btn-ghost"
                onClick={testConnection}
                disabled={testing}
              >
                {testing ? "Testing..." : "🔗 Test Connection"}
              </button>
              <button
                className="btn btn-primary"
                onClick={saveConfig}
                disabled={saving}
              >
                {saving ? "Saving..." : "💾 Save Configuration"}
              </button>
            </div>
            {testResult && (
              <div style={{
                marginTop: 12,
                padding: 12,
                borderRadius: "var(--r-sm)",
                background: testResult.success ? "rgba(34, 197, 94, 0.1)" : "rgba(239, 68, 68, 0.1)",
                border: `1px solid ${testResult.success ? "#22c55e" : "#ef4444"}`,
                fontSize: 12,
              }}>
                {testResult.success ? "✅" : "❌"} {testResult.message}
              </div>
            )}
          </div>

          {/* AI Data Access (shown after save) */}
          {aiSetupComplete && config.enabled && (
            <div className="card mb-4">
              <h3 style={{ fontSize: 15, fontWeight: 600, marginBottom: 16 }}>5. AI Data Access</h3>
              <p className="muted mb-4" style={{ fontSize: 12 }}>
                Configure what case data the AI can access. AI can only read data, never modify it.
              </p>
              <div style={{ display: "grid", gap: 8 }}>
                {[
                  { key: "emails", label: "Email metadata (from, to, subject, date)", default: true },
                  { key: "headers", label: "Email headers (Received, Authentication-Results)", default: true },
                  { key: "body", label: "Email body text", default: true },
                  { key: "attachments", label: "Attachment metadata (filename, hash, type)", default: true },
                  { key: "findings", label: "Forensic findings", default: true },
                  { key: "entities", label: "Entity profiles", default: true },
                  { key: "timeline", label: "Timeline events", default: true },
                  { key: "graph", label: "Communication graph", default: false },
                  { key: "notes", label: "Case notes", default: false },
                  { key: "custody", label: "Chain of custody", default: false },
                ].map(item => (
                  <label key={item.key} className="row gap-2" style={{ padding: 8, background: "var(--bg-3)", borderRadius: "var(--r-sm)", cursor: "pointer" }}>
                    <input type="checkbox" defaultChecked={item.default} />
                    <span style={{ fontSize: 12 }}>{item.label}</span>
                  </label>
                ))}
              </div>
            </div>
          )}
        </>
      )}
    </div>
  );
}
