import { AIConfig, PROVIDERS, MODELS } from "./types";

interface Props {
  config: AIConfig;
  setConfig: React.Dispatch<React.SetStateAction<AIConfig>>;
}

export function ProviderSelector({ config, setConfig }: Props) {
  return (
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
  );
}
