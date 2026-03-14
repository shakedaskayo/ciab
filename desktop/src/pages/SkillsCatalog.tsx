import { useState, useEffect, useCallback } from "react";
import {
  Search,
  X,
  Zap,
  Download,
  Sparkles,
  ExternalLink,
  Loader2,
  TrendingUp,
  BookOpen,
  Plus,
  Check,
  Layers,
} from "lucide-react";
import { useSkillSearch, useSkillMetadata, useTrendingSkills } from "@/lib/hooks/use-skills";
import { useWorkspaces, useUpdateWorkspace } from "@/lib/hooks/use-workspaces";
import type { SkillSearchResult } from "@/lib/api/endpoints";
import type { Workspace } from "@/lib/api/types";
import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";
import { toast } from "sonner";

export default function SkillsCatalog() {
  const [searchQuery, setSearchQuery] = useState("");
  const [debouncedQuery, setDebouncedQuery] = useState("");
  const [selectedSkill, setSelectedSkill] = useState<SkillSearchResult | null>(null);

  useEffect(() => {
    const timer = setTimeout(() => setDebouncedQuery(searchQuery), 300);
    return () => clearTimeout(timer);
  }, [searchQuery]);

  const { data: searchData, isLoading: searchLoading, isFetching } = useSkillSearch(debouncedQuery, 30);
  const { data: trendingData, isLoading: trendingLoading } = useTrendingSkills();

  const isSearching = searchQuery.length >= 2;
  const results = isSearching ? (searchData?.skills ?? []) : (trendingData?.skills ?? []);
  const isLoading = isSearching ? searchLoading : trendingLoading;

  const suggestions = [
    "react", "typescript", "python", "rust", "testing",
    "docker", "security", "api", "nextjs", "devops",
  ];

  return (
    <div className="flex h-[calc(100vh-8rem)]">
      {/* Main Content */}
      <div className="flex-1 min-w-0 overflow-auto pr-4">
        {/* Header */}
        <div className="mb-5">
          <div className="flex items-center gap-3 mb-1">
            <div className="w-9 h-9 rounded-xl bg-gradient-to-br from-ciab-copper/20 to-ciab-copper/5 border border-ciab-copper/20 flex items-center justify-center">
              <Zap className="w-5 h-5 text-ciab-copper" />
            </div>
            <div>
              <h1 className="text-xl font-bold text-ciab-text-primary">Skills Catalog</h1>
              <p className="text-xs text-ciab-text-muted font-mono">
                Browse and install skills from the{" "}
                <span className="text-ciab-copper">skills.sh</span> open registry
              </p>
            </div>
          </div>
        </div>

        {/* Search */}
        <div className="mb-4">
          <div className="flex items-center gap-2 bg-ciab-bg-secondary border border-ciab-border rounded-xl px-4 py-2.5 focus-within:border-ciab-copper/40 focus-within:ring-1 focus-within:ring-ciab-copper/20 transition-all">
            <Search className="w-4 h-4 text-ciab-text-muted flex-shrink-0" />
            <input
              type="text"
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              placeholder="Search skills by name or keyword..."
              className="flex-1 bg-transparent border-none outline-none text-sm text-ciab-text-primary placeholder:text-ciab-text-muted/40"
            />
            {isFetching && (
              <Loader2 className="w-3.5 h-3.5 text-ciab-copper animate-spin flex-shrink-0" />
            )}
            {searchQuery && !isFetching && (
              <button
                onClick={() => { setSearchQuery(""); setSelectedSkill(null); }}
                className="text-ciab-text-muted hover:text-ciab-text-secondary transition-colors"
              >
                <X className="w-4 h-4" />
              </button>
            )}
          </div>
        </div>

        {/* Quick search chips */}
        <div className="flex flex-wrap gap-1.5 mb-4">
          {suggestions.map((s) => (
            <button
              key={s}
              onClick={() => setSearchQuery(s)}
              className={`px-2.5 py-1 rounded-lg text-[11px] font-medium transition-colors ${
                searchQuery === s
                  ? "bg-ciab-copper/15 text-ciab-copper border border-ciab-copper/20"
                  : "bg-ciab-bg-secondary border border-ciab-border text-ciab-text-muted hover:text-ciab-copper hover:border-ciab-copper/30"
              }`}
            >
              {s}
            </button>
          ))}
        </div>

        {/* Section header */}
        <div className="flex items-center gap-2 mb-3">
          <TrendingUp className="w-3.5 h-3.5 text-ciab-copper" />
          <span className="text-[11px] font-mono text-ciab-text-muted uppercase tracking-wider">
            {isSearching
              ? `${results.length} results for "${debouncedQuery}"`
              : `Trending Skills`}
          </span>
          {isLoading && <Loader2 className="w-3 h-3 text-ciab-copper animate-spin" />}
        </div>

        {/* Loading */}
        {isLoading && results.length === 0 && (
          <div className="text-center py-16">
            <Loader2 className="w-6 h-6 text-ciab-copper animate-spin mx-auto mb-3" />
            <p className="text-sm text-ciab-text-muted">
              {isSearching ? "Searching skills.sh..." : "Loading trending skills..."}
            </p>
          </div>
        )}

        {/* Results grid */}
        {results.length > 0 && (
          <div className="grid grid-cols-1 gap-2">
            {results.map((skill) => (
              <SkillRow
                key={skill.id}
                skill={skill}
                isSelected={selectedSkill?.id === skill.id}
                onSelect={() => setSelectedSkill(selectedSkill?.id === skill.id ? null : skill)}
              />
            ))}
          </div>
        )}

        {/* No results */}
        {!isLoading && results.length === 0 && isSearching && (
          <div className="text-center py-12">
            <Sparkles className="w-8 h-8 text-ciab-text-muted/20 mx-auto mb-3" />
            <p className="text-sm text-ciab-text-secondary">No skills found</p>
            <p className="text-xs text-ciab-text-muted mt-1">Try a different search term</p>
          </div>
        )}

        {/* Query too short */}
        {searchQuery.length === 1 && (
          <div className="text-center py-8">
            <p className="text-xs text-ciab-text-muted">Type at least 2 characters to search</p>
          </div>
        )}
      </div>

      {/* Detail Panel */}
      {selectedSkill && (
        <SkillDetailPanel
          skill={selectedSkill}
          onClose={() => setSelectedSkill(null)}
        />
      )}
    </div>
  );
}

function formatInstalls(count: number): string {
  if (count >= 1_000_000) return `${(count / 1_000_000).toFixed(1)}M`;
  if (count >= 1_000) return `${(count / 1_000).toFixed(1)}K`;
  return String(count);
}

function SkillRow({
  skill,
  isSelected,
  onSelect,
}: {
  skill: SkillSearchResult;
  isSelected: boolean;
  onSelect: () => void;
}) {
  return (
    <button
      onClick={onSelect}
      className={`w-full text-left rounded-xl border p-3.5 transition-all group ${
        isSelected
          ? "border-ciab-copper/30 bg-ciab-copper/5"
          : "border-ciab-border bg-ciab-bg-card hover:border-ciab-copper/20 hover:bg-ciab-bg-hover/50"
      }`}
    >
      <div className="flex items-center gap-3">
        <div className="w-9 h-9 rounded-xl bg-ciab-bg-primary border border-ciab-border flex items-center justify-center flex-shrink-0">
          <Zap className="w-4 h-4 text-ciab-copper/60" />
        </div>

        <div className="flex-1 min-w-0">
          <span className="font-medium text-sm text-ciab-text-primary group-hover:text-ciab-copper transition-colors truncate block">
            {skill.name}
          </span>
          <span className="text-[10px] text-ciab-text-muted/60 font-mono truncate block">
            {skill.source}
          </span>
        </div>

        <div className="flex items-center gap-1 text-[10px] text-ciab-text-muted/50 font-mono flex-shrink-0">
          <Download className="w-3 h-3" />
          {formatInstalls(skill.installs)}
        </div>
      </div>
    </button>
  );
}

function SkillDetailPanel({
  skill,
  onClose,
}: {
  skill: SkillSearchResult;
  onClose: () => void;
}) {
  const { data: metadata, isLoading } = useSkillMetadata(skill.source, skill.skillId);
  const [showWorkspacePicker, setShowWorkspacePicker] = useState(false);

  return (
    <div className="w-[380px] flex-shrink-0 ml-4 border-l border-ciab-border pl-4 overflow-auto">
      {/* Close */}
      <div className="flex items-center justify-between mb-4">
        <span className="text-[10px] font-mono text-ciab-text-muted uppercase tracking-wider">
          Skill Details
        </span>
        <button
          onClick={onClose}
          className="p-1 rounded text-ciab-text-muted hover:text-ciab-text-secondary hover:bg-ciab-bg-hover transition-colors"
        >
          <X className="w-3.5 h-3.5" />
        </button>
      </div>

      {/* Header */}
      <div className="flex items-center gap-3 mb-4">
        <div className="w-12 h-12 rounded-xl bg-ciab-bg-primary border border-ciab-border flex items-center justify-center">
          <Zap className="w-5 h-5 text-ciab-copper" />
        </div>
        <div className="min-w-0">
          <h3 className="font-semibold text-ciab-text-primary truncate">{skill.name}</h3>
          <p className="text-[10px] font-mono text-ciab-text-muted truncate">{skill.source}</p>
        </div>
      </div>

      {/* Install to Workspace — primary action */}
      <div className="mb-4">
        {!showWorkspacePicker ? (
          <button
            onClick={() => setShowWorkspacePicker(true)}
            className="w-full flex items-center justify-center gap-2 px-4 py-2.5 rounded-xl bg-ciab-copper text-white font-medium text-sm hover:bg-ciab-copper-hover transition-colors"
          >
            <Plus className="w-4 h-4" />
            Add to Workspace
          </button>
        ) : (
          <WorkspacePicker
            skillSource={skill.id}
            skillName={skill.name}
            onClose={() => setShowWorkspacePicker(false)}
          />
        )}
      </div>

      {/* Stats */}
      <div className="grid grid-cols-2 gap-3 mb-4">
        <div className="bg-ciab-bg-primary rounded-lg p-2.5 border border-ciab-border">
          <p className="text-[9px] font-mono text-ciab-text-muted uppercase tracking-wider mb-0.5">Installs</p>
          <div className="flex items-center gap-1.5">
            <Download className="w-3 h-3 text-ciab-copper" />
            <span className="text-sm font-medium text-ciab-text-primary">{formatInstalls(skill.installs)}</span>
          </div>
        </div>
        <div className="bg-ciab-bg-primary rounded-lg p-2.5 border border-ciab-border">
          <p className="text-[9px] font-mono text-ciab-text-muted uppercase tracking-wider mb-0.5">Source</p>
          <a
            href={`https://github.com/${skill.source}`}
            target="_blank"
            rel="noopener noreferrer"
            className="text-xs font-mono text-ciab-copper hover:underline flex items-center gap-1"
          >
            GitHub <ExternalLink className="w-2.5 h-2.5" />
          </a>
        </div>
      </div>

      {/* Install command */}
      <div className="bg-ciab-bg-primary rounded-lg p-3 border border-ciab-border mb-4">
        <p className="text-[9px] font-mono text-ciab-text-muted uppercase tracking-wider mb-1.5">CLI Install</p>
        <code className="text-xs font-mono text-ciab-copper break-all select-all">
          npx skills add {skill.source}/{skill.skillId}
        </code>
      </div>

      {/* Loading metadata */}
      {isLoading && (
        <div className="flex items-center gap-2 py-4 justify-center">
          <Loader2 className="w-4 h-4 text-ciab-copper animate-spin" />
          <span className="text-xs text-ciab-text-muted">Fetching SKILL.md...</span>
        </div>
      )}

      {/* Metadata from SKILL.md */}
      {metadata && (
        <>
          {metadata.description && (
            <div className="mb-4">
              <p className="text-[9px] font-mono text-ciab-text-muted uppercase tracking-wider mb-1.5">Description</p>
              <p className="text-sm text-ciab-text-secondary leading-relaxed">{metadata.description}</p>
            </div>
          )}

          {metadata.available_skills.length > 1 && (
            <div className="mb-4">
              <p className="text-[9px] font-mono text-ciab-text-muted uppercase tracking-wider mb-1.5">
                Skills in this repo ({metadata.available_skills.length})
              </p>
              <div className="space-y-1">
                {metadata.available_skills.map((s) => (
                  <div
                    key={s.path}
                    className={`flex items-center gap-2 px-2.5 py-1.5 rounded-lg text-xs font-mono ${
                      s.skill_id === skill.skillId
                        ? "bg-ciab-copper/10 text-ciab-copper border border-ciab-copper/20"
                        : "bg-ciab-bg-secondary text-ciab-text-muted border border-ciab-border"
                    }`}
                  >
                    <Zap className="w-3 h-3 flex-shrink-0" />
                    <span className="truncate">{s.skill_id}</span>
                  </div>
                ))}
              </div>
            </div>
          )}

          {metadata.raw_content && (
            <div className="mb-4">
              <p className="text-[9px] font-mono text-ciab-text-muted uppercase tracking-wider mb-1.5 flex items-center gap-1">
                <BookOpen className="w-3 h-3" />
                SKILL.md
              </p>
              <div className="bg-ciab-bg-primary rounded-lg border border-ciab-border p-3 max-h-[300px] overflow-auto">
                <div className="text-xs prose prose-invert prose-sm max-w-none leading-relaxed
                  prose-p:my-1.5 prose-code:text-ciab-copper-light prose-code:bg-ciab-bg-secondary
                  prose-code:px-1 prose-code:py-0.5 prose-code:rounded prose-code:text-[10px]
                  prose-code:before:content-none prose-code:after:content-none
                  prose-headings:text-ciab-text-primary prose-headings:text-xs prose-headings:mt-3 prose-headings:mb-1
                  prose-ul:my-1 prose-li:my-0.5 prose-li:text-ciab-text-secondary">
                  <ReactMarkdown remarkPlugins={[remarkGfm]}>
                    {stripFrontmatter(metadata.raw_content)}
                  </ReactMarkdown>
                </div>
              </div>
            </div>
          )}
        </>
      )}
    </div>
  );
}

/** Workspace picker — lets user pick which workspace to install the skill into. */
function WorkspacePicker({
  skillSource,
  skillName,
  onClose,
}: {
  skillSource: string;
  skillName: string;
  onClose: () => void;
}) {
  const { data: workspaces, isLoading } = useWorkspaces();
  const updateWorkspace = useUpdateWorkspace();
  const [installedIds, setInstalledIds] = useState<Set<string>>(new Set());

  const handleInstall = useCallback(
    (workspace: Workspace) => {
      const existingSkills = workspace.spec.skills ?? [];
      if (existingSkills.some((s) => s.source === skillSource)) {
        toast.info(`"${skillName}" is already in ${workspace.name}`);
        setInstalledIds((prev) => new Set([...prev, workspace.id]));
        return;
      }

      updateWorkspace.mutate(
        {
          id: workspace.id,
          spec: {
            ...workspace.spec,
            skills: [
              ...existingSkills,
              { source: skillSource, name: skillName, enabled: true },
            ],
          },
        },
        {
          onSuccess: () => {
            toast.success(`Added "${skillName}" to ${workspace.name}`);
            setInstalledIds((prev) => new Set([...prev, workspace.id]));
          },
        }
      );
    },
    [skillSource, skillName, updateWorkspace]
  );

  return (
    <div className="rounded-xl border border-ciab-copper/20 bg-ciab-bg-secondary overflow-hidden animate-fade-in">
      <div className="flex items-center justify-between px-3 py-2 border-b border-ciab-border">
        <span className="text-[10px] font-mono text-ciab-text-muted uppercase tracking-wider">
          Choose Workspace
        </span>
        <button
          onClick={onClose}
          className="p-0.5 rounded text-ciab-text-muted hover:text-ciab-text-secondary"
        >
          <X className="w-3 h-3" />
        </button>
      </div>

      {isLoading && (
        <div className="flex items-center justify-center gap-2 py-6">
          <Loader2 className="w-4 h-4 text-ciab-copper animate-spin" />
        </div>
      )}

      {!isLoading && (!workspaces || workspaces.length === 0) && (
        <div className="text-center py-6 px-3">
          <Layers className="w-5 h-5 text-ciab-text-muted/30 mx-auto mb-2" />
          <p className="text-xs text-ciab-text-muted">No workspaces yet</p>
          <p className="text-[10px] text-ciab-text-muted/60 mt-1">Create a workspace first</p>
        </div>
      )}

      {workspaces && workspaces.length > 0 && (
        <div className="max-h-[200px] overflow-auto p-1.5 space-y-0.5">
          {workspaces.map((ws) => {
            const alreadyHas =
              installedIds.has(ws.id) ||
              (ws.spec.skills ?? []).some((s) => s.source === skillSource);
            return (
              <button
                key={ws.id}
                onClick={() => !alreadyHas && handleInstall(ws)}
                disabled={alreadyHas}
                className={`w-full text-left flex items-center gap-2.5 px-3 py-2 rounded-lg transition-colors ${
                  alreadyHas
                    ? "opacity-60 cursor-default"
                    : "hover:bg-ciab-bg-hover"
                }`}
              >
                <Layers className="w-3.5 h-3.5 text-ciab-text-muted flex-shrink-0" />
                <div className="flex-1 min-w-0">
                  <span className="text-xs font-medium text-ciab-text-primary truncate block">
                    {ws.name}
                  </span>
                  {ws.spec.agent?.provider && (
                    <span className="text-[9px] text-ciab-text-muted font-mono">
                      {ws.spec.agent.provider}
                    </span>
                  )}
                </div>
                {alreadyHas ? (
                  <Check className="w-3.5 h-3.5 text-state-running flex-shrink-0" />
                ) : (
                  <Plus className="w-3.5 h-3.5 text-ciab-copper flex-shrink-0" />
                )}
              </button>
            );
          })}
        </div>
      )}
    </div>
  );
}

function stripFrontmatter(content: string): string {
  const trimmed = content.trim();
  if (!trimmed.startsWith("---")) return trimmed;
  const endIdx = trimmed.indexOf("---", 3);
  if (endIdx === -1) return trimmed;
  return trimmed.slice(endIdx + 3).trim();
}
