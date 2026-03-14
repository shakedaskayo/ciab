import { useMemo } from "react";
import type { LlmModel, LlmProvider } from "@/lib/api/types";

function formatSize(bytes: number | null): string {
  if (!bytes) return "";
  if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(0)}MB`;
  return `${(bytes / (1024 * 1024 * 1024)).toFixed(1)}GB`;
}

interface Props {
  providers: LlmProvider[];
  models: Record<string, LlmModel[]>;
  value: string;
  onChange: (modelId: string, providerId: string) => void;
  className?: string;
}

export default function ModelPicker({ providers, models, value, onChange, className }: Props) {
  const groupedOptions = useMemo(() => {
    return providers
      .filter((p) => p.enabled)
      .map((p) => ({
        provider: p,
        models: models[p.id] ?? [],
      }))
      .filter((g) => g.models.length > 0);
  }, [providers, models]);

  return (
    <select
      value={value}
      onChange={(e) => {
        const [providerId, modelId] = e.target.value.split(":", 2);
        onChange(modelId, providerId);
      }}
      className={`input text-xs ${className ?? ""}`}
    >
      <option value="">Default model</option>
      {groupedOptions.map((group) => (
        <optgroup key={group.provider.id} label={group.provider.name}>
          {group.models.map((m) => (
            <option key={`${group.provider.id}:${m.id}`} value={`${group.provider.id}:${m.id}`}>
              {m.name}
              {m.context_window ? ` (${(m.context_window / 1000).toFixed(0)}k)` : ""}
              {m.size_bytes ? ` ${formatSize(m.size_bytes)}` : ""}
            </option>
          ))}
        </optgroup>
      ))}
    </select>
  );
}
