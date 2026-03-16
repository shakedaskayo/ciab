import { useState, useMemo } from "react";
import { X, Play, Loader2 } from "lucide-react";
import { useLlmProviders, useLlmProviderModels, useCompatibility, useClaudeHostAuth } from "@/lib/hooks/use-llm-providers";
import type { AgentConfig, LlmModel, LlmProvider } from "@/lib/api/types";
import ModelPicker from "@/features/settings/ModelPicker";

interface Props {
  agentProvider: string;
  currentAgentConfig?: AgentConfig;
  onClose: () => void;
  onCreate: (metadata: Record<string, unknown>) => void;
  isPending: boolean;
}

export default function NewSessionDialog({ agentProvider, currentAgentConfig, onClose, onCreate, isPending }: Props) {
  const [selectedModel, setSelectedModel] = useState(currentAgentConfig?.model ?? "");
  const [selectedLlmProviderId, setSelectedLlmProviderId] = useState(
    (currentAgentConfig?.extra?.llm_provider_id as string) ?? ""
  );

  const { data: llmProviders, isLoading: loadingProviders } = useLlmProviders();
  const { data: compatibility } = useCompatibility();
  const { data: hostAuth } = useClaudeHostAuth();

  const compatibleProviders = useMemo(() => {
    if (!llmProviders) return [];
    if (!compatibility) return llmProviders;
    const kinds = new Set(
      compatibility
        .filter((c) => c.agent_provider === agentProvider && c.supports_model_override)
        .map((c) => c.llm_provider_kind)
    );
    const filtered = llmProviders.filter((p) => kinds.has(p.kind));
    return filtered.length > 0 ? filtered : llmProviders;
  }, [llmProviders, compatibility, agentProvider]);

  const handleCreate = () => {
    const metadata: Record<string, unknown> = {};
    if (selectedModel) metadata.model_override = selectedModel;
    if (selectedLlmProviderId) metadata.llm_provider_id_override = selectedLlmProviderId;
    onCreate(metadata);
  };

  return (
    <div
      className="fixed inset-0 bg-black/60 backdrop-blur-sm flex items-center justify-center z-50 animate-fade-in"
      onClick={onClose}
    >
      <div
        className="bg-ciab-bg-card border border-ciab-border rounded-xl w-full max-w-sm animate-scale-in overflow-hidden"
        onClick={(e) => e.stopPropagation()}
      >
        <div className="h-0.5 bg-gradient-to-r from-ciab-copper to-ciab-copper-light" />

        {/* Header */}
        <div className="flex items-center justify-between px-4 py-3 border-b border-ciab-border">
          <div className="flex items-center gap-2">
            <Play className="w-3.5 h-3.5 text-ciab-copper" />
            <h2 className="text-sm font-semibold">New Session</h2>
          </div>
          <button onClick={onClose} className="text-ciab-text-muted hover:text-ciab-text-primary transition-colors p-0.5">
            <X className="w-4 h-4" />
          </button>
        </div>

        {/* Body */}
        <div className="p-4 space-y-3">
          {/* Current config summary */}
          <div className="flex items-center gap-2 px-2.5 py-1.5 rounded-md bg-ciab-bg-elevated/60 border border-ciab-border/50">
            <span className="text-[10px] font-mono text-ciab-text-muted uppercase tracking-wide w-14 flex-shrink-0">Agent</span>
            <span className="text-[11px] font-mono text-ciab-text-secondary">{agentProvider}</span>
            {currentAgentConfig?.model && (
              <>
                <span className="text-ciab-text-muted/30">·</span>
                <span className="text-[11px] font-mono text-ciab-text-muted">{currentAgentConfig.model}</span>
              </>
            )}
          </div>

          {/* Model override */}
          <div>
            <label className="label mb-1 flex items-center gap-1.5">
              LLM Model
              <span className="text-ciab-text-muted/50 normal-case tracking-normal font-normal">(optional override)</span>
            </label>
            {loadingProviders ? (
              <div className="flex items-center gap-2 py-2 text-[11px] text-ciab-text-muted">
                <Loader2 className="w-3.5 h-3.5 animate-spin" />
                Loading providers…
              </div>
            ) : (
              <ModelPickerWithProviders
                compatibleProviders={compatibleProviders}
                value={selectedLlmProviderId && selectedModel ? `${selectedLlmProviderId}:${selectedModel}` : ""}
                onChange={(modelId, providerId) => {
                  setSelectedModel(modelId);
                  setSelectedLlmProviderId(providerId);
                }}
                hostAuth={agentProvider === "claude-code" ? hostAuth ?? null : null}
              />
            )}
          </div>
        </div>

        {/* Footer */}
        <div className="flex items-center justify-between px-4 py-3 border-t border-ciab-border">
          <button onClick={onClose} className="btn-secondary text-sm px-3 py-1.5">
            Cancel
          </button>
          <button
            onClick={handleCreate}
            disabled={isPending}
            className="btn-primary flex items-center gap-1.5 text-sm px-4 py-1.5 disabled:opacity-50"
          >
            {isPending ? <Loader2 className="w-3.5 h-3.5 animate-spin" /> : <Play className="w-3.5 h-3.5" />}
            {isPending ? "Creating…" : "Start Session"}
          </button>
        </div>
      </div>
    </div>
  );
}


function ModelPickerWithProviders({
  compatibleProviders,
  value,
  onChange,
  hostAuth,
}: {
  compatibleProviders: { id: string; name: string; kind: string; enabled: boolean }[];
  value: string;
  onChange: (modelId: string, providerId: string) => void;
  hostAuth?: { found: boolean; expired: boolean; subscription_type: string | null; message: string } | null;
}) {
  const modelsMap: Record<string, LlmModel[]> = {};
  for (const p of compatibleProviders) {
    const { data } = useLlmProviderModels(p.id);
    if (data) modelsMap[p.id] = data;
  }

  return (
    <ModelPicker
      providers={compatibleProviders as LlmProvider[]}
      models={modelsMap}
      value={value}
      onChange={onChange}
      className="w-full"
      hostAuth={hostAuth}
    />
  );
}
