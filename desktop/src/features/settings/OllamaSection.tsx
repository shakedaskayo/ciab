import { useState } from "react";
import { Download, Loader2, HardDrive, Search, Trash2, ExternalLink, CheckCircle2 } from "lucide-react";
import type { LlmModel } from "@/lib/api/types";
import { useOllamaPull, useDeleteLlmProvider, useLlmProviders } from "@/lib/hooks/use-llm-providers";

function formatSize(bytes: number | null): string {
  if (!bytes) return "";
  if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(0)} MB`;
  return `${(bytes / (1024 * 1024 * 1024)).toFixed(1)} GB`;
}

const POPULAR_MODELS = [
  { id: "llama3.1", label: "Llama 3.1", desc: "Meta's best open model", size: "4.7 GB" },
  { id: "qwen2.5-coder:7b", label: "Qwen2.5 Coder 7B", desc: "Code-focused, fast", size: "4.7 GB" },
  { id: "deepseek-coder-v2:16b", label: "DeepSeek Coder V2", desc: "Strong coding model", size: "8.9 GB" },
  { id: "codellama:13b", label: "CodeLlama 13B", desc: "Meta code assistant", size: "7.4 GB" },
  { id: "mistral:7b", label: "Mistral 7B", desc: "Fast & capable", size: "4.1 GB" },
  { id: "phi4", label: "Phi-4", desc: "Microsoft small model", size: "9.1 GB" },
  { id: "gemma3:27b", label: "Gemma 3 27B", desc: "Google's open model", size: "17 GB" },
];

interface Props {
  providerId: string;
  models: LlmModel[];
  baseUrl?: string;
  showEmptyState?: boolean;
}

export default function OllamaSection({ models, baseUrl, showEmptyState }: Props) {
  const [pullInput, setPullInput] = useState("");
  const [modelSearch, setModelSearch] = useState("");
  const [pullingModel, setPullingModel] = useState<string | null>(null);
  const pullMutation = useOllamaPull();
  const deleteMutation = useDeleteLlmProvider();
  const { data: allProviders } = useLlmProviders();

  const installedIds = new Set(models.map((m) => m.id));

  const handlePull = (modelId?: string) => {
    const target = modelId ?? pullInput.trim();
    if (!target) return;
    setPullingModel(target);
    pullMutation.mutate(
      { model: target, base_url: baseUrl },
      {
        onSuccess: () => {
          if (!modelId) setPullInput("");
          setPullingModel(null);
        },
        onError: () => setPullingModel(null),
      }
    );
  };

  const filteredModels = models.filter(
    (m) =>
      !modelSearch ||
      m.id.toLowerCase().includes(modelSearch.toLowerCase()) ||
      (m.family ?? "").toLowerCase().includes(modelSearch.toLowerCase())
  );

  const isOllamaAvailable = allProviders
    ? allProviders.some((p) => p.kind === "ollama" && p.enabled)
    : true;

  return (
    <div className="space-y-4">
      {/* Empty state: no Ollama detected */}
      {showEmptyState && !isOllamaAvailable && (
        <div className="card p-4 border-dashed text-center space-y-2">
          <p className="text-sm font-medium text-ciab-text-secondary">Ollama not detected</p>
          <p className="text-xs text-ciab-text-muted">
            Install Ollama to run local models for free — no API key required.
          </p>
          <a
            href="https://ollama.ai"
            target="_blank"
            rel="noopener noreferrer"
            className="inline-flex items-center gap-1.5 text-xs text-ciab-copper hover:underline mt-1"
          >
            <ExternalLink className="w-3 h-3" />
            Install Ollama →
          </a>
        </div>
      )}

      {/* Pull by name */}
      <div>
        <label className="label mb-1">Pull model by name</label>
        <div className="flex items-center gap-1.5">
          <input
            type="text"
            value={pullInput}
            onChange={(e) => setPullInput(e.target.value)}
            placeholder="llama3:8b or any model tag"
            className="input flex-1 font-mono text-xs"
            onKeyDown={(e) => e.key === "Enter" && handlePull()}
          />
          <button
            onClick={() => handlePull()}
            disabled={!pullInput.trim() || pullMutation.isPending}
            className="btn-primary text-[10px] px-2.5 py-1.5 flex items-center gap-1 disabled:opacity-30"
          >
            {pullMutation.isPending && pullingModel === pullInput.trim() ? (
              <Loader2 className="w-3 h-3 animate-spin" />
            ) : (
              <Download className="w-3 h-3" />
            )}
            Pull
          </button>
        </div>
      </div>

      {/* Popular models grid */}
      <div>
        <p className="text-[9px] font-mono text-ciab-text-muted uppercase tracking-wider mb-2">
          Popular Models
        </p>
        <div className="grid grid-cols-1 gap-1">
          {POPULAR_MODELS.map((m) => {
            const installed = installedIds.has(m.id);
            const isPulling = pullingModel === m.id && pullMutation.isPending;
            return (
              <div
                key={m.id}
                className={`flex items-center justify-between px-3 py-2 rounded-md border transition-all ${
                  installed
                    ? "border-ciab-copper/30 bg-ciab-copper/5"
                    : "border-ciab-border hover:border-ciab-border-light"
                }`}
              >
                <div className="min-w-0 flex-1">
                  <div className="flex items-center gap-2">
                    <span className="text-xs font-medium truncate">{m.label}</span>
                    {installed && (
                      <CheckCircle2 className="w-3 h-3 text-ciab-copper flex-shrink-0" />
                    )}
                  </div>
                  <p className="text-[10px] text-ciab-text-muted">{m.desc} · {m.size}</p>
                </div>
                {!installed && (
                  <button
                    onClick={() => handlePull(m.id)}
                    disabled={isPulling || pullMutation.isPending}
                    className="btn-ghost text-[10px] px-2 py-1 flex items-center gap-1 flex-shrink-0 ml-2 disabled:opacity-40"
                  >
                    {isPulling ? (
                      <Loader2 className="w-3 h-3 animate-spin" />
                    ) : (
                      <Download className="w-3 h-3" />
                    )}
                    {isPulling ? "Pulling…" : "Pull"}
                  </button>
                )}
              </div>
            );
          })}
        </div>
      </div>

      {/* Installed models */}
      <div>
        <div className="flex items-center justify-between mb-2">
          <p className="text-[9px] font-mono text-ciab-text-muted uppercase tracking-wider">
            Installed ({models.length})
          </p>
          {models.length > 3 && (
            <div className="relative">
              <Search className="w-3 h-3 absolute left-2 top-1/2 -translate-y-1/2 text-ciab-text-muted pointer-events-none" />
              <input
                type="text"
                value={modelSearch}
                onChange={(e) => setModelSearch(e.target.value)}
                placeholder="Filter…"
                className="input text-[11px] pl-6 py-0.5 h-6 w-28"
              />
            </div>
          )}
        </div>

        {models.length === 0 && (
          <p className="text-[10px] text-ciab-text-muted text-center py-3">
            No models installed. Pull a model above to get started.
          </p>
        )}

        {filteredModels.length > 0 && (
          <div className="space-y-0.5">
            {filteredModels.map((model) => (
              <div
                key={model.id}
                className="group flex items-center justify-between px-2 py-1.5 rounded-md hover:bg-ciab-bg-hover/30 transition-colors"
              >
                <div className="flex items-center gap-2 min-w-0">
                  <HardDrive className="w-3 h-3 text-ciab-text-muted flex-shrink-0" />
                  <span className="text-xs font-mono truncate">{model.id}</span>
                </div>
                <div className="flex items-center gap-2 flex-shrink-0">
                  <div className="flex items-center gap-2 text-[10px] font-mono text-ciab-text-muted">
                    {model.family && <span>{model.family}</span>}
                    {model.size_bytes && <span>{formatSize(model.size_bytes)}</span>}
                  </div>
                  <button
                    onClick={() => {
                      if (confirm(`Delete model "${model.id}"?`)) {
                        deleteMutation.mutate(model.provider_id);
                      }
                    }}
                    className="opacity-0 group-hover:opacity-100 p-0.5 rounded text-ciab-text-muted hover:text-state-failed transition-all"
                    title="Delete model"
                  >
                    <Trash2 className="w-3 h-3" />
                  </button>
                </div>
              </div>
            ))}
          </div>
        )}

        {filteredModels.length === 0 && modelSearch && (
          <p className="text-[10px] text-ciab-text-muted text-center py-2">
            No models match "{modelSearch}"
          </p>
        )}
      </div>
    </div>
  );
}
