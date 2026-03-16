import { useState } from "react";
import { Link, useNavigate } from "react-router";
import {
  Plus,
  Layers,
  GitBranch,
  Zap,
  Trash2,
  Cloud,
  Sparkles,
  Terminal,
  Bot,
  Package,
  ArrowRight,
  Search,
  X,
  Globe,
  Server,
  Box,
  BarChart3,
  FileCode,
  Layout,
  Smartphone,
  BrainCircuit,
  Chrome,
  Blocks,
  RefreshCw,
  ArrowLeft,
  Link2,
  FileCode2,
  Loader2,
  ChevronDown,
  ChevronRight,
  Monitor,
  Settings2,
  Rocket,
  ExternalLink,
} from "lucide-react";
import {
  useWorkspaces,
  useCreateWorkspace,
  useDeleteWorkspace,
  useLaunchWorkspace,
} from "@/lib/hooks/use-workspaces";
import { useTemplates, useCreateFromTemplate } from "@/lib/hooks/use-templates";
import { formatRelativeTime } from "@/lib/utils/format";
import LoadingSpinner from "@/components/shared/LoadingSpinner";
import EmptyState from "@/components/shared/EmptyState";
import AgentProviderIcon from "@/components/shared/AgentProviderIcon";
import TemplateSyncDialog from "@/features/workspace/TemplateSyncDialog";
import type { Workspace, WorkspaceSpec, WorkspaceRepo, PreCommand, BinaryInstall, RuntimeBackend } from "@/lib/api/types";
import {
  STARTER_TEMPLATES,
  TEMPLATE_CATEGORIES,
  type StarterTemplate,
} from "@/lib/data/starter-templates";
import { workspaces as workspacesApi } from "@/lib/api/endpoints";
import { useQueryClient } from "@tanstack/react-query";
import { toast } from "sonner";

// Icon resolver for starter templates
const ICON_MAP: Record<string, typeof Globe> = {
  Globe, Layers, Server, Terminal, BarChart3, Box, FileCode, Layout,
  Smartphone, BrainCircuit, Chrome, Blocks,
};

const PROVIDER_LABELS: Record<string, string> = {
  "claude-code": "Claude Code",
  codex: "Codex",
  gemini: "Gemini CLI",
  cursor: "Cursor",
};

const PROVIDERS = [
  { value: "claude-code", label: "Claude Code", org: "Anthropic" },
  { value: "codex", label: "Codex", org: "OpenAI" },
  { value: "gemini", label: "Gemini CLI", org: "Google" },
  { value: "cursor", label: "Cursor", org: "Anysphere" },
];

type View = "workspaces" | "new";

export default function WorkspaceList() {
  const { data: allWorkspaces, isLoading: wsLoading, isFetching } = useWorkspaces();
  const { data: templateList, isLoading: tmplLoading } = useTemplates();
  const deleteWorkspace = useDeleteWorkspace();
  const launchWorkspace = useLaunchWorkspace();
  const navigate = useNavigate();
  const qc = useQueryClient();

  const [view, setView] = useState<View>("workspaces");
  const [searchQuery, setSearchQuery] = useState("");
  const [spinning, setSpinning] = useState(false);
  const [showSync, setShowSync] = useState(false);

  // New workspace flow state
  const [configTarget, setConfigTarget] = useState<{
    type: "starter" | "synced" | "url" | "blank";
    starter?: StarterTemplate;
    synced?: Workspace;
    spec?: WorkspaceSpec;
    name?: string;
    description?: string;
  } | null>(null);

  const isLoading = wsLoading || tmplLoading;

  const myWorkspaces = allWorkspaces?.filter(
    (ws) => ws.labels?.["ciab/kind"] !== "template"
  );

  const filteredWorkspaces = searchQuery
    ? myWorkspaces?.filter(
        (ws) =>
          ws.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
          ws.description?.toLowerCase().includes(searchQuery.toLowerCase())
      )
    : myWorkspaces;

  const handleRefresh = () => {
    setSpinning(true);
    qc.invalidateQueries({ queryKey: ["workspaces"] });
    qc.invalidateQueries({ queryKey: ["templates"] });
    setTimeout(() => setSpinning(false), 600);
  };

  if (isLoading) {
    return (
      <div className="flex items-center justify-center h-64">
        <LoadingSpinner />
      </div>
    );
  }

  // If configuring a template (step 2 of "New" flow)
  if (configTarget) {
    return (
      <ConfigureWorkspace
        target={configTarget}
        onBack={() => setConfigTarget(null)}
        onCreated={(ws) => {
          setConfigTarget(null);
          setView("workspaces");
          if (ws?.id) navigate(`/workspaces/${ws.id}`);
        }}
      />
    );
  }

  // "New Workspace" view - template browser
  if (view === "new") {
    return (
      <NewWorkspaceView
        templateList={templateList ?? []}
        onBack={() => setView("workspaces")}
        onSelectStarter={(s) =>
          setConfigTarget({
            type: "starter",
            starter: s,
            spec: s.spec,
            name: "",
            description: s.description,
          })
        }
        onSelectSynced={(ws) =>
          setConfigTarget({
            type: "synced",
            synced: ws,
            spec: ws.spec,
            name: "",
            description: ws.description ?? "",
          })
        }
        onSelectBlank={() =>
          setConfigTarget({
            type: "blank",
            spec: { agent: { provider: "claude-code" } },
            name: "",
            description: "",
          })
        }
        onImportUrl={(spec, name) =>
          setConfigTarget({
            type: "url",
            spec,
            name,
            description: "",
          })
        }
        onManageSources={() => setShowSync(true)}
      />
    );
  }

  // Main view: My Workspaces
  return (
    <div className="space-y-5 animate-fade-in">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-3">
          <div className="w-10 h-10 rounded-xl bg-gradient-to-br from-ciab-copper/20 to-ciab-copper/5 border border-ciab-copper/20 flex items-center justify-center">
            <Layers className="w-5 h-5 text-ciab-copper" />
          </div>
          <div>
            <h1 className="text-xl font-semibold tracking-tight">Workspaces</h1>
            <p className="text-xs text-ciab-text-muted font-mono">
              {myWorkspaces?.length ?? 0} workspace{(myWorkspaces?.length ?? 0) !== 1 ? "s" : ""}
            </p>
          </div>
        </div>
        <div className="flex items-center gap-2">
          <button
            onClick={handleRefresh}
            className="p-2 rounded-md text-ciab-text-muted hover:text-ciab-text-primary hover:bg-ciab-bg-hover transition-colors"
            title="Refresh"
          >
            <RefreshCw className={`w-4 h-4 ${spinning || isFetching ? "animate-spin" : ""}`} />
          </button>
          <button
            onClick={() => setView("new")}
            className="btn-primary flex items-center gap-2 text-sm"
          >
            <Plus className="w-4 h-4" />
            New
          </button>
        </div>
      </div>

      {/* Search bar (when there are workspaces) */}
      {myWorkspaces && myWorkspaces.length > 2 && (
        <div className="flex items-center gap-2 bg-ciab-bg-secondary border border-ciab-border rounded-lg px-3 py-2">
          <Search className="w-3.5 h-3.5 text-ciab-text-muted" />
          <input
            type="text"
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            placeholder="Search workspaces..."
            className="bg-transparent border-none outline-none text-xs text-ciab-text-primary placeholder:text-ciab-text-muted/40 flex-1"
          />
          {searchQuery && (
            <button onClick={() => setSearchQuery("")} className="text-ciab-text-muted hover:text-ciab-text-secondary">
              <X className="w-3.5 h-3.5" />
            </button>
          )}
        </div>
      )}

      {/* Workspace list */}
      {filteredWorkspaces && filteredWorkspaces.length > 0 ? (
        <div className="space-y-2">
          {filteredWorkspaces.map((ws) => (
            <WorkspaceRow
              key={ws.id}
              workspace={ws}
              onDelete={() => {
                if (confirm(`Delete workspace "${ws.name}"?`)) {
                  deleteWorkspace.mutate(ws.id);
                }
              }}
              onLaunch={() => launchWorkspace.mutate({ id: ws.id })}
              isLaunching={launchWorkspace.isPending}
            />
          ))}
        </div>
      ) : myWorkspaces && myWorkspaces.length === 0 ? (
        <EmptyState
          icon={Layers}
          title="No workspaces yet"
          description="Create your first workspace from a template or start from scratch."
          action={
            <button onClick={() => setView("new")} className="btn-primary text-sm flex items-center gap-2">
              <Plus className="w-4 h-4" />
              New Workspace
            </button>
          }
        />
      ) : (
        <p className="text-sm text-ciab-text-muted text-center py-8">
          No workspaces match "{searchQuery}"
        </p>
      )}

      {showSync && <TemplateSyncDialog onClose={() => setShowSync(false)} />}
    </div>
  );
}

// ============================================================
// Workspace Row - compact list item for active workspaces
// ============================================================

function WorkspaceRow({
  workspace,
  onDelete,
  onLaunch,
  isLaunching,
}: {
  workspace: Workspace;
  onDelete: () => void;
  onLaunch: () => void;
  isLaunching: boolean;
}) {
  const repoCount = workspace.spec.repositories?.length ?? 0;
  const skillCount = workspace.spec.skills?.length ?? 0;
  const cmdCount = workspace.spec.pre_commands?.length ?? 0;
  const binaryCount = workspace.spec.binaries?.length ?? 0;
  const provider = workspace.spec.agent?.provider;
  const runtime = workspace.spec.runtime?.backend ?? "default";
  const fromTemplate = workspace.labels?.["ciab/from_template"];

  return (
    <div className="group rounded-xl border border-ciab-border bg-ciab-bg-card hover:border-ciab-copper/25 transition-all">
      <div className="flex items-center gap-4 p-4">
        {/* Icon */}
        <div className="w-10 h-10 rounded-lg bg-ciab-bg-elevated flex items-center justify-center flex-shrink-0">
          {provider ? (
            <AgentProviderIcon provider={provider} size={20} />
          ) : (
            <Layers className="w-5 h-5 text-ciab-text-muted" />
          )}
        </div>

        {/* Info */}
        <Link to={`/workspaces/${workspace.id}`} className="flex-1 min-w-0">
          <div className="flex items-center gap-2">
            <h3 className="font-medium text-sm text-ciab-text-primary truncate">
              {workspace.name}
            </h3>
            {fromTemplate && (
              <span className="text-[9px] font-mono text-ciab-copper/60 bg-ciab-copper/8 px-1.5 py-0.5 rounded flex-shrink-0">
                template
              </span>
            )}
          </div>
          {workspace.description && (
            <p className="text-[11px] text-ciab-text-muted mt-0.5 truncate">
              {workspace.description}
            </p>
          )}

          {/* Spec summary inline */}
          <div className="flex items-center gap-3 mt-1.5 text-[10px] font-mono text-ciab-text-muted">
            {provider && (
              <span className="flex items-center gap-1">
                <Bot className="w-2.5 h-2.5" />
                {PROVIDER_LABELS[provider] ?? provider}
              </span>
            )}
            {runtime !== "default" && (
              <span className="flex items-center gap-1">
                <Monitor className="w-2.5 h-2.5" />
                {runtime}
              </span>
            )}
            {repoCount > 0 && (
              <span className="flex items-center gap-1">
                <GitBranch className="w-2.5 h-2.5" />
                {repoCount}
              </span>
            )}
            {skillCount > 0 && (
              <span className="flex items-center gap-1">
                <Zap className="w-2.5 h-2.5" />
                {skillCount}
              </span>
            )}
            {cmdCount > 0 && (
              <span className="flex items-center gap-1">
                <Terminal className="w-2.5 h-2.5" />
                {cmdCount}
              </span>
            )}
            {binaryCount > 0 && (
              <span className="flex items-center gap-1">
                <Package className="w-2.5 h-2.5" />
                {binaryCount}
              </span>
            )}
            <span className="ml-auto text-ciab-text-muted/50">
              {formatRelativeTime(workspace.updated_at)}
            </span>
          </div>
        </Link>

        {/* Actions */}
        <div className="flex items-center gap-1 flex-shrink-0">
          <button
            onClick={onLaunch}
            disabled={isLaunching}
            className="flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-[11px] font-medium bg-ciab-copper/10 text-ciab-copper border border-ciab-copper/20 hover:bg-ciab-copper hover:text-white hover:border-ciab-copper transition-all disabled:opacity-40"
          >
            <Rocket className="w-3 h-3" />
            Launch
          </button>
          <Link
            to={`/workspaces/${workspace.id}`}
            className="p-2 rounded-lg text-ciab-text-muted hover:text-ciab-text-primary hover:bg-ciab-bg-hover transition-colors"
            title="Edit"
          >
            <Settings2 className="w-3.5 h-3.5" />
          </Link>
          <button
            onClick={onDelete}
            className="p-2 rounded-lg text-ciab-text-muted hover:text-state-failed hover:bg-state-failed/10 transition-colors opacity-0 group-hover:opacity-100"
            title="Delete"
          >
            <Trash2 className="w-3.5 h-3.5" />
          </button>
        </div>
      </div>
    </div>
  );
}

// ============================================================
// New Workspace View - template browser
// ============================================================

function NewWorkspaceView({
  templateList,
  onBack,
  onSelectStarter,
  onSelectSynced,
  onSelectBlank,
  onImportUrl,
  onManageSources,
}: {
  templateList: Workspace[];
  onBack: () => void;
  onSelectStarter: (s: StarterTemplate) => void;
  onSelectSynced: (ws: Workspace) => void;
  onSelectBlank: () => void;
  onImportUrl: (spec: WorkspaceSpec, name: string) => void;
  onManageSources: () => void;
}) {
  const [templateCategory, setTemplateCategory] = useState("all");
  const [showUrlImport, setShowUrlImport] = useState(false);
  const [importUrl, setImportUrl] = useState("");
  const [importLoading, setImportLoading] = useState(false);

  const filteredStarters =
    templateCategory === "all"
      ? STARTER_TEMPLATES
      : STARTER_TEMPLATES.filter((t) => t.category === templateCategory);

  const handleUrlImport = async () => {
    if (!importUrl.trim()) return;
    setImportLoading(true);
    try {
      const resp = await fetch(importUrl.trim());
      if (!resp.ok) throw new Error(`HTTP ${resp.status}`);
      const text = await resp.text();
      // Try parsing as JSON first, then as TOML (send to server)
      let spec: WorkspaceSpec;
      let name = "imported-workspace";
      try {
        const json = JSON.parse(text);
        spec = json.spec ?? json;
        name = json.name ?? name;
      } catch {
        // Assume TOML - import via server
        const ws = await workspacesApi.importToml(text);
        toast.success(`Imported "${ws.name}" from URL`);
        onImportUrl(ws.spec, ws.name);
        return;
      }
      onImportUrl(spec, name);
    } catch (err) {
      toast.error(`Import failed: ${(err as Error).message}`);
    } finally {
      setImportLoading(false);
    }
  };

  return (
    <div className="space-y-5 animate-fade-in">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-3">
          <button
            onClick={onBack}
            className="p-2 rounded-lg text-ciab-text-muted hover:text-ciab-text-primary hover:bg-ciab-bg-hover transition-colors"
          >
            <ArrowLeft className="w-4 h-4" />
          </button>
          <div>
            <h1 className="text-xl font-semibold tracking-tight">New Workspace</h1>
            <p className="text-xs text-ciab-text-muted font-mono">
              Pick a template or start from scratch
            </p>
          </div>
        </div>
        <div className="flex items-center gap-2">
          <button
            onClick={onManageSources}
            className="btn-ghost flex items-center gap-1.5 text-xs"
          >
            <Cloud className="w-3.5 h-3.5" />
            Sources
          </button>
          <button
            onClick={() => setShowUrlImport(!showUrlImport)}
            className={`btn-ghost flex items-center gap-1.5 text-xs ${showUrlImport ? "bg-ciab-bg-hover" : ""}`}
          >
            <Link2 className="w-3.5 h-3.5" />
            From URL
          </button>
        </div>
      </div>

      {/* URL import panel */}
      {showUrlImport && (
        <div className="rounded-xl border border-ciab-copper/20 bg-ciab-bg-secondary p-4 animate-fade-in">
          <div className="flex items-center gap-2 mb-2">
            <Link2 className="w-3.5 h-3.5 text-ciab-copper" />
            <span className="text-xs font-medium text-ciab-text-secondary">Import from URL</span>
          </div>
          <p className="text-[10px] text-ciab-text-muted mb-3">
            Paste a URL to a workspace TOML or JSON file. The template will be loaded and you can customize it before creating.
          </p>
          <div className="flex gap-2">
            <input
              type="url"
              value={importUrl}
              onChange={(e) => setImportUrl(e.target.value)}
              placeholder="https://raw.githubusercontent.com/.../workspace.toml"
              className="input flex-1 font-mono text-xs"
              onKeyDown={(e) => e.key === "Enter" && handleUrlImport()}
            />
            <button
              onClick={handleUrlImport}
              disabled={!importUrl.trim() || importLoading}
              className="btn-primary text-xs px-4 disabled:opacity-30 flex items-center gap-1.5"
            >
              {importLoading ? (
                <Loader2 className="w-3 h-3 animate-spin" />
              ) : (
                <ExternalLink className="w-3 h-3" />
              )}
              Load
            </button>
          </div>
        </div>
      )}

      {/* Blank workspace option */}
      <button
        onClick={onSelectBlank}
        className="w-full rounded-xl border border-dashed border-ciab-border hover:border-ciab-copper/30 bg-ciab-bg-card p-4 text-left transition-all group flex items-center gap-4"
      >
        <div className="w-10 h-10 rounded-lg bg-ciab-bg-primary border border-ciab-border flex items-center justify-center flex-shrink-0 group-hover:border-ciab-copper/20">
          <FileCode2 className="w-5 h-5 text-ciab-text-muted group-hover:text-ciab-copper transition-colors" />
        </div>
        <div className="flex-1">
          <p className="text-sm font-medium text-ciab-text-primary group-hover:text-ciab-copper transition-colors">
            Blank Workspace
          </p>
          <p className="text-[11px] text-ciab-text-muted">
            Start from scratch with full control
          </p>
        </div>
        <ArrowRight className="w-4 h-4 text-ciab-text-muted group-hover:text-ciab-copper transition-colors" />
      </button>

      {/* Starter Templates */}
      <section>
        <div className="flex items-center justify-between mb-3">
          <div className="flex items-center gap-2">
            <Sparkles className="w-3.5 h-3.5 text-ciab-copper" />
            <h2 className="text-xs font-mono text-ciab-text-muted uppercase tracking-wider">
              Starter Templates
            </h2>
            <span className="text-[10px] font-mono text-ciab-copper bg-ciab-copper/10 px-1.5 py-0.5 rounded">
              {STARTER_TEMPLATES.length}
            </span>
          </div>
          <div className="flex items-center gap-1">
            {TEMPLATE_CATEGORIES.map((cat) => (
              <button
                key={cat.id}
                onClick={() => setTemplateCategory(cat.id)}
                className={`px-2 py-0.5 rounded text-[10px] font-mono transition-colors ${
                  templateCategory === cat.id
                    ? "bg-ciab-copper/15 text-ciab-copper border border-ciab-copper/20"
                    : "text-ciab-text-muted hover:text-ciab-text-secondary hover:bg-ciab-bg-hover"
                }`}
              >
                {cat.label}
              </button>
            ))}
          </div>
        </div>

        <div className="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-3 2xl:grid-cols-4 gap-3">
          {filteredStarters.map((starter) => {
            const IconComponent = ICON_MAP[starter.icon] ?? Sparkles;
            return (
              <button
                key={starter.id}
                onClick={() => onSelectStarter(starter)}
                className="group/card text-left rounded-xl border border-ciab-border bg-ciab-bg-card overflow-hidden transition-all hover:border-ciab-copper/30 hover:shadow-lg hover:shadow-ciab-copper/5"
              >
                <div className={`h-1 bg-gradient-to-r ${starter.color} opacity-50 group-hover/card:opacity-100 transition-opacity`} />
                <div className="p-3.5">
                  <div className="flex items-center gap-2.5 mb-2">
                    <div className={`w-8 h-8 rounded-lg bg-gradient-to-br ${starter.color} border border-white/5 flex items-center justify-center flex-shrink-0`}>
                      <IconComponent className="w-3.5 h-3.5 text-ciab-text-primary" />
                    </div>
                    <div className="min-w-0 flex-1">
                      <h3 className="font-medium text-[13px] text-ciab-text-primary truncate leading-tight">
                        {starter.name}
                      </h3>
                      <div className="flex items-center gap-1.5">
                        <AgentProviderIcon provider={starter.provider} size={10} />
                        <span className="text-[10px] text-ciab-text-muted font-mono">
                          {PROVIDER_LABELS[starter.provider]}
                        </span>
                      </div>
                    </div>
                    <span className="text-[9px] font-mono text-ciab-text-muted bg-ciab-bg-elevated px-1.5 py-0.5 rounded">
                      {starter.category}
                    </span>
                  </div>
                  <p className="text-[11px] text-ciab-text-secondary leading-relaxed line-clamp-2">
                    {starter.description}
                  </p>
                </div>
              </button>
            );
          })}
        </div>
      </section>

      {/* Synced Templates */}
      {templateList.length > 0 && (
        <section>
          <div className="flex items-center gap-2 mb-3">
            <Cloud className="w-3.5 h-3.5 text-ciab-steel-blue" />
            <h2 className="text-xs font-mono text-ciab-text-muted uppercase tracking-wider">
              Synced Templates
            </h2>
            <span className="text-[10px] font-mono text-ciab-steel-blue bg-ciab-steel-blue/10 px-1.5 py-0.5 rounded">
              {templateList.length}
            </span>
          </div>
          <div className="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-3 2xl:grid-cols-4 gap-3">
            {templateList.map((tmpl) => (
              <button
                key={tmpl.id}
                onClick={() => onSelectSynced(tmpl)}
                className="group/card text-left rounded-xl border border-ciab-border bg-ciab-bg-card overflow-hidden transition-all hover:border-ciab-steel-blue/30 hover:shadow-lg hover:shadow-ciab-steel-blue/5"
              >
                <div className="h-1 bg-gradient-to-r from-ciab-steel-blue via-ciab-steel-blue-light to-ciab-copper opacity-30 group-hover/card:opacity-80 transition-opacity" />
                <div className="p-3.5">
                  <div className="flex items-center gap-2.5 mb-2">
                    <div className="w-8 h-8 rounded-lg bg-ciab-steel-blue/10 border border-ciab-steel-blue/15 flex items-center justify-center flex-shrink-0">
                      {tmpl.spec.agent?.provider ? (
                        <AgentProviderIcon provider={tmpl.spec.agent.provider} size={16} />
                      ) : (
                        <Sparkles className="w-3.5 h-3.5 text-ciab-steel-blue" />
                      )}
                    </div>
                    <div className="min-w-0 flex-1">
                      <h3 className="font-medium text-[13px] text-ciab-text-primary truncate leading-tight">
                        {tmpl.name}
                      </h3>
                      {tmpl.spec.agent?.provider && (
                        <span className="text-[10px] text-ciab-text-muted font-mono">
                          {PROVIDER_LABELS[tmpl.spec.agent.provider] ?? tmpl.spec.agent.provider}
                        </span>
                      )}
                    </div>
                    <span className="flex items-center gap-1 text-[9px] font-mono text-ciab-steel-blue bg-ciab-steel-blue/10 px-1.5 py-0.5 rounded flex-shrink-0">
                      <Cloud className="w-2.5 h-2.5" />
                      synced
                    </span>
                  </div>
                  {tmpl.description && (
                    <p className="text-[11px] text-ciab-text-secondary leading-relaxed line-clamp-2">
                      {tmpl.description}
                    </p>
                  )}
                </div>
              </button>
            ))}
          </div>
        </section>
      )}
    </div>
  );
}

// ============================================================
// Configure Workspace - override settings before creating
// ============================================================

function ConfigureWorkspace({
  target,
  onBack,
  onCreated,
}: {
  target: {
    type: "starter" | "synced" | "url" | "blank";
    starter?: StarterTemplate;
    synced?: Workspace;
    spec?: WorkspaceSpec;
    name?: string;
    description?: string;
  };
  onBack: () => void;
  onCreated: (ws?: Workspace) => void;
}) {
  const createWorkspace = useCreateWorkspace();
  const createFromTemplate = useCreateFromTemplate();

  const baseSpec = target.spec ?? { agent: { provider: "claude-code" } };

  const [name, setName] = useState(target.name ?? "");
  const [description, setDescription] = useState(target.description ?? "");
  const [provider, setProvider] = useState(baseSpec.agent?.provider ?? "claude-code");
  const [runtimeBackend, setRuntimeBackend] = useState<RuntimeBackend>(baseSpec.runtime?.backend ?? "default");
  const [systemPrompt, setSystemPrompt] = useState(baseSpec.agent?.system_prompt ?? "");
  const [repos, setRepos] = useState<WorkspaceRepo[]>(baseSpec.repositories ?? []);
  const [preCommands, setPreCommands] = useState<PreCommand[]>(baseSpec.pre_commands ?? []);
  const [binaries, setBinaries] = useState<BinaryInstall[]>(baseSpec.binaries ?? []);
  const [envVars, setEnvVars] = useState<Array<{ key: string; value: string }>>(
    baseSpec.env_vars ? Object.entries(baseSpec.env_vars).map(([key, value]) => ({ key, value })) : []
  );

  // Collapsible sections
  const [showRepos, setShowRepos] = useState(repos.length > 0);
  const [showCommands, setShowCommands] = useState(preCommands.length > 0);
  const [showBinaries, setShowBinaries] = useState(binaries.length > 0);
  const [showEnv, setShowEnv] = useState(envVars.length > 0);
  const [showAgent, setShowAgent] = useState(false);

  const templateLabel =
    target.type === "starter"
      ? target.starter?.name
      : target.type === "synced"
        ? target.synced?.name
        : target.type === "url"
          ? "Imported Template"
          : null;

  const buildSpec = (): WorkspaceSpec => {
    const envMap: Record<string, string> = {};
    envVars.forEach(({ key, value }) => {
      if (key.trim()) envMap[key.trim()] = value;
    });

    return {
      ...baseSpec,
      agent: {
        ...baseSpec.agent,
        provider,
        ...(systemPrompt.trim() ? { system_prompt: systemPrompt.trim() } : {}),
      },
      ...(repos.length > 0 ? { repositories: repos.filter((r) => r.url.trim()) } : { repositories: undefined }),
      ...(preCommands.length > 0 ? { pre_commands: preCommands.filter((c) => c.command.trim()) } : { pre_commands: undefined }),
      ...(binaries.length > 0 ? { binaries: binaries.filter((b) => b.name.trim()) } : { binaries: undefined }),
      ...(Object.keys(envMap).length > 0 ? { env_vars: envMap } : { env_vars: undefined }),
      ...(runtimeBackend !== "default" ? { runtime: { backend: runtimeBackend } } : { runtime: undefined }),
    };
  };

  const handleCreate = () => {
    if (!name.trim()) return;

    if (target.type === "synced" && target.synced) {
      // Use createFromTemplate with overrides
      const overrides: Partial<WorkspaceSpec> = {};
      if (provider !== (baseSpec.agent?.provider ?? "claude-code")) {
        overrides.agent = { ...baseSpec.agent, provider };
      }
      if (systemPrompt.trim() !== (baseSpec.agent?.system_prompt ?? "")) {
        overrides.agent = { ...overrides.agent, provider: overrides.agent?.provider ?? provider, system_prompt: systemPrompt.trim() };
      }
      if (runtimeBackend !== (baseSpec.runtime?.backend ?? "default")) {
        overrides.runtime = { backend: runtimeBackend };
      }

      createFromTemplate.mutate(
        {
          templateId: target.synced.id,
          name: name.trim(),
          description: description.trim() || undefined,
          overrides: Object.keys(overrides).length > 0 ? overrides : undefined,
        },
        {
          onSuccess: (data: unknown) => onCreated(data as Workspace),
        }
      );
    } else {
      createWorkspace.mutate(
        {
          name: name.trim(),
          description: description.trim() || undefined,
          spec: buildSpec(),
        },
        {
          onSuccess: (data: unknown) => onCreated(data as Workspace),
        }
      );
    }
  };

  const isPending = createWorkspace.isPending || createFromTemplate.isPending;

  return (
    <div className="animate-fade-in max-w-2xl mx-auto">
      {/* Header */}
      <div className="flex items-center gap-3 mb-6">
        <button
          onClick={onBack}
          className="p-2 rounded-lg text-ciab-text-muted hover:text-ciab-text-primary hover:bg-ciab-bg-hover transition-colors"
        >
          <ArrowLeft className="w-4 h-4" />
        </button>
        <div className="flex-1">
          <h1 className="text-lg font-semibold tracking-tight">Configure Workspace</h1>
          {templateLabel && (
            <p className="text-xs text-ciab-text-muted font-mono flex items-center gap-1.5">
              <Sparkles className="w-3 h-3 text-ciab-copper" />
              Based on {templateLabel}
            </p>
          )}
        </div>
      </div>

      <div className="space-y-4">
        {/* Provisioning Preview */}
        <ProvisioningPreview
          provider={provider}
          repos={repos}
          skills={baseSpec.skills ?? []}
          preCommands={preCommands}
          binaries={binaries}
          envVarCount={envVars.filter((e) => e.key.trim()).length}
          runtime={runtimeBackend}
          systemPrompt={systemPrompt}
        />

        {/* Name & Description */}
        <div className="rounded-xl border border-ciab-border bg-ciab-bg-card p-4 space-y-3">
          <div className="grid grid-cols-2 gap-3">
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
                placeholder="Optional"
              />
            </div>
          </div>

          {/* Runtime */}
          <div>
            <label className="label">Runtime</label>
            <div className="grid grid-cols-5 gap-1.5">
              {(["default", "local", "opensandbox", "docker", "kubernetes"] as RuntimeBackend[]).map((b) => (
                <button
                  key={b}
                  onClick={() => setRuntimeBackend(b)}
                  className={`flex items-center gap-1.5 p-2 rounded-md border transition-all text-left ${
                    runtimeBackend === b
                      ? "border-ciab-copper/50 bg-ciab-copper/5"
                      : "border-ciab-border hover:border-ciab-border-light"
                  }`}
                >
                  <Monitor className="w-3.5 h-3.5 text-ciab-text-muted" />
                  <span className="text-[11px] font-medium capitalize">{b}</span>
                </button>
              ))}
            </div>
          </div>

          {/* Provider */}
          <div>
            <label className="label">Agent</label>
            <div className="grid grid-cols-4 gap-1.5">
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
        </div>

        {/* Collapsible overrides */}
        <div className="rounded-xl border border-ciab-border bg-ciab-bg-card overflow-hidden divide-y divide-ciab-border">
          {/* Repositories */}
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
                        placeholder="branch"
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
                        placeholder="dest path"
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

          {/* Binaries */}
          <CollapsibleSection
            title="Packages"
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
                <Plus className="w-3 h-3" /> Add package
              </button>
            </div>
          </CollapsibleSection>

          {/* Pre-commands */}
          <CollapsibleSection
            title="Setup Commands"
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
                    className="input w-20 text-[11px] py-1"
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

          {/* Environment Variables */}
          <CollapsibleSection
            title="Environment"
            icon={Zap}
            open={showEnv}
            onToggle={() => setShowEnv(!showEnv)}
            count={envVars.filter((e) => e.key.trim()).length}
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

          {/* System Prompt */}
          <CollapsibleSection
            title="System Prompt"
            icon={Bot}
            open={showAgent}
            onToggle={() => setShowAgent(!showAgent)}
            count={systemPrompt.trim() ? 1 : undefined}
          >
            <textarea
              value={systemPrompt}
              onChange={(e) => setSystemPrompt(e.target.value)}
              className="input w-full resize-none text-xs font-mono"
              rows={4}
              placeholder="You are a senior developer working on this project..."
            />
          </CollapsibleSection>
        </div>

        {/* Actions */}
        <div className="flex items-center justify-between pt-2 pb-4">
          <button onClick={onBack} className="btn-ghost text-xs">
            Back
          </button>
          <button
            onClick={handleCreate}
            disabled={!name.trim() || isPending}
            className="btn-primary flex items-center gap-2 text-sm px-5 py-2.5 disabled:opacity-30"
          >
            {isPending ? (
              <>
                <Loader2 className="w-3.5 h-3.5 animate-spin" />
                Creating...
              </>
            ) : (
              <>
                Create Workspace
                <ArrowRight className="w-3.5 h-3.5" />
              </>
            )}
          </button>
        </div>
      </div>
    </div>
  );
}

// ============================================================
// Provisioning Preview - visual summary of what will be set up
// ============================================================

function ProvisioningPreview({
  provider,
  repos,
  skills,
  preCommands,
  binaries,
  envVarCount,
  runtime,
  systemPrompt,
}: {
  provider: string;
  repos: WorkspaceRepo[];
  skills: Array<{ source: string; name?: string; enabled?: boolean }>;
  preCommands: PreCommand[];
  binaries: BinaryInstall[];
  envVarCount: number;
  runtime: RuntimeBackend;
  systemPrompt: string;
}) {
  const steps: Array<{ icon: typeof Bot; label: string; detail: string; color: string }> = [];

  // Agent
  steps.push({
    icon: Bot,
    label: "Agent",
    detail: PROVIDER_LABELS[provider] ?? provider,
    color: "text-ciab-copper",
  });

  // Runtime
  if (runtime !== "default") {
    steps.push({
      icon: Monitor,
      label: "Runtime",
      detail: runtime,
      color: "text-ciab-steel-blue",
    });
  }

  // Repos
  if (repos.length > 0) {
    steps.push({
      icon: GitBranch,
      label: `${repos.length} repo${repos.length > 1 ? "s" : ""}`,
      detail: repos.map((r) => r.url.split("/").pop()?.replace(".git", "")).filter(Boolean).join(", ") || "git clone",
      color: "text-emerald-400",
    });
  }

  // Binaries
  if (binaries.length > 0) {
    steps.push({
      icon: Package,
      label: `${binaries.length} pkg${binaries.length > 1 ? "s" : ""}`,
      detail: binaries.map((b) => b.name).filter(Boolean).join(", ") || "install",
      color: "text-amber-400",
    });
  }

  // Skills
  const enabledSkills = skills.filter((s) => s.enabled !== false);
  if (enabledSkills.length > 0) {
    steps.push({
      icon: Zap,
      label: `${enabledSkills.length} skill${enabledSkills.length > 1 ? "s" : ""}`,
      detail: enabledSkills.map((s) => s.name ?? s.source.split("/").pop()).join(", "),
      color: "text-violet-400",
    });
  }

  // Commands
  if (preCommands.length > 0) {
    steps.push({
      icon: Terminal,
      label: `${preCommands.length} cmd${preCommands.length > 1 ? "s" : ""}`,
      detail: preCommands.map((c) => c.name ?? c.command.split(" ")[0]).join(", "),
      color: "text-sky-400",
    });
  }

  // Env vars
  if (envVarCount > 0) {
    steps.push({
      icon: Zap,
      label: `${envVarCount} env`,
      detail: "environment variables",
      color: "text-ciab-text-muted",
    });
  }

  if (steps.length === 0) return null;

  return (
    <div className="rounded-xl border border-ciab-border bg-ciab-bg-card p-4">
      <div className="flex items-center gap-2 mb-3">
        <Rocket className="w-3.5 h-3.5 text-ciab-copper" />
        <span className="text-[10px] font-mono text-ciab-text-muted uppercase tracking-wider">
          Provisioning Steps
        </span>
      </div>
      <div className="flex flex-wrap gap-2">
        {steps.map((step, i) => (
          <div
            key={i}
            className="flex items-center gap-2 px-2.5 py-1.5 rounded-lg bg-ciab-bg-primary border border-ciab-border"
          >
            <step.icon className={`w-3 h-3 ${step.color}`} />
            <span className="text-[11px] font-medium text-ciab-text-secondary">{step.label}</span>
            <span className="text-[10px] text-ciab-text-muted font-mono truncate max-w-[140px]">
              {step.detail}
            </span>
          </div>
        ))}
      </div>
      {systemPrompt.trim() && (
        <div className="mt-2.5 pt-2.5 border-t border-ciab-border">
          <p className="text-[10px] text-ciab-text-muted line-clamp-1 font-mono">
            <Bot className="w-2.5 h-2.5 inline mr-1" />
            {systemPrompt.trim()}
          </p>
        </div>
      )}
    </div>
  );
}

// ============================================================
// Collapsible Section
// ============================================================

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
    <div>
      <button
        onClick={onToggle}
        className="flex items-center gap-2 w-full px-4 py-2.5 text-left hover:bg-ciab-bg-hover/30 transition-colors"
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
        <div className="px-4 pb-3 pt-1 animate-fade-in">
          {children}
        </div>
      )}
    </div>
  );
}
