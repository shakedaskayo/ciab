import type { SandboxState } from "@/lib/api/types";

interface Props {
  state: SandboxState;
  size?: "sm" | "md";
}

const STATE_STYLES: Record<string, { bg: string; text: string; dot: string }> = {
  running: { bg: "bg-state-running/10", text: "text-state-running", dot: "bg-state-running" },
  paused: { bg: "bg-state-paused/10", text: "text-state-paused", dot: "bg-state-paused" },
  pausing: { bg: "bg-state-paused/10", text: "text-state-paused", dot: "bg-state-paused" },
  stopped: { bg: "bg-ciab-bg-elevated", text: "text-ciab-text-muted", dot: "bg-ciab-text-muted" },
  stopping: { bg: "bg-ciab-bg-elevated", text: "text-ciab-text-muted", dot: "bg-ciab-text-muted" },
  failed: { bg: "bg-state-failed/10", text: "text-state-failed", dot: "bg-state-failed" },
  creating: { bg: "bg-state-creating/10", text: "text-state-creating", dot: "bg-state-creating" },
  pending: { bg: "bg-state-pending/10", text: "text-state-pending", dot: "bg-state-pending" },
};

export default function SandboxStateBadge({ state, size = "sm" }: Props) {
  const style = STATE_STYLES[state] ?? STATE_STYLES.stopped;
  const isAnimating = state === "creating" || state === "pending" || state === "running";

  return (
    <span
      className={`inline-flex items-center gap-1.5 rounded font-mono text-[10px] uppercase tracking-wider ${style.bg} ${style.text} ${
        size === "md" ? "px-2.5 py-1" : "px-2 py-0.5"
      }`}
    >
      <span
        className={`w-1.5 h-1.5 rounded-full ${style.dot} ${
          isAnimating ? "animate-pulse-slow" : ""
        }`}
      />
      {state}
    </span>
  );
}
