import { AIConfig, KiloAIModel, MODELS } from "./types";

interface Props {
  config: AIConfig;
  setConfig: React.Dispatch<React.SetStateAction<AIConfig>>;
  availableModels: string[];
  kiloAIModels: KiloAIModel[];
  detecting: boolean;
  onFetchKiloAIModels: () => void;
  onFetchOpenRouterModels: () => void;
}

export function ModelSelector({
  config,
  setConfig,
  availableModels,
  kiloAIModels,
  detecting,
  onFetchKiloAIModels,
  onFetchOpenRouterModels,
}: Props) {
  return (
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
              onClick={onFetchKiloAIModels}
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
              onClick={onFetchOpenRouterModels}
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
  );
}
