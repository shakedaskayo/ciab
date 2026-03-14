import { useState, useEffect } from "react";
import { X } from "lucide-react";
import type { LlmProvider, LlmProviderKind, CreateLlmProviderRequest, UpdateLlmProviderRequest } from "@/lib/api/types";
import { useCreateLlmProvider, useUpdateLlmProvider } from "@/lib/hooks/use-llm-providers";

const KINDS: { value: LlmProviderKind; label: string }[] = [
  { value: "anthropic", label: "Anthropic" },
  { value: "openai", label: "OpenAI" },
  { value: "google", label: "Google" },
  { value: "ollama", label: "Ollama" },
  { value: "openrouter", label: "OpenRouter" },
  { value: "custom", label: "Custom" },
];

interface Props {
  provider?: LlmProvider;
  onClose: () => void;
}

export default function LlmProviderDialog({ provider, onClose }: Props) {
  const isEdit = !!provider;
  const createMutation = useCreateLlmProvider();
  const updateMutation = useUpdateLlmProvider();

  const [name, setName] = useState(provider?.name ?? "");
  const [kind, setKind] = useState<LlmProviderKind>(provider?.kind ?? "anthropic");
  const [baseUrl, setBaseUrl] = useState(provider?.base_url ?? "");
  const [apiKey, setApiKey] = useState("");
  const [defaultModel, setDefaultModel] = useState(provider?.default_model ?? "");
  const [enabled, setEnabled] = useState(provider?.enabled ?? true);

  useEffect(() => {
    if (kind === "ollama" && !baseUrl) {
      setBaseUrl("http://localhost:11434");
    }
  }, [kind]);

  const handleSubmit = () => {
    if (isEdit && provider) {
      const req: UpdateLlmProviderRequest & { id: string } = {
        id: provider.id,
        name: name.trim() || undefined,
        kind,
        enabled,
        base_url: baseUrl.trim() || null,
        default_model: defaultModel.trim() || null,
        ...(apiKey.trim() ? { api_key: apiKey.trim() } : {}),
      };
      updateMutation.mutate(req, { onSuccess: onClose });
    } else {
      const req: CreateLlmProviderRequest = {
        name: name.trim(),
        kind,
        enabled,
        ...(baseUrl.trim() ? { base_url: baseUrl.trim() } : {}),
        ...(apiKey.trim() ? { api_key: apiKey.trim() } : {}),
        ...(defaultModel.trim() ? { default_model: defaultModel.trim() } : {}),
        is_local: kind === "ollama",
      };
      createMutation.mutate(req, { onSuccess: onClose });
    }
  };

  const isPending = createMutation.isPending || updateMutation.isPending;

  return (
    <div
      className="fixed inset-0 bg-black/70 backdrop-blur-sm flex items-end sm:items-center justify-center z-50 animate-fade-in"
      onClick={onClose}
    >
      <div
        className="bg-ciab-bg-card border border-ciab-border rounded-t-xl sm:rounded-lg w-full sm:max-w-md animate-scale-in"
        onClick={(e) => e.stopPropagation()}
      >
        {/* Header */}
        <div className="flex items-center justify-between p-4 border-b border-ciab-border">
          <h2 className="text-sm font-semibold">
            {isEdit ? "Edit LLM Provider" : "Add LLM Provider"}
          </h2>
          <button
            onClick={onClose}
            className="text-ciab-text-muted hover:text-ciab-text-primary transition-colors p-0.5"
          >
            <X className="w-4 h-4" />
          </button>
        </div>

        <div className="p-4 space-y-3">
          {/* Kind */}
          <div>
            <label className="label">Provider Type</label>
            <div className="grid grid-cols-3 gap-1.5">
              {KINDS.map((k) => (
                <button
                  key={k.value}
                  onClick={() => setKind(k.value)}
                  className={`p-2 rounded-md border text-[11px] font-medium transition-all ${
                    kind === k.value
                      ? "border-ciab-copper/50 bg-ciab-copper/5"
                      : "border-ciab-border hover:border-ciab-border-light"
                  }`}
                >
                  {k.label}
                </button>
              ))}
            </div>
          </div>

          {/* Name */}
          <div>
            <label className="label">Name</label>
            <input
              type="text"
              value={name}
              onChange={(e) => setName(e.target.value)}
              placeholder={`My ${KINDS.find((k) => k.value === kind)?.label ?? kind} provider`}
              className="input w-full"
            />
          </div>

          {/* Base URL */}
          {kind !== "anthropic" && kind !== "google" && (
            <div>
              <label className="label">
                Base URL{" "}
                <span className="text-ciab-text-muted/50 normal-case tracking-normal">
                  {kind === "ollama" ? "(default: localhost:11434)" : "(optional)"}
                </span>
              </label>
              <input
                type="text"
                value={baseUrl}
                onChange={(e) => setBaseUrl(e.target.value)}
                placeholder={
                  kind === "ollama"
                    ? "http://localhost:11434"
                    : kind === "openrouter"
                    ? "https://openrouter.ai/api/v1"
                    : "https://api.example.com"
                }
                className="input w-full font-mono text-xs"
              />
            </div>
          )}

          {/* API Key */}
          {kind !== "ollama" && (
            <div>
              <label className="label">
                API Key{" "}
                {isEdit && (
                  <span className="text-ciab-text-muted/50 normal-case tracking-normal">
                    (leave blank to keep existing)
                  </span>
                )}
              </label>
              <input
                type="password"
                value={apiKey}
                onChange={(e) => setApiKey(e.target.value)}
                placeholder="sk-..."
                className="input w-full font-mono text-xs"
              />
            </div>
          )}

          {/* Default Model */}
          <div>
            <label className="label">
              Default Model{" "}
              <span className="text-ciab-text-muted/50 normal-case tracking-normal">(optional)</span>
            </label>
            <input
              type="text"
              value={defaultModel}
              onChange={(e) => setDefaultModel(e.target.value)}
              placeholder={
                kind === "anthropic"
                  ? "claude-sonnet-4-20250514"
                  : kind === "openai"
                  ? "gpt-4o"
                  : kind === "ollama"
                  ? "llama3:8b"
                  : ""
              }
              className="input w-full font-mono text-xs"
            />
          </div>

          {/* Enabled */}
          <label className="flex items-center gap-2 cursor-pointer">
            <input
              type="checkbox"
              checked={enabled}
              onChange={(e) => setEnabled(e.target.checked)}
              className="rounded border-ciab-border"
            />
            <span className="text-xs text-ciab-text-secondary">Enabled</span>
          </label>
        </div>

        {/* Footer */}
        <div className="flex items-center justify-end gap-2 p-4 border-t border-ciab-border">
          <button onClick={onClose} className="btn-secondary text-sm px-3 py-1.5">
            Cancel
          </button>
          <button
            onClick={handleSubmit}
            disabled={!name.trim() || isPending}
            className="btn-primary disabled:opacity-30 text-sm px-3 py-1.5"
          >
            {isPending ? "Saving..." : isEdit ? "Update" : "Add Provider"}
          </button>
        </div>
      </div>
    </div>
  );
}
