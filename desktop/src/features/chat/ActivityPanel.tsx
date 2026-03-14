import { useMemo } from "react";
import {
  FileEdit,
  FileText,
  Terminal,
  Search,
  FolderOpen,
  PenTool,
  Eye,
  Activity,
  ChevronRight,
} from "lucide-react";

export interface FileActivity {
  id: string;
  tool_name: string;
  file_path: string;
  action: string;
  timestamp: Date;
}

interface Props {
  activities: FileActivity[];
  isProcessing: boolean;
  activeTool: string | null;
}

const ACTION_ICONS: Record<string, typeof FileEdit> = {
  edited: FileEdit,
  written: PenTool,
  read: Eye,
  executed: Terminal,
  searched: Search,
  listed: FolderOpen,
  accessed: FileText,
};

const ACTION_COLORS: Record<string, string> = {
  edited: "text-amber-400",
  written: "text-emerald-400",
  read: "text-ciab-steel-blue",
  executed: "text-ciab-copper",
  searched: "text-violet-400",
  listed: "text-ciab-text-muted",
  accessed: "text-ciab-text-muted",
};

/** Extract just the filename from a full path */
function basename(path: string): string {
  const parts = path.split("/");
  return parts[parts.length - 1] || path;
}

/** Extract the directory from a full path */
function dirname(path: string): string {
  const idx = path.lastIndexOf("/");
  if (idx <= 0) return "";
  return path.substring(0, idx);
}

/** Group consecutive activities on the same file */
function groupActivities(activities: FileActivity[]): FileActivity[] {
  const seen = new Map<string, FileActivity>();
  // Show most recent unique file paths, preserving order
  for (const act of activities) {
    seen.set(act.file_path, act);
  }
  return Array.from(seen.values()).reverse();
}

export default function ActivityPanel({ activities, isProcessing, activeTool }: Props) {
  const grouped = useMemo(() => groupActivities(activities), [activities]);

  // Count by action type
  const stats = useMemo(() => {
    const counts: Record<string, number> = {};
    for (const act of activities) {
      counts[act.action] = (counts[act.action] || 0) + 1;
    }
    return counts;
  }, [activities]);

  if (activities.length === 0 && !isProcessing) {
    return (
      <div className="h-full flex flex-col items-center justify-center text-center px-4">
        <Activity className="w-6 h-6 text-ciab-text-muted/20 mb-2" />
        <p className="text-xs text-ciab-text-muted/40 font-mono">
          File activity will appear here as the agent works
        </p>
      </div>
    );
  }

  return (
    <div className="h-full flex flex-col">
      {/* Header */}
      <div className="flex items-center gap-2 px-3 py-2 border-b border-ciab-border flex-shrink-0">
        <Activity className="w-3.5 h-3.5 text-ciab-copper" />
        <span className="text-[11px] font-mono font-semibold text-ciab-text-primary">
          Activity
        </span>
        {isProcessing && (
          <span className="ml-auto flex items-center gap-1">
            <span className="w-1.5 h-1.5 rounded-full bg-ciab-copper animate-pulse" />
            <span className="text-[9px] font-mono text-ciab-copper">Live</span>
          </span>
        )}
        {!isProcessing && activities.length > 0 && (
          <span className="ml-auto text-[9px] font-mono text-ciab-text-muted/50">
            {activities.length} action{activities.length !== 1 ? "s" : ""}
          </span>
        )}
      </div>

      {/* Stats bar */}
      {Object.keys(stats).length > 0 && (
        <div className="flex items-center gap-2 px-3 py-1.5 border-b border-ciab-border/50 flex-shrink-0 overflow-x-auto scrollbar-none">
          {Object.entries(stats).map(([action, count]) => {
            const Icon = ACTION_ICONS[action] ?? FileText;
            const color = ACTION_COLORS[action] ?? "text-ciab-text-muted";
            return (
              <span
                key={action}
                className="flex items-center gap-1 text-[9px] font-mono whitespace-nowrap"
              >
                <Icon className={`w-2.5 h-2.5 ${color}`} />
                <span className={color}>{count}</span>
                <span className="text-ciab-text-muted/40">{action}</span>
              </span>
            );
          })}
        </div>
      )}

      {/* Active tool indicator */}
      {activeTool && (
        <div className="flex items-center gap-2 px-3 py-2 bg-ciab-copper/5 border-b border-ciab-copper/10 flex-shrink-0 animate-fade-in">
          <div className="flex items-center gap-0.5">
            <div className="w-1 h-1 rounded-full bg-ciab-copper animate-bounce" style={{ animationDelay: "0ms", animationDuration: "0.8s" }} />
            <div className="w-1 h-1 rounded-full bg-ciab-copper animate-bounce" style={{ animationDelay: "100ms", animationDuration: "0.8s" }} />
            <div className="w-1 h-1 rounded-full bg-ciab-copper animate-bounce" style={{ animationDelay: "200ms", animationDuration: "0.8s" }} />
          </div>
          <span className="text-[10px] font-mono text-ciab-copper font-medium">{activeTool}</span>
        </div>
      )}

      {/* File activity list */}
      <div className="flex-1 overflow-y-auto scrollbar-none">
        {grouped.map((act) => {
          const Icon = ACTION_ICONS[act.action] ?? FileText;
          const color = ACTION_COLORS[act.action] ?? "text-ciab-text-muted";
          const isFilePath = !act.file_path.startsWith("grep:") &&
                             !act.file_path.startsWith("glob:") &&
                             !act.file_path.includes("...");
          const dir = isFilePath ? dirname(act.file_path) : "";
          const name = isFilePath ? basename(act.file_path) : act.file_path;

          return (
            <div
              key={act.id}
              className="flex items-start gap-2 px-3 py-2 hover:bg-ciab-bg-hover/50 transition-colors border-b border-ciab-border/30 group"
            >
              <Icon className={`w-3.5 h-3.5 ${color} flex-shrink-0 mt-0.5`} />
              <div className="flex-1 min-w-0">
                <div className="flex items-center gap-1.5">
                  <span className="text-[11px] font-mono font-medium text-ciab-text-primary truncate">
                    {name}
                  </span>
                  <span className={`text-[9px] font-mono ${color}`}>
                    {act.action}
                  </span>
                </div>
                {dir && (
                  <p className="text-[9px] font-mono text-ciab-text-muted/50 truncate flex items-center gap-0.5">
                    <ChevronRight className="w-2 h-2 inline" />
                    {dir}
                  </p>
                )}
              </div>
            </div>
          );
        })}
      </div>
    </div>
  );
}
