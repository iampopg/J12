import { AIConfig, DetectedInstance, PROVIDERS } from "./types";

interface Props {
  config: AIConfig;
  setConfig: React.Dispatch<React.SetStateAction<AIConfig>>;
  detecting: boolean;
  detectedInstances: DetectedInstance[];
  customEndpoint: string;
  setCustomEndpoint: (v: string) => void;
  useCustomEndpoint: boolean;
  setUseCustomEndpoint: (v: boolean) => void;
  setAvailableModels: (models: string[]) => void;
  onAutoDetectLocalAI: () => void;
  onFetchLocalModels: (endpoint: string) => void;
}

export function EndpointConfig({
  config,
  setConfig,
  detecting,
  detectedInstances,
  customEndpoint,
  setCustomEndpoint,
  useCustomEndpoint,
  setUseCustomEndpoint,
  setAvailableModels,
  onAutoDetectLocalAI,
  onFetchLocalModels,
}: Props) {
  const selectedProvider = PROVIDERS.find(p => p.id === config.provider);

  return (
    <>
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
                  onClick={onAutoDetectLocalAI}
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
                  onClick={() => onFetchLocalModels(useCustomEndpoint ? customEndpoint : config.endpoint)}
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
          
          {/* Non-local providers */}
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
    </>
  );
}
