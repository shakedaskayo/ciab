import { useState, useCallback } from "react";
import {
  ChevronDown,
  ChevronRight,
  Terminal,
  FileEdit,
  Search,
  FolderOpen,
  FileText,
  Globe,
  PenTool,
  Copy,
  Check,
  Loader2,
  ListTodo,
  Bot,
  Cpu,
} from "lucide-react";
import TodoListBlock, { parseTodoInput } from "./TodoListBlock";

interface Props {
  name: string;
  input: unknown;
  toolId?: string;
  isExecuting?: boolean;
  permissionStatus?: "approved" | "denied";
  agentName?: string;
}

const TOOL_ICONS: Record<string, typeof Terminal> = {
  Bash: Terminal,
  Edit: FileEdit,
  Write: PenTool,
  Read: FileText,
  Grep: Search,
  Glob: FolderOpen,
  WebFetch: Globe,
  WebSearch: Globe,
  TodoWrite: ListTodo,
  Agent: Bot,
  Task: Cpu,
};

const TOOL_COLORS: Record<string, string> = {
  Bash: "text-amber-400 bg-amber-400/10 border-amber-400/20",
  Edit: "text-blue-400 bg-blue-400/10 border-blue-400/20",
  Write: "text-violet-400 bg-violet-400/10 border-violet-400/20",
  Read: "text-emerald-400 bg-emerald-400/10 border-emerald-400/20",
  Grep: "text-orange-400 bg-orange-400/10 border-orange-400/20",
  Glob: "text-teal-400 bg-teal-400/10 border-teal-400/20",
  WebFetch: "text-sky-400 bg-sky-400/10 border-sky-400/20",
  WebSearch: "text-sky-400 bg-sky-400/10 border-sky-400/20",
};

const TOOL_COLORS_MAP: Record<string, string> = {
  ...TOOL_COLORS,
  TodoWrite: "text-ciab-copper bg-ciab-copper/10 border-ciab-copper/20",
  Agent: "text-violet-400 bg-violet-400/10 border-violet-400/20",
  Task: "text-indigo-400 bg-indigo-400/10 border-indigo-400/20",
};

const DEFAULT_COLOR = "text-ciab-steel-blue bg-ciab-steel-blue/10 border-ciab-steel-blue/20";

export default function ToolUseBlock({ name, input, toolId, isExecuting, permissionStatus, agentName }: Props) {
  const [expanded, setExpanded] = useState(name === "TodoWrite" || name === "Agent" || name === "Task");
  const [copied, setCopied] = useState(false);
  const Icon = TOOL_ICONS[name] ?? Terminal;
  const colorClass = TOOL_COLORS_MAP[name] ?? TOOL_COLORS[name] ?? DEFAULT_COLOR;
  const inputObj = (input && typeof input === "object" ? input : {}) as Record<
    string,
    unknown
  >;
  const todoItems = name === "TodoWrite" ? parseTodoInput(input) : null;

  const summary = getSummary(name, inputObj);
  const inputStr = JSON.stringify(input, null, 2);

  const handleCopy = useCallback(() => {
    navigator.clipboard.writeText(inputStr).then(() => {
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    });
  }, [inputStr]);

  return (
    <div className={`rounded-xl border max-w-[90%] overflow-hidden ${colorClass.split(" ").slice(1).join(" ")}`}>
      <button
        onClick={() => setExpanded(!expanded)}
        className="flex items-center gap-2 w-full px-3 py-2 text-left hover:bg-white/[0.02] transition-colors"
      >
        {/* Execution indicator */}
        {isExecuting ? (
          <Loader2 className={`w-3.5 h-3.5 animate-spin flex-shrink-0 ${colorClass.split(" ")[0]}`} />
        ) : expanded ? (
          <ChevronDown className="w-3 h-3 text-ciab-text-muted flex-shrink-0" />
        ) : (
          <ChevronRight className="w-3 h-3 text-ciab-text-muted flex-shrink-0" />
        )}

        {/* Tool icon */}
        <div className={`w-5 h-5 rounded flex items-center justify-center flex-shrink-0 ${colorClass.split(" ")[1]}`}>
          <Icon className={`w-3 h-3 ${colorClass.split(" ")[0]}`} />
        </div>

        {/* Agent name (for subagent tool calls) */}
        {agentName && (
          <span className="text-[9px] font-mono text-violet-400/70 bg-violet-400/10 px-1.5 py-0.5 rounded-full flex-shrink-0">
            {agentName}
          </span>
        )}

        {/* Tool name */}
        <span className={`text-xs font-mono font-semibold ${colorClass.split(" ")[0]}`}>
          {name}
        </span>

        {/* Summary */}
        {summary && (
          <span className="text-[10px] font-mono text-ciab-text-muted truncate ml-1 flex-1">
            {summary}
          </span>
        )}

        {/* Permission status badge */}
        {permissionStatus && (
          <span
            className={`text-[9px] font-mono font-medium px-1.5 py-0.5 rounded-full flex-shrink-0 ${
              permissionStatus === "approved"
                ? "bg-emerald-500/10 text-emerald-400"
                : "bg-red-500/10 text-red-400"
            }`}
          >
            {permissionStatus === "approved" ? "approved" : "denied"}
          </span>
        )}

        {/* Tool ID */}
        {toolId && (
          <span className="text-[9px] font-mono text-ciab-text-muted/40 ml-auto flex-shrink-0">
            {toolId.slice(0, 8)}
          </span>
        )}
      </button>

      {expanded && (
        <div className="border-t border-inherit animate-fade-in">
          {/* Bash */}
          {name === "Bash" && typeof inputObj.command === "string" && (
            <div className="px-3 py-2 bg-ciab-bg-primary/50">
              <div className="flex items-center gap-1.5 mb-1.5">
                <span className="text-[9px] font-mono text-ciab-text-muted uppercase tracking-wide">command</span>
              </div>
              <pre className="text-[11px] font-mono text-amber-300/90 whitespace-pre-wrap leading-relaxed">
                <span className="text-ciab-text-muted/40 select-none">$ </span>
                {inputObj.command as string}
              </pre>
            </div>
          )}

          {/* TodoWrite — render as structured todo list */}
          {name === "TodoWrite" && todoItems && (
            <div className="p-2">
              <TodoListBlock todos={todoItems} />
            </div>
          )}

          {/* Agent / Task — structured subagent view */}
          {(name === "Agent" || name === "Task") && (
            <AgentToolBody input={inputObj} isExecuting={isExecuting} />
          )}

          {name !== "Bash" && name !== "TodoWrite" && name !== "Agent" && name !== "Task" && (
            <div className="px-3 py-2 bg-ciab-bg-primary/50 relative group/input">
              <button
                onClick={handleCopy}
                className="absolute top-2 right-2 p-1 rounded text-ciab-text-muted hover:text-ciab-text-secondary
                  opacity-0 group-hover/input:opacity-100 transition-opacity"
              >
                {copied ? (
                  <Check className="w-3 h-3 text-state-running" />
                ) : (
                  <Copy className="w-3 h-3" />
                )}
              </button>
              <pre className="text-[11px] font-mono text-ciab-text-secondary whitespace-pre-wrap overflow-x-auto leading-relaxed">
                {inputStr}
              </pre>
            </div>
          )}
        </div>
      )}
    </div>
  );
}

/* ─── Subagent/Agent tool expanded body ─── */

function AgentToolBody({
  input,
  isExecuting,
}: {
  input: Record<string, unknown>;
  isExecuting?: boolean;
}) {
  const description = typeof input.description === "string" ? input.description : null;
  const subagentType = typeof input.subagent_type === "string" ? input.subagent_type : null;
  const prompt = typeof input.prompt === "string" ? input.prompt : null;
  const [promptExpanded, setPromptExpanded] = useState(false);

  const promptPreview = prompt && prompt.length > 200 ? prompt.slice(0, 200) + "…" : prompt;

  return (
    <div className="px-3 py-2.5 bg-ciab-bg-primary/50 space-y-2.5">
      {/* Header row: type badge + description */}
      <div className="flex items-start gap-2 flex-wrap">
        {subagentType && (
          <span className="text-[9px] font-mono font-semibold text-violet-300 bg-violet-500/15 border border-violet-500/20 px-2 py-0.5 rounded-full flex-shrink-0 uppercase tracking-wide">
            {subagentType}
          </span>
        )}
        {description && (
          <span className="text-[11px] font-mono text-ciab-text-secondary leading-snug">
            {description}
          </span>
        )}
      </div>

      {/* Prompt */}
      {prompt && (
        <div className="space-y-1">
          <div className="flex items-center gap-1.5">
            <span className="text-[9px] font-mono text-ciab-text-muted/60 uppercase tracking-wide">prompt</span>
            {isExecuting && (
              <span className="flex items-center gap-1 text-[9px] font-mono text-violet-400/70">
                <Loader2 className="w-2.5 h-2.5 animate-spin" />
                running
              </span>
            )}
          </div>
          <div
            className="rounded-lg border border-violet-500/10 bg-violet-500/[0.04] px-2.5 py-2 cursor-pointer hover:bg-violet-500/[0.07] transition-colors"
            onClick={() => setPromptExpanded((p) => !p)}
          >
            <pre className="text-[11px] font-mono text-ciab-text-secondary/80 whitespace-pre-wrap leading-relaxed">
              {promptExpanded ? prompt : promptPreview}
            </pre>
            {prompt.length > 200 && (
              <button className="mt-1.5 text-[9px] font-mono text-violet-400/60 hover:text-violet-400 transition-colors flex items-center gap-1">
                {promptExpanded ? (
                  <><ChevronDown className="w-2.5 h-2.5" /> Show less</>
                ) : (
                  <><ChevronRight className="w-2.5 h-2.5" /> Show full prompt ({prompt.length} chars)</>
                )}
              </button>
            )}
          </div>
        </div>
      )}
    </div>
  );
}

function getSummary(
  name: string,
  input: Record<string, unknown>
): string | null {
  switch (name) {
    case "Bash":
      return typeof input.command === "string"
        ? input.command.length > 80
          ? input.command.slice(0, 77) + "..."
          : input.command
        : null;
    case "Read":
      return typeof input.file_path === "string"
        ? input.file_path.split("/").slice(-2).join("/")
        : null;
    case "Edit":
    case "Write":
      return typeof input.file_path === "string"
        ? input.file_path.split("/").slice(-2).join("/")
        : null;
    case "Grep":
      return typeof input.pattern === "string"
        ? `/${input.pattern}/`
        : null;
    case "Glob":
      return typeof input.pattern === "string"
        ? input.pattern
        : null;
    case "WebFetch":
      return typeof input.url === "string"
        ? input.url
        : null;
    case "WebSearch":
      return typeof input.query === "string"
        ? input.query
        : null;
    case "TodoWrite": {
      const todos = input.todos as Array<Record<string, unknown>> | undefined;
      if (!Array.isArray(todos)) return null;
      const done = todos.filter((t) => t.status === "completed").length;
      return `${done}/${todos.length} tasks`;
    }
    case "Agent":
    case "Task": {
      const desc = typeof input.description === "string" ? input.description : null;
      const type = typeof input.subagent_type === "string" ? input.subagent_type : null;
      if (desc) return desc.length > 60 ? desc.slice(0, 57) + "..." : desc;
      if (type) return type;
      return null;
    }
    default:
      return null;
  }
}
