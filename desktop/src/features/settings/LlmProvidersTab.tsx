import { useState } from "react";
import { Plus, Search, Loader2 } from "lucide-react";
import {
  useLlmProviders,
  useDeleteLlmProvider,
  useDetectLlmProviders,
  useLlmProviderModels,
} from "@/lib/hooks/use-llm-providers";
import type { LlmProvider } from "@/lib/api/types";
import LlmProviderCard from "./LlmProviderCard";
import LlmProviderDialog from "./LlmProviderDialog";
import OllamaSection from "./OllamaSection";

export default function LlmProvidersTab() {
  const { data: providers, isLoading } = useLlmProviders();
  const deleteMutation = useDeleteLlmProvider();
  const detectMutation = useDetectLlmProviders();

  const [dialogOpen, setDialogOpen] = useState(false);
  const [editingProvider, setEditingProvider] = useState<LlmProvider | undefined>();
  const [expandedOllama, setExpandedOllama] = useState<string | null>(null);

  const handleEdit = (provider: LlmProvider) => {
    setEditingProvider(provider);
    setDialogOpen(true);
  };

  const handleAdd = () => {
    setEditingProvider(undefined);
    setDialogOpen(true);
  };

  const handleDelete = (id: string) => {
    deleteMutation.mutate(id);
  };

  const handleDetect = () => {
    detectMutation.mutate(undefined);
  };

  return (
    <div className="space-y-4">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <p className="text-xs text-ciab-text-muted">
            Manage LLM inference backends — API keys, Ollama, OpenRouter, and more.
          </p>
        </div>
        <div className="flex items-center gap-1.5">
          <button
            onClick={handleDetect}
            disabled={detectMutation.isPending}
            className="btn-ghost text-[10px] flex items-center gap-1 px-2 py-1"
          >
            {detectMutation.isPending ? (
              <Loader2 className="w-3 h-3 animate-spin" />
            ) : (
              <Search className="w-3 h-3" />
            )}
            Detect
          </button>
          <button onClick={handleAdd} className="btn-primary text-[10px] flex items-center gap-1 px-2 py-1.5">
            <Plus className="w-3 h-3" />
            Add Provider
          </button>
        </div>
      </div>

      {/* Detection results */}
      {detectMutation.data && detectMutation.data.detected.length > 0 && (
        <div className="card p-3 space-y-2">
          <span className="text-[10px] font-mono text-ciab-text-muted uppercase tracking-wider">
            Detected
          </span>
          {detectMutation.data.detected.map((d) => (
            <div
              key={d.kind}
              className="flex items-center justify-between text-xs"
            >
              <div>
                <span className="font-medium">{d.name}</span>
                {d.version && (
                  <span className="text-ciab-text-muted font-mono ml-1.5">v{d.version}</span>
                )}
              </div>
              {d.already_registered ? (
                <span className="text-[10px] text-ciab-text-muted">Already registered</span>
              ) : (
                <button
                  onClick={() => {
                    setEditingProvider(undefined);
                    setDialogOpen(true);
                  }}
                  className="btn-ghost text-[10px] px-2 py-0.5"
                >
                  Add
                </button>
              )}
            </div>
          ))}
        </div>
      )}

      {/* Provider list */}
      {isLoading && (
        <div className="flex items-center justify-center py-8">
          <Loader2 className="w-4 h-4 text-ciab-copper animate-spin" />
        </div>
      )}

      {providers && providers.length === 0 && !isLoading && (
        <div className="card p-6 text-center">
          <p className="text-sm text-ciab-text-muted">No LLM providers configured.</p>
          <p className="text-xs text-ciab-text-muted mt-1">
            Add a provider to manage API keys and models.
          </p>
        </div>
      )}

      {providers && providers.length > 0 && (
        <div className="grid gap-2">
          {providers.map((p) => (
            <div key={p.id}>
              <LlmProviderCard
                provider={p}
                onEdit={() => handleEdit(p)}
                onDelete={() => handleDelete(p.id)}
              />
              {/* Inline Ollama section */}
              {p.kind === "ollama" && (
                <div className="mt-1 ml-4">
                  <button
                    onClick={() =>
                      setExpandedOllama(expandedOllama === p.id ? null : p.id)
                    }
                    className="text-[10px] text-ciab-text-muted hover:text-ciab-copper transition-colors"
                  >
                    {expandedOllama === p.id ? "Hide models" : "Show models"}
                  </button>
                  {expandedOllama === p.id && (
                    <div className="mt-1.5 animate-fade-in">
                      <OllamaModelsWrapper providerId={p.id} baseUrl={p.base_url ?? undefined} />
                    </div>
                  )}
                </div>
              )}
            </div>
          ))}
        </div>
      )}

      {/* Dialog */}
      {dialogOpen && (
        <LlmProviderDialog
          provider={editingProvider}
          onClose={() => {
            setDialogOpen(false);
            setEditingProvider(undefined);
          }}
        />
      )}
    </div>
  );
}

function OllamaModelsWrapper({ providerId, baseUrl }: { providerId: string; baseUrl?: string }) {
  const { data: models } = useLlmProviderModels(providerId);
  return <OllamaSection providerId={providerId} models={models ?? []} baseUrl={baseUrl} />;
}
