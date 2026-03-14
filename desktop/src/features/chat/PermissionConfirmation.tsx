import { useState } from "react";
import {
  Terminal,
  FileEdit,
  FileText,
  Search,
  FolderOpen,
  Globe,
  PenTool,
  ChevronDown,
  ChevronRight,
  Check,
  X,
  ShieldAlert,
  ShieldCheck,
  ShieldPlus,
} from "lucide-react";
import type { RiskLevel, PermissionMode } from "@/lib/api/types";

const TOOL_ICONS: Record<string, typeof Terminal> = {
  Bash: Terminal,
  Edit: FileEdit,
  Write: PenTool,
  Read: FileText,
  Grep: Search,
  Glob: FolderOpen,
  WebFetch: Globe,
  WebSearch: Globe,
};

const RISK_STYLES: Record<RiskLevel, { border: string; badge: string; icon: string }> = {
  low: {
    border: "border-emerald-500/30",
    badge: "bg-emerald-500/10 text-emerald-400",
    icon: "text-emerald-400",
  },
  medium: {
    border: "border-amber-500/30",
    badge: "bg-amber-500/10 text-amber-400",
    icon: "text-amber-400",
  },
  high: {
    border: "border-red-500/30",
    badge: "bg-red-500/10 text-red-400",
    icon: "text-red-400",
  },
};

interface Props {
  requestId: string;
  toolName: string;
  toolInput: unknown;
  riskLevel: RiskLevel;
  status?: "approved" | "denied";
  onApprove: (requestId: string) => void;
  onDeny: (requestId: string) => void;
  onAllowTool?: (toolName: string) => void;
  onSwitchMode?: (mode: PermissionMode) => void;
}

/** Extract a human-readable summary from tool input */
function getToolSummary(toolName: string, input: unknown): string | null {
  if (!input || typeof input !== "object") return null;
  const obj = input as Record<string, unknown>;

  switch (toolName) {
    case "Bash": {
      const cmd = obj.command as string | undefined;
      return cmd ? `$ ${cmd.length > 100 ? cmd.slice(0, 97) + "..." : cmd}` : null;
    }
    case "Edit":
    case "MultiEdit":
    case "Write":
    case "NotebookEdit":
    case "Read": {
      const fp = (obj.file_path ?? obj.path) as string | undefined;
      return fp ?? null;
    }
    case "Grep": {
      const pattern = obj.pattern as string | undefined;
      const path = obj.path as string | undefined;
      return pattern ? `/${pattern}/${path ? ` in ${path}` : ""}` : null;
    }
    case "Glob": {
      const pattern = obj.pattern as string | undefined;
      return pattern ?? null;
    }
    default:
      return null;
  }
}

export default function PermissionConfirmation({
  requestId,
  toolName,
  toolInput,
  riskLevel,
  status,
  onApprove,
  onDeny,
  onAllowTool,
  onSwitchMode,
}: Props) {
  const [expanded, setExpanded] = useState(false);
  const Icon = TOOL_ICONS[toolName] ?? Terminal;
  const risk = RISK_STYLES[riskLevel];
  const inputStr = JSON.stringify(toolInput, null, 2);
  const resolved = status != null;
  const summary = getToolSummary(toolName, toolInput);

  return (
    <div
      className={`rounded-xl border-2 ${risk.border} bg-ciab-bg-card overflow-hidden animate-fade-in max-w-[90%]`}
    >
      {/* Header */}
      <div className="flex items-center gap-2.5 px-3.5 py-2.5">
        <ShieldAlert className={`w-4 h-4 flex-shrink-0 ${risk.icon}`} />

        <div className="flex items-center gap-2 flex-1 min-w-0">
          <Icon className={`w-3.5 h-3.5 ${risk.icon}`} />
          <span className="text-xs font-mono font-semibold text-ciab-text-primary">
            {toolName}
          </span>
          <span className={`text-[9px] font-mono px-1.5 py-0.5 rounded-full ${risk.badge}`}>
            {riskLevel}
          </span>
        </div>

        {/* Resolved badge */}
        {resolved && (
          <span
            className={`text-[10px] font-mono font-medium px-2 py-0.5 rounded-full ${
              status === "approved"
                ? "bg-emerald-500/10 text-emerald-400"
                : "bg-red-500/10 text-red-400"
            }`}
          >
            {status === "approved" ? "Approved" : "Denied"}
          </span>
        )}

        {/* Expand toggle */}
        <button
          onClick={() => setExpanded(!expanded)}
          className="p-1 rounded hover:bg-ciab-bg-hover transition-colors text-ciab-text-muted"
        >
          {expanded ? (
            <ChevronDown className="w-3 h-3" />
          ) : (
            <ChevronRight className="w-3 h-3" />
          )}
        </button>
      </div>

      {/* Tool summary (always visible when not expanded) */}
      {summary && !expanded && (
        <div className="px-3.5 pb-2 -mt-1">
          <p className="text-[11px] font-mono text-ciab-text-muted/70 truncate">
            {summary}
          </p>
        </div>
      )}

      {/* Expanded input details */}
      {expanded && (
        <div className="border-t border-ciab-border px-3.5 py-2 bg-ciab-bg-primary/50">
          <pre className="text-[11px] font-mono text-ciab-text-secondary whitespace-pre-wrap overflow-x-auto leading-relaxed max-h-[200px] overflow-y-auto">
            {inputStr}
          </pre>
        </div>
      )}

      {/* Action buttons — three approval options */}
      {!resolved && (
        <div className="border-t border-ciab-border bg-ciab-bg-secondary/30">
          {/* Main actions row */}
          <div className="flex items-center gap-2 px-3.5 py-2.5">
            <span className="text-[10px] text-ciab-text-muted font-mono flex-1">
              Allow this tool execution?
            </span>
            <button
              onClick={() => onDeny(requestId)}
              className="flex items-center gap-1 px-3 py-1.5 rounded-lg text-[11px] font-mono font-medium
                bg-red-500/10 text-red-400 border border-red-500/20
                hover:bg-red-500/20 transition-all"
            >
              <X className="w-3 h-3" />
              Deny
            </button>
            <button
              onClick={() => onApprove(requestId)}
              className="flex items-center gap-1 px-3 py-1.5 rounded-lg text-[11px] font-mono font-medium
                bg-emerald-500/10 text-emerald-400 border border-emerald-500/20
                hover:bg-emerald-500/20 transition-all"
            >
              <Check className="w-3 h-3" />
              Approve
            </button>
          </div>

          {/* Quick-approve options row */}
          <div className="flex items-center gap-2 px-3.5 pb-2.5 -mt-0.5">
            {onAllowTool && (
              <button
                onClick={() => {
                  onAllowTool(toolName);
                  onApprove(requestId);
                }}
                className="flex items-center gap-1.5 px-2.5 py-1 rounded-lg text-[10px] font-mono
                  text-ciab-steel-blue/80 hover:text-ciab-steel-blue
                  bg-ciab-steel-blue/5 hover:bg-ciab-steel-blue/10
                  border border-ciab-steel-blue/15 hover:border-ciab-steel-blue/25
                  transition-all"
                title={`Always allow ${toolName} for this session`}
              >
                <ShieldCheck className="w-3 h-3" />
                Allow {toolName} for session
              </button>
            )}
            {onSwitchMode && (
              <button
                onClick={() => {
                  onSwitchMode("auto_approve");
                  onApprove(requestId);
                }}
                className="flex items-center gap-1.5 px-2.5 py-1 rounded-lg text-[10px] font-mono
                  text-ciab-text-muted/60 hover:text-ciab-copper
                  hover:bg-ciab-copper/5
                  border border-transparent hover:border-ciab-copper/15
                  transition-all"
                title="Switch to auto-approve all tools"
              >
                <ShieldPlus className="w-3 h-3" />
                Auto-approve all
              </button>
            )}
          </div>
        </div>
      )}

      {/* Recovery actions when denied */}
      {status === "denied" && onAllowTool && onSwitchMode && (
        <div className="flex flex-col gap-2 px-3.5 py-2.5 border-t border-ciab-border bg-ciab-bg-secondary/30">
          <span className="text-[10px] text-ciab-text-muted font-mono">
            Tool was denied. Allow this tool or change mode, then resend your message.
          </span>
          <div className="flex items-center gap-2 flex-wrap">
            <button
              onClick={() => onAllowTool(toolName)}
              className="flex items-center gap-1 px-3 py-1.5 rounded-lg text-[11px] font-mono font-medium
                bg-ciab-steel-blue/10 text-ciab-steel-blue border border-ciab-steel-blue/20
                hover:bg-ciab-steel-blue/20 transition-all"
            >
              <ShieldCheck className="w-3 h-3" />
              Allow {toolName}
            </button>
            <button
              onClick={() => onSwitchMode("auto_approve")}
              className="text-[10px] font-mono text-ciab-text-muted hover:text-ciab-copper transition-colors underline"
            >
              Switch to Auto
            </button>
            <button
              onClick={() => onSwitchMode("unrestricted")}
              className="text-[10px] font-mono text-ciab-text-muted hover:text-red-400 transition-colors underline"
            >
              Switch to Full Access
            </button>
          </div>
        </div>
      )}
    </div>
  );
}
