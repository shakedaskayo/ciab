import { useState } from "react";
import { ChevronDown, ChevronRight, Cpu, Shield, Server, Network } from "lucide-react";
import type { WorkspaceRuntimeConfig } from "@/lib/api/types";

interface Props {
  config: WorkspaceRuntimeConfig;
  onChange: (updated: WorkspaceRuntimeConfig) => void;
}

export default function KubernetesRuntimePanel({ config, onChange }: Props) {
  const [openSections, setOpenSections] = useState<Set<string>>(new Set(["basic"]));

  const toggle = (section: string) => {
    setOpenSections((prev) => {
      const next = new Set(prev);
      if (next.has(section)) next.delete(section);
      else next.add(section);
      return next;
    });
  };

  const set = (key: keyof WorkspaceRuntimeConfig, value: string | undefined) => {
    onChange({ ...config, [key]: value || undefined });
  };

  const hasRuntimeClass = !!config.kubernetes_runtime_class;

  return (
    <div className="space-y-3 animate-fade-in">
      {/* Basic Config */}
      <Section
        id="basic"
        icon={<Server className="w-3.5 h-3.5" />}
        label="Basic Config"
        open={openSections.has("basic")}
        onToggle={() => toggle("basic")}
      >
        <div className="space-y-3">
          <div>
            <label className="label">Namespace</label>
            <input
              className="input"
              placeholder="ciab-agents"
              value={config.kubernetes_namespace ?? ""}
              onChange={(e) => set("kubernetes_namespace", e.target.value)}
            />
            <p className="text-[10px] text-ciab-text-muted mt-1">
              Kubernetes namespace where agent Pods will run
            </p>
          </div>

          <div>
            <div className="flex items-center gap-2">
              <label className="label mb-0">RuntimeClass</label>
              {hasRuntimeClass && (
                <span className="inline-flex items-center gap-1 px-1.5 py-0.5 rounded text-[10px] font-medium bg-ciab-copper/10 text-ciab-copper border border-ciab-copper/20">
                  <Cpu className="w-2.5 h-2.5" />
                  microVM isolation enabled
                </span>
              )}
            </div>
            <input
              className="input mt-1"
              placeholder="kata-containers"
              value={config.kubernetes_runtime_class ?? ""}
              onChange={(e) => set("kubernetes_runtime_class", e.target.value)}
            />
            <p className="text-[10px] text-ciab-text-muted mt-1">
              Set to <code className="font-mono">kata-containers</code> or{" "}
              <code className="font-mono">kata-qemu</code> for microVM isolation (Kata Containers must
              be installed on the cluster)
            </p>
          </div>

          <div>
            <label className="label">Agent Image</label>
            <input
              className="input"
              placeholder="ghcr.io/shakedaskayo/ciab-claude:latest"
              value={config.kubernetes_image ?? ""}
              onChange={(e) => set("kubernetes_image", e.target.value)}
            />
            <p className="text-[10px] text-ciab-text-muted mt-1">
              Container image override for agent Pods
            </p>
          </div>
        </div>
      </Section>

      {/* Scheduling */}
      <Section
        id="scheduling"
        icon={<Network className="w-3.5 h-3.5" />}
        label="Scheduling"
        open={openSections.has("scheduling")}
        onToggle={() => toggle("scheduling")}
      >
        <div className="space-y-3">
          <div>
            <label className="label">Node Selector</label>
            <NodeSelectorEditor
              value={config.kubernetes_node_selector ?? {}}
              onChange={(ns) =>
                onChange({
                  ...config,
                  kubernetes_node_selector: Object.keys(ns).length > 0 ? ns : undefined,
                })
              }
            />
            <p className="text-[10px] text-ciab-text-muted mt-1">
              Schedule agent Pods only on nodes with these labels
            </p>
          </div>
        </div>
      </Section>

      {/* Security */}
      <Section
        id="security"
        icon={<Shield className="w-3.5 h-3.5" />}
        label="Security"
        open={openSections.has("security")}
        onToggle={() => toggle("security")}
      >
        <p className="text-[10px] text-ciab-text-muted">
          Security settings are configured server-side in <code className="font-mono">config.toml</code>{" "}
          under <code className="font-mono">[runtime.kubernetes]</code>. Per-workspace overrides are not
          currently supported for security settings.
        </p>
      </Section>
    </div>
  );
}

// ---------------------------------------------------------------------------
// Section
// ---------------------------------------------------------------------------

function Section({
  id: _id,
  icon,
  label,
  open,
  onToggle,
  children,
}: {
  id: string;
  icon: React.ReactNode;
  label: string;
  open: boolean;
  onToggle: () => void;
  children: React.ReactNode;
}) {
  return (
    <div className="card overflow-hidden">
      <button
        type="button"
        onClick={onToggle}
        className="w-full flex items-center gap-2 px-3 py-2.5 text-left hover:bg-ciab-surface-hover transition-colors"
      >
        <span className="text-ciab-text-muted">{icon}</span>
        <span className="text-xs font-medium text-ciab-text-primary flex-1">{label}</span>
        {open ? (
          <ChevronDown className="w-3.5 h-3.5 text-ciab-text-muted" />
        ) : (
          <ChevronRight className="w-3.5 h-3.5 text-ciab-text-muted" />
        )}
      </button>
      {open && <div className="px-3 pb-3 pt-1 border-t border-ciab-border space-y-3">{children}</div>}
    </div>
  );
}

// ---------------------------------------------------------------------------
// NodeSelectorEditor
// ---------------------------------------------------------------------------

function NodeSelectorEditor({
  value,
  onChange,
}: {
  value: Record<string, string>;
  onChange: (v: Record<string, string>) => void;
}) {
  const entries = Object.entries(value);

  const update = (idx: number, k: string, v: string) => {
    const next = { ...value };
    const oldKey = entries[idx][0];
    delete next[oldKey];
    if (k) next[k] = v;
    onChange(next);
  };

  const remove = (idx: number) => {
    const next = { ...value };
    delete next[entries[idx][0]];
    onChange(next);
  };

  const add = () => {
    onChange({ ...value, "": "" });
  };

  return (
    <div className="space-y-1.5">
      {entries.map(([k, v], idx) => (
        <div key={idx} className="flex gap-1.5">
          <input
            className="input flex-1 text-xs"
            placeholder="key"
            value={k}
            onChange={(e) => update(idx, e.target.value, v)}
          />
          <input
            className="input flex-1 text-xs"
            placeholder="value"
            value={v}
            onChange={(e) => update(idx, k, e.target.value)}
          />
          <button
            type="button"
            onClick={() => remove(idx)}
            className="px-2 py-1 text-xs text-ciab-text-muted hover:text-red-400 border border-ciab-border rounded-md"
          >
            ×
          </button>
        </div>
      ))}
      <button
        type="button"
        onClick={add}
        className="text-xs text-ciab-copper hover:text-ciab-copper/80 transition-colors"
      >
        + Add label
      </button>
    </div>
  );
}
