import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  AIConfig,
  AISetupProps,
  DetectedInstance,
  KiloAIModel,
  PROVIDERS,
  OLLAMA_PORTS,
} from "./ai_setup/types";
import { ProviderSelector } from "./ai_setup/ProviderSelector";
import { ModelSelector } from "./ai_setup/ModelSelector";
import { EndpointConfig } from "./ai_setup/EndpointConfig";
import { TestAndSaveCard } from "./ai_setup/TestAndSaveCard";

export function AISetupPage({ caseId, onAIEnabled, onAIConfigured }: AISetupProps) {
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
          signal: AbortSignal.timeout(2000),
        });
        
        if (response.ok) {
          const data = await response.json();
          const models = data.models?.map((m: any) => m.name) || [];
          instances.push({ endpoint, status: "running", models });
        }
      } catch {
        // Skip
      }
    }
    
    setDetectedInstances(instances);
    const runningWithModels = instances.find(i => i.status === "running" && i.models.length > 0);
    if (runningWithModels) {
      setConfig(prev => ({
        ...prev,
        endpoint: runningWithModels.endpoint,
        model: runningWithModels.models[0] || prev.model,
      }));
      setAvailableModels(runningWithModels.models);
    } else if (instances.length === 0) {
      setAvailableModels([]);
    }
    setDetecting(false);
  };

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
        const response = await fetch(`${config.endpoint}/api/tags`, { method: "GET" });
        if (response.ok) {
          setTestResult({ success: true, message: "Connected to Local AI successfully!" });
        } else {
          setTestResult({ success: false, message: `Failed to connect: ${response.status}` });
        }
      } else if (config.provider === "kiloai") {
        if (!config.api_key) {
          setTestResult({ success: false, message: "API key is required for kilo.ai" });
        } else {
          setTestResult({ success: true, message: "kilo.ai configuration saved. Will test on first use." });
        }
      } else {
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
                You are configuring a <strong>remote AI provider</strong>. Evidence data will leave your device.
              </p>
            </div>
          )}

          {/* Provider Selection */}
          <ProviderSelector config={config} setConfig={setConfig} />

          {/* Model Selection */}
          <ModelSelector
            config={config}
            setConfig={setConfig}
            availableModels={availableModels}
            kiloAIModels={kiloAIModels}
            detecting={detecting}
            onFetchKiloAIModels={fetchKiloAIModels}
            onFetchOpenRouterModels={fetchOpenRouterModels}
          />

          {/* Endpoint URL and API Key */}
          <EndpointConfig
            config={config}
            setConfig={setConfig}
            detecting={detecting}
            detectedInstances={detectedInstances}
            customEndpoint={customEndpoint}
            setCustomEndpoint={setCustomEndpoint}
            useCustomEndpoint={useCustomEndpoint}
            setUseCustomEndpoint={setUseCustomEndpoint}
            setAvailableModels={setAvailableModels}
            onAutoDetectLocalAI={autoDetectLocalAI}
            onFetchLocalModels={fetchLocalModels}
          />

          {/* Test & Save */}
          <TestAndSaveCard
            testing={testing}
            saving={saving}
            testResult={testResult}
            aiSetupComplete={aiSetupComplete}
            configEnabled={config.enabled}
            onTestConnection={testConnection}
            onSaveConfig={saveConfig}
          />
        </>
      )}
    </div>
  );
}
