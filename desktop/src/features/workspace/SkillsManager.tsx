import { useState, useCallback, useEffect } from "react";
import {
  Zap,
  Plus,
  X,
  Search,
  Trash2,
  Check,
  ToggleLeft,
  ToggleRight,
  Package,
  Loader2,
  Download,
  ExternalLink,
  ChevronDown,
  ChevronRight,
} from "lucide-react";
import type { WorkspaceSkill } from "@/lib/api/types";
import { useSkillSearch, useSkillMetadata } from "@/lib/hooks/use-skills";

interface Props {
  skills: WorkspaceSkill[];
  onUpdate: (skills: WorkspaceSkill[]) => void;
}

export default function SkillsManager({ skills, onUpdate }: Props) {
  const [showAddPanel, setShowAddPanel] = useState(false);
  const [expandedSkill, setExpandedSkill] = useState<number | null>(null);
  const [customSource, setCustomSource] = useState("");

  const handleToggle = useCallback(
    (index: number) => {
      const updated = [...skills];
      updated[index] = { ...updated[index], enabled: updated[index].enabled === false ? true : false };
      onUpdate(updated);
    },
    [skills, onUpdate]
  );

  const handleRemove = useCallback(
    (index: number) => {
      const updated = skills.filter((_, i) => i !== index);
      onUpdate(updated);
      if (expandedSkill === index) setExpandedSkill(null);
    },
    [skills, onUpdate, expandedSkill]
  );

  const handleAddSkill = useCallback(
    (source: string, name?: string) => {
      if (skills.some((s) => s.source === source)) return;
      const newSkill: WorkspaceSkill = {
        source,
        name,
        enabled: true,
      };
      onUpdate([...skills, newSkill]);
    },
    [skills, onUpdate]
  );

  const handleAddCustom = useCallback(() => {
    if (!customSource.trim()) return;
    handleAddSkill(customSource.trim());
    setCustomSource("");
  }, [customSource, handleAddSkill]);

  return (
    <div className="space-y-4">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-2">
          <Zap className="w-4 h-4 text-ciab-copper" />
          <span className="text-sm font-medium text-ciab-text-primary">
            {skills.length} {skills.length === 1 ? "Skill" : "Skills"} Configured
          </span>
        </div>
        <button
          onClick={() => setShowAddPanel(!showAddPanel)}
          className={`flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-medium transition-all ${
            showAddPanel
              ? "bg-ciab-copper/15 text-ciab-copper border border-ciab-copper/20"
              : "bg-ciab-bg-secondary text-ciab-text-secondary border border-ciab-border hover:border-ciab-copper/30"
          }`}
        >
          {showAddPanel ? <X className="w-3.5 h-3.5" /> : <Plus className="w-3.5 h-3.5" />}
          {showAddPanel ? "Close" : "Add Skill"}
        </button>
      </div>

      {/* Add Panel */}
      {showAddPanel && (
        <AddSkillPanel
          onAddSkill={handleAddSkill}
          onAddCustom={handleAddCustom}
          customSource={customSource}
          onCustomSourceChange={setCustomSource}
          installedSources={new Set(skills.map((s) => s.source))}
        />
      )}

      {/* Installed Skills List */}
      {skills.length === 0 ? (
        <div className="text-center py-12 border border-dashed border-ciab-border rounded-xl">
          <Package className="w-8 h-8 text-ciab-text-muted/20 mx-auto mb-3" />
          <p className="text-sm text-ciab-text-secondary">No skills configured</p>
          <p className="text-xs text-ciab-text-muted mt-1">
            Search the skills.sh registry or add a custom source
          </p>
          <button
            onClick={() => setShowAddPanel(true)}
            className="mt-3 inline-flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-medium bg-ciab-copper/10 text-ciab-copper border border-ciab-copper/20 hover:bg-ciab-copper/15 transition-colors"
          >
            <Plus className="w-3 h-3" />
            Add Skill
          </button>
        </div>
      ) : (
        <div className="space-y-2">
          {skills.map((skill, index) => (
            <InstalledSkillCard
              key={skill.source}
              skill={skill}
              isExpanded={expandedSkill === index}
              onToggle={() => handleToggle(index)}
              onRemove={() => handleRemove(index)}
              onToggleExpand={() =>
                setExpandedSkill(expandedSkill === index ? null : index)
              }
            />
          ))}
        </div>
      )}
    </div>
  );
}

function InstalledSkillCard({
  skill,
  isExpanded,
  onToggle,
  onRemove,
  onToggleExpand,
}: {
  skill: WorkspaceSkill;
  isExpanded: boolean;
  onToggle: () => void;
  onRemove: () => void;
  onToggleExpand: () => void;
}) {
  const isEnabled = skill.enabled !== false;
  // Parse source for metadata lookup
  const sourceParts = skill.source.split("/");
  const hasValidSource = sourceParts.length >= 2;
  const repoSource = hasValidSource ? `${sourceParts[0]}/${sourceParts[1]}` : undefined;
  const skillId = sourceParts.length >= 3 ? sourceParts.slice(2).join("/") : undefined;

  const { data: metadata, isLoading: metaLoading } = useSkillMetadata(
    isExpanded ? repoSource : undefined,
    isExpanded ? skillId : undefined
  );

  return (
    <div
      className={`rounded-xl border transition-all ${
        isEnabled
          ? "border-ciab-border bg-ciab-bg-card"
          : "border-ciab-border/50 bg-ciab-bg-card/50 opacity-70"
      }`}
    >
      <div className="flex items-center gap-3 p-4">
        {/* Icon */}
        <div className="w-9 h-9 rounded-lg bg-ciab-bg-primary border border-ciab-border flex items-center justify-center flex-shrink-0">
          <Zap className={`w-4 h-4 ${isEnabled ? "text-ciab-copper" : "text-ciab-text-muted"}`} />
        </div>

        {/* Info */}
        <div className="flex-1 min-w-0">
          <div className="flex items-center gap-2">
            <span className="text-sm font-medium text-ciab-text-primary truncate">
              {skill.name ?? skill.source.split("/").pop()}
            </span>
          </div>
          <div className="flex items-center gap-2 mt-0.5">
            <code className="text-[10px] font-mono text-ciab-text-muted truncate">{skill.source}</code>
            {skill.version && (
              <span className="text-[10px] font-mono text-ciab-text-muted/50">@{skill.version}</span>
            )}
          </div>
        </div>

        {/* Actions */}
        <div className="flex items-center gap-1">
          {hasValidSource && (
            <button
              onClick={onToggleExpand}
              className={`p-1.5 rounded-lg transition-colors ${
                isExpanded
                  ? "bg-ciab-copper/10 text-ciab-copper"
                  : "text-ciab-text-muted hover:text-ciab-text-secondary hover:bg-ciab-bg-hover"
              }`}
              title="View details"
            >
              {isExpanded ? <ChevronDown className="w-3.5 h-3.5" /> : <ChevronRight className="w-3.5 h-3.5" />}
            </button>
          )}

          <button
            onClick={onToggle}
            className={`p-1.5 rounded-lg transition-colors ${
              isEnabled
                ? "text-state-running hover:bg-state-running/10"
                : "text-ciab-text-muted hover:bg-ciab-bg-hover"
            }`}
            title={isEnabled ? "Disable" : "Enable"}
          >
            {isEnabled ? (
              <ToggleRight className="w-4 h-4" />
            ) : (
              <ToggleLeft className="w-4 h-4" />
            )}
          </button>

          <button
            onClick={onRemove}
            className="p-1.5 rounded-lg text-ciab-text-muted hover:text-state-failed hover:bg-state-failed/10 transition-colors"
            title="Remove"
          >
            <Trash2 className="w-3.5 h-3.5" />
          </button>
        </div>
      </div>

      {/* Expanded metadata panel */}
      {isExpanded && (
        <div className="border-t border-ciab-border p-4 bg-ciab-bg-primary/50 animate-fade-in">
          {metaLoading && (
            <div className="flex items-center gap-2 py-3 justify-center">
              <Loader2 className="w-3.5 h-3.5 text-ciab-copper animate-spin" />
              <span className="text-xs text-ciab-text-muted">Fetching metadata from GitHub...</span>
            </div>
          )}
          {metadata && (
            <div className="space-y-3">
              {metadata.description && (
                <p className="text-xs text-ciab-text-secondary leading-relaxed">{metadata.description}</p>
              )}
              {metadata.available_skills.length > 1 && (
                <div>
                  <p className="text-[9px] font-mono text-ciab-text-muted uppercase tracking-wider mb-1">
                    {metadata.available_skills.length} skills in repo
                  </p>
                  <div className="flex flex-wrap gap-1">
                    {metadata.available_skills.map((s) => (
                      <span key={s.path} className="px-2 py-0.5 rounded bg-ciab-bg-secondary border border-ciab-border text-[10px] font-mono text-ciab-text-muted">
                        {s.skill_id}
                      </span>
                    ))}
                  </div>
                </div>
              )}
              <a
                href={`https://github.com/${repoSource}`}
                target="_blank"
                rel="noopener noreferrer"
                className="inline-flex items-center gap-1 text-[10px] font-mono text-ciab-copper hover:underline"
              >
                View on GitHub <ExternalLink className="w-2.5 h-2.5" />
              </a>
            </div>
          )}
          {!metaLoading && !metadata?.description && !metadata?.available_skills.length && (
            <p className="text-xs text-ciab-text-muted text-center py-2">
              Could not fetch metadata for this skill
            </p>
          )}
        </div>
      )}
    </div>
  );
}

function AddSkillPanel({
  onAddSkill,
  onAddCustom,
  customSource,
  onCustomSourceChange,
  installedSources,
}: {
  onAddSkill: (source: string, name?: string) => void;
  onAddCustom: () => void;
  customSource: string;
  onCustomSourceChange: (s: string) => void;
  installedSources: Set<string>;
}) {
  const [searchQuery, setSearchQuery] = useState("");
  const [debouncedQuery, setDebouncedQuery] = useState("");
  const [showCustom, setShowCustom] = useState(false);

  useEffect(() => {
    const timer = setTimeout(() => setDebouncedQuery(searchQuery), 300);
    return () => clearTimeout(timer);
  }, [searchQuery]);

  const { data: searchData, isLoading, isFetching } = useSkillSearch(debouncedQuery, 12);
  const results = searchData?.skills ?? [];

  return (
    <div className="rounded-xl border border-ciab-copper/20 bg-ciab-bg-secondary overflow-hidden animate-fade-in">
      {/* Tabs */}
      <div className="flex items-center border-b border-ciab-border">
        <button
          onClick={() => setShowCustom(false)}
          className={`flex-1 px-4 py-2.5 text-xs font-medium text-center transition-colors border-b-2 ${
            !showCustom
              ? "border-ciab-copper text-ciab-copper"
              : "border-transparent text-ciab-text-muted hover:text-ciab-text-secondary"
          }`}
        >
          Search Registry
        </button>
        <button
          onClick={() => setShowCustom(true)}
          className={`flex-1 px-4 py-2.5 text-xs font-medium text-center transition-colors border-b-2 ${
            showCustom
              ? "border-ciab-copper text-ciab-copper"
              : "border-transparent text-ciab-text-muted hover:text-ciab-text-secondary"
          }`}
        >
          Custom Source
        </button>
      </div>

      {showCustom ? (
        <div className="p-4">
          <p className="text-xs text-ciab-text-muted mb-3">
            Add a skill by its source reference. Use <code className="text-ciab-copper">owner/repo</code> format
            for GitHub repos, or <code className="text-ciab-copper">owner/repo/skill-id</code> for a specific skill within a repo.
          </p>
          <div className="flex items-center gap-2">
            <input
              type="text"
              value={customSource}
              onChange={(e) => onCustomSourceChange(e.target.value)}
              placeholder="e.g. vercel-labs/agent-skills/react-best-practices"
              className="flex-1 bg-ciab-bg-primary border border-ciab-border rounded-lg px-3 py-2 text-xs font-mono text-ciab-text-primary placeholder:text-ciab-text-muted/40 outline-none focus:border-ciab-copper/40"
              onKeyDown={(e) => e.key === "Enter" && onAddCustom()}
            />
            <button
              onClick={onAddCustom}
              disabled={!customSource.trim()}
              className="px-3 py-2 rounded-lg text-xs font-medium bg-ciab-copper text-white hover:bg-ciab-copper-hover disabled:opacity-40 disabled:cursor-not-allowed transition-colors"
            >
              Add
            </button>
          </div>
        </div>
      ) : (
        <div className="p-4 space-y-3">
          {/* Search */}
          <div className="flex items-center gap-2 bg-ciab-bg-primary border border-ciab-border rounded-lg px-3 py-1.5">
            <Search className="w-3 h-3 text-ciab-text-muted" />
            <input
              type="text"
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              placeholder="Search skills.sh registry..."
              className="flex-1 bg-transparent border-none outline-none text-xs text-ciab-text-primary placeholder:text-ciab-text-muted/40"
              autoFocus
            />
            {isFetching && <Loader2 className="w-3 h-3 text-ciab-copper animate-spin" />}
          </div>

          {/* Results */}
          <div className="max-h-[300px] overflow-auto space-y-1">
            {searchQuery.length < 2 && (
              <p className="text-[10px] text-ciab-text-muted text-center py-4">
                Type at least 2 characters to search the skills.sh registry
              </p>
            )}

            {searchQuery.length >= 2 && isLoading && (
              <div className="flex items-center justify-center gap-2 py-6">
                <Loader2 className="w-4 h-4 text-ciab-copper animate-spin" />
                <span className="text-xs text-ciab-text-muted">Searching...</span>
              </div>
            )}

            {searchQuery.length >= 2 && !isLoading && results.length === 0 && (
              <p className="text-xs text-ciab-text-muted text-center py-4">
                No skills found for "{debouncedQuery}"
              </p>
            )}

            {results.map((skill) => {
              // Use full id (source/skillId) as the installed source
              const fullSource = skill.id;
              const installed = installedSources.has(fullSource) || installedSources.has(skill.source);
              return (
                <button
                  key={skill.id}
                  onClick={() => !installed && onAddSkill(fullSource, skill.name)}
                  disabled={installed}
                  className={`w-full text-left flex items-center gap-3 p-2.5 rounded-lg transition-colors ${
                    installed
                      ? "opacity-50 cursor-not-allowed bg-ciab-bg-primary/50"
                      : "hover:bg-ciab-bg-hover"
                  }`}
                >
                  <Zap className="w-3.5 h-3.5 text-ciab-copper/50 flex-shrink-0" />
                  <div className="flex-1 min-w-0">
                    <div className="flex items-center gap-1.5">
                      <span className="text-xs font-medium text-ciab-text-primary truncate">
                        {skill.name}
                      </span>
                      <span className="text-[9px] font-mono text-ciab-text-muted/50 flex items-center gap-0.5 flex-shrink-0">
                        <Download className="w-2 h-2" />
                        {formatInstalls(skill.installs)}
                      </span>
                    </div>
                    <p className="text-[10px] text-ciab-text-muted truncate font-mono">{skill.source}</p>
                  </div>
                  {installed ? (
                    <Check className="w-3.5 h-3.5 text-state-running flex-shrink-0" />
                  ) : (
                    <Plus className="w-3.5 h-3.5 text-ciab-copper flex-shrink-0" />
                  )}
                </button>
              );
            })}
          </div>
        </div>
      )}
    </div>
  );
}

function formatInstalls(count: number): string {
  if (count >= 1_000_000) return `${(count / 1_000_000).toFixed(1)}M`;
  if (count >= 1_000) return `${(count / 1_000).toFixed(1)}K`;
  return String(count);
}
