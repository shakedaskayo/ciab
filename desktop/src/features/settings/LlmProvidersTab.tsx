import { useState } from "react";
import { Plus, Search, Loader2, ExternalLink, CheckCircle2, AlertTriangle, KeyRound, RefreshCw } from "lucide-react";
import {
  useLlmProviders,
  useDeleteLlmProvider,
  useDetectLlmProviders,
  useUpdateLlmProvider,
  useClaudeHostAuth,
} from "@/lib/hooks/use-llm-providers";
import type { LlmProvider } from "@/lib/api/types";
import LlmProviderCard from "./LlmProviderCard";
import LlmProviderDialog from "./LlmProviderDialog";

export default function LlmProvidersTab() {
  const { data: providers, isLoading } = useLlmProviders();
  const deleteMutation = useDeleteLlmProvider();
  const detectMutation = useDetectLlmProviders();
  const updateMutation = useUpdateLlmProvider();
  const { data: hostAuth, refetch: refetchHostAuth, isFetching: fetchingHostAuth } = useClaudeHostAuth();

  const [dialogOpen, setDialogOpen] = useState(false);
  const [editingProvider, setEditingProvider] = useState<LlmProvider | undefined>();

  const hasOllama = providers?.some((p) => p.kind === "ollama") ?? false;

  const defaultProviderId = providers?.find((p) => p.extra?.["is_default"] === true)?.id ?? null;

  const handleSetDefault = (provider: LlmProvider) => {
    // Clear old default, set new one
    providers?.forEach((p) => {
      if (p.extra?.["is_default"] === true && p.id !== provider.id) {
        updateMutation.mutate({ id: p.id, extra: { ...p.extra, is_default: false } });
      }
    });
    updateMutation.mutate({ id: provider.id, extra: { ...provider.extra, is_default: true } });
  };

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
      {/* Claude host auth status banner */}
      {hostAuth && (
        <div className={`flex items-start gap-2.5 rounded-lg border px-3 py-2.5 text-[11px] ${
          hostAuth.found && !hostAuth.expired
            ? (hostAuth.expires_in_secs !== null && hostAuth.expires_in_secs < 1800)
              ? "border-amber-500/30 bg-amber-500/5"
              : "border-state-running/20 bg-state-running/5"
            : "border-ciab-border/60 bg-ciab-bg-elevated/40"
        }`}>
          {hostAuth.found && !hostAuth.expired ? (
            (hostAuth.expires_in_secs !== null && hostAuth.expires_in_secs < 1800)
              ? <AlertTriangle className="w-3.5 h-3.5 text-amber-400 flex-shrink-0 mt-0.5" />
              : <CheckCircle2 className="w-3.5 h-3.5 text-state-running flex-shrink-0 mt-0.5" />
          ) : hostAuth.expired ? (
            <AlertTriangle className="w-3.5 h-3.5 text-state-failed flex-shrink-0 mt-0.5" />
          ) : (
            <KeyRound className="w-3.5 h-3.5 text-ciab-text-muted flex-shrink-0 mt-0.5" />
          )}
          <div className="flex-1 min-w-0">
            <div className="flex items-center justify-between gap-2">
              <span className={
                hostAuth.expired ? "text-state-failed" :
                hostAuth.found ? "text-ciab-text-primary" : "text-ciab-text-secondary"
              }>
                {hostAuth.expired
                  ? "Claude subscription token expired"
                  : hostAuth.found
                    ? `Claude subscription active (${hostAuth.subscription_type ?? "unknown"})`
                    : "Claude subscription: not found"}
              </span>
              <button
                onClick={() => refetchHostAuth()}
                disabled={fetchingHostAuth}
                className="p-0.5 text-ciab-text-muted hover:text-ciab-text-secondary transition-colors"
                title="Re-check"
              >
                <RefreshCw className={`w-3 h-3 ${fetchingHostAuth ? "animate-spin" : ""}`} />
              </button>
            </div>
            <p className="text-ciab-text-muted mt-0.5">{hostAuth.message}</p>
            {hostAuth.expired && (
              <p className="mt-1 text-ciab-text-muted">
                Run <code className="font-mono bg-ciab-bg-elevated px-1 rounded text-[10px]">claude</code> in a terminal and log in again, then click refresh above.
              </p>
            )}
          </div>
        </div>
      )}

      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <p className="text-xs text-ciab-text-muted">
            Manage LLM inference backends — API keys, Ollama, OpenRouter, and more.
          </p>
        </div>
        <div className="flex items-center gap-1.5">
          <button
            onClick={handleAdd}
            className="btn-primary text-[10px] flex items-center gap-1 px-2 py-1.5"
          >
            <Plus className="w-3 h-3" />
            Add Provider
          </button>
        </div>
      </div>

      {/* Empty state: no providers at all */}
      {providers && providers.length === 0 && !isLoading && (
        <div className="space-y-3">
          {/* Ollama not-detected banner */}
          <div className="card p-4 border border-ciab-border/60 rounded-lg space-y-3">
            <div className="flex items-start justify-between gap-3">
              <div>
                <p className="text-sm font-semibold text-ciab-text-primary">Ollama not detected</p>
                <p className="text-xs text-ciab-text-muted mt-0.5">
                  Run local models for free — no API key required.
                </p>
              </div>
              <a
                href="https://ollama.ai"
                target="_blank"
                rel="noopener noreferrer"
                className="btn-secondary text-[10px] flex items-center gap-1 px-2 py-1.5 flex-shrink-0"
              >
                <ExternalLink className="w-3 h-3" />
                Install Ollama
              </a>
            </div>
            <button
              onClick={handleDetect}
              disabled={detectMutation.isPending}
              className="btn-primary text-xs flex items-center gap-1.5 px-3 py-1.5 w-full justify-center"
            >
              {detectMutation.isPending ? (
                <Loader2 className="w-3.5 h-3.5 animate-spin" />
              ) : (
                <Search className="w-3.5 h-3.5" />
              )}
              {detectMutation.isPending ? "Detecting…" : "Detect Local Providers"}
            </button>
          </div>
          <div className="card p-6 text-center">
            <p className="text-sm text-ciab-text-muted">No LLM providers configured.</p>
            <p className="text-xs text-ciab-text-muted mt-1">
              Add a provider to manage API keys and models.
            </p>
          </div>
        </div>
      )}

      {/* Ollama banner when no Ollama providers but other providers exist */}
      {providers && providers.length > 0 && !hasOllama && (
        <div className="card p-3 border border-dashed border-ciab-border/60 rounded-lg flex items-center justify-between gap-3">
          <div>
            <p className="text-xs font-medium text-ciab-text-secondary">Ollama not detected</p>
            <p className="text-[10px] text-ciab-text-muted">
              Install Ollama to run local models for free.
            </p>
          </div>
          <div className="flex items-center gap-1.5 flex-shrink-0">
            <button
              onClick={handleDetect}
              disabled={detectMutation.isPending}
              className="btn-primary text-[10px] flex items-center gap-1 px-2 py-1.5"
            >
              {detectMutation.isPending ? (
                <Loader2 className="w-3 h-3 animate-spin" />
              ) : (
                <Search className="w-3 h-3" />
              )}
              Detect
            </button>
            <a
              href="https://ollama.ai"
              target="_blank"
              rel="noopener noreferrer"
              className="btn-ghost text-[10px] flex items-center gap-1 px-2 py-1.5"
            >
              <ExternalLink className="w-3 h-3" />
              Install
            </a>
          </div>
        </div>
      )}

      {/* Detection results */}
      {detectMutation.data && detectMutation.data.detected.length > 0 && (
        <div className="card p-3 space-y-2">
          <span className="text-[10px] font-mono text-ciab-text-muted uppercase tracking-wider">
            Detected
          </span>
          {detectMutation.data.detected.map((d) => (
            <div key={d.kind} className="flex items-center justify-between text-xs">
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

      {providers && providers.length > 0 && (
        <div className="grid gap-2">
          {providers.map((p) => (
            <LlmProviderCard
              key={p.id}
              provider={p}
              isDefault={p.id === defaultProviderId}
              onEdit={() => handleEdit(p)}
              onDelete={() => handleDelete(p.id)}
              onSetDefault={() => handleSetDefault(p)}
            />
          ))}
        </div>
      )}

      {/* Detect button in footer when there are providers */}
      {providers && providers.length > 0 && (
        <div className="flex justify-end">
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
            Re-detect providers
          </button>
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

