import { useState, useCallback, useEffect, useMemo } from "react";
import {
  X,
  Search,
  Zap,
  Check,
  ExternalLink,
  TrendingUp,
  Package,
  Loader2,
} from "lucide-react";
import { useSkillSearch, useTrendingSkills } from "@/lib/hooks/use-skills";
import type { SessionSkill } from "@/lib/hooks/use-sessions";
import type { WorkspaceSkill } from "@/lib/api/types";

interface Props {
  isOpen: boolean;
  onClose: () => void;
  activeSkills: SessionSkill[];
  workspaceSkills?: WorkspaceSkill[];
  onSave: (skills: SessionSkill[]) => void;
}

const QUICK_SEARCHES = [
  "react",
  "typescript",
  "python",
  "rust",
  "testing",
  "docker",
  "security",
  "api",
  "nextjs",
  "best-practices",
];

export default function SkillPicker({
  isOpen,
  onClose,
  activeSkills,
  workspaceSkills,
  onSave,
}: Props) {
  const [searchQuery, setSearchQuery] = useState("");
  const [debouncedQuery, setDebouncedQuery] = useState("");
  const [selected, setSelected] = useState<SessionSkill[]>(activeSkills);
  const [activeTab, setActiveTab] = useState<"search" | "workspace">(
    workspaceSkills?.length ? "workspace" : "search"
  );

  // Debounce search
  useEffect(() => {
    const timer = setTimeout(() => setDebouncedQuery(searchQuery), 300);
    return () => clearTimeout(timer);
  }, [searchQuery]);

  // Reset state when opening
  useEffect(() => {
    if (isOpen) {
      setSelected(activeSkills);
      setSearchQuery("");
      setDebouncedQuery("");
    }
  }, [isOpen, activeSkills]);

  const { data: searchResults, isLoading: isSearching } =
    useSkillSearch(debouncedQuery);
  const { data: trending, isLoading: isTrendingLoading } = useTrendingSkills();

  // Deduplicate results by source (trending can return duplicates from multiple queries)
  const rawResults = debouncedQuery.length >= 2 ? searchResults : trending;
  const results = useMemo(() => {
    if (!rawResults?.skills) return rawResults;
    const seen = new Set<string>();
    const deduped = rawResults.skills.filter((s) => {
      if (seen.has(s.source)) return false;
      seen.add(s.source);
      return true;
    });
    return { ...rawResults, skills: deduped };
  }, [rawResults]);
  const isLoading = debouncedQuery.length >= 2 ? isSearching : isTrendingLoading;

  const isSelected = useCallback(
    (source: string) => selected.some((s) => s.source === source),
    [selected]
  );

  const toggleSkill = useCallback(
    (source: string, name?: string) => {
      setSelected((prev) => {
        const exists = prev.some((s) => s.source === source);
        if (exists) {
          return prev.filter((s) => s.source !== source);
        }
        return [...prev, { source, name }];
      });
    },
    []
  );

  const handleSave = useCallback(() => {
    onSave(selected);
    onClose();
  }, [selected, onSave, onClose]);

  // Close on Escape
  useEffect(() => {
    if (!isOpen) return;
    const handler = (e: KeyboardEvent) => {
      if (e.key === "Escape") onClose();
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [isOpen, onClose]);

  const hasChanges = useMemo(() => {
    if (selected.length !== activeSkills.length) return true;
    return selected.some(
      (s) => !activeSkills.some((a) => a.source === s.source)
    );
  }, [selected, activeSkills]);

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center">
      {/* Backdrop */}
      <div
        className="absolute inset-0 bg-black/50 backdrop-blur-sm"
        onClick={onClose}
      />

      {/* Dialog */}
      <div className="relative w-full max-w-lg mx-4 bg-ciab-bg-card border border-ciab-border rounded-2xl shadow-2xl shadow-black/40 overflow-hidden animate-scale-in">
        {/* Header */}
        <div className="flex items-center justify-between px-5 py-4 border-b border-ciab-border">
          <div className="flex items-center gap-2.5">
            <div className="flex items-center justify-center w-8 h-8 rounded-xl bg-ciab-copper/10">
              <Zap className="w-4 h-4 text-ciab-copper" />
            </div>
            <div>
              <h2 className="text-sm font-semibold text-ciab-text-primary">
                Agent Skills
              </h2>
              <p className="text-[11px] text-ciab-text-muted">
                Attach skills to enhance this session
              </p>
            </div>
          </div>
          <button
            onClick={onClose}
            className="p-1.5 rounded-lg hover:bg-ciab-bg-hover transition-colors"
          >
            <X className="w-4 h-4 text-ciab-text-muted" />
          </button>
        </div>

        {/* Tabs */}
        {workspaceSkills && workspaceSkills.length > 0 && (
          <div className="flex border-b border-ciab-border">
            <button
              onClick={() => setActiveTab("search")}
              className={`flex-1 px-4 py-2.5 text-xs font-medium transition-colors ${
                activeTab === "search"
                  ? "text-ciab-copper border-b-2 border-ciab-copper"
                  : "text-ciab-text-muted hover:text-ciab-text-secondary"
              }`}
            >
              <Search className="w-3 h-3 inline mr-1.5 -mt-0.5" />
              Search Registry
            </button>
            <button
              onClick={() => setActiveTab("workspace")}
              className={`flex-1 px-4 py-2.5 text-xs font-medium transition-colors ${
                activeTab === "workspace"
                  ? "text-ciab-copper border-b-2 border-ciab-copper"
                  : "text-ciab-text-muted hover:text-ciab-text-secondary"
              }`}
            >
              <Package className="w-3 h-3 inline mr-1.5 -mt-0.5" />
              Workspace Skills
              <span className="ml-1.5 px-1.5 py-0.5 rounded-full bg-ciab-bg-elevated text-[10px] font-mono">
                {workspaceSkills.length}
              </span>
            </button>
          </div>
        )}

        {/* Content */}
        <div className="max-h-[400px] overflow-y-auto">
          {activeTab === "search" && (
            <div>
              {/* Search input */}
              <div className="px-4 py-3 border-b border-ciab-border/50">
                <div className="relative">
                  <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-3.5 h-3.5 text-ciab-text-muted" />
                  <input
                    type="text"
                    value={searchQuery}
                    onChange={(e) => setSearchQuery(e.target.value)}
                    placeholder="Search skills..."
                    autoFocus
                    className="w-full pl-9 pr-3 py-2 text-sm bg-ciab-bg-elevated border border-ciab-border rounded-lg
                      text-ciab-text-primary placeholder:text-ciab-text-muted/50 focus:outline-none
                      focus:ring-1 focus:ring-ciab-copper/30 focus:border-ciab-copper/40"
                  />
                </div>

                {/* Quick search tags */}
                {!searchQuery && (
                  <div className="flex flex-wrap gap-1.5 mt-2.5">
                    {QUICK_SEARCHES.map((tag) => (
                      <button
                        key={tag}
                        onClick={() => setSearchQuery(tag)}
                        className="px-2 py-1 text-[10px] font-mono rounded-md bg-ciab-bg-elevated
                          border border-ciab-border/50 text-ciab-text-muted hover:text-ciab-text-secondary
                          hover:border-ciab-copper/30 transition-colors"
                      >
                        {tag}
                      </button>
                    ))}
                  </div>
                )}
              </div>

              {/* Results */}
              <div className="py-1">
                {isLoading && (
                  <div className="flex items-center justify-center py-8 text-ciab-text-muted">
                    <Loader2 className="w-4 h-4 animate-spin mr-2" />
                    <span className="text-xs">
                      {debouncedQuery ? "Searching..." : "Loading trending..."}
                    </span>
                  </div>
                )}

                {!isLoading && !debouncedQuery && (
                  <div className="flex items-center gap-1.5 px-4 py-2">
                    <TrendingUp className="w-3 h-3 text-ciab-text-muted/40" />
                    <span className="text-[10px] font-semibold tracking-widest text-ciab-text-muted/40 uppercase">
                      Trending
                    </span>
                  </div>
                )}

                {results?.skills?.map((skill) => (
                  <SkillRow
                    key={skill.id || skill.skillId}
                    source={skill.source}
                    name={skill.name}
                    installs={skill.installs}
                    isActive={isSelected(skill.source)}
                    onToggle={() =>
                      toggleSkill(skill.source, skill.name)
                    }
                  />
                ))}

                {!isLoading &&
                  debouncedQuery.length >= 2 &&
                  !results?.skills?.length && (
                    <div className="px-4 py-8 text-center text-xs text-ciab-text-muted">
                      No skills found for "{debouncedQuery}"
                    </div>
                  )}
              </div>
            </div>
          )}

          {activeTab === "workspace" && workspaceSkills && (
            <div className="py-1">
              {workspaceSkills.map((ws) => (
                <SkillRow
                  key={ws.source}
                  source={ws.source}
                  name={ws.name}
                  isActive={isSelected(ws.source)}
                  onToggle={() =>
                    toggleSkill(ws.source, ws.name)
                  }
                />
              ))}
            </div>
          )}
        </div>

        {/* Footer */}
        <div className="flex items-center justify-between px-5 py-3.5 border-t border-ciab-border bg-ciab-bg-secondary/30">
          <span className="text-[11px] text-ciab-text-muted">
            {selected.length} skill{selected.length !== 1 ? "s" : ""} selected
          </span>
          <div className="flex items-center gap-2">
            <button
              onClick={onClose}
              className="px-3 py-1.5 text-xs font-medium text-ciab-text-muted hover:text-ciab-text-secondary
                bg-ciab-bg-elevated border border-ciab-border rounded-lg hover:bg-ciab-bg-hover transition-colors"
            >
              Cancel
            </button>
            <button
              onClick={handleSave}
              disabled={!hasChanges}
              className="px-4 py-1.5 text-xs font-medium text-white bg-ciab-copper hover:bg-ciab-copper/90
                rounded-lg transition-colors disabled:opacity-40 disabled:cursor-not-allowed"
            >
              Save
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}

function formatInstalls(n?: number): string {
  if (!n) return "";
  if (n >= 1_000_000) return `${(n / 1_000_000).toFixed(1)}M`;
  if (n >= 1_000) return `${(n / 1_000).toFixed(1)}K`;
  return n.toString();
}

function SkillRow({
  source,
  name,
  installs,
  isActive,
  onToggle,
}: {
  source: string;
  name?: string | null;
  installs?: number;
  isActive: boolean;
  onToggle: () => void;
}) {
  return (
    <button
      onClick={onToggle}
      className={`group w-full flex items-center gap-3 px-4 py-2.5 text-left transition-all duration-100
        ${
          isActive
            ? "bg-ciab-copper/8 border-l-2 border-l-ciab-copper"
            : "border-l-2 border-l-transparent hover:bg-ciab-bg-hover/60"
        }`}
    >
      {/* Checkbox */}
      <div
        className={`flex-shrink-0 w-4 h-4 rounded border transition-all ${
          isActive
            ? "bg-ciab-copper border-ciab-copper"
            : "border-ciab-border group-hover:border-ciab-text-muted"
        } flex items-center justify-center`}
      >
        {isActive && <Check className="w-3 h-3 text-white" />}
      </div>

      {/* Skill info */}
      <div className="flex-1 min-w-0">
        <div className="flex items-center gap-2">
          <span
            className={`text-sm font-medium truncate ${
              isActive ? "text-ciab-copper" : "text-ciab-text-primary"
            }`}
          >
            {name || source.split("/").pop()}
          </span>
        </div>
        <span className="text-[11px] text-ciab-text-muted font-mono truncate block">
          {source}
        </span>
      </div>

      {/* Right side */}
      <div className="flex items-center gap-2 flex-shrink-0">
        {installs != null && installs > 0 && (
          <span className="text-[10px] font-mono text-ciab-text-muted/60">
            {formatInstalls(installs)}
          </span>
        )}
        <a
          href={`https://github.com/${source}`}
          target="_blank"
          rel="noopener noreferrer"
          onClick={(e) => e.stopPropagation()}
          className="opacity-0 group-hover:opacity-100 transition-opacity p-1 rounded hover:bg-ciab-bg-elevated"
        >
          <ExternalLink className="w-3 h-3 text-ciab-text-muted" />
        </a>
      </div>
    </button>
  );
}
