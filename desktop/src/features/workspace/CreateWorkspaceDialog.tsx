import { useState } from "react";
import {
  X,
  Plus,
  Trash2,
  GitBranch,
  Zap,
  Terminal,
  Package,
  ChevronDown,
  ChevronRight,
  Sparkles,
  Loader2,
  FileCode2,
  Monitor,
} from "lucide-react";
import { useCreateWorkspace } from "@/lib/hooks/use-workspaces";
import { useTemplates } from "@/lib/hooks/use-templates";
import AgentProviderIcon from "@/components/shared/AgentProviderIcon";
import type {
  Workspace,
  WorkspaceRepo,
  PreCommand,
  BinaryInstall,
  WorkspaceSpec,
  RuntimeBackend,
} from "@/lib/api/types";

const PROVIDERS = [
  { value: "claude-code", label: "Claude Code", org: "Anthropic" },
  { value: "codex", label: "Codex", org: "OpenAI" },
  { value: "gemini", label: "Gemini CLI", org: "Google" },
  { value: "cursor", label: "Cursor", org: "Anysphere" },
];

interface Props {
  onClose: () => void;
}

export default function CreateWorkspaceDialog({ onClose }: Props) {
  const createWorkspace = useCreateWorkspace();
  const { data: templates, isLoading: templatesLoading } = useTemplates();
  const [step, setStep] = useState<"template" | "configure">("template");
  const [name, setName] = useState("");
  const [description, setDescription] = useState("");
  const [provider, setProvider] = useState("claude-code");
  const [repos, setRepos] = useState<WorkspaceRepo[]>([]);
  const [preCommands, setPreCommands] = useState<PreCommand[]>([]);
  const [binaries, setBinaries] = useState<BinaryInstall[]>([]);
  const [envVars, setEnvVars] = useState<Array<{ key: string; value: string }>>([]);
  const [systemPrompt, setSystemPrompt] = useState("");
  const [runtimeBackend, setRuntimeBackend] = useState<RuntimeBackend>("default");

  // Collapsible sections
  const [showRepos, setShowRepos] = useState(false);
  const [showCommands, setShowCommands] = useState(false);
  const [showBinaries, setShowBinaries] = useState(false);
  const [showEnv, setShowEnv] = useState(false);
  const [showAgent, setShowAgent] = useState(false);

  const applyTemplate = (template: Workspace | null) => {
    if (!template) {
      // Blank
      setStep("configure");
      return;
    }
    const spec = template.spec;
    setProvider(spec.agent?.provider ?? "claude-code");
    setRepos(spec.repositories ?? []);
    setPreCommands(spec.pre_commands ?? []);
    setBinaries(spec.binaries ?? []);
    if (spec.env_vars) {
      setEnvVars(Object.entries(spec.env_vars).map(([key, value]) => ({ key, value })));
    }
    if (spec.agent?.system_prompt) {
      setSystemPrompt(spec.agent.system_prompt);
    }
    if (spec.repositories && spec.repositories.length > 0) {
      setShowRepos(true);
    }
    setStep("configure");
  };

  const handleSubmit = () => {
    const envMap: Record<string, string> = {};
    envVars.forEach(({ key, value }) => {
      if (key.trim()) envMap[key.trim()] = value;
    });

    const spec: WorkspaceSpec = {
      agent: {
        provider,
        ...(systemPrompt.trim() ? { system_prompt: systemPrompt.trim() } : {}),
      },
      ...(repos.length > 0 ? { repositories: repos.filter((r) => r.url.trim()) } : {}),
      ...(preCommands.length > 0 ? { pre_commands: preCommands.filter((c) => c.command.trim()) } : {}),
      ...(binaries.length > 0 ? { binaries: binaries.filter((b) => b.name.trim()) } : {}),
      ...(Object.keys(envMap).length > 0 ? { env_vars: envMap } : {}),
      ...(runtimeBackend !== "default" ? { runtime: { backend: runtimeBackend } } : {}),
    };

    createWorkspace.mutate(
      {
        name: name.trim(),
        description: description.trim() || undefined,
        spec,
      },
      { onSuccess: onClose }
    );
  };

  return (
    <div
      className="fixed inset-0 bg-black/70 backdrop-blur-sm flex items-end sm:items-center justify-center z-50 animate-fade-in"
      onClick={onClose}
    >
      <div
        className="bg-ciab-bg-card border border-ciab-border rounded-t-xl sm:rounded-xl w-full sm:max-w-xl max-h-[90vh] sm:max-h-[85vh] flex flex-col animate-scale-in overflow-hidden"
        onClick={(e) => e.stopPropagation()}
      >
        {/* Accent */}
        <div className="h-1 bg-gradient-to-r from-ciab-copper to-ciab-copper-light" />

        {/* Header */}
        <div className="flex items-center justify-between p-4 border-b border-ciab-border flex-shrink-0">
          <div className="flex items-center gap-2">
            {step === "template" ? (
              <Sparkles className="w-4 h-4 text-ciab-copper" />
            ) : (
              <FileCode2 className="w-4 h-4 text-ciab-copper" />
            )}
            <h2 className="text-sm font-semibold">
              {step === "template" ? "Start from a Template" : "Configure Workspace"}
            </h2>
          </div>
          <button onClick={onClose} className="text-ciab-text-muted hover:text-ciab-text-primary transition-colors p-0.5">
            <X className="w-4 h-4" />
          </button>
        </div>

        {step === "template" ? (
          /* Template selection */
          <div className="p-4 overflow-y-auto flex-1">
            {/* Blank option */}
            <button
              onClick={() => applyTemplate(null)}
              className="w-full card-hover p-3.5 text-left mb-3 flex items-center gap-3"
            >
              <div className="w-9 h-9 rounded-lg bg-ciab-bg-primary border border-ciab-border flex items-center justify-center flex-shrink-0">
                <FileCode2 className="w-4 h-4 text-ciab-text-muted" />
              </div>
              <div>
                <p className="text-sm font-medium">Blank Workspace</p>
                <p className="text-[10px] text-ciab-text-muted">Start from scratch, configure everything manually</p>
              </div>
            </button>

            {templatesLoading && (
              <div className="flex items-center justify-center py-6">
                <Loader2 className="w-4 h-4 text-ciab-copper animate-spin" />
              </div>
            )}

            {templates && templates.length > 0 && (
              <>
                <p className="text-[9px] font-mono text-ciab-text-muted uppercase tracking-wider mb-2">
                  Templates ({templates.length})
                </p>
                <div className="grid grid-cols-2 gap-2">
                  {templates.map((t) => (
                    <button
                      key={t.id}
                      onClick={() => applyTemplate(t)}
                      className="card-hover p-3 text-left group"
                    >
                      <div className="flex items-center gap-2 mb-1.5">
                        {t.spec.agent?.provider ? (
                          <AgentProviderIcon provider={t.spec.agent.provider} size={14} />
                        ) : (
                          <Sparkles className="w-3.5 h-3.5 text-ciab-copper" />
                        )}
                        <span className="text-sm font-medium truncate group-hover:text-ciab-copper transition-colors">
                          {t.name}
                        </span>
                      </div>
                      {t.description && (
                        <p className="text-[10px] text-ciab-text-muted leading-relaxed line-clamp-2">
                          {t.description}
                        </p>
                      )}
                      <div className="flex items-center gap-2 mt-1.5 text-[9px] font-mono text-ciab-text-muted/50">
                        {(t.spec.repositories?.length ?? 0) > 0 && (
                          <span>{t.spec.repositories!.length} repos</span>
                        )}
                        {(t.spec.skills?.length ?? 0) > 0 && (
                          <span>{t.spec.skills!.length} skills</span>
                        )}
                      </div>
                    </button>
                  ))}
                </div>
              </>
            )}
          </div>
        ) : (
          /* Configuration form */
          <div className="p-4 overflow-y-auto flex-1 space-y-3">
            {/* Name & Description */}
            <div className="grid grid-cols-2 gap-2">
              <div>
                <label className="label">Name</label>
                <input
                  type="text"
                  value={name}
                  onChange={(e) => setName(e.target.value)}
                  className="input w-full"
                  placeholder="my-workspace"
                  autoFocus
                />
              </div>
              <div>
                <label className="label">Description</label>
                <input
                  type="text"
                  value={description}
                  onChange={(e) => setDescription(e.target.value)}
                  className="input w-full"
                  placeholder="Optional description"
                />
              </div>
            </div>

            {/* Runtime Backend */}
            <div>
              <label className="label">Runtime Backend</label>
              <div className="grid grid-cols-2 sm:grid-cols-4 gap-1.5">
                {([
                  { value: "default" as RuntimeBackend, label: "Default" },
                  { value: "local" as RuntimeBackend, label: "Local" },
                  { value: "opensandbox" as RuntimeBackend, label: "OpenSandbox" },
                  { value: "docker" as RuntimeBackend, label: "Docker" },
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
                    <Monitor className="w-3.5 h-3.5 text-ciab-text-muted" />
                    <span className="text-[11px] font-medium truncate">{b.label}</span>
                  </button>
                ))}
              </div>
            </div>

            {/* Provider selection */}
            <div>
              <label className="label">Agent Provider</label>
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
                    <AgentProviderIcon provider={p.value} size={14} />
                    <span className="text-[11px] font-medium truncate">{p.label}</span>
                  </button>
                ))}
              </div>
            </div>

            {/* Collapsible: Repositories */}
            <CollapsibleSection
              title="Repositories"
              icon={GitBranch}
              open={showRepos}
              onToggle={() => setShowRepos(!showRepos)}
              count={repos.length}
            >
              <div className="space-y-2">
                {repos.map((repo, i) => (
                  <div key={i} className="flex items-start gap-1.5">
                    <div className="flex-1 space-y-1">
                      <input
                        type="text"
                        value={repo.url}
                        onChange={(e) => {
                          const updated = [...repos];
                          updated[i] = { ...updated[i], url: e.target.value };
                          setRepos(updated);
                        }}
                        className="input w-full font-mono text-[11px]"
                        placeholder="https://github.com/org/repo.git"
                      />
                      <div className="flex gap-1">
                        <input
                          type="text"
                          value={repo.branch ?? ""}
                          onChange={(e) => {
                            const updated = [...repos];
                            updated[i] = { ...updated[i], branch: e.target.value || undefined };
                            setRepos(updated);
                          }}
                          className="input flex-1 font-mono text-[11px] py-1"
                          placeholder="branch (main)"
                        />
                        <input
                          type="text"
                          value={repo.dest_path ?? ""}
                          onChange={(e) => {
                            const updated = [...repos];
                            updated[i] = { ...updated[i], dest_path: e.target.value || undefined };
                            setRepos(updated);
                          }}
                          className="input flex-1 font-mono text-[11px] py-1"
                          placeholder="dest (/workspace/app)"
                        />
                      </div>
                    </div>
                    <button
                      onClick={() => setRepos(repos.filter((_, j) => j !== i))}
                      className="p-1 text-ciab-text-muted hover:text-state-failed transition-colors mt-1"
                    >
                      <Trash2 className="w-3 h-3" />
                    </button>
                  </div>
                ))}
                <button
                  onClick={() => setRepos([...repos, { url: "", branch: "main", dest_path: "/workspace/app" }])}
                  className="text-[11px] text-ciab-text-muted hover:text-ciab-copper transition-colors flex items-center gap-1"
                >
                  <Plus className="w-3 h-3" /> Add repository
                </button>
              </div>
            </CollapsibleSection>

            {/* Collapsible: Binaries */}
            <CollapsibleSection
              title="Binaries"
              icon={Package}
              open={showBinaries}
              onToggle={() => setShowBinaries(!showBinaries)}
              count={binaries.length}
            >
              <div className="space-y-1.5">
                {binaries.map((bin, i) => (
                  <div key={i} className="flex items-center gap-1.5">
                    <input
                      type="text"
                      value={bin.name}
                      onChange={(e) => {
                        const updated = [...binaries];
                        updated[i] = { ...updated[i], name: e.target.value };
                        setBinaries(updated);
                      }}
                      className="input flex-1 font-mono text-[11px] py-1"
                      placeholder="package-name"
                    />
                    <select
                      value={bin.method ?? "apt"}
                      onChange={(e) => {
                        const updated = [...binaries];
                        updated[i] = { ...updated[i], method: e.target.value as BinaryInstall["method"] };
                        setBinaries(updated);
                      }}
                      className="input text-[11px] py-1 w-20"
                    >
                      <option value="apt">apt</option>
                      <option value="npm">npm</option>
                      <option value="cargo">cargo</option>
                      <option value="pip">pip</option>
                    </select>
                    <button
                      onClick={() => setBinaries(binaries.filter((_, j) => j !== i))}
                      className="p-1 text-ciab-text-muted hover:text-state-failed transition-colors"
                    >
                      <Trash2 className="w-3 h-3" />
                    </button>
                  </div>
                ))}
                <button
                  onClick={() => setBinaries([...binaries, { name: "", method: "apt" }])}
                  className="text-[11px] text-ciab-text-muted hover:text-ciab-copper transition-colors flex items-center gap-1"
                >
                  <Plus className="w-3 h-3" /> Add binary
                </button>
              </div>
            </CollapsibleSection>

            {/* Collapsible: Pre-commands */}
            <CollapsibleSection
              title="Pre-commands"
              icon={Terminal}
              open={showCommands}
              onToggle={() => setShowCommands(!showCommands)}
              count={preCommands.length}
            >
              <div className="space-y-1.5">
                {preCommands.map((cmd, i) => (
                  <div key={i} className="flex items-center gap-1.5">
                    <input
                      type="text"
                      value={cmd.name ?? ""}
                      onChange={(e) => {
                        const updated = [...preCommands];
                        updated[i] = { ...updated[i], name: e.target.value || undefined };
                        setPreCommands(updated);
                      }}
                      className="input w-24 text-[11px] py-1"
                      placeholder="label"
                    />
                    <input
                      type="text"
                      value={cmd.command}
                      onChange={(e) => {
                        const updated = [...preCommands];
                        updated[i] = { ...updated[i], command: e.target.value };
                        setPreCommands(updated);
                      }}
                      className="input flex-1 font-mono text-[11px] py-1"
                      placeholder="npm install"
                    />
                    <input
                      type="text"
                      value={cmd.workdir ?? ""}
                      onChange={(e) => {
                        const updated = [...preCommands];
                        updated[i] = { ...updated[i], workdir: e.target.value || undefined };
                        setPreCommands(updated);
                      }}
                      className="input w-28 font-mono text-[11px] py-1"
                      placeholder="/workspace"
                    />
                    <button
                      onClick={() => setPreCommands(preCommands.filter((_, j) => j !== i))}
                      className="p-1 text-ciab-text-muted hover:text-state-failed transition-colors"
                    >
                      <Trash2 className="w-3 h-3" />
                    </button>
                  </div>
                ))}
                <button
                  onClick={() => setPreCommands([...preCommands, { command: "", workdir: "/workspace/app" }])}
                  className="text-[11px] text-ciab-text-muted hover:text-ciab-copper transition-colors flex items-center gap-1"
                >
                  <Plus className="w-3 h-3" /> Add command
                </button>
              </div>
            </CollapsibleSection>

            {/* Collapsible: Environment Variables */}
            <CollapsibleSection
              title="Environment"
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
            </CollapsibleSection>

            {/* Collapsible: Agent Config */}
            <CollapsibleSection
              title="Agent Configuration"
              icon={Zap}
              open={showAgent}
              onToggle={() => setShowAgent(!showAgent)}
            >
              <div>
                <label className="label">System Prompt</label>
                <textarea
                  value={systemPrompt}
                  onChange={(e) => setSystemPrompt(e.target.value)}
                  className="input w-full resize-none text-xs font-mono"
                  rows={3}
                  placeholder="You are a senior developer working on this project..."
                />
              </div>
            </CollapsibleSection>
          </div>
        )}

        {/* Footer */}
        <div className="flex items-center justify-between p-4 border-t border-ciab-border flex-shrink-0">
          {step === "configure" ? (
            <>
              <button
                onClick={() => setStep("template")}
                className="text-xs text-ciab-text-muted hover:text-ciab-text-secondary transition-colors"
              >
                Back to templates
              </button>
              <div className="flex gap-2">
                <button onClick={onClose} className="btn-secondary text-sm px-3 py-1.5">
                  Cancel
                </button>
                <button
                  onClick={handleSubmit}
                  className="btn-primary disabled:opacity-30 text-sm px-3 py-1.5"
                  disabled={!name.trim() || createWorkspace.isPending}
                >
                  {createWorkspace.isPending ? "Creating..." : "Create Workspace"}
                </button>
              </div>
            </>
          ) : (
            <div className="ml-auto">
              <button onClick={onClose} className="btn-secondary text-sm px-3 py-1.5">
                Cancel
              </button>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}

function CollapsibleSection({
  title,
  icon: Icon,
  open,
  onToggle,
  count,
  children,
}: {
  title: string;
  icon: typeof GitBranch;
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
        <div className="px-3 pb-3 pt-1 animate-fade-in">
          {children}
        </div>
      )}
    </div>
  );
}
