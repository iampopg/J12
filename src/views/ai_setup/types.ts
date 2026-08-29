export interface AIConfig {
  enabled: boolean;
  provider: string;
  api_key: string;
  model: string;
  endpoint: string;
}

export interface AISetupProps {
  caseId: string;
  onAIEnabled: () => void;
  onAIConfigured: (config: AIConfig) => void;
}

export interface DetectedInstance {
  endpoint: string;
  status: "running" | "not_found";
  models: string[];
}

export interface KiloAIModel {
  id: string;
  name: string;
  description: string;
  contextLength?: number;
  isRecommended?: boolean;
}

export const PROVIDERS = [
  { id: "local", name: "Local AI (Ollama)", description: "Auto-detect or provide URL", needsApiKey: false, needsEndpoint: true, defaultEndpoint: "http://localhost:11434" },
  { id: "kiloai", name: "kilo.ai (Free Testing)", description: "J12's free AI - uses Kilo Gateway API", needsApiKey: true, needsEndpoint: false, defaultEndpoint: "https://api.kilo.ai/api/gateway" },
  { id: "openrouter", name: "OpenRouter", description: "Access 300+ models via OpenRouter", needsApiKey: true, needsEndpoint: false, defaultEndpoint: "https://openrouter.ai/api/v1/chat/completions" },
  { id: "gemini", name: "Google Gemini", description: "Google's AI - data leaves your device", needsApiKey: true, needsEndpoint: false },
  { id: "chatgpt", name: "OpenAI ChatGPT", description: "OpenAI's GPT-4o - data leaves your device", needsApiKey: true, needsEndpoint: false },
  { id: "claude", name: "Anthropic Claude", description: "Anthropic's Claude - data leaves your device", needsApiKey: true, needsEndpoint: false },
];

export const MODELS: Record<string, string[]> = {
  local: ["llama3.2", "llama3.1", "mistral", "qwen2.5", "phi3", "deepseek-r1"],
  kiloai: ["kilo-default", "kilo-fast"],
  gemini: ["gemini-1.5-pro", "gemini-1.5-flash", "gemini-2.0-flash"],
  chatgpt: ["gpt-4o", "gpt-4o-mini", "gpt-4-turbo"],
  claude: ["claude-3-5-sonnet-20241022", "claude-3-5-haiku-20241022", "claude-3-opus-20240229"],
};

export const OLLAMA_PORTS = [11434, 11435, 11436, 8080, 8081, 5000, 5001];
