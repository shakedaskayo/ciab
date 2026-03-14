import { useState } from "react";
import { Download, Loader2, HardDrive } from "lucide-react";
import type { LlmModel } from "@/lib/api/types";
import { useOllamaPull } from "@/lib/hooks/use-llm-providers";

function formatSize(bytes: number | null): string {
  if (!bytes) return "";
  if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(0)} MB`;
  return `${(bytes / (1024 * 1024 * 1024)).toFixed(1)} GB`;
}

interface Props {
  providerId: string;
  models: LlmModel[];
  baseUrl?: string;
}

export default function OllamaSection({ models, baseUrl }: Props) {
  const [pullInput, setPullInput] = useState("");
  const pullMutation = useOllamaPull();

  const handlePull = () => {
    if (!pullInput.trim()) return;
    pullMutation.mutate(
      { model: pullInput.trim(), base_url: baseUrl },
      { onSuccess: () => setPullInput("") }
    );
  };

  return (
    <div className="space-y-2.5">
      {/* Pull Model */}
      <div className="flex items-center gap-1.5">
        <input
          type="text"
          value={pullInput}
          onChange={(e) => setPullInput(e.target.value)}
          placeholder="llama3:8b"
          className="input flex-1 font-mono text-xs"
          onKeyDown={(e) => e.key === "Enter" && handlePull()}
        />
        <button
          onClick={handlePull}
          disabled={!pullInput.trim() || pullMutation.isPending}
          className="btn-primary text-[10px] px-2.5 py-1.5 flex items-center gap-1 disabled:opacity-30"
        >
          {pullMutation.isPending ? (
            <Loader2 className="w-3 h-3 animate-spin" />
          ) : (
            <Download className="w-3 h-3" />
          )}
          Pull
        </button>
      </div>

      {/* Model list */}
      {models.length > 0 && (
        <div className="space-y-1">
          {models.map((model) => (
            <div
              key={model.id}
              className="flex items-center justify-between px-2 py-1.5 rounded-md hover:bg-ciab-bg-hover/30 transition-colors"
            >
              <div className="flex items-center gap-2 min-w-0">
                <HardDrive className="w-3 h-3 text-ciab-text-muted flex-shrink-0" />
                <span className="text-xs font-mono truncate">{model.id}</span>
              </div>
              <div className="flex items-center gap-2 text-[10px] font-mono text-ciab-text-muted flex-shrink-0">
                {model.family && <span>{model.family}</span>}
                {model.size_bytes && <span>{formatSize(model.size_bytes)}</span>}
              </div>
            </div>
          ))}
        </div>
      )}

      {models.length === 0 && (
        <p className="text-[10px] text-ciab-text-muted text-center py-2">
          No models loaded. Pull a model or click Refresh Models.
        </p>
      )}
    </div>
  );
}
