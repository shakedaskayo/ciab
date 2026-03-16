import { useCallback, useRef, useEffect, useState, useMemo } from "react";
import { useSandboxStream } from "@/lib/hooks/use-stream";
import type { StreamEvent } from "@/lib/api/types";
import {
  ScrollText,
  Filter,
  Trash2,
  Download,
  Search,
  AlertCircle,
  CheckCircle2,
  ArrowRight,
  Radio,
  X,
  Cpu,
  Wrench,
  Layers,
  GitBranch,
  ChevronDown,
  ChevronRight,
} from "lucide-react";
import EmptyState from "@/components/shared/EmptyState";

interface Props {
  sandboxId: string;
}

const EVENT_TYPE_CONFIG: Record<
  string,
  { label: string; color: string; icon: typeof AlertCircle }
> = {
  log_line: { label: "LOG", color: "text-ciab-text-primary", icon: ScrollText },
  provisioning_step: { label: "PROVISION", color: "text-ciab-steel-blue", icon: ArrowRight },
  provisioning_complete: { label: "COMPLETE", color: "text-state-running", icon: CheckCircle2 },
  provisioning_failed: { label: "FAILED", color: "text-state-failed", icon: AlertCircle },
  sandbox_state_changed: { label: "STATE", color: "text-state-paused", icon: Radio },
  error: { label: "ERROR", color: "text-state-failed", icon: AlertCircle },
};

const LOG_EVENT_TYPES = new Set(Object.keys(EVENT_TYPE_CONFIG));

export default function LogViewer({ sandboxId }: Props) {
  const scrollRef = useRef<HTMLDivElement>(null);
  const [autoScroll, setAutoScroll] = useState(true);
  const [filterText, setFilterText] = useState("");
  const [showFilter, setShowFilter] = useState(false);
  const [activeFilters, setActiveFilters] = useState<Set<string>>(new Set());

  const handleEvent = useCallback(() => {
    if (autoScroll) {
      requestAnimationFrame(() => {
        scrollRef.current?.scrollTo({ top: scrollRef.current.scrollHeight });
      });
    }
  }, [autoScroll]);

  const { events, clear } = useSandboxStream(sandboxId, handleEvent);

  const logEvents = useMemo(() => {
    let filtered = events.filter((e) => LOG_EVENT_TYPES.has(e.event_type));

    if (activeFilters.size > 0) {
      filtered = filtered.filter((e) => activeFilters.has(e.event_type));
    }

    if (filterText) {
      const lower = filterText.toLowerCase();
      filtered = filtered.filter((e) => {
        const text = getLogText(e);
        return text.toLowerCase().includes(lower);
      });
    }

    return filtered;
  }, [events, activeFilters, filterText]);

  // Detect scroll position for auto-scroll toggle
  useEffect(() => {
    const el = scrollRef.current;
    if (!el) return;
    const handler = () => {
      const distFromBottom = el.scrollHeight - el.scrollTop - el.clientHeight;
      setAutoScroll(distFromBottom < 50);
    };
    el.addEventListener("scroll", handler, { passive: true });
    return () => el.removeEventListener("scroll", handler);
  }, []);

  useEffect(() => {
    if (autoScroll) {
      scrollRef.current?.scrollTo({ top: scrollRef.current.scrollHeight });
    }
  }, [logEvents.length, autoScroll]);

  const toggleFilter = useCallback((eventType: string) => {
    setActiveFilters((prev) => {
      const next = new Set(prev);
      if (next.has(eventType)) {
        next.delete(eventType);
      } else {
        next.add(eventType);
      }
      return next;
    });
  }, []);

  const handleExport = useCallback(() => {
    const text = logEvents
      .map((e) => {
        const time = new Date(e.timestamp).toISOString();
        return `[${time}] [${e.event_type}] ${getLogText(e)}`;
      })
      .join("\n");
    const blob = new Blob([text], { type: "text/plain" });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = `ciab-logs-${sandboxId.slice(0, 8)}.txt`;
    a.click();
    URL.revokeObjectURL(url);
  }, [logEvents, sandboxId]);

  if (events.filter((e) => LOG_EVENT_TYPES.has(e.event_type)).length === 0) {
    return (
      <EmptyState
        icon={ScrollText}
        title="No logs yet"
        description="Logs will stream here in real-time as the sandbox produces output."
      />
    );
  }

  return (
    <div className="flex flex-col h-full">
      {/* Toolbar */}
      <div className="flex items-center justify-between mb-2 flex-shrink-0">
        <div className="flex items-center gap-2">
          <div className="flex items-center gap-1">
            <span className="w-2 h-2 rounded-full bg-state-running animate-pulse" />
            <span className="text-[10px] font-mono text-ciab-text-muted uppercase tracking-wider">
              Live
            </span>
          </div>
          <span className="text-[10px] font-mono text-ciab-text-muted/50">
            {logEvents.length} entries
          </span>
        </div>

        <div className="flex items-center gap-1">
          {/* Search toggle */}
          <button
            onClick={() => setShowFilter(!showFilter)}
            className={`p-1.5 rounded-lg transition-colors ${
              showFilter || filterText
                ? "bg-ciab-copper/10 text-ciab-copper"
                : "text-ciab-text-muted hover:text-ciab-text-secondary hover:bg-ciab-bg-hover"
            }`}
            title="Filter logs"
          >
            <Search className="w-3.5 h-3.5" />
          </button>

          {/* Type filter */}
          <button
            onClick={() => {
              if (activeFilters.size > 0) {
                setActiveFilters(new Set());
              } else {
                setShowFilter(!showFilter);
              }
            }}
            className={`p-1.5 rounded-lg transition-colors ${
              activeFilters.size > 0
                ? "bg-ciab-copper/10 text-ciab-copper"
                : "text-ciab-text-muted hover:text-ciab-text-secondary hover:bg-ciab-bg-hover"
            }`}
            title="Filter by type"
          >
            <Filter className="w-3.5 h-3.5" />
          </button>

          {/* Export */}
          <button
            onClick={handleExport}
            className="p-1.5 rounded-lg text-ciab-text-muted hover:text-ciab-text-secondary hover:bg-ciab-bg-hover transition-colors"
            title="Export logs"
          >
            <Download className="w-3.5 h-3.5" />
          </button>

          {/* Clear */}
          <button
            onClick={clear}
            className="p-1.5 rounded-lg text-ciab-text-muted hover:text-ciab-text-secondary hover:bg-ciab-bg-hover transition-colors"
            title="Clear logs"
          >
            <Trash2 className="w-3.5 h-3.5" />
          </button>
        </div>
      </div>

      {/* Filter bar */}
      {showFilter && (
        <div className="flex items-center gap-2 mb-2 animate-fade-in flex-shrink-0">
          <div className="flex-1 flex items-center gap-2 bg-ciab-bg-secondary border border-ciab-border rounded-lg px-3 py-1.5">
            <Search className="w-3 h-3 text-ciab-text-muted" />
            <input
              type="text"
              value={filterText}
              onChange={(e) => setFilterText(e.target.value)}
              placeholder="Filter logs..."
              className="flex-1 bg-transparent border-none outline-none text-xs font-mono text-ciab-text-primary placeholder:text-ciab-text-muted/40"
              autoFocus
            />
            {filterText && (
              <button
                onClick={() => setFilterText("")}
                className="text-ciab-text-muted hover:text-ciab-text-secondary"
              >
                <X className="w-3 h-3" />
              </button>
            )}
          </div>

          {/* Type filter pills */}
          <div className="flex items-center gap-1">
            {Object.entries(EVENT_TYPE_CONFIG).map(([type, config]) => (
              <button
                key={type}
                onClick={() => toggleFilter(type)}
                className={`px-2 py-1 rounded-md text-[9px] font-mono transition-colors ${
                  activeFilters.size === 0 || activeFilters.has(type)
                    ? `${config.color} bg-ciab-bg-elevated`
                    : "text-ciab-text-muted/40 bg-ciab-bg-elevated/50"
                }`}
              >
                {config.label}
              </button>
            ))}
          </div>
        </div>
      )}

      {/* Log output */}
      <div
        ref={scrollRef}
        className="flex-1 bg-ciab-bg-primary rounded-xl border border-ciab-border overflow-y-auto font-mono text-[11px] leading-[1.7] min-h-0"
      >
        <div className="p-3">
          {logEvents.map((event) => (
            <LogLine key={event.id} event={event} />
          ))}
        </div>
      </div>

      {/* Auto-scroll indicator */}
      {!autoScroll && (
        <button
          onClick={() => {
            setAutoScroll(true);
            scrollRef.current?.scrollTo({
              top: scrollRef.current.scrollHeight,
              behavior: "smooth",
            });
          }}
          className="mt-1.5 flex items-center justify-center gap-1.5 text-[10px] font-mono text-ciab-copper
            hover:text-ciab-copper-light transition-colors"
        >
          <span className="w-1.5 h-1.5 rounded-full bg-ciab-copper animate-pulse" />
          Auto-scroll paused — click to resume
        </button>
      )}
    </div>
  );
}

// Parse a log_line's `line` field which may be a JSON string
function parseLogLine(line: string): Record<string, unknown> | null {
  try {
    const parsed = JSON.parse(line);
    if (parsed && typeof parsed === "object") return parsed as Record<string, unknown>;
  } catch { /* not JSON */ }
  return null;
}

function getLogText(event: StreamEvent): string {
  const data = event.data as Record<string, unknown>;
  switch (event.event_type) {
    case "log_line": {
      const line = data.line as string ?? "";
      const parsed = parseLogLine(line);
      if (parsed?.type === "system" && parsed?.subtype === "init") {
        return `Agent initialized · model: ${parsed.model ?? "?"} · session: ${(parsed.session_id as string)?.slice(0, 8) ?? "?"}`;
      }
      return line || JSON.stringify(data);
    }
    case "provisioning_step":
      return `[${data.step}] ${data.detail ?? ""}`;
    case "provisioning_complete":
      return "Provisioning complete";
    case "provisioning_failed":
      return `Provisioning failed: ${data.error}`;
    case "sandbox_state_changed":
      return `State: ${data.from} \u2192 ${data.to}`;
    case "error":
      return `Error: ${data.message}`;
    default:
      return JSON.stringify(data);
  }
}

// Render a rich system/init block with model, tools, skills
function SystemInitBlock({ parsed }: { parsed: Record<string, unknown> }) {
  const [expanded, setExpanded] = useState(false);
  const model = parsed.model as string | undefined;
  const sessionId = parsed.session_id as string | undefined;
  const tools = (parsed.tools as string[]) ?? [];
  const skills = (parsed.skills as string[]) ?? [];
  const agents = (parsed.agents as string[]) ?? [];
  const cwd = parsed.cwd as string | undefined;
  const permMode = parsed.permissionMode as string | undefined;
  const mcpServers = (parsed.mcp_servers as unknown[]) ?? [];
  const slashCommands = (parsed.slash_commands as string[]) ?? [];

  return (
    <div className="space-y-1.5 py-0.5">
      {/* Summary row */}
      <button
        onClick={() => setExpanded(e => !e)}
        className="flex items-center gap-2 hover:text-ciab-text-primary transition-colors text-left w-full"
      >
        {expanded ? <ChevronDown className="w-3 h-3 flex-shrink-0" /> : <ChevronRight className="w-3 h-3 flex-shrink-0" />}
        <span className="text-emerald-400 font-semibold">Agent initialized</span>
        {model && (
          <span className="flex items-center gap-1 text-ciab-text-muted">
            <Cpu className="w-3 h-3" />
            <span className="font-mono">{model}</span>
          </span>
        )}
        {sessionId && (
          <span className="text-ciab-text-muted/50 font-mono text-[9px]">
            {sessionId.slice(0, 8)}
          </span>
        )}
        {permMode && (
          <span className="text-[9px] font-mono px-1.5 py-0.5 rounded bg-ciab-bg-elevated border border-ciab-border/50 text-ciab-text-muted">
            {permMode}
          </span>
        )}
      </button>

      {expanded && (
        <div className="ml-5 space-y-2 text-[10px] animate-fade-in">
          {cwd && (
            <div className="flex items-start gap-2">
              <GitBranch className="w-3 h-3 text-ciab-text-muted/60 mt-0.5 flex-shrink-0" />
              <span className="font-mono text-ciab-text-muted break-all">{cwd}</span>
            </div>
          )}
          {tools.length > 0 && (
            <div className="flex items-start gap-2">
              <Wrench className="w-3 h-3 text-ciab-steel-blue/70 mt-0.5 flex-shrink-0" />
              <div>
                <span className="text-ciab-text-muted uppercase tracking-wider text-[9px] font-semibold">Tools ({tools.length})</span>
                <div className="flex flex-wrap gap-1 mt-1">
                  {tools.map(t => (
                    <span key={t} className="px-1.5 py-0.5 rounded bg-ciab-bg-elevated border border-ciab-border/40 font-mono text-ciab-text-muted/80">{t}</span>
                  ))}
                </div>
              </div>
            </div>
          )}
          {skills.length > 0 && (
            <div className="flex items-start gap-2">
              <Layers className="w-3 h-3 text-ciab-copper/70 mt-0.5 flex-shrink-0" />
              <div>
                <span className="text-ciab-text-muted uppercase tracking-wider text-[9px] font-semibold">Skills ({skills.length})</span>
                <div className="flex flex-wrap gap-1 mt-1">
                  {skills.map(s => (
                    <span key={s} className="px-1.5 py-0.5 rounded bg-ciab-copper/10 border border-ciab-copper/20 font-mono text-ciab-copper/80">{s}</span>
                  ))}
                </div>
              </div>
            </div>
          )}
          {agents.length > 0 && (
            <div className="flex items-start gap-2">
              <Cpu className="w-3 h-3 text-violet-400/70 mt-0.5 flex-shrink-0" />
              <div>
                <span className="text-ciab-text-muted uppercase tracking-wider text-[9px] font-semibold">Sub-agents ({agents.length})</span>
                <div className="flex flex-wrap gap-1 mt-1">
                  {agents.map(a => (
                    <span key={a} className="px-1.5 py-0.5 rounded bg-violet-400/10 border border-violet-400/20 font-mono text-violet-400/80">{a}</span>
                  ))}
                </div>
              </div>
            </div>
          )}
          {mcpServers.length > 0 && (
            <div className="flex items-start gap-2">
              <Wrench className="w-3 h-3 text-amber-400/70 mt-0.5 flex-shrink-0" />
              <div>
                <span className="text-ciab-text-muted uppercase tracking-wider text-[9px] font-semibold">MCP Servers ({mcpServers.length})</span>
                <div className="flex flex-wrap gap-1 mt-1">
                  {mcpServers.map((s, i) => {
                    const name = typeof s === "object" && s !== null ? (s as Record<string, unknown>).name as string : String(s);
                    return (
                      <span key={i} className="px-1.5 py-0.5 rounded bg-amber-400/10 border border-amber-400/20 font-mono text-amber-400/80">{name}</span>
                    );
                  })}
                </div>
              </div>
            </div>
          )}
          {slashCommands.length > 0 && (
            <div className="text-ciab-text-muted/50">
              <span className="uppercase tracking-wider text-[9px] font-semibold">Slash commands: </span>
              <span className="font-mono">{slashCommands.join(", ")}</span>
            </div>
          )}
        </div>
      )}
    </div>
  );
}

// Render a provisioning step with icon + step name + detail
const STEP_ICONS: Record<string, string> = {
  validate: "✓",
  prepare_image: "📦",
  resolve_credentials: "🔑",
  create_sandbox: "🏗",
  start_sandbox: "▶",
  mount_local_dirs: "📁",
  inject_credentials: "💉",
  clone_repositories: "📥",
  setup_agentfs: "🗄",
  run_scripts: "⚙",
  start_agent: "🤖",
};

function ProvisioningStepLine({ data }: { data: Record<string, unknown> }) {
  const step = data.step as string ?? "";
  const detail = data.detail as string ?? "";
  const icon = STEP_ICONS[step] ?? "→";
  return (
    <span className="flex items-center gap-2">
      <span className="text-[11px]">{icon}</span>
      <span className="text-ciab-steel-blue font-semibold">{step}</span>
      {detail && <span className="text-ciab-text-muted">{detail}</span>}
    </span>
  );
}

function LogLine({ event }: { event: StreamEvent }) {
  const time = new Date(event.timestamp).toLocaleTimeString("en-US", {
    hour12: false,
    hour: "2-digit",
    minute: "2-digit",
    second: "2-digit",
  });

  const config = EVENT_TYPE_CONFIG[event.event_type];
  const data = event.data as Record<string, unknown>;
  const isStderr = event.event_type === "log_line" && (data.stream as string) === "stderr";

  // Detect rich log_line types
  let richContent: React.ReactNode = null;
  if (event.event_type === "log_line") {
    const line = data.line as string ?? "";
    const parsed = parseLogLine(line);
    if (parsed?.type === "system" && parsed?.subtype === "init") {
      richContent = <SystemInitBlock parsed={parsed} />;
    } else if (typeof line === "string" && line.startsWith("[ciab]")) {
      const isWarning = line.toLowerCase().includes("warning") || line.toLowerCase().includes("not reachable");
      richContent = (
        <span className={`flex items-center gap-1.5 ${isWarning ? "text-amber-400/90" : "text-ciab-steel-blue/90"}`}>
          <span className="text-[9px] font-mono font-semibold px-1 py-0.5 rounded border border-current/30 bg-current/10 opacity-70 flex-shrink-0">CIAB</span>
          <span>{line.replace(/^\[ciab\]\s*/, "")}</span>
        </span>
      );
    } else if (parsed?.type === "stream_event") {
      // Skip verbose stream_event lines entirely from display (they're internal SSE frames)
      return null;
    } else if (parsed?.type === "rate_limit_event") {
      const info = parsed.rate_limit_info as Record<string, unknown> | undefined;
      richContent = (
        <span className="text-amber-400/80">
          Rate limit: {info?.status as string ?? "warning"}
          {info?.resetsAt ? ` · resets ${new Date(info.resetsAt as string).toLocaleTimeString()}` : ""}
        </span>
      );
    } else if (parsed?.type === "assistant" || parsed?.type === "user") {
      // Skip raw message objects — they appear in the chat view
      return null;
    }
  } else if (event.event_type === "provisioning_step") {
    richContent = <ProvisioningStepLine data={data} />;
  }

  const textContent = richContent ? null : (
    <span
      className={`flex-1 min-w-0 break-words ${
        isStderr
          ? "text-state-failed/80"
          : event.event_type === "error"
            ? "text-state-failed"
            : event.event_type === "provisioning_complete"
              ? "text-state-running"
              : event.event_type === "sandbox_state_changed"
                ? "text-state-paused"
                : "text-ciab-text-secondary"
      }`}
    >
      {getLogText(event)}
    </span>
  );

  return (
    <div className="flex gap-3 hover:bg-ciab-bg-hover/20 rounded px-1.5 -mx-1.5 group">
      <span className="text-ciab-text-muted/30 flex-shrink-0 select-none tabular-nums mt-0.5">
        {time}
      </span>
      {config && (
        <span
          className={`flex-shrink-0 w-[72px] text-right text-[9px] font-semibold uppercase tracking-wider ${config.color}/60 mt-0.5`}
        >
          {config.label}
        </span>
      )}
      <div className="flex-1 min-w-0">
        {richContent ?? textContent}
      </div>
    </div>
  );
}
