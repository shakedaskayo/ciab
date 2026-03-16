import { useState } from "react";
import {
  ChevronDown,
  ChevronRight,
  Trash2,
  Pencil,
  Zap,
  RefreshCw,
  CheckCircle,
  XCircle,
  Loader2,
  Star,
  Eye,
  Wrench,
  HardDrive,
  Cpu,
} from "lucide-react";
import type { LlmProvider, LlmModel } from "@/lib/api/types";
import {
  useTestLlmProvider,
  useRefreshModels,
  useLlmProviderModels,
  useUpdateLlmProvider,
} from "@/lib/hooks/use-llm-providers";
import LlmProviderIcon from "@/components/shared/LlmProviderIcon";
import OllamaSection from "./OllamaSection";

function formatSize(bytes: number | null): string {
  if (!bytes) return "";
  if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(0)} MB`;
  return `${(bytes / (1024 * 1024 * 1024)).toFixed(1)} GB`;
}

function formatCtx(tokens: number | null): string {
  if (!tokens) return "";
  if (tokens >= 1_000_000) return `${(tokens / 1_000_000).toFixed(0)}M`;
  if (tokens >= 1_000) return `${(tokens / 1_000).toFixed(0)}k`;
  return `${tokens}`;
}

const KIND_LABELS: Record<string, string> = {
  anthropic: "Anthropic",
  openai: "OpenAI",
  google: "Google",
  ollama: "Ollama",
  openrouter: "OpenRouter",
  custom: "Custom",
};

interface Props {
  provider: LlmProvider;
  isDefault: boolean;
  onEdit: () => void;
  onDelete: () => void;
  onSetDefault: () => void;
}

export default function LlmProviderCard({ provider, isDefault, onEdit, onDelete, onSetDefault }: Props) {
  const [expanded, setExpanded] = useState(provider.kind === "ollama");
  const [testResult, setTestResult] = useState<{ success: boolean; message: string } | null>(null);

  const testMutation = useTestLlmProvider();
  const refreshMutation = useRefreshModels();
  const updateMutation = useUpdateLlmProvider();
  const { data: models, isLoading: loadingModels } = useLlmProviderModels(provider.id);

  const handleTest = async () => {
    setTestResult(null);
    const result = await testMutation.mutateAsync(provider.id);
    setTestResult(result);
  };

  const handleRefresh = () => refreshMutation.mutate(provider.id);

  const handleToggleEnabled = () => {
    updateMutation.mutate({ id: provider.id, enabled: !provider.enabled });
  };

  const handleSetDefaultModel = (modelId: string) => {
    updateMutation.mutate({ id: provider.id, default_model: modelId });
  };

  const defaultModel = provider.default_model;

  return (
    <div className={`rounded-xl border overflow-hidden transition-all ${
      provider.enabled
        ? isDefault
          ? "border-ciab-copper/50 bg-ciab-copper/[0.03]"
          : "border-ciab-border bg-ciab-bg-card"
        : "border-ciab-border/40 bg-ciab-bg-card/50 opacity-60"
    }`}>
      {/* Header row */}
      <div className="flex items-center gap-2 px-3 py-2.5">
        {/* Expand toggle */}
        <button
          onClick={() => setExpanded((e) => !e)}
          className="text-ciab-text-muted hover:text-ciab-text-primary transition-colors flex-shrink-0"
        >
          {expanded
            ? <ChevronDown className="w-3.5 h-3.5" />
            : <ChevronRight className="w-3.5 h-3.5" />}
        </button>

        {/* Icon */}
        <LlmProviderIcon kind={provider.kind} size={18} />

        {/* Name + kind */}
        <div className="flex-1 min-w-0">
          <div className="flex items-center gap-1.5 flex-wrap">
            <span className="text-sm font-semibold truncate">{provider.name}</span>
            <span className="text-[9px] font-mono text-ciab-text-muted bg-ciab-bg-elevated border border-ciab-border/50 px-1.5 py-0.5 rounded-full uppercase tracking-wide">
              {KIND_LABELS[provider.kind] ?? provider.kind}
            </span>
            {provider.is_local && (
              <span className="text-[9px] font-mono text-emerald-400 bg-emerald-400/10 border border-emerald-400/20 px-1.5 py-0.5 rounded-full uppercase tracking-wide">
                local
              </span>
            )}
            {provider.auto_detected && (
              <span className="text-[9px] font-mono text-ciab-steel-blue bg-ciab-steel-blue/10 border border-ciab-steel-blue/20 px-1.5 py-0.5 rounded-full uppercase tracking-wide">
                auto
              </span>
            )}
            {isDefault && (
              <span className="text-[9px] font-mono text-ciab-copper bg-ciab-copper/10 border border-ciab-copper/20 px-1.5 py-0.5 rounded-full flex items-center gap-0.5 uppercase tracking-wide">
                <Star className="w-2 h-2" /> default
              </span>
            )}
          </div>
          {defaultModel && (
            <p className="text-[10px] font-mono text-ciab-text-muted truncate mt-0.5">
              default: {defaultModel}
            </p>
          )}
        </div>

        {/* Model count */}
        {models && models.length > 0 && (
          <span className="text-[10px] font-mono text-ciab-text-muted flex-shrink-0">
            {models.length} model{models.length > 1 ? "s" : ""}
          </span>
        )}

        {/* Enable toggle */}
        <button
          onClick={handleToggleEnabled}
          disabled={updateMutation.isPending}
          title={provider.enabled ? "Disable provider" : "Enable provider"}
          className={`relative w-8 h-4.5 rounded-full flex-shrink-0 transition-colors focus:outline-none ${
            provider.enabled ? "bg-ciab-copper" : "bg-ciab-bg-elevated border border-ciab-border"
          }`}
          style={{ height: "18px" }}
        >
          <span className={`absolute top-0.5 w-3.5 h-3.5 rounded-full bg-white shadow-sm transition-transform ${
            provider.enabled ? "translate-x-4" : "translate-x-0.5"
          }`} />
        </button>

        {/* Action buttons */}
        <div className="flex items-center gap-0.5 flex-shrink-0">
          <button
            onClick={onSetDefault}
            title={isDefault ? "Already default provider" : "Set as default provider"}
            disabled={isDefault}
            className={`p-1.5 rounded transition-colors ${
              isDefault
                ? "text-ciab-copper cursor-default"
                : "text-ciab-text-muted hover:text-ciab-copper hover:bg-ciab-copper/5"
            }`}
          >
            <Star className={`w-3.5 h-3.5 ${isDefault ? "fill-ciab-copper" : ""}`} />
          </button>
          <button onClick={onEdit} className="p-1.5 rounded text-ciab-text-muted hover:text-ciab-text-primary hover:bg-ciab-bg-hover transition-colors">
            <Pencil className="w-3.5 h-3.5" />
          </button>
          <button onClick={onDelete} className="p-1.5 rounded text-ciab-text-muted hover:text-state-failed hover:bg-state-failed/5 transition-colors">
            <Trash2 className="w-3.5 h-3.5" />
          </button>
        </div>
      </div>

      {/* Expanded section */}
      {expanded && (
        <div className="border-t border-ciab-border/50 animate-fade-in">
          {/* Action bar */}
          <div className="flex items-center gap-1.5 px-3 py-2 bg-ciab-bg-primary/30 border-b border-ciab-border/30">
            <button
              onClick={handleTest}
              disabled={testMutation.isPending}
              className="btn-ghost text-[10px] flex items-center gap-1 px-2 py-1"
            >
              {testMutation.isPending ? <Loader2 className="w-3 h-3 animate-spin" /> : <Zap className="w-3 h-3" />}
              Test connection
            </button>
            <button
              onClick={handleRefresh}
              disabled={refreshMutation.isPending}
              className="btn-ghost text-[10px] flex items-center gap-1 px-2 py-1"
            >
              {refreshMutation.isPending ? <Loader2 className="w-3 h-3 animate-spin" /> : <RefreshCw className="w-3 h-3" />}
              Refresh models
            </button>
            {testResult && (
              <div className={`flex items-center gap-1 text-[10px] px-2 py-0.5 rounded ml-1 ${
                testResult.success ? "text-state-running" : "text-state-failed"
              }`}>
                {testResult.success ? <CheckCircle className="w-3 h-3" /> : <XCircle className="w-3 h-3" />}
                <span className="truncate max-w-[180px]">{testResult.message}</span>
              </div>
            )}
          </div>

          {/* Models list */}
          <div className="px-3 py-2">
            {loadingModels ? (
              <div className="flex items-center justify-center py-4">
                <Loader2 className="w-4 h-4 animate-spin text-ciab-text-muted" />
              </div>
            ) : !models || models.length === 0 ? (
              provider.kind !== "ollama" && (
                <p className="text-[11px] text-ciab-text-muted/60 text-center py-3 italic">
                  No models loaded — click Refresh models
                </p>
              )
            ) : (
              <div className="space-y-0.5">
                <div className="flex items-center px-2 py-1 mb-1">
                  <span className="text-[9px] font-mono text-ciab-text-muted/50 uppercase tracking-widest flex-1">Model</span>
                  <span className="text-[9px] font-mono text-ciab-text-muted/50 uppercase tracking-widest w-12 text-right">Ctx</span>
                  <span className="text-[9px] font-mono text-ciab-text-muted/50 uppercase tracking-widest w-16 text-right mr-10">Caps</span>
                </div>
                {models.map((model) => (
                  <ModelRow
                    key={model.id}
                    model={model}
                    isDefault={model.id === defaultModel}
                    onSetDefault={() => handleSetDefaultModel(model.id)}
                  />
                ))}
              </div>
            )}
          </div>

          {/* Ollama: pull/manage models UI */}
          {provider.kind === "ollama" && (
            <div className="border-t border-ciab-border/30 px-3 py-3">
              <OllamaSection
                providerId={provider.id}
                models={models ?? []}
                baseUrl={provider.base_url ?? undefined}
              />
            </div>
          )}
        </div>
      )}
    </div>
  );
}

function ModelRow({
  model,
  isDefault,
  onSetDefault,
}: {
  model: LlmModel;
  isDefault: boolean;
  onSetDefault: () => void;
}) {
  return (
    <div className={`group flex items-center gap-2 px-2 py-1.5 rounded-lg transition-colors ${
      isDefault
        ? "bg-ciab-copper/5 border border-ciab-copper/20"
        : "hover:bg-ciab-bg-hover/40 border border-transparent"
    }`}>
      {/* Icon */}
      <div className={`w-5 h-5 rounded flex items-center justify-center flex-shrink-0 ${
        model.is_local ? "bg-emerald-400/10" : "bg-ciab-bg-elevated"
      }`}>
        {model.is_local
          ? <HardDrive className="w-3 h-3 text-emerald-400" />
          : <Cpu className="w-3 h-3 text-ciab-text-muted/60" />}
      </div>

      {/* Model name */}
      <div className="flex-1 min-w-0">
        <div className="flex items-center gap-1.5">
          <span className="text-[11px] font-mono text-ciab-text-secondary truncate">{model.name || model.id}</span>
          {isDefault && (
            <Star className="w-2.5 h-2.5 text-ciab-copper fill-ciab-copper flex-shrink-0" />
          )}
        </div>
        {model.name && model.name !== model.id && (
          <span className="text-[9px] font-mono text-ciab-text-muted/50 truncate block">{model.id}</span>
        )}
      </div>

      {/* Context window */}
      <span className="text-[10px] font-mono text-ciab-text-muted w-12 text-right flex-shrink-0">
        {formatCtx(model.context_window)}
      </span>

      {/* Capabilities */}
      <div className="flex items-center gap-1 w-16 justify-end flex-shrink-0">
        {model.supports_tools && (
          <span title="Tool use" className="text-ciab-steel-blue">
            <Wrench className="w-3 h-3" />
          </span>
        )}
        {model.supports_vision && (
          <span title="Vision" className="text-violet-400">
            <Eye className="w-3 h-3" />
          </span>
        )}
        {model.size_bytes && (
          <span className="text-[9px] font-mono text-ciab-text-muted/60">
            {formatSize(model.size_bytes)}
          </span>
        )}
      </div>

      {/* Set default button */}
      <button
        onClick={onSetDefault}
        disabled={isDefault}
        title={isDefault ? "Default model" : "Set as default model"}
        className={`flex-shrink-0 p-1 rounded transition-all opacity-0 group-hover:opacity-100 ${
          isDefault ? "opacity-100 cursor-default text-ciab-copper" : "text-ciab-text-muted hover:text-ciab-copper hover:bg-ciab-copper/5"
        }`}
      >
        <Star className={`w-3 h-3 ${isDefault ? "fill-ciab-copper" : ""}`} />
      </button>
    </div>
  );
}
