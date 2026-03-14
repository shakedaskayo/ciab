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

function getLogText(event: StreamEvent): string {
  const data = event.data as Record<string, unknown>;
  switch (event.event_type) {
    case "log_line":
      return (data.line as string) ?? JSON.stringify(data);
    case "provisioning_step":
      return `[Step ${data.step}] ${data.status}: ${data.message ?? ""}`;
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

function LogLine({ event }: { event: StreamEvent }) {
  const time = new Date(event.timestamp).toLocaleTimeString("en-US", {
    hour12: false,
    hour: "2-digit",
    minute: "2-digit",
    second: "2-digit",
  });

  const config = EVENT_TYPE_CONFIG[event.event_type];
  const data = event.data as Record<string, unknown>;
  const text = getLogText(event);
  const isStderr =
    event.event_type === "log_line" && (data.stream as string) === "stderr";

  return (
    <div className="flex gap-3 hover:bg-ciab-bg-hover/20 rounded px-1.5 -mx-1.5 group">
      <span className="text-ciab-text-muted/30 flex-shrink-0 select-none tabular-nums">
        {time}
      </span>
      {config && (
        <span
          className={`flex-shrink-0 w-[72px] text-right text-[9px] font-semibold uppercase tracking-wider ${config.color}/60`}
        >
          {config.label}
        </span>
      )}
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
                  : event.event_type === "provisioning_step"
                    ? "text-ciab-steel-blue"
                    : "text-ciab-text-secondary"
        }`}
      >
        {text}
      </span>
    </div>
  );
}
