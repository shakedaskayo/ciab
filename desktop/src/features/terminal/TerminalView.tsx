import { useState, useRef, useEffect, useCallback } from "react";
import { useExec } from "@/lib/hooks/use-exec";
import {
  Terminal,
  ArrowUp,
  Trash2,
  Copy,
  Check,
  ChevronUp,
  ChevronDown,
  Clock,
  AlertCircle,
  CheckCircle2,
} from "lucide-react";

interface Props {
  sandboxId: string;
}

interface OutputEntry {
  type: "command" | "stdout" | "stderr" | "info" | "exit";
  text: string;
  exitCode?: number;
  durationMs?: number;
  timestamp: Date;
}

export default function TerminalView({ sandboxId }: Props) {
  const [command, setCommand] = useState("");
  const [workdir, setWorkdir] = useState("/workspace");
  const [history, setHistory] = useState<string[]>([]);
  const [historyIndex, setHistoryIndex] = useState(-1);
  const [output, setOutput] = useState<OutputEntry[]>([
    {
      type: "info",
      text: "Terminal ready. Type a command and press Enter.",
      timestamp: new Date(),
    },
  ]);
  const [copiedIndex, setCopiedIndex] = useState<number | null>(null);
  const execMutation = useExec(sandboxId);
  const scrollRef = useRef<HTMLDivElement>(null);
  const inputRef = useRef<HTMLInputElement>(null);

  // Auto-scroll
  useEffect(() => {
    scrollRef.current?.scrollTo({
      top: scrollRef.current.scrollHeight,
      behavior: "smooth",
    });
  }, [output]);

  // Focus input on mount
  useEffect(() => {
    inputRef.current?.focus();
  }, []);

  const handleExecute = useCallback(() => {
    if (!command.trim() || execMutation.isPending) return;

    const cmd = command.trim();
    setHistory((prev) => [cmd, ...prev.slice(0, 99)]);
    setHistoryIndex(-1);
    setOutput((prev) => [
      ...prev,
      { type: "command", text: cmd, timestamp: new Date() },
    ]);
    setCommand("");

    execMutation.mutate(
      {
        command: ["sh", "-c", cmd],
        workdir,
        timeout_secs: 60,
      },
      {
        onSuccess: (result) => {
          const entries: OutputEntry[] = [];
          if (result.stdout) {
            entries.push({
              type: "stdout",
              text: result.stdout,
              timestamp: new Date(),
            });
          }
          if (result.stderr) {
            entries.push({
              type: "stderr",
              text: result.stderr,
              timestamp: new Date(),
            });
          }
          entries.push({
            type: "exit",
            text: "",
            exitCode: result.exit_code,
            durationMs: result.duration_ms,
            timestamp: new Date(),
          });
          setOutput((prev) => [...prev, ...entries]);
        },
        onError: (error) => {
          setOutput((prev) => [
            ...prev,
            {
              type: "stderr",
              text: `Error: ${error.message}`,
              timestamp: new Date(),
            },
          ]);
        },
      }
    );
  }, [command, workdir, execMutation]);

  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent) => {
      if (e.key === "Enter") {
        e.preventDefault();
        handleExecute();
      }
      // Command history navigation
      if (e.key === "ArrowUp") {
        e.preventDefault();
        if (history.length > 0) {
          const newIndex = Math.min(historyIndex + 1, history.length - 1);
          setHistoryIndex(newIndex);
          setCommand(history[newIndex]);
        }
      }
      if (e.key === "ArrowDown") {
        e.preventDefault();
        if (historyIndex > 0) {
          const newIndex = historyIndex - 1;
          setHistoryIndex(newIndex);
          setCommand(history[newIndex]);
        } else {
          setHistoryIndex(-1);
          setCommand("");
        }
      }
      // Ctrl+L to clear
      if (e.key === "l" && (e.ctrlKey || e.metaKey)) {
        e.preventDefault();
        setOutput([]);
      }
      // Ctrl+C to cancel
      if (e.key === "c" && e.ctrlKey && !command) {
        setOutput((prev) => [
          ...prev,
          { type: "info", text: "^C", timestamp: new Date() },
        ]);
      }
    },
    [handleExecute, history, historyIndex, command]
  );

  const handleCopyOutput = useCallback(
    (index: number) => {
      const entry = output[index];
      navigator.clipboard.writeText(entry.text).then(() => {
        setCopiedIndex(index);
        setTimeout(() => setCopiedIndex(null), 2000);
      });
    },
    [output]
  );

  const handleClear = useCallback(() => {
    setOutput([]);
    inputRef.current?.focus();
  }, []);

  // Global focus shortcut
  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key === "`") {
        e.preventDefault();
        inputRef.current?.focus();
      }
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, []);

  return (
    <div
      className="flex flex-col h-full"
      onClick={() => inputRef.current?.focus()}
    >
      {/* Toolbar */}
      <div className="flex items-center justify-between mb-2 flex-shrink-0">
        <div className="flex items-center gap-2">
          <Terminal className="w-3.5 h-3.5 text-ciab-text-muted" />
          <div className="flex items-center gap-1">
            <span className="text-[10px] font-mono text-ciab-text-muted uppercase tracking-wider">
              cwd:
            </span>
            <input
              type="text"
              value={workdir}
              onChange={(e) => setWorkdir(e.target.value)}
              className="bg-transparent border-none outline-none text-xs font-mono text-ciab-text-secondary
                px-1.5 py-0.5 rounded hover:bg-ciab-bg-hover focus:bg-ciab-bg-hover transition-colors w-48"
            />
          </div>
        </div>

        <div className="flex items-center gap-1">
          <span className="text-[9px] font-mono text-ciab-text-muted/50 mr-2">
            {history.length} commands
          </span>
          <button
            onClick={handleClear}
            className="p-1.5 rounded-lg text-ciab-text-muted hover:text-ciab-text-secondary hover:bg-ciab-bg-hover transition-colors"
            title="Clear (Ctrl+L)"
          >
            <Trash2 className="w-3.5 h-3.5" />
          </button>
        </div>
      </div>

      {/* Output area */}
      <div
        ref={scrollRef}
        className="flex-1 bg-ciab-bg-primary rounded-xl border border-ciab-border overflow-y-auto font-mono text-[12px] leading-[1.6] min-h-0"
      >
        <div className="p-4 space-y-0.5">
          {output.map((entry, i) => (
            <div
              key={i}
              className={`group relative ${
                entry.type === "command" ? "mt-3 first:mt-0" : ""
              }`}
            >
              {entry.type === "command" && (
                <div className="flex items-center gap-2">
                  <span className="text-ciab-copper select-none">$</span>
                  <span className="text-ciab-text-primary font-medium">
                    {entry.text}
                  </span>
                  <span className="text-ciab-text-muted/30 text-[9px] ml-auto">
                    {entry.timestamp.toLocaleTimeString("en-US", {
                      hour12: false,
                      hour: "2-digit",
                      minute: "2-digit",
                      second: "2-digit",
                    })}
                  </span>
                </div>
              )}

              {entry.type === "stdout" && (
                <div className="relative group/output pl-4">
                  <pre className="whitespace-pre-wrap text-ciab-text-secondary">
                    {entry.text}
                  </pre>
                  <button
                    onClick={() => handleCopyOutput(i)}
                    className="absolute top-0 right-0 p-1 rounded opacity-0 group-hover/output:opacity-100 transition-opacity
                      text-ciab-text-muted hover:text-ciab-text-secondary hover:bg-ciab-bg-hover"
                  >
                    {copiedIndex === i ? (
                      <Check className="w-3 h-3 text-state-running" />
                    ) : (
                      <Copy className="w-3 h-3" />
                    )}
                  </button>
                </div>
              )}

              {entry.type === "stderr" && (
                <div className="pl-4">
                  <pre className="whitespace-pre-wrap text-state-failed/80">
                    {entry.text}
                  </pre>
                </div>
              )}

              {entry.type === "exit" && (
                <div className="flex items-center gap-2 pl-4 mt-0.5">
                  {entry.exitCode === 0 ? (
                    <CheckCircle2 className="w-3 h-3 text-state-running/60" />
                  ) : (
                    <AlertCircle className="w-3 h-3 text-state-failed/60" />
                  )}
                  <span
                    className={`text-[10px] ${
                      entry.exitCode === 0
                        ? "text-state-running/50"
                        : "text-state-failed/50"
                    }`}
                  >
                    exit {entry.exitCode}
                  </span>
                  {entry.durationMs != null && (
                    <span className="flex items-center gap-0.5 text-[10px] text-ciab-text-muted/40">
                      <Clock className="w-2.5 h-2.5" />
                      {entry.durationMs}ms
                    </span>
                  )}
                </div>
              )}

              {entry.type === "info" && (
                <div className="text-ciab-text-muted/60 italic pl-4">
                  {entry.text}
                </div>
              )}
            </div>
          ))}

          {execMutation.isPending && (
            <div className="flex items-center gap-2 pl-4 text-ciab-copper/70">
              <div className="flex items-center gap-1">
                <div
                  className="w-1.5 h-1.5 rounded-full bg-ciab-copper/60 animate-bounce"
                  style={{ animationDelay: "0ms", animationDuration: "1s" }}
                />
                <div
                  className="w-1.5 h-1.5 rounded-full bg-ciab-copper/60 animate-bounce"
                  style={{ animationDelay: "150ms", animationDuration: "1s" }}
                />
                <div
                  className="w-1.5 h-1.5 rounded-full bg-ciab-copper/60 animate-bounce"
                  style={{ animationDelay: "300ms", animationDuration: "1s" }}
                />
              </div>
              <span className="text-[10px] font-mono">Running...</span>
            </div>
          )}
        </div>
      </div>

      {/* Command input */}
      <div className="flex items-center gap-2 mt-2 flex-shrink-0">
        <div className="flex-1 flex items-center gap-2 bg-ciab-bg-secondary border border-ciab-border rounded-xl px-3 py-2
          focus-within:border-ciab-copper/40 focus-within:ring-1 focus-within:ring-ciab-copper/20 transition-all">
          <span className="text-ciab-copper font-mono text-sm font-bold select-none">$</span>
          <input
            ref={inputRef}
            type="text"
            value={command}
            onChange={(e) => {
              setCommand(e.target.value);
              setHistoryIndex(-1);
            }}
            onKeyDown={handleKeyDown}
            placeholder="Enter command..."
            className="flex-1 bg-transparent border-none outline-none font-mono text-sm text-ciab-text-primary placeholder:text-ciab-text-muted/40"
            disabled={execMutation.isPending}
          />
          {history.length > 0 && (
            <div className="flex items-center gap-0.5 text-ciab-text-muted/40">
              <ChevronUp className="w-3 h-3" />
              <ChevronDown className="w-3 h-3" />
            </div>
          )}
        </div>
        <button
          onClick={handleExecute}
          disabled={!command.trim() || execMutation.isPending}
          className={`w-9 h-9 rounded-xl flex items-center justify-center transition-all
            ${
              command.trim() && !execMutation.isPending
                ? "bg-ciab-copper hover:bg-ciab-copper-dark text-white hover:scale-105 active:scale-95 shadow-lg shadow-ciab-copper/20"
                : "bg-ciab-bg-elevated text-ciab-text-muted cursor-not-allowed"
            }`}
        >
          <ArrowUp className="w-4 h-4" strokeWidth={2.5} />
        </button>
      </div>

      {/* Shortcut hints */}
      <div className="flex items-center gap-3 mt-1.5 px-1 text-[9px] text-ciab-text-muted/40 font-mono flex-shrink-0">
        <span>
          <kbd className="px-1 py-0.5 rounded bg-ciab-bg-elevated text-ciab-text-muted text-[8px]">
            {"\u2191\u2193"}
          </kbd>{" "}
          history
        </span>
        <span>
          <kbd className="px-1 py-0.5 rounded bg-ciab-bg-elevated text-ciab-text-muted text-[8px]">
            Ctrl+L
          </kbd>{" "}
          clear
        </span>
        <span>
          <kbd className="px-1 py-0.5 rounded bg-ciab-bg-elevated text-ciab-text-muted text-[8px]">
            {"\u2318`"}
          </kbd>{" "}
          focus
        </span>
      </div>
    </div>
  );
}
