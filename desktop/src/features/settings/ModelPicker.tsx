import { useState, useRef, useEffect } from "react";
import type { LlmModel, LlmProvider } from "@/lib/api/types";
import LlmProviderIcon from "@/components/shared/LlmProviderIcon";
import { ChevronDown, CheckCircle2, Circle, Cpu, HardDrive } from "lucide-react";

function formatSize(bytes: number | null): string {
  if (!bytes) return "";
  if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(0)} MB`;
  return `${(bytes / (1024 * 1024 * 1024)).toFixed(1)} GB`;
}

function formatCtx(tokens: number | null): string {
  if (!tokens) return "";
  if (tokens >= 1_000_000) return `${(tokens / 1_000_000).toFixed(0)}M ctx`;
  return `${(tokens / 1000).toFixed(0)}k ctx`;
}

interface Props {
  providers: LlmProvider[];
  models: Record<string, LlmModel[]>;
  value: string; // "providerId:modelId" or ""
  onChange: (modelId: string, providerId: string) => void;
  className?: string;
  /** If true, show a "Default / Inherit" option at the top */
  showDefault?: boolean;
  /** Optional: show a special "Claude subscription" entry for claude-code */
  hostAuth?: { found: boolean; expired: boolean; subscription_type: string | null; message: string } | null;
}

interface Option {
  type: "default" | "host-auth" | "model";
  label: string;
  sublabel?: string;
  providerName?: string;
  providerKind?: string;
  providerId?: string;
  modelId?: string;
  value: string;
  size?: string;
  ctx?: string;
  expired?: boolean;
  isLocal?: boolean;
}

export default function ModelPicker({ providers, models, value, onChange, className, showDefault = true, hostAuth }: Props) {
  const [open, setOpen] = useState(false);
  const ref = useRef<HTMLDivElement>(null);

  // Close on outside click
  useEffect(() => {
    const handler = (e: MouseEvent) => {
      if (ref.current && !ref.current.contains(e.target as Node)) setOpen(false);
    };
    document.addEventListener("mousedown", handler);
    return () => document.removeEventListener("mousedown", handler);
  }, []);

  // Build option list
  const options: Option[] = [];

  if (showDefault) {
    options.push({
      type: "default",
      label: "Default model",
      sublabel: "Use the model configured in the workspace spec",
      value: "",
    });
  }

  // Host Claude subscription option (for claude-code agent)
  if (hostAuth) {
    options.push({
      type: "host-auth",
      label: hostAuth.found && !hostAuth.expired
        ? `Claude subscription (${hostAuth.subscription_type ?? "plan"})`
        : hostAuth.found && hostAuth.expired
          ? "Claude subscription (expired)"
          : "Claude subscription (not found)",
      sublabel: hostAuth.found && !hostAuth.expired
        ? "Inherited from this machine — subscription pricing"
        : hostAuth.expired
          ? "Token expired — run `claude` in terminal to refresh"
          : "Log in via `claude` in a terminal to enable",
      providerKind: "anthropic",
      expired: hostAuth.expired || !hostAuth.found,
      value: "__host_auth__",
    });
  }

  // Providers with models
  for (const p of providers.filter((p) => p.enabled)) {
    const providerModels = models[p.id] ?? [];
    for (const m of providerModels) {
      options.push({
        type: "model",
        label: m.name || m.id,
        sublabel: m.id !== m.name ? m.id : undefined,
        providerName: p.name,
        providerKind: p.kind,
        providerId: p.id,
        modelId: m.id,
        value: `${p.id}:${m.id}`,
        size: formatSize(m.size_bytes),
        ctx: formatCtx(m.context_window),
        isLocal: m.is_local,
      });
    }
  }

  const selected = options.find((o) => o.value === value) ?? options[0];

  return (
    <div ref={ref} className={`relative ${className ?? ""}`}>
      {/* Trigger */}
      <button
        type="button"
        onClick={() => setOpen((o) => !o)}
        className="w-full flex items-center gap-2 px-3 py-2 rounded-lg border border-ciab-border bg-ciab-bg-secondary hover:border-ciab-border-light transition-colors text-left"
      >
        {selected && selected.type !== "default" && selected.providerKind && (
          <span className="flex-shrink-0">
            <LlmProviderIcon kind={selected.providerKind} size={14} />
          </span>
        )}
        <span className="flex-1 min-w-0">
          <span className="text-xs text-ciab-text-primary truncate block">{selected?.label ?? "Default model"}</span>
          {selected?.providerName && (
            <span className="text-[10px] text-ciab-text-muted truncate block">{selected.providerName}</span>
          )}
        </span>
        <ChevronDown className={`w-3.5 h-3.5 text-ciab-text-muted flex-shrink-0 transition-transform ${open ? "rotate-180" : ""}`} />
      </button>

      {/* Dropdown */}
      {open && (
        <div className="absolute z-50 bottom-full mb-1 w-full min-w-[280px] max-h-72 overflow-y-auto rounded-xl border border-ciab-border bg-ciab-bg-card shadow-2xl">
          {options.length === 0 && (
            <div className="px-3 py-4 text-[11px] text-ciab-text-muted text-center">No models available</div>
          )}

          {/* Group by provider */}
          {renderGrouped(options, value, (opt) => {
            onChange(opt.modelId ?? "", opt.providerId ?? "");
            setOpen(false);
          })}
        </div>
      )}
    </div>
  );
}

function renderGrouped(
  options: Option[],
  currentValue: string,
  onSelect: (opt: Option) => void
) {
  const rows: React.ReactNode[] = [];

  // Default option first
  const def = options.find((o) => o.type === "default");
  if (def) {
    rows.push(
      <OptionRow key="__default__" opt={def} selected={currentValue === ""} onSelect={onSelect} />
    );
  }

  // Host auth option
  const hostAuth = options.find((o) => o.type === "host-auth");
  if (hostAuth) {
    rows.push(
      <div key="__host-auth-group__">
        <GroupHeader label="Claude Subscription" />
        <OptionRow opt={hostAuth} selected={currentValue === hostAuth.value} onSelect={onSelect} />
      </div>
    );
  }

  // Group models by provider
  const byProvider = new Map<string, Option[]>();
  for (const opt of options.filter((o) => o.type === "model")) {
    const key = opt.providerId ?? "__unknown__";
    if (!byProvider.has(key)) byProvider.set(key, []);
    byProvider.get(key)!.push(opt);
  }

  for (const [, providerOptions] of byProvider) {
    const first = providerOptions[0];
    rows.push(
      <div key={first.providerId}>
        <GroupHeader
          label={first.providerName ?? "Unknown"}
          kind={first.providerKind}
          isLocal={first.isLocal}
        />
        {providerOptions.map((opt) => (
          <OptionRow key={opt.value} opt={opt} selected={currentValue === opt.value} onSelect={onSelect} />
        ))}
      </div>
    );
  }

  return rows;
}

function GroupHeader({ label, kind, isLocal }: { label: string; kind?: string; isLocal?: boolean }) {
  return (
    <div className="flex items-center gap-2 px-3 pt-2 pb-1 border-t border-ciab-border/40 first:border-t-0">
      {kind && <LlmProviderIcon kind={kind} size={12} />}
      <span className="text-[9px] font-mono font-semibold text-ciab-text-muted uppercase tracking-wider">{label}</span>
      {isLocal && (
        <span className="ml-auto text-[8px] font-mono bg-ciab-bg-elevated border border-ciab-border px-1 py-0.5 rounded text-ciab-text-muted/70">LOCAL</span>
      )}
    </div>
  );
}

function OptionRow({ opt, selected, onSelect }: { opt: Option; selected: boolean; onSelect: (o: Option) => void }) {
  const disabled = opt.expired;

  return (
    <button
      type="button"
      disabled={disabled}
      onClick={() => !disabled && onSelect(opt)}
      className={`w-full flex items-center gap-2.5 px-3 py-2 text-left transition-colors
        ${selected ? "bg-ciab-copper/10 text-ciab-copper" : "hover:bg-ciab-bg-hover/40 text-ciab-text-primary"}
        ${disabled ? "opacity-40 cursor-not-allowed" : "cursor-pointer"}
      `}
    >
      {/* Selection indicator */}
      <span className="flex-shrink-0">
        {selected
          ? <CheckCircle2 className="w-3.5 h-3.5 text-ciab-copper" />
          : <Circle className="w-3.5 h-3.5 text-ciab-text-muted/30" />
        }
      </span>

      {/* Provider icon for host-auth */}
      {opt.type === "host-auth" && opt.providerKind && (
        <span className="flex-shrink-0">
          <LlmProviderIcon kind={opt.providerKind} size={14} />
        </span>
      )}

      {/* Model icon for local models */}
      {opt.type === "model" && opt.isLocal && (
        <HardDrive className="w-3.5 h-3.5 text-ciab-text-muted/50 flex-shrink-0" />
      )}
      {opt.type === "model" && !opt.isLocal && (
        <Cpu className="w-3.5 h-3.5 text-ciab-text-muted/30 flex-shrink-0" />
      )}

      {/* Label */}
      <div className="flex-1 min-w-0">
        <div className={`text-xs truncate ${opt.type === "default" ? "text-ciab-text-muted italic" : ""}`}>
          {opt.label}
        </div>
        {opt.sublabel && (
          <div className="text-[10px] text-ciab-text-muted/70 truncate">{opt.sublabel}</div>
        )}
      </div>

      {/* Meta: ctx + size */}
      <div className="flex items-center gap-1.5 flex-shrink-0 text-[9px] font-mono text-ciab-text-muted/60">
        {opt.ctx && <span>{opt.ctx}</span>}
        {opt.size && <span>{opt.size}</span>}
      </div>
    </button>
  );
}
