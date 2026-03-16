import { useState, useEffect, useMemo } from "react";
import {
  X,
  Play,
  Plus,
  ChevronRight,
  Loader2,
  Layers,
  Bot,
  BookOpen,
  Cpu,
  Zap,
  CheckCircle2,
  ArrowLeft,
  Sparkles,
} from "lucide-react";
import { useWorkspaces, useCreateWorkspace, useLaunchWorkspace } from "@/lib/hooks/use-workspaces";
import { useLlmProviders, useLlmProviderModels, useCompatibility, useClaudeHostAuth } from "@/lib/hooks/use-llm-providers";
import type { Workspace, WorkspaceSpec } from "@/lib/api/types";
import AgentProviderIcon from "@/components/shared/AgentProviderIcon";
import ModelPicker from "@/features/settings/ModelPicker";

interface Props {
  onClose: () => void;
  onSuccess?: (sandboxId: string) => void;
}

type Step = "workspace" | "configure";

export default function CreateSandboxDialog({ onClose, onSuccess }: Props) {
  const { data: workspaceList, isLoading: loadingWorkspaces } = useWorkspaces();
  const createWorkspace = useCreateWorkspace();
  const launchWorkspace = useLaunchWorkspace();
  const [step, setStep] = useState<Step>("workspace");
  const [selectedWorkspaceId, setSelectedWorkspaceId] = useState<string | null>(null);
  const [creatingDefault, setCreatingDefault] = useState(false);

  // Auto-select first workspace
  useEffect(() => {
    if (workspaceList && workspaceList.length > 0 && !selectedWorkspaceId) {
      setSelectedWorkspaceId(workspaceList[0].id);
    }
  }, [workspaceList, selectedWorkspaceId]);

  const selectedWorkspace = workspaceList?.find((w) => w.id === selectedWorkspaceId) ?? null;

  const handleCreateDefault = () => {
    setCreatingDefault(true);
    createWorkspace.mutate(
      {
        name: "Default",
        description: "Default workspace",
        spec: {
          agent: { provider: "claude-code" },
          runtime: { backend: "local" },
        },
      },
      {
        onSuccess: (ws) => {
          setCreatingDefault(false);
          setSelectedWorkspaceId(ws.id);
          setStep("configure");
        },
        onError: () => setCreatingDefault(false),
      }
    );
  };

  const handleNext = () => {
    if (selectedWorkspace) setStep("configure");
  };

  const handleLaunch = (overrides: Partial<WorkspaceSpec>) => {
    if (!selectedWorkspaceId) return;
    launchWorkspace.mutate(
      { id: selectedWorkspaceId, spec_overrides: Object.keys(overrides).length > 0 ? overrides : undefined },
      {
        onSuccess: (result) => {
          onSuccess?.(result.sandbox_id);
          onClose();
        },
      }
    );
  };

  return (
    <div
      className="fixed inset-0 bg-black/70 backdrop-blur-sm flex items-center justify-center z-50 animate-fade-in"
      onClick={onClose}
    >
      <div
        className="bg-ciab-bg-card border border-ciab-border rounded-xl w-full max-w-lg max-h-[88vh] flex flex-col animate-scale-in overflow-hidden"
        onClick={(e) => e.stopPropagation()}
      >
        {/* Accent bar */}
        <div className="h-0.5 bg-gradient-to-r from-ciab-copper via-ciab-copper-light to-ciab-steel-blue" />

        {/* Header */}
        <div className="flex items-center justify-between px-4 py-3 border-b border-ciab-border flex-shrink-0">
          <div className="flex items-center gap-2.5">
            {step === "configure" && (
              <button
                onClick={() => setStep("workspace")}
                className="p-1 rounded text-ciab-text-muted hover:text-ciab-text-primary transition-colors"
              >
                <ArrowLeft className="w-3.5 h-3.5" />
              </button>
            )}
            <div className="flex items-center gap-1.5">
              <Play className="w-3.5 h-3.5 text-ciab-copper" />
              <h2 className="text-sm font-semibold">New Sandbox</h2>
            </div>
            {/* Step indicator */}
            <div className="flex items-center gap-1 ml-1">
              <StepDot active={step === "workspace"} done={step === "configure"} label="1" />
              <div className="w-4 h-px bg-ciab-border" />
              <StepDot active={step === "configure"} done={false} label="2" />
            </div>
          </div>
          <button onClick={onClose} className="text-ciab-text-muted hover:text-ciab-text-primary transition-colors p-0.5">
            <X className="w-4 h-4" />
          </button>
        </div>

        {/* Body */}
        <div className="flex-1 overflow-y-auto">
          {step === "workspace" ? (
            <WorkspaceStep
              workspaces={workspaceList ?? []}
              isLoading={loadingWorkspaces}
              selectedId={selectedWorkspaceId}
              onSelect={setSelectedWorkspaceId}
              onCreateDefault={handleCreateDefault}
              creatingDefault={creatingDefault}
            />
          ) : selectedWorkspace ? (
            <ConfigureStep
              workspace={selectedWorkspace}
              onLaunch={handleLaunch}
              isLaunching={launchWorkspace.isPending}
            />
          ) : null}
        </div>

        {/* Footer */}
        {step === "workspace" && (
          <div className="flex items-center justify-between px-4 py-3 border-t border-ciab-border flex-shrink-0">
            <button onClick={onClose} className="btn-secondary text-sm px-3 py-1.5">
              Cancel
            </button>
            <button
              onClick={handleNext}
              disabled={!selectedWorkspaceId}
              className="btn-primary flex items-center gap-1.5 text-sm px-4 py-1.5 disabled:opacity-40"
            >
              Configure & Launch
              <ChevronRight className="w-3.5 h-3.5" />
            </button>
          </div>
        )}
      </div>
    </div>
  );
}

/* ─── Step dot ─── */

function StepDot({ active, done, label }: { active: boolean; done: boolean; label: string }) {
  return (
    <div
      className={`w-5 h-5 rounded-full flex items-center justify-center text-[10px] font-mono font-bold transition-colors ${
        done
          ? "bg-ciab-copper/20 text-ciab-copper"
          : active
          ? "bg-ciab-copper text-white"
          : "bg-ciab-bg-elevated text-ciab-text-muted border border-ciab-border"
      }`}
    >
      {done ? <CheckCircle2 className="w-3 h-3" /> : label}
    </div>
  );
}

/* ─── Step 1: Workspace picker ─── */

function WorkspaceStep({
  workspaces,
  isLoading,
  selectedId,
  onSelect,
  onCreateDefault,
  creatingDefault,
}: {
  workspaces: Workspace[];
  isLoading: boolean;
  selectedId: string | null;
  onSelect: (id: string) => void;
  onCreateDefault: () => void;
  creatingDefault: boolean;
}) {
  if (isLoading) {
    return (
      <div className="flex items-center justify-center py-12">
        <Loader2 className="w-5 h-5 animate-spin text-ciab-text-muted" />
      </div>
    );
  }

  if (workspaces.length === 0) {
    return (
      <div className="p-6 flex flex-col items-center gap-4 text-center">
        <div className="w-14 h-14 rounded-2xl bg-ciab-copper/10 border border-ciab-copper/20 flex items-center justify-center">
          <Layers className="w-7 h-7 text-ciab-copper" />
        </div>
        <div>
          <p className="text-sm font-semibold text-ciab-text-primary">No workspaces yet</p>
          <p className="text-xs text-ciab-text-muted mt-1 max-w-xs">
            Workspaces bundle your repos, skills, agent config, and runtime settings. Create one to get started.
          </p>
        </div>
        <button
          onClick={onCreateDefault}
          disabled={creatingDefault}
          className="btn-primary flex items-center gap-2 px-4 py-2"
        >
          {creatingDefault ? (
            <Loader2 className="w-4 h-4 animate-spin" />
          ) : (
            <Sparkles className="w-4 h-4" />
          )}
          {creatingDefault ? "Creating…" : "Create Default Workspace"}
        </button>
        <p className="text-[10px] text-ciab-text-muted/60">
          You can customise it later in the Workspaces section.
        </p>
      </div>
    );
  }

  return (
    <div className="p-4 space-y-3">
      <div>
        <p className="text-xs font-medium text-ciab-text-secondary mb-0.5">Choose a workspace</p>
        <p className="text-[11px] text-ciab-text-muted">
          Settings, model, skills, and repos will be inherited from the workspace.
        </p>
      </div>

      <div className="space-y-2">
        {workspaces.map((ws) => (
          <WorkspaceCard
            key={ws.id}
            workspace={ws}
            selected={ws.id === selectedId}
            onSelect={() => onSelect(ws.id)}
          />
        ))}
      </div>

      {/* Create new workspace link */}
      <button
        onClick={onCreateDefault}
        disabled={creatingDefault}
        className="w-full flex items-center gap-2 px-3 py-2.5 rounded-lg border border-dashed border-ciab-border/60
          text-ciab-text-muted hover:text-ciab-copper hover:border-ciab-copper/40 hover:bg-ciab-copper/3
          transition-all text-[11px] font-medium"
      >
        {creatingDefault ? (
          <Loader2 className="w-3.5 h-3.5 animate-spin flex-shrink-0" />
        ) : (
          <Plus className="w-3.5 h-3.5 flex-shrink-0" />
        )}
        {creatingDefault ? "Creating workspace…" : "Create new default workspace"}
      </button>
    </div>
  );
}

function WorkspaceCard({
  workspace,
  selected,
  onSelect,
}: {
  workspace: Workspace;
  selected: boolean;
  onSelect: () => void;
}) {
  const spec = workspace.spec;
  const provider = spec.agent?.provider ?? "claude-code";
  const model = spec.agent?.model;
  const skillCount = spec.skills?.filter((s) => s.enabled !== false).length ?? 0;
  const repoCount = spec.repositories?.length ?? 0;
  const runtime = spec.runtime?.backend ?? "local";

  return (
    <button
      onClick={onSelect}
      className={`w-full text-left rounded-lg border px-3 py-2.5 transition-all ${
        selected
          ? "border-ciab-copper/60 bg-ciab-copper/5 ring-1 ring-ciab-copper/20"
          : "border-ciab-border hover:border-ciab-border-light hover:bg-ciab-bg-hover/40"
      }`}
    >
      <div className="flex items-start gap-2.5">
        {/* Selection indicator */}
        <div className={`mt-0.5 w-3.5 h-3.5 rounded-full border-2 flex items-center justify-center flex-shrink-0 transition-colors ${
          selected ? "border-ciab-copper bg-ciab-copper" : "border-ciab-border"
        }`}>
          {selected && <div className="w-1.5 h-1.5 rounded-full bg-white" />}
        </div>

        <div className="flex-1 min-w-0">
          <div className="flex items-center gap-2">
            <span className="text-sm font-medium truncate">{workspace.name}</span>
            {selected && (
              <span className="text-[9px] font-mono text-ciab-copper bg-ciab-copper/10 px-1.5 py-0.5 rounded-full flex-shrink-0">
                selected
              </span>
            )}
          </div>
          {workspace.description && (
            <p className="text-[11px] text-ciab-text-muted truncate mt-0.5">{workspace.description}</p>
          )}

          {/* Metadata pills */}
          <div className="flex items-center gap-1.5 mt-1.5 flex-wrap">
            <MetaPill icon={<AgentProviderIcon provider={provider} size={10} />} label={provider} />
            {model && <MetaPill icon={<Cpu className="w-2.5 h-2.5" />} label={model} />}
            {skillCount > 0 && <MetaPill icon={<BookOpen className="w-2.5 h-2.5" />} label={`${skillCount} skill${skillCount > 1 ? "s" : ""}`} />}
            {repoCount > 0 && <MetaPill icon={<Bot className="w-2.5 h-2.5" />} label={`${repoCount} repo${repoCount > 1 ? "s" : ""}`} />}
            <MetaPill icon={<Zap className="w-2.5 h-2.5" />} label={runtime} muted />
          </div>
        </div>
      </div>
    </button>
  );
}

function MetaPill({ icon, label, muted }: { icon: React.ReactNode; label: string; muted?: boolean }) {
  return (
    <span className={`inline-flex items-center gap-1 px-1.5 py-0.5 rounded text-[10px] font-mono border
      ${muted
        ? "border-ciab-border/50 text-ciab-text-muted/60 bg-transparent"
        : "border-ciab-border bg-ciab-bg-elevated text-ciab-text-muted"
      }`}>
      {icon}
      {label}
    </span>
  );
}

/* ─── Step 2: Configure & Launch ─── */

function ConfigureStep({
  workspace,
  onLaunch,
  isLaunching,
}: {
  workspace: Workspace;
  onLaunch: (overrides: Partial<WorkspaceSpec>) => void;
  isLaunching: boolean;
}) {
  const spec = workspace.spec;

  // Overrideable: model/LLM provider only (main ask)
  const [selectedModel, setSelectedModel] = useState(spec.agent?.model ?? "");
  const [selectedLlmProviderId, setSelectedLlmProviderId] = useState(
    (spec.agent?.extra?.llm_provider_id as string) ?? ""
  );

  const { data: llmProviders, isLoading: loadingProviders } = useLlmProviders();
  const { data: compatibility } = useCompatibility();
  const { data: hostAuth } = useClaudeHostAuth();

  const agentProvider = spec.agent?.provider ?? "claude-code";

  // Providers compatible with model override for this agent
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

  const handleLaunch = () => {
    const overrides: Partial<WorkspaceSpec> = {};

    const modelChanged = selectedModel !== (spec.agent?.model ?? "");
    const providerChanged = selectedLlmProviderId !== ((spec.agent?.extra?.llm_provider_id as string) ?? "");

    if (modelChanged || providerChanged) {
      const extra: Record<string, unknown> = { ...(spec.agent?.extra ?? {}) };
      if (selectedLlmProviderId) extra.llm_provider_id = selectedLlmProviderId;
      else delete extra.llm_provider_id;

      overrides.agent = {
        ...spec.agent,
        provider: agentProvider,
        ...(selectedModel ? { model: selectedModel } : {}),
        ...(Object.keys(extra).length > 0 ? { extra } : {}),
      };
    }

    onLaunch(overrides);
  };

  const skills = spec.skills?.filter((s) => s.enabled !== false) ?? [];
  const repos = spec.repositories ?? [];
  const envCount = Object.keys(spec.env_vars ?? {}).length;

  return (
    <div className="p-4 space-y-4">
      {/* Workspace summary — inherited settings read-only */}
      <div>
        <p className="text-[10px] font-mono font-semibold text-ciab-text-muted uppercase tracking-wide mb-2">
          Inherited from <span className="text-ciab-copper">{workspace.name}</span>
        </p>
        <div className="rounded-lg border border-ciab-border bg-ciab-bg-primary/30 divide-y divide-ciab-border/50">
          {/* Agent provider */}
          <InheritedRow
            label="Agent"
            value={
              <span className="flex items-center gap-1.5">
                <AgentProviderIcon provider={agentProvider} size={12} />
                <span>{agentProvider}</span>
              </span>
            }
          />
          {/* Current model */}
          <InheritedRow
            label="Model"
            value={
              <span className="text-ciab-text-secondary">
                {spec.agent?.model ?? <span className="text-ciab-text-muted/60 italic">default</span>}
              </span>
            }
          />
          {/* Runtime */}
          <InheritedRow
            label="Runtime"
            value={spec.runtime?.backend ?? "local"}
          />
          {/* Skills */}
          {skills.length > 0 && (
            <InheritedRow
              label="Skills"
              value={
                <span className="flex items-center gap-1 flex-wrap">
                  {skills.map((s, i) => (
                    <span key={i} className="text-[10px] font-mono bg-ciab-bg-elevated border border-ciab-border px-1.5 py-0.5 rounded">
                      {s.name ?? s.source.split("/").pop()}
                    </span>
                  ))}
                </span>
              }
            />
          )}
          {/* Repos */}
          {repos.length > 0 && (
            <InheritedRow
              label="Repos"
              value={`${repos.length} repositor${repos.length > 1 ? "ies" : "y"}`}
            />
          )}
          {/* Env vars */}
          {envCount > 0 && (
            <InheritedRow
              label="Env vars"
              value={`${envCount} variable${envCount > 1 ? "s" : ""}`}
            />
          )}
        </div>
      </div>

      {/* Model override — always visible */}
      <div>
        <label className="label mb-1 flex items-center gap-1.5">
          LLM Model
          <span className="text-ciab-text-muted/50 normal-case tracking-normal font-normal">(optional override)</span>
        </label>
        <p className="text-[11px] text-ciab-text-muted mb-2">
          Use a different model for this launch — e.g. switch to a local Ollama model instead of the default.
        </p>
        {loadingProviders ? (
          <div className="flex items-center gap-2 py-2 text-[11px] text-ciab-text-muted">
            <Loader2 className="w-3.5 h-3.5 animate-spin" />
            Loading providers…
          </div>
        ) : (
          <ModelPickerWithProviders
            agentProvider={agentProvider}
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

      {/* Note */}
      <div className="rounded-md bg-ciab-bg-primary/40 border border-ciab-border/50 px-3 py-2">
        <p className="text-[10px] text-ciab-text-muted">
          Model overrides apply to this launch only — the workspace spec is unchanged.
        </p>
      </div>

      {/* Launch button */}
      <button
        onClick={handleLaunch}
        disabled={isLaunching}
        className="w-full btn-primary flex items-center justify-center gap-2 py-2.5 disabled:opacity-50"
      >
        {isLaunching ? (
          <Loader2 className="w-4 h-4 animate-spin" />
        ) : (
          <Play className="w-4 h-4" />
        )}
        {isLaunching ? "Launching…" : "Launch Sandbox"}
      </button>
    </div>
  );
}


function InheritedRow({
  label,
  value,
}: {
  label: string;
  value: React.ReactNode;
}) {
  return (
    <div className="flex items-start gap-3 px-3 py-2">
      <span className="text-[10px] font-mono text-ciab-text-muted/70 w-16 flex-shrink-0 pt-0.5 uppercase tracking-wide">{label}</span>
      <span className="text-[11px] font-mono text-ciab-text-secondary flex-1 min-w-0">{value}</span>
    </div>
  );
}

/* ─── Model picker with per-provider model fetch ─── */

function ModelPickerWithProviders({
  agentProvider: _agentProvider,
  compatibleProviders,
  value,
  onChange,
  hostAuth,
}: {
  agentProvider: string;
  compatibleProviders: { id: string; name: string; kind: string; enabled: boolean }[];
  value: string;
  onChange: (modelId: string, providerId: string) => void;
  hostAuth?: { found: boolean; expired: boolean; subscription_type: string | null; message: string } | null;
}) {
  const modelsMap: Record<string, import("@/lib/api/types").LlmModel[]> = {};
  for (const p of compatibleProviders) {
    const { data } = useLlmProviderModels(p.id);
    if (data) modelsMap[p.id] = data;
  }

  return (
    <ModelPicker
      providers={compatibleProviders as import("@/lib/api/types").LlmProvider[]}
      models={modelsMap}
      value={value}
      onChange={onChange}
      className="w-full"
      hostAuth={hostAuth}
    />
  );
}
