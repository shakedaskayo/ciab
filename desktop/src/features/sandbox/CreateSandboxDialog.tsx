import { useState } from "react";
import { X, Plus, Trash2 } from "lucide-react";
import type { SandboxSpec } from "@/lib/api/types";
import AgentProviderIcon from "@/components/shared/AgentProviderIcon";

const PROVIDERS = [
  { value: "claude-code", label: "Claude Code", org: "Anthropic" },
  { value: "codex", label: "Codex", org: "OpenAI" },
  { value: "gemini", label: "Gemini CLI", org: "Google" },
  { value: "cursor", label: "Cursor", org: "Anysphere" },
];

interface Props {
  onClose: () => void;
  onCreate: (spec: SandboxSpec) => void;
}

export default function CreateSandboxDialog({ onClose, onCreate }: Props) {
  const [provider, setProvider] = useState("claude-code");
  const [name, setName] = useState("");
  const [envVars, setEnvVars] = useState<Array<{ key: string; value: string }>>(
    [{ key: "", value: "" }]
  );

  const handleCreate = () => {
    const envMap: Record<string, string> = {};
    envVars.forEach(({ key, value }) => {
      if (key.trim()) envMap[key.trim()] = value;
    });

    onCreate({
      agent_provider: provider,
      name: name.trim() || undefined,
      env_vars: Object.keys(envMap).length > 0 ? envMap : undefined,
    });
  };

  return (
    <div
      className="fixed inset-0 bg-black/70 backdrop-blur-sm flex items-end sm:items-center justify-center z-50 animate-fade-in"
      onClick={onClose}
    >
      <div
        className="bg-ciab-bg-card border border-ciab-border rounded-t-xl sm:rounded-lg w-full sm:max-w-md animate-scale-in"
        onClick={(e) => e.stopPropagation()}
      >
        {/* Header */}
        <div className="flex items-center justify-between p-4 border-b border-ciab-border">
          <h2 className="text-sm font-semibold">Create Sandbox</h2>
          <button onClick={onClose} className="text-ciab-text-muted hover:text-ciab-text-primary transition-colors p-0.5">
            <X className="w-4 h-4" />
          </button>
        </div>

        <div className="p-4 space-y-4">
          {/* Provider selection */}
          <div>
            <label className="label">Agent Provider</label>
            <div className="grid grid-cols-2 gap-2">
              {PROVIDERS.map((p) => (
                <button
                  key={p.value}
                  onClick={() => setProvider(p.value)}
                  className={`flex items-center gap-2.5 p-2.5 rounded-md border transition-all text-left ${
                    provider === p.value
                      ? "border-ciab-copper/50 bg-ciab-copper/5"
                      : "border-ciab-border hover:border-ciab-border-light"
                  }`}
                >
                  <AgentProviderIcon provider={p.value} size={18} />
                  <div>
                    <span className="text-sm font-medium block leading-tight">{p.label}</span>
                    <span className="text-[10px] text-ciab-text-muted font-mono">{p.org}</span>
                  </div>
                </button>
              ))}
            </div>
          </div>

          {/* Name */}
          <div>
            <label className="label">
              Name <span className="text-ciab-text-muted/50 normal-case tracking-normal">(optional)</span>
            </label>
            <input
              type="text"
              value={name}
              onChange={(e) => setName(e.target.value)}
              placeholder="my-project"
              className="input w-full"
            />
          </div>

          {/* Environment variables */}
          <div>
            <label className="label">Environment Variables</label>
            <div className="space-y-1.5">
              {envVars.map((env, i) => (
                <div key={i} className="flex items-center gap-1.5">
                  <input
                    type="text"
                    value={env.key}
                    onChange={(e) => {
                      const updated = [...envVars];
                      updated[i].key = e.target.value;
                      setEnvVars(updated);
                    }}
                    placeholder="KEY"
                    className="input flex-1 font-mono text-xs"
                  />
                  <input
                    type="text"
                    value={env.value}
                    onChange={(e) => {
                      const updated = [...envVars];
                      updated[i].value = e.target.value;
                      setEnvVars(updated);
                    }}
                    placeholder="value"
                    className="input flex-1 font-mono text-xs"
                  />
                  {envVars.length > 1 && (
                    <button
                      onClick={() =>
                        setEnvVars(envVars.filter((_, j) => j !== i))
                      }
                      className="p-1.5 text-ciab-text-muted hover:text-state-failed transition-colors"
                    >
                      <Trash2 className="w-3.5 h-3.5" />
                    </button>
                  )}
                </div>
              ))}
              <button
                onClick={() => setEnvVars([...envVars, { key: "", value: "" }])}
                className="btn-ghost text-xs flex items-center gap-1 px-1.5 py-1"
              >
                <Plus className="w-3 h-3" />
                Add variable
              </button>
            </div>
          </div>
        </div>

        {/* Footer */}
        <div className="flex items-center justify-end gap-2 p-4 border-t border-ciab-border">
          <button onClick={onClose} className="btn-secondary text-sm px-3 py-1.5">
            Cancel
          </button>
          <button onClick={handleCreate} className="btn-primary text-sm px-3 py-1.5">
            Create
          </button>
        </div>
      </div>
    </div>
  );
}
