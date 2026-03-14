import { useState, useCallback, useEffect, useRef } from "react";
import { useParams, useNavigate } from "react-router";
import {
  ArrowLeft,
  Play,
  Download,
  Trash2,
  GitBranch,
  Zap,
  Terminal,
  HardDrive,
  Bot,
  KeyRound,
  Settings2,
  Users,
  Plus,
  X,
  Save,
  Undo2,
  ChevronDown,
  ChevronRight,
  FolderOpen,
  FileText,
  Eye,
  EyeOff,
  Upload,
  Package,
  Monitor,
  AlertTriangle,
  Search,
  Github,
  Lock,
  Globe,
  Loader2,
} from "lucide-react";
import {
  useWorkspace,
  useDeleteWorkspace,
  useLaunchWorkspace,
  useUpdateWorkspace,
} from "@/lib/hooks/use-workspaces";
import type {
  WorkspaceSpec,
  WorkspaceSkill,
  WorkspaceRepo,
  LocalMount,
  SyncMode,
  PreCommand,
  BinaryInstall,
  FilesystemConfig,
  WorkspaceAgentConfig,
  SubagentConfig,
  WorkspaceCredential,
  RuntimeBackend,
  GitCloneStrategy,
  WorkspaceRuntimeConfig,
} from "@/lib/api/types";
import SkillsManager from "@/features/workspace/SkillsManager";
import { formatRelativeTime } from "@/lib/utils/format";
import LoadingSpinner from "@/components/shared/LoadingSpinner";
import { useDirectoryPicker } from "@/lib/hooks/use-directory-picker";
import { useGitHubRepos, type GitHubRepo } from "@/lib/hooks/use-github-repos";

const tabs = [
  { id: "overview", label: "Overview", icon: Settings2 },
  { id: "runtime", label: "Runtime", icon: Monitor },
  { id: "repos", label: "Repositories", icon: GitBranch },
  { id: "skills", label: "Skills", icon: Zap },
  { id: "agent", label: "Agent", icon: Bot },
  { id: "subagents", label: "Subagents", icon: Users },
  { id: "commands", label: "Commands", icon: Terminal },
  { id: "filesystem", label: "Filesystem", icon: HardDrive },
  { id: "credentials", label: "Credentials", icon: KeyRound },
] as const;

type TabId = (typeof tabs)[number]["id"];

export default function WorkspaceDetail() {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const { data: workspace, isLoading } = useWorkspace(id!);
  const deleteWorkspace = useDeleteWorkspace();
  const launchWorkspace = useLaunchWorkspace();
  const updateWorkspace = useUpdateWorkspace();
  const [activeTab, setActiveTab] = useState<TabId>("overview");

  // Edit state — null means no edits pending
  const [editSpec, setEditSpec] = useState<WorkspaceSpec | null>(null);
  const [editName, setEditName] = useState<string | null>(null);
  const [editDescription, setEditDescription] = useState<string | null>(null);

  const hasEdits = editSpec !== null || editName !== null || editDescription !== null;

  // Reset edits when workspace data changes (e.g. after save)
  useEffect(() => {
    setEditSpec(null);
    setEditName(null);
    setEditDescription(null);
  }, [workspace?.updated_at]);

  const currentSpec = editSpec ?? workspace?.spec;
  const currentName = editName ?? workspace?.name;
  const currentDescription = editDescription ?? workspace?.description;

  const updateSpec = useCallback(
    (updater: (prev: WorkspaceSpec) => WorkspaceSpec) => {
      setEditSpec((prev) => {
        const base = prev ?? workspace?.spec ?? {};
        return updater(base as WorkspaceSpec);
      });
    },
    [workspace?.spec]
  );

  const handleSave = useCallback(() => {
    if (!workspace || !hasEdits) return;
    updateWorkspace.mutate({
      id: workspace.id,
      ...(editName !== null ? { name: editName } : {}),
      ...(editDescription !== null ? { description: editDescription } : {}),
      ...(editSpec !== null ? { spec: editSpec } : {}),
    });
  }, [workspace, editSpec, editName, editDescription, hasEdits, updateWorkspace]);

  const handleDiscard = useCallback(() => {
    setEditSpec(null);
    setEditName(null);
    setEditDescription(null);
  }, []);

  const handleSkillsUpdate = useCallback(
    (skills: WorkspaceSkill[]) => {
      updateSpec((prev) => ({ ...prev, skills }));
    },
    [updateSpec]
  );

  if (isLoading || !workspace) {
    return (
      <div className="flex items-center justify-center h-64">
        <LoadingSpinner />
      </div>
    );
  }

  return (
    <div className="flex flex-col h-[calc(100vh-8rem)]">
      {/* Header */}
      <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-3 mb-4">
        <div className="flex items-center gap-3 min-w-0">
          <button
            onClick={() => navigate("/workspaces")}
            className="btn-ghost p-1.5 flex-shrink-0"
          >
            <ArrowLeft className="w-5 h-5" />
          </button>
          <div className="min-w-0">
            <h1 className="text-xl font-bold truncate">{currentName}</h1>
            {currentDescription && (
              <p className="text-sm text-ciab-text-muted truncate">
                {currentDescription}
              </p>
            )}
          </div>
        </div>

        <div className="flex items-center gap-2 flex-shrink-0 pl-10 sm:pl-0">
          {hasEdits && (
            <>
              <button
                onClick={handleSave}
                className="btn-primary flex items-center gap-2"
                disabled={updateWorkspace.isPending}
              >
                <Save className="w-4 h-4" />
                Save
              </button>
              <button
                onClick={handleDiscard}
                className="btn-secondary flex items-center gap-2"
              >
                <Undo2 className="w-4 h-4" />
                Discard
              </button>
            </>
          )}
          <button
            onClick={() => launchWorkspace.mutate(workspace.id)}
            className="btn-primary flex items-center gap-2"
            disabled={launchWorkspace.isPending}
          >
            <Play className="w-4 h-4" />
            Launch
          </button>
          <button className="btn-secondary flex items-center gap-2">
            <Download className="w-4 h-4" />
            Export
          </button>
          <button
            onClick={() => {
              if (confirm("Delete this workspace?")) {
                deleteWorkspace.mutate(workspace.id, {
                  onSuccess: () => navigate("/workspaces"),
                });
              }
            }}
            className="p-1.5 rounded hover:bg-state-failed/10 text-state-failed transition-colors"
            title="Delete"
          >
            <Trash2 className="w-4 h-4" />
          </button>
        </div>
      </div>

      {/* Unsaved changes banner */}
      {hasEdits && (
        <div className="mb-3 px-3 py-2 rounded-lg border border-ciab-copper/30 bg-ciab-copper/5 text-xs text-ciab-copper flex items-center gap-2 animate-fade-in">
          <div className="w-1.5 h-1.5 rounded-full bg-ciab-copper animate-glow-pulse" />
          You have unsaved changes
        </div>
      )}

      {/* Tabs */}
      <div className="flex items-center gap-1 border-b border-ciab-border mb-4 overflow-x-auto scrollbar-none">
        {tabs.map((tab) => (
          <button
            key={tab.id}
            onClick={() => setActiveTab(tab.id)}
            className={`flex items-center gap-2 px-4 py-2.5 text-sm font-medium border-b-2 transition-colors ${
              activeTab === tab.id
                ? "border-ciab-copper text-ciab-copper"
                : "border-transparent text-ciab-text-secondary hover:text-ciab-text-primary"
            }`}
          >
            <tab.icon className="w-4 h-4" />
            {tab.label}
          </button>
        ))}
      </div>

      {/* Tab content */}
      <div className="flex-1 min-h-0 overflow-auto">
        {activeTab === "overview" && currentSpec && (
          <OverviewTab
            spec={currentSpec}
            name={currentName ?? ""}
            description={currentDescription ?? ""}
            createdAt={workspace.created_at}
            updatedAt={workspace.updated_at}
            onNameChange={(v) => setEditName(v)}
            onDescriptionChange={(v) => setEditDescription(v)}
            onSpecChange={updateSpec}
          />
        )}

        {activeTab === "runtime" && currentSpec && (
          <RuntimeTab spec={currentSpec} onSpecChange={updateSpec} />
        )}

        {activeTab === "repos" && currentSpec && (
          <ReposTab spec={currentSpec} onSpecChange={updateSpec} />
        )}

        {activeTab === "skills" && currentSpec && (
          <SkillsManager
            skills={currentSpec.skills ?? []}
            onUpdate={handleSkillsUpdate}
          />
        )}

        {activeTab === "agent" && currentSpec && (
          <AgentTab spec={currentSpec} onSpecChange={updateSpec} />
        )}

        {activeTab === "subagents" && currentSpec && (
          <SubagentsTab spec={currentSpec} onSpecChange={updateSpec} />
        )}

        {activeTab === "commands" && currentSpec && (
          <CommandsTab spec={currentSpec} onSpecChange={updateSpec} />
        )}

        {activeTab === "filesystem" && currentSpec && (
          <FilesystemTab spec={currentSpec} onSpecChange={updateSpec} />
        )}

        {activeTab === "credentials" && currentSpec && (
          <CredentialsTab spec={currentSpec} onSpecChange={updateSpec} />
        )}
      </div>
    </div>
  );
}

// ---------------------------------------------------------------------------
// Overview Tab
// ---------------------------------------------------------------------------

function OverviewTab({
  spec,
  name,
  description,
  createdAt,
  updatedAt,
  onNameChange,
  onDescriptionChange,
  onSpecChange,
}: {
  spec: WorkspaceSpec;
  name: string;
  description: string;
  createdAt: string;
  updatedAt: string;
  onNameChange: (v: string) => void;
  onDescriptionChange: (v: string) => void;
  onSpecChange: (fn: (s: WorkspaceSpec) => WorkspaceSpec) => void;
}) {
  const [showValues, setShowValues] = useState(false);

  return (
    <div className="space-y-6">
      {/* Stats */}
      <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
        <StatCard label="Repositories" value={spec.repositories?.length ?? 0} />
        <StatCard label="Local Mounts" value={spec.local_mounts?.length ?? 0} />
        <StatCard label="Skills" value={spec.skills?.length ?? 0} />
        <StatCard label="Subagents" value={spec.subagents?.length ?? 0} />
      </div>

      {/* Name & Description */}
      <div className="card p-4 space-y-3">
        <h3 className="text-sm font-medium text-ciab-text-primary">Details</h3>
        <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
          <div>
            <label className="label">Name</label>
            <input
              type="text"
              className="input w-full"
              value={name}
              onChange={(e) => onNameChange(e.target.value)}
            />
          </div>
          <div>
            <label className="label">Description</label>
            <input
              type="text"
              className="input w-full"
              value={description}
              onChange={(e) => onDescriptionChange(e.target.value)}
              placeholder="Workspace description..."
            />
          </div>
        </div>
      </div>

      {/* Environment Variables */}
      <div className="card p-4 space-y-3">
        <div className="flex items-center justify-between">
          <h3 className="text-sm font-medium text-ciab-text-primary">
            Environment Variables
          </h3>
          <div className="flex items-center gap-2">
            <button
              onClick={() => setShowValues((v) => !v)}
              className="p-1.5 rounded-lg text-ciab-text-muted hover:text-ciab-text-secondary transition-colors"
              title={showValues ? "Hide values" : "Show values"}
            >
              {showValues ? (
                <EyeOff className="w-3.5 h-3.5" />
              ) : (
                <Eye className="w-3.5 h-3.5" />
              )}
            </button>
            <button
              onClick={() =>
                onSpecChange((s) => ({
                  ...s,
                  env_vars: { ...s.env_vars, "": "" },
                }))
              }
              className="flex items-center gap-1 text-[11px] text-ciab-text-muted hover:text-ciab-copper transition-colors"
            >
              <Plus className="w-3 h-3" /> Add
            </button>
          </div>
        </div>

        {spec.env_file && (
          <div className="flex items-center gap-2 text-xs text-ciab-text-muted bg-ciab-bg-primary rounded-lg px-3 py-2 border border-ciab-border">
            <FileText className="w-3.5 h-3.5" />
            <span>
              Loading from: <code className="text-ciab-copper">{spec.env_file}</code>
            </span>
            <button
              onClick={() => onSpecChange((s) => ({ ...s, env_file: undefined }))}
              className="ml-auto p-0.5 hover:text-state-failed transition-colors"
            >
              <X className="w-3 h-3" />
            </button>
          </div>
        )}

        <EnvVarsEditor
          envVars={spec.env_vars ?? {}}
          showValues={showValues}
          onChange={(env_vars) => onSpecChange((s) => ({ ...s, env_vars }))}
        />

        {/* .env file upload */}
        <EnvFileUpload
          onParsed={(vars) =>
            onSpecChange((s) => ({
              ...s,
              env_vars: { ...vars, ...s.env_vars },
            }))
          }
        />
      </div>

      <div className="text-xs text-ciab-text-muted">
        Created {formatRelativeTime(createdAt)} · Updated{" "}
        {formatRelativeTime(updatedAt)}
      </div>
    </div>
  );
}

// ---------------------------------------------------------------------------
// Env Vars Editor
// ---------------------------------------------------------------------------

function EnvVarsEditor({
  envVars,
  showValues,
  onChange,
}: {
  envVars: Record<string, string>;
  showValues: boolean;
  onChange: (vars: Record<string, string>) => void;
}) {
  const entries = Object.entries(envVars);

  if (entries.length === 0) {
    return (
      <p className="text-xs text-ciab-text-muted py-2">
        No environment variables configured
      </p>
    );
  }

  return (
    <div className="space-y-1.5">
      {entries.map(([key, value], i) => (
        <div key={i} className="flex items-center gap-1.5">
          <input
            type="text"
            className="input flex-1 font-mono text-[11px] py-1"
            value={key}
            placeholder="KEY"
            onChange={(e) => {
              const newEntries = [...entries];
              newEntries[i] = [e.target.value, value];
              onChange(Object.fromEntries(newEntries));
            }}
          />
          <span className="text-ciab-text-muted text-xs">=</span>
          <input
            type={showValues ? "text" : "password"}
            className="input flex-1 font-mono text-[11px] py-1"
            value={value}
            placeholder="value"
            onChange={(e) => {
              const newEntries = [...entries];
              newEntries[i] = [key, e.target.value];
              onChange(Object.fromEntries(newEntries));
            }}
          />
          <button
            onClick={() => {
              const newEntries = entries.filter((_, j) => j !== i);
              onChange(Object.fromEntries(newEntries));
            }}
            className="p-1 text-ciab-text-muted hover:text-state-failed transition-colors"
          >
            <Trash2 className="w-3 h-3" />
          </button>
        </div>
      ))}
    </div>
  );
}

// ---------------------------------------------------------------------------
// .env File Upload
// ---------------------------------------------------------------------------

function EnvFileUpload({
  onParsed,
}: {
  onParsed: (vars: Record<string, string>) => void;
}) {
  const [isDragging, setIsDragging] = useState(false);
  const fileInputRef = useRef<HTMLInputElement>(null);

  const parseEnvContent = (content: string) => {
    const vars: Record<string, string> = {};
    for (const line of content.split("\n")) {
      const trimmed = line.trim();
      if (!trimmed || trimmed.startsWith("#")) continue;
      const eqIdx = trimmed.indexOf("=");
      if (eqIdx === -1) continue;
      const key = trimmed.slice(0, eqIdx).trim();
      let value = trimmed.slice(eqIdx + 1).trim();
      if (
        (value.startsWith('"') && value.endsWith('"')) ||
        (value.startsWith("'") && value.endsWith("'"))
      ) {
        value = value.slice(1, -1);
      }
      if (key) vars[key] = value;
    }
    return vars;
  };

  const handleFile = (file: File) => {
    const reader = new FileReader();
    reader.onload = (e) => {
      const content = e.target?.result as string;
      const vars = parseEnvContent(content);
      if (Object.keys(vars).length > 0) {
        onParsed(vars);
      }
    };
    reader.readAsText(file);
  };

  return (
    <div
      className={`border border-dashed rounded-lg px-3 py-2 text-center transition-colors cursor-pointer ${
        isDragging
          ? "border-ciab-copper bg-ciab-copper/5"
          : "border-ciab-border hover:border-ciab-border-light"
      }`}
      onDragOver={(e) => {
        e.preventDefault();
        setIsDragging(true);
      }}
      onDragLeave={() => setIsDragging(false)}
      onDrop={(e) => {
        e.preventDefault();
        setIsDragging(false);
        const file = e.dataTransfer.files[0];
        if (file) handleFile(file);
      }}
      onClick={() => fileInputRef.current?.click()}
    >
      <input
        ref={fileInputRef}
        type="file"
        className="hidden"
        accept=".env,.env.*"
        onChange={(e) => {
          const file = e.target.files?.[0];
          if (file) handleFile(file);
        }}
      />
      <div className="flex items-center justify-center gap-2 text-xs text-ciab-text-muted">
        <Upload className="w-3.5 h-3.5" />
        <span>
          Drop <code>.env</code> file or click to upload (merges into env vars)
        </span>
      </div>
    </div>
  );
}

// ---------------------------------------------------------------------------
// Runtime Tab
// ---------------------------------------------------------------------------

function RuntimeTab({
  spec,
  onSpecChange,
}: {
  spec: WorkspaceSpec;
  onSpecChange: (fn: (s: WorkspaceSpec) => WorkspaceSpec) => void;
}) {
  const runtime = spec.runtime ?? {};
  const backend = runtime.backend ?? "default";

  const updateRuntime = (updater: (r: WorkspaceRuntimeConfig) => WorkspaceRuntimeConfig) => {
    onSpecChange((s) => ({
      ...s,
      runtime: updater(s.runtime ?? {}),
    }));
  };

  const backends: { value: RuntimeBackend; label: string; description: string }[] = [
    { value: "default", label: "Default", description: "Inherit from server config.toml" },
    { value: "local", label: "Local", description: "Run agents as local processes" },
    { value: "opensandbox", label: "OpenSandbox", description: "Run in OpenSandbox containers" },
    { value: "docker", label: "Docker", description: "Run in Docker containers" },
  ];

  return (
    <div className="card p-4 space-y-4">
      <h3 className="text-sm font-medium text-ciab-text-primary">Runtime Backend</h3>
      <p className="text-xs text-ciab-text-muted -mt-2">
        Choose how this workspace's sandboxes are executed. "Default" inherits from server config.toml.
      </p>

      <div className="grid grid-cols-2 sm:grid-cols-4 gap-2">
        {backends.map((b) => (
          <button
            key={b.value}
            onClick={() => updateRuntime((r) => ({ ...r, backend: b.value }))}
            className={`px-3 py-3 rounded-lg border text-left transition-all ${
              backend === b.value
                ? "border-ciab-copper/50 bg-ciab-copper/5"
                : "border-ciab-border hover:border-ciab-border-light"
            }`}
          >
            <div className={`text-xs font-semibold ${backend === b.value ? "text-ciab-copper" : "text-ciab-text-primary"}`}>
              {b.label}
            </div>
            <div className="text-[10px] text-ciab-text-muted mt-0.5">{b.description}</div>
          </button>
        ))}
      </div>

      {backend === "local" && (
        <div className="animate-fade-in">
          <label className="label">Local Working Directory (override)</label>
          <DirectoryPickerInput
            value={runtime.local_workdir ?? ""}
            onChange={(v) =>
              updateRuntime((r) => ({
                ...r,
                local_workdir: v || undefined,
              }))
            }
            placeholder="Default from server config"
          />
          <p className="text-[10px] text-ciab-text-muted mt-1">
            Override the base directory where local sandboxes are created
          </p>
        </div>
      )}
    </div>
  );
}

// ---------------------------------------------------------------------------
// Repos Tab (with Local Mounts)
// ---------------------------------------------------------------------------

function ReposTab({
  spec,
  onSpecChange,
}: {
  spec: WorkspaceSpec;
  onSpecChange: (fn: (s: WorkspaceSpec) => WorkspaceSpec) => void;
}) {
  const [showAddRepo, setShowAddRepo] = useState(false);
  const [showAddMount, setShowAddMount] = useState(false);
  const [expandedRepo, setExpandedRepo] = useState<number | null>(null);
  const [expandedMount, setExpandedMount] = useState<number | null>(null);

  const repos = spec.repositories ?? [];
  const mounts = spec.local_mounts ?? [];

  const updateRepo = (idx: number, updater: (r: WorkspaceRepo) => WorkspaceRepo) => {
    onSpecChange((s) => {
      const updated = [...(s.repositories ?? [])];
      updated[idx] = updater(updated[idx]);
      return { ...s, repositories: updated };
    });
  };

  const removeRepo = (idx: number) => {
    onSpecChange((s) => ({
      ...s,
      repositories: (s.repositories ?? []).filter((_, i) => i !== idx),
    }));
  };

  const addRepo = (repo: WorkspaceRepo) => {
    onSpecChange((s) => ({
      ...s,
      repositories: [...(s.repositories ?? []), repo],
    }));
    setShowAddRepo(false);
  };

  const updateMount = (idx: number, updater: (m: LocalMount) => LocalMount) => {
    onSpecChange((s) => {
      const updated = [...(s.local_mounts ?? [])];
      updated[idx] = updater(updated[idx]);
      return { ...s, local_mounts: updated };
    });
  };

  const removeMount = (idx: number) => {
    onSpecChange((s) => ({
      ...s,
      local_mounts: (s.local_mounts ?? []).filter((_, i) => i !== idx),
    }));
  };

  const addMount = (mount: LocalMount) => {
    onSpecChange((s) => ({
      ...s,
      local_mounts: [...(s.local_mounts ?? []), mount],
    }));
    setShowAddMount(false);
  };

  return (
    <div className="space-y-6">
      {/* Git Repositories */}
      <div>
        <div className="flex items-center justify-between mb-3">
          <div className="flex items-center gap-2">
            <GitBranch className="w-4 h-4 text-ciab-copper" />
            <span className="text-sm font-medium text-ciab-text-primary">
              {repos.length} Repositor{repos.length === 1 ? "y" : "ies"}
            </span>
          </div>
          <button
            onClick={() => setShowAddRepo(!showAddRepo)}
            className={`flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-medium transition-all ${
              showAddRepo
                ? "bg-ciab-copper/15 text-ciab-copper border border-ciab-copper/20"
                : "text-ciab-text-muted hover:text-ciab-text-secondary hover:bg-ciab-bg-hover border border-transparent"
            }`}
          >
            {showAddRepo ? <X className="w-3.5 h-3.5" /> : <Plus className="w-3.5 h-3.5" />}
            {showAddRepo ? "Cancel" : "Add"}
          </button>
        </div>

        {showAddRepo && <AddRepoPanel onAdd={addRepo} onCancel={() => setShowAddRepo(false)} />}

        {repos.length === 0 && !showAddRepo ? (
          <div className="text-center py-12 border border-dashed border-ciab-border rounded-xl">
            <GitBranch className="w-8 h-8 text-ciab-text-muted/20 mx-auto mb-3" />
            <p className="text-sm text-ciab-text-secondary">No repositories configured</p>
          </div>
        ) : (
          <div className="space-y-2">
            {repos.map((repo, i) => (
              <div
                key={i}
                className="rounded-xl border transition-all border-ciab-border bg-ciab-bg-card"
              >
                <div className="flex items-center gap-3 p-4">
                  <div className="w-9 h-9 rounded-lg bg-ciab-bg-primary border border-ciab-border flex items-center justify-center flex-shrink-0">
                    <GitBranch className="w-4 h-4 text-ciab-copper" />
                  </div>
                  <div className="flex-1 min-w-0">
                    <div className="text-sm font-medium text-ciab-text-primary font-mono truncate">
                      {repo.url}
                    </div>
                    <div className="flex flex-wrap gap-2 mt-0.5 text-[10px] text-ciab-text-muted">
                      {repo.branch && <span>branch: {repo.branch}</span>}
                      {repo.tag && <span>tag: {repo.tag}</span>}
                      {repo.commit && <span>commit: {repo.commit.slice(0, 8)}</span>}
                      {repo.depth && <span>depth: {repo.depth}</span>}
                      {repo.submodules && (
                        <span className="text-ciab-copper">+submodules</span>
                      )}
                      {(repo.sparse_paths?.length ?? 0) > 0 && (
                        <span className="text-ciab-steel-blue">sparse</span>
                      )}
                      {repo.strategy === "worktree" && (
                        <span className="badge bg-state-paused/10 text-state-paused text-[10px]">worktree</span>
                      )}
                    </div>
                  </div>
                  <div className="flex items-center gap-1">
                    <button
                      onClick={() =>
                        setExpandedRepo(expandedRepo === i ? null : i)
                      }
                      className="p-1.5 rounded-lg text-ciab-text-muted hover:text-ciab-text-secondary hover:bg-ciab-bg-hover transition-colors"
                    >
                      {expandedRepo === i ? (
                        <ChevronDown className="w-3.5 h-3.5" />
                      ) : (
                        <ChevronRight className="w-3.5 h-3.5" />
                      )}
                    </button>
                    <button
                      onClick={() => removeRepo(i)}
                      className="p-1.5 rounded-lg text-ciab-text-muted hover:text-state-failed hover:bg-state-failed/10 transition-colors"
                    >
                      <Trash2 className="w-3.5 h-3.5" />
                    </button>
                  </div>
                </div>

                {expandedRepo === i && (
                  <div className="border-t border-ciab-border px-4 pb-4 pt-3 space-y-3 animate-fade-in bg-ciab-bg-primary/50">
                    <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
                      <div>
                        <label className="label">URL</label>
                        <input
                          type="text"
                          className="input w-full font-mono text-[11px]"
                          value={repo.url}
                          onChange={(e) =>
                            updateRepo(i, (r) => ({ ...r, url: e.target.value }))
                          }
                        />
                      </div>
                      <div>
                        <label className="label">Branch</label>
                        <input
                          type="text"
                          className="input w-full"
                          value={repo.branch ?? ""}
                          placeholder="main"
                          onChange={(e) =>
                            updateRepo(i, (r) => ({
                              ...r,
                              branch: e.target.value || undefined,
                            }))
                          }
                        />
                      </div>
                      <div>
                        <label className="label">Tag</label>
                        <input
                          type="text"
                          className="input w-full"
                          value={repo.tag ?? ""}
                          placeholder="v1.0.0"
                          onChange={(e) =>
                            updateRepo(i, (r) => ({
                              ...r,
                              tag: e.target.value || undefined,
                            }))
                          }
                        />
                      </div>
                      <div>
                        <label className="label">Commit</label>
                        <input
                          type="text"
                          className="input w-full font-mono text-[11px]"
                          value={repo.commit ?? ""}
                          placeholder="abc123..."
                          onChange={(e) =>
                            updateRepo(i, (r) => ({
                              ...r,
                              commit: e.target.value || undefined,
                            }))
                          }
                        />
                      </div>
                      <div>
                        <label className="label">Destination Path</label>
                        <input
                          type="text"
                          className="input w-full font-mono text-[11px]"
                          value={repo.dest_path ?? ""}
                          placeholder="/workspace/repo-name"
                          onChange={(e) =>
                            updateRepo(i, (r) => ({
                              ...r,
                              dest_path: e.target.value || undefined,
                            }))
                          }
                        />
                      </div>
                      <div>
                        <label className="label">Clone Depth</label>
                        <input
                          type="number"
                          className="input w-full"
                          value={repo.depth ?? ""}
                          placeholder="1"
                          onChange={(e) =>
                            updateRepo(i, (r) => ({
                              ...r,
                              depth: e.target.value
                                ? parseInt(e.target.value)
                                : undefined,
                            }))
                          }
                        />
                      </div>
                      <div>
                        <label className="label">Credential ID</label>
                        <input
                          type="text"
                          className="input w-full"
                          value={repo.credential_id ?? ""}
                          placeholder="github-token"
                          onChange={(e) =>
                            updateRepo(i, (r) => ({
                              ...r,
                              credential_id: e.target.value || undefined,
                            }))
                          }
                        />
                      </div>
                    </div>
                    <div className="flex items-center gap-4 flex-wrap">
                      <label className="flex items-center gap-2 text-xs text-ciab-text-secondary cursor-pointer">
                        <input
                          type="checkbox"
                          checked={repo.submodules ?? false}
                          onChange={(e) =>
                            updateRepo(i, (r) => ({
                              ...r,
                              submodules: e.target.checked,
                            }))
                          }
                          className="accent-ciab-copper"
                        />
                        Init submodules
                      </label>
                    </div>
                    <div>
                      <label className="label">Clone Strategy</label>
                      <div className="grid grid-cols-2 gap-2 mt-1">
                        {(["clone", "worktree"] as GitCloneStrategy[]).map(
                          (strategy) => (
                            <button
                              key={strategy}
                              onClick={() =>
                                updateRepo(i, (r) => ({
                                  ...r,
                                  strategy,
                                }))
                              }
                              className={`px-3 py-2 rounded-lg border text-xs font-medium transition-all text-left ${
                                (repo.strategy ?? "clone") === strategy
                                  ? "border-ciab-copper/50 bg-ciab-copper/5 text-ciab-copper"
                                  : "border-ciab-border text-ciab-text-secondary hover:border-ciab-border-light"
                              }`}
                            >
                              <div className="font-semibold">{strategy}</div>
                              <div className="text-[10px] text-ciab-text-muted mt-0.5">
                                {strategy === "clone"
                                  ? "Full git clone"
                                  : "Lightweight worktree from shared base"}
                              </div>
                            </button>
                          )
                        )}
                      </div>
                    </div>
                    <div>
                      <label className="label">Sparse Paths (one per line)</label>
                      <textarea
                        className="input w-full font-mono text-[11px] min-h-[4rem]"
                        value={(repo.sparse_paths ?? []).join("\n")}
                        placeholder="src/&#10;docs/"
                        onChange={(e) =>
                          updateRepo(i, (r) => ({
                            ...r,
                            sparse_paths: e.target.value
                              .split("\n")
                              .filter((p) => p.trim()),
                          }))
                        }
                      />
                    </div>
                  </div>
                )}
              </div>
            ))}
          </div>
        )}
      </div>

      {/* Local Mounts */}
      <div>
        <div className="flex items-center justify-between mb-3">
          <div className="flex items-center gap-2">
            <FolderOpen className="w-4 h-4 text-ciab-steel-blue" />
            <span className="text-sm font-medium text-ciab-text-primary">
              {mounts.length} Local Mount{mounts.length !== 1 ? "s" : ""}
            </span>
          </div>
          <button
            onClick={() => setShowAddMount(!showAddMount)}
            className={`flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-medium transition-all ${
              showAddMount
                ? "bg-ciab-steel-blue/15 text-ciab-steel-blue border border-ciab-steel-blue/20"
                : "text-ciab-text-muted hover:text-ciab-text-secondary hover:bg-ciab-bg-hover border border-transparent"
            }`}
          >
            {showAddMount ? (
              <X className="w-3.5 h-3.5" />
            ) : (
              <Plus className="w-3.5 h-3.5" />
            )}
            {showAddMount ? "Cancel" : "Add"}
          </button>
        </div>

        {showAddMount && (
          <AddLocalMountPanel
            onAdd={addMount}
            onCancel={() => setShowAddMount(false)}
          />
        )}

        {mounts.length === 0 && !showAddMount ? (
          <div className="text-center py-12 border border-dashed border-ciab-border rounded-xl">
            <FolderOpen className="w-8 h-8 text-ciab-text-muted/20 mx-auto mb-3" />
            <p className="text-sm text-ciab-text-secondary">
              No local directories mounted
            </p>
            <p className="text-xs text-ciab-text-muted mt-1">
              Mount host directories into sandboxes with copy, symlink, or bind
              modes
            </p>
          </div>
        ) : (
          <div className="space-y-2">
            {mounts.map((mount, i) => (
              <div
                key={i}
                className="rounded-xl border transition-all border-ciab-border bg-ciab-bg-card"
              >
                <div className="flex items-center gap-3 p-4">
                  <div className="w-9 h-9 rounded-lg bg-ciab-bg-primary border border-ciab-border flex items-center justify-center flex-shrink-0">
                    <FolderOpen className="w-4 h-4 text-ciab-steel-blue" />
                  </div>
                  <div className="flex-1 min-w-0">
                    <div className="text-sm font-medium text-ciab-text-primary font-mono truncate">
                      {mount.source}
                    </div>
                    <div className="flex flex-wrap gap-2 mt-0.5 text-[10px] text-ciab-text-muted">
                      <span
                        className={`badge text-[10px] ${
                          mount.sync_mode === "link"
                            ? "bg-state-paused/10 text-state-paused"
                            : mount.sync_mode === "bind"
                              ? "bg-state-creating/10 text-state-creating"
                              : "bg-state-running/10 text-state-running"
                        }`}
                      >
                        {mount.sync_mode ?? "copy"}
                      </span>
                      {mount.dest_path && <span>→ {mount.dest_path}</span>}
                      {mount.writeback && (
                        <span className="text-ciab-copper">writeback</span>
                      )}
                    </div>
                  </div>
                  <div className="flex items-center gap-1">
                    <button
                      onClick={() =>
                        setExpandedMount(expandedMount === i ? null : i)
                      }
                      className="p-1.5 rounded-lg text-ciab-text-muted hover:text-ciab-text-secondary hover:bg-ciab-bg-hover transition-colors"
                    >
                      {expandedMount === i ? (
                        <ChevronDown className="w-3.5 h-3.5" />
                      ) : (
                        <ChevronRight className="w-3.5 h-3.5" />
                      )}
                    </button>
                    <button
                      onClick={() => removeMount(i)}
                      className="p-1.5 rounded-lg text-ciab-text-muted hover:text-state-failed hover:bg-state-failed/10 transition-colors"
                    >
                      <Trash2 className="w-3.5 h-3.5" />
                    </button>
                  </div>
                </div>

                {expandedMount === i && (
                  <div className="border-t border-ciab-border px-4 pb-4 pt-3 space-y-3 animate-fade-in bg-ciab-bg-primary/50">
                    <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
                      <div>
                        <label className="label">Source Path</label>
                        <DirectoryPickerInput
                          value={mount.source}
                          onChange={(v) =>
                            updateMount(i, (m) => ({
                              ...m,
                              source: v,
                            }))
                          }
                          placeholder="/Users/me/projects/my-app"
                        />
                      </div>
                      <div>
                        <label className="label">Destination Path</label>
                        <input
                          type="text"
                          className="input w-full font-mono text-[11px]"
                          value={mount.dest_path ?? ""}
                          placeholder="/workspace/dir-name"
                          onChange={(e) =>
                            updateMount(i, (m) => ({
                              ...m,
                              dest_path: e.target.value || undefined,
                            }))
                          }
                        />
                      </div>
                    </div>
                    <div>
                      <label className="label">Sync Mode</label>
                      <div className="grid grid-cols-3 gap-2 mt-1">
                        {(["copy", "link", "bind"] as SyncMode[]).map(
                          (mode) => (
                            <button
                              key={mode}
                              onClick={() =>
                                updateMount(i, (m) => ({
                                  ...m,
                                  sync_mode: mode,
                                }))
                              }
                              className={`px-3 py-2 rounded-lg border text-xs font-medium transition-all text-left ${
                                (mount.sync_mode ?? "copy") === mode
                                  ? "border-ciab-copper/50 bg-ciab-copper/5 text-ciab-copper"
                                  : "border-ciab-border text-ciab-text-secondary hover:border-ciab-border-light"
                              }`}
                            >
                              <div className="font-semibold">{mode}</div>
                              <div className="text-[10px] text-ciab-text-muted mt-0.5">
                                {mode === "copy"
                                  ? "Isolated copy"
                                  : mode === "link"
                                    ? "Symlink (live)"
                                    : "Bind (Docker)"}
                              </div>
                            </button>
                          )
                        )}
                      </div>
                    </div>
                    <div className="flex items-center gap-4">
                      <label className="flex items-center gap-2 text-xs text-ciab-text-secondary cursor-pointer">
                        <input
                          type="checkbox"
                          checked={mount.writeback ?? false}
                          onChange={(e) =>
                            updateMount(i, (m) => ({
                              ...m,
                              writeback: e.target.checked,
                            }))
                          }
                          className="accent-ciab-copper"
                          disabled={(mount.sync_mode ?? "copy") !== "copy"}
                        />
                        Writeback on stop
                      </label>
                      <label className="flex items-center gap-2 text-xs text-ciab-text-secondary cursor-pointer">
                        <input
                          type="checkbox"
                          checked={mount.watch ?? false}
                          onChange={(e) =>
                            updateMount(i, (m) => ({
                              ...m,
                              watch: e.target.checked,
                            }))
                          }
                          className="accent-ciab-copper"
                        />
                        Watch for changes
                      </label>
                    </div>
                    <div>
                      <label className="label">
                        Exclude Patterns (one per line)
                      </label>
                      <textarea
                        className="input w-full font-mono text-[11px] min-h-[4rem]"
                        value={(mount.exclude_patterns ?? []).join("\n")}
                        placeholder="node_modules/**&#10;.git/**&#10;target/**"
                        onChange={(e) =>
                          updateMount(i, (m) => ({
                            ...m,
                            exclude_patterns: e.target.value
                              .split("\n")
                              .filter((p) => p.trim()),
                          }))
                        }
                      />
                    </div>
                  </div>
                )}
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}

// ---------------------------------------------------------------------------
// Directory Picker Input — text input with a Browse button
// ---------------------------------------------------------------------------

function DirectoryPickerInput({
  value,
  onChange,
  placeholder,
  className = "",
}: {
  value: string;
  onChange: (v: string) => void;
  placeholder?: string;
  className?: string;
}) {
  const { pickDirectory } = useDirectoryPicker();

  return (
    <div className="flex gap-1.5">
      <input
        type="text"
        className={`input flex-1 font-mono text-[11px] ${className}`}
        value={value}
        onChange={(e) => onChange(e.target.value)}
        placeholder={placeholder}
      />
      <button
        type="button"
        onClick={async () => {
          const dir = await pickDirectory();
          if (dir) onChange(dir);
        }}
        className="px-2 py-1 rounded-md border border-ciab-border text-ciab-text-muted hover:text-ciab-text-secondary hover:bg-ciab-bg-hover hover:border-ciab-border-light transition-colors flex-shrink-0"
        title="Browse..."
      >
        <FolderOpen className="w-3.5 h-3.5" />
      </button>
    </div>
  );
}

// ---------------------------------------------------------------------------
// Add Repo Panel — with GitHub CLI integration
// ---------------------------------------------------------------------------

function AddRepoPanel({
  onAdd,
  onCancel,
}: {
  onAdd: (repo: WorkspaceRepo) => void;
  onCancel: () => void;
}) {
  const [mode, setMode] = useState<"manual" | "github">("manual");
  const [url, setUrl] = useState("");
  const [branch, setBranch] = useState("");
  const [destPath, setDestPath] = useState("");

  // GitHub search state
  const {
    repos: ghRepos,
    loading: ghLoading,
    ghAvailable,
    error: ghError,
    checkAvailability,
    searchRepos,
  } = useGitHubRepos();
  const [ghQuery, setGhQuery] = useState("");
  const searchTimeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  // Check gh availability on first render
  useEffect(() => {
    checkAvailability();
  }, [checkAvailability]);

  const handleGhSearch = useCallback(
    (query: string) => {
      setGhQuery(query);
      if (searchTimeoutRef.current) clearTimeout(searchTimeoutRef.current);
      searchTimeoutRef.current = setTimeout(() => {
        searchRepos(query);
      }, 400);
    },
    [searchRepos]
  );

  const selectGhRepo = useCallback(
    (repo: GitHubRepo) => {
      setUrl(repo.url.endsWith(".git") ? repo.url : `${repo.url}.git`);
      setBranch(repo.defaultBranch);
      const repoName = repo.fullName.split("/").pop() || "repo";
      setDestPath(`/workspace/${repoName}`);
      setMode("manual"); // Switch to manual to let user review/edit
    },
    []
  );

  return (
    <div className="mb-3 rounded-xl border border-ciab-copper/20 bg-ciab-bg-card p-4 space-y-3 animate-fade-in">
      {/* Mode tabs */}
      {ghAvailable && (
        <div className="flex gap-1 p-0.5 bg-ciab-bg-primary rounded-lg border border-ciab-border w-fit">
          <button
            onClick={() => setMode("manual")}
            className={`px-3 py-1 rounded-md text-[11px] font-medium transition-all ${
              mode === "manual"
                ? "bg-ciab-bg-card text-ciab-text-primary shadow-sm"
                : "text-ciab-text-muted hover:text-ciab-text-secondary"
            }`}
          >
            Manual
          </button>
          <button
            onClick={() => {
              setMode("github");
              if (ghRepos.length === 0) searchRepos("");
            }}
            className={`px-3 py-1 rounded-md text-[11px] font-medium transition-all flex items-center gap-1.5 ${
              mode === "github"
                ? "bg-ciab-bg-card text-ciab-text-primary shadow-sm"
                : "text-ciab-text-muted hover:text-ciab-text-secondary"
            }`}
          >
            <Github className="w-3 h-3" />
            GitHub
          </button>
        </div>
      )}

      {mode === "github" ? (
        <div className="space-y-2">
          <div className="relative">
            <Search className="w-3.5 h-3.5 text-ciab-text-muted absolute left-3 top-1/2 -translate-y-1/2" />
            <input
              type="text"
              className="input w-full pl-9 text-[12px]"
              value={ghQuery}
              onChange={(e) => handleGhSearch(e.target.value)}
              placeholder="Search your GitHub repos..."
              autoFocus
            />
            {ghLoading && (
              <Loader2 className="w-3.5 h-3.5 text-ciab-copper absolute right-3 top-1/2 -translate-y-1/2 animate-spin" />
            )}
          </div>

          {ghError && (
            <p className="text-[11px] text-state-failed">{ghError}</p>
          )}

          <div className="max-h-48 overflow-y-auto border border-ciab-border rounded-lg divide-y divide-ciab-border">
            {ghRepos.length === 0 && !ghLoading && (
              <div className="px-3 py-6 text-center text-xs text-ciab-text-muted">
                {ghQuery ? "No repos found" : "Type to search or view your repos"}
              </div>
            )}
            {ghRepos.map((repo) => (
              <button
                key={repo.fullName}
                onClick={() => selectGhRepo(repo)}
                className="w-full text-left px-3 py-2.5 hover:bg-ciab-bg-hover/50 transition-colors"
              >
                <div className="flex items-center gap-2">
                  {repo.isPrivate ? (
                    <Lock className="w-3 h-3 text-state-paused flex-shrink-0" />
                  ) : (
                    <Globe className="w-3 h-3 text-ciab-text-muted flex-shrink-0" />
                  )}
                  <span className="text-[12px] font-medium text-ciab-text-primary font-mono truncate">
                    {repo.fullName}
                  </span>
                  <span className="text-[10px] text-ciab-text-muted ml-auto flex-shrink-0">
                    {repo.defaultBranch}
                  </span>
                </div>
                {repo.description && (
                  <p className="text-[10px] text-ciab-text-muted mt-0.5 truncate pl-5">
                    {repo.description}
                  </p>
                )}
              </button>
            ))}
          </div>
        </div>
      ) : (
        <>
          <div>
            <label className="label">Repository URL</label>
            <input
              type="text"
              className="input w-full font-mono text-[11px]"
              value={url}
              onChange={(e) => setUrl(e.target.value)}
              placeholder="https://github.com/org/repo.git"
              autoFocus
            />
          </div>
          <div className="grid grid-cols-2 gap-3">
            <div>
              <label className="label">Branch</label>
              <input
                type="text"
                className="input w-full"
                value={branch}
                onChange={(e) => setBranch(e.target.value)}
                placeholder="main"
              />
            </div>
            <div>
              <label className="label">Destination</label>
              <input
                type="text"
                className="input w-full font-mono text-[11px]"
                value={destPath}
                onChange={(e) => setDestPath(e.target.value)}
                placeholder="/workspace/repo"
              />
            </div>
          </div>
        </>
      )}

      <div className="flex items-center justify-end gap-2">
        <button onClick={onCancel} className="btn-ghost text-xs px-3 py-1.5">
          Cancel
        </button>
        <button
          onClick={() => {
            if (!url.trim()) return;
            onAdd({
              url: url.trim(),
              branch: branch || undefined,
              dest_path: destPath || undefined,
            });
          }}
          className="btn-primary text-xs px-3 py-1.5"
          disabled={!url.trim()}
        >
          Add Repository
        </button>
      </div>
    </div>
  );
}

// ---------------------------------------------------------------------------
// Add Local Mount Panel
// ---------------------------------------------------------------------------

function AddLocalMountPanel({
  onAdd,
  onCancel,
}: {
  onAdd: (mount: LocalMount) => void;
  onCancel: () => void;
}) {
  const [source, setSource] = useState("");
  const [destPath, setDestPath] = useState("");
  const [syncMode, setSyncMode] = useState<SyncMode>("copy");

  return (
    <div className="mb-3 rounded-xl border border-ciab-steel-blue/20 bg-ciab-bg-card p-4 space-y-3 animate-fade-in">
      <div>
        <label className="label">Source Directory</label>
        <DirectoryPickerInput
          value={source}
          onChange={(v) => {
            setSource(v);
            // Auto-fill dest_path from directory name
            if (!destPath && v) {
              const name = v.split("/").filter(Boolean).pop();
              if (name) setDestPath(`/workspace/${name}`);
            }
          }}
          placeholder="/Users/me/projects/my-app"
        />
      </div>
      <div className="grid grid-cols-2 gap-3">
        <div>
          <label className="label">Destination</label>
          <input
            type="text"
            className="input w-full font-mono text-[11px]"
            value={destPath}
            onChange={(e) => setDestPath(e.target.value)}
            placeholder="/workspace/my-app"
          />
        </div>
        <div>
          <label className="label">Sync Mode</label>
          <div className="grid grid-cols-3 gap-1.5 mt-1">
            {(["copy", "link", "bind"] as SyncMode[]).map((mode) => (
              <button
                key={mode}
                onClick={() => setSyncMode(mode)}
                className={`px-2 py-1.5 rounded-md border text-[11px] font-medium transition-all ${
                  syncMode === mode
                    ? "border-ciab-copper/50 bg-ciab-copper/5 text-ciab-copper"
                    : "border-ciab-border text-ciab-text-secondary hover:border-ciab-border-light"
                }`}
              >
                {mode}
              </button>
            ))}
          </div>
        </div>
      </div>
      <div className="flex items-center justify-end gap-2">
        <button onClick={onCancel} className="btn-ghost text-xs px-3 py-1.5">
          Cancel
        </button>
        <button
          onClick={() => {
            if (!source.trim()) return;
            onAdd({
              source: source.trim(),
              dest_path: destPath || undefined,
              sync_mode: syncMode,
            });
          }}
          className="btn-primary text-xs px-3 py-1.5"
          disabled={!source.trim()}
        >
          Add Mount
        </button>
      </div>
    </div>
  );
}

// ---------------------------------------------------------------------------
// Agent Tab
// ---------------------------------------------------------------------------

function AgentTab({
  spec,
  onSpecChange,
}: {
  spec: WorkspaceSpec;
  onSpecChange: (fn: (s: WorkspaceSpec) => WorkspaceSpec) => void;
}) {
  const agent = spec.agent;

  const updateAgent = (updater: (a: WorkspaceAgentConfig) => WorkspaceAgentConfig) => {
    onSpecChange((s) => ({
      ...s,
      agent: updater(
        s.agent ?? {
          provider: "claude-code",
          tools_enabled: true,
        }
      ),
    }));
  };

  if (!agent) {
    return (
      <div className="text-center py-12 border border-dashed border-ciab-border rounded-xl">
        <Bot className="w-8 h-8 text-ciab-text-muted/20 mx-auto mb-3" />
        <p className="text-sm text-ciab-text-secondary">No agent configured</p>
        <button
          onClick={() =>
            updateAgent((a) => ({ ...a, provider: "claude-code", tools_enabled: true }))
          }
          className="mt-3 btn-primary text-xs px-3 py-1.5"
        >
          Configure Agent
        </button>
      </div>
    );
  }

  const providers = [
    { value: "claude-code", label: "Claude Code" },
    { value: "codex", label: "Codex" },
    { value: "gemini", label: "Gemini CLI" },
    { value: "cursor", label: "Cursor" },
  ];

  return (
    <div className="space-y-4">
      <div className="card p-4 space-y-4">
        <div className="flex items-center justify-between">
          <h3 className="text-sm font-medium text-ciab-text-primary">Configuration</h3>
          <button
            onClick={() => onSpecChange((s) => ({ ...s, agent: undefined }))}
            className="text-[11px] text-ciab-text-muted hover:text-state-failed transition-colors"
          >
            Remove agent
          </button>
        </div>

        {/* Provider selection */}
        <div>
          <label className="label">Provider</label>
          <div className="grid grid-cols-2 sm:grid-cols-4 gap-1.5 mt-1">
            {providers.map((p) => (
              <button
                key={p.value}
                onClick={() => updateAgent((a) => ({ ...a, provider: p.value }))}
                className={`flex items-center gap-1.5 p-2 rounded-md border transition-all text-left ${
                  agent.provider === p.value
                    ? "border-ciab-copper/50 bg-ciab-copper/5"
                    : "border-ciab-border hover:border-ciab-border-light"
                }`}
              >
                <Bot className="w-3.5 h-3.5 text-ciab-text-muted" />
                <span className="text-[11px] font-medium truncate">
                  {p.label}
                </span>
              </button>
            ))}
          </div>
        </div>

        <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
          <div>
            <label className="label">Model</label>
            <input
              type="text"
              className="input w-full"
              value={agent.model ?? ""}
              placeholder="claude-sonnet-4-20250514"
              onChange={(e) =>
                updateAgent((a) => ({
                  ...a,
                  model: e.target.value || undefined,
                }))
              }
            />
          </div>
          <div>
            <label className="label">Temperature</label>
            <input
              type="number"
              className="input w-full"
              value={agent.temperature ?? ""}
              placeholder="0.0"
              step="0.1"
              min="0"
              max="2"
              onChange={(e) =>
                updateAgent((a) => ({
                  ...a,
                  temperature: e.target.value
                    ? parseFloat(e.target.value)
                    : undefined,
                }))
              }
            />
          </div>
          <div>
            <label className="label">Max Tokens</label>
            <input
              type="number"
              className="input w-full"
              value={agent.max_tokens ?? ""}
              placeholder="4096"
              onChange={(e) =>
                updateAgent((a) => ({
                  ...a,
                  max_tokens: e.target.value
                    ? parseInt(e.target.value)
                    : undefined,
                }))
              }
            />
          </div>
          <div className="flex items-end pb-2">
            <label className="flex items-center gap-2 text-xs text-ciab-text-secondary cursor-pointer">
              <input
                type="checkbox"
                checked={agent.tools_enabled !== false}
                onChange={(e) =>
                  updateAgent((a) => ({
                    ...a,
                    tools_enabled: e.target.checked,
                  }))
                }
                className="accent-ciab-copper"
              />
              Tools enabled
            </label>
          </div>
        </div>

        <div>
          <label className="label">System Prompt</label>
          <textarea
            className="input w-full min-h-[8rem]"
            value={agent.system_prompt ?? ""}
            placeholder="System prompt for the agent..."
            onChange={(e) =>
              updateAgent((a) => ({
                ...a,
                system_prompt: e.target.value || undefined,
              }))
            }
          />
        </div>

        <div>
          <label className="label">Allowed Tools (comma-separated, empty = all)</label>
          <input
            type="text"
            className="input w-full font-mono text-[11px]"
            value={(agent.allowed_tools ?? []).join(", ")}
            placeholder=""
            onChange={(e) =>
              updateAgent((a) => ({
                ...a,
                allowed_tools: e.target.value
                  .split(",")
                  .map((t) => t.trim())
                  .filter(Boolean),
              }))
            }
          />
        </div>

        <div>
          <label className="label">Denied Tools (comma-separated)</label>
          <input
            type="text"
            className="input w-full font-mono text-[11px]"
            value={(agent.denied_tools ?? []).join(", ")}
            placeholder=""
            onChange={(e) =>
              updateAgent((a) => ({
                ...a,
                denied_tools: e.target.value
                  .split(",")
                  .map((t) => t.trim())
                  .filter(Boolean),
              }))
            }
          />
        </div>
      </div>

      {/* MCP Servers */}
      <div className="card p-4 space-y-3">
        <div className="flex items-center justify-between">
          <h3 className="text-sm font-medium text-ciab-text-primary">
            MCP Servers
          </h3>
          <button
            onClick={() =>
              updateAgent((a) => ({
                ...a,
                mcp_servers: [
                  ...(a.mcp_servers ?? []),
                  { name: "", url: "" },
                ],
              }))
            }
            className="flex items-center gap-1 text-[11px] text-ciab-text-muted hover:text-ciab-copper transition-colors"
          >
            <Plus className="w-3 h-3" /> Add
          </button>
        </div>

        {(agent.mcp_servers ?? []).length === 0 ? (
          <p className="text-xs text-ciab-text-muted py-2">
            No MCP servers configured
          </p>
        ) : (
          <div className="space-y-2">
            {(agent.mcp_servers ?? []).map((server, i) => (
              <div key={i} className="flex items-center gap-1.5">
                <input
                  type="text"
                  className="input flex-1 text-[11px] py-1"
                  value={server.name}
                  placeholder="name"
                  onChange={(e) =>
                    updateAgent((a) => {
                      const servers = [...(a.mcp_servers ?? [])];
                      servers[i] = { ...servers[i], name: e.target.value };
                      return { ...a, mcp_servers: servers };
                    })
                  }
                />
                <input
                  type="text"
                  className="input flex-[2] font-mono text-[11px] py-1"
                  value={server.url}
                  placeholder="url or command"
                  onChange={(e) =>
                    updateAgent((a) => {
                      const servers = [...(a.mcp_servers ?? [])];
                      servers[i] = { ...servers[i], url: e.target.value };
                      return { ...a, mcp_servers: servers };
                    })
                  }
                />
                <button
                  onClick={() =>
                    updateAgent((a) => ({
                      ...a,
                      mcp_servers: (a.mcp_servers ?? []).filter(
                        (_, j) => j !== i
                      ),
                    }))
                  }
                  className="p-1 text-ciab-text-muted hover:text-state-failed transition-colors"
                >
                  <Trash2 className="w-3 h-3" />
                </button>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}

// ---------------------------------------------------------------------------
// Subagents Tab
// ---------------------------------------------------------------------------

function SubagentsTab({
  spec,
  onSpecChange,
}: {
  spec: WorkspaceSpec;
  onSpecChange: (fn: (s: WorkspaceSpec) => WorkspaceSpec) => void;
}) {
  const subagents = spec.subagents ?? [];
  const [expandedIdx, setExpandedIdx] = useState<number | null>(null);

  const updateSubagent = (
    idx: number,
    updater: (sa: SubagentConfig) => SubagentConfig
  ) => {
    onSpecChange((s) => {
      const updated = [...(s.subagents ?? [])];
      updated[idx] = updater(updated[idx]);
      return { ...s, subagents: updated };
    });
  };

  const removeSubagent = (idx: number) => {
    onSpecChange((s) => ({
      ...s,
      subagents: (s.subagents ?? []).filter((_, i) => i !== idx),
    }));
  };

  const addSubagent = () => {
    onSpecChange((s) => ({
      ...s,
      subagents: [
        ...(s.subagents ?? []),
        {
          name: `subagent-${(s.subagents?.length ?? 0) + 1}`,
          provider: "claude-code",
          activation: "on_demand" as const,
        },
      ],
    }));
  };

  return (
    <div className="space-y-3">
      <div className="flex items-center justify-between mb-1">
        <div className="flex items-center gap-2">
          <Users className="w-4 h-4 text-ciab-copper" />
          <span className="text-sm font-medium text-ciab-text-primary">
            {subagents.length} Subagent{subagents.length !== 1 ? "s" : ""}
          </span>
        </div>
        <button
          onClick={addSubagent}
          className="flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-medium text-ciab-text-muted hover:text-ciab-text-secondary hover:bg-ciab-bg-hover border border-transparent transition-all"
        >
          <Plus className="w-3.5 h-3.5" /> Add
        </button>
      </div>

      {subagents.length === 0 ? (
        <div className="text-center py-12 border border-dashed border-ciab-border rounded-xl">
          <Users className="w-8 h-8 text-ciab-text-muted/20 mx-auto mb-3" />
          <p className="text-sm text-ciab-text-secondary">
            No subagents configured
          </p>
        </div>
      ) : (
        subagents.map((sa, i) => (
          <div
            key={i}
            className="rounded-xl border transition-all border-ciab-border bg-ciab-bg-card"
          >
            <div className="flex items-center gap-3 p-4">
              <div className="w-9 h-9 rounded-lg bg-ciab-bg-primary border border-ciab-border flex items-center justify-center flex-shrink-0">
                <Users className="w-4 h-4 text-ciab-copper" />
              </div>
              <div className="flex-1 min-w-0">
                <div className="text-sm font-medium text-ciab-text-primary truncate">
                  {sa.name}
                </div>
                <div className="flex gap-2 mt-0.5 text-[10px] text-ciab-text-muted">
                  <span>{sa.provider}</span>
                  {sa.model && <span>· {sa.model}</span>}
                  <span className="badge bg-ciab-bg-hover text-ciab-text-secondary text-[10px]">
                    {typeof sa.activation === "string"
                      ? sa.activation
                      : "on_event"}
                  </span>
                </div>
              </div>
              <div className="flex items-center gap-1">
                <button
                  onClick={() =>
                    setExpandedIdx(expandedIdx === i ? null : i)
                  }
                  className="p-1.5 rounded-lg text-ciab-text-muted hover:text-ciab-text-secondary hover:bg-ciab-bg-hover transition-colors"
                >
                  {expandedIdx === i ? (
                    <ChevronDown className="w-3.5 h-3.5" />
                  ) : (
                    <ChevronRight className="w-3.5 h-3.5" />
                  )}
                </button>
                <button
                  onClick={() => removeSubagent(i)}
                  className="p-1.5 rounded-lg text-ciab-text-muted hover:text-state-failed hover:bg-state-failed/10 transition-colors"
                >
                  <Trash2 className="w-3.5 h-3.5" />
                </button>
              </div>
            </div>

            {expandedIdx === i && (
              <div className="border-t border-ciab-border px-4 pb-4 pt-3 space-y-3 animate-fade-in bg-ciab-bg-primary/50">
                <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
                  <div>
                    <label className="label">Name</label>
                    <input
                      type="text"
                      className="input w-full"
                      value={sa.name}
                      onChange={(e) =>
                        updateSubagent(i, (s) => ({
                          ...s,
                          name: e.target.value,
                        }))
                      }
                    />
                  </div>
                  <div>
                    <label className="label">Provider</label>
                    <input
                      type="text"
                      className="input w-full"
                      value={sa.provider}
                      onChange={(e) =>
                        updateSubagent(i, (s) => ({
                          ...s,
                          provider: e.target.value,
                        }))
                      }
                    />
                  </div>
                  <div>
                    <label className="label">Model</label>
                    <input
                      type="text"
                      className="input w-full"
                      value={sa.model ?? ""}
                      placeholder="claude-sonnet-4-20250514"
                      onChange={(e) =>
                        updateSubagent(i, (s) => ({
                          ...s,
                          model: e.target.value || undefined,
                        }))
                      }
                    />
                  </div>
                  <div>
                    <label className="label">Activation</label>
                    <select
                      className="input w-full"
                      value={
                        typeof sa.activation === "string"
                          ? sa.activation
                          : "on_event"
                      }
                      onChange={(e) =>
                        updateSubagent(i, (s) => ({
                          ...s,
                          activation: e.target.value as
                            | "always"
                            | "on_demand",
                        }))
                      }
                    >
                      <option value="on_demand">On Demand</option>
                      <option value="always">Always</option>
                    </select>
                  </div>
                </div>
                <div>
                  <label className="label">System Prompt</label>
                  <textarea
                    className="input w-full min-h-[4rem]"
                    value={sa.system_prompt ?? ""}
                    placeholder="System prompt for this subagent..."
                    onChange={(e) =>
                      updateSubagent(i, (s) => ({
                        ...s,
                        system_prompt: e.target.value || undefined,
                      }))
                    }
                  />
                </div>
              </div>
            )}
          </div>
        ))
      )}
    </div>
  );
}

// ---------------------------------------------------------------------------
// Commands Tab
// ---------------------------------------------------------------------------

function CommandsTab({
  spec,
  onSpecChange,
}: {
  spec: WorkspaceSpec;
  onSpecChange: (fn: (s: WorkspaceSpec) => WorkspaceSpec) => void;
}) {
  const commands = spec.pre_commands ?? [];
  const binaries = spec.binaries ?? [];
  const [expandedCmd, setExpandedCmd] = useState<number | null>(null);

  const updateCommand = (idx: number, updater: (c: PreCommand) => PreCommand) => {
    onSpecChange((s) => {
      const updated = [...(s.pre_commands ?? [])];
      updated[idx] = updater(updated[idx]);
      return { ...s, pre_commands: updated };
    });
  };

  const removeCommand = (idx: number) => {
    onSpecChange((s) => ({
      ...s,
      pre_commands: (s.pre_commands ?? []).filter((_, i) => i !== idx),
    }));
  };

  const addCommand = () => {
    onSpecChange((s) => ({
      ...s,
      pre_commands: [
        ...(s.pre_commands ?? []),
        { command: "", fail_on_error: true },
      ],
    }));
  };

  const removeBinary = (idx: number) => {
    onSpecChange((s) => ({
      ...s,
      binaries: (s.binaries ?? []).filter((_, i) => i !== idx),
    }));
  };

  const updateBinary = (idx: number, updater: (b: BinaryInstall) => BinaryInstall) => {
    onSpecChange((s) => {
      const updated = [...(s.binaries ?? [])];
      updated[idx] = updater(updated[idx]);
      return { ...s, binaries: updated };
    });
  };

  const addBinary = () => {
    onSpecChange((s) => ({
      ...s,
      binaries: [...(s.binaries ?? []), { name: "" }],
    }));
  };

  return (
    <div className="space-y-6">
      {/* Pre-commands */}
      <div>
        <div className="flex items-center justify-between mb-3">
          <div className="flex items-center gap-2">
            <Terminal className="w-4 h-4 text-ciab-copper" />
            <span className="text-sm font-medium text-ciab-text-primary">
              {commands.length} Pre-command{commands.length !== 1 ? "s" : ""}
            </span>
          </div>
          <button
            onClick={addCommand}
            className="flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-medium text-ciab-text-muted hover:text-ciab-text-secondary hover:bg-ciab-bg-hover border border-transparent transition-all"
          >
            <Plus className="w-3.5 h-3.5" /> Add
          </button>
        </div>

        {commands.length === 0 ? (
          <div className="text-center py-8 border border-dashed border-ciab-border rounded-xl">
            <Terminal className="w-8 h-8 text-ciab-text-muted/20 mx-auto mb-3" />
            <p className="text-sm text-ciab-text-secondary">
              No pre-commands configured
            </p>
          </div>
        ) : (
          <div className="space-y-2">
            {commands.map((cmd, i) => (
              <div
                key={i}
                className="rounded-xl border transition-all border-ciab-border bg-ciab-bg-card"
              >
                <div className="flex items-center gap-3 p-4">
                  <div className="w-9 h-9 rounded-lg bg-ciab-bg-primary border border-ciab-border flex items-center justify-center flex-shrink-0">
                    <Terminal className="w-4 h-4 text-ciab-copper" />
                  </div>
                  <div className="flex-1 min-w-0">
                    {cmd.name && (
                      <p className="text-xs text-ciab-text-muted">{cmd.name}</p>
                    )}
                    <code className="text-sm text-ciab-copper font-mono truncate block">
                      {cmd.command} {cmd.args?.join(" ") ?? ""}
                    </code>
                    {cmd.workdir && (
                      <p className="text-[10px] text-ciab-text-muted mt-0.5">
                        in {cmd.workdir}
                      </p>
                    )}
                  </div>
                  <div className="flex items-center gap-1">
                    <button
                      onClick={() =>
                        setExpandedCmd(expandedCmd === i ? null : i)
                      }
                      className="p-1.5 rounded-lg text-ciab-text-muted hover:text-ciab-text-secondary hover:bg-ciab-bg-hover transition-colors"
                    >
                      {expandedCmd === i ? (
                        <ChevronDown className="w-3.5 h-3.5" />
                      ) : (
                        <ChevronRight className="w-3.5 h-3.5" />
                      )}
                    </button>
                    <button
                      onClick={() => removeCommand(i)}
                      className="p-1.5 rounded-lg text-ciab-text-muted hover:text-state-failed hover:bg-state-failed/10 transition-colors"
                    >
                      <Trash2 className="w-3.5 h-3.5" />
                    </button>
                  </div>
                </div>

                {expandedCmd === i && (
                  <div className="border-t border-ciab-border px-4 pb-4 pt-3 space-y-3 animate-fade-in bg-ciab-bg-primary/50">
                    <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
                      <div>
                        <label className="label">Name</label>
                        <input
                          type="text"
                          className="input w-full"
                          value={cmd.name ?? ""}
                          placeholder="Step name"
                          onChange={(e) =>
                            updateCommand(i, (c) => ({
                              ...c,
                              name: e.target.value || undefined,
                            }))
                          }
                        />
                      </div>
                      <div>
                        <label className="label">Command</label>
                        <input
                          type="text"
                          className="input w-full font-mono text-[11px]"
                          value={cmd.command}
                          onChange={(e) =>
                            updateCommand(i, (c) => ({
                              ...c,
                              command: e.target.value,
                            }))
                          }
                        />
                      </div>
                      <div>
                        <label className="label">
                          Args (comma-separated)
                        </label>
                        <input
                          type="text"
                          className="input w-full font-mono text-[11px]"
                          value={(cmd.args ?? []).join(", ")}
                          placeholder="install, --save-dev"
                          onChange={(e) =>
                            updateCommand(i, (c) => ({
                              ...c,
                              args: e.target.value
                                .split(",")
                                .map((a) => a.trim())
                                .filter(Boolean),
                            }))
                          }
                        />
                      </div>
                      <div>
                        <label className="label">Working Directory</label>
                        <input
                          type="text"
                          className="input w-full font-mono text-[11px]"
                          value={cmd.workdir ?? ""}
                          placeholder="/workspace"
                          onChange={(e) =>
                            updateCommand(i, (c) => ({
                              ...c,
                              workdir: e.target.value || undefined,
                            }))
                          }
                        />
                      </div>
                      <div>
                        <label className="label">Timeout (seconds)</label>
                        <input
                          type="number"
                          className="input w-full"
                          value={cmd.timeout_secs ?? ""}
                          placeholder="120"
                          onChange={(e) =>
                            updateCommand(i, (c) => ({
                              ...c,
                              timeout_secs: e.target.value
                                ? parseInt(e.target.value)
                                : undefined,
                            }))
                          }
                        />
                      </div>
                      <div className="flex items-end pb-2">
                        <label className="flex items-center gap-2 text-xs text-ciab-text-secondary cursor-pointer">
                          <input
                            type="checkbox"
                            checked={cmd.fail_on_error !== false}
                            onChange={(e) =>
                              updateCommand(i, (c) => ({
                                ...c,
                                fail_on_error: e.target.checked,
                              }))
                            }
                            className="accent-ciab-copper"
                          />
                          Fail on error
                        </label>
                      </div>
                    </div>
                  </div>
                )}
              </div>
            ))}
          </div>
        )}
      </div>

      {/* Binaries */}
      <div>
        <div className="flex items-center justify-between mb-3">
          <div className="flex items-center gap-2">
            <Package className="w-4 h-4 text-ciab-steel-blue" />
            <span className="text-sm font-medium text-ciab-text-primary">
              {binaries.length} Binar{binaries.length === 1 ? "y" : "ies"}
            </span>
          </div>
          <button
            onClick={addBinary}
            className="flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-medium text-ciab-text-muted hover:text-ciab-text-secondary hover:bg-ciab-bg-hover border border-transparent transition-all"
          >
            <Plus className="w-3.5 h-3.5" /> Add
          </button>
        </div>

        {binaries.length === 0 ? (
          <p className="text-ciab-text-muted text-sm">No additional binaries</p>
        ) : (
          <div className="space-y-1.5">
            {binaries.map((bin, i) => (
              <div key={i} className="flex items-center gap-1.5">
                <input
                  type="text"
                  className="input flex-1 font-mono text-[11px] py-1"
                  value={bin.name}
                  placeholder="package-name"
                  onChange={(e) =>
                    updateBinary(i, (b) => ({ ...b, name: e.target.value }))
                  }
                />
                <select
                  className="input text-[11px] py-1 w-20"
                  value={bin.method ?? "apt"}
                  onChange={(e) =>
                    updateBinary(i, (b) => ({
                      ...b,
                      method: e.target.value as BinaryInstall["method"],
                    }))
                  }
                >
                  <option value="apt">apt</option>
                  <option value="cargo">cargo</option>
                  <option value="npm">npm</option>
                  <option value="pip">pip</option>
                  <option value="custom">custom</option>
                </select>
                <input
                  type="text"
                  className="input w-24 font-mono text-[11px] py-1"
                  value={bin.version ?? ""}
                  placeholder="version"
                  onChange={(e) =>
                    updateBinary(i, (b) => ({
                      ...b,
                      version: e.target.value || undefined,
                    }))
                  }
                />
                <button
                  onClick={() => removeBinary(i)}
                  className="p-1 text-ciab-text-muted hover:text-state-failed transition-colors"
                >
                  <Trash2 className="w-3 h-3" />
                </button>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}

// ---------------------------------------------------------------------------
// Filesystem Tab
// ---------------------------------------------------------------------------

function FilesystemTab({
  spec,
  onSpecChange,
}: {
  spec: WorkspaceSpec;
  onSpecChange: (fn: (s: WorkspaceSpec) => WorkspaceSpec) => void;
}) {
  const fs = spec.filesystem ?? {};

  const updateFs = (updater: (f: FilesystemConfig) => FilesystemConfig) => {
    onSpecChange((s) => ({
      ...s,
      filesystem: updater(s.filesystem ?? {}),
    }));
  };

  return (
    <div className="card p-4 space-y-4">
      <h3 className="text-sm font-medium text-ciab-text-primary">
        Filesystem Settings
      </h3>

      <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
        <div>
          <label className="label">Working Directory</label>
          <DirectoryPickerInput
            value={fs.workdir ?? "/workspace"}
            onChange={(v) => updateFs((f) => ({ ...f, workdir: v }))}
            placeholder="/workspace"
          />
        </div>
        <div>
          <label className="label">Max File Size (bytes)</label>
          <input
            type="number"
            className="input w-full"
            value={fs.max_file_size_bytes ?? ""}
            placeholder="10485760"
            onChange={(e) =>
              updateFs((f) => ({
                ...f,
                max_file_size_bytes: e.target.value
                  ? parseInt(e.target.value)
                  : undefined,
              }))
            }
          />
          {fs.max_file_size_bytes && (
            <p className="text-[10px] text-ciab-text-muted mt-0.5">
              {(fs.max_file_size_bytes / 1024 / 1024).toFixed(1)} MB
            </p>
          )}
        </div>
        <div>
          <label className="label">Temp Dir Size (MB)</label>
          <input
            type="number"
            className="input w-full"
            value={fs.tmp_size_mb ?? ""}
            placeholder="1024"
            onChange={(e) =>
              updateFs((f) => ({
                ...f,
                tmp_size_mb: e.target.value
                  ? parseInt(e.target.value)
                  : undefined,
              }))
            }
          />
        </div>
      </div>

      <div className="flex flex-wrap gap-4">
        <label className="flex items-center gap-2 text-xs text-ciab-text-secondary cursor-pointer">
          <input
            type="checkbox"
            checked={fs.cow_isolation ?? false}
            onChange={(e) =>
              updateFs((f) => ({ ...f, cow_isolation: e.target.checked }))
            }
            className="accent-ciab-copper"
          />
          Copy-on-Write Isolation
        </label>
        <label className="flex items-center gap-2 text-xs text-ciab-text-secondary cursor-pointer">
          <input
            type="checkbox"
            checked={fs.persist_changes ?? false}
            onChange={(e) =>
              updateFs((f) => ({ ...f, persist_changes: e.target.checked }))
            }
            className="accent-ciab-copper"
          />
          Persist Changes
        </label>
      </div>

      {/* AgentFS Section */}
      <div className="border-t border-ciab-border pt-4 mt-2">
        <div className="flex items-center justify-between mb-3">
          <div className="flex items-center gap-2">
            <HardDrive className="w-4 h-4 text-ciab-copper" />
            <span className="text-sm font-medium text-ciab-text-primary">AgentFS (CoW Isolation)</span>
            <span className="badge bg-state-paused/10 text-state-paused text-[9px] font-mono">alpha</span>
          </div>
          <label className="flex items-center gap-2 text-xs text-ciab-text-secondary cursor-pointer">
            <input
              type="checkbox"
              checked={fs.agentfs?.enabled ?? false}
              onChange={(e) =>
                updateFs((f) => ({
                  ...f,
                  agentfs: {
                    ...f.agentfs,
                    enabled: e.target.checked,
                  },
                }))
              }
              className="accent-ciab-copper"
            />
            Enable
          </label>
        </div>

        {(fs.agentfs?.enabled) && (
          <div className="space-y-3 animate-fade-in">
            <div className="flex items-center gap-2 p-2 rounded-lg bg-state-paused/5 border border-state-paused/20 text-[11px] text-state-paused">
              <AlertTriangle className="w-3.5 h-3.5 flex-shrink-0" />
              AgentFS provides copy-on-write filesystem isolation. Requires the agentfs binary in the sandbox.
            </div>
            <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
              <div>
                <label className="label">Binary Path</label>
                <input
                  type="text"
                  className="input w-full font-mono text-[11px]"
                  value={fs.agentfs?.binary ?? "agentfs"}
                  placeholder="agentfs"
                  onChange={(e) =>
                    updateFs((f) => ({
                      ...f,
                      agentfs: {
                        ...f.agentfs,
                        enabled: true,
                        binary: e.target.value || undefined,
                      },
                    }))
                  }
                />
              </div>
              <div>
                <label className="label">Database Path</label>
                <input
                  type="text"
                  className="input w-full font-mono text-[11px]"
                  value={fs.agentfs?.db_path ?? ""}
                  placeholder="auto-generated"
                  onChange={(e) =>
                    updateFs((f) => ({
                      ...f,
                      agentfs: {
                        ...f.agentfs,
                        enabled: true,
                        db_path: e.target.value || undefined,
                      },
                    }))
                  }
                />
              </div>
            </div>
            <label className="flex items-center gap-2 text-xs text-ciab-text-secondary cursor-pointer">
              <input
                type="checkbox"
                checked={fs.agentfs?.operation_logging !== false}
                onChange={(e) =>
                  updateFs((f) => ({
                    ...f,
                    agentfs: {
                      ...f.agentfs,
                      enabled: true,
                      operation_logging: e.target.checked,
                    },
                  }))
                }
                className="accent-ciab-copper"
              />
              Log filesystem operations
            </label>
          </div>
        )}
      </div>

      <div>
        <label className="label">Exclude Patterns (one per line)</label>
        <textarea
          className="input w-full font-mono text-[11px] min-h-[4rem]"
          value={(fs.exclude_patterns ?? []).join("\n")}
          placeholder="**/node_modules/**&#10;**/target/**&#10;**/.git/**"
          onChange={(e) =>
            updateFs((f) => ({
              ...f,
              exclude_patterns: e.target.value
                .split("\n")
                .filter((p) => p.trim()),
            }))
          }
        />
      </div>

      <div>
        <label className="label">Read-only Paths (one per line)</label>
        <textarea
          className="input w-full font-mono text-[11px] min-h-[3rem]"
          value={(fs.readonly_paths ?? []).join("\n")}
          placeholder="/etc&#10;/usr"
          onChange={(e) =>
            updateFs((f) => ({
              ...f,
              readonly_paths: e.target.value
                .split("\n")
                .filter((p) => p.trim()),
            }))
          }
        />
      </div>

      <div>
        <label className="label">Writable Paths (one per line)</label>
        <textarea
          className="input w-full font-mono text-[11px] min-h-[3rem]"
          value={(fs.writable_paths ?? []).join("\n")}
          placeholder="/workspace&#10;/tmp"
          onChange={(e) =>
            updateFs((f) => ({
              ...f,
              writable_paths: e.target.value
                .split("\n")
                .filter((p) => p.trim()),
            }))
          }
        />
      </div>
    </div>
  );
}

// ---------------------------------------------------------------------------
// Credentials Tab
// ---------------------------------------------------------------------------

function CredentialsTab({
  spec,
  onSpecChange,
}: {
  spec: WorkspaceSpec;
  onSpecChange: (fn: (s: WorkspaceSpec) => WorkspaceSpec) => void;
}) {
  const creds = spec.credentials ?? [];
  const [expandedIdx, setExpandedIdx] = useState<number | null>(null);

  const updateCred = (
    idx: number,
    updater: (c: WorkspaceCredential) => WorkspaceCredential
  ) => {
    onSpecChange((s) => {
      const updated = [...(s.credentials ?? [])];
      updated[idx] = updater(updated[idx]);
      return { ...s, credentials: updated };
    });
  };

  const removeCred = (idx: number) => {
    onSpecChange((s) => ({
      ...s,
      credentials: (s.credentials ?? []).filter((_, i) => i !== idx),
    }));
  };

  const addCred = () => {
    onSpecChange((s) => ({
      ...s,
      credentials: [
        ...(s.credentials ?? []),
        { name: "", vault_provider: "local" },
      ],
    }));
  };

  return (
    <div className="space-y-3">
      <div className="flex items-center justify-between mb-1">
        <div className="flex items-center gap-2">
          <KeyRound className="w-4 h-4 text-ciab-copper" />
          <span className="text-sm font-medium text-ciab-text-primary">
            {creds.length} Credential{creds.length !== 1 ? "s" : ""}
          </span>
        </div>
        <button
          onClick={addCred}
          className="flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-medium text-ciab-text-muted hover:text-ciab-text-secondary hover:bg-ciab-bg-hover border border-transparent transition-all"
        >
          <Plus className="w-3.5 h-3.5" /> Add
        </button>
      </div>

      {creds.length === 0 ? (
        <div className="text-center py-12 border border-dashed border-ciab-border rounded-xl">
          <KeyRound className="w-8 h-8 text-ciab-text-muted/20 mx-auto mb-3" />
          <p className="text-sm text-ciab-text-secondary">
            No credentials configured
          </p>
        </div>
      ) : (
        creds.map((cred, i) => (
          <div
            key={i}
            className="rounded-xl border transition-all border-ciab-border bg-ciab-bg-card"
          >
            <div className="flex items-center gap-3 p-4">
              <div className="w-9 h-9 rounded-lg bg-ciab-bg-primary border border-ciab-border flex items-center justify-center flex-shrink-0">
                <KeyRound className="w-4 h-4 text-ciab-copper" />
              </div>
              <div className="flex-1 min-w-0">
                <div className="text-sm font-medium text-ciab-text-primary truncate">
                  {cred.name ?? cred.id ?? "unnamed"}
                </div>
                <div className="flex gap-2 mt-0.5 text-[10px] text-ciab-text-muted">
                  <span>vault: {cred.vault_provider ?? "local"}</span>
                  {cred.env_var && <span>env: {cred.env_var}</span>}
                  {cred.file_path && <span>file: {cred.file_path}</span>}
                </div>
              </div>
              <div className="flex items-center gap-1">
                <button
                  onClick={() =>
                    setExpandedIdx(expandedIdx === i ? null : i)
                  }
                  className="p-1.5 rounded-lg text-ciab-text-muted hover:text-ciab-text-secondary hover:bg-ciab-bg-hover transition-colors"
                >
                  {expandedIdx === i ? (
                    <ChevronDown className="w-3.5 h-3.5" />
                  ) : (
                    <ChevronRight className="w-3.5 h-3.5" />
                  )}
                </button>
                <button
                  onClick={() => removeCred(i)}
                  className="p-1.5 rounded-lg text-ciab-text-muted hover:text-state-failed hover:bg-state-failed/10 transition-colors"
                >
                  <Trash2 className="w-3.5 h-3.5" />
                </button>
              </div>
            </div>

            {expandedIdx === i && (
              <div className="border-t border-ciab-border px-4 pb-4 pt-3 space-y-3 animate-fade-in bg-ciab-bg-primary/50">
                <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
                  <div>
                    <label className="label">Name</label>
                    <input
                      type="text"
                      className="input w-full"
                      value={cred.name ?? ""}
                      placeholder="credential-name"
                      onChange={(e) =>
                        updateCred(i, (c) => ({
                          ...c,
                          name: e.target.value || undefined,
                        }))
                      }
                    />
                  </div>
                  <div>
                    <label className="label">Vault Provider</label>
                    <select
                      className="input w-full"
                      value={cred.vault_provider ?? "local"}
                      onChange={(e) =>
                        updateCred(i, (c) => ({
                          ...c,
                          vault_provider: e.target.value,
                        }))
                      }
                    >
                      <option value="local">Local</option>
                      <option value="aws-secrets-manager">
                        AWS Secrets Manager
                      </option>
                      <option value="hashicorp-vault">HashiCorp Vault</option>
                      <option value="1password">1Password</option>
                    </select>
                  </div>
                  <div>
                    <label className="label">Environment Variable</label>
                    <input
                      type="text"
                      className="input w-full font-mono text-[11px]"
                      value={cred.env_var ?? ""}
                      placeholder="ANTHROPIC_API_KEY"
                      onChange={(e) =>
                        updateCred(i, (c) => ({
                          ...c,
                          env_var: e.target.value || undefined,
                        }))
                      }
                    />
                  </div>
                  <div>
                    <label className="label">Vault Path</label>
                    <input
                      type="text"
                      className="input w-full font-mono text-[11px]"
                      value={cred.vault_path ?? ""}
                      placeholder="prod/ciab/secret"
                      onChange={(e) =>
                        updateCred(i, (c) => ({
                          ...c,
                          vault_path: e.target.value || undefined,
                        }))
                      }
                    />
                  </div>
                  <div>
                    <label className="label">File Path</label>
                    <input
                      type="text"
                      className="input w-full font-mono text-[11px]"
                      value={cred.file_path ?? ""}
                      placeholder="/path/to/secret"
                      onChange={(e) =>
                        updateCred(i, (c) => ({
                          ...c,
                          file_path: e.target.value || undefined,
                        }))
                      }
                    />
                  </div>
                </div>
              </div>
            )}
          </div>
        ))
      )}
    </div>
  );
}

// ---------------------------------------------------------------------------
// Shared
// ---------------------------------------------------------------------------

function StatCard({ label, value }: { label: string; value: number }) {
  return (
    <div className="card p-4 text-center">
      <p className="text-2xl font-semibold text-ciab-text-primary">{value}</p>
      <p className="text-xs text-ciab-text-muted mt-1">{label}</p>
    </div>
  );
}
