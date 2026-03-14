import { useState } from "react";
import {
  MoreVertical,
  Trash2,
  Pencil,
  Zap,
  RefreshCw,
  CheckCircle,
  XCircle,
  Loader2,
  Circle,
} from "lucide-react";
import type { LlmProvider } from "@/lib/api/types";
import { useTestLlmProvider, useRefreshModels, useLlmProviderModels } from "@/lib/hooks/use-llm-providers";

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
  onEdit: () => void;
  onDelete: () => void;
}

export default function LlmProviderCard({ provider, onEdit, onDelete }: Props) {
  const [showMenu, setShowMenu] = useState(false);
  const testMutation = useTestLlmProvider();
  const refreshMutation = useRefreshModels();
  const { data: models } = useLlmProviderModels(provider.id);
  const [testResult, setTestResult] = useState<{ success: boolean; message: string } | null>(null);

  const handleTest = async () => {
    setTestResult(null);
    const result = await testMutation.mutateAsync(provider.id);
    setTestResult(result);
  };

  const handleRefresh = () => {
    refreshMutation.mutate(provider.id);
  };

  return (
    <div className="card p-3 space-y-2.5">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-2">
          <Circle
            className={`w-2 h-2 ${
              provider.enabled
                ? "text-state-running fill-state-running"
                : "text-state-stopped fill-state-stopped"
            }`}
          />
          <span className="text-sm font-medium">{provider.name}</span>
          {provider.auto_detected && (
            <span className="text-[9px] font-mono text-ciab-text-muted bg-ciab-bg-hover px-1.5 py-0.5 rounded">
              AUTO
            </span>
          )}
        </div>
        <div className="relative">
          <button
            onClick={() => setShowMenu(!showMenu)}
            className="p-1 text-ciab-text-muted hover:text-ciab-text-primary transition-colors"
          >
            <MoreVertical className="w-3.5 h-3.5" />
          </button>
          {showMenu && (
            <>
              <div className="fixed inset-0 z-40" onClick={() => setShowMenu(false)} />
              <div className="absolute right-0 top-6 z-50 bg-ciab-bg-card border border-ciab-border rounded-md shadow-lg py-1 min-w-[120px]">
                <button
                  onClick={() => {
                    setShowMenu(false);
                    onEdit();
                  }}
                  className="flex items-center gap-2 w-full px-3 py-1.5 text-xs hover:bg-ciab-bg-hover transition-colors"
                >
                  <Pencil className="w-3 h-3" /> Edit
                </button>
                <button
                  onClick={() => {
                    setShowMenu(false);
                    onDelete();
                  }}
                  className="flex items-center gap-2 w-full px-3 py-1.5 text-xs text-state-failed hover:bg-ciab-bg-hover transition-colors"
                >
                  <Trash2 className="w-3 h-3" /> Delete
                </button>
              </div>
            </>
          )}
        </div>
      </div>

      {/* Info */}
      <div className="flex items-center gap-3 text-[10px] font-mono text-ciab-text-muted">
        <span className="bg-ciab-bg-hover px-1.5 py-0.5 rounded">
          {KIND_LABELS[provider.kind] ?? provider.kind}
        </span>
        {provider.is_local && (
          <span className="bg-ciab-bg-hover px-1.5 py-0.5 rounded">LOCAL</span>
        )}
        {provider.default_model && (
          <span className="truncate max-w-[140px]" title={provider.default_model}>
            {provider.default_model}
          </span>
        )}
        {models && (
          <span>{models.length} models</span>
        )}
      </div>

      {/* Actions */}
      <div className="flex items-center gap-1.5 pt-0.5">
        <button
          onClick={handleTest}
          disabled={testMutation.isPending}
          className="btn-ghost text-[10px] flex items-center gap-1 px-2 py-1"
        >
          {testMutation.isPending ? (
            <Loader2 className="w-3 h-3 animate-spin" />
          ) : (
            <Zap className="w-3 h-3" />
          )}
          Test
        </button>
        <button
          onClick={handleRefresh}
          disabled={refreshMutation.isPending}
          className="btn-ghost text-[10px] flex items-center gap-1 px-2 py-1"
        >
          {refreshMutation.isPending ? (
            <Loader2 className="w-3 h-3 animate-spin" />
          ) : (
            <RefreshCw className="w-3 h-3" />
          )}
          Refresh Models
        </button>
      </div>

      {/* Test result */}
      {testResult && (
        <div
          className={`flex items-center gap-1.5 text-[10px] px-2 py-1 rounded ${
            testResult.success
              ? "bg-state-running/10 text-state-running"
              : "bg-state-failed/10 text-state-failed"
          }`}
        >
          {testResult.success ? (
            <CheckCircle className="w-3 h-3" />
          ) : (
            <XCircle className="w-3 h-3" />
          )}
          <span className="truncate">{testResult.message}</span>
        </div>
      )}
    </div>
  );
}
