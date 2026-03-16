import { useState } from "react";
import {
  X,
  Play,
  Plus,
  Trash2,
  Bot,
  Zap,
  Monitor,
  FileText,
  BookOpen,
  Loader2,
  ChevronDown,
  ChevronRight,
} from "lucide-react";
import type {
  Workspace,
  WorkspaceSpec,
  WorkspaceSkill,
  RuntimeBackend,
} from "@/lib/api/types";
import { useLlmProviders, useLlmProviderModels, useCompatibility } from "@/lib/hooks/use-llm-providers";
import ModelPicker from "@/features/settings/ModelPicker";
import AgentProviderIcon from "@/components/shared/AgentProviderIcon";

const PROVIDERS = [
  { value: "claude-code", label: "Claude Code" },
  { value: "codex", label: "Codex" },
  { value: "gemini", label: "Gemini CLI" },
  { value: "cursor", label: "Cursor" },
];

interface Props {
  workspace: Workspace;
  onClose: () => void;
  onLaunch: (overrides: Partial<WorkspaceSpec>) => void;
  isPending: boolean;
}

export default function LaunchOverrideDialog({ workspace, onClose, onLaunch, isPending }: Props) {
  const spec = workspace.spec;

  // Agent overrides
  const [provider, setProvider] = useState(spec.agent?.provider ?? "claude-code");
  const [selectedModel, setSelectedModel] = useState(spec.agent?.model ?? "");
  const [selectedLlmProviderId, setSelectedLlmProviderId] = useState(
    (spec.agent?.extra?.llm_provider_id as string) ?? ""
  );
  const [systemPrompt, setSystemPrompt] = useState(spec.agent?.system_prompt ?? "");

  // Env vars pre-filled from workspace
  const [envVars, setEnvVars] = useState<Array<{ key: string; value: string }>>(
    spec.env_vars ? Object.entries(spec.env_vars).map(([key, value]) => ({ key, value })) : []
  );

  // Skills
  const [skills, setSkills] = useState<WorkspaceSkill[]>(spec.skills ?? []);

  // Runtime
  const [runtimeBackend, setRuntimeBackend] = useState<RuntimeBackend>(
    spec.runtime?.backend ?? "default"
  );

  // Section collapse state
  const [showAgent, setShowAgent] = useState(true);
  const [showEnv, setShowEnv] = useState(envVars.length > 0);
  const [showSkills, setShowSkills] = useState(false);
  const [showRuntime, setShowRuntime] = useState(false);

  const { data: llmProviders } = useLlmProviders();
  const { data: compatibility } = useCompatibility();

  const handleLaunch = () => {
    const envMap: Record<string, string> = {};
    envVars.forEach(({ key, value }) => {
      if (key.trim()) envMap[key.trim()] = value;
    });

    const agentExtra: Record<string, unknown> = {
      ...(spec.agent?.extra ?? {}),
    };
    if (selectedLlmProviderId) agentExtra.llm_provider_id = selectedLlmProviderId;

    const overrides: Partial<WorkspaceSpec> = {};

    // Only include changed fields to avoid bloating the spec
    const agentChanged =
      provider !== (spec.agent?.provider ?? "claude-code") ||
      selectedModel !== (spec.agent?.model ?? "") ||
      systemPrompt !== (spec.agent?.system_prompt ?? "") ||
      selectedLlmProviderId !== ((spec.agent?.extra?.llm_provider_id as string) ?? "");

    if (agentChanged) {
      overrides.agent = {
        ...spec.agent,
        provider,
        ...(selectedModel ? { model: selectedModel } : {}),
        ...(systemPrompt.trim() ? { system_prompt: systemPrompt.trim() } : {}),
        ...(Object.keys(agentExtra).length > 0 ? { extra: agentExtra } : {}),
      };
    }

    if (Object.keys(envMap).length > 0) {
      overrides.env_vars = { ...spec.env_vars, ...envMap };
    }

    if (skills.length !== (spec.skills?.length ?? 0)) {
      overrides.skills = skills.filter((s) => s.source.trim());
    }

    if (runtimeBackend !== (spec.runtime?.backend ?? "default")) {
      overrides.runtime = { ...spec.runtime, backend: runtimeBackend };
    }

    onLaunch(overrides);
  };

  return (
    <div
      className="fixed inset-0 bg-black/70 backdrop-blur-sm flex items-center justify-center z-50 animate-fade-in"
      onClick={onClose}
    >
      <div
        className="bg-ciab-bg-card border border-ciab-border rounded-xl w-full max-w-lg max-h-[85vh] flex flex-col animate-scale-in overflow-hidden"
        onClick={(e) => e.stopPropagation()}
      >
        {/* Accent bar */}
        <div className="h-1 bg-gradient-to-r from-ciab-copper to-ciab-copper-light" />

        {/* Header */}
        <div className="flex items-center justify-between p-4 border-b border-ciab-border flex-shrink-0">
          <div className="flex items-center gap-2">
            <Play className="w-4 h-4 text-ciab-copper" />
            <div>
              <h2 className="text-sm font-semibold">Launch Override</h2>
              <p className="text-[10px] text-ciab-text-muted">{workspace.name} · one-time overrides</p>
            </div>
          </div>
          <button onClick={onClose} className="text-ciab-text-muted hover:text-ciab-text-primary transition-colors p-0.5">
            <X className="w-4 h-4" />
          </button>
        </div>

        {/* Body */}
        <div className="overflow-y-auto flex-1 p-4 space-y-3">
          {/* Agent */}
          <OverrideSection
            title="Agent"
            icon={Bot}
            open={showAgent}
            onToggle={() => setShowAgent(!showAgent)}
          >
            <div>
              <label className="label mb-1">Provider</label>
              <div className="grid grid-cols-2 sm:grid-cols-4 gap-1.5">
                {PROVIDERS.map((p) => (
                  <button
                    key={p.value}
                    onClick={() => setProvider(p.value)}
                    className={`flex items-center gap-1.5 p-2 rounded-md border transition-all text-left ${
                      provider === p.value
                        ? "border-ciab-copper/50 bg-ciab-copper/5"
                        : "border-ciab-border hover:border-ciab-border-light"
                    }`}
                  >
                    <AgentProviderIcon provider={p.value} size={12} />
                    <span className="text-[11px] font-medium truncate">{p.label}</span>
                  </button>
                ))}
              </div>
            </div>

            {/* Model picker */}
            {llmProviders && llmProviders.length > 0 && (
              <div className="mt-2">
                <label className="label mb-1">
                  Model Override{" "}
                  <span className="text-ciab-text-muted/50 normal-case tracking-normal">(optional)</span>
                </label>
                <OverrideModelPicker
                  provider={provider}
                  llmProviders={llmProviders}
                  compatibility={compatibility ?? []}
                  value={selectedLlmProviderId && selectedModel ? `${selectedLlmProviderId}:${selectedModel}` : ""}
                  onChange={(modelId, providerId) => {
                    setSelectedModel(modelId);
                    setSelectedLlmProviderId(providerId);
                  }}
                />
              </div>
            )}

            {/* System prompt */}
            <div className="mt-2">
              <label className="label">System Prompt</label>
              <textarea
                value={systemPrompt}
                onChange={(e) => setSystemPrompt(e.target.value)}
                className="input w-full resize-none text-xs font-mono mt-1"
                rows={3}
                placeholder={spec.agent?.system_prompt ? "(inherited from workspace)" : "Optional system prompt override…"}
              />
            </div>
          </OverrideSection>

          {/* Environment variables */}
          <OverrideSection
            title="Environment Variables"
            icon={Zap}
            open={showEnv}
            onToggle={() => setShowEnv(!showEnv)}
            count={envVars.length}
          >
            <div className="space-y-1.5">
              {envVars.map((env, i) => (
                <div key={i} className="flex items-center gap-1.5">
                  <input
                    type="text"
                    value={env.key}
                    onChange={(e) => {
                      const updated = [...envVars];
                      updated[i] = { ...updated[i], key: e.target.value };
                      setEnvVars(updated);
                    }}
                    className="input flex-1 font-mono text-[11px] py-1"
                    placeholder="KEY"
                  />
                  <input
                    type="text"
                    value={env.value}
                    onChange={(e) => {
                      const updated = [...envVars];
                      updated[i] = { ...updated[i], value: e.target.value };
                      setEnvVars(updated);
                    }}
                    className="input flex-1 font-mono text-[11px] py-1"
                    placeholder="value"
                  />
                  <button
                    onClick={() => setEnvVars(envVars.filter((_, j) => j !== i))}
                    className="p-1 text-ciab-text-muted hover:text-state-failed transition-colors"
                  >
                    <Trash2 className="w-3 h-3" />
                  </button>
                </div>
              ))}
              <button
                onClick={() => setEnvVars([...envVars, { key: "", value: "" }])}
                className="text-[11px] text-ciab-text-muted hover:text-ciab-copper transition-colors flex items-center gap-1"
              >
                <Plus className="w-3 h-3" /> Add variable
              </button>
            </div>
          </OverrideSection>

          {/* Skills */}
          <OverrideSection
            title="Skills"
            icon={BookOpen}
            open={showSkills}
            onToggle={() => setShowSkills(!showSkills)}
            count={skills.length}
          >
            <div className="space-y-1.5">
              {skills.map((skill, i) => (
                <div key={i} className="flex items-center gap-1.5">
                  <input
                    type="text"
                    value={skill.name ?? ""}
                    onChange={(e) => {
                      const updated = [...skills];
                      updated[i] = { ...updated[i], name: e.target.value || undefined };
                      setSkills(updated);
                    }}
                    className="input w-24 text-[11px] py-1"
                    placeholder="name"
                  />
                  <input
                    type="text"
                    value={skill.source}
                    onChange={(e) => {
                      const updated = [...skills];
                      updated[i] = { ...updated[i], source: e.target.value };
                      setSkills(updated);
                    }}
                    className="input flex-1 font-mono text-[11px] py-1"
                    placeholder="source path or URL"
                  />
                  <button
                    onClick={() => setSkills(skills.filter((_, j) => j !== i))}
                    className="p-1 text-ciab-text-muted hover:text-state-failed transition-colors"
                  >
                    <Trash2 className="w-3 h-3" />
                  </button>
                </div>
              ))}
              <button
                onClick={() => setSkills([...skills, { source: "" }])}
                className="text-[11px] text-ciab-text-muted hover:text-ciab-copper transition-colors flex items-center gap-1"
              >
                <Plus className="w-3 h-3" /> Add skill
              </button>
            </div>
          </OverrideSection>

          {/* Runtime backend */}
          <OverrideSection
            title="Runtime Backend"
            icon={Monitor}
            open={showRuntime}
            onToggle={() => setShowRuntime(!showRuntime)}
          >
            <div className="grid grid-cols-2 sm:grid-cols-5 gap-1.5">
              {([
                { value: "default" as RuntimeBackend, label: "Default" },
                { value: "local" as RuntimeBackend, label: "Local" },
                { value: "opensandbox" as RuntimeBackend, label: "OpenSandbox" },
                { value: "docker" as RuntimeBackend, label: "Docker" },
                { value: "kubernetes" as RuntimeBackend, label: "Kubernetes" },
              ]).map((b) => (
                <button
                  key={b.value}
                  onClick={() => setRuntimeBackend(b.value)}
                  className={`flex items-center gap-1.5 p-2 rounded-md border transition-all text-left ${
                    runtimeBackend === b.value
                      ? "border-ciab-copper/50 bg-ciab-copper/5"
                      : "border-ciab-border hover:border-ciab-border-light"
                  }`}
                >
                  <Monitor className="w-3 h-3 text-ciab-text-muted" />
                  <span className="text-[11px] font-medium truncate">{b.label}</span>
                </button>
              ))}
            </div>
          </OverrideSection>

          {/* Summary note */}
          <div className="rounded-md bg-ciab-bg-primary/40 border border-ciab-border/50 px-3 py-2">
            <p className="text-[10px] text-ciab-text-muted flex items-center gap-1.5">
              <FileText className="w-3 h-3 flex-shrink-0" />
              Overrides apply to this launch only — the saved workspace spec is not changed.
            </p>
          </div>
        </div>

        {/* Footer */}
        <div className="flex items-center justify-between p-4 border-t border-ciab-border flex-shrink-0">
          <button onClick={onClose} className="btn-secondary text-sm px-3 py-1.5">
            Cancel
          </button>
          <button
            onClick={handleLaunch}
            disabled={isPending}
            className="btn-primary flex items-center gap-2 text-sm px-4 py-1.5 disabled:opacity-50"
          >
            {isPending ? (
              <Loader2 className="w-4 h-4 animate-spin" />
            ) : (
              <Play className="w-4 h-4" />
            )}
            {isPending ? "Launching…" : "Launch"}
          </button>
        </div>
      </div>
    </div>
  );
}

// ---------------------------------------------------------------------------
// OverrideSection
// ---------------------------------------------------------------------------

function OverrideSection({
  title,
  icon: Icon,
  open,
  onToggle,
  count,
  children,
}: {
  title: string;
  icon: typeof Bot;
  open: boolean;
  onToggle: () => void;
  count?: number;
  children: React.ReactNode;
}) {
  return (
    <div className="border border-ciab-border rounded-md overflow-hidden">
      <button
        onClick={onToggle}
        className="flex items-center gap-2 w-full px-3 py-2 text-left hover:bg-ciab-bg-hover/30 transition-colors"
      >
        {open ? (
          <ChevronDown className="w-3 h-3 text-ciab-text-muted" />
        ) : (
          <ChevronRight className="w-3 h-3 text-ciab-text-muted" />
        )}
        <Icon className="w-3 h-3 text-ciab-text-muted" />
        <span className="text-[11px] font-medium text-ciab-text-secondary">{title}</span>
        {count !== undefined && count > 0 && (
          <span className="text-[10px] font-mono text-ciab-copper bg-ciab-copper/10 px-1.5 py-0.5 rounded">
            {count}
          </span>
        )}
      </button>
      {open && (
        <div className="px-3 pb-3 pt-1 border-t border-ciab-border/30 animate-fade-in">
          {children}
        </div>
      )}
    </div>
  );
}

// ---------------------------------------------------------------------------
// OverrideModelPicker
// ---------------------------------------------------------------------------

function OverrideModelPicker({
  provider,
  llmProviders,
  compatibility,
  value,
  onChange,
}: {
  provider: string;
  llmProviders: { id: string; name: string; kind: string; enabled: boolean }[];
  compatibility: { agent_provider: string; llm_provider_kind: string; supports_model_override: boolean }[];
  value: string;
  onChange: (modelId: string, providerId: string) => void;
}) {
  const compatibleKinds = new Set(
    compatibility
      .filter((c) => c.agent_provider === provider && c.supports_model_override)
      .map((c) => c.llm_provider_kind)
  );
  const filteredProviders = llmProviders.filter((p) => compatibleKinds.has(p.kind));
  if (filteredProviders.length === 0) return null;

  const modelsMap: Record<string, any[]> = {};
  for (const p of filteredProviders) {
    const { data } = useLlmProviderModels(p.id);
    if (data) modelsMap[p.id] = data;
  }

  return (
    <ModelPicker
      providers={filteredProviders as any}
      models={modelsMap}
      value={value}
      onChange={onChange}
      className="w-full"
    />
  );
}
